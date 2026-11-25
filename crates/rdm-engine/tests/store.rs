//! Guards of the campaign store: open and validate, mint, revisions, the primitives.
//!
//! A campaign on disk is built by `Disk`, which writes canonical files with the codec, the real
//! producer of node files. The damage a merge leaves is made by what makes it: `git merge` of two
//! branches, a `cp`, an editor's rewrite of one file.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use common::campaign::*;
use common::hooked::{frozen_stat, no_hard_links, options_with, race_at_link};
use common::soak_cases;
use rdm_engine::cache::CachedCampaign;
use rdm_engine::codec::encode;
use rdm_engine::fs::{Durability, Effect, RealFs, RecordingFs};
use rdm_engine::ids::{IdRng, MAX_COLLISIONS, MintError, mint_id};
use rdm_engine::intent::{self, INTENT_FORMAT, Intent, IntentEffect};
use rdm_engine::lock::WriterLock;
use rdm_engine::store::Options;
use rdm_schema::error::{Current, Rule, StoreError};
use rdm_schema::ids::*;
use rdm_schema::links::RecordRef;
use rdm_schema::ops::{
    BatchOp, CommentOp, CreateOp, DeleteOp, Header, Op, Position, RelocateOp, ReplaceOp, Request,
};
use rdm_schema::stored::*;
use serde_json::{Value, json};

// ---------------------------------------------------------------------------------------------
// Building campaigns

