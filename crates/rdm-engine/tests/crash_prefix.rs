//! Guard of the write order: every prefix of the effects of every store operation loads, and
//! recovery turns it into the state before the operation or the state after it (`store.md`,
//! Write order, Intent and recovery; S2, S18). The method is Pillai et al., OSDI 2014: log the
//! effects once, then apply each prefix to a copy of the starting campaign.

mod common;

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use common::campaign::*;
use rdm_engine::fs::{Durability, Effect, Fs, RecordingFs, replay_prefix};
use rdm_engine::store::{Options, Store};
use rdm_schema::error::Rule;
use rdm_schema::ids::*;
use rdm_schema::ops::{
    BatchOp, CommentOp, CreateOp, DeleteOp, Op, Receipt, RelocateOp, ReplaceOp, Request,
};
use rdm_schema::stored::*;
use serde_json::Value;

const SEED: u64 = 0x5EED;

// ---------------------------------------------------------------------------------------------
// The campaign every row starts from, and the operations

/// ```text
/// demo (1)
///   core (2)    alpha (3), beta (4)
///   spare (5)   child (6)
///   deep (7)    d1 (8)    d2 (9)  leaf (10), leaf (11)
///                         leaf (12)
///   s1 (13)     p (14)    x (15)  y (16)
///   s2 (17)     q (18)
/// ```
fn base() -> Vec<Node> {
    let n = nid;
    vec![
        dir_node(n(1), "demo", None, ""),
        dir_node(n(2), "core", Some(n(1)), "V"),
        leaf_node(n(3), "alpha", n(2), "V"),
        leaf_node(n(4), "beta", n(2), "k"),
        dir_node(n(5), "spare", Some(n(1)), "k"),
        leaf_node(n(6), "child", n(5), "V"),
        dir_node(n(7), "deep", Some(n(1)), "s"),
        dir_node(n(8), "d1", Some(n(7)), "V"),
        dir_node(n(9), "d2", Some(n(8)), "V"),
        leaf_node(n(10), "g", n(9), "V"),
        leaf_node(n(11), "h", n(9), "k"),
        leaf_node(n(12), "i", n(8), "k"),
        dir_node(n(13), "s1", Some(n(1)), "t"),
        dir_node(n(14), "p", Some(n(13)), "V"),
        dir_node(n(15), "x", Some(n(14)), "V"),
        leaf_node(n(16), "y", n(15), "V"),
        dir_node(n(17), "s2", Some(n(1)), "u"),
        dir_node(n(18), "q", Some(n(17)), "V"),
    ]
}

/// What a merge of two branches leaves: `cy1` under `cy2` on one, `cy2` under `cy1` on the
/// other, with a leaf below.
fn cycle() -> Vec<Node> {
    let n = nid;
    vec![
        dir_node(n(20), "cy1", Some(n(21)), "w"),
        dir_node(n(21), "cy2", Some(n(20)), "x"),
        leaf_node(n(22), "below", n(20), "V"),
    ]
}

fn req<T>(note: Option<&str>, op: T) -> Request<T> {
    Request {
        header: header(note, Some("crash-op")),
        op,
    }
}

fn node_of(store: &Store, n: u32) -> Node {
    store.campaign().get(&nid(n)).unwrap().clone()
}

fn rev_of(store: &Store, n: u32) -> Rev {
    store.rev(&nid(n)).unwrap().clone()
}

fn delete_op(store: &Store, n: u32) -> DeleteOp {
    DeleteOp {
        id: nid(n),
        expected_subtree_rev: store.subtree_rev(&nid(n)),
    }
}

fn move_op(store: &Store, n: u32, parent: Option<u32>, new_name: Option<&str>) -> RelocateOp {
    RelocateOp {
        id: nid(n),
        expected_rev: Some(rev_of(store, n)),
        parent_id: parent.map(nid),
        name: new_name.map(name),
        position: None,
    }
}

