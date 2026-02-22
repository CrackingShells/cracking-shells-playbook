# Roadmap Execution Guide

## Contents

- [CRUD: Update](#crud-update)
- [CRUD: Delete](#crud-delete)
- [Breadth-First Traversal Algorithm](#breadth-first-traversal-algorithm)
- [Execution Rules](#execution-rules)
- [Git Workflow](#git-workflow)
- [Step Execution](#step-execution)
- [Progress Tracking](#progress-tracking)
- [Failure Handling](#failure-handling)
- [Completion Checklist](#completion-checklist)
- [Key References](#key-references)

---

## CRUD: Update

**Status updates** (during execution):
- Starting task: Set node as `:::inprogress`
- Completing task: Set node as `:::done`, update Progress table
- Starting directory: Set node as `:::inprogress`
- Completing directory: Set node as `:::done`
- Discovering blocker: Set node as `:::blocked` with note

**Amendment workflow** (planned changes):
1. Produce gap analysis report in `__reports__/`
2. Submit for review (any entity generically: human, LLM, or automated)
3. On approval: Create new task files at appropriate depth
4. Update README.md in affected level (add nodes + amendment log entry)
5. Use `:::amendment` styling until executed, then `:::done`
6. Never rename or renumber existing nodes

---

## CRUD: Delete

- Never delete or rename roadmap nodes
- Mark abandoned work as `:::blocked`
- Embed reason in node notes

---

## Breadth-First Traversal Algorithm

```
ENTER directory
  READ README.md → understand goal, context, status
  IDENTIFY leaf files and subdirectories
  EXECUTE all leaf files (parallel — dispatch to subagents)
  WAIT for all leaves to complete
  EXECUTE all subdirectories (parallel — dispatch to subagents)
  WAIT for all subdirectories to complete
  MARK this directory as done in parent README.md
EXIT directory
```

---

## Execution Rules

1. **Leaves before subdirectories**: Always execute files first
2. **Siblings are parallel**: Never execute sequentially what tree says is parallel
3. **All depth N complete before depth N+1**: Wait for sibling leaves to complete
4. **Dispatch workers**: Use subagents for parallel execution, document if not possible

---

## Git Workflow

```bash
Before task: git checkout -b task/<name> milestone/<campaign>
Per step:    Implement → Run checks → Commit
After task:  Verify success gates → Merge task into milestone → Update README.md
```

---

## Step Execution

For each step:
1. **Read step**: Goal, Implementation Logic, References
2. **Produce**: Generate deliverables
3. **Check consistency**: Run required commands
4. **Commit immediately**: Use exact commit message from task file
5. **Update progress**: Record in parent README.md Progress table
6. **If checks fail**: Diagnostic ladder (see Failure Handling)

---

## Progress Tracking

### When to Update

| Event | Action |
|:------|:-------|
| Starting a leaf task | Mark `:::inprogress` in parent README |
| Completing a leaf task | Mark `:::done`, update Progress table |
| Starting a subdirectory | Mark `:::inprogress` in parent README |
| Completing a subdirectory | Mark `:::done` in parent README |
| Discovering blocker | Mark `:::blocked` with note |

### What to Update

1. **Mermaid status graph**: Change `classDef` of the node
2. **Nodes table**: Update Status column
3. **Progress table**: Record branch name, commit count, notes

---

## Failure Handling

When a consistency check fails unexpectedly, follow this escalation ladder in order. Do NOT immediately raise an amendment.

**Level 1: Check Yourself**
- Re-read Implementation Logic
- Re-read References
- Check for typos, wrong paths, syntax errors
- Run check again to rule out flakiness

**Level 2: Check Downstream**
- Read ahead in task for later steps
- Check sibling leaves or subdirectories
- Check parent README context

**Level 3: Check Upstream**
- Check git history for previous bugs
- Verify pre-conditions are met
- Is prior work in unexpected state?

**Level 4: Prove the Problem**
- Write minimal reproduction
- Document exact failure
- Identify root cause

**Level 5: Raise an Amendment**
- Produce gap analysis in `__reports__/`
- Submit for review
- On approval: Create new task files, update README
- See [amendments.md](amendments.md) for full amendment cycle

### What is NOT an Amendment
- UX/API usability issues (log for future campaign)
- "Nice to have" improvements out of scope
- Speculative problems without evidence

---

## Completion Checklist

When all nodes at all levels are `done`:

- [ ] All leaf task success gates met
- [ ] All README.md status visualizations show all-green
- [ ] All Progress tables have branch names and commit counts
- [ ] All task branches merged into milestone
- [ ] Full test suite passes on the milestone branch
- [ ] Final agent review (human or automated)
- [ ] Milestone branch merged into `dev`

---

## Key References

- [amendments.md](amendments.md) — Full amendment cycle, gap analysis format, common patterns, and escalation vs. amendment decision guide
- [dirtree-schema-validation.md](dirtree-schema-validation.md) — Validation rules for steps, commits, and progress tracking compliance
