//! Guards of the hot cache: a cached read equals a cold read at that moment, whatever changed
//! the files, and a write is on disk when it returns.
//!
//! The oracle is a fresh `Store::open` of the same directory. Every external change is made by
//! what makes it in real life: another process using the store (this test binary run again as a
//! child), an editor writing in place, `git checkout`, a copy, an `rm`. The campaign files come
//! from the codec and from store writes.

use std::collections::BTreeSet;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use rdm_engine::cache::CachedCampaign;
use rdm_engine::codec::encode;
use rdm_engine::fs::{Effect, RecordingFs};
use rdm_engine::store::{Options, Store};
use rdm_schema::ids::*;
use rdm_schema::ops::{
    BatchOp, CommentOp, CreateOp, DeleteOp, EditNote, Header, Op, Position, RelocateOp, ReplaceOp,
    Request,
};
use rdm_schema::stored::*;
use serde_json::Value;

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

// ---------------------------------------------------------------------------------------------
// Requests

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

fn req<T>(note: Option<&str>, op_id: Option<&str>, op: T) -> Request<T> {
    Request {
        header: header(note, op_id),
        op,
    }
}

fn leaf_of(store: &Store, n: u32) -> Leaf {
    match store.campaign().get(&nid(n)).unwrap() {
        Node::Leaf(l) => l.clone(),
        Node::Directory(_) => panic!("not a leaf"),
    }
}

fn edit_goal(store: &Store, n: u32, goal: &str) -> ReplaceOp {
    let mut leaf = leaf_of(store, n);
    leaf.goal = goal.into();
    ReplaceOp {
        expected_rev: store.rev(&nid(n)).cloned(),
        node: Node::Leaf(leaf),
    }
}

fn comment_on(n: u32, text: &str) -> CommentOp {
    CommentOp {
        node_id: nid(n),
        text: vec![text.into()],
    }
}

fn delete_of(store: &Store, n: u32) -> DeleteOp {
    DeleteOp {
        id: nid(n),
        expected_subtree_rev: store.subtree_rev(&nid(n)),
    }
}

// ---------------------------------------------------------------------------------------------
// The oracle: a cold open

fn cold(root: &Path) -> Store {
    Store::open(root, &name("demo"), Options::default()).expect("a cold open of the files")
}

/// Everything a reader can ask of a store, warm against the fresh open of the same directory:
/// the model, the revisions, the diagnostics, the reverse index and the op index.
fn assert_same_as_cold(row: &str, warm: &Store, root: &Path) {
    let cold = cold(root);
    assert_eq!(warm.campaign(), cold.campaign(), "{row}: campaign");
    assert_eq!(warm.diagnostics(), cold.diagnostics(), "{row}: diagnostics");
    assert_eq!(warm.comments(), cold.comments(), "{row}: comments");
    assert_eq!(warm.amendments(), cold.amendments(), "{row}: amendments");
    assert_eq!(warm.reverse(), cold.reverse(), "{row}: reverse index");
    assert_eq!(warm.ops(), cold.ops(), "{row}: op index");
    assert_eq!(warm.tree_rev(), cold.tree_rev(), "{row}: tree_rev");
    for node in cold.campaign().nodes() {
        assert_eq!(warm.rev(node.id()), cold.rev(node.id()), "{row}: rev");
    }
}

fn read_same(row: &str, cache: &mut CachedCampaign, root: &Path) {
    let warm = cache
        .read()
        .unwrap_or_else(|e| panic!("{row}: the read failed: {e:?}"));
    assert_same_as_cold(row, warm, root);
}

fn opened(disk: &Disk) -> CachedCampaign {
    let mut cache = CachedCampaign::open(&disk.root, &name("demo"), Options::default()).unwrap();
    read_same("opened", &mut cache, &disk.root);
    cache
}

fn sound() -> Disk {
    let disk = Disk::new();
    disk.write_all(&base());
    disk
}

