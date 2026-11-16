//! The stored schema (`stored-schema.md`). Field order is the canonical order.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

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
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+$"))]
    pub schema_version: String,
    pub id: String,
    pub kind: DirectoryKind,
    pub title: String,
    pub context: Lines,
    pub references: Vec<Reference>,
    pub goal: String,
    pub preconditions: Vec<Criterion>,
    pub success_gates: Vec<Criterion>,
    pub gotchas: Lines,
    pub notes: Lines,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `<leaf>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Leaf {
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+$"))]
    pub schema_version: String,
    pub id: String,
    pub kind: LeafKind,
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
    #[serde(flatten)]
    pub extra: Extra,
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
    pub id: String,
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
    pub op_id: Option<String>,
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
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+$"))]
    pub schema_version: String,
    pub id: String,
    pub node_id: String,
    pub ts: Ts,
    pub actor: Actor,
    pub text: Lines,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_id: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `_amendments/<ulid>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Amendment {
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+$"))]
    pub schema_version: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_id: Option<String>,
    pub ts: Ts,
    pub actor: Actor,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_id: Option<String>,
    pub changes: Vec<Change>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<Value>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// An element of `Amendment.changes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Change {
    pub node_id: String,
    pub op: ChangeOp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_after: Option<String>,
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
