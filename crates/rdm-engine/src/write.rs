//! The write algorithm (`store.md`, §Write order) and the primitives built on it.
//!
//! Every primitive runs the same steps under the writer lock: recover, read the campaign from
//! disk, replay a known `op_id`, check revisions and structure, build every after-image, apply
//! the effects in the order of the three rules, and bring the in-memory state up to date from
//! what was written. A single operation is a batch of one.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

use rdm_schema::campaign::{Campaign, Diagnostic, node_file};
use rdm_schema::diff::{diff, is_state};
use rdm_schema::error::LockHolder;
use rdm_schema::error::{Current, Rule, StoreError};
use rdm_schema::ids::{AmendmentId, CommentId, Name, NodeId, OpId, OrderKey, Rev, SubtreeRev};
use rdm_schema::links::{LINKS, OnDelete, RecordRef};
use rdm_schema::ops::{
    BatchOp, CommentOp, Commented, CreateOp, DeleteOp, Deleted, Header, Op, OpResult, Position,
    Receipt, RelocateOp, ReplaceOp, Request, Written,
};
use rdm_schema::stored::{Actor, Amendment, Change, ChangeOp, Comment, Node};
use rdm_schema::validate::check_node;
use rdm_schema::{FORMAT_VERSION, version_string};
use serde_json::Value;

use crate::clock::{rfc3339, unix_ms};
use crate::codec::{decode, encode};
use crate::fs::Fs;
use crate::intent::{self, Intent, IntentEffect, Pending};
use crate::links::Record;
use crate::lock::{LockError, WriterLock};
use crate::order::{MAX_KEY_LEN, between, spread};
use crate::publish;
use crate::rev;
use crate::store::{OpRef, State, Store, io_error, load};

const REQUEST: &str = "<request>";

fn request_fault(rule: Rule, detail: impl Into<String>) -> StoreError {
    StoreError::Invalid {
        file: REQUEST.to_owned(),
        rule,
        detail: detail.into(),
    }
}

fn node_fault(id: &NodeId, rule: Rule, detail: impl Into<String>) -> StoreError {
    StoreError::Invalid {
        file: node_file(id),
        rule,
        detail: detail.into(),
    }
}

fn not_found(id: &NodeId) -> StoreError {
    StoreError::NotFound {
        target: RecordRef::Node(id.clone()),
    }
}

/// Sets the stamp of an after-image: a write always writes the current version.
fn stamp(node: &mut Node) {
    let v = version_string(FORMAT_VERSION);
    match node {
        Node::Directory(d) => d.schema_version = v,
        Node::Leaf(l) => l.schema_version = v,
    }
}

/// What a recovery did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Recovered {
    /// Intent records redone to the end and removed, oldest first.
    pub redone: Vec<String>,
    /// Records that did not decode and were removed.
    pub undecodable: Vec<String>,
    /// Stray files removed from `.rdm/tmp/`.
    pub stray_temp_files: usize,
}

/// The state a request is applied to: a copy of the campaign as read from disk that each op of
/// the batch changes in turn, with the record of what the ops did.
struct Working {
    campaign: Campaign,
    /// The `rev` of every node as read from disk: what a caller's `expected_rev` is judged against.
    disk_revs: BTreeMap<NodeId, Rev>,
    /// Nodes created by this batch, in creation order (parents first).
    created: Vec<NodeId>,
    changes: Vec<Change>,
    comments: Vec<(NodeId, Vec<String>)>,
    results: Vec<OpResult>,
    /// A create, relocate or delete was applied: the batch needs a note.
    structural: bool,
    snapshot: serde_json::Map<String, Value>,
    /// The nodes the ops removed, deepest first, in the order they are unlinked.
    removed: Vec<NodeId>,
}

/// Everything a validated request will write, in the order it is written.
struct Plan {
    effects: Vec<IntentEffect>,
    puts: Vec<Node>,
    creates: Vec<Node>,
    deletes: Vec<NodeId>,
    comments: Vec<Comment>,
    amendment: Option<Amendment>,
}

impl Store {
    fn fs(&self) -> &dyn Fs {
        &*self.options.fs
    }

    fn tmp_dir(&self) -> PathBuf {
        self.root.join(".rdm").join("tmp")
    }

    fn now(&self) -> std::time::SystemTime {
        (self.options.clock)()
    }

    fn ulid(&mut self) -> String {
        let ms = unix_ms((self.options.clock)());
        self.ulids.mint(ms, &mut *self.options.rng)
    }

    /// Takes the writer lock, naming the holder in the record it leaves.
    fn lock(&self) -> Result<WriterLock, StoreError> {
        let holder = LockHolder {
            pid: std::process::id(),
            host: std::env::var("HOSTNAME")
                .or_else(|_| std::env::var("COMPUTERNAME"))
                .unwrap_or_else(|_| "unknown".to_owned()),
            since: rfc3339(self.now()),
        };
        let record = serde_json::to_vec(&holder).unwrap_or_default();
        match WriterLock::acquire(&self.root, self.options.lock_timeout, &record) {
            Ok(lock) => Ok(lock),
            Err(LockError::Busy { holder }) => {
                // A holder record that is garbage is an unknown holder, never a failure.
                Err(StoreError::Locked {
                    holder: holder.and_then(|b| serde_json::from_slice(&b).ok()),
                })
            }
            Err(LockError::Io(e)) => Err(io_error(".rdm/lock", &e)),
        }
    }

    /// Makes sure `.rdm/.gitignore` holds `*`, so the repository needs no entry of its own.
    fn ignore_scratch(&self) -> Result<(), StoreError> {
        let path = self.root.join(".rdm").join(".gitignore");
        match self.fs().read(&path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => publish::put(
                self.fs(),
                &path,
                &self.tmp_dir(),
                b"*\n",
                self.options.durability,
            )
            .map_err(|e| io_error(".rdm/.gitignore", &e)),
            Err(e) => Err(io_error(".rdm/.gitignore", &e)),
        }
    }