// ---------------------------------------------------------------------------------------------
// Another process

/// What the child process does: a content edit and a comment, each with an `op_id`, through a
/// store of its own.
fn child_job(root: &Path) {
    let mut store = cold(root);
    let op = edit_goal(&store, 3, "written by the child");
    store
        .replace(req(Some("child edit"), Some("child-op-1"), op))
        .unwrap();
    store
        .comment(req(None, Some("child-op-2"), comment_on(4, "child says")))
        .unwrap();
}

/// Runs `test` again as a child process that executes `child_job` on `root`.
fn run_child(test: &str, root: &Path) {
    let out = Command::new(std::env::current_exe().unwrap())
        .args([test, "--exact", "--nocapture", "--test-threads=1"])
        .env("RDM_CACHE_CHILD", root)
        .output()
        .expect("the test binary runs again");
    assert!(
        out.status.success(),
        "the child failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn child_role() -> Option<PathBuf> {
    std::env::var_os("RDM_CACHE_CHILD").map(PathBuf::from)
}

// ---------------------------------------------------------------------------------------------
// cached_read_equals_cold_read

/// A file that an editor rewrote in place: the same descriptor is not reopened with truncation,
/// the bytes are overwritten from offset 0 at the same length, and the modification time is the
/// one the file had, which is what a coarse timestamp (FAT's two seconds, HFS+'s one) gives two
/// writes in one tick.
fn rewrite_in_place_same_size(path: &Path, from: &str, to: &str) {
    assert_eq!(from.len(), to.len());
    let before = std::fs::metadata(path).unwrap();
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.contains(from));
    let bytes = text.replace(from, to).into_bytes();
    assert_eq!(bytes.len() as u64, before.len());
    let mut file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.rewind().unwrap();
    file.write_all(&bytes).unwrap();
    file.set_modified(before.modified().unwrap()).unwrap();
}

