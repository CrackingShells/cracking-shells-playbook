//! Rule codes, the error type of every store operation and the lock record
//! (`contracts.md`, `Errors`).

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ids::{NodeId, Rev, SubtreeRev, TreeRev};
use crate::links::RecordRef;
use crate::stored::Ts;
use crate::{Version, version_string};

/// A rule a file or a request can break, or a warning a load reports. Each variant has a stable
/// snake-case code, its serde name; M2 may add codes without changing existing ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Rule {
    /// The file is not JSON, or not an object of the expected type.
    Parse,
    /// `schema_version` is missing, not a string, or not `N.N.N`.
    SchemaVersion,
    /// No layer reads this version though it is not newer.
    UnknownFormat,
    /// A layer reports `NotInVersion` for a behaviour the file needs.
    NotInVersion,
    /// `kind` is not `leaf`, `directory` or `campaign`.
    Kind,
    /// An id field does not parse as its type.
    MalformedId,
    /// `parent_id` and `order` are both absent exactly when `kind` is `campaign`.
    RootShape,
    /// `name` does not match the `Name` pattern.
    NameFormat,
    /// A key the decoder keeps without interpreting (a warning).
    UnknownKey,
    /// A write builds on a node that does not trace back to the root.
    UnvalidatedNode,
    /// `nodes/<id>.json` holds another `id`.
    FileIdMismatch,
    /// A criterion or step id repeats within its node file.
    SubItemIdUnique,
    /// Two node records hold one `NodeId`.
    DuplicateId,
    /// No parentless node carries the expected slug, or more than one does.
    RootCount,
    /// A parentless node carries another name than the slug.
    RootNameMismatch,
    /// A `parent_id` names no node.
    ParentMissing,
    /// A `parent_id` names a leaf.
    ParentNotDirectory,
    /// Following `parent_id` never reaches a parentless node.
    Cycle,
    /// Two children of one parent share a `name`.
    NameCollision,
    /// A write sets a campaign-scope link to a node that does not exist.
    DanglingId,
    /// A `Reference` holds both `node_id` and `target`.
    ReferenceTargetDerived,
    /// A reference whose `node_id` names no node (a warning).
    DanglingReference,
    /// A leaf with no reference (a warning).
    ReferencesEmpty,
    /// A `goal` has a line break.
    GoalLine,
    /// `preconditions` or `success_gates` is empty.
    CriteriaRequired,
    /// A leaf has no step or more than five.
    StepsRange,
    /// The steps disagree with the status (`planned` with a step done, `done` with a step not done).
    StatusSteps,
    /// `status_reason` is present other than exactly when `status` is `paused` or `cancelled`, or differs from the last history reason.
    StatusReason,
    /// `commit_sha` or `done_at` on a step that is not done.
    StepState,
    /// A mark names a criterion neither in the file nor in a removing amendment, or `met` differs from the last mark.
    MarkTarget,
    /// A record with `correction: true` has a blank reason.
    CorrectionReason,
    /// A revision a request must present is absent.
    ExpectedRevRequired,
    /// A content or structure edit carries no note.
    NoteRequired,
    /// A comment carries a note.
    NoteForbidden,
    /// A note with no change to record.
    EmptyEdit,
    /// A replace changes `id`.
    IdChanged,
    /// A replace changes `name`, `parent_id` or `order`.
    UseRelocate,
    /// A replace makes a node with children a leaf.
    LeafHasChildren,
    /// A relocate changes none of `name`, `parent_id`, `position`.
    NothingToDo,
    /// A `Position` names a node that is not a sibling.
    PositionNotSibling,
    /// The campaign root is deleted or given a parent or an order.
    RootDeleteOrMove,
    /// An `op_id` that the records know names a request they cannot rebuild the result of.
    OpIdReused,
}

impl Rule {
    /// The stable snake-case code of the rule.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::SchemaVersion => "schema_version",
            Self::UnknownFormat => "unknown_format",
            Self::NotInVersion => "not_in_version",
            Self::Kind => "kind",
            Self::MalformedId => "malformed_id",
            Self::RootShape => "root_shape",
            Self::NameFormat => "name_format",
            Self::UnknownKey => "unknown_key",
            Self::UnvalidatedNode => "unvalidated_node",
            Self::FileIdMismatch => "file_id_mismatch",
            Self::SubItemIdUnique => "sub_item_id_unique",
            Self::DuplicateId => "duplicate_id",
            Self::RootCount => "root_count",
            Self::RootNameMismatch => "root_name_mismatch",
            Self::ParentMissing => "parent_missing",
            Self::ParentNotDirectory => "parent_not_directory",
            Self::Cycle => "cycle",
            Self::NameCollision => "name_collision",
            Self::DanglingId => "dangling_id",
            Self::ReferenceTargetDerived => "reference_target_derived",
            Self::DanglingReference => "dangling_reference",
            Self::ReferencesEmpty => "references_empty",
            Self::GoalLine => "goal_line",
            Self::CriteriaRequired => "criteria_required",
            Self::StepsRange => "steps_range",
            Self::StatusSteps => "status_steps",
            Self::StatusReason => "status_reason",
            Self::StepState => "step_state",
            Self::MarkTarget => "mark_target",
            Self::CorrectionReason => "correction_reason",
            Self::ExpectedRevRequired => "expected_rev_required",
            Self::NoteRequired => "note_required",
            Self::NoteForbidden => "note_forbidden",
            Self::EmptyEdit => "empty_edit",
            Self::IdChanged => "id_changed",
            Self::UseRelocate => "use_relocate",
            Self::LeafHasChildren => "leaf_has_children",
            Self::NothingToDo => "nothing_to_do",
            Self::PositionNotSibling => "position_not_sibling",
            Self::RootDeleteOrMove => "root_delete_or_move",
            Self::OpIdReused => "op_id_reused",
        }
    }
}

