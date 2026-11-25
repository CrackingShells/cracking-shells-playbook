//! Guard of the durability contract under real process kills (`store.md`, Durability; S3): a
//! write that returned `Ok` survives a SIGKILL of its writer, and the campaign a kill leaves
//! loads. The test re-executes its own binary as the workers (`RDM_CHILD_ROLE`), which run
//! seeded random operations on one campaign and note each acknowledgement in a side file; the
//! parent kills them at random, restarts them, and at the end checks every acknowledgement
//! against the records.
//!
//! Knobs: `RDM_CRASH_ITERS` is the number of kill rounds (default 25, a few seconds; the CI soak
//! sets thousands), `RDM_CRASH_SEED` seeds the kill schedule and the workers' choices,
//! `RDM_CRASH_WORKERS` is the worker count (default 3).

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use common::campaign::*;
use common::{ChildProc, child_role, spawn_child};
use rdm_engine::ids::mint_id;
use rdm_engine::store::{Options, Store};
use rdm_schema::error::StoreError;
use rdm_schema::ids::*;
use rdm_schema::ops::{BatchOp, CommentOp, CreateOp, DeleteOp, Op, RelocateOp, ReplaceOp, Request};
use rdm_schema::stored::*;

const TEST: &str = "killed_workers_lose_no_acknowledged_write";
const DEFAULT_ITERS: u64 = 25;
const DEFAULT_WORKERS: usize = 3;
/// The directories every worker edits: the contended nodes.
const HOT: [u32; 3] = [2, 3, 4];
/// A worker that outlives its parent by this long (a parent killed without cleanup) ends itself.
const WORKER_LIFETIME: Duration = Duration::from_secs(600);

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------------------------
// The campaign

/// ```text
/// demo (1)    hot1 (2), hot2 (3), hot3 (4)
/// ```
/// Workers create leaves under the hot directories and delete them again.
fn base() -> Vec<Node> {
    let n = nid;
    vec![
        dir_node(n(1), "demo", None, ""),
        dir_node(n(2), "hot1", Some(n(1)), "V"),
        dir_node(n(3), "hot2", Some(n(1)), "k"),
        dir_node(n(4), "hot3", Some(n(1)), "t"),
    ]
}

fn open_default(root: &Path) -> Result<Store, StoreError> {
    Store::open(root, &name("demo"), Options::default())
}

// ---------------------------------------------------------------------------------------------
// The worker

/// The side file of a worker: one line per event, written whole with one `write` so a kill
/// leaves at most a last line with no newline, which the parent ignores.
///
/// - `B op kind node...`: about to run `op`, which may touch these nodes
/// - `A op kind a b`: `op` returned `Ok`; `a` and `b` say what it did
/// - `F op`: `op` failed and wrote nothing
struct Acks(std::fs::File);

impl Acks {
    fn line(&mut self, fields: &[&str]) {
        let mut line = fields.join("\t");
        line.push('\n');
        self.0.write_all(line.as_bytes()).unwrap();
    }
}

struct Worker {
    root: PathBuf,
    id: u64,
    incarnation: u64,
    rng: SeededRng,
    acks: Acks,
    counter: u64,
    /// Leaves this incarnation created and has not deleted: id, name, parent.
    owned: Vec<(NodeId, String, NodeId)>,
}

/// A store as it is on disk now. A reader that races a delete can find a file listed and gone;
/// that and nothing else is retried.
fn open_fresh(root: &Path) -> Store {
    loop {
        match open_default(root) {
            Ok(store) => return store,
            Err(StoreError::Io { .. }) => std::thread::sleep(Duration::from_millis(1)),
            Err(other) => panic!("a campaign a kill left must load: {other:?}"),
        }
    }
}

impl Worker {
    fn pick(&mut self, n: usize) -> usize {
        usize::try_from(self.rng.next_u64() % (n as u64)).unwrap()
    }

    fn next_op_id(&mut self) -> String {
        self.counter += 1;
        format!("w{}-i{}-{}", self.id, self.incarnation, self.counter)
    }

    fn unique_name(&mut self, stem: &str) -> String {
        self.counter += 1;
        format!("{stem}{}x{}x{}", self.id, self.incarnation, self.counter)
    }

