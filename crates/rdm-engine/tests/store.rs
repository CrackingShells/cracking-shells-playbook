//! Guards of the campaign store: open and validate, mint, revisions, the primitives.
//!
//! A campaign on disk is built by `Disk`, which writes canonical files with the codec, the real
//! producer of node files. The damage a merge leaves is made by what makes it: `git merge` of two
//! branches, a `cp`, an editor's rewrite of one file.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use rdm_engine::codec::encode;
use rdm_engine::ids::{IdRng, MAX_COLLISIONS, MintError, mint_id};
use rdm_engine::store::{Options, Store};
use rdm_schema::error::{Rule, StoreError};
use rdm_schema::ids::*;
use rdm_schema::stored::*;
use serde_json::{Value, json};

// ---------------------------------------------------------------------------------------------
// Building campaigns

const ALPHABET: &str = "0123456789abcdefghjkmnpqrstvwxyz";

/// A deterministic body of `len` characters for `n`.
fn body(mut n: u32, len: usize) -> String {
    let a = ALPHABET.as_bytes();
    let mut s = vec![b'0'; len];
    for i in (0..len).rev() {
        s[i] = a[(n % 32) as usize];
        n /= 32;
    }
    String::from_utf8(s).unwrap()
}

fn nid(n: u32) -> NodeId {
    NodeId::parse(&format!("n_{}", body(n, 8))).unwrap()
}

fn cid(n: u32) -> CriterionId {
    CriterionId::parse(&format!("c_{}", body(n, 6))).unwrap()
}

fn sid(n: u32) -> StepId {
    StepId::parse(&format!("s_{}", body(n, 6))).unwrap()
}

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

fn criterion(n: u32) -> Criterion {
    Criterion {
        id: cid(n),
        text: format!("criterion {n}"),
        met: false,
        extra: Default::default(),
    }
}

/// A directory (`parent` given) or the campaign root (`parent` none).
fn dir_node(n: u32, nm: &str, parent: Option<u32>, order: &str) -> Node {
    Node::Directory(Directory {
        schema_version: "0.1.0".into(),
        id: nid(n),
        kind: if parent.is_some() {
            DirectoryKind::Directory
        } else {
            DirectoryKind::Campaign
        },
        name: name(nm),
        parent_id: parent.map(nid),
        order: parent.map(|_| OrderKey::new(order)),
        title: format!("Directory {nm}"),
        context: vec!["context".into()],
        references: vec![],
        goal: "a goal".into(),
        preconditions: vec![criterion(n * 10 + 1)],
        success_gates: vec![criterion(n * 10 + 2)],
        gotchas: vec![],
        notes: vec![],
        marks: vec![],
        extra: Default::default(),
    })
}

fn leaf_node(n: u32, nm: &str, parent: u32, order: &str) -> Node {
    Node::Leaf(Leaf {
        schema_version: "0.1.0".into(),
        id: nid(n),
        kind: LeafKind::Leaf,
        name: name(nm),
        parent_id: nid(parent),
        order: OrderKey::new(order),
        title: format!("Leaf {nm}"),
        status: Status::Planned,
        status_reason: None,
        goal: "a goal".into(),
        preconditions: vec![criterion(n * 10 + 1)],
        success_gates: vec![criterion(n * 10 + 2)],
        references: vec![Reference {
            label: None,
            title: "design".into(),
            target: Some("__design__/x.md".into()),
            note: None,
            node_id: None,
            extra: Default::default(),
        }],
        steps: vec![Step {
            id: sid(n * 10 + 3),
            title: "step".into(),
            goal: "a goal".into(),
            implementation_logic: vec!["do it".into()],
            deliverables: vec!["a file".into()],
            references: vec![],
            checks: vec![Check {
                text: "check".into(),
                expected: Expected::Pass,
                command: None,
                extra: Default::default(),
            }],
            commit: Commit {
                r#type: "feat".into(),
                scope: None,
                breaking: false,
                summary: "do".into(),
                extra: Default::default(),
            },
            requires: None,
            done: false,
            commit_sha: None,
            done_at: None,
            extra: Default::default(),
        }],
        progress: Progress {
            branch: None,
            commits: None,
            notes: vec![],
            extra: Default::default(),
        },
        history: vec![],
        marks: vec![],
        extra: Default::default(),
    })
}

/// A campaign directory under a temporary one.
struct Disk {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Disk {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("demo");
        std::fs::create_dir_all(root.join("nodes")).unwrap();
        Self { _tmp: tmp, root }
    }

    fn write(&self, node: &Node) {
        std::fs::write(
            self.root.join(format!("nodes/{}.json", node.id())),
            encode(node),
        )
        .unwrap();
    }