/// A behaviour no layer of the format chain defines (`contracts.md`, `Version chain`). The caller
/// turns it into `Invalid` with the rule `not_in_version`, naming the file and its version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotInVersion {
    /// The behaviour asked for, by the name of the layer method.
    pub behaviour: &'static str,
}

/// The answer of a layer: the behaviour, or `NotInVersion`.
pub type Behaviour<T> = Result<T, NotInVersion>;

/// What the presented revision should have been, in a [`StoreError::Stale`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Current {
    /// The current `rev` of a node. A node that no longer exists is `NotFound`, never `Stale`.
    Node { id: NodeId, rev: Rev },
    /// The current `SubtreeRev` of a node.
    Subtree { id: NodeId, rev: SubtreeRev },
    /// The current `TreeRev` of the campaign.
    Tree { tree_rev: TreeRev },
}

/// The advisory record of the writer lock (`store.md`, §Lock). Nothing decides by it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LockHolder {
    pub pid: u32,
    pub host: String,
    /// The UTC instant of the acquisition.
    pub since: Ts,
}

/// The one error type of every store operation and load. M4 maps its variants to tool errors and
/// the CLI to exit codes; nothing maps a message string.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoreError {
    /// A presented `expected_rev`, `expected_subtree_rev` or `expected_tree_rev` is not the
    /// current one (I4). Nothing was written.
    Stale { current: Current },
    /// The writer lock was not taken within the wait. `holder` is the advisory record, `None` if
    /// it is absent or unreadable. Nothing was written.
    Locked { holder: Option<LockHolder> },
    /// A file's `schema_version` is newer than this binary reads. Raised before anything else of
    /// the file is parsed.
    NewerFormat { found: Version, supported: Version, file: String },
    /// A file or a request breaks a rule. `file` is the path relative to the campaign root
    /// (`nodes/<id>.json`), or `<request>` when the fault is in the request.
    Invalid { file: String, rule: Rule, detail: String },
    /// A node a request names does not exist.
    NotFound { target: RecordRef },
    /// The operating system refused a read or a write.
    Io { file: String, message: String },
    /// One op of a batch failed. `source` is any of the above; nothing was written.
    InBatch { index: usize, source: Box<StoreError> },
}

impl StoreError {
    /// The error itself, with `InBatch` unwrapped.
    #[must_use]
    pub fn root(&self) -> &StoreError {
        let mut error = self;
        while let Self::InBatch { source, .. } = error {
            error = source;
        }
        error
    }

    /// The stable code of the root: `STALE`, `LOCKED`, `NEWER_FORMAT`, `INVALID`, `NOT_FOUND` or
    /// `IO`.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Stale { .. } => "STALE",
            Self::Locked { .. } => "LOCKED",
            Self::NewerFormat { .. } => "NEWER_FORMAT",
            Self::Invalid { .. } => "INVALID",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Io { .. } => "IO",
            Self::InBatch { source, .. } => source.code(),
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stale { .. } => f.write_str("stale revision: re-read and reapply the intent"),
            Self::Locked { holder: Some(h) } => write!(f, "campaign locked by pid {} on {} since {}", h.pid, h.host, h.since),
            Self::Locked { holder: None } => f.write_str("campaign locked by an unknown writer"),
            Self::NewerFormat { found, supported, file } => write!(
                f,
                "{file}: format {} is newer than the supported {}",
                version_string(*found),
                version_string(*supported)
            ),
            Self::Invalid { file, rule, detail } => write!(f, "{file}: {}: {detail}", rule.code()),
            Self::NotFound { target } => write!(f, "not found: {target:?}"),
            Self::Io { file, message } => write!(f, "{file}: {message}"),
            Self::InBatch { index, source } => write!(f, "op {index} of the batch failed: {source}"),
        }
    }
}

impl std::error::Error for StoreError {}
