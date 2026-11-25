//! Campaigns for the crash guards: node builders, a seeded generator and clock for the store's
//! own ULIDs, and whole-directory snapshots. Node files are written with the codec, the real
//! producer of node files; everything after that is written by the store itself.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use rdm_engine::codec::encode;
use rdm_engine::fs::Fs;
use rdm_engine::ids::IdRng;
use rdm_engine::store::Options;
use rdm_schema::ids::*;
use rdm_schema::ops::{EditNote, Header};
use rdm_schema::stored::*;
use std::sync::Arc;

const ALPHABET: &str = "0123456789abcdefghjkmnpqrstvwxyz";

/// A deterministic id body of `len` characters for `n`.
pub fn body(mut n: u32, len: usize) -> String {
    let a = ALPHABET.as_bytes();
    let mut s = vec![b'0'; len];
    for i in (0..len).rev() {
        s[i] = a[(n % 32) as usize];
        n /= 32;
    }
    String::from_utf8(s).unwrap()
}

pub fn nid(n: u32) -> NodeId {
    NodeId::parse(&format!("n_{}", body(n, 8))).unwrap()
}

pub fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

pub fn file_of(id: &NodeId) -> String {
    format!("nodes/{id}.json")
}

/// The ids of the criteria and the step of a node are derived from its own id, so they are
/// unique wherever the node id is.
fn item(prefix: &str, lead: char, id: &NodeId) -> String {
    let b = id.as_str();
    format!("{prefix}{lead}{}", &b[b.len() - 5..])
}

fn criterion(lead: char, id: &NodeId) -> Criterion {
    Criterion {
        id: CriterionId::parse(&item("c_", lead, id)).unwrap(),
        text: "criterion".into(),
        met: false,
        extra: Default::default(),
    }
}

/// A directory (`parent` given) or the campaign root (`parent` none).
pub fn dir_node(id: NodeId, nm: &str, parent: Option<NodeId>, order: &str) -> Node {
    Node::Directory(Directory {
        schema_version: "0.1.0".into(),
        kind: if parent.is_some() {
            DirectoryKind::Directory
        } else {
            DirectoryKind::Campaign
        },
        name: name(nm),
        order: parent.as_ref().map(|_| OrderKey::new(order)),
        parent_id: parent,
        title: format!("Directory {nm}"),
        context: vec!["context".into()],
        references: vec![],
        goal: "a goal".into(),
        preconditions: vec![criterion('p', &id)],
        success_gates: vec![criterion('q', &id)],
        gotchas: vec![],
        notes: vec![],
        marks: vec![],
        extra: Default::default(),
        id,
    })
}

pub fn leaf_node(id: NodeId, nm: &str, parent: NodeId, order: &str) -> Node {
    Node::Leaf(Leaf {
        schema_version: "0.1.0".into(),
        kind: LeafKind::Leaf,
        name: name(nm),
        parent_id: parent,
        order: OrderKey::new(order),
        title: format!("Leaf {nm}"),
        status: Status::Planned,
        status_reason: None,
        goal: "a goal".into(),
        preconditions: vec![criterion('p', &id)],
        success_gates: vec![criterion('q', &id)],
        references: vec![Reference {
            label: None,
            title: "design".into(),
            target: Some("__design__/x.md".into()),
            note: None,
            node_id: None,
            extra: Default::default(),
        }],
        steps: vec![Step {
            id: StepId::parse(&item("s_", 'r', &id)).unwrap(),
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
        id,
    })
}

/// Writes `nodes` as the campaign `demo` under a fresh temporary directory.
pub struct Disk {
    pub _tmp: tempfile::TempDir,
    pub root: PathBuf,
}

impl Disk {
    pub fn with(nodes: &[Node]) -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("demo");
        std::fs::create_dir_all(root.join("nodes")).unwrap();
        for n in nodes {
            std::fs::write(root.join(file_of(n.id())), encode(n)).unwrap();
        }
        Self { _tmp: tmp, root }
    }
}

pub fn header(note: Option<&str>, op_id: Option<&str>) -> Header {
    Header {
        op_id: op_id.map(|o| OpId::parse(o).unwrap()),
        actor: Actor {
            kind: ActorKind::Agent,
            name: Some("crash-test".into()),
            extra: Default::default(),
        },
        note: note.map(|d| EditNote {
            description: d.into(),
            source: None,
            correction: false,
        }),
    }
}

/// A SplitMix64 stream: the seeded generator the store draws its ULIDs from.
pub struct SeededRng(pub u64);

impl SeededRng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

impl IdRng for SeededRng {
    fn fill(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let word = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
    }
}

/// Store options with a seeded generator and a fixed clock: two runs of one operation write the
/// same bytes. `fs` replaces the real file system when given.
pub fn seeded_options(seed: u64, fs: Option<Arc<dyn Fs + Send + Sync>>) -> Options {
    let mut options = Options {
        rng: Box::new(SeededRng(seed)),
        clock: Box::new(|| SystemTime::UNIX_EPOCH + Duration::from_secs(1_800_000_000)),
        ..Options::default()
    };
    if let Some(fs) = fs {
        options.fs = fs;
    }
    options
}

pub fn copy_tree(src: &Path, dst: &Path) {
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

/// Every file of a campaign directory except the scratch area, by relative path.
pub type Files = BTreeMap<String, Vec<u8>>;

pub fn files_of(root: &Path) -> Files {
    fn walk(dir: &Path, base: &Path, out: &mut Files) {
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
    let mut out = Files::new();
    walk(root, root, &mut out);
    out
}

/// Nothing is left in `.rdm/tmp` or `.rdm/intent`.
pub fn scratch_is_empty(root: &Path) -> bool {
    ["intent", "tmp"].iter().all(|d| {
        std::fs::read_dir(root.join(".rdm").join(d))
            .map(|e| e.count() == 0)
            .unwrap_or(true)
    })
}