fn edited_goal(store: &Store, n: u32) -> ReplaceOp {
    let Node::Leaf(mut leaf) = node_of(store, n) else {
        panic!("a leaf")
    };
    leaf.goal = "edited goal".into();
    ReplaceOp {
        expected_rev: Some(rev_of(store, n)),
        node: Node::Leaf(leaf),
    }
}

fn batch(store: &mut Store, note: &str, ops: Vec<Op>) -> Receipt<Vec<rdm_schema::ops::OpResult>> {
    let op = BatchOp {
        expected_tree_rev: None,
        ops,
        dry_run: false,
    };
    store.batch(req(Some(note), op)).unwrap()
}

type Run = Box<dyn Fn(&mut Store)>;

struct Row {
    name: &'static str,
    /// Nodes added to the base campaign.
    extra: Vec<Node>,
    /// Diagnostics a prefix may show besides those of the states before and after.
    allowed: Vec<(String, Rule)>,
    /// Where recovery itself is cut (S18): 0 not at all, 1 right after the commit point (where it
    /// has the most to redo), 2 also half way through the redo.
    recovery_cuts: u8,
    run: Run,
}

fn rows() -> Vec<Row> {
    let row = |name, run: Run| Row {
        name,
        extra: vec![],
        allowed: vec![],
        recovery_cuts: 0,
        run,
    };
    vec![
        row(
            "create a leaf",
            Box::new(|s| {
                let node = leaf_node(nid(30), "fresh", nid(2), "z");
                let op = CreateOp {
                    node,
                    position: None,
                };
                s.create(req(Some("add"), op)).unwrap();
            }),
        ),
        row(
            "replace with a note",
            Box::new(|s| {
                let op = edited_goal(s, 3);
                s.replace(req(Some("edit"), op)).unwrap();
            }),
        ),
        row(
            "replace without a note (a criterion met)",
            Box::new(|s| {
                let Node::Leaf(mut leaf) = node_of(s, 4) else {
                    panic!("a leaf")
                };
                leaf.success_gates[0].met = true;
                let op = ReplaceOp {
                    expected_rev: Some(rev_of(s, 4)),
                    node: Node::Leaf(leaf),
                };
                s.replace(req(None, op)).unwrap();
            }),
        ),
        row(
            "relocate: move",
            Box::new(|s| {
                let op = move_op(s, 3, Some(5), None);
                s.move_node(req(Some("move"), op)).unwrap();
            }),
        ),
        row(
            "relocate: rename only",
            Box::new(|s| {
                let op = move_op(s, 4, None, Some("beta2"));
                s.move_node(req(Some("rename"), op)).unwrap();
            }),
        ),
        Row {
            recovery_cuts: 1,
            ..row(
                "delete a deep subtree",
                Box::new(|s| {
                    let op = delete_op(s, 7);
                    s.delete_subtree(req(Some("delete"), op)).unwrap();
                }),
            )
        },
        Row {
            name: "delete a node on a parent_id cycle",
            extra: cycle(),
            // Whichever member goes first leaves the other with a missing parent: that is
            // already damage, and a cycle has no order without it.
            allowed: [20, 21]
                .iter()
                .map(|n| (file_of(&nid(*n)), Rule::ParentMissing))
                .collect(),
            recovery_cuts: 0,
            run: Box::new(|s| {
                let op = delete_op(s, 21);
                s.delete_subtree(req(Some("delete the cycle"), op)).unwrap();
            }),
        },
        row(
            "comment",
            Box::new(|s| {
                let op = CommentOp {
                    node_id: nid(4),
                    text: vec!["a remark".into()],
                };
                s.comment(req(None, op)).unwrap();
            }),
        ),
        Row {
            recovery_cuts: 2,
            ..row(
                "batch: create, replace, move, comment and delete",
                Box::new(|s| {
                    let ops = vec![
                        Op::Create(CreateOp {
                            node: leaf_node(nid(31), "made", nid(2), "z"),
                            position: None,
                        }),
                        Op::Replace(edited_goal(s, 3)),
                        Op::Relocate(move_op(s, 4, Some(5), None)),
                        Op::Comment(CommentOp {
                            node_id: nid(6),
                            text: vec!["noted".into()],
                        }),
                        Op::Delete(delete_op(s, 7)),
                    ];
                    batch(s, "mixed", ops);
                }),
            )
        },
        row(
            // A node leaves a subtree the batch deletes and enters another one it deletes.
            "batch: move a node between two subtrees the batch deletes",
            Box::new(|s| {
                let ops = vec![
                    Op::Relocate(move_op(s, 15, Some(18), None)),
                    Op::Delete(delete_op(s, 13)),
                    Op::Delete(delete_op(s, 17)),
                ];
                batch(s, "between", ops);
            }),
        ),
    ]
}