#[test]
fn cached_read_equals_cold_read() {
    if let Some(root) = child_role() {
        return child_job(&root);
    }

    // Bug caught: a cache that trusts a signature the child's atomic replacement changed, or
    // that never sees the op index of a record another process wrote.
    let disk = sound();
    let mut cache = opened(&disk);
    run_child("cached_read_equals_cold_read", &disk.root);
    read_same("a store write from another process", &mut cache, &disk.root);
    let warm = cache.read().unwrap();
    assert_eq!(leaf_of(warm, 3).goal, "written by the child");
    assert!(
        warm.ops()
            .records(&OpId::parse("child-op-2").unwrap())
            .is_some(),
        "the op index holds the comment the child wrote"
    );

    // Bug caught: revalidating by the directory's modification time alone (an in-place write
    // does not change it), or skipping the racy re-read (the write leaves the size and the
    // modification time as they were).
    let disk = sound();
    let mut cache = opened(&disk);
    let file = disk.path(&format!("nodes/{}.json", nid(3)));
    rewrite_in_place_same_size(&file, "a goal", "b goal");
    read_same(
        "an editor writing in place at the same size within one tick",
        &mut cache,
        &disk.root,
    );
    assert_eq!(leaf_of(cache.read().unwrap(), 3).goal, "b goal");

    // Bug caught: a cache that keeps the unknown key an editor added out of the diagnostics.
    let disk = sound();
    let mut cache = opened(&disk);
    let file = disk.path(&format!("nodes/{}.json", nid(4)));
    let mut v: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    v["zz"] = serde_json::json!(1);
    std::fs::write(&file, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
    read_same("an unknown key added by an editor", &mut cache, &disk.root);
    assert!(
        cache
            .read()
            .unwrap()
            .diagnostics()
            .iter()
            .any(|d| d.rule == rdm_schema::error::Rule::UnknownKey),
        "the warning is reported"
    );

    // Bug caught: a cache that revalidates only the files it knew, or only the first of many
    // replaced files: `git checkout` replaces, adds and removes several at once.
    let disk = sound();
    git(&disk.root, &["init", "-b", "main"]);
    git(&disk.root, &["add", "-A"]);
    git(&disk.root, &["commit", "-m", "base"]);
    git(&disk.root, &["checkout", "-b", "other"]);
    {
        let mut store = cold(&disk.root);
        let (a, b) = (
            edit_goal(&store, 3, "other a"),
            edit_goal(&store, 4, "other b"),
        );
        store.replace(req(Some("a"), None, a)).unwrap();
        store.replace(req(Some("b"), None, b)).unwrap();
        store
            .create(req(
                Some("new"),
                None,
                CreateOp {
                    node: leaf_node(20, "added", 2, "t"),
                    position: None,
                },
            ))
            .unwrap();
        let del = delete_of(&store, 5);
        store.delete_subtree(req(Some("gone"), None, del)).unwrap();
        store.comment(req(None, None, comment_on(3, "hi"))).unwrap();
    }
    git(&disk.root, &["add", "-A"]);
    git(&disk.root, &["commit", "-m", "other"]);
    git(&disk.root, &["checkout", "main"]);
    let mut cache = opened(&disk);
    git(&disk.root, &["checkout", "other"]);
    read_same("git checkout of the other branch", &mut cache, &disk.root);
    assert_eq!(leaf_of(cache.read().unwrap(), 3).goal, "other a");
    git(&disk.root, &["checkout", "main"]);
    read_same("git checkout back", &mut cache, &disk.root);
    assert_eq!(leaf_of(cache.read().unwrap(), 3).goal, "a goal");

    // Bug caught: a cache that never lists the directory again.
    let disk = sound();
    let mut cache = opened(&disk);
    disk.write(&leaf_node(30, "dropped-in", 2, "t"));
    read_same("a file added", &mut cache, &disk.root);
    assert!(cache.read().unwrap().campaign().get(&nid(30)).is_some());

    // Bug caught: a cache that keeps the entry of a file that is gone.
    let disk = sound();
    let mut cache = opened(&disk);
    std::fs::remove_file(disk.path(&format!("nodes/{}.json", nid(4)))).unwrap();
    read_same("a file deleted", &mut cache, &disk.root);
    assert!(cache.read().unwrap().campaign().get(&nid(4)).is_none());

    // Bug caught: the append-only directories checked less than `nodes/`: a comment and an
    // amendment file produced by a store elsewhere and copied in, then removed.
    let disk = sound();
    let other = sound();
    {
        let mut s = cold(&other.root);
        let op = edit_goal(&s, 3, "elsewhere");
        s.replace(req(Some("edit"), Some("elsewhere-1"), op))
            .unwrap();
        s.comment(req(None, Some("elsewhere-2"), comment_on(3, "c")))
            .unwrap();
    }
    let mut cache = opened(&disk);
    let mut copied = Vec::new();
    for dir in ["_comments", "_amendments"] {
        std::fs::create_dir_all(disk.path(dir)).unwrap();
        for e in std::fs::read_dir(other.path(dir)).unwrap() {
            let from = e.unwrap().path();
            let to = disk.path(dir).join(from.file_name().unwrap());
            std::fs::copy(&from, &to).unwrap();
            copied.push(to);
        }
    }
    read_same(
        "a comment and an amendment copied in",
        &mut cache,
        &disk.root,
    );
    assert!(!cache.read().unwrap().comments().is_empty());
    copied.iter().for_each(|p| std::fs::remove_file(p).unwrap());
    read_same("a comment and an amendment removed", &mut cache, &disk.root);
    assert!(cache.read().unwrap().comments().is_empty());
}

#[test]
fn a_write_by_this_process_then_a_read_equals_a_cold_read() {
    // Bug caught: write-through that updates the model but not the op index, the reverse index,
    // the diagnostics or the revisions. While every write reloaded from disk, none of these
    // could be seen to be wrong. Each step is followed by the comparison with a fresh open.
    let disk = sound();
    let mut cache = opened(&disk);
    let root = disk.root.clone();

    cache
        .comment(req(None, Some("w-1"), comment_on(3, "first")))
        .unwrap();
    read_same("a comment with an op_id", &mut cache, &root);
    assert!(
        cache
            .read()
            .unwrap()
            .ops()
            .records(&OpId::parse("w-1").unwrap())
            .is_some()
    );

    let op = edit_goal(cache.read().unwrap(), 3, "edited");
    cache.replace(req(Some("e"), Some("w-2"), op)).unwrap();
    read_same("a content edit with an op_id", &mut cache, &root);

    // A node that points at another: the reverse index must learn the link.
    let mut linked = leaf_node(21, "linked", 2, "t");
    if let Node::Leaf(l) = &mut linked {
        l.references[0].target = None;
        l.references[0].node_id = Some(nid(4));
    }
    cache
        .create(req(
            Some("new"),
            Some("w-3"),
            CreateOp {
                node: linked,
                position: None,
            },
        ))
        .unwrap();
    read_same("a node created with a link", &mut cache, &root);
    assert!(
        !cache
            .read()
            .unwrap()
            .reverse()
            .referrers(&nid(4))
            .is_empty(),
        "the reverse index holds the new link"
    );

    let mv = RelocateOp {
        id: nid(21),
        expected_rev: cache.read().unwrap().rev(&nid(21)).cloned(),
        parent_id: Some(nid(5)),
        name: Some(name("moved")),
        position: Some(Position::First),
    };
    cache.move_node(req(Some("m"), None, mv)).unwrap();
    read_same("a node moved", &mut cache, &root);

    let del = delete_of(cache.read().unwrap(), 21);
    cache.delete_subtree(req(Some("d"), None, del)).unwrap();
    read_same("a node deleted", &mut cache, &root);
    assert!(
        cache
            .read()
            .unwrap()
            .reverse()
            .referrers(&nid(4))
            .is_empty(),
        "the link of the deleted node is gone from the reverse index"
    );

    // A status move carries its `op_id` in `History`, a record the op index also holds.
    let mut moved = leaf_of(cache.read().unwrap(), 4);
    moved.status = Status::Ongoing;
    moved.history.push(History {
        ts: "2026-01-01T00:00:00Z".into(),
        from: Status::Planned,
        to: Status::Ongoing,
        actor: actor(),
        reason: None,
        r#override: None,
        op_id: Some(OpId::parse("w-hist").unwrap()),
        correction: None,
        extra: Default::default(),
    });
    let op = ReplaceOp {
        expected_rev: cache.read().unwrap().rev(&nid(4)).cloned(),
        node: Node::Leaf(moved),
    };
    cache.replace(req(None, Some("w-hist"), op)).unwrap();
    read_same("a status move with an op_id", &mut cache, &root);
    assert!(
        cache
            .read()
            .unwrap()
            .ops()
            .records(&OpId::parse("w-hist").unwrap())
            .is_some()
    );

    let store = cache.read().unwrap();
    let ops = vec![
        Op::Replace(edit_goal(store, 4, "batched")),
        Op::Create(CreateOp {
            node: leaf_node(22, "batch-new", 2, "u"),
            position: None,
        }),
    ];
    cache
        .batch(req(
            Some("b"),
            Some("w-4"),
            BatchOp {
                expected_tree_rev: None,
                ops,
                dry_run: false,
            },
        ))
        .unwrap();
    read_same("a batch", &mut cache, &root);

    // An unknown key the store keeps (I12) is still reported after the store rewrote the file.
    let file = disk.path(&format!("nodes/{}.json", nid(3)));
    let mut v: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    v["zz"] = serde_json::json!(1);
    std::fs::write(&file, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
    read_same("an unknown key added", &mut cache, &root);
    let mut cleaned = leaf_of(cache.read().unwrap(), 3);
    cleaned.extra.clear();
    let op = ReplaceOp {
        expected_rev: cache.read().unwrap().rev(&nid(3)).cloned(),
        node: Node::Leaf(cleaned),
    };
    cache.replace(req(Some("e2"), None, op)).unwrap();
    read_same(
        "a node rewritten without its unknown key",
        &mut cache,
        &root,
    );
    assert!(
        cache
            .read()
            .unwrap()
            .diagnostics()
            .iter()
            .all(|d| d.rule != rdm_schema::error::Rule::UnknownKey),
        "the warning went with the key"
    );
    let mut keyed = leaf_of(cache.read().unwrap(), 3);
    keyed.extra.insert("yy".into(), serde_json::json!(2));
    let op = ReplaceOp {
        expected_rev: cache.read().unwrap().rev(&nid(3)).cloned(),
        node: Node::Leaf(keyed),
    };
    cache.replace(req(Some("e3"), None, op)).unwrap();
    read_same("a node rewritten with an unknown key", &mut cache, &root);

    // The node that holds a history `op_id` goes: the op index must forget it.
    let del = delete_of(cache.read().unwrap(), 4);
    cache.delete_subtree(req(Some("d4"), None, del)).unwrap();
    read_same("a node with a history op_id deleted", &mut cache, &root);
}

#[test]
fn a_write_recovers_an_interrupted_operation_and_the_cache_follows() {
    // Bug caught: a recovery that changes files under the cache, and the cache keeps serving the
    // entries it read before (the node the interrupted batch deleted, the node it edited), or a
    // report of the recovery that the write-through lost.
    let disk = sound();
    disk.write(&leaf_node(6, "child", 5, "V"));
    let pre = tempfile::tempdir().unwrap();
    copy_tree(&disk.root, &pre.path().join("demo"));
    let rec = Arc::new(RecordingFs::new(&disk.root));
    let log = {
        let mut s = Store::open(
            &disk.root,
            &name("demo"),
            Options {
                fs: rec.clone(),
                ..Options::default()
            },
        )
        .unwrap();
        let skip = rec.effects().len();
        let ops = vec![
            Op::Delete(delete_of(&s, 5)),
            Op::Replace(edit_goal(&s, 4, "batched")),
        ];
        s.batch(req(
            Some("two kinds"),
            None,
            BatchOp {
                expected_tree_rev: None,
                ops,
                dry_run: false,
            },
        ))
        .unwrap();
        rec.effects()[skip..].to_vec()
    };
    let published = log
        .iter()
        .position(|e| matches!(e, Effect::Link { link, .. } | Effect::Rename { to: link, .. } if link.starts_with(".rdm/intent")))
        .unwrap();
    let work_dir = tempfile::tempdir().unwrap();
    let work = work_dir.path().join("demo");
    copy_tree(&pre.path().join("demo"), &work);
    rdm_engine::fs::replay_prefix(&log, published + 1, &work).unwrap();

    let mut cache = CachedCampaign::open(&work, &name("demo"), Options::default()).unwrap();
    read_same("before the recovery", &mut cache, &work);
    assert!(cache.read().unwrap().campaign().get(&nid(5)).is_some());
    cache.comment(req(None, None, comment_on(3, "hi"))).unwrap();
    assert_eq!(cache.last_recovered().unwrap().redone.len(), 1);
    read_same("after a write that recovered", &mut cache, &work);
    assert!(cache.read().unwrap().campaign().get(&nid(5)).is_none());
    cache
        .comment(req(None, None, comment_on(3, "again")))
        .unwrap();
    assert_eq!(cache.last_recovered().unwrap().redone.len(), 0);
    read_same("after the next write", &mut cache, &work);
}

#[test]
fn a_replayed_op_id_is_found_after_another_process_wrote_it() {
    if let Some(root) = child_role() {
        return child_job(&root);
    }
    // Bug caught: the replay check run on the cache as it was before the lock was taken. The
    // child wrote `child-op-1` after the cache last looked, so the retry would find no record,
    // judge its `expected_rev` against a state that has moved, and fail with `Stale` (or write
    // twice) instead of returning the first outcome with nothing written.
    let disk = sound();
    let mut cache = opened(&disk);
    let retry = {
        let store = cache.read().unwrap();
        req(
            Some("child edit"),
            Some("child-op-1"),
            edit_goal(store, 3, "written by the child"),
        )
    };
    run_child(
        "a_replayed_op_id_is_found_after_another_process_wrote_it",
        &disk.root,
    );
    let files = names_under(&disk.root);
    let receipt = cache
        .replace(retry)
        .expect("the retry is a replay, not a stale write");
    assert!(
        receipt.replayed,
        "the retry was written as a first-time request"
    );
    assert_eq!(names_under(&disk.root), files, "a replay writes nothing");
}

fn names_under(root: &Path) -> BTreeSet<String> {
    fn walk(dir: &Path, base: &Path, out: &mut BTreeSet<String>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n == ".rdm" || n == ".git") {
                    continue;
                }
                walk(&p, base, out);
            } else {
                let m = std::fs::metadata(&p).unwrap();
                out.insert(format!(
                    "{} {} {:?}",
                    p.strip_prefix(base).unwrap().display(),
                    m.len(),
                    m.modified().unwrap()
                ));
            }
        }
    }
    let mut out = BTreeSet::new();
    walk(root, root, &mut out);
    out
}

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

