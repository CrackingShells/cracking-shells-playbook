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
            assert_eq!((id, rev), (nid(3), Some(fresh)));
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
}
