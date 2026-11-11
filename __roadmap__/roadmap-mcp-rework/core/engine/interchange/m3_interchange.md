# M3 Interchange

**Goal**: `rdm-core` exports any node to markdown for a human reader and imports a legacy markdown roadmap into the JSON store.
**Pre-conditions**:
- [ ] `../m2_graph_engine.md` is `done`
- [ ] Reminder `skills-rdm-M3` read
**Success Gates**:
- ⬜ Any leaf, directory or campaign exports to markdown, with the node table and the status graph derived at export time (I6)
- ⬜ The seven valid campaigns under `__roadmap__/` import, pass validation and export with no loss of content, compared as parsed models
- ⬜ `__roadmap__/create-reporting-skill/`, invalid by design, imports to a fixed list of diagnostics and writes nothing
- ⬜ An import into a non-empty target is refused, and a dry-run writes nothing
**References**: [R01 §Data Model, §Key Flows, §Deliberately open](../../../../../__design__/rdm/architecture.md) — what is stored and what is derived, and the migrate flow

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m3-interchange/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding: the markdown layout of an exported leaf, directory and campaign (close to today's format, since humans know it); how the existing Amendment Log and Progress rows map to amendments, history and progress; how ids are minted on import; what the importer does with input the old grammar rejects; how the eight campaigns become fixtures without copying their history; the import report.
Adversarial verification is required for the importer (a silent loss of content during migration is found months later).
**Deliverables**: `__roadmap__/m3-interchange/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m3-interchange/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m3-interchange milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m3-interchange`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: `crates/rdm-core` modules for export and import; fixtures of the existing campaigns; golden tests
**Consistency Checks**: `cargo test -p rdm-core` (expected: PASS)
**Commit**: `feat(rdm-core): land export and import`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m3-interchange/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M3` complete; `skills-rdm-M4`, `skills-rdm-M5` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m3-interchange/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/core/engine/interchange/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/core/engine/interchange/` (expected: PASS)
**Commit**: `docs(reports): close the m3-interchange milestone`