    fn hot_dir(store: &Store, n: u32) -> Directory {
        match store.campaign().get(&nid(n)).unwrap() {
            Node::Directory(d) => d.clone(),
            Node::Leaf(_) => unreachable!("the hot nodes are directories"),
        }
    }

    /// A replace of a hot directory that adds one token to its `context`: an update that is lost
    /// if another writer's replace lands between this one's revision check and its write.
    fn token_replace(&self, store: &Store, hot: u32, token: &str) -> ReplaceOp {
        let mut dir = Self::hot_dir(store, hot);
        dir.context.push(token.to_owned());
        ReplaceOp {
            expected_rev: store.rev(&nid(hot)).cloned(),
            node: Node::Directory(dir),
        }
    }

    fn new_leaf(&mut self, parent: u32) -> (NodeId, String, CreateOp) {
        let id: NodeId = mint_id(&mut self.rng, |_| false).unwrap();
        let leaf_name = self.unique_name("c");
        let node = leaf_node(id.clone(), &leaf_name, nid(parent), "z");
        (
            id,
            leaf_name,
            CreateOp {
                node,
                position: None,
            },
        )
    }

    /// Runs one random operation and notes its acknowledgement.
    fn one(&mut self) {
        let mut store = open_fresh(&self.root);
        let op_id = self.next_op_id();
        let note = Some("worker");
        let roll = self.pick(100);
        let hot = HOT[self.pick(HOT.len())];
        let kind = if self.owned.len() >= 4 {
            "delete"
        } else {
            match roll {
                0..45 => "tok",
                45..60 => "comment",
                60..75 => "create",
                75..85 if !self.owned.is_empty() => "rename",
                85..92 if !self.owned.is_empty() => "delete",
                92..100 => "batch",
                _ => "tok",
            }
        };
        let request = |note: Option<&str>| header(note, Some(&op_id));
        match kind {
            "tok" => {
                self.acks.line(&["B", &op_id, "tok"]);
                let op = self.token_replace(&store, hot, &op_id);
                let r = store.replace(Request {
                    header: request(note),
                    op,
                });
                self.finish(
                    &op_id,
                    r.map(|_| vec![vec!["tok".into(), nid(hot).to_string(), op_id.clone()]]),
                );
            }
            "comment" => {
                self.acks.line(&["B", &op_id, "comment"]);
                let op = CommentOp {
                    node_id: nid(hot),
                    text: vec![format!("comment {op_id}")],
                };
                let r = store.comment(Request {
                    header: request(None),
                    op,
                });
                self.finish(
                    &op_id,
                    r.map(|_| vec![vec!["comment".into(), nid(hot).to_string(), String::new()]]),
                );
            }
            "create" => {
                let (id, leaf_name, op) = self.new_leaf(hot);
                self.acks.line(&["B", &op_id, "create", id.as_str()]);
                let r = store.create(Request {
                    header: request(note),
                    op,
                });
                let done = r.is_ok();
                self.finish(
                    &op_id,
                    r.map(|_| vec![vec!["create".into(), id.to_string(), leaf_name.clone()]]),
                );
                if done {
                    self.owned.push((id, leaf_name, nid(hot)));
                }
            }
            "rename" => {
                let i = self.pick(self.owned.len());
                let (id, _, parent) = self.owned[i].clone();
                let new_name = self.unique_name("r");
                self.acks.line(&["B", &op_id, "rename", id.as_str()]);
                let op = RelocateOp {
                    id: id.clone(),
                    expected_rev: store.rev(&id).cloned(),
                    parent_id: None,
                    name: Some(name(&new_name)),
                    position: None,
                };
                let r = store.move_node(Request {
                    header: request(note),
                    op,
                });
                let done = r.is_ok();
                self.finish(
                    &op_id,
                    r.map(|_| vec![vec!["rename".into(), id.to_string(), new_name.clone()]]),
                );
                if done {
                    self.owned[i] = (id, new_name, parent);
                }
            }
            "delete" => {
                let i = self.pick(self.owned.len());
                let (id, _, _) = self.owned[i].clone();
                self.acks.line(&["B", &op_id, "delete", id.as_str()]);
                let op = DeleteOp {
                    id: id.clone(),
                    expected_subtree_rev: store.subtree_rev(&id),
                };
                let r = store.delete_subtree(Request {
                    header: request(note),
                    op,
                });
                let done = r.is_ok();
                self.finish(
                    &op_id,
                    r.map(|_| vec![vec!["delete".into(), id.to_string(), String::new()]]),
                );
                if done {
                    self.owned.remove(i);
                }
            }
            _ => {
                // A batch: a create, a token on another hot directory and a comment.
                let other = HOT[self.pick(HOT.len())];
                let (id, leaf_name, create) = self.new_leaf(hot);
                self.acks.line(&["B", &op_id, "batch", id.as_str()]);
                let ops = vec![
                    Op::Create(create),
                    Op::Replace(self.token_replace(&store, other, &op_id)),
                    Op::Comment(CommentOp {
                        node_id: nid(other),
                        text: vec![format!("batch {op_id}")],
                    }),
                ];
                let r = store.batch(Request {
                    header: request(note),
                    op: BatchOp {
                        expected_tree_rev: None,
                        ops,
                        dry_run: false,
                    },
                });
                let done = r.is_ok();
                self.finish(
                    &op_id,
                    r.map(|_| {
                        vec![
                            vec!["create".into(), id.to_string(), leaf_name.clone()],
                            vec!["tok".into(), nid(other).to_string(), op_id.clone()],
                            vec!["comment".into(), nid(other).to_string(), String::new()],
                        ]
                    }),
                );
                if done {
                    self.owned.push((id, leaf_name, nid(hot)));
                }
            }
        }
    }

