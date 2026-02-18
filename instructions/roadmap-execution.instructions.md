---
applyTo: '**/*'
description: 'Operational manual for executing against a roadmap directory tree'
---

# Roadmap Execution Manual

This document tells an executing agent (human, LLM, or program) **how to navigate and execute** a roadmap directory tree. For how to create and maintain roadmaps, see [roadmap-generation.instructions.md](./roadmap-generation.instructions.md).

---

## 1. The Breadth-First Traversal Algorithm

Execution follows a recursive breadth-first traversal of the directory tree. At every directory level:

```
ENTER directory
  READ README.md → understand goal, context, status
  IDENTIFY leaf files and subdirectories
  EXECUTE all leaf files (parallel — dispatch to subagents when possible)
  WAIT for all leaves to complete
  EXECUTE all subdirectories (parallel — dispatch to subagents when possible)
  WAIT for all subdirectories to complete
  MARK this directory as done in parent README.md
EXIT directory
```

### Rules

1. **Leaves before subdirectories.** Always. No exceptions.
2. **Siblings are parallel.** If there are multiple leaves, dispatch them to parallel agents (subagents). Do not execute sequentially what the tree says is parallel.
3. **All of depth N completes before depth N+1.** Do not enter a subdirectory until every sibling leaf at the same level is done.
4. **Recurse.** Each subdirectory is a self-contained unit — apply the same algorithm from step 1.

### Parallelization Discipline

When multiple sibling leaves exist at the same level:
- **Dispatch each to a subagent** with the leaf task file content and any referenced reports as context
- The dispatching agent waits for all subagents to complete before proceeding to subdirectories
- If subagents are not available, execute leaves sequentially — but document that parallelism was not exploited

When multiple sibling subdirectories exist:
- Same rule — dispatch each to a subagent if possible
  - WARNING: 1 subagent per subdirectory can overflow memory. Escalating back to orchestrating agent might be more advisable unless depth of the implementation tree is small.
- Each subagent recursively applies this algorithm within its subdirectory

---

## 2. Executing a Leaf Task

A leaf task file contains sequential steps. For each step:

```
READ step (Goal, Implementation Logic, References)
PRODUCE deliverables
RUN consistency checks
  IF checks pass (or match expected outcome):
    COMMIT immediately with the prescribed message
    UPDATE progress in parent README.md (optional per-step, required per-task)
  IF checks fail unexpectedly:
    DIAGNOSE per §4 (Failure Handling)
```

### The Commit Bijection

**1 step = 1 commit. No exceptions.**

- Commit immediately after each step's consistency checks pass (or match the expected outcome, e.g., FAIL for a TDD test step)
- Do not batch multiple steps into one commit
- Do not skip the commit for "trivial" steps
- The commit message is prescribed in the task file — use it exactly

This is the single most important operational discipline. If you find yourself wanting to batch commits, you are violating the bijection.

### Git Workflow Per Leaf Task

1. **Before first step**: create `task/<name>` branch from `milestone/<campaign>`
2. **Per step**: produce deliverables → run checks → commit
3. **After last step**: verify task-level success gates → merge task branch into milestone
4. **Update parent README.md**: mark node as `done`, record branch name and commit count in Progress table

---

## 3. Status Update Discipline

Keep `README.md` status current as you work. This is how other agents (and humans) track progress.

### When to Update

| Event | Action |
|:------|:-------|
| Starting a leaf task | Mark node `:::inprogress` in parent README.md |
| Completing a leaf task | Mark node `:::done`, update Progress table |
| Starting a subdirectory | Mark node `:::inprogress` in parent README.md |
| Completing a subdirectory (all internal nodes done) | Mark node `:::done` in parent README.md |
| Discovering a blocker | Mark node `:::blocked` with a note |

### What to Update

In the parent `README.md`:
1. **Mermaid status graph**: change the `classDef` of the node
2. **Nodes table**: update the Status column
3. **Progress table**: record branch name, commit count, notes

---

## 4. Failure Handling — The Escalation Ladder

When a consistency check fails unexpectedly, do NOT immediately raise an amendment. Follow this escalation ladder in order:

### Level 1: Check Yourself

