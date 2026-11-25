//! One request type and one result type per store primitive (`contracts.md`, `Operations`).
//!
//! These are the signatures M2 composes and M4 wraps as tool inputs; tool envelopes, names and
//! output budgets are M4's. The trait that implements them lives in `rdm-engine`, because this
//! crate holds no I/O.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ids::{AmendmentId, CommentId, Name, NodeId, OpId, Rev, SubtreeRev, TreeRev};
use crate::links::Referrer;
use crate::stored::{Actor, Lines, Node};

/// What every request carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Header {
    /// The retry key: a retried write with a known `op_id` returns the first outcome.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_id: Option<OpId>,
    /// Stamped on the amendment or comment the operation writes.
    pub actor: Actor,
    /// The amendment's words (`The amendment rule`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<EditNote>,
}

/// The note of an amendment: why a content or structure edit was made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EditNote {
    /// Non-blank. When `correction` is true, it is the reason.
    pub description: String,
    /// The report that justified the edit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub correction: bool,
}

/// A request: a header and one operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Request<T> {
    pub header: Header,
    pub op: T,
}

/// Where a node goes among its siblings. The store resolves it to an `OrderKey`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    First,
    Last,
    /// Just before this sibling.
    Before(NodeId),
    /// Just after this sibling.
    After(NodeId),
}

/// Writes a new node file. The ids in `node` are minted by the caller; `node.order` is not read,
/// the store sets it from `position`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CreateOp {
    pub node: Node,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// Replaces one node file if its current `rev` is `expected_rev`. It cannot change `id`, `name`,
/// `parent_id` or `order`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplaceOp {
    /// Required outside a batch; inside one, `None` only for a node the batch created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_rev: Option<Rev>,
    pub node: Node,
}

/// Renames or moves one node: changes `name`, `parent_id` and `order`. A field left `None` is
/// kept; a new `parent_id` with no `position` means `Last`. At least one is given.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RelocateOp {
    pub id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_rev: Option<Rev>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Name>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// Removes a node and every descendant if the `SubtreeRev` matches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DeleteOp {
    pub id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_subtree_rev: Option<SubtreeRev>,
}

/// Writes one comment file. It has no revision: it is append-only and never conflicts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CommentOp {
    pub node_id: NodeId,
    pub text: Lines,
}

/// Applies `ops` in order as one transaction: all or nothing. `dry_run` runs every check and
/// writes nothing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BatchOp {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_tree_rev: Option<TreeRev>,
    pub ops: Vec<Op>,
    pub dry_run: bool,
}

/// One operation of a batch. A batch has no nested batch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    Create(CreateOp),
    Replace(ReplaceOp),
    Relocate(RelocateOp),
    Delete(DeleteOp),
    Comment(CommentOp),
}

/// The result of a create, a replace or a relocate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Written {
    pub id: NodeId,
    pub rev: Rev,
}

/// The result of a delete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Deleted {
    /// The removed nodes, in pre-order.
    pub removed: Vec<NodeId>,
    /// The records outside the subtree that point into it, `Report` links only.
    pub referrers: Vec<Referrer>,
}

/// The result of a comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Commented {
    pub id: CommentId,
}

/// The result of one op of a batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpResult {
    Written(Written),
    Deleted(Deleted),
    Commented(Commented),
}

/// What a primitive returns: its result and where the campaign stands after it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Receipt<R> {
    pub result: R,
    /// The one amendment the operation wrote, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment: Option<AmendmentId>,
    /// After the write.
    pub tree_rev: TreeRev,
    /// `true`: nothing was written and the first outcome is returned.
    pub replayed: bool,
}
