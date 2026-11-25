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
use rdm_schema::error::{Current, Rule, StoreError};
use rdm_schema::ids::*;
use rdm_schema::links::RecordRef;
use rdm_schema::ops::{EditNote, Header, ReplaceOp, Request};
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

    // Two branches each add a campaign root: the two roots are one defect, no tree is picked,
    // and every node is unvalidated until a root is renamed.
    let disk = merged(vec![Edit::new(dir_node(8, "demo", None, ""))], vec![]);
    let store = disk.open().unwrap();
    assert_eq!(rules(&store), vec![(file(1), Rule::RootCount)]);
    assert_eq!(store.campaign().root(), None, "two roots, no root");
    assert_eq!(validated(&store), BTreeSet::new());
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

// ---------------------------------------------------------------------------------------------
// Revisions and replace

fn actor() -> Actor {
    Actor {
        kind: ActorKind::Agent,
        name: Some("tester".into()),
        extra: Default::default(),
    }
}

fn header(note: Option<&str>, op_id: Option<&str>) -> Header {
    Header {
        op_id: op_id.map(|o| OpId::parse(o).unwrap()),
        actor: actor(),
        note: note.map(|d| EditNote {
            description: d.into(),
            source: None,
            correction: false,
        }),
    }
}

fn replace_req(h: Header, expected: Option<&Rev>, node: Node) -> Request<ReplaceOp> {
    Request {
        header: h,
        op: ReplaceOp {
            expected_rev: expected.cloned(),
            node,
        },
    }
}

/// Every file of the campaign directory except the scratch area, by relative path.
fn snapshot(disk: &Disk) -> std::collections::BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, base: &Path, out: &mut std::collections::BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            let rel = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
            if rel == ".rdm" || rel == ".git" {
                continue;
            }
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                out.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    let mut out = Default::default();
    walk(&disk.root, &disk.root, &mut out);
    out
}

fn leaf_of(store: &Store, n: u32) -> Leaf {
    match store.campaign().get(&nid(n)).unwrap() {
        Node::Leaf(l) => l.clone(),
        Node::Directory(_) => panic!("not a leaf"),
    }
}

