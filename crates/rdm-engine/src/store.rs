//! The campaign store (`store.md`): opens a campaign from disk into the pure
//! [`rdm_schema::campaign::Campaign`] model, validates it, computes revisions, keeps the reverse
//! link index, and applies every primitive through ordered writes under the writer lock.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rdm_schema::campaign::{Campaign, Diagnostic, node_file};
use rdm_schema::error::{Rule, StoreError};
use rdm_schema::ids::{AmendmentId, CommentId, Name, NodeId, OpId, Rev, SubtreeRev, TreeRev};
use rdm_schema::stored::{Amendment, ChangeOp, Comment, Node};

use crate::codec::{Stored, decode};
use crate::fs::{Durability, Fs, RealFs};
use crate::ids::{IdRng, OsRng, UlidMint};
use crate::links::ReverseIndex;
use crate::rev;

/// How a store runs. Every field has a default; tests replace the generator, the clock and the
/// file system.
pub struct Options {
    /// Sync tier of every publish (`store.md`, §Durability).
    pub durability: Durability,
    /// How long a writer waits for the lock before it fails with `Locked`.
    pub lock_timeout: Duration,
    /// The file system every read and write goes through. A `RecordingFs` logs the writes.
    pub fs: Arc<dyn Fs + Send + Sync>,
    /// Draws the random parts of the ULIDs the store mints.
    pub rng: Box<dyn IdRng + Send>,
    /// The wall clock, for the instants the store stamps.
    pub clock: Box<dyn Fn() -> SystemTime + Send + Sync>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            durability: Durability::default(),
            lock_timeout: Duration::from_secs(5),
            fs: Arc::new(RealFs),
            rng: Box::new(OsRng),
            clock: Box::new(SystemTime::now),
        }
    }
}

/// Everything read from the files of one campaign, and what is derived from it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct State {
    pub(crate) campaign: Campaign,
    pub(crate) comments: BTreeMap<CommentId, Comment>,
    pub(crate) amendments: BTreeMap<AmendmentId, Amendment>,
    pub(crate) revs: BTreeMap<NodeId, Rev>,
    /// The warnings of each file the codec read (`unknown_key`), by file.
    pub(crate) file_diags: BTreeMap<String, Vec<Diagnostic>>,
    pub(crate) reverse: ReverseIndex,
    pub(crate) ops: OpIndex,
}

/// A record that carries an `op_id`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum OpRef {
    Amendment(AmendmentId),
    Comment(CommentId),
    /// A node with a `History.op_id` entry.
    History(NodeId),
}

/// Maps an `op_id` to the records that carry it (`store.md`, §Cache): amendments, comments and
/// `History.op_id` entries. Built on open and kept in step by write-through.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct OpIndex {
    by_op: BTreeMap<OpId, BTreeSet<OpRef>>,
    /// The op ids each node's history holds, so a replaced node drops its old entries.
    by_node: BTreeMap<NodeId, Vec<OpId>>,
}

impl OpIndex {
    pub(crate) fn records(&self, op: &OpId) -> Option<&BTreeSet<OpRef>> {
        self.by_op.get(op)
    }

    fn add(&mut self, op: &OpId, r: OpRef) {
        self.by_op.entry(op.clone()).or_default().insert(r);
    }

    pub(crate) fn add_amendment(&mut self, a: &Amendment) {
        if let Some(op) = &a.op_id {
            self.add(op, OpRef::Amendment(a.id.clone()));
        }
    }

    pub(crate) fn add_comment(&mut self, c: &Comment) {
        if let Some(op) = &c.op_id {
            self.add(op, OpRef::Comment(c.id.clone()));
        }
    }

    /// Replaces what the history of node `id` contributes; `None`: the node is gone.
    pub(crate) fn replace_node(&mut self, id: &NodeId, node: Option<&Node>) {
        for op in self.by_node.remove(id).unwrap_or_default() {
            if let Some(set) = self.by_op.get_mut(&op) {
                set.remove(&OpRef::History(id.clone()));
                if set.is_empty() {
                    self.by_op.remove(&op);
                }
            }
        }
        let Some(Node::Leaf(leaf)) = node else { return };
        let mut ops: Vec<OpId> = leaf
            .history
            .iter()
            .filter_map(|h| h.op_id.clone())
            .collect();
        ops.sort();
        ops.dedup();
        for op in &ops {
            self.add(op, OpRef::History(id.clone()));
        }
        if !ops.is_empty() {
            self.by_node.insert(id.clone(), ops);
        }
    }
}