// ---------------------------------------------------------------------------------------------
// States and what they may show

/// Options of a store that reads and recovers copies: no sync, since a copy is never read after
/// a power cut and the effects replayed are the ones of the default tier (`record`).
fn copy_options(fs: Option<Arc<dyn Fs + Send + Sync>>) -> Options {
    Options {
        durability: Durability::Process,
        ..seeded_options(SEED, fs)
    }
}

fn open(root: &Path) -> Store {
    Store::open(root, &name("demo"), copy_options(None))
        .unwrap_or_else(|e| panic!("the campaign must load: {e:?}"))
}

fn diagnostic_keys(store: &Store) -> BTreeSet<(String, Rule)> {
    store
        .diagnostics()
        .iter()
        .map(|d| (d.file.clone(), d.rule))
        .collect()
}

fn diagnostics(root: &Path) -> BTreeSet<(String, Rule)> {
    diagnostic_keys(&open(root))
}

/// The ids a stored amendment lists in its `snapshot`.
fn snapshot_keys(bytes: &[u8]) -> BTreeSet<String> {
    let v: Value = serde_json::from_slice(bytes).unwrap();
    v.get("snapshot")
        .and_then(Value::as_object)
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

fn node_id_of(path: &str) -> Option<&str> {
    path.strip_prefix("nodes/")?.strip_suffix(".json")
}

fn has_intent(root: &Path) -> bool {
    std::fs::read_dir(root.join(".rdm/intent")).is_ok_and(|mut e| e.next().is_some())
}

fn json_of(files: &Files, id: u32) -> Value {
    let bytes = files
        .get(&file_of(&nid(id)))
        .unwrap_or_else(|| panic!("n{id} is absent"));
    serde_json::from_slice(bytes).unwrap()
}

/// The ids of `root` and everything below it, by a plain walk of `parent_id` over the node
/// files; a visited set ends it on a cycle.
fn subtree_by_walk(files: &Files, root: u32) -> BTreeSet<String> {
    let parent_of: Vec<(String, Option<String>)> = files
        .iter()
        .filter_map(|(p, b)| {
            let id = node_id_of(p)?.to_owned();
            let v: Value = serde_json::from_slice(b).unwrap();
            Some((
                id,
                v.get("parent_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
        .collect();
    let mut found = BTreeSet::from([nid(root).to_string()]);
    loop {
        let before = found.len();
        for (id, parent) in &parent_of {
            if parent.as_ref().is_some_and(|p| found.contains(p)) {
                found.insert(id.clone());
            }
        }
        if found.len() == before {
            return found;
        }
    }
}

fn node_ids(files: &Files) -> BTreeSet<String> {
    files
        .keys()
        .filter_map(|p| node_id_of(p))
        .map(str::to_owned)
        .collect()
}

/// What each operation did, as its request says and not as the store's run shows: a few facts
/// of the state after, read from the node files by plain JSON and a walk of `parent_id`.
fn facts(row: &str, pre: &Files, post: &Files) {
    let gone = |roots: &[u32]| -> BTreeSet<String> {
        roots
            .iter()
            .flat_map(|r| subtree_by_walk(pre, *r))
            .collect()
    };
    let removed = |roots: &[u32]| {
        let want: BTreeSet<String> = node_ids(pre).difference(&gone(roots)).cloned().collect();
        assert_eq!(
            node_ids(post),
            want,
            "{row}: the nodes left are those outside the removed subtrees"
        );
    };
    match row {
        "create a leaf" => {
            let n = json_of(post, 30);
            assert_eq!(n["parent_id"], nid(2).to_string(), "{row}");
            assert_eq!(n["name"], "fresh", "{row}");
            assert_eq!(node_ids(post).len(), node_ids(pre).len() + 1, "{row}");
        }
        "replace with a note" => assert_eq!(json_of(post, 3)["goal"], "edited goal", "{row}"),
        "relocate: move" => assert_eq!(json_of(post, 3)["parent_id"], nid(5).to_string(), "{row}"),
        "relocate: rename only" => {
            assert_eq!(json_of(post, 4)["name"], "beta2", "{row}");
            assert_eq!(
                json_of(post, 4)["parent_id"],
                json_of(pre, 4)["parent_id"],
                "{row}"
            );
        }
        "delete a deep subtree" => removed(&[7]),
        "delete a node on a parent_id cycle" => removed(&[21]),
        "batch: move a node between two subtrees the batch deletes" => removed(&[13, 17]),
        "comment" => {
            let new: Vec<_> = post
                .keys()
                .filter(|p| p.starts_with("_comments/"))
                .collect();
            assert_eq!(new.len(), 1, "{row}");
            let c: Value = serde_json::from_slice(&post[new[0]]).unwrap();
            assert_eq!(c["node_id"], nid(4).to_string(), "{row}");
        }
        "batch: create, replace, move, comment and delete" => {
            assert_eq!(json_of(post, 3)["goal"], "edited goal", "{row}");
            assert_eq!(json_of(post, 4)["parent_id"], nid(5).to_string(), "{row}");
            assert!(post.contains_key(&file_of(&nid(31))), "{row}");
            assert!(
                !node_ids(post).iter().any(|i| gone(&[7]).contains(i)),
                "{row}"
            );
        }
        _ => {}
    }
}

/// The oracle of a row: the state before, the state after, and the diagnostics each shows.
struct Oracle {
    pre: Files,
    post: Files,
    allowed: BTreeSet<(String, Rule)>,
    pre_diags: BTreeSet<(String, Rule)>,
    post_diags: BTreeSet<(String, Rule)>,
}

/// What a state that is neither whole before nor whole after may look like to a reader: the
/// three rules of `store.md` (Write order) as properties of the files on disk.
fn assert_reader_sees_a_valid_state(label: &str, o: &Oracle, work: &Path) -> Store {
    let state = files_of(work);
    // Whole files only: each file is as it was before or as it will be after.
    for (path, bytes) in &state {
        let known = o.pre.get(path) == Some(bytes) || o.post.get(path) == Some(bytes);
        assert!(known, "{label}: {path} holds bytes of neither state");
    }
    // The campaign loads, and shows no damage the two states do not show (no orphan).
    let store = open(work);
    let shown = diagnostic_keys(&store);
    let extra: Vec<_> = shown.difference(&o.allowed).collect();
    assert!(extra.is_empty(), "{label}: a reader sees damage: {extra:?}");
    // Rule 3: a node that is gone has its before-image in the snapshot of an amendment that is
    // already written.
    let written: BTreeSet<String> = state
        .iter()
        .filter(|(p, b)| p.starts_with("_amendments/") && o.pre.get(*p) != Some(*b))
        .flat_map(|(_, b)| snapshot_keys(b))
        .collect();
    for path in o.pre.keys().filter(|p| !state.contains_key(*p)) {
        if let Some(id) = node_id_of(path) {
            assert!(
                written.contains(id),
                "{label}: {path} was removed before an amendment snapshot held it"
            );
        }
    }
    // Rule 1 and the amendment rule: an amendment that is written claims only edits that are
    // done: every node file the operation creates or replaces is already in place.
    let amended = state
        .iter()
        .any(|(p, b)| p.starts_with("_amendments/") && o.pre.get(p) != Some(b));
    if amended {
        for (path, bytes) in &o.post {
            if path.starts_with("nodes/") && o.pre.get(path) != Some(bytes) {
                assert!(
                    state.get(path) == Some(bytes),
                    "{label}: the amendment is written before {path} is"
                );
            }
        }
    }
    store
}

/// Recovery on a copy: the state must be the one before or the one after, with nothing left in
/// the scratch area, and a second recovery must find nothing to do. Returns whether it is after.
/// `store` is the one a reader opened on the cut state.
fn recover_and_judge(label: &str, o: &Oracle, work: &Path, mut store: Store) -> bool {
    let committed = has_intent(work);
    store
        .recover()
        .unwrap_or_else(|e| panic!("{label}: recovery failed: {e:?}"));
    let state = files_of(work);
    let after = state == o.post;
    assert!(
        after || state == o.pre,
        "{label}: recovery gave neither the state before nor the state after"
    );
    assert!(
        scratch_is_empty(work),
        "{label}: .rdm/tmp or .rdm/intent is not empty after recovery"
    );
    if committed {
        assert!(
            after,
            "{label}: an intent was published, so it must complete"
        );
    }
    let expected = if after { &o.post_diags } else { &o.pre_diags };
    assert_eq!(
        &diagnostics(work),
        expected,
        "{label}: diagnostics after recovery"
    );
    assert_eq!(
        store.recover().unwrap(),
        rdm_engine::write::Recovered::default(),
        "{label}: a second recovery has nothing to do"
    );
    after
}

fn at_prefix(pre_dir: &Path, log: &[Effect], k: usize, into: &Path) {
    let _ = std::fs::remove_dir_all(into);
    copy_tree(pre_dir, into);
    replay_prefix(log, k, into).unwrap();
}

fn is_sync(e: &Effect) -> bool {
    matches!(e, Effect::SyncFile { .. } | Effect::SyncDir { .. })
}

/// The operation, run in full over a recording file system.
fn record(row: &Row) -> (tempfile::TempDir, Vec<Effect>, Files) {
    let mut nodes = base();
    nodes.extend(row.extra.clone());
    let disk = Disk::with(&nodes);
    let pre_dir = tempfile::tempdir().unwrap();
    copy_tree(&disk.root, &pre_dir.path().join("demo"));
    let rec = Arc::new(RecordingFs::new(&disk.root));
    let mut store = Store::open(
        &disk.root,
        &name("demo"),
        seeded_options(SEED, Some(rec.clone())),
    )
    .unwrap();
    let skip = rec.effects().len();
    (row.run)(&mut store);
    let log = rec.effects()[skip..].to_vec();
    assert!(
        scratch_is_empty(&disk.root),
        "{}: the operation left .rdm/tmp or .rdm/intent not empty",
        row.name
    );
    (pre_dir, log, files_of(&disk.root))
}

/// The states before and after come from running the operation fully and not at all, each on
/// its own copy: never from replaying the log.
fn oracle(row: &Row) -> (Oracle, Files) {
    let mut nodes = base();
    nodes.extend(row.extra.clone());
    let disk = Disk::with(&nodes);
    let pre = files_of(&disk.root);
    let pre_diags = diagnostics(&disk.root);
    let mut store = open(&disk.root);
    (row.run)(&mut store);
    let post = files_of(&disk.root);
    let post_diags = diagnostics(&disk.root);
    let mut allowed: BTreeSet<_> = pre_diags.union(&post_diags).cloned().collect();
    allowed.extend(row.allowed.iter().cloned());
    (
        Oracle {
            pre,
            post: post.clone(),
            allowed,
            pre_diags,
            post_diags,
        },
        post,
    )
}

/// Guards that every prefix of the effects of each operation, applied to a copy of the
/// campaign, loads with no damage beyond the states before and after, never removes a node
/// before an amendment holds it, and recovers to exactly the state before or the state after
/// (S2); and that a kill at each effect of recovery itself is repaired by the next recovery
/// (S18).
///
/// Bugs it catches: a node file removed before the amendment with its snapshot is written (data
/// loss on a kill); an amendment written before the edit it records; a delete order that leaves
/// a child on disk after its parent is gone, such as ordering by the tree after a batch moved
/// the child out (parent_missing in a prefix); an intent that is published after a node file
/// (recovery cannot complete the operation); a recovery that is not restartable.
#[test]
fn every_write_prefix_loads_and_recovers_to_pre_or_post() {
    for row in rows() {
        let (o, post) = oracle(&row);
        assert_ne!(
            o.pre, o.post,
            "{}: the operation changes the campaign",
            row.name
        );
        facts(row.name, &o.pre, &o.post);
        let (pre_dir, log, recorded_end) = record(&row);
        assert_eq!(
            recorded_end, post,
            "{}: two runs of one operation write the same bytes",
            row.name
        );
        let pre_dir = pre_dir.path().join("demo");
        let scratch = tempfile::tempdir().unwrap();
        let work = scratch.path().join("demo");

        let mut outcomes = Vec::new();
        for k in 0..=log.len() {
            let last = k == log.len();
            if k > 0 && is_sync(&log[k - 1]) && !last {
                continue; // a sync changes nothing a killed process leaves behind
            }
            let label = format!("{} / prefix {k} of {}", row.name, log.len());
            at_prefix(&pre_dir, &log, k, &work);
            if last {
                // Before any recovery: an operation that ran to the end leaves nothing behind.
                assert!(
                    scratch_is_empty(&work),
                    "{label}: .rdm/tmp or .rdm/intent is not empty after the last effect"
                );
            }
            let store = assert_reader_sees_a_valid_state(&label, &o, &work);
            outcomes.push(recover_and_judge(&label, &o, &work, store));
        }
        assert!(
            outcomes.windows(2).all(|w| !w[0] || w[1]),
            "{}: recovery went from after back to before: {outcomes:?}",
            row.name
        );
        assert_eq!(
            outcomes.first(),
            Some(&false),
            "{}: nothing done is before",
            row.name
        );
        assert_eq!(
            outcomes.last(),
            Some(&true),
            "{}: everything done is after",
            row.name
        );

        // S18: record what recovery does to a cut campaign, then cut recovery at each of its
        // effects and recover again.
        let publish = log
            .iter()
            .position(|e| matches!(e, Effect::Rename { to, .. } | Effect::Link { link: to, .. } if to.starts_with(".rdm/intent")))
            .or_else(|| log.iter().rposition(|e| matches!(e, Effect::Write { .. })))
            .unwrap();
        let mid = (publish + 1 + log.len()) / 2;
        let cuts = [publish + 1, mid];
        for &k in cuts.iter().take(row.recovery_cuts.into()) {
            at_prefix(&pre_dir, &log, k, &work);
            let rec = Arc::new(RecordingFs::new(&work));
            let mut store =
                Store::open(&work, &name("demo"), copy_options(Some(rec.clone()))).unwrap();
            store.recover().unwrap();
            let recovery = rec.effects();
            for j in 0..=recovery.len() {
                if j > 0 && is_sync(&recovery[j - 1]) {
                    continue;
                }
                let label = format!(
                    "{} / prefix {k}, recovery cut after {j} of {} effects",
                    row.name,
                    recovery.len()
                );
                at_prefix(&pre_dir, &log, k, &work);
                replay_prefix(&recovery, j, &work).unwrap();
                let store = assert_reader_sees_a_valid_state(&label, &o, &work);
                recover_and_judge(&label, &o, &work, store);
            }
        }
    }
}