fn sha16_of_jq(bytes: &[u8]) -> String {
    use std::io::Write;
    let mut jq = Command::new("jq")
        .args(["del(.schema_version)"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    jq.stdin.take().unwrap().write_all(bytes).unwrap();
    let out = jq.wait_with_output().unwrap().stdout;
    let mut sum = Command::new("shasum")
        .args(["-a", "256"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    sum.stdin.take().unwrap().write_all(&out).unwrap();
    let hash = String::from_utf8(sum.wait_with_output().unwrap().stdout).unwrap();
    hash[..16].to_owned()
}

#[test]
fn rev_tracks_value_not_formatting() {
    let disk = Disk::new();
    disk.write_all(&base());
    let original = std::fs::read_to_string(disk.path(&file(3))).unwrap();
    let rev_of = |text: &str| {
        std::fs::write(disk.path(&file(3)), text).unwrap();
        disk.open()
            .unwrap()
            .rev(&nid(3))
            .unwrap()
            .as_str()
            .to_owned()
    };
    let known = rev_of(&original);
    // The answer, computed by jq and shasum from the producer's bytes.
    assert_eq!(
        known,
        sha16_of_jq(original.as_bytes()),
        "the rev is SHA-256 of the file without schema_version"
    );

    let compact =
        serde_json::to_string(&serde_json::from_str::<Value>(&original).unwrap()).unwrap();
    assert_eq!(rev_of(&compact), known, "whitespace does not count");
    assert_eq!(
        rev_of(&original.replace('\n', "\r\n")),
        known,
        "CRLF does not count"
    );
    assert_eq!(
        rev_of(&original.replace("a goal", "a \\u0067oal")),
        known,
        "a \\u escape does not count"
    );

    assert_ne!(
        rev_of(&original.replace("a goal", "another goal")),
        known,
        "a value counts"
    );
    let with_unknown = |v: &str| original.replacen("{\n", &format!("{{\n  \"zz\": {v},\n"), 1);
    let one = rev_of(&with_unknown("100000"));
    assert_ne!(one, known, "an unknown key counts");
    assert_ne!(
        rev_of(&with_unknown("1e5")),
        one,
        "an unknown value is kept as written"
    );
}

#[test]
fn stale_replace_changes_nothing() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut a = disk.open().unwrap();
    let mut b = disk.open().unwrap();
    let old = a.rev(&nid(3)).unwrap().clone();
    let mut changed = leaf_of(&b, 3);
    changed.goal = "b was here".into();
    let won = b
        .replace(replace_req(
            header(Some("b edits"), None),
            Some(&old),
            Node::Leaf(changed),
        ))
        .unwrap();
    let fresh = won.result.rev.clone();
    let before = snapshot(&disk);

    // `a` still holds the old revision in memory; the write is judged against the disk.
    let mut mine = leaf_of(&a, 3);
    mine.goal = "a was here".into();
    let err = a
        .replace(replace_req(
            header(Some("a edits"), None),
            Some(&old),
            Node::Leaf(mine),
        ))
        .unwrap_err();
    match err {
        StoreError::Stale {
            current: Current::Node { id, rev },
        } => {
            assert_eq!((id, rev), (nid(3), fresh));
        }
        other => panic!("want Stale, got {other:?}"),
    }
    assert_eq!(snapshot(&disk), before, "a stale replace writes no file");
    assert_eq!(
        std::fs::read_dir(disk.path(".rdm/intent"))
            .map(|d| d.count())
            .unwrap_or(0),
        0
    );
}

#[test]
fn replayed_op_id_writes_nothing() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    let old = store.rev(&nid(3)).unwrap().clone();
    let mut changed = leaf_of(&store, 3);
    changed.goal = "retry me".into();
    let req = replace_req(
        header(Some("edit"), Some("op-1")),
        Some(&old),
        Node::Leaf(changed),
    );
    let first = store.replace(req.clone()).unwrap();
    assert!(!first.replayed && first.amendment.is_some());
    let after_first = snapshot(&disk);

    // The same request again: its `expected_rev` is now stale, and it must not matter.
    let again = store.replace(req.clone()).unwrap();
    assert!(again.replayed);
    assert_eq!(
        (again.result.clone(), again.amendment.clone()),
        (first.result.clone(), first.amendment.clone())
    );
    assert_eq!(snapshot(&disk), after_first, "a replay writes nothing");

    // A new process learns the op from the files.
    let mut reopened = disk.open().unwrap();
    let third = reopened.replace(req).unwrap();
    assert!(third.replayed);
    assert_eq!(third.amendment, first.amendment);
    assert_eq!(snapshot(&disk), after_first);
}

#[test]
fn requests_breaking_a_rule_are_refused() {
    let mut nodes = base();
    nodes.push(leaf_node(6, "orphan", 99, "V"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let mut store = disk.open().unwrap();
    let before = snapshot(&disk);
    let rev3 = store.rev(&nid(3)).unwrap().clone();
    let rev2 = store.rev(&nid(2)).unwrap().clone();
    let rev6 = store.rev(&nid(6)).unwrap().clone();

    let edit = |f: &dyn Fn(&mut Leaf)| {
        let mut l = leaf_of(&store, 3);
        f(&mut l);
        Node::Leaf(l)
    };
    let one_step_less = edit(&|l| l.steps.clear());
    let renamed = edit(&|l| l.name = name("renamed"));
    let reparented = edit(&|l| l.parent_id = nid(5));
    let reordered = edit(&|l| l.order = OrderKey::new("z"));
    let dangling = edit(&|l| {
        l.references.push(Reference {
            label: None,
            title: "t".into(),
            target: None,
            note: None,
            node_id: Some(nid(77)),
            extra: Default::default(),
        })
    });
    let edited = edit(&|l| l.goal = "edited".into());
    let mut as_leaf = leaf_of(&store, 3);
    as_leaf.id = nid(2);
    as_leaf.parent_id = nid(1);
    as_leaf.name = name("core");
    as_leaf.order = OrderKey::new("V");
    let mut orphan = leaf_of(&store, 3);
    orphan.id = nid(6);
    orphan.name = name("orphan");
    orphan.parent_id = nid(99);
    orphan.goal = "x".into();
    let mut missing = leaf_of(&store, 3);
    missing.id = nid(42);

    let note = Some("why");
    let rows: Vec<(&str, Request<ReplaceOp>, String, Rule)> = vec![
        (
            "no expected_rev",
            replace_req(header(note, None), None, edited.clone()),
            "<request>".to_owned(),
            Rule::ExpectedRevRequired,
        ),
        (
            "a content edit without a note",
            replace_req(header(None, None), Some(&rev3), edited.clone()),
            "<request>".to_owned(),
            Rule::NoteRequired,
        ),
        (
            "a blank note",
            replace_req(header(Some("  "), None), Some(&rev3), edited.clone()),
            "<request>".to_owned(),
            Rule::NoteRequired,
        ),
        (
            "a note with nothing changed",
            replace_req(
                header(note, None),
                Some(&rev3),
                Node::Leaf(leaf_of(&store, 3)),
            ),
            "<request>".to_owned(),
            Rule::EmptyEdit,
        ),
        (
            "a new name",
            replace_req(header(note, None), Some(&rev3), renamed),
            file(3),
            Rule::UseRelocate,
        ),
        (
            "a new parent",
            replace_req(header(note, None), Some(&rev3), reparented),
            file(3),
            Rule::UseRelocate,
        ),
        (
            "a new order",
            replace_req(header(note, None), Some(&rev3), reordered),
            file(3),
            Rule::UseRelocate,
        ),
        (
            "no steps",
            replace_req(header(note, None), Some(&rev3), one_step_less),
            file(3),
            Rule::StepsRange,
        ),
        (
            "a reference to a missing node",
            replace_req(header(note, None), Some(&rev3), dangling),
            file(3),
            Rule::DanglingId,
        ),
        (
            "a directory with children made a leaf",
            replace_req(header(note, None), Some(&rev2), Node::Leaf(as_leaf)),
            file(2),
            Rule::LeafHasChildren,
        ),
        (
            "a node that is not validated",
            replace_req(header(note, None), Some(&rev6), Node::Leaf(orphan)),
            file(6),
            Rule::UnvalidatedNode,
        ),
    ];
    for (row, req, want_file, want_rule) in rows {
        match store.replace(req) {
            Err(StoreError::Invalid { file, rule, .. }) => {
                assert_eq!((file, rule), (want_file, want_rule), "{row}");
            }
            other => panic!("{row}: want Invalid, got {other:?}"),
        }
        assert_eq!(snapshot(&disk), before, "{row}: nothing was written");
    }
    match store.replace(replace_req(
        header(note, None),
        Some(&rev3),
        Node::Leaf(missing),
    )) {
        Err(StoreError::NotFound { .. }) => {}
        other => panic!("a missing node: want NotFound, got {other:?}"),
    }

    // The accepted side: a state-only edit needs no note and leaves no amendment; a content edit
    // with a note writes exactly one.
    let mut done = leaf_of(&store, 3);
    done.status = Status::Ongoing;
    done.steps[0].done = true;
    done.steps[0].commit_sha = Some("abc1234".into());
    done.steps[0].done_at = Some("2026-01-01T00:00:00Z".into());
    let state_only = store
        .replace(replace_req(
            header(None, None),
            Some(&rev3),
            Node::Leaf(done),
        ))
        .unwrap();
    assert!(
        state_only.amendment.is_none(),
        "a state change is not an amendment"
    );
    assert!(leaf_of(&store, 3).steps[0].done);
    let rev3 = state_only.result.rev;
    let content = store
        .replace(replace_req(header(note, None), Some(&rev3), edited))
        .unwrap();
    assert!(content.amendment.is_some());
    assert_eq!(store.amendments().len(), 1);
    assert_eq!(disk.open().unwrap().amendments().len(), 1);
    structure_rows();
}

// ---------------------------------------------------------------------------------------------
// Order keys

mod order_keys {
    use proptest::prelude::*;
    use rdm_engine::order::{KeyError, between, spread};

    const ALPHA: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    fn sound(k: &str) -> bool {
        !k.is_empty() && !k.ends_with('0') && k.chars().all(|c| ALPHA.contains(c))
    }

    /// Any well-formed key: digits, the last of them not `0`.
    fn key() -> impl Strategy<Value = String> {
        ("[0-9A-Za-z]{0,6}", "[1-9A-Za-z]").prop_map(|(head, last)| format!("{head}{last}"))
    }

    proptest! {
        #[test]
        fn order_keys_sort_strictly_between_their_bounds(a in key(), b in key()) {
            prop_assume!(a != b);
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            for (l, h) in [(Some(lo.as_str()), Some(hi.as_str())), (None, Some(lo.as_str())), (Some(hi.as_str()), None)] {
                let k = between(l, h).unwrap();
                prop_assert!(sound(&k), "{k:?} is a key");
                prop_assert!(l.is_none_or(|l| l < k.as_str()), "{k:?} is above {l:?}");
                prop_assert!(h.is_none_or(|h| k.as_str() < h), "{k:?} is below {h:?}");
            }
        }

        #[test]
        fn keys_stay_short_under_repeated_inserts(spot in 0usize..3) {
            // Appends at the end, prepends at the start, and inserts at one spot: 200 each.
            let mut lo: Option<String> = None;
            let mut hi: Option<String> = None;
            let mut longest = 0;
            for _ in 0..200 {
                let k = between(lo.as_deref(), hi.as_deref()).unwrap();
                longest = longest.max(k.len());
                match spot {
                    0 => lo = Some(k),
                    1 => hi = Some(k),
                    _ => { if lo.is_none() { lo = Some(k) } else { hi = Some(k) } }
                }
            }
            // The document measures 34 for appends and 41 for one spot; the bound is the rule.
            prop_assert!(longest <= 60, "{longest}");
        }

        #[test]
        fn spread_is_ascending_and_short(n in 1usize..1500) {
            let keys = spread(n);
            prop_assert_eq!(keys.len(), n);
            prop_assert!(keys.windows(2).all(|w| w[0] < w[1]));
            prop_assert!(keys.iter().all(|k| sound(k)));
            prop_assert!(keys.iter().all(|k| k.len() <= 3), "a count of {n} needs at most 3 characters");
        }
    }

    #[test]
    fn the_table_of_the_document() {
        let rows: [(Option<&str>, Option<&str>, &str); 6] = [
            (None, None, "V"),
            (Some("V"), None, "k"),
            (None, Some("V"), "F"),
            (Some("V"), Some("o"), "e"),
            (Some("V"), Some("W"), "VV"),
            (Some("V"), Some("VV"), "VF"),
        ];
        for (a, b, want) in rows {
            assert_eq!(between(a, b).unwrap(), want, "between {a:?} and {b:?}");
        }
        assert_eq!(between(Some("V"), Some("V")), Err(KeyError::NotAscending));
        assert_eq!(between(Some("W"), Some("V")), Err(KeyError::NotAscending));
        assert_eq!(between(Some("V0"), None), Err(KeyError::BadKey));
    }
}

// ---------------------------------------------------------------------------------------------
// The other primitives

use rdm_engine::fs::{Effect, RecordingFs};
use rdm_schema::ops::{BatchOp, CommentOp, CreateOp, DeleteOp, Op, Position, RelocateOp};
use std::collections::BTreeMap;
use std::sync::Arc;

fn req<T>(note: Option<&str>, op: T) -> Request<T> {
    Request {
        header: header(note, None),
        op,
    }
}

fn rev_of(store: &Store, n: u32) -> Rev {
    store.rev(&nid(n)).unwrap().clone()
}

fn ids_of(store: &Store) -> BTreeSet<NodeId> {
    store.campaign().nodes().map(|n| n.id().clone()).collect()
}

fn create_op(node: Node, position: Option<Position>) -> CreateOp {
    CreateOp { node, position }
}

fn relocate_op(
    store: &Store,
    n: u32,
    name_to: Option<&str>,
    parent: Option<u32>,
    position: Option<Position>,
) -> RelocateOp {
    RelocateOp {
        id: nid(n),
        expected_rev: Some(rev_of(store, n)),
        parent_id: parent.map(nid),
        name: name_to.map(name),
        position,
    }
}

fn delete_op(store: &Store, n: u32) -> DeleteOp {
    DeleteOp {
        id: nid(n),
        expected_subtree_rev: store.subtree_rev(&nid(n)),
    }
}

fn nodes_changed(
    before: &BTreeMap<String, Vec<u8>>,
    after: &BTreeMap<String, Vec<u8>>,
) -> Vec<String> {
    let mut keys: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    keys.retain(|k| k.starts_with("nodes/") && before.get(*k) != after.get(*k));
    keys.into_iter().cloned().collect()
}

fn new_files(
    before: &BTreeMap<String, Vec<u8>>,
    after: &BTreeMap<String, Vec<u8>>,
    dir: &str,
) -> usize {
    after
        .keys()
        .filter(|k| k.starts_with(dir) && !before.contains_key(*k))
        .count()
}

fn with_reference(node: Node, target: u32) -> Node {
    let Node::Leaf(mut l) = node else {
        panic!("a leaf")
    };
    l.references.push(Reference {
        label: None,
        title: "ref".into(),
        target: None,
        note: None,
        node_id: Some(nid(target)),
        extra: Default::default(),
    });
    Node::Leaf(l)
}

#[test]
fn move_writes_one_file_and_keeps_ids() {
    let mut nodes = base();
    nodes.push(dir_node(10, "inner", Some(2), "t"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let rec = Arc::new(RecordingFs::new(&disk.root));
    let mut store = Store::open(
        &disk.root,
        &name("demo"),
        Options {
            fs: rec.clone(),
            ..Options::default()
        },
    )
    .unwrap();
    let ids = ids_of(&store);

    // (what moves, the request, where it is afterwards)
    type Row = (
        u32,
        Option<&'static str>,
        Option<u32>,
        Option<Position>,
        &'static str,
    );
    let rows: Vec<Row> = vec![
        (3, Some("gamma"), None, None, "core/gamma"),
        (3, None, Some(5), None, "spare/gamma"),
        (2, None, Some(5), Some(Position::First), "spare/core"),
        (
            4,
            None,
            None,
            Some(Position::Before(nid(10))),
            "spare/core/beta",
        ),
    ];
    for (n, new_name, parent, position, path) in rows {
        let before = snapshot(&disk);
        let op = relocate_op(&store, n, new_name, parent, position);
        let skip = rec.effects().len();
        let done = store.move_node(req(Some("tidy"), op)).unwrap();
        let after = snapshot(&disk);
        let touched: Vec<String> = published(&rec.effects()[skip..])
            .into_iter()
            .filter(|p| p.contains("nodes/"))
            .collect();
        assert_eq!(
            touched,
            vec![format!("+{}", file(n))],
            "one node file is written, and it is the moved one"
        );
        assert_eq!(nodes_changed(&before, &after), vec![file(n)]);
        assert_eq!(new_files(&before, &after, "_amendments/"), 1);
        assert!(done.amendment.is_some());
        assert_eq!(ids_of(&store), ids, "no id changes");
        assert_eq!(ids_of(&disk.open().unwrap()), ids);
        assert_eq!(
            store.campaign().path(&nid(n)).as_deref(),
            Some(path),
            "the path follows the moved node"
        );
        assert!(store.campaign().is_validated(&nid(n)));
    }
    // What the amendment records: a rename, then a move.
    let first = store.amendments().values().next().unwrap();
    assert_eq!(
        first
            .changes
            .iter()
            .map(|c| (c.node_id.clone(), c.op, c.old.clone(), c.new.clone()))
            .collect::<Vec<_>>(),
        vec![(
            nid(3),
            ChangeOp::Rename,
            Some(json!("alpha")),
            Some(json!("gamma"))
        )]
    );
}

#[test]
fn tree_rev_changes_on_node_ops_not_comments() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    let mut seen = BTreeSet::from([store.tree_rev()]);

    let mut changed = leaf_of(&store, 3);
    changed.goal = "edited".into();
    type TreeOp = Box<dyn Fn(&mut Store) -> TreeRev>;
    let ops: Vec<(&str, TreeOp)> = vec![
        (
            "create",
            Box::new(|s| {
                let n = leaf_node(20, "newleaf", 2, "V");
                s.create(req(Some("n"), create_op(n, None)))
                    .unwrap()
                    .tree_rev
            }),
        ),
        (
            "replace content",
            Box::new(move |s| {
                let rev = rev_of(s, 3);
                s.replace(replace_req(
                    header(Some("e"), None),
                    Some(&rev),
                    Node::Leaf(changed.clone()),
                ))
                .unwrap()
                .tree_rev
            }),
        ),
        (
            "relocate",
            Box::new(|s| {
                let op = relocate_op(s, 4, Some("beta2"), None, None);
                s.move_node(req(Some("m"), op)).unwrap().tree_rev
            }),
        ),
        (
            "delete",
            Box::new(|s| {
                let op = delete_op(s, 5);
                s.delete_subtree(req(Some("d"), op)).unwrap().tree_rev
            }),
        ),
        (
            "batch",
            Box::new(|s| {
                let n = leaf_node(21, "batched", 2, "V");
                let b = BatchOp {
                    expected_tree_rev: None,
                    ops: vec![Op::Create(create_op(n, None))],
                    dry_run: false,
                };
                s.batch(req(Some("b"), b)).unwrap().tree_rev
            }),
        ),
    ];
    for (row, op) in ops {
        let before = store.tree_rev();
        let after = op(&mut store);
        assert_ne!(after, before, "{row} changes the tree_rev");
        assert!(
            seen.insert(after.clone()),
            "{row}: a tree_rev never comes back"
        );
        assert_eq!(
            store.tree_rev(),
            after,
            "{row}: the receipt names the tree_rev the store holds"
        );
        assert_eq!(
            disk.open().unwrap().tree_rev(),
            after,
            "{row}: a cold open agrees; the amendment is not counted"
        );
    }
    // A comment writes a file and changes no rev.
    let before = store.tree_rev();
    let c = CommentOp {
        node_id: nid(3),
        text: vec!["noted".into()],
    };
    let done = store.comment(req(None, c)).unwrap();
    assert_eq!(done.tree_rev, before);
    assert_eq!(disk.open().unwrap().tree_rev(), before);
    assert_eq!(store.comments().len(), 1);
}

#[test]
fn failed_batch_writes_nothing() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    let before = snapshot(&disk);
    let good = |store: &Store| {
        let mut edited = leaf_of(store, 3);
        edited.goal = "batch edit".into();
        vec![
            Op::Create(create_op(leaf_node(20, "fresh", 2, "V"), None)),
            Op::Replace(ReplaceOp {
                expected_rev: Some(rev_of(store, 3)),
                node: Node::Leaf(edited),
            }),
        ]
    };

    // The third op breaks a rule: the first two are not written.
    let mut ops = good(&store);
    ops.push(Op::Create(create_op(leaf_node(21, "alpha", 2, "V"), None)));
    let err = store
        .batch(req(
            Some("b"),
            BatchOp {
                expected_tree_rev: None,
                ops,
                dry_run: false,
            },
        ))
        .unwrap_err();
    match err {
        StoreError::InBatch { index, source } => {
            assert_eq!(index, 2);
            assert!(
                matches!(
                    *source,
                    StoreError::Invalid {
                        rule: Rule::NameCollision,
                        ..
                    }
                ),
                "{source:?}"
            );
        }
        other => panic!("want InBatch, got {other:?}"),
    }
    assert_eq!(snapshot(&disk), before, "a failed batch writes nothing");
    assert!(
        store.campaign().get(&nid(20)).is_none(),
        "and the store holds nothing of it"
    );

    // A stale tree_rev is Stale and writes nothing.
    let stale = disk.open().unwrap().tree_rev();
    store
        .replace(replace_req(header(None, None), Some(&rev_of(&store, 3)), {
            let mut l = leaf_of(&store, 3);
            l.steps[0].done = false;
            l.status = Status::Planned;
            l.progress.notes = vec!["p".into()];
            Node::Leaf(l)
        }))
        .unwrap();
    let before = snapshot(&disk);
    let err = store
        .batch(req(
            Some("b"),
            BatchOp {
                expected_tree_rev: Some(stale),
                ops: good(&store),
                dry_run: false,
            },
        ))
        .unwrap_err();
    assert!(
        matches!(
            err,
            StoreError::Stale {
                current: Current::Tree { .. }
            }
        ),
        "{err:?}"
    );
    assert_eq!(snapshot(&disk), before);

    // A dry run checks everything and writes nothing; the same batch then writes one amendment.
    let dry = store
        .batch(req(
            Some("b"),
            BatchOp {
                expected_tree_rev: None,
                ops: good(&store),
                dry_run: true,
            },
        ))
        .unwrap();
    assert_eq!(dry.result.len(), 2);
    assert_eq!(snapshot(&disk), before, "a dry run writes nothing");
    let real = store
        .batch(req(
            Some("b"),
            BatchOp {
                expected_tree_rev: None,
                ops: good(&store),
                dry_run: false,
            },
        ))
        .unwrap();
    let after = snapshot(&disk);
    assert_eq!(
        new_files(&before, &after, "_amendments/"),
        1,
        "one amendment for the whole batch"
    );
    assert_eq!(nodes_changed(&before, &after), vec![file(3), file(20)]);
    let amendment = &store.amendments()[real.amendment.as_ref().unwrap()];
    assert_eq!(
        amendment
            .changes
            .iter()
            .filter(|c| c.op == ChangeOp::Add)
            .count(),
        1
    );
    assert!(
        amendment
            .changes
            .iter()
            .any(|c| c.field.as_deref() == Some("/goal"))
    );
    assert_eq!(
        std::fs::read_dir(disk.path(".rdm/intent"))
            .map(|d| d.count())
            .unwrap_or(0),
        0
    );
}

#[test]
fn delete_reports_referrers_and_touches_only_the_subtree() {
    // demo(1) / core(2) / alpha(3), beta(4);  spare(5) / inner(6) / deep(7);  elsewhere(8)
    // alpha references deep (outside the subtree of spare); deep references inner (inside it, so
    // its record goes with it and is not reported); a comment names deep.
    let mut nodes = base();
    nodes.push(dir_node(6, "inner", Some(5), "V"));
    nodes.push(with_reference(leaf_node(7, "deep", 6, "V"), 6));
    nodes[2] = with_reference(nodes[2].clone(), 7);
    let disk = Disk::new();
    disk.write_all(&nodes);
    let mut store = disk.open().unwrap();
    store
        .comment(req(
            None,
            CommentOp {
                node_id: nid(7),
                text: vec!["watch".into()],
            },
        ))
        .unwrap();
    let before = snapshot(&disk);

    let op = delete_op(&store, 5);
    let done = store.delete_subtree(req(Some("drop spare"), op)).unwrap();
    assert_eq!(
        done.result.removed,
        vec![nid(5), nid(6), nid(7)],
        "pre-order"
    );
    let reported: Vec<(RecordRef, String)> = done
        .result
        .referrers
        .iter()
        .map(|r| (r.from.clone(), r.at.clone()))
        .collect();
    assert_eq!(
        reported,
        vec![(RecordRef::Node(nid(3)), "/references/1/node_id".to_owned())],
        "only a Report-class record outside the subtree: not the reference inside it, not the comment"
    );
    let after = snapshot(&disk);
    assert_eq!(
        nodes_changed(&before, &after),
        vec![file(5), file(6), file(7)],
        "only the subtree's files change"
    );
    assert!(
        after
            .keys()
            .all(|k| !k.starts_with("nodes/") || !after.contains_key(&file(5)))
    );
    assert_eq!(new_files(&before, &after, "_amendments/"), 1);
    assert_eq!(
        before.get(&file(3)),
        after.get(&file(3)),
        "the referrer is not rewritten"
    );
    assert_eq!(
        store.comments().len(),
        1,
        "a comment on a removed node stays"
    );
    let amendment = &store.amendments()[done.amendment.as_ref().unwrap()];
    let snapshot_keys: Vec<&String> = amendment.snapshot.as_ref().unwrap().keys().collect();
    assert_eq!(
        snapshot_keys,
        vec![
            &nid(5).to_string(),
            &nid(6).to_string(),
            &nid(7).to_string()
        ]
    );
    assert_eq!(
        amendment
            .changes
            .iter()
            .map(|c| (c.node_id.clone(), c.op))
            .collect::<Vec<_>>(),
        vec![
            (nid(5), ChangeOp::Remove),
            (nid(6), ChangeOp::Remove),
            (nid(7), ChangeOp::Remove)
        ]
    );
    // The dangling reference is now a diagnostic, and the node is gone from every read.
    assert!(rules(&store).contains(&(file(3), Rule::DanglingReference)));
    assert_eq!(ids_of(&store), ids_of(&disk.open().unwrap()));
}

/// The published names an operation leaves, in order: the links and renames into a published
/// name, and the unlinks of one.
fn published(effects: &[Effect]) -> Vec<String> {
    let visible = |p: &Path| !p.starts_with(".rdm/tmp") && !p.starts_with(".rdm/.gitignore");
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Link { link: p, .. } | Effect::Rename { to: p, .. } if visible(p) => {
                Some(format!("+{}", p.display()))
            }
            Effect::Remove { path } if visible(path) => Some(format!("-{}", path.display())),
            _ => None,
        })
        .collect()
}

#[test]
fn write_order_follows_the_three_rules() {
    let mut nodes = base();
    nodes.push(leaf_node(6, "child", 5, "V"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let rec = Arc::new(RecordingFs::new(&disk.root));
    let mut store = Store::open(
        &disk.root,
        &name("demo"),
        Options {
            fs: rec.clone(),
            ..Options::default()
        },
    )
    .unwrap();
    let skip = rec.effects().len();
    let mut edited = leaf_of(&store, 4);
    edited.goal = "edit".into();
    let ops = vec![
        Op::Delete(delete_op(&store, 5)),
        Op::Replace(ReplaceOp {
            expected_rev: Some(rev_of(&store, 4)),
            node: Node::Leaf(edited),
        }),
        Op::Create(create_op(dir_node(20, "made", Some(2), ""), None)),
        Op::Create(create_op(leaf_node(21, "madeleaf", 20, "V"), None)),
    ];
    let done = store
        .batch(req(
            Some("all kinds"),
            BatchOp {
                expected_tree_rev: None,
                ops,
                dry_run: false,
            },
        ))
        .unwrap();
    let log = published(&rec.effects()[skip..]);
    let a = format!("+_amendments/{}.json", done.amendment.unwrap());
    let kinds: Vec<String> = log
        .iter()
        .map(|p| {
            if p.starts_with("+.rdm/intent/") {
                "intent".to_owned()
            } else {
                p.clone()
            }
        })
        .collect();
    let intent_end = log
        .iter()
        .position(|p| p.starts_with("-.rdm/intent/"))
        .unwrap();
    let mut body: Vec<String> = kinds[..intent_end].to_vec();
    assert_eq!(
        body.remove(0),
        "intent",
        "the intent record is published first"
    );
    assert_eq!(
        body,
        vec![
            format!("+nodes/{}.json", nid(20)),
            format!("+nodes/{}.json", nid(21)),
            format!("+nodes/{}.json", nid(4)),
            a,
            format!("-nodes/{}.json", nid(6)),
            format!("-nodes/{}.json", nid(5)),
        ],
        "creates (parents first), replaces, the amendment, deletes (deepest first), then the intent goes"
    );
    assert_eq!(intent_end, log.len() - 1);
}

#[test]
fn rebalance_keeps_order_and_length() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    // Each insert goes first: the key shrinks towards `0` by about one character per six.
    let mut order = Vec::new();
    for i in 0..150u32 {
        let n = 100 + i;
        let node = leaf_node(n, &format!("x{i}"), 5, "V");
        store
            .create(req(Some("grow"), create_op(node, Some(Position::First))))
            .unwrap();
        order.insert(0, nid(n));
        let keys: Vec<String> = store
            .campaign()
            .children(&nid(5))
            .iter()
            .map(|c| {
                store
                    .campaign()
                    .get(c)
                    .unwrap()
                    .order()
                    .unwrap()
                    .as_str()
                    .to_owned()
            })
            .collect();
        assert!(
            keys.iter().all(|k| k.len() <= 25),
            "after {i} inserts a key is {} long",
            keys.iter().map(String::len).max().unwrap()
        );
        assert!(keys.windows(2).all(|w| w[0] <= w[1]));
    }
    assert_eq!(
        store.campaign().children(&nid(5)),
        order.as_slice(),
        "the listed order is the insert order"
    );
    let rebalances = store
        .amendments()
        .values()
        .filter(|a| {
            a.changes
                .iter()
                .any(|c| c.field.as_deref() == Some("/order"))
        })
        .count();
    assert!(
        rebalances >= 1,
        "a rebalance recorded its `set` of `/order`"
    );
    assert_eq!(
        disk.open().unwrap().campaign().children(&nid(5)),
        order.as_slice()
    );
}

// --- the reverse index against a scan -------------------------------------------------------

/// Every string equal to a node id in every record file, with its record and pointer: what the
/// index must hold. Skipped: a record's own `id`, and the historical copies (`old`, `new`, the
/// values of a snapshot); the keys of a snapshot count.
fn scan(disk: &Disk) -> BTreeSet<(String, String, String)> {
    fn walk(
        v: &Value,
        ptr: String,
        record: &str,
        own: bool,
        out: &mut BTreeSet<(String, String, String)>,
    ) {
        match v {
            Value::String(s) => {
                if NodeId::parse(s).is_ok() && !own {
                    out.insert((s.clone(), record.to_owned(), ptr));
                }
            }
            Value::Array(a) => {
                for (i, x) in a.iter().enumerate() {
                    walk(x, format!("{ptr}/{i}"), record, false, out);
                }
            }
            Value::Object(m) => {
                for (k, x) in m {
                    let p = format!("{ptr}/{k}");
                    if record.starts_with("amendment:") {
                        if k == "old" || k == "new" {
                            continue;
                        }
                        if ptr == "/snapshot" {
                            if NodeId::parse(k).is_ok() {
                                out.insert((k.clone(), record.to_owned(), p));
                            }
                            continue;
                        }
                    }
                    walk(x, p, record, ptr.is_empty() && k == "id", out);
                }
            }
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    for (dir, kind) in [
        ("nodes", "node"),
        ("_comments", "comment"),
        ("_amendments", "amendment"),
    ] {
        let Ok(entries) = std::fs::read_dir(disk.path(dir)) else {
            continue;
        };
        for e in entries {
            let p = e.unwrap().path();
            let stem = p.file_stem().unwrap().to_string_lossy().into_owned();
            let v: Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
            walk(
                &v,
                String::new(),
                &format!("{kind}:{stem}"),
                false,
                &mut out,
            );
        }
    }
    out
}

fn held(store: &Store) -> BTreeSet<(String, String, String)> {
    let mut out = BTreeSet::new();
    for (target, referrers) in store.reverse().entries() {
        for r in referrers {
            let from = match &r.from {
                RecordRef::Node(n) => format!("node:{n}"),
                RecordRef::Comment(c) => format!("comment:{c}"),
                RecordRef::Amendment(a) => format!("amendment:{a}"),
            };
            out.insert((target.to_string(), from, r.at.clone()));
        }
    }
    out
}

mod reverse {
    use super::*;
    use proptest::prelude::*;

    fn apply(store: &mut Store, step: (u8, u8, u8), next: &mut u32) {
        let ids: Vec<NodeId> = ids_of(store).into_iter().collect();
        let pick = |x: u8| ids[x as usize % ids.len()].clone();
        let number = |id: &NodeId| (1..400).find(|n| nid(*n) == *id).unwrap_or(0);
        let (kind, a, b) = step;
        let leaves: Vec<NodeId> = ids
            .iter()
            .filter(|i| matches!(store.campaign().get(i), Some(Node::Leaf(_))))
            .cloned()
            .collect();
        let dirs: Vec<NodeId> = ids
            .iter()
            .filter(|i| matches!(store.campaign().get(i), Some(Node::Directory(_))))
            .cloned()
            .collect();
        match kind % 6 {
            0 => {
                *next += 1;
                let parent = dirs[a as usize % dirs.len()].clone();
                let mut node = leaf_node(*next, &format!("x{next}"), number(&parent), "V");
                if b % 2 == 0 {
                    node = with_reference(node, number(&pick(b)));
                }
                let _ = store.create(req(Some("c"), create_op(node, None)));
            }
            1 if !leaves.is_empty() => {
                let id = leaves[a as usize % leaves.len()].clone();
                let node =
                    with_reference(store.campaign().get(&id).unwrap().clone(), number(&pick(b)));
                let rev = store.rev(&id).unwrap().clone();
                let _ = store.replace(replace_req(header(Some("r"), None), Some(&rev), node));
            }
            2 if !leaves.is_empty() => {
                let id = leaves[a as usize % leaves.len()].clone();
                let Node::Leaf(mut l) = store.campaign().get(&id).unwrap().clone() else {
                    return;
                };
                l.references.clear();
                let rev = store.rev(&id).unwrap().clone();
                let _ = store.replace(replace_req(
                    header(Some("r"), None),
                    Some(&rev),
                    Node::Leaf(l),
                ));
            }
            3 => {
                let id = pick(a);
                let parent = dirs[b as usize % dirs.len()].clone();
                let op = RelocateOp {
                    id: id.clone(),
                    expected_rev: store.rev(&id).cloned(),
                    parent_id: Some(parent),
                    name: None,
                    position: None,
                };
                let _ = store.move_node(req(Some("m"), op));
            }
            4 => {
                let id = pick(a);
                let op = DeleteOp {
                    id: id.clone(),
                    expected_subtree_rev: store.subtree_rev(&id),
                };
                let _ = store.delete_subtree(req(Some("d"), op));
            }
            _ => {
                let _ = store.comment(req(
                    None,
                    CommentOp {
                        node_id: pick(a),
                        text: vec!["c".into()],
                    },
                ));
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]
        #[test]
        fn reverse_index_equals_brute_force_scan(steps in prop::collection::vec((any::<u8>(), any::<u8>(), any::<u8>()), 1..14)) {
            let mut nodes = base();
            nodes.push(with_reference(leaf_node(6, "linked", 5, "V"), 3));
            let disk = Disk::new();
            disk.write_all(&nodes);
            let mut store = disk.open().unwrap();
            prop_assert_eq!(held(&store), scan(&disk));
            let mut next = 100;
            for step in steps {
                apply(&mut store, step, &mut next);
                let oracle = scan(&disk);
                prop_assert_eq!(&held(&store), &oracle, "the index kept up with the write");
                prop_assert_eq!(&held(&disk.open().unwrap()), &oracle, "a cold build agrees");
            }
        }
    }
}

type Row<'a> = (
    &'a str,
    Box<dyn Fn(&mut Store) -> Result<(), StoreError> + 'a>,
    String,
    Rule,
);

fn structure_rows() {
    // demo(1) / core(2) { alpha(3), beta(4), inner(10) };  spare(5) { twin(7), twin(8) };
    // orphans: leaf(6) and directory(9) under a parent that does not exist.
    let mut nodes = base();
    nodes.push(leaf_node(6, "orphan", 99, "V"));
    nodes.push(leaf_node(7, "twin", 5, "V"));
    nodes.push(leaf_node(8, "twin", 5, "k"));
    nodes.push(dir_node(9, "lost", Some(99), "V"));
    nodes.push(dir_node(10, "inner", Some(2), "t"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let mut store = disk.open().unwrap();
    let before = snapshot(&disk);
    let note = Some("why");

    let rows: Vec<Row> = vec![
        (
            "create: an id that exists",
            Box::new(move |s| {
                s.create(req(note, create_op(leaf_node(3, "zz", 2, "V"), None)))
                    .map(drop)
            }),
            file(3),
            Rule::DuplicateId,
        ),
        (
            "create: no such parent",
            Box::new(move |s| {
                s.create(req(note, create_op(leaf_node(20, "a20", 77, "V"), None)))
                    .map(drop)
            }),
            file(20),
            Rule::ParentMissing,
        ),
        (
            "create: the parent is a leaf",
            Box::new(move |s| {
                s.create(req(note, create_op(leaf_node(20, "a20", 3, "V"), None)))
                    .map(drop)
            }),
            file(20),
            Rule::ParentNotDirectory,
        ),
        (
            "create: the parent is not validated",
            Box::new(move |s| {
                s.create(req(note, create_op(leaf_node(20, "a20", 9, "V"), None)))
                    .map(drop)
            }),
            file(20),
            Rule::UnvalidatedNode,
        ),
        (
            "create: a sibling has the name",
            Box::new(move |s| {
                s.create(req(note, create_op(leaf_node(20, "alpha", 2, "V"), None)))
                    .map(drop)
            }),
            file(20),
            Rule::NameCollision,
        ),
        (
            "create: a second root",
            Box::new(move |s| {
                s.create(req(note, create_op(dir_node(20, "demo", None, ""), None)))
                    .map(drop)
            }),
            file(20),
            Rule::RootCount,
        ),
        (
            "create: a position that is no sibling",
            Box::new(move |s| {
                s.create(req(
                    note,
                    create_op(leaf_node(20, "a20", 2, "V"), Some(Position::Before(nid(5)))),
                ))
                .map(drop)
            }),
            "<request>".into(),
            Rule::PositionNotSibling,
        ),
        (
            "create: no note",
            Box::new(move |s| {
                s.create(req(None, create_op(leaf_node(20, "a20", 2, "V"), None)))
                    .map(drop)
            }),
            "<request>".into(),
            Rule::NoteRequired,
        ),
        (
            "relocate: nothing to do",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, None, None);
                s.move_node(req(note, op)).map(drop)
            }),
            "<request>".into(),
            Rule::NothingToDo,
        ),
        (
            "relocate: the root gets a parent",
            Box::new(move |s| {
                let op = relocate_op(s, 1, None, Some(5), None);
                s.move_node(req(note, op)).map(drop)
            }),
            file(1),
            Rule::RootDeleteOrMove,
        ),
        (
            "relocate: into its own subtree",
            Box::new(move |s| {
                let op = relocate_op(s, 2, None, Some(10), None);
                s.move_node(req(note, op)).map(drop)
            }),
            file(2),
            Rule::Cycle,
        ),
        (
            "relocate: a name a sibling has",
            Box::new(move |s| {
                let op = relocate_op(s, 4, Some("alpha"), None, None);
                s.move_node(req(note, op)).map(drop)
            }),
            file(4),
            Rule::NameCollision,
        ),
        (
            "relocate: a position that is no sibling",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, None, Some(Position::Before(nid(5))));
                s.move_node(req(note, op)).map(drop)
            }),
            "<request>".into(),
            Rule::PositionNotSibling,
        ),
        (
            "relocate: no such parent",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, Some(77), None);
                s.move_node(req(note, op)).map(drop)
            }),
            file(3),
            Rule::ParentMissing,
        ),
        (
            "relocate: the parent is a leaf",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, Some(4), None);
                s.move_node(req(note, op)).map(drop)
            }),
            file(3),
            Rule::ParentNotDirectory,
        ),
        (
            "relocate: into an unvalidated parent",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, Some(9), None);
                s.move_node(req(note, op)).map(drop)
            }),
            file(3),
            Rule::UnvalidatedNode,
        ),
        (
            "relocate: a place among the siblings of an unvalidated node",
            Box::new(move |s| {
                let op = relocate_op(s, 9, None, None, Some(Position::First));
                s.move_node(req(note, op)).map(drop)
            }),
            file(9),
            Rule::UnvalidatedNode,
        ),
        (
            "relocate: no note",
            Box::new(move |s| {
                let op = relocate_op(s, 3, Some("q"), None, None);
                s.move_node(req(None, op)).map(drop)
            }),
            "<request>".into(),
            Rule::NoteRequired,
        ),
        (
            "relocate: no expected_rev",
            Box::new(move |s| {
                let mut op = relocate_op(s, 3, Some("q"), None, None);
                op.expected_rev = None;
                s.move_node(req(note, op)).map(drop)
            }),
            "<request>".into(),
            Rule::ExpectedRevRequired,
        ),
        (
            "delete: no expected_subtree_rev",
            Box::new(move |s| {
                s.delete_subtree(req(
                    note,
                    DeleteOp {
                        id: nid(5),
                        expected_subtree_rev: None,
                    },
                ))
                .map(drop)
            }),
            "<request>".into(),
            Rule::ExpectedRevRequired,
        ),
        (
            "delete: the root",
            Box::new(move |s| {
                let op = delete_op(s, 1);
                s.delete_subtree(req(note, op)).map(drop)
            }),
            file(1),
            Rule::RootDeleteOrMove,
        ),
        (
            "delete: no note",
            Box::new(move |s| {
                let op = delete_op(s, 5);
                s.delete_subtree(req(None, op)).map(drop)
            }),
            "<request>".into(),
            Rule::NoteRequired,
        ),
        (
            "comment: with a note",
            Box::new(move |s| {
                s.comment(req(
                    note,
                    CommentOp {
                        node_id: nid(3),
                        text: vec!["c".into()],
                    },
                ))
                .map(drop)
            }),
            "<request>".into(),
            Rule::NoteForbidden,
        ),
    ];
    for (row, run, want_file, want_rule) in rows {
        match run(&mut store) {
            Err(StoreError::Invalid { file, rule, .. }) => {
                assert_eq!((file, rule), (want_file, want_rule), "{row}")
            }
            other => panic!("{row}: want Invalid, got {other:?}"),
        }
        assert_eq!(snapshot(&disk), before, "{row}: nothing was written");
    }
    // Stale and missing.
    let wrong = store.subtree_rev(&nid(2));
    match store.delete_subtree(req(
        note,
        DeleteOp {
            id: nid(5),
            expected_subtree_rev: wrong,
        },
    )) {
        Err(StoreError::Stale {
            current: Current::Subtree { id, rev },
        }) => {
            assert_eq!((id, Some(rev)), (nid(5), store.subtree_rev(&nid(5))));
        }
        other => panic!("a stale subtree: want Stale, got {other:?}"),
    }
    assert!(matches!(
        store.delete_subtree(req(
            note,
            DeleteOp {
                id: nid(77),
                expected_subtree_rev: None
            }
        )),
        Err(StoreError::NotFound { .. })
    ));
    assert!(matches!(
        store.comment(req(
            None,
            CommentOp {
                node_id: nid(77),
                text: vec!["c".into()]
            }
        )),
        Err(StoreError::NotFound { .. })
    ));
    assert_eq!(snapshot(&disk), before);

    // A repair is accepted: a comment on the damage, a node moved into the tree, a colliding
    // sibling renamed, an orphan subtree deleted.
    store
        .comment(req(
            None,
            CommentOp {
                node_id: nid(9),
                text: vec!["lost".into()],
            },
        ))
        .unwrap();
    let op = relocate_op(&store, 6, None, Some(2), None);
    store.move_node(req(note, op)).unwrap();
    assert!(store.campaign().is_validated(&nid(6)));
    assert!(!store.campaign().is_validated(&nid(8)), "still colliding");
    let op = relocate_op(&store, 8, Some("twin2"), None, None);
    store.move_node(req(note, op)).unwrap();
    assert!(store.campaign().is_validated(&nid(7)) && store.campaign().is_validated(&nid(8)));
    let op = delete_op(&store, 9);
    store.delete_subtree(req(note, op)).unwrap();
    let damage: Vec<Rule> = rules(&store)
        .into_iter()
        .map(|(_, r)| r)
        .filter(|r| {
            matches!(
                r,
                Rule::ParentMissing
                    | Rule::NameCollision
                    | Rule::Cycle
                    | Rule::RootCount
                    | Rule::RootNameMismatch
            )
        })
        .collect();
    assert_eq!(damage, vec![], "every defect was repaired");
    assert_eq!(rules(&disk.open().unwrap()), rules(&store));

    // The root. A healthy campaign cannot lose its root name, a renamed campaign repairs it.
    let healthy = Disk::new();
    healthy.write_all(&base());
    let mut store = healthy.open().unwrap();
    let before = snapshot(&healthy);
    let op = relocate_op(&store, 1, Some("other"), None, None);
    match store.move_node(req(note, op)) {
        Err(StoreError::Invalid {
            rule: Rule::RootCount | Rule::RootNameMismatch,
            ..
        }) => {}
        other => panic!("renaming a sound root: want a root rule, got {other:?}"),
    }
    assert_eq!(snapshot(&healthy), before);
    // A stray parentless node does not make the sound root renamable: the exemption is for a
    // campaign with no validated root.
    let stray = Disk::new();
    let mut nodes = base();
    nodes.push(dir_node(8, "other", None, ""));
    stray.write_all(&nodes);
    let mut store = stray.open().unwrap();
    assert_eq!(store.campaign().root(), Some(&nid(1)));
    let before = snapshot(&stray);
    let op = relocate_op(&store, 1, Some("zzz"), None, None);
    match store.move_node(req(note, op)) {
        Err(StoreError::Invalid {
            rule: Rule::RootCount | Rule::RootNameMismatch,
            ..
        }) => {}
        other => {
            panic!("renaming a sound root beside a stray root: want a root rule, got {other:?}")
        }
    }
    assert_eq!(snapshot(&stray), before);
    assert_eq!(
        validated(&store).len(),
        5,
        "the sound root is still the root"
    );
    let mut renamed = Store::open(&healthy.root, &name("other"), Options::default()).unwrap();
    assert_eq!(renamed.campaign().root(), None);
    let op = relocate_op(&renamed, 1, Some("other"), None, None);
    renamed.move_node(req(note, op)).unwrap();
    assert_eq!(
        renamed.campaign().root(),
        Some(&nid(1)),
        "the rename repaired the root"
    );
    assert_eq!(validated(&renamed).len(), 5);

    // Two roots after a merge of two inits: every node is unvalidated; renaming or deleting one
    // of them is the repair.
    for (row, rename) in [("rename", true), ("delete", false)] {
        let disk = merged(vec![Edit::new(dir_node(8, "demo", None, ""))], vec![]);
        let mut store = disk.open().unwrap();
        assert_eq!(store.campaign().root(), None);
        if rename {
            let op = relocate_op(&store, 8, Some("second"), None, None);
            store.move_node(req(note, op)).unwrap();
        } else {
            let op = delete_op(&store, 8);
            store.delete_subtree(req(note, op)).unwrap();
        }
        assert_eq!(
            store.campaign().root(),
            Some(&nid(1)),
            "{row}: one root is left"
        );
        assert_eq!(validated(&store).len(), 5, "{row}");
    }

    // A cycle that a merge of two branches leaves: core under spare on one branch, spare under
    // core on the other. Deleting a member deletes the cycle and everything below it.
    let disk = merged(
        vec![Edit::new(dir_node(2, "core", Some(5), "V"))],
        vec![Edit::new(dir_node(5, "spare", Some(2), "k"))],
    );
    let mut store = disk.open().unwrap();
    assert!(rules(&store).iter().any(|(_, r)| *r == Rule::Cycle));
    let op = delete_op(&store, 5);
    let done = store.delete_subtree(req(note, op)).unwrap();
    assert_eq!(done.result.removed, vec![nid(5), nid(2), nid(3), nid(4)]);
    for n in [2, 3, 4, 5] {
        assert!(!disk.path(&file(n)).exists(), "n{n} is gone from the disk");
    }
    assert!(disk.path(&file(1)).exists());
    assert_eq!(
        ids_of(&store),
        BTreeSet::from([nid(1)]),
        "and from the store"
    );
    let amendment = &store.amendments()[done.amendment.as_ref().unwrap()];
    let listed: BTreeSet<NodeId> = amendment
        .changes
        .iter()
        .map(|c| c.node_id.clone())
        .collect();
    assert_eq!(listed, BTreeSet::from([nid(2), nid(3), nid(4), nid(5)]));
    let reopened = disk.open().unwrap();
    assert_eq!(ids_of(&reopened), ids_of(&store));
    assert!(
        rules(&reopened).iter().all(|(_, r)| *r != Rule::Cycle),
        "no cycle is reported"
    );
}