    /// Recovery, under the lock (`store.md`, §Intent and recovery): remove the stray temporary
    /// files, redo every intent record oldest first, remove the records.
    pub(crate) fn recover_locked(&self) -> Result<Recovered, StoreError> {
        let fs = self.fs();
        let tier = self.options.durability;
        let mut report = Recovered::default();
        let tmp = self.tmp_dir();
        match fs.list_dir(&tmp) {
            Ok(names) => {
                for name in names {
                    match fs.remove_file(&tmp.join(&name)) {
                        Ok(()) => report.stray_temp_files += 1,
                        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                        Err(e) => return Err(io_error(&format!(".rdm/tmp/{name}"), &e)),
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(io_error(".rdm/tmp", &e)),
        }
        let pending = intent::pending(fs, &self.root).map_err(|e| io_error(".rdm/intent", &e))?;
        for p in pending {
            let file = format!(".rdm/intent/{}.json", p.ulid());
            match p {
                Pending::Newer { found, .. } => {
                    let found = u16::try_from(found).unwrap_or(u16::MAX);
                    return Err(StoreError::NewerFormat {
                        found: (0, found, 0),
                        supported: (
                            0,
                            u16::try_from(intent::INTENT_FORMAT).unwrap_or(u16::MAX),
                            0,
                        ),
                        file,
                    });
                }
                Pending::Undecodable { ulid } => {
                    intent::complete(fs, &self.root, &ulid, tier)
                        .map_err(|e| io_error(&file, &e))?;
                    report.undecodable.push(ulid);
                }
                Pending::Record { ulid, intent }
                    if !intent.effects.iter().all(effect_path_is_ours) =>
                {
                    // A record that names a file outside the campaign's records does not decode.
                    intent::complete(fs, &self.root, &ulid, tier)
                        .map_err(|e| io_error(&file, &e))?;
                    report.undecodable.push(ulid);
                }
                Pending::Record { ulid, intent } => {
                    for effect in &intent.effects {
                        self.redo(effect)?;
                    }
                    intent::complete(fs, &self.root, &ulid, tier)
                        .map_err(|e| io_error(&file, &e))?;
                    report.redone.push(ulid);
                }
            }
        }
        Ok(report)
    }

    /// Applies one recorded effect again: a put or a create whose destination already holds
    /// these bytes is skipped; a create over other bytes stops recovery.
    fn redo(&self, effect: &IntentEffect) -> Result<(), StoreError> {
        let (fs, tier, tmp) = (self.fs(), self.options.durability, self.tmp_dir());
        match effect {
            IntentEffect::Put { path, bytes } => {
                let dest = self.root.join(path);
                if fs.read(&dest).is_ok_and(|b| b == bytes.as_bytes()) {
                    return Ok(());
                }
                publish::put(fs, &dest, &tmp, bytes.as_bytes(), tier)
                    .map_err(|e| io_error(path, &e))
            }
            IntentEffect::Create { path, bytes } => {
                let dest = self.root.join(path);
                match fs.read(&dest) {
                    Ok(b) if b == bytes.as_bytes() => Ok(()),
                    Ok(_) => Err(StoreError::Invalid {
                        file: path.clone(),
                        rule: Rule::DuplicateId,
                        detail: "recovery found other bytes where it must create a file".to_owned(),
                    }),
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {
                        publish::create(fs, &dest, &tmp, bytes.as_bytes(), tier)
                            .map_err(|e| io_error(path, &e))
                    }
                    Err(e) => Err(io_error(path, &e)),
                }
            }
            IntentEffect::Del { path } => {
                publish::del(fs, &self.root.join(path), tier).map_err(|e| io_error(path, &e))
            }
        }
    }

    // -----------------------------------------------------------------------------------------
    // The write algorithm

    fn run(
        &mut self,
        header: &Header,
        ops: &[Op],
        expected_tree_rev: Option<&rdm_schema::ids::TreeRev>,
        in_batch: bool,
        dry_run: bool,
    ) -> Result<Receipt<Vec<OpResult>>, StoreError> {
        let wrap = |index: usize, e: StoreError| {
            if in_batch {
                StoreError::InBatch {
                    index,
                    source: Box::new(e),
                }
            } else {
                e
            }
        };
        // 1. Take the lock. 2. Recover. The guard releases the lock on every return.
        let _lock = self.lock()?;
        self.ignore_scratch()?;
        let report = self.recover_locked()?;
        self.last_recovered = Some(report);
        // 3. Read the campaign from disk, not from what an earlier read held.
        self.state = load(self.fs(), &self.root, self.state.campaign.slug())?;

        // 3.1 Replay.
        if let Some(op_id) = &header.op_id
            && let Some(receipt) = self.replay(op_id, ops)?
        {
            return Ok(receipt);
        }
        // 3.2 The revision of the whole campaign, when presented.
        if let Some(expected) = expected_tree_rev {
            let current = self.tree_rev();
            if *expected != current {
                return Err(StoreError::Stale {
                    current: Current::Tree { tree_rev: current },
                });
            }
        }
        let mut w = Working {
            campaign: self.state.campaign.clone(),
            disk_revs: self.state.revs.clone(),
            created: Vec::new(),
            changes: Vec::new(),
            comments: Vec::new(),
            results: Vec::new(),
            structural: false,
            snapshot: serde_json::Map::new(),
            removed: Vec::new(),
        };
        for (i, op) in ops.iter().enumerate() {
            self.apply_op(&mut w, header, op, in_batch)
                .map_err(|e| wrap(i, e))?;
        }
        self.check_whole(&w, header).map_err(|e| wrap(0, e))?;
        // 4. Every after-image, the amendment's ULID.
        let plan = self.plan(&w, header);
        let mut minted = plan.comments.iter().map(|c| c.id.clone());
        for r in &mut w.results {
            if let OpResult::Commented(c) = r
                && let Some(id) = minted.next()
            {
                c.id = id;
            }
        }
        if dry_run {
            return Ok(Receipt {
                result: w.results,
                amendment: None,
                tree_rev: self.tree_rev(),
                replayed: false,
            });
        }
        // 5. Apply. 6. Bring the state up to date from what was written.
        let amendment = plan.amendment.as_ref().map(|a| a.id.clone());
        if let Err(e) = self.apply_plan(&plan) {
            // Whatever was written is on disk; the next read starts from the files.
            if let Ok(fresh) = load(self.fs(), &self.root, self.state.campaign.slug()) {
                self.state = fresh;
            }
            return Err(e);
        }
        self.write_through(w.campaign, &plan);
        Ok(Receipt {
            result: w.results,
            amendment,
            tree_rev: self.tree_rev(),
            replayed: false,
        })
    }

    /// 3.1: the first outcome of a known `op_id`, with nothing written. The result is rebuilt from
    /// the records, never made up: a request the records cannot answer is `op_id_reused`.
    fn replay(
        &self,
        op_id: &OpId,
        ops: &[Op],
    ) -> Result<Option<Receipt<Vec<OpResult>>>, StoreError> {
        let Some(records) = self.state.ops.records(op_id) else {
            return Ok(None);
        };
        let amendment = records.iter().find_map(|r| match r {
            OpRef::Amendment(id) => self.state.amendments.get(id),
            _ => None,
        });
        let mut comment_ids = records.iter().filter_map(|r| match r {
            OpRef::Comment(id) => Some(id.clone()),
            _ => None,
        });
        let names = |id: &NodeId, kinds: &[ChangeOp]| {
            amendment.is_some_and(|a| {
                a.changes
                    .iter()
                    .any(|c| &c.node_id == id && (kinds.is_empty() || kinds.contains(&c.op)))
            })
        };
        let reused = |what: String| request_fault(Rule::OpIdReused, what);
        let mut results = Vec::new();
        for op in ops {
            let result = match op {
                Op::Create(CreateOp { node, .. }) => {
                    let id = node.id();
                    if !names(id, &[ChangeOp::Add]) {
                        return Err(reused(format!("no record of this `op_id` created {id}")));
                    }
                    self.written(id)?
                }
                Op::Replace(ReplaceOp { node, .. }) => {
                    let id = node.id();
                    if !names(id, &[]) && !records.contains(&OpRef::History(id.clone())) {
                        return Err(reused(format!("no record of this `op_id` edited {id}")));
                    }
                    self.written(id)?
                }
                Op::Relocate(r) => {
                    if !names(&r.id, &[ChangeOp::Rename, ChangeOp::Move]) {
                        return Err(reused(format!("no record of this `op_id` moved {}", r.id)));
                    }
                    self.written(&r.id)?
                }
                Op::Delete(d) => {
                    let in_snapshot = amendment
                        .and_then(|a| a.snapshot.as_ref())
                        .is_some_and(|s| s.contains_key(d.id.as_str()));
                    if !names(&d.id, &[ChangeOp::Remove]) || !in_snapshot {
                        return Err(reused(format!(
                            "no record of this `op_id` removed {}",
                            d.id
                        )));
                    }
                    OpResult::Deleted(self.replayed_delete(&d.id, amendment))
                }
                Op::Comment(_) => match comment_ids.next() {
                    Some(id) => OpResult::Commented(Commented { id }),
                    None => {
                        return Err(reused(
                            "the request names more comments than carry this `op_id`".to_owned(),
                        ));
                    }
                },
            };
            results.push(result);
        }
        Ok(Some(Receipt {
            result: results,
            amendment: amendment.map(|a| a.id.clone()),
            tree_rev: self.tree_rev(),
            replayed: true,
        }))
    }

    /// The current revision of a node a replayed write names; a node that is gone cannot be
    /// answered.
    fn written(&self, id: &NodeId) -> Result<OpResult, StoreError> {
        match self.state.revs.get(id) {
            Some(rev) => Ok(OpResult::Written(Written {
                id: id.clone(),
                rev: rev.clone(),
            })),
            None => Err(request_fault(
                Rule::OpIdReused,
                format!("{id} is gone: the first result cannot be rebuilt"),
            )),
        }
    }

    /// The nodes a delete removed, read back from the snapshot of its amendment.
    fn replayed_delete(&self, id: &NodeId, amendment: Option<&Amendment>) -> Deleted {
        let mut removed = Vec::new();
        if let Some(snapshot) = amendment.and_then(|a| a.snapshot.as_ref()) {
            let parent_of = |key: &str| -> Option<String> {
                snapshot
                    .get(key)?
                    .get("parent_id")?
                    .as_str()
                    .map(str::to_owned)
            };
            let order_of = |key: &str| -> String {
                snapshot
                    .get(key)
                    .and_then(|v| v.get("order")?.as_str())
                    .unwrap_or("")
                    .to_owned()
            };
            let mut stack = vec![id.to_string()];
            while let Some(k) = stack.pop() {
                if !snapshot.contains_key(&k) || removed.iter().any(|r: &NodeId| r.as_str() == k) {
                    continue;
                }
                if let Ok(n) = NodeId::parse(&k) {
                    removed.push(n);
                }
                let mut kids: Vec<&String> = snapshot
                    .keys()
                    .filter(|c| parent_of(c).as_deref() == Some(&k))
                    .collect();
                kids.sort_by(|a, b| (order_of(a), a).cmp(&(order_of(b), b)));
                stack.extend(kids.into_iter().rev().cloned());
            }
        }
        Deleted {
            referrers: self.referrers_into(&removed),
            removed,
        }
    }

    /// The `Report`-class referrers outside `removed` that point into it.
    fn referrers_into(&self, removed: &[NodeId]) -> Vec<rdm_schema::links::Referrer> {
        let inside: BTreeSet<&NodeId> = removed.iter().collect();
        let mut out = Vec::new();
        for id in removed {
            for r in self.state.reverse.referrers(id) {
                let outside = !matches!(&r.from, RecordRef::Node(n) if inside.contains(n));
                if outside && LINKS[r.link].on_delete == OnDelete::Report {
                    out.push(r.clone());
                }
            }
        }
        out
    }

    fn apply_op(
        &mut self,
        w: &mut Working,
        header: &Header,
        op: &Op,
        in_batch: bool,
    ) -> Result<(), StoreError> {
        match op {
            Op::Replace(op) => self.op_replace(w, header, op, in_batch),
            Op::Create(op) => op_create(w, op),
            Op::Relocate(op) => self.op_relocate(w, op, in_batch),
            Op::Delete(op) => self.op_delete(w, op, in_batch),
            Op::Comment(op) => op_comment(w, header, op, in_batch),
        }
    }

    /// The caller's revision of a node, judged against the disk. `None` is allowed only inside a
    /// batch, for a node the same batch created.
    fn check_expected(
        &self,
        w: &Working,
        id: &NodeId,
        expected: Option<&Rev>,
        in_batch: bool,
    ) -> Result<(), StoreError> {
        let created_here = w.created.contains(id);
        match expected {
            None if in_batch && created_here => Ok(()),
            None => Err(request_fault(
                Rule::ExpectedRevRequired,
                "`expected_rev` is required",
            )),
            Some(_) if created_here => Ok(()),
            Some(e) => match w.disk_revs.get(id) {
                Some(current) if current == e => Ok(()),
                Some(current) => Err(StoreError::Stale {
                    current: Current::Node {
                        id: id.clone(),
                        rev: current.clone(),
                    },
                }),
                None => Err(not_found(id)),
            },
        }
    }

    fn op_replace(
        &mut self,
        w: &mut Working,
        header: &Header,
        op: &ReplaceOp,
        in_batch: bool,
    ) -> Result<(), StoreError> {
        let id = op.node.id().clone();
        let current = w.campaign.get(&id).ok_or_else(|| not_found(&id))?.clone();
        self.check_expected(w, &id, op.expected_rev.as_ref(), in_batch)?;
        if !w.campaign.is_validated(&id) {
            return Err(node_fault(
                &id,
                Rule::UnvalidatedNode,
                "the node does not trace back to the root",
            ));
        }
        let mut after = op.node.clone();
        stamp(&mut after);
        if after.name() != current.name()
            || after.parent_id() != current.parent_id()
            || after.order() != current.order()
        {
            return Err(node_fault(
                &id,
                Rule::UseRelocate,
                "a replace cannot change `name`, `parent_id` or `order`",
            ));
        }
        if matches!(after, Node::Leaf(_)) && !w.campaign.children(&id).is_empty() {
            return Err(node_fault(
                &id,
                Rule::LeafHasChildren,
                "a node with children cannot become a leaf",
            ));
        }
        check_node(&after).map_err(|f| node_fault(&id, f.rule, f.detail))?;
        introduced_references_exist(&w.campaign, Some(&current), &after)?;
        introduced_marks_name_criteria(Some(&current), &after)?;
        let entries = diff(&current, &after);
        if header.note.is_none() && entries.iter().any(|c| !is_state(c)) {
            return Err(request_fault(
                Rule::NoteRequired,
                "a content edit needs a note",
            ));
        }
        w.changes.extend(entries);
        w.campaign.insert(after.clone());
        w.results.push(OpResult::Written(Written {
            id,
            rev: rev::rev(&after),
        }));
        Ok(())
    }

    /// The checks of the whole result, once all ops are applied.
    fn check_whole(&self, w: &Working, header: &Header) -> Result<(), StoreError> {
        if let Some(note) = &header.note {
            if note.description.trim().is_empty() {
                return Err(request_fault(
                    Rule::NoteRequired,
                    "the note's description is blank",
                ));
            }
            if w.changes.is_empty() {
                return Err(request_fault(
                    Rule::EmptyEdit,
                    "a note with no change to record",
                ));
            }
        }
        if w.structural && header.note.is_none() {
            return Err(request_fault(
                Rule::NoteRequired,
                "a structure edit needs a note",
            ));
        }
        // A write never adds a defect: what is damaged after was damaged before.
        let before: BTreeSet<(&str, Rule)> = self
            .state
            .campaign
            .diagnostics()
            .iter()
            .map(|d| (d.file.as_str(), d.rule))
            .collect();
        // While the campaign has no validated root, renaming a root is the repair of a root
        // defect: the root rules are one family, so a rename that trades `root_count` for
        // `root_name_mismatch` does not add a defect. With a validated root they follow the
        // rule like every other code, so a write cannot take a sound root away.
        let root_family = |r: Rule| matches!(r, Rule::RootCount | Rule::RootNameMismatch);
        let no_root_before = self.state.campaign.root().is_none();
        for d in w.campaign.diagnostics() {
            if root_family(d.rule) && no_root_before {
                continue;
            }
            if structural_rule(d.rule) && !before.contains(&(d.file.as_str(), d.rule)) {
                return Err(StoreError::Invalid {
                    file: d.file.clone(),
                    rule: d.rule,
                    detail: d.detail.clone(),
                });
            }
        }
        Ok(())
    }

    /// Builds every after-image and the ordered effects: creates parents first, replaces, the
    /// amendment, deletes deepest first.
    fn plan(&mut self, w: &Working, header: &Header) -> Plan {
        let before = &self.state.campaign;
        let stamped = version_string(FORMAT_VERSION);
        let mut creates = Vec::new();
        for id in &w.created {
            if let Some(n) = w.campaign.get(id) {
                creates.push(n.clone());
            }
        }
        let mut puts: Vec<Node> = Vec::new();
        for n in w.campaign.nodes() {
            if !w.created.contains(n.id()) && before.get(n.id()).is_some_and(|old| old != n) {
                puts.push(n.clone());
            }
        }
        // Deepest first along the `parent_id` chain as it is on disk before this write: a node
        // that a batch moved between two deleted subtrees still hangs from its old parent on
        // disk (it is deleted, so it is not rewritten), and the old parent must outlive it. A
        // cycle has no root to start from, so the removed ids themselves are the list. The sort
        // is stable: nodes of one depth keep the order `op_delete` listed them in.
        let mut seen = BTreeSet::new();
        let mut deletes: Vec<NodeId> = w
            .removed
            .iter()
            .filter(|id| before.get(id).is_some() && seen.insert((*id).clone()))
            .cloned()
            .collect();
        deletes.sort_by_key(|id| std::cmp::Reverse(chain_depth(before, id)));
        let ts = rfc3339(self.now());
        let comments: Vec<Comment> = w
            .comments
            .iter()
            .map(|(node_id, text)| Comment {
                schema_version: stamped.clone(),
                id: CommentId::parse(&self.ulid()).unwrap_or_else(|_| minted_ulid_invalid()),
                node_id: node_id.clone(),
                ts: ts.clone(),
                actor: header.actor.clone(),
                text: text.clone(),
                op_id: header.op_id.clone(),
                extra: serde_json::Map::new(),
            })
            .collect();
        let amendment = header
            .note
            .as_ref()
            .filter(|_| !w.changes.is_empty())
            .map(|note| Amendment {
                schema_version: stamped.clone(),
                id: AmendmentId::parse(&self.ulid()).unwrap_or_else(|_| minted_ulid_invalid()),
                legacy_id: None,
                ts: ts.clone(),
                actor: header.actor.clone(),
                description: note.description.clone(),
                source: note.source.clone(),
                op_id: header.op_id.clone(),
                correction: note.correction.then_some(true),
                changes: w.changes.clone(),
                snapshot: (!w.snapshot.is_empty()).then(|| w.snapshot.clone()),
                extra: serde_json::Map::new(),
            });

        let mut effects = Vec::new();
        let text = |n: &Node| encode(n);
        let node_path = |n: &Node| format!("nodes/{}.json", n.id());
        effects.extend(creates.iter().map(|n| IntentEffect::Create {
            path: node_path(n),
            bytes: text(n),
        }));
        effects.extend(comments.iter().map(|c| IntentEffect::Create {
            path: format!("_comments/{}.json", c.id),
            bytes: encode(c),
        }));
        effects.extend(puts.iter().map(|n| IntentEffect::Put {
            path: node_path(n),
            bytes: text(n),
        }));
        if let Some(a) = &amendment {
            effects.push(IntentEffect::Create {
                path: format!("_amendments/{}.json", a.id),
                bytes: encode(a),
            });
        }
        effects.extend(deletes.iter().map(|id| IntentEffect::Del {
            path: format!("nodes/{id}.json"),
        }));
        Plan {
            effects,
            puts,
            creates,
            deletes,
            comments,
            amendment,
        }
    }

    /// Step 5: publish the intent first when two or more files change, then every effect in
    /// order, then remove the intent.
    fn apply_plan(&mut self, plan: &Plan) -> Result<(), StoreError> {
        let tier = self.options.durability;
        let multi = plan.effects.len() >= 2;
        let intent_ulid = if multi { Some(self.ulid()) } else { None };
        let fs = self.fs();
        if let Some(ulid) = &intent_ulid {
            intent::begin(
                self.fs(),
                &self.root,
                ulid,
                &Intent::new(plan.effects.clone()),
                tier,
            )
            .map_err(|e| io_error(&format!(".rdm/intent/{ulid}.json"), &e))?;
        }
        for effect in &plan.effects {
            self.redo_fresh(effect)?;
        }
        if let Some(ulid) = &intent_ulid {
            intent::complete(fs, &self.root, ulid, tier)
                .map_err(|e| io_error(&format!(".rdm/intent/{ulid}.json"), &e))?;
        }
        Ok(())
    }

    /// Applies one effect of a live write. A create over an existing file is a collision.
    fn redo_fresh(&self, effect: &IntentEffect) -> Result<(), StoreError> {
        let (fs, tier, tmp) = (self.fs(), self.options.durability, self.tmp_dir());
        match effect {
            IntentEffect::Put { path, bytes } => {
                publish::put(fs, &self.root.join(path), &tmp, bytes.as_bytes(), tier)
                    .map_err(|e| io_error(path, &e))
            }
            IntentEffect::Create { path, bytes } => {
                publish::create(fs, &self.root.join(path), &tmp, bytes.as_bytes(), tier).map_err(
                    |e| {
                        if e.kind() == io::ErrorKind::AlreadyExists {
                            StoreError::Invalid {
                                file: path.clone(),
                                rule: Rule::DuplicateId,
                                detail: "the file already exists".to_owned(),
                            }
                        } else {
                            io_error(path, &e)
                        }
                    },
                )
            }
            IntentEffect::Del { path } => {
                publish::del(fs, &self.root.join(path), tier).map_err(|e| io_error(path, &e))
            }
        }
    }

    /// 6. The cache is updated from what was written, never before it is on disk.
    fn write_through(&mut self, campaign: Campaign, plan: &Plan) {
        let st: &mut State = &mut self.state;
        st.campaign = campaign;
        for n in plan.creates.iter().chain(&plan.puts) {
            let id = n.id().clone();
            st.revs.insert(id.clone(), rev::rev(n));
            st.reverse
                .replace_record(RecordRef::Node(id.clone()), Some(&Record::Node(n)));
            st.ops.replace_node(&id, Some(n));
            let file = node_file(&id);
            let diags = decode::<Node>(&file, encode(n).as_bytes())
                .map(|d| d.diagnostics)
                .unwrap_or_default();
            if diags.is_empty() {
                st.file_diags.remove(&file);
            } else {
                st.file_diags.insert(file, diags);
            }
        }
        for id in &plan.deletes {
            st.revs.remove(id);
            st.reverse.replace_record(RecordRef::Node(id.clone()), None);
            st.ops.replace_node(id, None);
            st.file_diags.remove(&node_file(id));
        }
        for c in &plan.comments {
            st.reverse
                .replace_record(RecordRef::Comment(c.id.clone()), Some(&Record::Comment(c)));
            st.ops.add_comment(c);
            st.comments.insert(c.id.clone(), c.clone());
        }
        if let Some(a) = &plan.amendment {
            st.reverse.replace_record(
                RecordRef::Amendment(a.id.clone()),
                Some(&Record::Amendment(a)),
            );
            st.ops.add_amendment(a);
            st.amendments.insert(a.id.clone(), a.clone());
        }
    }

    // -----------------------------------------------------------------------------------------
    // The primitives

    /// What the last recovery did: the one a write ran on taking the lock, or [`Store::recover`].
    /// `None` until a recovery has run; it holds the report until the next one replaces it.
    #[must_use]
    pub fn last_recovered(&self) -> Option<&Recovered> {
        self.last_recovered.as_ref()
    }

    /// Recovery on request: takes the writer lock, removes the stray temporary files, redoes every
    /// intent record left by an interrupted write, and reads the campaign again. Every write does
    /// this on its own on taking the lock; a reader that wants the settled state calls it.
    ///
    /// # Errors
    /// `Locked`, `NewerFormat` for an intent record newer than this binary, `Invalid` when a
    /// create finds other bytes where it must create a file, `Io`.
    pub fn recover(&mut self) -> Result<Recovered, StoreError> {
        let _lock = self.lock()?;
        self.ignore_scratch()?;
        let report = self.recover_locked()?;
        self.state = load(self.fs(), &self.root, self.state.campaign.slug())?;
        self.last_recovered = Some(report.clone());
        Ok(report)
    }

    /// Replaces one node file if its current `rev` is `expected_rev` (I4). It cannot change
    /// `id`, `name`, `parent_id` or `order`; a content edit needs a note and writes an amendment.
    ///
    /// # Errors
    /// `Stale`, `Locked`, `NotFound`, `Invalid` with the rule that is broken, `Io`.
    pub fn replace(&mut self, req: Request<ReplaceOp>) -> Result<Receipt<Written>, StoreError> {
        let one = self.run(&req.header, &[Op::Replace(req.op)], None, false, false)?;
        single(one, |r| match r {
            OpResult::Written(w) => Some(w),
            _ => None,
        })
    }
}

impl Store {
    /// Writes a new node file. The caller minted the ids in `node`; `node.order` is not read, the
    /// store sets it from `position` (default last) and rebalances the siblings when a key grows
    /// past the limit.
    ///
    /// # Errors
    /// `Locked`, `Invalid` with the rule that is broken (`duplicate_id`, `parent_missing`,
    /// `parent_not_directory`, `name_collision`, `root_count`, `root_name_mismatch`,
    /// `unvalidated_node`, `position_not_sibling`, `note_required`, a shape rule), `Io`.
    pub fn create(&mut self, req: Request<CreateOp>) -> Result<Receipt<Written>, StoreError> {
        let one = self.run(&req.header, &[Op::Create(req.op)], None, false, false)?;
        single(one, |r| match r {
            OpResult::Written(w) => Some(w),
            _ => None,
        })
    }

    /// Renames or moves one node: one node file and one amendment, no id changes.
    ///
    /// # Errors
    /// `Stale`, `Locked`, `NotFound`, `Invalid` (`nothing_to_do`, `parent_missing`,
    /// `parent_not_directory`, `cycle`, `name_collision`, `position_not_sibling`,
    /// `root_delete_or_move`, `unvalidated_node`, `note_required`), `Io`.
    pub fn move_node(&mut self, req: Request<RelocateOp>) -> Result<Receipt<Written>, StoreError> {
        let one = self.run(&req.header, &[Op::Relocate(req.op)], None, false, false)?;
        single(one, |r| match r {
            OpResult::Written(w) => Some(w),
            _ => None,
        })
    }

    /// Removes a node and every descendant if the `SubtreeRev` matches. The amendment, with the
    /// snapshot of the removed nodes, is written before the first node file is removed.
    ///
    /// # Errors
    /// `Stale`, `Locked`, `NotFound`, `Invalid` (`root_delete_or_move`, `note_required`,
    /// `expected_rev_required`), `Io`.
    pub fn delete_subtree(
        &mut self,
        req: Request<DeleteOp>,
    ) -> Result<Receipt<Deleted>, StoreError> {
        let one = self.run(&req.header, &[Op::Delete(req.op)], None, false, false)?;
        single(one, |r| match r {
            OpResult::Deleted(d) => Some(d),
            _ => None,
        })
    }

    /// Writes one comment file. It has no revision and never conflicts.
    ///
    /// # Errors
    /// `Locked`, `NotFound` for a missing node, `Invalid` (`note_forbidden`), `Io`.
    pub fn comment(&mut self, req: Request<CommentOp>) -> Result<Receipt<Commented>, StoreError> {
        let one = self.run(&req.header, &[Op::Comment(req.op)], None, false, false)?;
        single(one, |r| match r {
            OpResult::Commented(c) => Some(c),
            _ => None,
        })
    }

    /// Applies the ops in order as one transaction: each is checked against the state the
    /// earlier ones produced, the whole result is checked once, and either everything is
    /// written or nothing. `dry_run` runs every check and writes nothing.
    ///
    /// # Errors
    /// The first failing op as `InBatch { index, source }`; `Stale` for a stale
    /// `expected_tree_rev`; `Locked`; `Io`.
    pub fn batch(&mut self, req: Request<BatchOp>) -> Result<Receipt<Vec<OpResult>>, StoreError> {
        let BatchOp {
            expected_tree_rev,
            ops,
            dry_run,
        } = req.op;
        self.run(&req.header, &ops, expected_tree_rev.as_ref(), true, dry_run)
    }

    fn op_relocate(
        &mut self,
        w: &mut Working,
        op: &RelocateOp,
        in_batch: bool,
    ) -> Result<(), StoreError> {
        let id = op.id.clone();
        let current = w.campaign.get(&id).ok_or_else(|| not_found(&id))?.clone();
        self.check_expected(w, &id, op.expected_rev.as_ref(), in_batch)?;
        if op.name.is_none() && op.parent_id.is_none() && op.position.is_none() {
            return Err(request_fault(
                Rule::NothingToDo,
                "none of `name`, `parent_id`, `position` is given",
            ));
        }
        let is_root = current.parent_id().is_none();
        if is_root {
            if op.parent_id.is_some() || op.position.is_some() {
                return Err(node_fault(
                    &id,
                    Rule::RootDeleteOrMove,
                    "the root takes no parent and no order",
                ));
            }
        } else if !w.campaign.is_validated(&id) && op.parent_id.is_none() && op.position.is_some() {
            // A repair is a rename or a move into the validated tree; a new place among the
            // siblings of a node that is not in the tree builds on nothing.
            return Err(node_fault(
                &id,
                Rule::UnvalidatedNode,
                "the node does not trace back to the root",
            ));
        }
        let name = op.name.clone().unwrap_or_else(|| current.name().clone());
        let parent = match (&op.parent_id, current.parent_id()) {
            (Some(p), _) | (None, Some(p)) => Some(p.clone()),
            (None, None) => None,
        };
        let mut order = current.order().cloned();
        if let Some(parent) = &parent {
            let named = op.parent_id.is_some();
            match w.campaign.get(parent) {
                None if named => {
                    return Err(node_fault(
                        &id,
                        Rule::ParentMissing,
                        format!("the parent does not exist: {parent}"),
                    ));
                }
                Some(Node::Leaf(_)) if named => {
                    return Err(node_fault(
                        &id,
                        Rule::ParentNotDirectory,
                        format!("the parent is a leaf: {parent}"),
                    ));
                }
                _ => {}
            }
            // A new parent inside the node's own subtree is a `cycle`: the check of the whole
            // result reports it, with the same rule and file.
            if named && !w.campaign.is_validated(parent) {
                return Err(node_fault(
                    &id,
                    Rule::UnvalidatedNode,
                    "the new parent does not trace back to the root",
                ));
            }
            if w.campaign
                .children(parent)
                .iter()
                .any(|c| *c != id && w.campaign.get(c).is_some_and(|n| n.name() == &name))
            {
                return Err(node_fault(
                    &id,
                    Rule::NameCollision,
                    format!("a sibling is already named `{name}`"),
                ));
            }
            if named || op.position.is_some() {
                let position = op.position.clone().unwrap_or(Position::Last);
                order = Some(place(w, parent, &position, Some(&id))?);
            }
        } else if w
            .campaign
            .nodes()
            .any(|n| n.parent_id().is_none() && n.id() != &id && n.name() == &name)
            && name == *w.campaign.slug()
        {
            return Err(node_fault(
                &id,
                Rule::RootCount,
                "another parentless node already carries this name",
            ));
        }
        // Rebalancing may have rewritten siblings but never the node itself.
        let mut after = w.campaign.get(&id).unwrap_or(&current).clone();
        set_place(&mut after, &name, parent.as_ref(), order.as_ref());
        stamp(&mut after);
        if after.name() == current.name()
            && after.parent_id() == current.parent_id()
            && after.order() == current.order()
        {
            return Err(request_fault(
                Rule::NothingToDo,
                "the node already has this name and place",
            ));
        }
        check_node(&after).map_err(|f| node_fault(&id, f.rule, f.detail))?;
        if after.name() != current.name() {
            w.changes.push(Change {
                node_id: id.clone(),
                op: ChangeOp::Rename,
                item_id: None,
                field: None,
                old: Some(Value::String(current.name().to_string())),
                new: Some(Value::String(after.name().to_string())),
                extra: serde_json::Map::new(),
            });
        }
        if after.parent_id() != current.parent_id() || after.order() != current.order() {
            let place = |n: &Node| {
                serde_json::json!({
                    "parent_id": n.parent_id().map(ToString::to_string),
                    "order": n.order().map(|o| o.as_str().to_owned()),
                })
            };
            w.changes.push(Change {
                node_id: id.clone(),
                op: ChangeOp::Move,
                item_id: None,
                field: None,
                old: Some(place(&current)),
                new: Some(place(&after)),
                extra: serde_json::Map::new(),
            });
        }
        w.structural = true;
        w.campaign.insert(after.clone());
        w.results.push(OpResult::Written(Written {
            id,
            rev: rev::rev(&after),
        }));
        Ok(())
    }

    fn op_delete(
        &mut self,
        w: &mut Working,
        op: &DeleteOp,
        in_batch: bool,
    ) -> Result<(), StoreError> {
        let id = op.id.clone();
        if w.campaign.get(&id).is_none() {
            return Err(not_found(&id));
        }
        if w.campaign.root() == Some(&id) {
            return Err(node_fault(
                &id,
                Rule::RootDeleteOrMove,
                "the campaign root cannot be deleted",
            ));
        }
        let created_here = w.created.contains(&id);
        match (&op.expected_subtree_rev, created_here) {
            (_, true) => {}
            (None, false) => {
                let why = if in_batch {
                    "`expected_subtree_rev` is required for a node the batch did not create"
                } else {
                    "`expected_subtree_rev` is required"
                };
                return Err(request_fault(Rule::ExpectedRevRequired, why));
            }
            (Some(expected), false) => {
                if self.state.campaign.get(&id).is_none() {
                    return Err(not_found(&id));
                }
                let current: SubtreeRev =
                    rev::subtree_rev(self.state.campaign.subtree(&id), &w.disk_revs);
                if &current != expected {
                    return Err(StoreError::Stale {
                        current: Current::Subtree { id, rev: current },
                    });
                }
            }
        }
        let removed: Vec<NodeId> = w.campaign.subtree(&id).cloned().collect();
        for n in &removed {
            if let Some(node) = w.campaign.get(n) {
                w.snapshot.insert(
                    n.to_string(),
                    serde_json::to_value(node).unwrap_or(Value::Null),
                );
            }
            w.changes.push(Change {
                node_id: n.clone(),
                op: ChangeOp::Remove,
                item_id: None,
                field: None,
                old: None,
                new: None,
                extra: serde_json::Map::new(),
            });
        }
        let referrers = self.referrers_into(&removed);
        w.campaign.remove_subtree(&id);
        w.created.retain(|c| !removed.contains(c));
        w.removed.extend(removed.iter().rev().cloned());
        w.structural = true;
        w.results
            .push(OpResult::Deleted(Deleted { removed, referrers }));
        Ok(())
    }
}

/// Builds the node a create writes: the order key comes from the position, never from the node.
fn op_create(w: &mut Working, op: &CreateOp) -> Result<(), StoreError> {
    let mut node = op.node.clone();
    stamp(&mut node);
    let id = node.id().clone();
    if w.campaign.get(&id).is_some() {
        return Err(node_fault(
            &id,
            Rule::DuplicateId,
            "a node with this id exists",
        ));
    }
    match node.parent_id().cloned() {
        None => {
            if w.campaign.nodes().next().is_some() {
                return Err(node_fault(
                    &id,
                    Rule::RootCount,
                    "a root is created into an empty campaign",
                ));
            }
            if node.name() != w.campaign.slug() {
                return Err(node_fault(
                    &id,
                    Rule::RootNameMismatch,
                    "the root is named like the campaign",
                ));
            }
        }
        Some(parent) => {
            let target = w.campaign.get(&parent).ok_or_else(|| {
                node_fault(
                    &id,
                    Rule::ParentMissing,
                    format!("the parent does not exist: {parent}"),
                )
            })?;
            if matches!(target, Node::Leaf(_)) {
                return Err(node_fault(
                    &id,
                    Rule::ParentNotDirectory,
                    format!("the parent is a leaf: {parent}"),
                ));
            }
            if !w.campaign.is_validated(&parent) {
                return Err(node_fault(
                    &id,
                    Rule::UnvalidatedNode,
                    "the parent does not trace back to the root",
                ));
            }
            if w.campaign
                .children(&parent)
                .iter()
                .any(|c| w.campaign.get(c).is_some_and(|n| n.name() == node.name()))
            {
                return Err(node_fault(
                    &id,
                    Rule::NameCollision,
                    format!("a sibling is already named `{}`", node.name()),
                ));
            }
            let position = op.position.clone().unwrap_or(Position::Last);
            let key = place(w, &parent, &position, None)?;
            let name = node.name().clone();
            set_place(&mut node, &name, Some(&parent), Some(&key));
        }
    }
    check_node(&node).map_err(|f| node_fault(&id, f.rule, f.detail))?;
    introduced_references_exist(&w.campaign, None, &node)?;
    introduced_marks_name_criteria(None, &node)?;
    w.changes.push(Change {
        node_id: id.clone(),
        op: ChangeOp::Add,
        item_id: None,
        field: None,
        old: None,
        new: None,
        extra: serde_json::Map::new(),
    });
    w.created.push(id.clone());
    w.structural = true;
    w.campaign.insert(node.clone());
    w.results.push(OpResult::Written(Written {
        id,
        rev: rev::rev(&node),
    }));
    Ok(())
}

fn op_comment(
    w: &mut Working,
    header: &Header,
    op: &CommentOp,
    in_batch: bool,
) -> Result<(), StoreError> {
    if !in_batch && header.note.is_some() {
        return Err(request_fault(
            Rule::NoteForbidden,
            "a comment carries no note",
        ));
    }
    if w.campaign.get(&op.node_id).is_none() {
        return Err(not_found(&op.node_id));
    }
    w.comments.push((op.node_id.clone(), op.text.clone()));
    // The id is minted with the plan; the placeholder is replaced before the result is returned.
    w.results.push(OpResult::Commented(Commented {
        id: placeholder_comment_id(),
    }));
    Ok(())
}

fn placeholder_comment_id() -> CommentId {
    CommentId::parse("00000000000000000000000000").unwrap_or_else(|_| minted_ulid_invalid())
}

/// Sets the place fields of a node. A root takes no parent and no order.
fn set_place(node: &mut Node, name: &Name, parent: Option<&NodeId>, order: Option<&OrderKey>) {
    match node {
        Node::Directory(d) => {
            d.name = name.clone();
            d.parent_id = parent.cloned();
            d.order = order.cloned();
        }
        Node::Leaf(l) => {
            l.name = name.clone();
            if let Some(p) = parent {
                l.parent_id = p.clone();
            }
            if let Some(o) = order {
                l.order = o.clone();
            }
        }
    }
}

/// The validated siblings under `parent`, in `(order, id)` order, leaving out `exclude`.
fn siblings(
    campaign: &Campaign,
    parent: &NodeId,
    exclude: Option<&NodeId>,
) -> Vec<(NodeId, String)> {
    campaign
        .children(parent)
        .iter()
        .filter(|c| Some(*c) != exclude && campaign.is_validated(c))
        .filter_map(|c| Some((c.clone(), campaign.get(c)?.order()?.as_str().to_owned())))
        .collect()
}

/// Resolves a position to an order key. A key past the length limit, or no room between equal
/// neighbours, rewrites every sibling with evenly spread keys first.
fn place(
    w: &mut Working,
    parent: &NodeId,
    position: &Position,
    exclude: Option<&NodeId>,
) -> Result<OrderKey, StoreError> {
    for attempt in 0..2 {
        let sibs = siblings(&w.campaign, parent, exclude);
        let at = |target: &NodeId| -> Result<usize, StoreError> {
            sibs.iter().position(|(id, _)| id == target).ok_or_else(|| {
                request_fault(
                    Rule::PositionNotSibling,
                    format!("{target} is not a sibling under {parent}"),
                )
            })
        };
        let key_at = |i: usize| sibs.get(i).map(|(_, k)| k.as_str());
        let (lower, upper) = match position {
            Position::First => (None, key_at(0)),
            Position::Last => (sibs.len().checked_sub(1).and_then(key_at), None),
            Position::Before(s) => {
                let i = at(s)?;
                (i.checked_sub(1).and_then(key_at), key_at(i))
            }
            Position::After(s) => {
                let i = at(s)?;
                (key_at(i), key_at(i + 1))
            }
        };
        match between(lower, upper) {
            Ok(k) if k.len() <= MAX_KEY_LEN || attempt == 1 => return Ok(OrderKey::new(k)),
            _ if attempt == 0 => rebalance(w, parent, exclude),
            Ok(k) => return Ok(OrderKey::new(k)),
            Err(_) => break,
        }
    }
    Err(request_fault(
        Rule::PositionNotSibling,
        "no order key fits at this position",
    ))
}

/// Rewrites the order keys of the validated siblings under `parent` with evenly spread ones; each
/// rewritten key is a `set` of `/order` in the amendment.
fn rebalance(w: &mut Working, parent: &NodeId, exclude: Option<&NodeId>) {
    let sibs = siblings(&w.campaign, parent, exclude);
    for ((id, old), new) in sibs.iter().zip(spread(sibs.len())) {
        if *old == new {
            continue;
        }
        let Some(node) = w.campaign.get(id) else {
            continue;
        };
        let mut node = node.clone();
        let name = node.name().clone();
        let parent_id = node.parent_id().cloned();
        set_place(
            &mut node,
            &name,
            parent_id.as_ref(),
            Some(&OrderKey::new(new.clone())),
        );
        stamp(&mut node);
        w.changes.push(Change {
            node_id: id.clone(),
            op: ChangeOp::Set,
            item_id: None,
            field: Some("/order".to_owned()),
            old: Some(Value::String(old.clone())),
            new: Some(Value::String(new)),
            extra: serde_json::Map::new(),
        });
        w.campaign.insert(node);
    }
}

/// A mark the write introduces names a criterion the file holds (a declared link, like a
/// reference). Marks already there keep their meaning: a removed criterion stays named by the
/// marks and by the amendment that removed it.
fn introduced_marks_name_criteria(before: Option<&Node>, after: &Node) -> Result<(), StoreError> {
    let value = serde_json::to_value(after).unwrap_or(Value::Null);
    let marks_of = |v: &Value| {
        v.get("marks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let old = before
        .map(|b| marks_of(&serde_json::to_value(b).unwrap_or(Value::Null)))
        .unwrap_or_default();
    let criteria: BTreeSet<&str> = ["preconditions", "success_gates"]
        .iter()
        .filter_map(|k| value.get(*k).and_then(Value::as_array))
        .flatten()
        .filter_map(|c| c.get("id").and_then(Value::as_str))
        .collect();
    for mark in marks_of(&value) {
        if old.contains(&mark) {
            continue;
        }
        let target = mark
            .get("criterion_id")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !criteria.contains(target) {
            return Err(node_fault(
                after.id(),
                Rule::MarkTarget,
                format!("a new mark names a criterion the file does not hold: {target}"),
            ));
        }
    }
    Ok(())
}

/// A recorded effect may touch only a campaign record: a relative path with no `..`, under
/// `nodes/`, `_amendments/` or `_comments/`.
fn effect_path_is_ours(effect: &IntentEffect) -> bool {
    use std::path::{Component, Path};
    let (IntentEffect::Put { path, .. }
    | IntentEffect::Create { path, .. }
    | IntentEffect::Del { path }) = effect;
    let mut parts = Path::new(path).components();
    let under = matches!(
        parts.next(),
        Some(Component::Normal(d)) if ["nodes", "_amendments", "_comments"].iter().any(|k| d == *k)
    );
    let rest: Vec<Component> = parts.collect();
    under && !rest.is_empty() && rest.iter().all(|c| matches!(c, Component::Normal(_)))
}

/// How many ancestors a node has along its `parent_id` chain in `campaign`, counting each node
/// once: a chain that ends at a missing parent stops there, and one that comes back on itself (a
/// cycle) stops at the repeat. A child is always deeper than its parent unless a cycle is in
/// between, which has no order to give.
fn chain_depth(campaign: &Campaign, id: &NodeId) -> usize {
    let mut seen = BTreeSet::from([id]);
    let mut at = campaign.get(id).and_then(Node::parent_id);
    while let Some(parent) = at {
        if !seen.insert(parent) {
            break;
        }
        at = campaign.get(parent).and_then(Node::parent_id);
    }
    seen.len() - 1
}

/// An op that is a batch of one returns exactly one result of its own kind.
fn single<R>(
    receipt: Receipt<Vec<OpResult>>,
    pick: impl FnOnce(OpResult) -> Option<R>,
) -> Result<Receipt<R>, StoreError> {
    let Receipt {
        result,
        amendment,
        tree_rev,
        replayed,
    } = receipt;
    let one = result.into_iter().next().and_then(pick);
    match one {
        Some(result) => Ok(Receipt {
            result,
            amendment,
            tree_rev,
            replayed,
        }),
        None => Err(request_fault(
            Rule::NothingToDo,
            "the operation returned no result",
        )),
    }
}

/// The minter builds 26 characters of the ULID alphabet, which always parse.
fn minted_ulid_invalid() -> ! {
    unreachable!("a minted ULID always parses")
}

/// Rules that report damage between files: a write may not add one.
fn structural_rule(rule: Rule) -> bool {
    matches!(
        rule,
        Rule::ParentMissing
            | Rule::ParentNotDirectory
            | Rule::Cycle
            | Rule::RootCount
            | Rule::RootNameMismatch
            | Rule::NameCollision
            | Rule::DuplicateId
    )
}

/// A write that sets a link to a node checks the node exists (`dangling_id`); only references
/// the write introduces are checked.
fn introduced_references_exist(
    campaign: &Campaign,
    before: Option<&Node>,
    after: &Node,
) -> Result<(), StoreError> {
    let targets = |n: &Node| -> BTreeSet<NodeId> {
        let v = serde_json::to_value(n).unwrap_or(Value::Null);
        let mut out = BTreeSet::new();
        let mut take = |refs: Option<&Value>| {
            for r in refs.and_then(Value::as_array).into_iter().flatten() {
                if let Some(id) = r
                    .get("node_id")
                    .and_then(Value::as_str)
                    .and_then(|s| NodeId::parse(s).ok())
                {
                    out.insert(id);
                }
            }
        };
        take(v.get("references"));
        for s in v
            .get("steps")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            take(s.get("references"));
        }
        out
    };
    let old = before.map(targets).unwrap_or_default();
    for t in targets(after) {
        if !old.contains(&t) && campaign.get(&t).is_none() {
            return Err(node_fault(
                after.id(),
                Rule::DanglingId,
                format!("a reference names a node that does not exist: {t}"),
            ));
        }
    }
    Ok(())
}

/// The actor type is part of the header every request carries.
#[allow(dead_code)]
fn _actor(_: &Actor, _: &Diagnostic) {}
