//! The rules a node file breaks by itself (`contracts.md`, `Node fields` and `Campaign model`;
//! `stored-schema.md`, `Directory`, `Leaf`, `Step`, `Mark`, `History`, `Reference`).
//!
//! Serde and the codec enforce the shapes of the types and the formats of ids and names. What is
//! left is checked here, once, on a decoded node: `Campaign::open` refuses a file that breaks one
//! and the store refuses a request whose after-image breaks one.

use std::collections::BTreeSet;

use crate::error::Rule;
use crate::stored::{Criterion, DirectoryKind, Leaf, Node, Reference, Status};

/// A rule a node breaks, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    pub rule: Rule,
    pub detail: String,
}

fn fault(rule: Rule, detail: impl Into<String>) -> Result<(), Fault> {
    Err(Fault {
        rule,
        detail: detail.into(),
    })
}

fn blank(s: &str) -> bool {
    s.trim().is_empty()
}

fn goal_line(goal: &str) -> Result<(), Fault> {
    if goal.contains(['\n', '\r']) {
        return fault(Rule::GoalLine, "`goal` has a line break");
    }
    Ok(())
}

fn references(owner: &str, refs: &[Reference]) -> Result<(), Fault> {
    for (i, r) in refs.iter().enumerate() {
        if r.node_id.is_some() && r.target.is_some() {
            return fault(
                Rule::ReferenceTargetDerived,
                format!("{owner}/references/{i} holds both `node_id` and `target`"),
            );
        }
    }
    Ok(())
}

fn criteria(owner: &str, list: &[Criterion], required: bool) -> Result<(), Fault> {
    if required && list.is_empty() {
        return fault(Rule::CriteriaRequired, format!("`{owner}` is empty"));
    }
    Ok(())
}

fn mark_target(
    marks: &[crate::stored::Mark],
    preconditions: &[Criterion],
    success_gates: &[Criterion],
) -> Result<(), Fault> {
    for c in preconditions.iter().chain(success_gates) {
        if let Some(last) = marks.iter().rev().find(|m| m.criterion_id == c.id)
            && last.met != c.met
        {
            return fault(
                Rule::MarkTarget,
                format!(
                    "criterion {} is met={} but its last mark says {}",
                    c.id, c.met, last.met
                ),
            );
        }
    }
    Ok(())
}

fn correction_reasons(
    marks: &[crate::stored::Mark],
    history: &[crate::stored::History],
) -> Result<(), Fault> {
    for (i, m) in marks.iter().enumerate() {
        if m.correction == Some(true) && m.reason.as_deref().is_none_or(blank) {
            return fault(
                Rule::CorrectionReason,
                format!("/marks/{i} is a correction with no reason"),
            );
        }
    }
    for (i, h) in history.iter().enumerate() {
        if h.correction == Some(true)
            && (h.reason.as_deref().is_none_or(blank) || h.from != Status::Done)
        {
            return fault(
                Rule::CorrectionReason,
                format!("/history/{i} is a correction that does not leave `done` with a reason"),
            );
        }
    }
    Ok(())
}

fn unique_ids(ids: impl Iterator<Item = String>) -> Result<(), Fault> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id.clone()) {
            return fault(
                Rule::SubItemIdUnique,
                format!("the id {id} repeats in the file"),
            );
        }
    }
    Ok(())
}

fn leaf(l: &Leaf) -> Result<(), Fault> {
    goal_line(&l.goal)?;
    criteria("preconditions", &l.preconditions, true)?;
    criteria("success_gates", &l.success_gates, true)?;
    unique_ids(
        l.preconditions
            .iter()
            .chain(&l.success_gates)
            .map(|c| c.id.to_string())
            .chain(l.steps.iter().map(|s| s.id.to_string())),
    )?;
    if !(1..=5).contains(&l.steps.len()) {
        return fault(
            Rule::StepsRange,
            format!("a leaf has 1 to 5 steps, this has {}", l.steps.len()),
        );
    }
    match l.status {
        Status::Planned if l.steps.iter().any(|s| s.done) => {
            return fault(Rule::StatusSteps, "`planned` with a step done");
        }
        Status::Done if l.steps.iter().any(|s| !s.done) => {
            return fault(Rule::StatusSteps, "`done` with a step not done");
        }
        _ => {}
    }
    let needs_reason = matches!(l.status, Status::Paused | Status::Cancelled);
    if l.status_reason.is_some() != needs_reason {
        return fault(
            Rule::StatusReason,
            "`status_reason` is present exactly when the status is `paused` or `cancelled`",
        );
    }
    if let (Some(reason), Some(last)) = (&l.status_reason, l.history.last())
        && last.reason.as_ref() != Some(reason)
    {
        return fault(
            Rule::StatusReason,
            "`status_reason` differs from the reason of the last history entry",
        );
    }
    for (i, s) in l.steps.iter().enumerate() {
        if !s.done && (s.commit_sha.is_some() || s.done_at.is_some()) {
            return fault(
                Rule::StepState,
                format!("/steps/{i} has `commit_sha` or `done_at` and is not done"),
            );
        }
        references(&format!("/steps/{i}"), &s.references)?;
    }
    references("", &l.references)?;
    mark_target(&l.marks, &l.preconditions, &l.success_gates)?;
    correction_reasons(&l.marks, &l.history)
}

/// Checks the rules a node breaks by itself.
///
/// # Errors
/// The first [`Fault`]: `root_shape`, `goal_line`, `criteria_required`, `steps_range`,
/// `status_steps`, `status_reason`, `step_state`, `mark_target` (a criterion's `met` against its
/// last mark), `correction_reason`, `reference_target_derived` or `sub_item_id_unique`.
pub fn check_node(node: &Node) -> Result<(), Fault> {
    match node {
        Node::Leaf(l) => leaf(l),
        Node::Directory(d) => {
            let rooted = d.parent_id.is_none() && d.order.is_none();
            let placed = d.parent_id.is_some() && d.order.is_some();
            let shape_ok = match d.kind {
                DirectoryKind::Campaign => rooted,
                DirectoryKind::Directory => placed,
            };
            if !shape_ok {
                return fault(
                    Rule::RootShape,
                    "`parent_id` and `order` are both absent exactly when `kind` is `campaign`, and both present otherwise",
                );
            }
            goal_line(&d.goal)?;
            criteria("preconditions", &d.preconditions, true)?;
            criteria("success_gates", &d.success_gates, true)?;
            unique_ids(
                d.preconditions
                    .iter()
                    .chain(&d.success_gates)
                    .map(|c| c.id.to_string()),
            )?;
            references("", &d.references)?;
            mark_target(&d.marks, &d.preconditions, &d.success_gates)?;
            correction_reasons(&d.marks, &[])
        }
    }
}