    fn write_all(&self, nodes: &[Node]) {
        nodes.iter().for_each(|n| self.write(n));
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn open(&self) -> Result<Store, StoreError> {
        Store::open(&self.root, &name("demo"), Options::default())
    }
}

/// The sound campaign every row of `open_names_invalid_campaigns` starts from:
///
/// ```text
/// demo (1)
///   core (2)         alpha (3), beta (4)
///   spare (5)
/// ```
fn base() -> Vec<Node> {
    vec![
        dir_node(1, "demo", None, ""),
        dir_node(2, "core", Some(1), "V"),
        leaf_node(3, "alpha", 2, "V"),
        leaf_node(4, "beta", 2, "k"),
        dir_node(5, "spare", Some(1), "k"),
    ]
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@example.com",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// What one branch does to the files.
enum Edit {
    Write(Box<Node>),
    Remove(u32),
}

impl Edit {
    fn new(node: Node) -> Self {
        Edit::Write(Box::new(node))
    }
}

/// Two branches from the sound campaign, each with its own edits, merged by git: the damage is
/// what a merge leaves, since git merges files and not the invariants between them.
fn merged(first: Vec<Edit>, second: Vec<Edit>) -> Disk {
    let disk = Disk::new();
    disk.write_all(&base());
    git(&disk.root, &["init", "-q", "-b", "main"]);
    git(&disk.root, &["add", "-A"]);
    git(&disk.root, &["commit", "-q", "-m", "base"]);
    let apply = |branch: &str, edits: Vec<Edit>| {
        git(&disk.root, &["checkout", "-q", "-b", branch, "main"]);
        for e in edits {
            match e {
                Edit::Write(n) => disk.write(&n),
                Edit::Remove(n) => {
                    std::fs::remove_file(disk.path(&format!("nodes/{}.json", nid(n)))).unwrap()
                }
            }
        }
        git(&disk.root, &["add", "-A"]);
        git(&disk.root, &["commit", "-q", "--allow-empty", "-m", branch]);
    };
    apply("one", first);
    apply("two", second);
    git(&disk.root, &["checkout", "-q", "one"]);
    git(&disk.root, &["merge", "-q", "--no-edit", "two"]);
    disk
}

fn file(n: u32) -> String {
    format!("nodes/{}.json", nid(n))
}

fn rules(store: &Store) -> Vec<(String, Rule)> {
    store
        .diagnostics()
        .iter()
        .map(|d| (d.file.clone(), d.rule))
        .collect()
}

fn refused(r: Result<Store, StoreError>) -> (String, Rule) {
    match r {
        Err(StoreError::Invalid { file, rule, .. }) => (file, rule),
        Err(other) => panic!("want Invalid, got {other:?}"),
        Ok(_) => panic!("want Invalid, but the campaign opened"),
    }
}

/// Rewrites one node file the way an editor does: parse, edit, write back pretty.
fn edit_file(disk: &Disk, n: u32, f: impl FnOnce(&mut Value)) {
    let path = disk.path(&file(n));
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    f(&mut v);
    std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
}

fn validated(store: &Store) -> BTreeSet<NodeId> {
    store
        .campaign()
        .nodes()
        .map(|n| n.id().clone())
        .filter(|id| store.campaign().is_validated(id))
        .collect()
}

#[test]
fn open_names_invalid_campaigns() {
    // A sound campaign opens clean and is entirely validated.
    let disk = Disk::new();
    disk.write_all(&base());
    let store = disk.open().unwrap();
    assert_eq!(store.diagnostics(), &[], "a sound campaign reports nothing");
    assert_eq!(validated(&store).len(), 5);
    assert_eq!(
        store.campaign().path(&nid(3)).as_deref(),
        Some("core/alpha")
    );
    assert_eq!(store.campaign().depth(&nid(3)), Some(1));
    assert_eq!(store.campaign().depth(&nid(2)), Some(0));
    assert_eq!(store.campaign().resolve("core/beta"), Some(&nid(4)));

    // A leftover temporary file, an editor's backup and the store's own scratch are not nodes.
    std::fs::write(disk.path("nodes/n_00000099.json.tmp"), "{").unwrap();
    std::fs::write(disk.path("nodes/.n_00000003.json.swp"), "x").unwrap();
    std::fs::write(disk.path("nodes/notes.json"), "not a node").unwrap();
    std::fs::write(
        disk.path("nodes/n_00000003 (copy).json"),
        "a file manager's copy",
    )
    .unwrap();
    std::fs::create_dir_all(disk.path(".rdm/tmp")).unwrap();
    std::fs::write(disk.path(".rdm/tmp/1-0.tmp"), "{").unwrap();
    let store = disk.open().unwrap();
    assert_eq!(
        store.diagnostics(),
        &[],
        "a leftover temp file is ignored, not an error"
    );

    // Per-file faults refuse the open, naming the file and the rule.
    // `cp` of one node file over another name.
    let disk = Disk::new();
    disk.write_all(&base());
    std::fs::copy(disk.path(&file(3)), disk.path(&file(7))).unwrap();
    assert_eq!(refused(disk.open()), (file(7), Rule::FileIdMismatch));

    // An editor duplicates a criterion block, so one id repeats in the file.
    let disk = Disk::new();
    disk.write_all(&base());
    edit_file(&disk, 3, |v| {
        let dup = v["success_gates"][0].clone();
        v["success_gates"].as_array_mut().unwrap().push(dup);
    });
    assert_eq!(refused(disk.open()), (file(3), Rule::SubItemIdUnique));

    // An editor adds `target` beside `node_id`, so a value derivable from an id is stored.
    let disk = Disk::new();
    disk.write_all(&base());
    edit_file(&disk, 3, |v| {
        v["references"][0]["node_id"] = json!(nid(4).to_string());
    });
    assert_eq!(
        refused(disk.open()),
        (file(3), Rule::ReferenceTargetDerived)
    );

    // An editor truncates a file mid-write: the text is not JSON.
    let disk = Disk::new();
    disk.write_all(&base());
    let text = std::fs::read_to_string(disk.path(&file(4))).unwrap();
    std::fs::write(disk.path(&file(4)), &text[..text.len() / 2]).unwrap();
    assert_eq!(refused(disk.open()), (file(4), Rule::Parse));

    // A hand-typed parent id with a letter outside the alphabet.
    let disk = Disk::new();
    disk.write_all(&base());
    edit_file(&disk, 4, |v| v["parent_id"] = json!("n_0000000i"));
    assert_eq!(refused(disk.open()), (file(4), Rule::MalformedId));

    // A file written by a newer binary is refused before anything else of it is read.
    let disk = Disk::new();
    disk.write_all(&base());
    edit_file(&disk, 5, |v| v["schema_version"] = json!("0.1.1"));
    assert!(matches!(disk.open(), Err(StoreError::NewerFormat { file: f, .. }) if f == file(5)));

    // A campaign directory renamed by hand: the root carries another name than the slug. Every
    // node loads, none is validated, and the defect is reported.
    let disk = Disk::new();
    disk.write_all(&base());
    let renamed = Store::open(&disk.root, &name("other"), Options::default()).unwrap();
    assert_eq!(renamed.campaign().nodes().count(), 5);
    assert_eq!(validated(&renamed), BTreeSet::new());
    assert_eq!(renamed.campaign().root(), None);
    assert_eq!(
        rules(&renamed),
        vec![
            ("nodes/".to_owned(), Rule::RootCount),
            (file(1), Rule::RootNameMismatch)
        ]
    );

    // Cross-file damage that a git merge leaves loads, is reported once per defect, and the
    // sound files stay validated.

    // One branch deletes `core` while the other adds a leaf under it: orphans.
    let disk = merged(
        vec![Edit::Remove(2)],
        vec![Edit::new(leaf_node(6, "gamma", 2, "s"))],
    );
    let store = disk.open().unwrap();
    assert_eq!(
        rules(&store),
        vec![
            (file(3), Rule::ParentMissing),
            (file(4), Rule::ParentMissing),
            (file(6), Rule::ParentMissing)
        ]
    );
    assert_eq!(validated(&store), BTreeSet::from([nid(1), nid(5)]));
    assert_eq!(
        store.campaign().nodes().count(),
        5,
        "the orphans are loaded"
    );
    assert_eq!(store.campaign().path(&nid(3)), None);

    // One branch makes a directory out of a leaf's id while the other adds a child under it.
    let disk = merged(
        vec![Edit::new(leaf_node(5, "spare", 1, "k"))],
        vec![Edit::new(leaf_node(6, "gamma", 5, "s"))],
    );
    let store = disk.open().unwrap();
    assert_eq!(rules(&store), vec![(file(6), Rule::ParentNotDirectory)]);
    assert_eq!(
        validated(&store),
        BTreeSet::from([nid(1), nid(2), nid(3), nid(4), nid(5)])
    );

    // Two branches each move one of two directories under the other: a cycle.
    let disk = merged(
        vec![Edit::new(dir_node(2, "core", Some(5), "V"))],
        vec![Edit::new(dir_node(5, "spare", Some(2), "V"))],
    );
    let store = disk.open().unwrap();
    assert_eq!(
        rules(&store),
        vec![(file(2), Rule::Cycle)],
        "one diagnostic for the cycle"
    );
    assert_eq!(
        validated(&store),
        BTreeSet::from([nid(1)]),
        "the cycle and everything below it is unvalidated"
    );
    assert_eq!(store.campaign().nodes().count(), 5);

    // Two branches each add a node of one name under one parent: the files differ, so git sees no
    // conflict, and the colliding siblings and their subtrees are unvalidated.
    let disk = merged(
        vec![Edit::new(leaf_node(6, "gamma", 2, "s"))],
        vec![Edit::new(leaf_node(7, "gamma", 2, "t"))],
    );
    let store = disk.open().unwrap();
    assert_eq!(rules(&store), vec![(file(6), Rule::NameCollision)]);
    assert_eq!(
        validated(&store),
        BTreeSet::from([nid(1), nid(2), nid(3), nid(4), nid(5)])
    );

    // Two branches each add a second campaign root: one root_count for the second.
    let disk = merged(vec![Edit::new(dir_node(8, "demo", None, ""))], vec![]);
    let store = disk.open().unwrap();
    assert_eq!(rules(&store), vec![(file(8), Rule::RootCount)]);
    assert_eq!(
        store.campaign().root(),
        Some(&nid(1)),
        "the first root by id is the root"
    );
    assert!(!store.campaign().is_validated(&nid(8)));
}

// ---------------------------------------------------------------------------------------------
// Minting

/// Returns the scripted bytes in order, then zeros, and counts the bytes drawn.
struct Script {
    bytes: Vec<u8>,
    at: usize,
    draws: usize,
}

impl Script {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            at: 0,
            draws: 0,
        }
    }
}