// ---------------------------------------------------------------------------------------------
// write_through_visible_to_cold_reader_on_return

#[test]
fn write_through_visible_to_cold_reader_on_return() {
    // Bug caught: a write-behind cache that answers from memory and flushes later. A cold reader
    // (a fresh open, which shares nothing with the cache) that starts after the return must find
    // every file the operation was to write, with its content.
    let disk = sound();
    let mut cache = opened(&disk);
    let root = disk.root.clone();
    let exists = |rel: String| root.join(rel).is_file();

    let op = edit_goal(cache.read().unwrap(), 3, "durable");
    cache.replace(req(Some("e"), None, op)).unwrap();
    assert_eq!(leaf_of(&cold(&root), 3).goal, "durable", "replace");

    let created = cache
        .create(req(
            Some("c"),
            None,
            CreateOp {
                node: leaf_node(40, "fresh", 2, "t"),
                position: None,
            },
        ))
        .unwrap();
    assert!(exists(format!("nodes/{}.json", nid(40))), "create: file");
    assert!(cold(&root).campaign().get(&nid(40)).is_some(), "create");
    assert!(
        exists(format!("_amendments/{}.json", created.amendment.unwrap())),
        "create: its amendment"
    );

    let commented = cache
        .comment(req(None, None, comment_on(3, "kept")))
        .unwrap();
    assert!(
        exists(format!("_comments/{}.json", commented.result.id)),
        "comment"
    );
    assert!(cold(&root).comments().contains_key(&commented.result.id));

    let mv = RelocateOp {
        id: nid(40),
        expected_rev: cache.read().unwrap().rev(&nid(40)).cloned(),
        parent_id: Some(nid(5)),
        name: None,
        position: None,
    };
    cache.move_node(req(Some("m"), None, mv)).unwrap();
    assert_eq!(
        cold(&root).campaign().get(&nid(40)).unwrap().parent_id(),
        Some(&nid(5)),
        "move"
    );

    let del = delete_of(cache.read().unwrap(), 5);
    cache.delete_subtree(req(Some("d"), None, del)).unwrap();
    assert!(!exists(format!("nodes/{}.json", nid(5))), "delete: file");
    assert!(cold(&root).campaign().get(&nid(5)).is_none(), "delete");
}