impl Disk {
    fn open(&self) -> Result<CachedCampaign, StoreError> {
        CachedCampaign::open(&self.root, &name("demo"), Options::default())
    }
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

fn rules(store: &mut CachedCampaign) -> Vec<(String, Rule)> {
    store
        .read()
        .unwrap()
        .diagnostics()
        .iter()
        .map(|d| (d.file.clone(), d.rule))
        .collect()
}

fn refused(r: Result<CachedCampaign, StoreError>) -> (String, Rule) {
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

fn validated(store: &mut CachedCampaign) -> BTreeSet<NodeId> {
    let campaign = store.read().unwrap().campaign();
    campaign
        .nodes()
        .map(|n| n.id().clone())
        .filter(|id| campaign.is_validated(id))
        .collect()
}

#[test]
fn open_names_invalid_campaigns() {
    // A sound campaign opens clean and is entirely validated.
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    assert_eq!(
        store.read().unwrap().diagnostics(),
        &[],
        "a sound campaign reports nothing"
    );
    assert_eq!(validated(&mut store).len(), 5);
    assert_eq!(
        store.read().unwrap().campaign().path(&nid(3)).as_deref(),
        Some("core/alpha")
    );
    assert_eq!(store.read().unwrap().campaign().depth(&nid(3)), Some(1));
    assert_eq!(store.read().unwrap().campaign().depth(&nid(2)), Some(0));
    assert_eq!(
        store.read().unwrap().campaign().resolve("core/beta"),
        Some(&nid(4))
    );

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
    let mut store = disk.open().unwrap();
    assert_eq!(
        store.read().unwrap().diagnostics(),
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
    let mut renamed = CachedCampaign::open(&disk.root, &name("other"), Options::default()).unwrap();
    assert_eq!(renamed.read().unwrap().campaign().nodes().count(), 5);
    assert_eq!(validated(&mut renamed), BTreeSet::new());
    assert_eq!(renamed.read().unwrap().campaign().root(), None);
    assert_eq!(
        rules(&mut renamed),
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
        vec![Edit::new(leaf_n(6, "gamma", 2, "s"))],
    );
    let mut store = disk.open().unwrap();
    assert_eq!(
        rules(&mut store),
        vec![
            (file(3), Rule::ParentMissing),
            (file(4), Rule::ParentMissing),
            (file(6), Rule::ParentMissing)
        ]
    );
    assert_eq!(validated(&mut store), BTreeSet::from([nid(1), nid(5)]));
    assert_eq!(
        store.read().unwrap().campaign().nodes().count(),
        5,
        "the orphans are loaded"
    );
    assert_eq!(store.read().unwrap().campaign().path(&nid(3)), None);

    // One branch makes a directory out of a leaf's id while the other adds a child under it.
    let disk = merged(
        vec![Edit::new(leaf_n(5, "spare", 1, "k"))],
        vec![Edit::new(leaf_n(6, "gamma", 5, "s"))],
    );
    let mut store = disk.open().unwrap();
    assert_eq!(rules(&mut store), vec![(file(6), Rule::ParentNotDirectory)]);
    assert_eq!(
        validated(&mut store),
        BTreeSet::from([nid(1), nid(2), nid(3), nid(4), nid(5)])
    );

    // Two branches each move one of two directories under the other: a cycle.
    let disk = merged(
        vec![Edit::new(dir_n(2, "core", Some(5), "V"))],
        vec![Edit::new(dir_n(5, "spare", Some(2), "V"))],
    );
    let mut store = disk.open().unwrap();
    assert_eq!(
        rules(&mut store),
        vec![(file(2), Rule::Cycle)],
        "one diagnostic for the cycle"
    );
    assert_eq!(
        validated(&mut store),
        BTreeSet::from([nid(1)]),
        "the cycle and everything below it is unvalidated"
    );
    assert_eq!(store.read().unwrap().campaign().nodes().count(), 5);

    // Two branches each add a node of one name under one parent: the files differ, so git sees no
    // conflict, and the colliding siblings and their subtrees are unvalidated.
    let disk = merged(
        vec![Edit::new(leaf_n(6, "gamma", 2, "s"))],
        vec![Edit::new(leaf_n(7, "gamma", 2, "t"))],
    );
    let mut store = disk.open().unwrap();
    assert_eq!(rules(&mut store), vec![(file(6), Rule::NameCollision)]);
    assert_eq!(
        validated(&mut store),
        BTreeSet::from([nid(1), nid(2), nid(3), nid(4), nid(5)])
    );

    // Two branches each add a campaign root: the two roots are one defect, no tree is picked,
    // and every node is unvalidated until a root is renamed.
    let disk = merged(vec![Edit::new(dir_n(8, "demo", None, ""))], vec![]);
    let mut store = disk.open().unwrap();
    assert_eq!(rules(&mut store), vec![(file(1), Rule::RootCount)]);
    assert_eq!(
        store.read().unwrap().campaign().root(),
        None,
        "two roots, no root"
    );
    assert_eq!(validated(&mut store), BTreeSet::new());
    assert!(!store.read().unwrap().campaign().is_validated(&nid(8)));
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
        .map(|(i, c)| ID_ALPHABET.find(c).unwrap() as u8 | [0xE0, 0x20, 0x80, 0x40][i % 4])
        .collect()
}

#[test]
fn minting_retries_on_collision() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    let campaign = store.read().unwrap().campaign();
    let taken = |id: &NodeId| campaign.get(id).is_some();

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

fn replace_req(h: Header, expected: Option<&Rev>, node: Node) -> Request<ReplaceOp> {
    Request {
        header: h,
        op: ReplaceOp {
            expected_rev: expected.cloned(),
            node,
        },
    }
}

/// The first 16 hex digits of SHA-256 over the file without `schema_version`: `jq` is the
/// independent canonicaliser; the hash is the `sha2` crate's, since a stock Windows runner has
/// no `shasum`.
fn sha16_of_jq(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use std::io::Write;
    let mut jq = Command::new("jq")
        .args(["del(.schema_version)"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    jq.stdin.take().unwrap().write_all(bytes).unwrap();
    // A native jq.exe writes its text output with CRLF line ends; the canonical encoding has
    // none (a carriage return inside a string is escaped), so dropping them recovers jq's bytes.
    let out: Vec<u8> = jq
        .wait_with_output()
        .unwrap()
        .stdout
        .into_iter()
        .filter(|&b| b != b'\r')
        .collect();
    let hex: String = Sha256::digest(&out)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    hex[..16].to_owned()
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
            .read()
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
    // Bug caught: a write judged against what the cache holds instead of the disk. The cache `a`
    // holds the old revision of node 3 while the disk has moved on, as two actors leave it: another
    // handle replaced the node through the store, or an editor wrote in place at the same size and
    // the signature hides it (a coarse-timestamp file system), which only the write's own read
    // from the disk can see. The stale `expected_rev` would match the entry and the write would go
    // over the other actor's edit.
    type Make = fn(&Disk) -> (CachedCampaign, Rev);
    let rows: [(&str, Make, &str); 2] = [
        (
            "another handle replaced the node",
            |disk| {
                let mut a = disk.open().unwrap();
                let mut b = disk.open().unwrap();
                let old = a.read().unwrap().rev(&nid(3)).unwrap().clone();
                let mut changed = leaf_of(&mut b, 3);
                changed.goal = "b was here".into();
                b.replace(replace_req(
                    header(Some("b edits"), None),
                    Some(&old),
                    Node::Leaf(changed),
                ))
                .unwrap();
                (a, old)
            },
            "b was here",
        ),
        (
            "an editor wrote in place at the same size",
            |disk| {
                age(&disk.root);
                let mut a =
                    CachedCampaign::open(&disk.root, &name("demo"), options_with(frozen_stat()))
                        .unwrap();
                let old = a.read().unwrap().rev(&nid(3)).unwrap().clone();
                rewrite_in_place(&disk.path(&file(3)), "a goal", "b goal");
                assert_eq!(
                    leaf_of(&mut a, 3).goal,
                    "a goal",
                    "the signature hides the edit from a read (the documented limit)"
                );
                (a, old)
            },
            "b goal",
        ),
    ];
    for (row, make, goal) in rows {
        let disk = Disk::new();
        disk.write_all(&base());
        let (mut a, old) = make(&disk);
        let fresh = disk
            .open()
            .unwrap()
            .read()
            .unwrap()
            .rev(&nid(3))
            .unwrap()
            .clone();
        assert_ne!(fresh, old, "{row}: the disk moved on");
        let before = files_of(&disk.root);

        // `a` still holds the old revision in memory; the write is judged against the disk.
        let mut mine = leaf_of(&mut a, 3);
        mine.goal = "a was here".into();
        let Err(err) = a.replace(replace_req(
            header(Some("a edits"), None),
            Some(&old),
            Node::Leaf(mine),
        )) else {
            panic!("{row}: the stale replace was accepted");
        };
        match err {
            StoreError::Stale {
                current: Current::Node { id, rev },
            } => assert_eq!((id, rev), (nid(3), fresh), "{row}"),
            other => panic!("{row}: want Stale, got {other:?}"),
        }
        assert_eq!(
            files_of(&disk.root),
            before,
            "{row}: a stale replace writes no file"
        );
        assert_eq!(
            std::fs::read_dir(disk.path(".rdm/intent"))
                .map(|d| d.count())
                .unwrap_or(0),
            0,
            "{row}"
        );
        assert_eq!(
            leaf_of(&mut a, 3).goal,
            goal,
            "{row}: the write learnt the file it read"
        );
    }
}

#[test]
fn replayed_op_id_writes_nothing() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    let old = store.read().unwrap().rev(&nid(3)).unwrap().clone();
    let mut changed = leaf_of(&mut store, 3);
    changed.goal = "retry me".into();
    let req = replace_req(
        header(Some("edit"), Some("op-1")),
        Some(&old),
        Node::Leaf(changed),
    );
    let first = store.replace(req.clone()).unwrap();
    assert!(!first.replayed && first.amendment.is_some());
    let after_first = files_of(&disk.root);

    // The same request again: its `expected_rev` is now stale, and it must not matter.
    let again = store.replace(req.clone()).unwrap();
    assert!(again.replayed);
    assert_eq!(
        (again.result.clone(), again.amendment.clone()),
        (first.result.clone(), first.amendment.clone())
    );
    assert_eq!(files_of(&disk.root), after_first, "a replay writes nothing");

    // A new process learns the op from the files.
    let mut reopened = disk.open().unwrap();
    let third = reopened.replace(req).unwrap();
    assert!(third.replayed);
    assert_eq!(third.amendment, first.amendment);
    assert_eq!(files_of(&disk.root), after_first);
}

#[test]
fn requests_breaking_a_rule_are_refused() {
    let mut nodes = base();
    nodes.push(leaf_n(6, "orphan", 99, "V"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let mut store = disk.open().unwrap();
    let before = files_of(&disk.root);
    let rev3 = store.read().unwrap().rev(&nid(3)).unwrap().clone();
    let rev2 = store.read().unwrap().rev(&nid(2)).unwrap().clone();
    let rev6 = store.read().unwrap().rev(&nid(6)).unwrap().clone();

    let mut edit = |f: &dyn Fn(&mut Leaf)| {
        let mut l = leaf_of(&mut store, 3);
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
    let ref_to = |n: u32| Reference {
        label: None,
        title: "t".into(),
        target: None,
        note: None,
        node_id: Some(nid(n)),
        extra: Default::default(),
    };
    let dangling_in_step = edit(&|l| l.steps[0].references.push(ref_to(78)));
    let mut spare = match store.read().unwrap().campaign().get(&nid(5)).unwrap() {
        Node::Directory(d) => d.clone(),
        other => panic!("node 5 is a directory, got {other:?}"),
    };
    spare.references.push(ref_to(79));
    let rev5 = store.read().unwrap().rev(&nid(5)).unwrap().clone();
    let mut as_leaf = leaf_of(&mut store, 3);
    as_leaf.id = nid(2);
    as_leaf.parent_id = nid(1);
    as_leaf.name = name("core");
    as_leaf.order = OrderKey::new("V");
    let mut orphan = leaf_of(&mut store, 3);
    orphan.id = nid(6);
    orphan.name = name("orphan");
    orphan.parent_id = nid(99);
    orphan.goal = "x".into();
    let mut missing = leaf_of(&mut store, 3);
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
                Node::Leaf(leaf_of(&mut store, 3)),
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
            "a step reference to a missing node",
            replace_req(header(note, None), Some(&rev3), dangling_in_step),
            file(3),
            Rule::DanglingId,
        ),
        (
            "a directory reference to a missing node",
            replace_req(header(note, None), Some(&rev5), Node::Directory(spare)),
            file(5),
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
        assert_eq!(files_of(&disk.root), before, "{row}: nothing was written");
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
    let mut done = leaf_of(&mut store, 3);
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
    assert!(leaf_of(&mut store, 3).steps[0].done);
    let rev3 = state_only.result.rev;
    let content = store
        .replace(replace_req(header(note, None), Some(&rev3), edited))
        .unwrap();
    assert!(content.amendment.is_some());
    assert_eq!(store.read().unwrap().amendments().len(), 1);
    assert_eq!(disk.open().unwrap().read().unwrap().amendments().len(), 1);
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

fn ids_of(store: &mut CachedCampaign) -> BTreeSet<NodeId> {
    store
        .read()
        .unwrap()
        .campaign()
        .nodes()
        .map(|n| n.id().clone())
        .collect()
}

fn create_op(node: Node, position: Option<Position>) -> CreateOp {
    CreateOp { node, position }
}

fn relocate_op(
    store: &mut CachedCampaign,
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
    nodes.push(dir_n(10, "inner", Some(2), "t"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let rec = Arc::new(RecordingFs::new(&disk.root));
    let mut store = CachedCampaign::open(
        &disk.root,
        &name("demo"),
        Options {
            fs: rec.clone(),
            ..Options::default()
        },
    )
    .unwrap();
    let ids = ids_of(&mut store);

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
        let before = files_of(&disk.root);
        let op = relocate_op(&mut store, n, new_name, parent, position);
        let skip = rec.effects().len();
        let done = store.move_node(req(Some("tidy"), None, op)).unwrap();
        let after = files_of(&disk.root);
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
        assert_eq!(ids_of(&mut store), ids, "no id changes");
        assert_eq!(ids_of(&mut disk.open().unwrap()), ids);
        assert_eq!(
            store.read().unwrap().campaign().path(&nid(n)).as_deref(),
            Some(path),
            "the path follows the moved node"
        );
        assert!(store.read().unwrap().campaign().is_validated(&nid(n)));
    }
    // What the amendment records: a rename, then a move.
    let first = store.read().unwrap().amendments().values().next().unwrap();
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
    // The tree and subtree domains, against hex computed outside the code over the files the
    // codec wrote for `base()` (a rev is the first 16 hex digits of SHA-256 of the file without
    // `schema_version`; the tree line is `<id> <rev>`):
    //   cd <campaign>/nodes
    //   for f in n_*.json; do echo "${f%.json} $(jq 'del(.schema_version)' $f | shasum -a 256 | cut -c1-16)"; done > revs
    //   (printf 'rdm-tree\n'; cat revs) | shasum -a 256 | cut -c1-16                          # fc89487f2b437ecf
    //   (printf 'rdm-subtree\n'; grep -E 'n_0000000[234]' revs) | shasum -a 256 | cut -c1-16  # a2193ef8f52ef79a
    assert_eq!(
        store.read().unwrap().tree_rev().as_str(),
        "fc89487f2b437ecf"
    );
    assert_eq!(
        store.read().unwrap().subtree_rev(&nid(2)).unwrap().as_str(),
        "a2193ef8f52ef79a"
    );
    let mut seen = BTreeSet::from([store.read().unwrap().tree_rev()]);

    let mut changed = leaf_of(&mut store, 3);
    changed.goal = "edited".into();
    type TreeOp = Box<dyn Fn(&mut CachedCampaign) -> TreeRev>;
    let ops: Vec<(&str, TreeOp)> = vec![
        (
            "create",
            Box::new(|s| {
                let n = leaf_n(20, "newleaf", 2, "V");
                s.create(req(Some("n"), None, create_op(n, None)))
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
                s.move_node(req(Some("m"), None, op)).unwrap().tree_rev
            }),
        ),
        (
            "delete",
            Box::new(|s| {
                let op = delete_op(s, 5);
                s.delete_subtree(req(Some("d"), None, op)).unwrap().tree_rev
            }),
        ),
        (
            "batch",
            Box::new(|s| {
                let n = leaf_n(21, "batched", 2, "V");
                let b = BatchOp {
                    expected_tree_rev: None,
                    ops: vec![Op::Create(create_op(n, None))],
                    dry_run: false,
                };
                s.batch(req(Some("b"), None, b)).unwrap().tree_rev
            }),
        ),
    ];
    for (row, op) in ops {
        let before = store.read().unwrap().tree_rev();
        let after = op(&mut store);
        assert_ne!(after, before, "{row} changes the tree_rev");
        assert!(
            seen.insert(after.clone()),
            "{row}: a tree_rev never comes back"
        );
        assert_eq!(
            store.read().unwrap().tree_rev(),
            after,
            "{row}: the receipt names the tree_rev the store holds"
        );
        assert_eq!(
            disk.open().unwrap().read().unwrap().tree_rev(),
            after,
            "{row}: a cold open agrees; the amendment is not counted"
        );
    }
    // A comment writes a file and changes no rev.
    let before = store.read().unwrap().tree_rev();
    let c = CommentOp {
        node_id: nid(3),
        text: vec!["noted".into()],
    };
    let done = store.comment(req(None, None, c)).unwrap();
    assert_eq!(done.tree_rev, before);
    assert_eq!(disk.open().unwrap().read().unwrap().tree_rev(), before);
    assert_eq!(store.read().unwrap().comments().len(), 1);
}

#[test]
fn failed_batch_writes_nothing() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    let before = files_of(&disk.root);
    let good = |store: &mut CachedCampaign| {
        let mut edited = leaf_of(store, 3);
        edited.goal = "batch edit".into();
        vec![
            Op::Create(create_op(leaf_n(20, "fresh", 2, "V"), None)),
            Op::Replace(ReplaceOp {
                expected_rev: Some(rev_of(store, 3)),
                node: Node::Leaf(edited),
            }),
        ]
    };

    // The third op breaks a rule: the first two are not written.
    let mut ops = good(&mut store);
    ops.push(Op::Create(create_op(leaf_n(21, "alpha", 2, "V"), None)));
    let err = store
        .batch(req(
            Some("b"),
            None,
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
    assert_eq!(
        files_of(&disk.root),
        before,
        "a failed batch writes nothing"
    );
    assert!(
        store.read().unwrap().campaign().get(&nid(20)).is_none(),
        "and the store holds nothing of it"
    );

    // A stale tree_rev is Stale and writes nothing.
    let stale = disk.open().unwrap().read().unwrap().tree_rev();
    let rev3 = rev_of(&mut store, 3);
    let mut l = leaf_of(&mut store, 3);
    l.steps[0].done = false;
    l.status = Status::Planned;
    l.progress.notes = vec!["p".into()];
    store
        .replace(replace_req(header(None, None), Some(&rev3), Node::Leaf(l)))
        .unwrap();
    let before = files_of(&disk.root);
    let ops = good(&mut store);
    let err = store
        .batch(req(
            Some("b"),
            None,
            BatchOp {
                expected_tree_rev: Some(stale),
                ops,
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
    assert_eq!(files_of(&disk.root), before);

    // A dry run checks everything and writes nothing; the same batch then writes one amendment.
    let ops = good(&mut store);
    let dry = store
        .batch(req(
            Some("b"),
            None,
            BatchOp {
                expected_tree_rev: None,
                ops,
                dry_run: true,
            },
        ))
        .unwrap();
    assert_eq!(dry.result.len(), 2);
    assert_eq!(files_of(&disk.root), before, "a dry run writes nothing");
    let ops = good(&mut store);
    let real = store
        .batch(req(
            Some("b"),
            None,
            BatchOp {
                expected_tree_rev: None,
                ops,
                dry_run: false,
            },
        ))
        .unwrap();
    let after = files_of(&disk.root);
    assert_eq!(
        new_files(&before, &after, "_amendments/"),
        1,
        "one amendment for the whole batch"
    );
    assert_eq!(nodes_changed(&before, &after), vec![file(3), file(20)]);
    let amendment = &store.read().unwrap().amendments()[real.amendment.as_ref().unwrap()];
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
    nodes.push(dir_n(6, "inner", Some(5), "V"));
    nodes.push(with_reference(leaf_n(7, "deep", 6, "V"), 6));
    nodes[2] = with_reference(nodes[2].clone(), 7);
    let disk = Disk::new();
    disk.write_all(&nodes);
    let mut store = disk.open().unwrap();
    store
        .comment(req(
            None,
            None,
            CommentOp {
                node_id: nid(7),
                text: vec!["watch".into()],
            },
        ))
        .unwrap();
    let before = files_of(&disk.root);

    let op = delete_op(&mut store, 5);
    let done = store
        .delete_subtree(req(Some("drop spare"), None, op))
        .unwrap();
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
    let after = files_of(&disk.root);
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
        store.read().unwrap().comments().len(),
        1,
        "a comment on a removed node stays"
    );
    let amendment = &store.read().unwrap().amendments()[done.amendment.as_ref().unwrap()];
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
    assert!(rules(&mut store).contains(&(file(3), Rule::DanglingReference)));
    assert_eq!(ids_of(&mut store), ids_of(&mut disk.open().unwrap()));
}

/// The published names an operation leaves, in order: the links and renames into a published
/// name, and the unlinks of one.
fn published(effects: &[Effect]) -> Vec<String> {
    let visible = |p: &Path| !p.starts_with(".rdm/tmp") && !p.starts_with(".rdm/.gitignore");
    // The logical name, `/`-joined, whatever the platform spells a path with.
    let logical = |p: &Path| p.to_string_lossy().replace('\\', "/");
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Link { link: p, .. } | Effect::Rename { to: p, .. } if visible(p) => {
                Some(format!("+{}", logical(p)))
            }
            Effect::Remove { path } if visible(path) => Some(format!("-{}", logical(path))),
            _ => None,
        })
        .collect()
}

#[test]
fn write_order_follows_the_three_rules() {
    let mut nodes = base();
    nodes.push(leaf_n(6, "child", 5, "V"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let rec = Arc::new(RecordingFs::new(&disk.root));
    let mut store = CachedCampaign::open(
        &disk.root,
        &name("demo"),
        Options {
            fs: rec.clone(),
            ..Options::default()
        },
    )
    .unwrap();
    let skip = rec.effects().len();
    let mut edited = leaf_of(&mut store, 4);
    edited.goal = "edit".into();
    let ops = vec![
        Op::Delete(delete_op(&mut store, 5)),
        Op::Replace(ReplaceOp {
            expected_rev: Some(rev_of(&mut store, 4)),
            node: Node::Leaf(edited),
        }),
        Op::Create(create_op(dir_n(20, "made", Some(2), ""), None)),
        Op::Create(create_op(leaf_n(21, "madeleaf", 20, "V"), None)),
    ];
    let done = store
        .batch(req(
            Some("all kinds"),
            None,
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
        let node = leaf_n(n, &format!("x{i}"), 5, "V");
        store
            .create(req(
                Some("grow"),
                None,
                create_op(node, Some(Position::First)),
            ))
            .unwrap();
        order.insert(0, nid(n));
        let campaign = store.read().unwrap().campaign();
        let keys: Vec<String> = campaign
            .children(&nid(5))
            .iter()
            .map(|c| {
                campaign
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
        store.read().unwrap().campaign().children(&nid(5)),
        order.as_slice(),
        "the listed order is the insert order"
    );
    let rebalances = store
        .read()
        .unwrap()
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
        disk.open()
            .unwrap()
            .read()
            .unwrap()
            .campaign()
            .children(&nid(5)),
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

fn held(store: &mut CachedCampaign) -> BTreeSet<(String, String, String)> {
    let mut out = BTreeSet::new();
    for (target, referrers) in store.read().unwrap().reverse().entries() {
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

    fn step_reference(node: Node, target: u32) -> Node {
        let Node::Leaf(mut l) = node else { return node };
        l.steps[0].references.push(Reference {
            label: None,
            title: "step ref".into(),
            target: None,
            note: None,
            node_id: Some(nid(target)),
            extra: Default::default(),
        });
        Node::Leaf(l)
    }

    fn dir_reference(node: Node, target: u32) -> Node {
        let Node::Directory(mut d) = node else {
            return node;
        };
        d.references.push(Reference {
            label: None,
            title: "dir ref".into(),
            target: None,
            note: None,
            node_id: Some(nid(target)),
            extra: Default::default(),
        });
        Node::Directory(d)
    }

    fn apply(store: &mut CachedCampaign, step: (u8, u8, u8), next: &mut u32) {
        let ids: Vec<NodeId> = ids_of(store).into_iter().collect();
        let pick = |x: u8| ids[x as usize % ids.len()].clone();
        let number = |id: &NodeId| (1..400).find(|n| nid(*n) == *id).unwrap_or(0);
        let (kind, a, b) = step;
        let mut of = |id: &NodeId| store.read().unwrap().campaign().get(id).cloned();
        let leaves: Vec<NodeId> = ids
            .iter()
            .filter(|i| matches!(of(i), Some(Node::Leaf(_))))
            .cloned()
            .collect();
        let dirs: Vec<NodeId> = ids
            .iter()
            .filter(|i| matches!(of(i), Some(Node::Directory(_))))
            .cloned()
            .collect();
        let replace = |store: &mut CachedCampaign, node: Node| {
            let rev = store.read().unwrap().rev(node.id()).cloned();
            let _ = store.replace(Request {
                header: header(Some("r"), None),
                op: ReplaceOp {
                    expected_rev: rev,
                    node,
                },
            });
        };
        match kind % 10 {
            0 => {
                *next += 1;
                let parent = dirs[a as usize % dirs.len()].clone();
                let mut node = leaf_n(*next, &format!("x{next}"), number(&parent), "V");
                if b % 2 == 0 {
                    node = with_reference(node, number(&pick(b)));
                }
                let _ = store.create(req(Some("c"), None, create_op(node, None)));
            }
            1 if !leaves.is_empty() => {
                let id = leaves[a as usize % leaves.len()].clone();
                let node = with_reference(of(&id).unwrap(), number(&pick(b)));
                replace(store, node);
            }
            2 if !leaves.is_empty() => {
                let id = leaves[a as usize % leaves.len()].clone();
                let Some(Node::Leaf(mut l)) = of(&id) else {
                    return;
                };
                l.references.clear();
                l.steps[0].references.clear();
                replace(store, Node::Leaf(l));
            }
            3 => {
                let id = pick(a);
                let parent = dirs[b as usize % dirs.len()].clone();
                let op = RelocateOp {
                    id: id.clone(),
                    expected_rev: store.read().unwrap().rev(&id).cloned(),
                    parent_id: Some(parent),
                    name: None,
                    position: None,
                };
                let _ = store.move_node(req(Some("m"), None, op));
            }
            4 => {
                let id = pick(a);
                let op = DeleteOp {
                    id: id.clone(),
                    expected_subtree_rev: store.read().unwrap().subtree_rev(&id),
                };
                let _ = store.delete_subtree(req(Some("d"), None, op));
            }
            5 => {
                let _ = store.comment(req(
                    None,
                    None,
                    CommentOp {
                        node_id: pick(a),
                        text: vec!["c".into()],
                    },
                ));
            }
            6 if !leaves.is_empty() => {
                let id = leaves[a as usize % leaves.len()].clone();
                let node = step_reference(of(&id).unwrap(), number(&pick(b)));
                replace(store, node);
            }
            7 => {
                // A batch: a leaf with a step reference, and a comment on a node.
                *next += 1;
                let parent = dirs[a as usize % dirs.len()].clone();
                let node = step_reference(
                    leaf_n(*next, &format!("x{next}"), number(&parent), "V"),
                    number(&pick(b)),
                );
                let ops = vec![
                    Op::Create(create_op(node, None)),
                    Op::Comment(CommentOp {
                        node_id: pick(b),
                        text: vec!["c".into()],
                    }),
                ];
                let _ = store.batch(req(
                    Some("b"),
                    None,
                    BatchOp {
                        expected_tree_rev: None,
                        ops,
                        dry_run: false,
                    },
                ));
            }
            8 => {
                // A rename only: no parent, no place.
                *next += 1;
                let id = pick(a);
                let op = RelocateOp {
                    id: id.clone(),
                    expected_rev: store.read().unwrap().rev(&id).cloned(),
                    parent_id: None,
                    name: Some(name(&format!("r{next}"))),
                    position: None,
                };
                let _ = store.move_node(req(Some("m"), None, op));
            }
            9 if !dirs.is_empty() => {
                let id = dirs[a as usize % dirs.len()].clone();
                let node = dir_reference(of(&id).unwrap(), number(&pick(b)));
                replace(store, node);
            }
            _ => {
                let _ = store.comment(req(
                    None,
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
        #![proptest_config(ProptestConfig { cases: soak_cases(24), ..ProptestConfig::default() })]
        #[test]
        fn reverse_index_equals_brute_force_scan(steps in prop::collection::vec((any::<u8>(), any::<u8>(), any::<u8>()), 1..16)) {
            let mut nodes = base();
            nodes.push(step_reference(with_reference(leaf_n(6, "linked", 5, "V"), 3), 4));
            let disk = Disk::new();
            disk.write_all(&nodes);
            let mut store = disk.open().unwrap();
            prop_assert_eq!(held(&mut store), scan(&disk));
            let mut next = 100;
            for step in steps {
                apply(&mut store, step, &mut next);
                let oracle = scan(&disk);
                prop_assert_eq!(&held(&mut store), &oracle, "the index kept up with the write");
                prop_assert_eq!(&held(&mut disk.open().unwrap()), &oracle, "a cold build agrees");
            }
        }
    }
}

type Row<'a> = (
    &'a str,
    Box<dyn Fn(&mut CachedCampaign) -> Result<(), StoreError> + 'a>,
    String,
    Rule,
);

fn structure_rows() {
    // demo(1) / core(2) { alpha(3), beta(4), inner(10) };  spare(5) { twin(7), twin(8) };
    // orphans: leaf(6) and directory(9) under a parent that does not exist.
    let mut nodes = base();
    nodes.push(leaf_n(6, "orphan", 99, "V"));
    nodes.push(leaf_n(7, "twin", 5, "V"));
    nodes.push(leaf_n(8, "twin", 5, "k"));
    nodes.push(dir_n(9, "lost", Some(99), "V"));
    nodes.push(dir_n(10, "inner", Some(2), "t"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let mut store = disk.open().unwrap();
    let before = files_of(&disk.root);
    let note = Some("why");

    let rows: Vec<Row> = vec![
        (
            "create: an id that exists",
            Box::new(move |s| {
                s.create(req(note, None, create_op(leaf_n(3, "zz", 2, "V"), None)))
                    .map(drop)
            }),
            file(3),
            Rule::DuplicateId,
        ),
        (
            "create: no such parent",
            Box::new(move |s| {
                s.create(req(note, None, create_op(leaf_n(20, "a20", 77, "V"), None)))
                    .map(drop)
            }),
            file(20),
            Rule::ParentMissing,
        ),
        (
            "create: the parent is a leaf",
            Box::new(move |s| {
                s.create(req(note, None, create_op(leaf_n(20, "a20", 3, "V"), None)))
                    .map(drop)
            }),
            file(20),
            Rule::ParentNotDirectory,
        ),
        (
            "create: the parent is not validated",
            Box::new(move |s| {
                s.create(req(note, None, create_op(leaf_n(20, "a20", 9, "V"), None)))
                    .map(drop)
            }),
            file(20),
            Rule::UnvalidatedNode,
        ),
        (
            "create: a sibling has the name",
            Box::new(move |s| {
                s.create(req(
                    note,
                    None,
                    create_op(leaf_n(20, "alpha", 2, "V"), None),
                ))
                .map(drop)
            }),
            file(20),
            Rule::NameCollision,
        ),
        (
            "create: a second root",
            Box::new(move |s| {
                s.create(req(
                    note,
                    None,
                    create_op(dir_n(20, "demo", None, ""), None),
                ))
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
                    None,
                    create_op(leaf_n(20, "a20", 2, "V"), Some(Position::Before(nid(5)))),
                ))
                .map(drop)
            }),
            "<request>".into(),
            Rule::PositionNotSibling,
        ),
        (
            "create: no note",
            Box::new(move |s| {
                s.create(req(None, None, create_op(leaf_n(20, "a20", 2, "V"), None)))
                    .map(drop)
            }),
            "<request>".into(),
            Rule::NoteRequired,
        ),
        (
            "relocate: nothing to do",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, None, None);
                s.move_node(req(note, None, op)).map(drop)
            }),
            "<request>".into(),
            Rule::NothingToDo,
        ),
        (
            "relocate: the root gets a parent",
            Box::new(move |s| {
                let op = relocate_op(s, 1, None, Some(5), None);
                s.move_node(req(note, None, op)).map(drop)
            }),
            file(1),
            Rule::RootDeleteOrMove,
        ),
        (
            "relocate: into its own subtree",
            Box::new(move |s| {
                let op = relocate_op(s, 2, None, Some(10), None);
                s.move_node(req(note, None, op)).map(drop)
            }),
            file(2),
            Rule::Cycle,
        ),
        (
            "relocate: a name a sibling has",
            Box::new(move |s| {
                let op = relocate_op(s, 4, Some("alpha"), None, None);
                s.move_node(req(note, None, op)).map(drop)
            }),
            file(4),
            Rule::NameCollision,
        ),
        (
            "relocate: a position that is no sibling",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, None, Some(Position::Before(nid(5))));
                s.move_node(req(note, None, op)).map(drop)
            }),
            "<request>".into(),
            Rule::PositionNotSibling,
        ),
        (
            "relocate: no such parent",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, Some(77), None);
                s.move_node(req(note, None, op)).map(drop)
            }),
            file(3),
            Rule::ParentMissing,
        ),
        (
            "relocate: the parent is a leaf",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, Some(4), None);
                s.move_node(req(note, None, op)).map(drop)
            }),
            file(3),
            Rule::ParentNotDirectory,
        ),
        (
            "relocate: into an unvalidated parent",
            Box::new(move |s| {
                let op = relocate_op(s, 3, None, Some(9), None);
                s.move_node(req(note, None, op)).map(drop)
            }),
            file(3),
            Rule::UnvalidatedNode,
        ),
        (
            "relocate: a place among the siblings of an unvalidated node",
            Box::new(move |s| {
                let op = relocate_op(s, 9, None, None, Some(Position::First));
                s.move_node(req(note, None, op)).map(drop)
            }),
            file(9),
            Rule::UnvalidatedNode,
        ),
        (
            "relocate: no note",
            Box::new(move |s| {
                let op = relocate_op(s, 3, Some("q"), None, None);
                s.move_node(req(None, None, op)).map(drop)
            }),
            "<request>".into(),
            Rule::NoteRequired,
        ),
        (
            "relocate: no expected_rev",
            Box::new(move |s| {
                let mut op = relocate_op(s, 3, Some("q"), None, None);
                op.expected_rev = None;
                s.move_node(req(note, None, op)).map(drop)
            }),
            "<request>".into(),
            Rule::ExpectedRevRequired,
        ),
        (
            "delete: no expected_subtree_rev",
            Box::new(move |s| {
                s.delete_subtree(req(
                    note,
                    None,
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
                s.delete_subtree(req(note, None, op)).map(drop)
            }),
            file(1),
            Rule::RootDeleteOrMove,
        ),
        (
            "delete: no note",
            Box::new(move |s| {
                let op = delete_op(s, 5);
                s.delete_subtree(req(None, None, op)).map(drop)
            }),
            "<request>".into(),
            Rule::NoteRequired,
        ),
        (
            "comment: with a note",
            Box::new(move |s| {
                s.comment(req(
                    note,
                    None,
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
        assert_eq!(files_of(&disk.root), before, "{row}: nothing was written");
    }
    // Stale and missing.
    let wrong = store.read().unwrap().subtree_rev(&nid(2));
    match store.delete_subtree(req(
        note,
        None,
        DeleteOp {
            id: nid(5),
            expected_subtree_rev: wrong,
        },
    )) {
        Err(StoreError::Stale {
            current: Current::Subtree { id, rev },
        }) => {
            assert_eq!(
                (id, Some(rev)),
                (nid(5), store.read().unwrap().subtree_rev(&nid(5)))
            );
        }
        other => panic!("a stale subtree: want Stale, got {other:?}"),
    }
    assert!(matches!(
        store.delete_subtree(req(
            note,
            None,
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
            None,
            CommentOp {
                node_id: nid(77),
                text: vec!["c".into()]
            }
        )),
        Err(StoreError::NotFound { .. })
    ));
    assert_eq!(files_of(&disk.root), before);

    // A repair is accepted: a comment on the damage, a node moved into the tree, a colliding
    // sibling renamed, an orphan subtree deleted.
    store
        .comment(req(
            None,
            None,
            CommentOp {
                node_id: nid(9),
                text: vec!["lost".into()],
            },
        ))
        .unwrap();
    let op = relocate_op(&mut store, 6, None, Some(2), None);
    store.move_node(req(note, None, op)).unwrap();
    assert!(store.read().unwrap().campaign().is_validated(&nid(6)));
    assert!(
        !store.read().unwrap().campaign().is_validated(&nid(8)),
        "still colliding"
    );
    let op = relocate_op(&mut store, 8, Some("twin2"), None, None);
    store.move_node(req(note, None, op)).unwrap();
    assert!(
        store.read().unwrap().campaign().is_validated(&nid(7))
            && store.read().unwrap().campaign().is_validated(&nid(8))
    );
    let op = delete_op(&mut store, 9);
    store.delete_subtree(req(note, None, op)).unwrap();
    let damage: Vec<Rule> = rules(&mut store)
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
    assert_eq!(rules(&mut disk.open().unwrap()), rules(&mut store));

    // The root. A healthy campaign cannot lose its root name, a renamed campaign repairs it.
    let healthy = Disk::new();
    healthy.write_all(&base());
    let mut store = healthy.open().unwrap();
    let before = files_of(&healthy.root);
    let op = relocate_op(&mut store, 1, Some("other"), None, None);
    match store.move_node(req(note, None, op)) {
        Err(StoreError::Invalid {
            rule: Rule::RootCount | Rule::RootNameMismatch,
            ..
        }) => {}
        other => panic!("renaming a sound root: want a root rule, got {other:?}"),
    }
    assert_eq!(files_of(&healthy.root), before);
    // A stray parentless node does not make the sound root renamable: the exemption is for a
    // campaign with no validated root.
    let stray = Disk::new();
    let mut nodes = base();
    nodes.push(dir_n(8, "other", None, ""));
    stray.write_all(&nodes);
    let mut store = stray.open().unwrap();
    assert_eq!(store.read().unwrap().campaign().root(), Some(&nid(1)));
    let before = files_of(&stray.root);
    let op = relocate_op(&mut store, 1, Some("zzz"), None, None);
    match store.move_node(req(note, None, op)) {
        Err(StoreError::Invalid {
            rule: Rule::RootCount | Rule::RootNameMismatch,
            ..
        }) => {}
        other => {
            panic!("renaming a sound root beside a stray root: want a root rule, got {other:?}")
        }
    }
    assert_eq!(files_of(&stray.root), before);
    assert_eq!(
        validated(&mut store).len(),
        5,
        "the sound root is still the root"
    );
    let mut renamed =
        CachedCampaign::open(&healthy.root, &name("other"), Options::default()).unwrap();
    assert_eq!(renamed.read().unwrap().campaign().root(), None);
    let op = relocate_op(&mut renamed, 1, Some("other"), None, None);
    renamed.move_node(req(note, None, op)).unwrap();
    assert_eq!(
        renamed.read().unwrap().campaign().root(),
        Some(&nid(1)),
        "the rename repaired the root"
    );
    assert_eq!(validated(&mut renamed).len(), 5);

    // Two roots after a merge of two inits: every node is unvalidated; renaming or deleting one
    // of them is the repair.
    for (row, rename) in [("rename", true), ("delete", false)] {
        let disk = merged(vec![Edit::new(dir_n(8, "demo", None, ""))], vec![]);
        let mut store = disk.open().unwrap();
        assert_eq!(store.read().unwrap().campaign().root(), None);
        if rename {
            let op = relocate_op(&mut store, 8, Some("second"), None, None);
            store.move_node(req(note, None, op)).unwrap();
        } else {
            let op = delete_op(&mut store, 8);
            store.delete_subtree(req(note, None, op)).unwrap();
        }
        assert_eq!(
            store.read().unwrap().campaign().root(),
            Some(&nid(1)),
            "{row}: one root is left"
        );
        assert_eq!(validated(&mut store).len(), 5, "{row}");
    }

    // A cycle that a merge of two branches leaves: core under spare on one branch, spare under
    // core on the other. Deleting a member deletes the cycle and everything below it.
    let disk = merged(
        vec![Edit::new(dir_n(2, "core", Some(5), "V"))],
        vec![Edit::new(dir_n(5, "spare", Some(2), "k"))],
    );
    let mut store = disk.open().unwrap();
    assert!(rules(&mut store).iter().any(|(_, r)| *r == Rule::Cycle));
    let op = delete_op(&mut store, 5);
    let done = store.delete_subtree(req(note, None, op)).unwrap();
    assert_eq!(done.result.removed, vec![nid(5), nid(2), nid(3), nid(4)]);
    for n in [2, 3, 4, 5] {
        assert!(!disk.path(&file(n)).exists(), "n{n} is gone from the disk");
    }
    assert!(disk.path(&file(1)).exists());
    assert_eq!(
        ids_of(&mut store),
        BTreeSet::from([nid(1)]),
        "and from the store"
    );
    let amendment = &store.read().unwrap().amendments()[done.amendment.as_ref().unwrap()];
    let listed: BTreeSet<NodeId> = amendment
        .changes
        .iter()
        .map(|c| c.node_id.clone())
        .collect();
    assert_eq!(listed, BTreeSet::from([nid(2), nid(3), nid(4), nid(5)]));
    let mut reopened = disk.open().unwrap();
    assert_eq!(ids_of(&mut reopened), ids_of(&mut store));
    assert!(
        rules(&mut reopened).iter().all(|(_, r)| *r != Rule::Cycle),
        "no cycle is reported"
    );
}

#[test]
fn marks_name_criteria() {
    let marked = |gates_with_extra: bool| {
        let Node::Leaf(mut l) = leaf_n(3, "alpha", 2, "V") else {
            unreachable!()
        };
        let mut extra = extra_criterion(39);
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
        rules(&mut store),
        vec![],
        "a mark that names a criterion of its file is sound"
    );

    // A request that leaves a new mark naming a missing criterion is refused.
    let mut bad = leaf_of(&mut store, 3);
    let mut stray = bad.marks[0].clone();
    stray.criterion_id = cid(999);
    bad.marks.push(stray);
    let before = files_of(&disk.root);
    let rev = rev_of(&mut store, 3);
    match store.replace(replace_req(header(None, None), Some(&rev), Node::Leaf(bad))) {
        Err(StoreError::Invalid {
            file: f,
            rule: Rule::MarkTarget,
            ..
        }) => assert_eq!(f, file(3)),
        other => panic!("want mark_target, got {other:?}"),
    }
    assert_eq!(files_of(&disk.root), before);

    // A removal through the store leaves the amendment that explains the mark; no diagnostic.
    let mut fewer = leaf_of(&mut store, 3);
    fewer.success_gates.retain(|c| c.id != cid(39));
    store
        .replace(replace_req(
            header(Some("drop a gate"), None),
            Some(&rev),
            Node::Leaf(fewer),
        ))
        .unwrap();
    assert_eq!(rules(&mut store), vec![]);
    assert_eq!(rules(&mut disk.open().unwrap()), vec![]);

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
        rules(&mut hand.open().unwrap()),
        vec![(file(3), Rule::MarkTarget)]
    );
}

// ---------------------------------------------------------------------------------------------
// Recovery

/// An operation that writes several files, run over a recording file system: the state before,
/// the state after and the effects between.
type Snap = BTreeMap<String, Vec<u8>>;
type Recorded = (tempfile::TempDir, Snap, Snap, Vec<Effect>);

fn recorded(run: impl FnOnce(&mut CachedCampaign)) -> Recorded {
    let mut nodes = base();
    nodes.push(leaf_n(6, "child", 5, "V"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    let pre_dir = tempfile::tempdir().unwrap();
    copy_tree(&disk.root, &pre_dir.path().join("demo"));
    let rec = Arc::new(RecordingFs::new(&disk.root));
    let mut store = CachedCampaign::open(
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
    let pre = files_of(&pre_dir.path().join("demo"));
    let post = files_of(&disk.root);
    (pre_dir, pre, post, log)
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
            None,
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
    let work = work_dir.path().join("demo");
    at_prefix(&pre_dir.path().join("demo"), &log, intent + 1, &work);
    std::fs::create_dir_all(work.join(".rdm/tmp")).unwrap();
    std::fs::write(work.join(".rdm/tmp/stray"), b"half").unwrap();
    assert_eq!(files_of(&work), pre, "a reader sees the state before");
    let mut store = CachedCampaign::open(&work, &name("demo"), Options::default()).unwrap();
    let report = store.recover().unwrap();
    assert_eq!(
        (
            report.redone.len(),
            report.stray_temp_files,
            report.undecodable.len()
        ),
        (1, 1, 0)
    );
    assert_eq!(files_of(&work), post, "recovery completes the operation");
    assert!(scratch_is_empty(&work));
    assert_eq!(
        store.read().unwrap().campaign().get(&nid(5)),
        None,
        "and the store holds the result"
    );
    assert_eq!(
        store.recover().unwrap(),
        rdm_engine::write::Recovered::default(),
        "a second recovery has nothing to do"
    );
}

/// Guards S19's rule that recovery never overwrites a create-only file: a create whose
/// destination now holds other bytes (another writer's create of that id, as a merge or a
/// checkout leaves it) stops recovery with `duplicate_id` naming the file, and the record stays.
///
/// Bug it catches: recovery that redoes a create as a put and silently replaces a file it did not
/// write.
/// A root that does not exist is refused by `open`, and neither it nor a write afterwards
/// creates the path: a typo in the root must not leave a stray directory tree.
#[test]
fn open_on_a_path_that_does_not_exist_errors_and_creates_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("typo").join("demo");
    let refused = CachedCampaign::open(&missing, &name("demo"), Options::default())
        .err()
        .expect("a root that is not a directory is refused");
    assert!(matches!(refused, StoreError::Io { ref file, .. } if file == "."), "{refused:?}");
    assert!(!tmp.path().join("typo").exists(), "nothing was created");
    let lock = WriterLock::acquire(&missing, Duration::from_millis(50), b"x");
    assert!(lock.is_err(), "the lock does not create a missing root");
    assert!(!tmp.path().join("typo").exists(), "the lock created nothing");
}

/// A volume that refuses hard links still takes a create (store.md, Create-only publish: check
/// the name is free, then rename), and a name an external writer took in the meantime is refused
/// with the error a volume with hard links gives.
#[test]
fn create_on_a_volume_without_hard_links_publishes_and_still_refuses_a_taken_name() {
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store =
        CachedCampaign::open(&disk.root, &name("demo"), options_with(no_hard_links())).unwrap();
    store
        .create(req(Some("n"), None, create_op(leaf_n(20, "fresh", 2, "Z"), None)))
        .expect("a create works without hard links");
    assert!(disk.root.join(file(20)).is_file());

    let taken = |refuse_links: bool| {
        let disk = Disk::new();
        disk.write_all(&base());
        let (fs, armed) = race_at_link(refuse_links, &disk.root.join(file(21)), b"external");
        let mut store = CachedCampaign::open(&disk.root, &name("demo"), options_with(fs)).unwrap();
        armed.store(true, std::sync::atomic::Ordering::SeqCst);
        let refused = store
            .create(req(Some("n"), None, create_op(leaf_n(21, "late", 2, "Z"), None)))
            .unwrap_err();
        assert_eq!(std::fs::read(disk.root.join(file(21))).unwrap(), b"external");
        refused
    };
    let without_links = taken(true);
    assert_eq!(
        without_links,
        StoreError::Invalid {
            file: file(21),
            rule: Rule::DuplicateId,
            detail: "the file already exists".into(),
        }
    );
    assert_eq!(without_links, taken(false), "the same error as on a normal volume");
}

#[test]
fn recovery_does_not_overwrite_a_create_whose_destination_holds_other_bytes() {
    let created = leaf_n(20, "fresh", 2, "V");
    let (pre_dir, _pre, _post, log) = recorded(|s| {
        s.create(req(Some("add"), None, create_op(created.clone(), None)))
            .unwrap();
    });
    // Cut just after the intent was published: the create of the node has not happened.
    let intent = log
        .iter()
        .position(|e| matches!(e, Effect::Link { link, .. } | Effect::Rename { to: link, .. } if link.starts_with(".rdm/intent")))
        .unwrap();
    let work_dir = tempfile::tempdir().unwrap();
    let work = work_dir.path().join("demo");
    at_prefix(&pre_dir.path().join("demo"), &log, intent + 1, &work);
    // The other actor: a create of the same id with other content, written by the codec.
    let mut other = leaf_of_node(&created);
    other.goal = "the other writer's goal".into();
    let dest = work.join(file(20));
    std::fs::write(&dest, encode(&Node::Leaf(other))).unwrap();
    let theirs = std::fs::read(&dest).unwrap();

    let mut store = CachedCampaign::open(&work, &name("demo"), Options::default()).unwrap();
    for attempt in ["first", "second"] {
        match store.recover() {
            Err(StoreError::Invalid { file: f, rule, .. }) => {
                assert_eq!((f, rule), (file(20), Rule::DuplicateId), "{attempt}");
            }
            other => panic!("{attempt}: want Invalid duplicate_id, got {other:?}"),
        }
        assert!(
            std::fs::read(&dest).unwrap() == theirs,
            "{attempt}: recovery overwrote the file it must only create"
        );
        let records = std::fs::read_dir(work.join(".rdm/intent")).unwrap().count();
        assert_eq!(
            records, 1,
            "{attempt}: the record is left for a person to resolve"
        );
    }
}

fn leaf_of_node(n: &Node) -> Leaf {
    match n {
        Node::Leaf(l) => l.clone(),
        Node::Directory(_) => panic!("a leaf"),
    }
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
            op: create_op(leaf_n(20, "fresh", 2, "V"), None),
        })
        .unwrap();
    assert!(!c.replayed);
    let op = relocate_op(&mut store, 3, Some("moved"), None, None);
    store
        .move_node(Request {
            header: header(note, Some("op-move")),
            op,
        })
        .unwrap();
    let op = delete_op(&mut store, 5);
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
    let before = files_of(&disk.root);

    let with = |op_id: &str| header(note, Some(op_id));
    // A different node under the op_id of a create or a relocate.
    reused(
        "create of another node",
        store.create(Request {
            header: with("op-create"),
            op: create_op(leaf_n(21, "other", 2, "V"), None),
        }),
    );
    let op = relocate_op(&mut store, 4, Some("beta2"), None, None);
    reused(
        "relocate of another node",
        store.move_node(Request {
            header: with("op-move"),
            op,
        }),
    );
    // A delete of a node the amendment did not remove: it would have returned no node.
    let op = delete_op(&mut store, 4);
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
            op: create_op(leaf_n(22, "third", 2, "V"), None),
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
    assert_eq!(files_of(&disk.root), before, "no refusal wrote anything");
    // The node of the first create is gone: its rev cannot be rebuilt.
    let op = delete_op(&mut store, 20);
    store.delete_subtree(req(note, None, op)).unwrap();
    let before_gone = files_of(&disk.root);
    reused(
        "a create whose node is gone",
        store.create(Request {
            header: with("op-create"),
            op: create_op(leaf_n(20, "fresh", 2, "V"), None),
        }),
    );
    assert_eq!(
        files_of(&disk.root),
        before_gone,
        "a refusal writes nothing"
    );
}

// ---------------------------------------------------------------------------------------------
// Recovery, at the store level

const PLANTED: &str = "01M88Y1HSKTXC19PQH7AQK75A3";

fn plant(disk: &Disk, effects: Vec<IntentEffect>) {
    intent::begin(
        &RealFs,
        &disk.root,
        PLANTED,
        &Intent::new(effects),
        Durability::default(),
    )
    .unwrap();
}

#[test]
fn an_intent_naming_a_file_outside_the_records_is_removed_not_redone() {
    let outside = |disk: &Disk, f: &str| disk.root.parent().unwrap().join(f);
    type Planted = Box<dyn Fn(&Disk) -> IntentEffect>;
    let rows: Vec<(&str, Planted)> = vec![
        (
            "a parent directory",
            Box::new(|_| IntentEffect::Del {
                path: "../victim.txt".into(),
            }),
        ),
        (
            "a parent inside a record path",
            Box::new(|_| IntentEffect::Del {
                path: "nodes/../../victim.txt".into(),
            }),
        ),
        (
            "an absolute path",
            Box::new(move |d| IntentEffect::Del {
                path: outside(d, "victim.txt").display().to_string(),
            }),
        ),
        (
            "the lock file",
            Box::new(|_| IntentEffect::Del {
                path: ".rdm/lock".into(),
            }),
        ),
        (
            "a file of the campaign that is no record",
            Box::new(|_| IntentEffect::Del {
                path: "README.md".into(),
            }),
        ),
        (
            "a directory alone",
            Box::new(|_| IntentEffect::Del {
                path: "nodes".into(),
            }),
        ),
        (
            "a create outside",
            Box::new(|_| IntentEffect::Create {
                path: "../made.txt".into(),
                bytes: "x".into(),
            }),
        ),
    ];
    for (row, effect) in rows {
        let disk = Disk::new();
        disk.write_all(&base());
        let victim = disk.root.parent().unwrap().join("victim.txt");
        std::fs::write(&victim, "keep").unwrap();
        std::fs::write(disk.path("README.md"), "keep").unwrap();
        // A second effect that is fine: it must not run either, the record is one unit.
        plant(
            &disk,
            vec![effect(&disk), IntentEffect::Del { path: file(3) }],
        );
        let mut store = disk.open().unwrap();
        let report = store.recover().unwrap();
        assert_eq!(report.undecodable, vec![PLANTED.to_owned()], "{row}");
        assert!(report.redone.is_empty(), "{row}");
        assert!(victim.exists(), "{row}: the file outside is untouched");
        assert!(disk.path("README.md").exists(), "{row}");
        assert!(
            disk.path(&file(3)).exists(),
            "{row}: nothing of the record ran"
        );
        assert!(
            !disk.root.parent().unwrap().join("made.txt").exists(),
            "{row}"
        );
        assert!(scratch_is_empty(&disk.root), "{row}: the record is gone");
    }
}

#[test]
fn a_write_completes_an_interrupted_operation_first_and_keeps_the_report() {
    let (pre_dir, _pre, post, log) = recorded(|s| {
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
            None,
            BatchOp {
                expected_tree_rev: None,
                ops,
                dry_run: false,
            },
        ))
        .unwrap();
    });
    let intent_at = log
        .iter()
        .position(|e| matches!(e, Effect::Link { link, .. } | Effect::Rename { to: link, .. } if link.starts_with(".rdm/intent")))
        .unwrap();
    let work_dir = tempfile::tempdir().unwrap();
    let work = work_dir.path().join("demo");
    at_prefix(&pre_dir.path().join("demo"), &log, intent_at + 1, &work);
    let mut store = CachedCampaign::open(&work, &name("demo"), Options::default()).unwrap();
    assert!(store.last_recovered().is_none(), "no recovery has run yet");
    // An unrelated write: a comment on a node the interrupted operation does not touch.
    store
        .comment(req(
            None,
            None,
            CommentOp {
                node_id: nid(3),
                text: vec!["hi".into()],
            },
        ))
        .unwrap();
    let report = store.last_recovered().expect("the write ran a recovery");
    assert_eq!(
        report.redone.len(),
        1,
        "it completed the interrupted operation"
    );
    let mut now = files_of(&work);
    now.retain(|k, _| !k.starts_with("_comments/"));
    assert_eq!(now, post, "the interrupted operation is complete");
    assert_eq!(store.read().unwrap().campaign().get(&nid(5)), None);
    // The next write recovers nothing, and the store says so.
    store
        .comment(req(
            None,
            None,
            CommentOp {
                node_id: nid(3),
                text: vec!["again".into()],
            },
        ))
        .unwrap();
    assert_eq!(store.last_recovered().unwrap().redone.len(), 0);
}

#[test]
fn an_intent_newer_than_this_binary_refuses_recovery_and_every_write() {
    let disk = Disk::new();
    disk.write_all(&base());
    let newer = Intent {
        format: INTENT_FORMAT + 1,
        effects: vec![IntentEffect::Del { path: file(3) }],
    };
    intent::begin(&RealFs, &disk.root, PLANTED, &newer, Durability::default()).unwrap();
    let mut store = disk.open().unwrap();
    let before = files_of(&disk.root);
    let want = |e: StoreError| match e {
        StoreError::NewerFormat {
            found,
            supported,
            file,
        } => {
            assert_eq!(found, (0, u16::try_from(INTENT_FORMAT + 1).unwrap(), 0));
            assert_eq!(supported, (0, u16::try_from(INTENT_FORMAT).unwrap(), 0));
            assert!(file.ends_with(&format!("{PLANTED}.json")), "{file}");
        }
        other => panic!("want NewerFormat, got {other:?}"),
    };
    want(store.recover().unwrap_err());
    let mut edited = leaf_of(&mut store, 3);
    edited.goal = "edit".into();
    let rev = rev_of(&mut store, 3);
    want(
        store
            .replace(replace_req(
                header(Some("e"), None),
                Some(&rev),
                Node::Leaf(edited),
            ))
            .unwrap_err(),
    );
    assert_eq!(files_of(&disk.root), before, "nothing was written");
    assert!(
        disk.path(&format!(".rdm/intent/{PLANTED}.json")).exists(),
        "the record is left"
    );
}

#[test]
fn a_garbage_holder_record_gives_locked_with_no_holder_at_the_store() {
    let disk = Disk::new();
    disk.write_all(&base());
    let held = WriterLock::acquire(&disk.root, Duration::from_secs(1), b"\xff garbage").unwrap();
    let mut store = CachedCampaign::open(
        &disk.root,
        &name("demo"),
        Options {
            lock_timeout: Duration::from_millis(60),
            ..Options::default()
        },
    )
    .unwrap();
    let before = files_of(&disk.root);
    let mut edited = leaf_of(&mut store, 3);
    edited.goal = "edit".into();
    let rev = rev_of(&mut store, 3);
    let err = store
        .replace(replace_req(
            header(Some("e"), None),
            Some(&rev),
            Node::Leaf(edited),
        ))
        .unwrap_err();
    assert!(
        matches!(err, StoreError::Locked { holder: None }),
        "{err:?}"
    );
    assert_eq!(files_of(&disk.root), before);
    drop(held);
}

#[test]
fn diagnostics_are_sorted_across_their_producers() {
    // Three producers, each in a different file: a file warning (n3), a campaign defect (n6) and a
    // mark diagnostic (n4). The store collects them in that order of producer, n3 then n6 then n4,
    // so only a sort puts n4 before n6.
    let mut nodes = base();
    nodes.push(leaf_n(6, "orphan", 99, "V"));
    let disk = Disk::new();
    disk.write_all(&nodes);
    edit_file(&disk, 3, |v| v["zz"] = json!(1));
    edit_file(&disk, 4, |v| {
        v["marks"] = json!([{
            "ts": "2026-01-01T00:00:00Z",
            "actor": {"kind": "agent"},
            "criterion_id": cid(999).to_string(),
            "met": true
        }]);
    });
    let mut store = disk.open().unwrap();
    assert_eq!(
        rules(&mut store),
        vec![
            (file(3), Rule::UnknownKey),
            (file(4), Rule::MarkTarget),
            (file(6), Rule::ParentMissing)
        ]
    );
}

fn stamped_ok(disk: &Disk, before: &BTreeMap<String, Vec<u8>>, row: &str) {
    let want = rdm_schema::version_string(rdm_schema::FORMAT_VERSION);
    let after = files_of(&disk.root);
    let mut checked = 0;
    for (path, bytes) in &after {
        if before.get(path) == Some(bytes) {
            continue;
        }
        let v: Value = serde_json::from_slice(bytes).unwrap();
        assert_eq!(v["schema_version"], json!(want), "{row}: {path}");
        checked += 1;
    }
    assert!(checked > 0, "{row}: something was written");
}

fn with_stamp(node: &Node, stamp: &str) -> Node {
    let mut n = node.clone();
    match &mut n {
        Node::Directory(d) => d.schema_version = stamp.into(),
        Node::Leaf(l) => l.schema_version = stamp.into(),
    }
    n
}

#[test]
fn every_write_stamps_the_current_format_version() {
    // No real producer writes an older stamp the store can read, so the payloads carry one: a
    // node a caller sends with a stale stamp is written with the current one, and so is every
    // amendment and comment the store makes.
    let disk = Disk::new();
    disk.write_all(&base());
    let mut store = disk.open().unwrap();
    let stale = "0.0.1";

    let before = files_of(&disk.root);
    let node = with_stamp(&leaf_n(20, "fresh", 2, "V"), stale);
    store
        .create(req(Some("c"), None, create_op(node, None)))
        .unwrap();
    stamped_ok(&disk, &before, "create");

    let before = files_of(&disk.root);
    let mut edited = leaf_of(&mut store, 3);
    edited.goal = "edit".into();
    let rev = rev_of(&mut store, 3);
    store
        .replace(replace_req(
            header(Some("e"), None),
            Some(&rev),
            with_stamp(&Node::Leaf(edited), stale),
        ))
        .unwrap();
    stamped_ok(&disk, &before, "replace");

    let before = files_of(&disk.root);
    let op = relocate_op(&mut store, 4, Some("beta2"), None, None);
    store.move_node(req(Some("m"), None, op)).unwrap();
    stamped_ok(&disk, &before, "relocate");

    let before = files_of(&disk.root);
    store
        .comment(req(
            None,
            None,
            CommentOp {
                node_id: nid(3),
                text: vec!["c".into()],
            },
        ))
        .unwrap();
    stamped_ok(&disk, &before, "comment");

    let before = files_of(&disk.root);
    let op = delete_op(&mut store, 5);
    store.delete_subtree(req(Some("d"), None, op)).unwrap();
    stamped_ok(&disk, &before, "delete");

    let before = files_of(&disk.root);
    let mut edited = leaf_of(&mut store, 4);
    edited.goal = "again".into();
    let ops = vec![
        Op::Create(create_op(
            with_stamp(&leaf_n(21, "batched", 2, "V"), stale),
            None,
        )),
        Op::Replace(ReplaceOp {
            expected_rev: Some(rev_of(&mut store, 4)),
            node: with_stamp(&Node::Leaf(edited), stale),
        }),
    ];
    store
        .batch(req(
            Some("b"),
            None,
            BatchOp {
                expected_tree_rev: None,
                ops,
                dry_run: false,
            },
        ))
        .unwrap();
    stamped_ok(&disk, &before, "batch");
}