    /// Notes what an operation did. A stale revision, a lock that stayed busy, and a read that
    /// raced a delete are expected under contention and wrote nothing; anything else is a bug
    /// and ends the worker, which the parent reports.
    fn finish(&mut self, op_id: &str, result: Result<Vec<Vec<String>>, StoreError>) {
        match result {
            Ok(acked) => {
                for a in acked {
                    self.acks.line(&["A", op_id, &a[0], &a[1], &a[2]]);
                }
            }
            Err(e)
                if matches!(
                    e.root(),
                    StoreError::Stale { .. } | StoreError::Locked { .. } | StoreError::Io { .. }
                ) =>
            {
                self.acks.line(&["F", op_id]);
            }
            Err(e) => panic!("worker {} op {op_id}: unexpected {e:?}", self.id),
        }
    }
}

fn run_worker(root: &Path) -> ! {
    // The parent holds our stdin: if it goes away without killing us, stop.
    std::thread::spawn(|| {
        let mut buf = [0u8; 64];
        let mut stdin = std::io::stdin();
        while matches!(stdin.read(&mut buf), Ok(n) if n > 0) {}
        std::process::exit(0);
    });
    std::thread::spawn(|| {
        std::thread::sleep(WORKER_LIFETIME);
        std::process::exit(3);
    });
    let id = env_u64("RDM_KILL_WORKER", 0);
    let incarnation = env_u64("RDM_KILL_INC", 0);
    let acks = std::env::var("RDM_KILL_ACKS").expect("RDM_KILL_ACKS");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(acks)
        .unwrap();
    let mut worker = Worker {
        root: root.to_path_buf(),
        id,
        incarnation,
        rng: SeededRng(env_u64("RDM_KILL_SEED", 0)),
        acks: Acks(file),
        counter: 0,
        owned: Vec::new(),
    };
    loop {
        worker.one();
    }
}

// ---------------------------------------------------------------------------------------------
// The parent

fn spawn_worker(root: &Path, acks: &Path, seed: u64, id: usize, incarnation: u64) -> ChildProc {
    let stream = seed ^ ((id as u64 + 1) << 48) ^ (incarnation.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    spawn_child(
        TEST,
        "worker",
        root,
        &[
            ("RDM_KILL_WORKER", &id.to_string()),
            ("RDM_KILL_INC", &incarnation.to_string()),
            ("RDM_KILL_SEED", &stream.to_string()),
            ("RDM_KILL_ACKS", &acks.display().to_string()),
        ],
    )
}

/// One event of a side file.
#[derive(Debug)]
enum Event {
    Begin {
        op: String,
        nodes: Vec<String>,
    },
    Ack {
        op: String,
        kind: String,
        a: String,
        b: String,
    },
    Failed {
        op: String,
    },
}

/// The complete lines of a side file; a last line a kill cut short is dropped.
fn read_events(path: &Path) -> Vec<Event> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let complete = text.rfind('\n').map_or("", |i| &text[..=i]);
    complete
        .lines()
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            match f[0] {
                "B" => Event::Begin {
                    op: f[1].to_owned(),
                    nodes: f[3..].iter().map(|s| (*s).to_owned()).collect(),
                },
                "A" => Event::Ack {
                    op: f[1].to_owned(),
                    kind: f[2].to_owned(),
                    a: f[3].to_owned(),
                    b: f[4].to_owned(),
                },
                "F" => Event::Failed {
                    op: f[1].to_owned(),
                },
                other => panic!("unknown event {other}"),
            }
        })
        .collect()
}