#[test]
fn marks_name_criteria() {
    let marked = |gates_with_extra: bool| {
        let Node::Leaf(mut l) = leaf_node(3, "alpha", 2, "V") else {
            unreachable!()
        };
        let mut extra = criterion(39);
        extra.met = true;
        if gates_with_extra {
            l.success_gates.push(extra);
        }
        l.marks.push(Mark {
            ts: "2026-01-01T00:00:00Z".into(),
            actor: actor(),
            criterion_id: cid(39),
            met: true,
            evidence: None,
            reason: None,
            correction: None,
            extra: Default::default(),
        });
        Node::Leaf(l)
    };
    let mut nodes = base();
    nodes[2] = marked(true);
    let disk = Disk::new();
    disk.write_all(&nodes);
    let mut store = disk.open().unwrap();
    assert_eq!(
        rules(&store),
        vec![],
        "a mark that names a criterion of its file is sound"
    );

    // A request that leaves a new mark naming a missing criterion is refused.
    let mut bad = leaf_of(&store, 3);
    let mut stray = bad.marks[0].clone();
    stray.criterion_id = cid(999);
    bad.marks.push(stray);
    let before = snapshot(&disk);
    let rev = rev_of(&store, 3);
    match store.replace(replace_req(header(None, None), Some(&rev), Node::Leaf(bad))) {
        Err(StoreError::Invalid {
            file: f,
            rule: Rule::MarkTarget,
            ..
        }) => assert_eq!(f, file(3)),
        other => panic!("want mark_target, got {other:?}"),
    }
    assert_eq!(snapshot(&disk), before);

    // A removal through the store leaves the amendment that explains the mark; no diagnostic.
    let mut fewer = leaf_of(&store, 3);
    fewer.success_gates.retain(|c| c.id != cid(39));
    store
        .replace(replace_req(
            header(Some("drop a gate"), None),
            Some(&rev),
            Node::Leaf(fewer),
        ))
        .unwrap();
    assert_eq!(rules(&store), vec![]);
    assert_eq!(rules(&disk.open().unwrap()), vec![]);

    // The same removal by an editor, with no amendment, is reported on open.
    let hand = Disk::new();
    hand.write_all(&nodes);
    edit_file(&hand, 3, |v| {
        v["success_gates"]
            .as_array_mut()
            .unwrap()
            .retain(|c| c["id"] != json!(cid(39).to_string()));
    });
    assert_eq!(
        rules(&hand.open().unwrap()),
        vec![(file(3), Rule::MarkTarget)]
    );
}

