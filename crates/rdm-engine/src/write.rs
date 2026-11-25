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
use rdm_schema::ids::{AmendmentId, CommentId, NodeId, OpId, Rev};
use rdm_schema::links::{LINKS, OnDelete, RecordRef};
use rdm_schema::ops::{
    Commented, CreateOp, Deleted, Header, Op, OpResult, Receipt, ReplaceOp, Request, Written,
};
use rdm_schema::stored::{Actor, Amendment, Change, Comment, Node};
use rdm_schema::validate::check_node;
use rdm_schema::{FORMAT_VERSION, version_string};
use serde_json::Value;

use crate::clock::{rfc3339, unix_ms};
use crate::codec::{decode, encode};
use crate::fs::Fs;
use crate::intent::{self, Intent, IntentEffect, Pending};
use crate::links::Record;
use crate::lock::{LockError, WriterLock};
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
        self.recover_locked()?;
        // 3. Read the campaign from disk, not from what an earlier read held.
        self.state = load(self.fs(), &self.root, self.state.campaign.slug())?;

        // 3.1 Replay.
        if let Some(op_id) = &header.op_id
            && let Some(receipt) = self.replay(op_id, ops)
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
        };
        for (i, op) in ops.iter().enumerate() {
            self.apply_op(&mut w, header, op, in_batch)
                .map_err(|e| wrap(i, e))?;
        }
        self.check_whole(&w, header).map_err(|e| wrap(0, e))?;
        // 4. Every after-image, the amendment's ULID.
        let plan = self.plan(&w, header);
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

    /// 3.1: the first outcome of a known `op_id`, with nothing written.
    fn replay(&self, op_id: &OpId, ops: &[Op]) -> Option<Receipt<Vec<OpResult>>> {
        let records = self.state.ops.records(op_id)?;
        let amendment = records.iter().find_map(|r| match r {
            OpRef::Amendment(id) => self.state.amendments.get(id),
            _ => None,
        });
        let mut comment_ids = records.iter().filter_map(|r| match r {
            OpRef::Comment(id) => Some(id.clone()),
            _ => None,
        });
        let mut results = Vec::new();
        for op in ops {
            let result = match op {
                Op::Create(CreateOp { node, .. }) | Op::Replace(ReplaceOp { node, .. }) => {
                    self.written(node.id())
                }
                Op::Relocate(r) => self.written(&r.id),
                Op::Delete(d) => OpResult::Deleted(self.replayed_delete(&d.id, amendment)),
                Op::Comment(_) => OpResult::Commented(Commented {
                    id: comment_ids.next()?,
                }),
            };
            results.push(result);
        }
        Some(Receipt {
            result: results,
            amendment: amendment.map(|a| a.id.clone()),
            tree_rev: self.tree_rev(),
            replayed: true,
        })
    }

    fn written(&self, id: &NodeId) -> OpResult {
        let rev = self
            .state
            .revs
            .get(id)
            .cloned()
            .unwrap_or_else(|| Rev::new(String::new()));
        OpResult::Written(Written {
            id: id.clone(),
            rev,
        })
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
            Op::Create(_) | Op::Relocate(_) | Op::Delete(_) | Op::Comment(_) => Err(request_fault(
                Rule::NothingToDo,
                "this primitive is not available yet",
            )),
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
                current => Err(StoreError::Stale {
                    current: Current::Node {
                        id: id.clone(),
                        rev: current.cloned(),
                    },
                }),
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
        introduced_references_exist(&w.campaign, &current, &after)?;
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
        for d in w.campaign.diagnostics() {
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
        // Deepest first: the reverse of the pre-order of every removed subtree.
        let gone: BTreeSet<NodeId> = before
            .nodes()
            .map(|n| n.id().clone())
            .filter(|id| w.campaign.get(id).is_none())
            .collect();
        let mut deletes = Vec::new();
        for id in &gone {
            let parent_gone = before
                .get(id)
                .and_then(Node::parent_id)
                .is_some_and(|p| gone.contains(p));
            if !parent_gone {
                let mut sub: Vec<NodeId> = before.subtree(id).cloned().collect();
                sub.reverse();
                deletes.extend(sub);
            }
        }
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
    before: &Node,
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
    let old = targets(before);
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