fn kill_and_restart(
    procs: &mut [ChildProc],
    incarnations: &mut [u64],
    i: usize,
    root: &Path,
    acks_dir: &Path,
    seed: u64,
) {
    procs[i].kill_and_reap();
    incarnations[i] += 1;
    let fresh = spawn_worker(
        root,
        &acks_dir.join(format!("w{i}.log")),
        seed,
        i,
        incarnations[i],
    );
    // The old handle is dropped (and reaped again, harmlessly) as it is replaced.
    procs[i] = fresh;
}

/// Guards that every write acknowledged before its worker was killed is in the campaign
/// afterwards, that the campaign a kill leaves loads with no damage it did not start with, and
/// that recovery clears the scratch area.
///
/// Bugs it catches: a lock released between the revision check and the write (two writers both
/// pass the check and one overwrites the other: an acknowledged token is missing); an
/// acknowledgement sent before the data is published; a write that a kill leaves half done and
/// recovery cannot finish (a created node with no way to complete it, damage on load).
#[test]
fn killed_workers_lose_no_acknowledged_write() {
    if let Some((role, root)) = child_role() {
        assert_eq!(role, "worker");
        run_worker(&root);
    }

    let iters = env_u64("RDM_CRASH_ITERS", DEFAULT_ITERS);
    let seed = env_u64("RDM_CRASH_SEED", 0xC0FFEE);
    let workers = usize::try_from(env_u64("RDM_CRASH_WORKERS", DEFAULT_WORKERS as u64))
        .unwrap()
        .max(1);
    let disk = Disk::with(&base());
    let acks_dir = disk._tmp.path().join("acks");
    std::fs::create_dir_all(&acks_dir).unwrap();
    let initial = diagnostic_keys(&open_default(&disk.root).unwrap());

    // The handles kill and reap their child when dropped, so a panic below leaves nothing running.
    let mut incarnations = vec![0u64; workers];
    let mut procs: Vec<ChildProc> = (0..workers)
        .map(|i| spawn_worker(&disk.root, &acks_dir.join(format!("w{i}.log")), seed, i, 0))
        .collect();
    let mut rng = SeededRng(seed);
    for round in 0..iters {
        std::thread::sleep(Duration::from_millis(30 + rng.next_u64() % 120));
        for (i, p) in procs.iter_mut().enumerate() {
            if let Some(status) = p.child.try_wait().unwrap() {
                panic!("round {round}: worker {i} ended by itself with {status}");
            }
        }
        // One round in six kills every worker at once.
        let everyone = rng.next_u64() % 6 < 1;
        let victims: Vec<usize> = if everyone {
            (0..workers).collect()
        } else {
            vec![usize::try_from(rng.next_u64() % workers as u64).unwrap()]
        };
        for v in victims {
            kill_and_restart(
                &mut procs,
                &mut incarnations,
                v,
                &disk.root,
                &acks_dir,
                seed,
            );
        }
    }
    std::thread::sleep(Duration::from_millis(30 + rng.next_u64() % 120));
    for p in &mut procs {
        p.kill_and_reap();
    }

    // The campaign as the kills left it, then as recovery leaves it.
    let mut store = open_default(&disk.root).expect("the campaign a kill left loads");
    assert_no_new_diagnostics("after the kills", &initial, &store);
    store.recover().unwrap();
    assert!(
        scratch_is_empty(&disk.root),
        ".rdm/tmp or .rdm/intent is not empty after recovery"
    );
    let store = open_default(&disk.root).unwrap();
    assert_no_new_diagnostics("after recovery", &initial, &store);

    check_acks(&store, &acks_dir, workers);
}

