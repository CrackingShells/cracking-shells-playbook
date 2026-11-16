# M2 Graph engine

**Goal**: `rdm-core` validates a whole campaign, computes its order, enforces the status machine and applies every create, read, update, delete and restructuring operation, singly or as an atomic batch, with amendments recorded.
**Pre-conditions**:
- [ ] `../m1_core_foundation.md` is `done`
- [ ] Reminder `skills-rdm-M2` read
**Success Gates**:
- ⬜ Whole-campaign validation returns diagnostics with stable codes, and a golden test pins the wave semantics (the ordering rule M0b fixes, checked against `skills/managing-roadmaps/references/dirtree-execution.md`)
- ⬜ The read-model projections of `rdm-schema` are computed by one set of functions in `rdm-core`, and a golden test pins each against the worked examples of M0b
- ⬜ Property tests: random trees under random operation sequences stay valid, keep ids unique and conserve leaves except on delete
- ⬜ Every content or structure edit writes exactly one amendment with a non-empty description; status and progress write none (I5)
- ⬜ A batch is atomic: a failing operation leaves the campaign untouched, and a dry-run writes nothing
- ⬜ `done` is refused while a step lacks its commit (I8); deleting or reopening done work and moving work in progress are refused without an acknowledgement (I9)
**References**: [R01 §Graph operations, §Contracts & Invariants](../../../../__design__/rdm/architecture.md) — validation, order, the status machine, the operations, impact analysis and batch

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m2-graph-engine/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding: the diagnostic codes; the ready-set and next-leaf rules; the status transitions, taking the meanings and the ordering rule fixed by M0b as given, and how a directory's status is derived; the primitive and composite operations with their pre- and postconditions; how 'run X after Y' is expressed as depth edits and how candidate plans are ranked; what impact analysis reports and which effects need an acknowledgement; batch semantics (aliases for nodes created earlier, limits, dry-run); how an amendment records a change and a removed subtree; whether a cross-worktree claim hint is worth having.
Adversarial verification is required for the structural operations and impact analysis (a wrong edit reorders work silently) and for the status machine.
**Deliverables**: `__roadmap__/m2-graph-engine/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m2-graph-engine/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m2-graph-engine milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m2-graph-engine`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: `crates/rdm-core` modules for validation, order, the status machine, operations, impact analysis and batch; the property tests
**Consistency Checks**: `cargo test -p rdm-core` (expected: PASS)
**Commit**: `feat(rdm-core): land the graph engine`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m2-graph-engine/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M2` complete; `skills-rdm-M3` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m2-graph-engine/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/core/engine/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/core/engine/` (expected: PASS)
**Commit**: `docs(reports): close the m2-graph-engine milestone`