// ---------------------------------------------------------------------------------------------
// Recovery

fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let to = dst.join(e.file_name());
        if e.path().is_dir() {
            copy_tree(&e.path(), &to);
        } else {
            std::fs::copy(e.path(), &to).unwrap();
        }
    }
}

fn snapshot_dir(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let disk = Disk {
        _tmp: tempfile::tempdir().unwrap(),
        root: root.to_path_buf(),
    };
    snapshot(&disk)
}

/// An operation that writes several files, run over a recording file system: the state before,
/// the state after and the effects between.
type Snap = BTreeMap<String, Vec<u8>>;
type Recorded = (tempfile::TempDir, Snap, Snap, Vec<Effect>);

fn recorded(run: impl FnOnce(&mut Store)) -> Recorded {
    let mut nodes = base();
    nodes.push(leaf_node(6, "child", 5, "V"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let pre_dir = tempfile::tempdir().unwrap();
    copy_tree(&disk.root, &pre_dir.path().join("demo"));
    let rec = Arc::new(RecordingFs::new(&disk.root));
    let mut store = Store::open(
        &disk.root,
        &name("demo"),
        Options {
            fs: rec.clone(),
            ..Options::default()
        },
    )
    .unwrap();
    let skip = rec.effects().len();
    run(&mut store);
    let log = rec.effects()[skip..].to_vec();
    let pre = snapshot_dir(&pre_dir.path().join("demo"));
    let post = snapshot(&disk);
    (pre_dir, pre, post, log)
}

fn at_prefix(pre_dir: &Path, log: &[Effect], k: usize, into: &Path) -> PathBuf {
    let work = into.join("demo");
    let _ = std::fs::remove_dir_all(&work);
    copy_tree(&pre_dir.join("demo"), &work);
    rdm_engine::fs::replay_prefix(log, k, &work).unwrap();
    work
}

fn scratch_is_empty(root: &Path) -> bool {
    ["intent", "tmp"].iter().all(|d| {
        std::fs::read_dir(root.join(".rdm").join(d))
            .map(|e| e.count() == 0)
            .unwrap_or(true)
    })
}

#[test]
fn recover_redoes_an_interrupted_operation() {
    let (pre_dir, pre, post, log) = recorded(|s| {
        let mut edited = leaf_of(s, 4);
        edited.goal = "edit".into();
        let ops = vec![
            Op::Delete(delete_op(s, 5)),
            Op::Replace(ReplaceOp {
                expected_rev: Some(rev_of(s, 4)),
                node: Node::Leaf(edited),
            }),
        ];
        s.batch(req(
            Some("two kinds"),
            BatchOp {
                expected_tree_rev: None,
                ops,
                dry_run: false,
            },
        ))
        .unwrap();
    });
    // Cut just after the intent record was published, with a half-written file left in tmp.
    let intent = log
        .iter()
        .position(|e| matches!(e, Effect::Link { link, .. } | Effect::Rename { to: link, .. } if link.starts_with(".rdm/intent")))
        .unwrap();
    let work_dir = tempfile::tempdir().unwrap();
    let work = at_prefix(pre_dir.path(), &log, intent + 1, work_dir.path());
    std::fs::create_dir_all(work.join(".rdm/tmp")).unwrap();
    std::fs::write(work.join(".rdm/tmp/stray"), b"half").unwrap();
    assert_eq!(snapshot_dir(&work), pre, "a reader sees the state before");
    let mut store = Store::open(&work, &name("demo"), Options::default()).unwrap();
    let report = store.recover().unwrap();
    assert_eq!(
        (
            report.redone.len(),
            report.stray_temp_files,
            report.undecodable.len()
        ),
        (1, 1, 0)
    );
    assert_eq!(
        snapshot_dir(&work),
        post,
        "recovery completes the operation"
    );
    assert!(scratch_is_empty(&work));
    assert_eq!(
        store.campaign().get(&nid(5)),
        None,
        "and the store holds the result"
    );
    assert_eq!(
        store.recover().unwrap(),
        rdm_engine::write::Recovered::default(),
        "a second recovery has nothing to do"
    );
}

fn reused<T: std::fmt::Debug>(row: &str, r: Result<T, StoreError>) {
    match r {
        Err(StoreError::Invalid {
            file,
            rule: Rule::OpIdReused,
            ..
        }) => assert_eq!(file, "<request>", "{row}"),
        other => panic!("{row}: want op_id_reused, got {other:?}"),
    }
}

#[test]
fn an_op_id_the_records_cannot_answer_is_refused_not_made_up() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    let note = Some("n");

    // The first outcomes, each under its own op_id.
    let c = store
        .create(Request {
            header: header(note, Some("op-create")),
            op: create_op(leaf_node(20, "fresh", 2, "V"), None),
        })
        .unwrap();
    assert!(!c.replayed);
    let op = relocate_op(&store, 3, Some("moved"), None, None);
    store
        .move_node(Request {
            header: header(note, Some("op-move")),
            op,
        })
        .unwrap();
    let op = delete_op(&store, 5);
    store
        .delete_subtree(Request {
            header: header(note, Some("op-delete")),
            op,
        })
        .unwrap();
    store
        .comment(Request {
            header: header(None, Some("op-comment")),
            op: CommentOp {
                node_id: nid(4),
                text: vec!["one".into()],
            },
        })
        .unwrap();
    let before = snapshot(&disk);

    let with = |op_id: &str| header(note, Some(op_id));
    // A different node under the op_id of a create or a relocate.
    reused(
        "create of another node",
        store.create(Request {
            header: with("op-create"),
            op: create_op(leaf_node(21, "other", 2, "V"), None),
        }),
    );
    let op = relocate_op(&store, 4, Some("beta2"), None, None);
    reused(
        "relocate of another node",
        store.move_node(Request {
            header: with("op-move"),
            op,
        }),
    );
    // A delete of a node the amendment did not remove: it would have returned no node.
    let op = delete_op(&store, 4);
    reused(
        "delete of another node",
        store.delete_subtree(Request {
            header: with("op-delete"),
            op,
        }),
    );
    // More comments than carry the op_id: it used to fall through and write.
    let two = BatchOp {
        expected_tree_rev: None,
        dry_run: false,
        ops: vec![
            Op::Comment(CommentOp {
                node_id: nid(4),
                text: vec!["one".into()],
            }),
            Op::Comment(CommentOp {
                node_id: nid(4),
                text: vec!["two".into()],
            }),
        ],
    };
    reused(
        "two comments under the op_id of one",
        store.batch(Request {
            header: header(None, Some("op-comment")),
            op: two,
        }),
    );
    // Records of another kind than the request.
    reused(
        "a create under the op_id of a comment",
        store.create(Request {
            header: with("op-comment"),
            op: create_op(leaf_node(22, "third", 2, "V"), None),
        }),
    );
    reused(
        "a comment under the op_id of a create",
        store.comment(Request {
            header: header(None, Some("op-create")),
            op: CommentOp {
                node_id: nid(4),
                text: vec!["x".into()],
            },
        }),
    );
    assert_eq!(snapshot(&disk), before, "no refusal wrote anything");
    // The node of the first create is gone: its rev cannot be rebuilt.
    let op = delete_op(&store, 20);
    store.delete_subtree(req(note, op)).unwrap();
    let before_gone = snapshot(&disk);
    reused(
        "a create whose node is gone",
        store.create(Request {
            header: with("op-create"),
            op: create_op(leaf_node(20, "fresh", 2, "V"), None),
        }),
    );
    assert_eq!(snapshot(&disk), before_gone, "a refusal writes nothing");
}