impl IdRng for Script {
    fn fill(&mut self, buf: &mut [u8]) {
        self.draws += 1;
        for b in buf {
            *b = self.bytes.get(self.at).copied().unwrap_or(0);
            self.at += 1;
        }
    }
}

/// The bytes that spell `id`'s body, each with its high bits set to garbage: only the low five count.
fn spell(id: &NodeId) -> Vec<u8> {
    id.as_str()[2..]
        .chars()
        .enumerate()
        .map(|(i, c)| ALPHABET.find(c).unwrap() as u8 | [0xE0, 0x20, 0x80, 0x40][i % 4])
        .collect()
}

#[test]
fn minting_retries_on_collision() {
    let disk = Disk::new();
    disk.write_all(&base());
    let store = disk.open().unwrap();
    let taken = |id: &NodeId| store.campaign().get(id).is_some();

    // The first draw is an id of the campaign, the second is free: the second is returned.
    let mut bytes = spell(&nid(3));
    bytes.extend(spell(&nid(900)));
    let mut rng = Script::new(bytes);
    let id: NodeId = mint_id(&mut rng, taken).unwrap();
    assert_eq!(id, nid(900), "the id of the campaign is drawn again");
    assert_eq!(rng.draws, 2);

    // Ids the same batch minted are taken too: the caller says so in `taken`.
    let mut minted = vec![nid(900)];
    let mut bytes = spell(&nid(900));
    bytes.extend(spell(&nid(901)));
    let mut rng = Script::new(bytes);
    let id: NodeId = mint_id(&mut rng, |id| taken(id) || minted.contains(id)).unwrap();
    minted.push(id.clone());
    assert_eq!(id, nid(901));

    // A generator that always collides ends, after exactly `MAX_COLLISIONS` draws.
    let mut rng = Script::new(Vec::new());
    let always = |_: &NodeId| true;
    assert_eq!(
        mint_id::<NodeId>(&mut rng, always),
        Err(MintError::Exhausted)
    );
    assert_eq!((rng.draws, MAX_COLLISIONS), (32, 32));

    // Each character takes five bits of a byte: all 256 byte values land on the 32 characters
    // uniformly, 0xFF is the last character and 0x20 the first.
    let mut rng = Script::new(vec![0xFF, 0x20, 0x21, 0xE1, 0x00, 0x1F, 0x5F, 0x9F]);
    let id: NodeId = mint_id(&mut rng, |_| false).unwrap();
    assert_eq!(id.as_str(), "n_z0110zzz");

    // Sub-item ids have six characters and their own prefix.
    let mut rng = Script::new(spell(&nid(77))[..6].to_vec());
    let c: CriterionId = mint_id(&mut rng, |_| false).unwrap();
    assert!(c.as_str().starts_with("c_") && c.as_str().len() == 8);
    let s: StepId = mint_id(&mut Script::new(vec![1; 6]), |_| false).unwrap();
    assert!(s.as_str().starts_with("s_") && s.as_str().len() == 8);
}