fn diagnostic_keys(store: &Store) -> BTreeSet<(String, rdm_schema::error::Rule)> {
    store
        .diagnostics()
        .iter()
        .map(|d| (d.file.clone(), d.rule))
        .collect()
}

fn assert_no_new_diagnostics(
    when: &str,
    initial: &BTreeSet<(String, rdm_schema::error::Rule)>,
    store: &Store,
) {
    let now = diagnostic_keys(store);
    let new: Vec<_> = now.difference(initial).collect();
    assert!(
        new.is_empty(),
        "{when}: the campaign shows damage it did not start with: {new:?}"
    );
}

/// Every acknowledged operation is in the records, and what the acknowledged operations of one
/// worker did to its own nodes is what the campaign holds.
fn check_acks(store: &Store, acks_dir: &Path, workers: usize) {
    let by_op: BTreeMap<&OpId, &Amendment> = store
        .amendments()
        .values()
        .filter_map(|a| Some((a.op_id.as_ref()?, a)))
        .collect();
    let comments: BTreeSet<(String, String)> = store
        .comments()
        .values()
        .filter_map(|c| Some((c.op_id.as_ref()?.to_string(), c.node_id.to_string())))
        .collect();
    let mut tokens: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for n in HOT {
        if let Some(Node::Directory(d)) = store.campaign().get(&nid(n)) {
            tokens.insert(nid(n).to_string(), d.context.iter().cloned().collect());
        }
    }

    let mut acknowledged = 0usize;
    for w in 0..workers {
        let events = read_events(&acks_dir.join(format!("w{w}.log")));
        let finished: BTreeSet<&str> = events
            .iter()
            .filter_map(|e| match e {
                Event::Ack { op, .. } | Event::Failed { op } => Some(op.as_str()),
                Event::Begin { .. } => None,
            })
            .collect();
        // Nodes an operation was cut in the middle of: either outcome is allowed for them.
        let unknown: BTreeSet<&str> = events
            .iter()
            .filter_map(|e| match e {
                Event::Begin { op, nodes } if !finished.contains(op.as_str()) => Some(nodes),
                _ => None,
            })
            .flatten()
            .map(String::as_str)
            .collect();
        // What the worker's acknowledged operations did to its nodes: present, and named so.
        let mut expected: BTreeMap<&str, Option<&str>> = BTreeMap::new();
        for e in &events {
            let Event::Ack { op, kind, a, b } = e else {
                continue;
            };
            acknowledged += 1;
            let op_id = OpId::parse(op).unwrap();
            let changed = |want: ChangeOp| {
                by_op.get(&op_id).is_some_and(|am| {
                    am.changes
                        .iter()
                        .any(|c| c.node_id.as_str() == a && c.op == want)
                })
            };
            match kind.as_str() {
                "tok" => {
                    assert!(
                        by_op.contains_key(&op_id),
                        "acknowledged {op}: no amendment carries it"
                    );
                    assert!(
                        tokens.get(a).is_some_and(|t| t.contains(b)),
                        "acknowledged {op}: its token is not in {a} (a lost update)"
                    );
                }
                "comment" => assert!(
                    comments.contains(&(op.clone(), a.clone())),
                    "acknowledged {op}: no comment on {a} carries it"
                ),
                "create" => {
                    assert!(
                        changed(ChangeOp::Add),
                        "acknowledged {op}: no amendment adds {a}"
                    );
                    expected.insert(a, Some(b));
                }
                "rename" => {
                    assert!(
                        changed(ChangeOp::Rename),
                        "acknowledged {op}: no amendment renames {a}"
                    );
                    expected.insert(a, Some(b));
                }
                "delete" => {
                    assert!(
                        changed(ChangeOp::Remove),
                        "acknowledged {op}: no amendment removes {a}"
                    );
                    expected.insert(a, None);
                }
                other => panic!("unknown kind {other}"),
            }
        }
        for (id, want) in expected {
            if unknown.contains(id) {
                continue;
            }
            let held = NodeId::parse(id)
                .ok()
                .and_then(|n| store.campaign().get(&n))
                .map(|n| n.name().to_string());
            assert_eq!(
                held.as_deref(),
                want,
                "worker {w}: node {id} is not as its acknowledged operations left it"
            );
        }
    }
    assert!(
        acknowledged > 0,
        "no worker acknowledged anything: the test checked nothing"
    );
}