Before assuming the roadmap is wrong, verify your own work:
- Re-read the step's Implementation Logic — did you miss something?
- Re-read the References — did you misinterpret the architecture report?
- Check for typos, wrong file paths, missing imports, syntax errors
- Run the check again to rule out flaky failures

**Gate**: Can you fix the failure by correcting your own implementation? If yes, fix and proceed. No amendment needed.

### Level 2: Check Downstream

Before assuming the current step is wrong, check if a later step in the same task or a deeper node in the tree is designed to address the issue:
- Read ahead in the current leaf task — does a subsequent step fix this?
- Check sibling leaves or subdirectories — does another node handle this concern?
- Check the parent README.md context — is this a known intermediate state?

**Gate**: Is the failure expected to be resolved by downstream work? If yes, proceed. No amendment needed.

### Level 3: Check Upstream

Before assuming the roadmap needs amendment, verify the roadmap's consistency:
- Is there a bug in a previously completed step? (Check git history)
- Did a prior task leave something in an unexpected state?
- Is the pre-condition for this step actually met?

**Gate**: Is the failure caused by an upstream bug? If yes, fix the upstream issue (new commit on the relevant task branch or a patch step). This is a bug fix, not an amendment.

### Level 4: Prove the Problem

If levels 1–3 don't resolve the failure, you may have found a genuine gap. But before raising an amendment, you must **prove it with evidence**:
- Write a minimal reproduction (debug statements, isolated test case)
- Document the exact failure: what was expected, what happened, why levels 1–3 don't apply
- Identify the root cause with specificity — "something is wrong" is not sufficient

**Gate**: Can you articulate the gap with measurable evidence? If not, keep investigating. Do not raise speculative amendments.

### Level 5: Raise an Amendment

Only after levels 1–4 are exhausted:
1. Produce a gap analysis report in `__reports__/` with the evidence from level 4
2. Submit for agent review (human, LLM, or automated reviewer)
3. On approval: create new task files at the appropriate depth, update README.md
4. Follow the amendment workflow in [roadmap-generation.instructions.md](./roadmap-generation.instructions.md)

### What is NOT an Amendment

- **UX or API usability issues** discovered during implementation are not roadmap gaps — they are feedback for a future campaign. Log them in a report, do not amend the current roadmap unless the issue blocks a success gate.
- **"Nice to have" improvements** that go beyond the current task's scope. The roadmap bounds the work — if it's not in the Implementation Logic, it's out of scope.
- **Speculative problems** that "might" cause issues downstream. If you can't prove it with a failing test or a concrete scenario, it's not a gap yet.

---

## 5. Subagent Dispatch Protocol

When dispatching work to subagents (parallel leaf execution or parallel subdirectory execution):

### What to Provide

1. **The leaf task file** (or subdirectory path for recursive execution)
2. **Referenced reports** — the architecture report sections cited in the task's References
3. **Parent README.md context** — so the subagent understands where this task fits
4. **Git instructions** — branch name to create, milestone branch to merge into

### What to Expect Back

1. Commits on the task branch (1 per step)
2. Confirmation that success gates are met
3. Any failure escalation per §4 if consistency checks failed unexpectedly

### Coordination

- The dispatching agent is responsible for waiting on all parallel subagents before proceeding to the next depth level
- The dispatching agent updates the parent README.md status after subagents complete
- If a subagent raises a level 5 escalation (amendment), the dispatching agent pauses execution at that level until the amendment is reviewed

---

## 6. Completion Checklist

When all nodes at all levels are `done`:

- [ ] All leaf task success gates met
- [ ] All README.md status visualizations show all-green
- [ ] All Progress tables have branch names and commit counts
- [ ] All task branches merged into milestone
- [ ] Full test suite passes on the milestone branch
- [ ] Final agent review (human or automated)
- [ ] Milestone branch merged into `dev`

---

## Cross-References

- [roadmap-generation.instructions.md](./roadmap-generation.instructions.md) — how to create and maintain roadmap trees
- [code-change-phases.instructions.md](./code-change-phases.instructions.md) — the 3-stage workflow (Analysis → Roadmap → Execution)
- [git-workflow-milestone.instructions.md](./git-workflow-milestone.instructions.md) — flat branching model
