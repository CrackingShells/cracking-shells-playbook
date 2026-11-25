//! The move table of `statuses.md` as data (`contracts.md`, `Moves`).
//!
//! The document is the oracle: the test `move_table_matches_statuses_doc` parses its first table
//! after the heading `## Transitions` and compares it with [`MOVES`], so a change to the table
//! fails the build until this file follows.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::stored::Status;

/// Who may make a move: the actor kind of `history[].actor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Who {
    Agent,
    Human,
    Both,
}

/// Whether the history entry of a move needs a reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReasonRule {
    /// A non-blank reason is required.
    Required,
    /// A reason is required only for an override of the ready set.
    OnlyForOverride,
    /// No reason is needed.
    NotRequired,
}

/// What a move does to the ready set (`statuses.md`, `Ready set`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReadyEffect {
    /// The leaf may enter the ready set.
    Enter,
    /// The leaf leaves the ready set.
    Leave,
    /// Deeper leaves may become ready.
    Release,
    /// A finished leaf is unfinished again: deeper ready leaves stop being ready.
    Hold,
}

/// One row of the move table: an ordered pair of statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Status,
    pub to: Status,
    pub allowed: bool,
    /// `None` exactly when the move is refused.
    pub who: Option<Who>,
    pub reason: ReasonRule,
    /// Verbatim from the table, backticks kept. On a refused row it is the refusal message; M2
    /// owns the logic that checks an allowed row's precondition, keyed on `(from, to)`.
    pub precondition: &'static str,
    pub ready_set: &'static [ReadyEffect],
}

/// All 25 ordered pairs of the five statuses, row-major: the index is `from * 5 + to`.
pub static MOVES: [Move; 25] = [
    Move {
        from: Status::Planned,
        to: Status::Planned,
        allowed: false,
        who: None,
        reason: ReasonRule::NotRequired,
        precondition: "A move to the same status is a no-op and writes no history",
        ready_set: &[],
    },
    Move {
        from: Status::Planned,
        to: Status::Ongoing,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::OnlyForOverride,
        precondition: "Every pre-condition is `met`, and the leaf is in the ready set. Outside the set, an override with a reason is needed, and the entry records `override: true`. The override skips the ready set, never the pre-conditions (Eliott, 2026-11-16)",
        ready_set: &[ReadyEffect::Leave],
    },
    Move {
        from: Status::Planned,
        to: Status::Paused,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "none",
        ready_set: &[ReadyEffect::Leave],
    },
    Move {
        from: Status::Planned,
        to: Status::Cancelled,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "none",
        ready_set: &[ReadyEffect::Leave, ReadyEffect::Release],
    },
    Move {
        from: Status::Planned,
        to: Status::Done,
        allowed: false,
        who: None,
        reason: ReasonRule::NotRequired,
        precondition: "Start the leaf first: `done` needs the steps recorded (I8) and the gates `met`",
        ready_set: &[],
    },
    Move {
        from: Status::Ongoing,
        to: Status::Planned,
        allowed: false,
        who: None,
        reason: ReasonRule::NotRequired,
        precondition: "Use `paused`, then `planned` while no step is done",
        ready_set: &[],
    },
    Move {
        from: Status::Ongoing,
        to: Status::Ongoing,
        allowed: false,
        who: None,
        reason: ReasonRule::NotRequired,
        precondition: "No-op",
        ready_set: &[],
    },
    Move {
        from: Status::Ongoing,
        to: Status::Paused,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "none",
        ready_set: &[],
    },
    Move {
        from: Status::Ongoing,
        to: Status::Cancelled,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "none. Steps already done stay recorded",
        ready_set: &[ReadyEffect::Release],
    },
    Move {
        from: Status::Ongoing,
        to: Status::Done,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::NotRequired,
        precondition: "Every step is done with its commit recorded (I8), and every success gate is `met` (Eliott, 2026-11-16). See `Criteria`",
        ready_set: &[ReadyEffect::Release],
    },
    Move {
        from: Status::Paused,
        to: Status::Planned,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "No step is recorded done (coordinator, 2026-11-16)",
        ready_set: &[ReadyEffect::Enter],
    },
    Move {
        from: Status::Paused,
        to: Status::Ongoing,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "The leaf was `ongoing` before the pause (last pause entry has `from: ongoing`), or it is in the ready set. Otherwise an override with a reason",
        ready_set: &[],
    },
    Move {
        from: Status::Paused,
        to: Status::Paused,
        allowed: false,
        who: None,
        reason: ReasonRule::NotRequired,
        precondition: "No-op",
        ready_set: &[],
    },
    Move {
        from: Status::Paused,
        to: Status::Cancelled,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "none",
        ready_set: &[ReadyEffect::Release],
    },
    Move {
        from: Status::Paused,
        to: Status::Done,
        allowed: false,
        who: None,
        reason: ReasonRule::NotRequired,
        precondition: "Resume to `ongoing` first",
        ready_set: &[],
    },
    Move {
        from: Status::Cancelled,
        to: Status::Planned,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "Undo: the status before the cancel, read from history (the `from` of the last cancel entry), is `planned`. Eliott's intent 2026-11-16, ruling (coordinator, 2026-11-16)",
        ready_set: &[ReadyEffect::Enter],
    },
    Move {
        from: Status::Cancelled,
        to: Status::Ongoing,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "Undo: the status before the cancel was `ongoing`. Any other target is refused (coordinator, 2026-11-16)",
        ready_set: &[],
    },
    Move {
        from: Status::Cancelled,
        to: Status::Paused,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "Undo: the status before the cancel was `paused`. Any other target is refused (coordinator, 2026-11-16)",
        ready_set: &[],
    },
    Move {
        from: Status::Cancelled,
        to: Status::Cancelled,
        allowed: false,
        who: None,
        reason: ReasonRule::NotRequired,
        precondition: "No-op",
        ready_set: &[],
    },
    Move {
        from: Status::Cancelled,
        to: Status::Done,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "Undo only: the status before the cancel was `done` (a leaf cancelled by a reversal out of `done`). Flagged `correction: true`. Any other route into `done` from `cancelled` is refused (coordinator, 2026-11-16)",
        ready_set: &[ReadyEffect::Release],
    },
    Move {
        from: Status::Done,
        to: Status::Planned,
        allowed: false,
        who: None,
        reason: ReasonRule::NotRequired,
        precondition: "The leaf has steps recorded done, and `planned` means not started. Reverse to `ongoing`",
        ready_set: &[],
    },
    Move {
        from: Status::Done,
        to: Status::Ongoing,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "A reversal (Eliott, 2026-11-16). Recorded with `correction: true`. No gate or step is changed: they stay as recorded",
        ready_set: &[ReadyEffect::Hold],
    },
    Move {
        from: Status::Done,
        to: Status::Paused,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "A reversal, `correction: true`",
        ready_set: &[ReadyEffect::Hold],
    },
    Move {
        from: Status::Done,
        to: Status::Cancelled,
        allowed: true,
        who: Some(Who::Both),
        reason: ReasonRule::Required,
        precondition: "A reversal, `correction: true`. Steps and gates stay as recorded",
        ready_set: &[],
    },
    Move {
        from: Status::Done,
        to: Status::Done,
        allowed: false,
        who: None,
        reason: ReasonRule::NotRequired,
        precondition: "No-op",
        ready_set: &[],
    },
];

/// The row of a move. Total: every pair of statuses has one.
#[must_use]
pub fn lookup(from: Status, to: Status) -> &'static Move {
    // CAST: `Status` is a fieldless enum of five variants, so the discriminants are 0 to 4.
    &MOVES[from as usize * 5 + to as usize]
}
