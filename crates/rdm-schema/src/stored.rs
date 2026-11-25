//! The stored schema (`stored-schema.md`). Field order is the canonical order.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::error::{Rule, StoreError};
use crate::ids::{
    AmendmentId, CommentId, CriterionId, ItemId, MALFORMED_ID_TAG, Name, NodeId, OpId, OrderKey, StepId,
};

/// A multi-line prose field, stored as one line per element.
pub type Lines = Vec<String>;
/// A UTC instant, RFC 3339.
pub type Ts = String;

/// Unknown keys of an object, kept in order and written back after the known ones (I12).
pub type Extra = Map<String, Value>;

/// The five statuses of a leaf (`statuses.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Planned,
    Ongoing,
    Paused,
    Cancelled,
    Done,
}

/// `Directory.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DirectoryKind {
    Campaign,
    Directory,
}

/// `Leaf.kind`, the one value `leaf`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LeafKind {
    Leaf,
}

/// `Check.expected`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Expected {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "FAIL")]
    Fail,
}

/// `Actor.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ActorKind {
    Agent,
    Human,
    Import,
}

/// `Change.op`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChangeOp {
    Add,
    Remove,
    Set,
    Rename,
    Move,
}

/// `_dir.json`: the campaign root and every directory node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Directory {
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+\.[0-9]+$"))]
    pub schema_version: String,
    pub id: NodeId,
    pub kind: DirectoryKind,
    /// The path segment; for the campaign root, the campaign slug.
    pub name: Name,
    /// The directory this node is in. Absent exactly for the campaign root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<NodeId>,
    /// Position among siblings. Absent exactly for the campaign root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<OrderKey>,
    pub title: String,
    pub context: Lines,
    pub references: Vec<Reference>,
    pub goal: String,
    pub preconditions: Vec<Criterion>,
    pub success_gates: Vec<Criterion>,
    pub gotchas: Lines,
    pub notes: Lines,
    pub marks: Vec<Mark>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `<leaf>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Leaf {
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+\.[0-9]+$"))]
    pub schema_version: String,
    pub id: NodeId,
    pub kind: LeafKind,
    /// The path segment, unique among the children of one parent.
    pub name: Name,
    /// The directory this leaf is in.
    pub parent_id: NodeId,
    /// Position among siblings, opaque.
    pub order: OrderKey,
    pub title: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<String>,
    pub goal: String,
    pub preconditions: Vec<Criterion>,
    pub success_gates: Vec<Criterion>,
    pub references: Vec<Reference>,
    pub steps: Vec<Step>,
    pub progress: Progress,
    pub history: Vec<History>,
    pub marks: Vec<Mark>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A node of the campaign: a directory (or the campaign root) or a leaf.
///
/// Not a serde-tagged enum: `Directory.kind` already holds `campaign` or `directory`, so the
/// deserialiser reads `kind` and picks the type.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Node {
    Directory(Directory),
    Leaf(Leaf),
}

impl Node {
    /// The node's identity, immutable (I7).
    #[must_use]
    pub fn id(&self) -> &NodeId {
        match self {
            Self::Directory(d) => &d.id,
            Self::Leaf(l) => &l.id,
        }
    }

    /// The node's path segment; for the campaign root, the campaign slug.
    #[must_use]
    pub fn name(&self) -> &Name {
        match self {
            Self::Directory(d) => &d.name,
            Self::Leaf(l) => &l.name,
        }
    }

    /// The directory the node is in; `None` for the campaign root.
    #[must_use]
    pub fn parent_id(&self) -> Option<&NodeId> {
        match self {
            Self::Directory(d) => d.parent_id.as_ref(),
            Self::Leaf(l) => Some(&l.parent_id),
        }
    }

    /// The node's position among its siblings; `None` for the campaign root.
    #[must_use]
    pub fn order(&self) -> Option<&OrderKey> {
        match self {
            Self::Directory(d) => d.order.as_ref(),
            Self::Leaf(l) => Some(&l.order),
        }
    }
}

impl Node {
    /// Reads a node from JSON by its `kind`: `leaf` is a [`Leaf`], `campaign` and `directory`
    /// are a [`Directory`].
    ///
    /// # Errors
    /// `Invalid` with the rule `kind` when `kind` is missing or another value, and with the rule
    /// `parse` when the value is not an object of the type its `kind` names. The file is
    /// `<request>`: the codec, which knows the file, names it.
    pub fn decode(raw: Value) -> Result<Node, StoreError> {
        let invalid = |rule, detail: String| StoreError::invalid_request(rule, detail);
        match raw.get("kind").and_then(Value::as_str) {
            Some("leaf") => serde_json::from_value(raw).map(Self::Leaf).map_err(|e| invalid(Rule::Parse, e.to_string())),
            Some("campaign" | "directory") => {
                serde_json::from_value(raw).map(Self::Directory).map_err(|e| invalid(Rule::Parse, e.to_string()))
            }
            _ => Err(invalid(Rule::Kind, "`kind` is not `leaf`, `directory` or `campaign`".to_owned())),
        }
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        Self::decode(Value::deserialize(d)?).map_err(D::Error::custom)
    }
}

/// `Leaf.progress`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Progress {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commits: Option<u64>,
    pub notes: Lines,
    #[serde(flatten)]
    pub extra: Extra,
}

/// An element of `Leaf.steps`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Step {
    pub id: StepId,
    pub title: String,
    pub goal: String,
    pub implementation_logic: Lines,
    pub deliverables: Lines,
    pub references: Vec<Reference>,
    pub checks: Vec<Check>,
    pub commit: Commit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<Lines>,
    pub done: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done_at: Option<Ts>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// An element of `Step.checks`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Check {
    pub text: String,
    pub expected: Expected,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// An element of `preconditions` or `success_gates`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Criterion {
    /// Stable id, minted at creation, never a position (`Stable identity`).
    pub id: CriterionId,
    pub text: String,
    pub met: bool,
    #[serde(flatten)]
    pub extra: Extra,
}

/// An element of `references`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Reference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// The campaign node the reference names, when it names one. `target` is then absent: it is
    /// derived on read from the path of that node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `Step.commit`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Commit {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub breaking: bool,
    pub summary: String,
    #[serde(flatten)]
    pub extra: Extra,
}

/// An element of `Leaf.history`: one status move.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct History {
    pub ts: Ts,
    pub from: Status,
    pub to: Status,
    pub actor: Actor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#override: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_id: Option<OpId>,
    /// `true` for a reversal out of `done`; `reason` is then required and not blank.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction: Option<bool>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// An element of `Leaf.marks` or `Directory.marks`: one change of one criterion's `met`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Mark {
    pub ts: Ts,
    pub actor: Actor,
    /// The `id` of the criterion, in `preconditions` or `success_gates`. Never a position.
    pub criterion_id: CriterionId,
    /// The new value of `met`.
    pub met: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    /// Why the mark was made. Required and not blank when `correction` is `true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// `true` when the mark was made on a finished node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction: Option<bool>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Who made a move, a comment or an amendment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Actor {
    pub kind: ActorKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `_comments/<ulid>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Comment {
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+\.[0-9]+$"))]
    pub schema_version: String,
    pub id: CommentId,
    pub node_id: NodeId,
    pub ts: Ts,
    pub actor: Actor,
    pub text: Lines,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_id: Option<OpId>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `_amendments/<ulid>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Amendment {
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+\.[0-9]+$"))]
    pub schema_version: String,
    pub id: AmendmentId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_id: Option<String>,
    pub ts: Ts,
    pub actor: Actor,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_id: Option<OpId>,
    /// `true` for an edit of a finished node or of a step recorded done; `description` is its reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction: Option<bool>,
    pub changes: Vec<Change>,
    /// The removed subtree of a delete, as stored JSON keyed by `NodeId`. The keys are links; the
    /// values are historical copies.
    #[serde(default, deserialize_with = "snapshot_keys", skip_serializing_if = "Option::is_none")]
    #[schemars(schema_with = "snapshot_schema")]
    pub snapshot: Option<Map<String, Value>>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// An element of `Amendment.changes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Change {
    pub node_id: NodeId,
    pub op: ChangeOp,
    /// The id of the sub-item (a criterion or a step) the change names; `field` is then a pointer
    /// inside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_id: Option<ItemId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(default, deserialize_with = "some_value", skip_serializing_if = "Option::is_none")]
    pub old: Option<Value>,
    #[serde(default, deserialize_with = "some_value", skip_serializing_if = "Option::is_none")]
    pub new: Option<Value>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Keeps a JSON `null` as `Some(Value::Null)`, so "absent" and "null" stay distinct.
fn some_value<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Value>, D::Error> {
    Value::deserialize(d).map(Some)
}

/// A `snapshot` is an object whose keys are node ids: a key that is not a `NodeId` is refused.
fn snapshot_keys<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Map<String, Value>>, D::Error> {
    let map = Option::<Map<String, Value>>::deserialize(d)?;
    if let Some(m) = &map {
        for key in m.keys() {
            NodeId::parse(key).map_err(|e| serde::de::Error::custom(format!("{MALFORMED_ID_TAG}snapshot key {key:?}: {e}")))?;
        }
    }
    Ok(map)
}

fn snapshot_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    let key = generator.subschema_for::<NodeId>();
    schemars::json_schema!({ "type": ["object", "null"], "propertyNames": key })
}