/// An open campaign.
pub struct Store {
    pub(crate) root: PathBuf,
    pub(crate) options: Options,
    pub(crate) ulids: UlidMint,
    pub(crate) state: State,
}

pub(crate) fn io_error(file: &str, e: &io::Error) -> StoreError {
    StoreError::Io {
        file: file.to_owned(),
        message: e.to_string(),
    }
}

/// The file names a loader reads in a directory: `<ulid>.json` or `n_<body>.json` and no other.
fn listed(
    fs: &dyn Fs,
    root: &Path,
    dir: &str,
    keep: impl Fn(&str) -> bool,
) -> Result<Vec<String>, StoreError> {
    match fs.list_dir(&root.join(dir)) {
        Ok(names) => Ok(names
            .into_iter()
            .filter(|n| n.strip_suffix(".json").is_some_and(&keep))
            .collect()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(io_error(dir, &e)),
    }
}

fn invalid(file: &str, rule: Rule, detail: String) -> StoreError {
    StoreError::Invalid {
        file: file.to_owned(),
        rule,
        detail,
    }
}

/// Reads every file of a kind, decodes it, and checks that it holds the id it is named for.
fn read_all<T: Stored>(
    fs: &dyn Fs,
    root: &Path,
    dir: &str,
    keep: impl Fn(&str) -> bool,
    id_of: impl Fn(&T) -> String,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<Vec<T>, StoreError> {
    let mut out = Vec::new();
    for name in listed(fs, root, dir, keep)? {
        let file = format!("{dir}/{name}");
        let bytes = fs
            .read(&root.join(dir).join(&name))
            .map_err(|e| io_error(&file, &e))?;
        let decoded = decode::<T>(&file, &bytes)?;
        let named = name.strip_suffix(".json").unwrap_or(&name);
        let held = id_of(&decoded.value);
        if held != named {
            return Err(invalid(
                &file,
                Rule::FileIdMismatch,
                format!("the file is named {named} and holds the id {held}"),
            ));
        }
        diagnostics.extend(decoded.diagnostics);
        out.push(decoded.value);
    }
    Ok(out)
}

/// Reads the whole campaign from disk.
pub(crate) fn load(fs: &dyn Fs, root: &Path, slug: &Name) -> Result<State, StoreError> {
    let mut diagnostics = Vec::new();
    let nodes: Vec<Node> = read_all(
        fs,
        root,
        "nodes",
        |s| NodeId::parse(s).is_ok(),
        |n: &Node| n.id().to_string(),
        &mut diagnostics,
    )?;
    let comments: Vec<Comment> = read_all(
        fs,
        root,
        "_comments",
        |s| CommentId::parse(s).is_ok(),
        |c: &Comment| c.id.to_string(),
        &mut diagnostics,
    )?;
    let amendments: Vec<Amendment> = read_all(
        fs,
        root,
        "_amendments",
        |s| AmendmentId::parse(s).is_ok(),
        |a: &Amendment| a.id.to_string(),
        &mut diagnostics,
    )?;
    let opened = Campaign::open(nodes, slug)?;
    let campaign = opened.campaign;
    let mut file_diags: BTreeMap<String, Vec<Diagnostic>> = BTreeMap::new();
    for d in diagnostics {
        file_diags.entry(d.file.clone()).or_default().push(d);
    }
    let revs = campaign
        .nodes()
        .map(|n| (n.id().clone(), rev::rev(n)))
        .collect();
    let comments: BTreeMap<CommentId, Comment> =
        comments.into_iter().map(|c| (c.id.clone(), c)).collect();
    let amendments: BTreeMap<AmendmentId, Amendment> =
        amendments.into_iter().map(|a| (a.id.clone(), a)).collect();
    let reverse = ReverseIndex::build(
        &campaign,
        &comments.values().cloned().collect::<Vec<_>>(),
        &amendments.values().cloned().collect::<Vec<_>>(),
    );
    let mut ops = OpIndex::default();
    amendments.values().for_each(|a| ops.add_amendment(a));
    comments.values().for_each(|c| ops.add_comment(c));
    campaign
        .nodes()
        .for_each(|n| ops.replace_node(n.id(), Some(n)));
    Ok(State {
        campaign,
        comments,
        amendments,
        revs,
        file_diags,
        reverse,
        ops,
    })
}

impl Store {
    /// Opens the campaign at `root`, given the slug its caller knows it by: the root is the
    /// parentless node that carries it.
    ///
    /// # Errors
    /// A per-file fault refuses the open, naming the file: `Invalid` (an unparsable file, a
    /// malformed id, `file_id_mismatch`, a rule of the node itself), `NewerFormat`, or `Io`.
    /// Damage between files loads, and is in [`Store::diagnostics`].
    pub fn open(
        root: impl Into<PathBuf>,
        slug: &Name,
        options: Options,
    ) -> Result<Self, StoreError> {
        let root = root.into();
        let state = load(&*options.fs, &root, slug)?;
        Ok(Self {
            root,
            options,
            ulids: UlidMint::default(),
            state,
        })
    }

    /// The campaign root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The model: nodes, tree, validated set.
    #[must_use]
    pub fn campaign(&self) -> &Campaign {
        &self.state.campaign
    }

    /// Every warning of the files read, and the damage between them, sorted by
    /// `(file, rule, detail)`.
    #[must_use]
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut all: Vec<Diagnostic> = self.state.file_diags.values().flatten().cloned().collect();
        all.extend(self.state.campaign.diagnostics().iter().cloned());
        all.extend(self.mark_diagnostics());
        all.sort();
        all
    }

    /// One `mark_target` per mark that names a criterion neither in its file nor in an amendment
    /// that removed it: a mark is a declared id link like a reference.
    fn mark_diagnostics(&self) -> Vec<Diagnostic> {
        let removed: BTreeSet<(String, String)> = self
            .state
            .amendments
            .values()
            .flat_map(|a| &a.changes)
            .filter(|c| c.op == ChangeOp::Remove)
            .filter_map(|c| {
                let item = serde_json::to_value(c.item_id.as_ref()?).ok()?;
                Some((c.node_id.to_string(), item.as_str()?.to_owned()))
            })
            .collect();
        let mut out = Vec::new();
        for node in self.state.campaign.nodes() {
            let value = serde_json::to_value(node).unwrap_or(serde_json::Value::Null);
            let held: BTreeSet<&str> = ["preconditions", "success_gates"]
                .iter()
                .filter_map(|k| value.get(*k).and_then(serde_json::Value::as_array))
                .flatten()
                .filter_map(|c| c.get("id").and_then(serde_json::Value::as_str))
                .collect();
            let marks = value.get("marks").and_then(serde_json::Value::as_array);
            for (i, mark) in marks.into_iter().flatten().enumerate() {
                let target = mark
                    .get("criterion_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                if !held.contains(target)
                    && !removed.contains(&(node.id().to_string(), target.to_owned()))
                {
                    out.push(Diagnostic {
                        file: node_file(node.id()),
                        rule: Rule::MarkTarget,
                        detail: format!(
                            "/marks/{i} names the criterion {target}, which the file does not hold"
                        ),
                    });
                }
            }
        }
        out
    }

    /// The reverse link index.
    #[must_use]
    pub fn reverse(&self) -> &ReverseIndex {
        &self.state.reverse
    }

    /// The comments read, by id.
    #[must_use]
    pub fn comments(&self) -> &BTreeMap<CommentId, Comment> {
        &self.state.comments
    }

    /// The amendments read, by id.
    #[must_use]
    pub fn amendments(&self) -> &BTreeMap<AmendmentId, Amendment> {
        &self.state.amendments
    }

    /// The `rev` of a node as read.
    #[must_use]
    pub fn rev(&self, id: &NodeId) -> Option<&Rev> {
        self.state.revs.get(id)
    }

    /// The `tree_rev` of the nodes as read.
    #[must_use]
    pub fn tree_rev(&self) -> TreeRev {
        rev::tree_rev(&self.state.revs)
    }

    /// The `SubtreeRev` of a node and its descendants; `None` for a node that is not loaded.
    #[must_use]
    pub fn subtree_rev(&self, id: &NodeId) -> Option<SubtreeRev> {
        self.state.campaign.get(id)?;
        Some(rev::subtree_rev(
            self.state.campaign.subtree(id),
            &self.state.revs,
        ))
    }

    /// The relative path of a node's file.
    #[must_use]
    pub fn file_of(id: &NodeId) -> String {
        node_file(id)
    }
}
