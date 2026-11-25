//! Guards of the hot cache: a cached read equals a cold read at that moment, whatever changed
//! the files, and a write is on disk when it returns.
//!
//! The oracle is a fresh `CachedCampaign::open` of the same directory. Every external change is made by
//! what makes it in real life: another process using the store (this test binary run again as a
//! child), an editor writing in place, `git checkout`, a copy, an `rm`. The campaign files come
//! from the codec and from store writes.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use common::campaign::*;
use common::hooked::{At, frozen_stat, options_with, vanish_at};
use common::{child_role, spawn_child};
use rdm_engine::cache::CachedCampaign;
use rdm_engine::codec::decode;
use rdm_engine::fs::{Effect, RecordingFs};
use rdm_engine::rev::rev;
use rdm_engine::store::{Options, Store};
use rdm_schema::ids::*;
use rdm_schema::ops::{
    BatchOp, CommentOp, CreateOp, DeleteOp, Op, Position, RelocateOp, ReplaceOp,
};
use rdm_schema::stored::*;
use serde_json::Value;

// ---------------------------------------------------------------------------------------------
// Building campaigns

// ---------------------------------------------------------------------------------------------
// Requests

fn edit_goal(store: &Store, n: u32, goal: &str) -> ReplaceOp {
    let mut leaf = leaf_in(store, n);
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

fn cold(root: &Path) -> CachedCampaign {
    CachedCampaign::open(root, &name("demo"), Options::default()).expect("a cold open of the files")
}

/// Everything a reader can ask of a store, warm against the fresh open of the same directory:
/// the model, the revisions, the diagnostics, the reverse index and the op index.
fn assert_same_as_cold(row: &str, warm: &Store, root: &Path) {
    let mut cold = cold(root);
    let cold = cold.read().unwrap();
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

/// A loader that shares nothing with the cache's build: every file decoded by the codec and
/// hashed by `rev`, in file order.
type Held = BTreeMap<NodeId, (Node, Rev)>;

fn independent(root: &Path) -> (Held, BTreeSet<(String, String)>) {
    let mut nodes = BTreeMap::new();
    let mut warnings = BTreeSet::new();
    for e in std::fs::read_dir(root.join("nodes")).unwrap() {
        let path = e.unwrap().path();
        let file = format!("nodes/{}", path.file_name().unwrap().to_string_lossy());
        let d = decode::<Node>(&file, &std::fs::read(&path).unwrap()).unwrap();
        warnings.extend(
            d.diagnostics
                .iter()
                .map(|w| (w.file.clone(), w.detail.clone())),
        );
        nodes.insert(d.value.id().clone(), (d.value.clone(), rev(&d.value)));
    }
    (nodes, warnings)
}

/// The cache against the independent loader: the same nodes, each with the same `rev`, and the
/// file warnings it reports.
fn assert_same_as_files(row: &str, warm: &Store, root: &Path) {
    let (nodes, warnings) = independent(root);
    let held: Held = warm
        .campaign()
        .nodes()
        .map(|n| {
            (
                n.id().clone(),
                (n.clone(), warm.rev(n.id()).unwrap().clone()),
            )
        })
        .collect();
    assert_eq!(held, nodes, "{row}: nodes and revs against the files");
    let reported: BTreeSet<(String, String)> = warm
        .diagnostics()
        .into_iter()
        .filter(|d| d.rule == rdm_schema::error::Rule::UnknownKey)
        .map(|d| (d.file, d.detail))
        .collect();
    assert_eq!(reported, warnings, "{row}: file warnings against the files");
}

fn read_same(row: &str, cache: &mut CachedCampaign, root: &Path) {
    let warm = cache
        .read()
        .unwrap_or_else(|e| panic!("{row}: the read failed: {e:?}"));
    assert_same_as_cold(row, warm, root);
    assert_same_as_files(row, warm, root);
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

/// Runs `test` again as a child process that executes `child_job` on `root`.
fn run_child(test: &str, root: &Path) {
    spawn_child(test, "job", root, &[]).wait_ok();
}

/// What the child process does: a content edit and a comment, each with an `op_id`, through a
/// store of its own.
fn child_job(root: &Path) {
    let mut store = cold(root);
    let op = edit_goal(store.read().unwrap(), 3, "written by the child");
    store
        .replace(req(Some("child edit"), Some("child-op-1"), op))
        .unwrap();
    store
        .comment(req(None, Some("child-op-2"), comment_on(4, "child says")))
        .unwrap();
}

// ---------------------------------------------------------------------------------------------
// cached_read_equals_cold_read

fn sound_aged() -> Disk {
    let disk = sound();
    age(&disk.root);
    disk
}

#[test]
fn cached_read_equals_cold_read() {
    if let Some((_, root)) = child_role() {
        return child_job(&root);
    }

    // Bug caught: a cache that trusts a signature the child's atomic replacement changed, or
    // that never sees the op index of a record another process wrote.
    let disk = sound();
    let mut cache = opened(&disk);
    run_child("cached_read_equals_cold_read", &disk.root);
    read_same("a store write from another process", &mut cache, &disk.root);
    let warm = cache.read().unwrap();
    assert_eq!(leaf_in(warm, 3).goal, "written by the child");
    assert!(
        warm.ops()
            .records(&OpId::parse("child-op-2").unwrap())
            .is_some(),
        "the op index holds the comment the child wrote"
    );

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
            edit_goal(store.read().unwrap(), 3, "other a"),
            edit_goal(store.read().unwrap(), 4, "other b"),
        );
        store.replace(req(Some("a"), None, a)).unwrap();
        store.replace(req(Some("b"), None, b)).unwrap();
        store
            .create(req(
                Some("new"),
                None,
                CreateOp {
                    node: leaf_n(20, "added", 2, "t"),
                    position: None,
                },
            ))
            .unwrap();
        let del = delete_of(store.read().unwrap(), 5);
        store.delete_subtree(req(Some("gone"), None, del)).unwrap();
        store.comment(req(None, None, comment_on(3, "hi"))).unwrap();
    }
    git(&disk.root, &["add", "-A"]);
    git(&disk.root, &["commit", "-m", "other"]);
    git(&disk.root, &["checkout", "main"]);
    let mut cache = opened(&disk);
    git(&disk.root, &["checkout", "other"]);
    read_same("git checkout of the other branch", &mut cache, &disk.root);
    assert_eq!(leaf_in(cache.read().unwrap(), 3).goal, "other a");
    git(&disk.root, &["checkout", "main"]);
    read_same("git checkout back", &mut cache, &disk.root);
    assert_eq!(leaf_in(cache.read().unwrap(), 3).goal, "a goal");

    // Bug caught: a cache that never lists the directory again.
    let disk = sound();
    let mut cache = opened(&disk);
    disk.write(&leaf_n(30, "dropped-in", 2, "t"));
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
        let op = edit_goal(s.read().unwrap(), 3, "elsewhere");
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

/// A file changes and the directory's listing does not. Each row is an actor and what it leaves:
/// the signature (identity, mtime, size, and on Unix the ctime) is all a read has to see it by.
/// Aged files are clean to the cache, so the racy re-read cannot rescue them; the fresh file of
/// the last row is racy, which is what a coarse timestamp gives two writes in one tick. On Unix
/// the ctime moves for every row, so each row is red only when the component it is named for is
/// dropped together with the ctime (what Windows, which has none, runs natively).
#[test]
fn an_aged_file_changes_and_the_directory_does_not() {
    // Bug caught, by row: a signature without the identity (a same-size rename-over that keeps the
    // mtime); a signature without the size (an in-place edit that grows the file and restores the
    // mtime); a signature without the mtime, or a skipped racy re-read (an in-place edit at the same
    // size inside one tick).
    type Change = fn(&Path);
    let rows: [(&str, bool, Change, &str); 3] = [
        (
            "rename-over keeping the mtime (rsync --times, a checkout)",
            true,
            |file| {
                let kept = std::fs::metadata(file).unwrap().modified().unwrap();
                let text = std::fs::read_to_string(file)
                    .unwrap()
                    .replace("a goal", "b goal");
                let side = file
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("replacement.tmp");
                std::fs::write(&side, text).unwrap();
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(&side)
                    .unwrap()
                    .set_modified(kept)
                    .unwrap();
                std::fs::rename(&side, file).unwrap();
            },
            "b goal",
        ),
        (
            "touch -r after an edit that changes the size",
            true,
            |file| rewrite_in_place(file, "a goal", "a much longer goal"),
            "a much longer goal",
        ),
        (
            "the racy same-tick in-place edit",
            false,
            |file| rewrite_in_place(file, "a goal", "b goal"),
            "b goal",
        ),
    ];
    for (row, aged, change, goal) in rows {
        let disk = if aged { sound_aged() } else { sound() };
        let mut cache = opened(&disk);
        change(&disk.path(&file(3)));
        read_same(row, &mut cache, &disk.root);
        assert_eq!(leaf_in(cache.read().unwrap(), 3).goal, goal, "{row}");
    }
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
    let mut linked = leaf_n(21, "linked", 2, "t");
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
    let mut moved = leaf_in(cache.read().unwrap(), 4);
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
            node: leaf_n(22, "batch-new", 2, "u"),
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
    let mut cleaned = leaf_in(cache.read().unwrap(), 3);
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
    let mut keyed = leaf_in(cache.read().unwrap(), 3);
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
    // Bug caught: a write that does not revalidate after its recovery, so that it is computed on
    // the entries read before it (the node the interrupted batch deleted, the node it edited): the
    // create below is refused with `name_collision`; or a report of the recovery that the
    // write-through lost.
    let disk = sound();
    disk.write(&leaf_n(6, "child", 5, "V"));
    let pre = tempfile::tempdir().unwrap();
    copy_tree(&disk.root, &pre.path().join("demo"));
    let rec = Arc::new(RecordingFs::new(&disk.root));
    let log = {
        let mut s = CachedCampaign::open(
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
            Op::Delete(delete_of(s.read().unwrap(), 5)),
            Op::Replace(edit_goal(s.read().unwrap(), 4, "batched")),
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
    // The write builds on the recovered state: it takes the name the interrupted batch's delete
    // freed. Judged on the state before the recovery, node 5 still holds the name.
    cache
        .create(req(
            Some("reuse the name"),
            None,
            CreateOp {
                node: leaf_n(60, "spare", 1, "x"),
                position: None,
            },
        ))
        .expect("the name of the node the recovery deleted is free");
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
    if let Some((_, root)) = child_role() {
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

// ---------------------------------------------------------------------------------------------
// write_through_visible_to_cold_reader_on_return

/// Every file with its size and modification time: a write that rewrote identical bytes shows.
fn names_under(root: &Path) -> BTreeSet<String> {
    files_of(root)
        .keys()
        .map(|rel| {
            let m = std::fs::metadata(root.join(rel)).unwrap();
            format!("{rel} {} {:?}", m.len(), m.modified().unwrap())
        })
        .collect()
}

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
    assert_eq!(
        leaf_in(cold(&root).read().unwrap(), 3).goal,
        "durable",
        "replace"
    );

    let created = cache
        .create(req(
            Some("c"),
            None,
            CreateOp {
                node: leaf_n(40, "fresh", 2, "t"),
                position: None,
            },
        ))
        .unwrap();
    assert!(exists(format!("nodes/{}.json", nid(40))), "create: file");
    assert!(
        cold(&root)
            .read()
            .unwrap()
            .campaign()
            .get(&nid(40))
            .is_some(),
        "create"
    );
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
    assert!(
        cold(&root)
            .read()
            .unwrap()
            .comments()
            .contains_key(&commented.result.id)
    );

    let mv = RelocateOp {
        id: nid(40),
        expected_rev: cache.read().unwrap().rev(&nid(40)).cloned(),
        parent_id: Some(nid(5)),
        name: None,
        position: None,
    };
    cache.move_node(req(Some("m"), None, mv)).unwrap();
    assert_eq!(
        cold(&root)
            .read()
            .unwrap()
            .campaign()
            .get(&nid(40))
            .unwrap()
            .parent_id(),
        Some(&nid(5)),
        "move"
    );

    let del = delete_of(cache.read().unwrap(), 5);
    cache.delete_subtree(req(Some("d"), None, del)).unwrap();
    assert!(!exists(format!("nodes/{}.json", nid(5))), "delete: file");
    assert!(
        cold(&root)
            .read()
            .unwrap()
            .campaign()
            .get(&nid(5))
            .is_none(),
        "delete"
    );
}

// ---------------------------------------------------------------------------------------------
// Faults the real file system cannot be made to produce on cue: the seam

#[test]
fn a_file_that_vanishes_between_the_listing_and_its_stat_or_read_is_absent_not_an_error() {
    // Bug caught: a revalidation that reports a concurrent delete or rename as an `Io` error,
    // which fails every read and every write while a tool or `git checkout` replaces files
    // (store.md, §Cache, revalidation step 4). The Fs seam removes the file after the listing
    // named it, at the stat or at the read.
    for at in [At::Stat, At::Read] {
        // A cold open that races a delete.
        let disk = sound();
        let (fs, armed) = vanish_at(at, &disk.path(&file(4)));
        armed.store(true, Ordering::SeqCst);
        let mut store = CachedCampaign::open(&disk.root, &name("demo"), options_with(fs))
            .expect("a file gone before its read is absent, not an error");
        assert!(store.read().unwrap().campaign().get(&nid(4)).is_none());
        assert_same_as_cold(
            "a cold open that raced a delete",
            store.read().unwrap(),
            &disk.root,
        );

        // A warm read that races a delete of a file it holds an entry for. The entry is racy
        // (fresh files), so the read reaches the stat and the read.
        let disk = sound();
        let (fs, armed) = vanish_at(at, &disk.path(&file(4)));
        let mut cache = CachedCampaign::open(&disk.root, &name("demo"), options_with(fs)).unwrap();
        armed.store(true, Ordering::SeqCst);
        read_same("a warm read that raced a delete", &mut cache, &disk.root);
        assert!(cache.read().unwrap().campaign().get(&nid(4)).is_none());
    }
}

#[test]
fn a_rebalance_rewrites_a_sibling_from_the_disk_not_the_cache() {
    // Bug caught: a rebalance that renumbers a sibling from its cache entry. The sibling is not
    // named by the request, so the up-front read does not reach it: only the plan knows it will be
    // rewritten. With a signature that hides an external same-size edit (the frozen stat), the
    // cache holds the old sibling and the rewrite would put it back over the edit.
    //
    // Two siblings share an order key (what a merge can leave), and a create after the first
    // finds no room between them, so every sibling is renumbered. The code rewrites the sibling
    // from the disk bytes: the edit survives, with its new key.
    let disk = Disk::new();
    let mut nodes = base();
    nodes[3] = leaf_n(4, "beta", 2, "V");
    disk.write_all(&nodes);
    age(&disk.root);
    let mut cache =
        CachedCampaign::open(&disk.root, &name("demo"), options_with(frozen_stat())).unwrap();
    read_same("opened", &mut cache, &disk.root);
    let file = disk.path(&file(4));
    rewrite_in_place(&file, "a goal", "b goal");
    assert_eq!(
        leaf_in(cache.read().unwrap(), 4).goal,
        "a goal",
        "the edit is hidden from reads"
    );

    cache
        .create(req(
            Some("between"),
            None,
            CreateOp {
                node: leaf_n(70, "between", 2, "V"),
                position: Some(Position::After(nid(3))),
            },
        ))
        .unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains("b goal"),
        "the external edit survived the rebalance"
    );
    let keys: Vec<String> = [3, 70, 4]
        .iter()
        .map(|n| {
            cold(&disk.root)
                .read()
                .unwrap()
                .campaign()
                .get(&nid(*n))
                .unwrap()
                .order()
                .unwrap()
                .as_str()
                .to_owned()
        })
        .collect();
    assert!(
        keys.windows(2).all(|w| w[0] < w[1]),
        "renumbered in order: {keys:?}"
    );
    assert_eq!(leaf_in(cold(&disk.root).read().unwrap(), 4).goal, "b goal");
}
