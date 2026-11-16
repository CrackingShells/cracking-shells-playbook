//! The read model (`read-model.md`): four projections over the stored files.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::stored::{Mark, 
    Actor, Change, Comment, Criterion, Extra, History, Lines, Progress, Reference, Status, Step, Ts,
};

/// The rule that produced a `level`. One value today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LevelSource {
    Depth,
}

/// Which projection a payload is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Projection {
    Outline,
    Node,
    Ready,
    Amendments,
}

/// `NodeKind` of a child reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Leaf,
    Directory,
}

/// Identity of the campaign in an envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CampaignRef {
    pub id: String,
    pub path: String,
    pub title: String,
    #[serde(flatten)]
    pub extra: Extra,
}

/// The fields every projection starts with. Flattened into each view, whose own `extra`
/// keeps the unknown fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Envelope {
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+\.[0-9]+$"))]
    pub read_model_version: String,
    #[schemars(regex(pattern = r"^[0-9]+\.[0-9]+\.[0-9]+$"))]
    pub format_version: String,
    pub projection: Projection,
    pub campaign: CampaignRef,
    pub tree_rev: String,
}

/// Leaves counted by status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StatusCounts {
    pub planned: u32,
    pub ongoing: u32,
    pub paused: u32,
    pub cancelled: u32,
    pub done: u32,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `{done, total}` of a leaf's steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StepCounts {
    pub done: u32,
    pub total: u32,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A leaf that holds another back.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Holder {
    pub path: String,
    pub title: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// One wave of the outline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WaveView {
    pub level: u32,
    pub level_source: LevelSource,
    pub leaf_ids: Vec<String>,
    pub counts: StatusCounts,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A directory in the outline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OutlineDirectory {
    pub id: String,
    pub path: String,
    pub title: String,
    pub is_campaign: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    pub status: Status,
    pub counts: StatusCounts,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment_count: Option<u32>,
    pub rev: String,
    pub children: Vec<OutlineNode>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A leaf in the outline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OutlineLeaf {
    pub id: String,
    pub path: String,
    pub title: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<String>,
    pub level: u32,
    pub level_source: LevelSource,
    pub ready: bool,
    pub steps: StepCounts,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_by: Option<String>,
    pub rev: String,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A node of the outline, discriminated by `kind`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum OutlineNode {
    Directory(OutlineDirectory),
    Leaf(OutlineLeaf),
}

/// The campaign outline with waves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OutlineView {
    #[serde(flatten)]
    pub envelope: Envelope,
    pub root: OutlineNode,
    pub waves: Vec<WaveView>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A stored step plus its derived number.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StepView {
    pub number: u32,
    #[serde(flatten)]
    pub step: Step,
}

/// A child in a directory's detail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ChildRef {
    pub id: String,
    pub path: String,
    pub kind: NodeKind,
    pub title: String,
    pub status: Status,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Detail of a leaf.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LeafDetail {
    pub id: String,
    pub path: String,
    pub parent: String,
    pub rev: String,
    pub title: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_since: Option<Ts>,
    pub level: u32,
    pub level_source: LevelSource,
    pub ready: bool,
    pub held_by: Vec<String>,
    pub sealed: bool,
    pub goal: String,
    pub preconditions: Vec<Criterion>,
    pub success_gates: Vec<Criterion>,
    pub references: Vec<Reference>,
    pub steps: Vec<StepView>,
    pub progress: Progress,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_count: Option<u64>,
    pub step_progress: StepCounts,
    pub history: Vec<History>,
    pub marks: Vec<Mark>,
    pub comments: Vec<Comment>,
    pub amendments: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_by: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Detail of a directory or the campaign root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DirectoryDetail {
    pub id: String,
    pub path: String,
    pub parent: String,
    pub rev: String,
    pub title: String,
    pub sealed: bool,
    pub comments: Vec<Comment>,
    pub amendments: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_by: Option<String>,
    pub is_campaign: bool,
    pub status: Status,
    pub counts: StatusCounts,
    pub context: Lines,
    pub goal: String,
    pub preconditions: Vec<Criterion>,
    pub success_gates: Vec<Criterion>,
    pub references: Vec<Reference>,
    pub gotchas: Lines,
    pub notes: Lines,
    pub marks: Vec<Mark>,
    pub children: Vec<ChildRef>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// The detail of one node, discriminated by `kind`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum NodeDetail {
    Leaf(LeafDetail),
    Directory(DirectoryDetail),
}

/// Node detail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NodeView {
    #[serde(flatten)]
    pub envelope: Envelope,
    pub node: NodeDetail,
    pub holders: BTreeMap<String, Holder>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A `planned` leaf that nothing holds back.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReadyLeaf {
    pub id: String,
    pub path: String,
    pub title: String,
    pub level: u32,
    pub level_source: LevelSource,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A `planned` leaf held back by unfinished leaves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StalledLeaf {
    pub id: String,
    pub path: String,
    pub title: String,
    pub level: u32,
    pub level_source: LevelSource,
    pub held_by: Vec<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// The ready set with what stalls the rest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReadyView {
    #[serde(flatten)]
    pub envelope: Envelope,
    pub ready: Vec<ReadyLeaf>,
    pub stalled: Vec<StalledLeaf>,
    pub holders: BTreeMap<String, Holder>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A stored change plus the node's current path and title.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ChangeView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_title: Option<String>,
    #[serde(flatten)]
    pub change: Change,
}

/// One amendment of the log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AmendmentEntry {
    pub id: String,
    pub display_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_id: Option<String>,
    pub ts: Ts,
    pub actor: Actor,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_id: Option<String>,
    pub has_snapshot: bool,
    pub changes: Vec<ChangeView>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// The amendment log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AmendmentsView {
    #[serde(flatten)]
    pub envelope: Envelope,
    pub total: u32,
    pub amendments: Vec<AmendmentEntry>,
    #[serde(flatten)]
    pub extra: Extra,
}
