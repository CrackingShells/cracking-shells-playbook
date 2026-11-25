//! Rule codes, the error type of every store operation and the lock record
//! (`contracts.md`, `Errors`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
        }
    }
}
