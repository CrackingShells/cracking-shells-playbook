# M8 Cutover

**Goal**: `managing-roadmaps` ships with the MCP server and no script: the skill teaches the tools through usage patterns, the old crate is gone and this repository's roadmaps are JSON.
**Pre-conditions**:
- [ ] `../m6_apps_view.md` and `../m7_distribution.md` are `done`
- [ ] Reminder `skills-rdm-M8` read
**Success Gates**:
- ⬜ `SKILL.md` is built around usage patterns, each a short tool sequence that produces a named result
- ⬜ `skills/managing-roadmaps/scripts/` and its copy under `plugins/managing-roadmaps/` no longer exist, and no file in the repository refers to `dirtree-rdm` except history
- ⬜ Every campaign under `__roadmap__/` is migrated to JSON, or its exclusion is recorded with the reason
- ⬜ `README.md`, `CONTRIBUTING.md`, `instructions/code-change-phases.instructions.md` and the `writing-reports` handoff text describe the new tooling
- ⬜ A fresh agent session, given only the installed plugin, authors and amends a small roadmap
**References**: [R01 §Executive Summary, §Key Flows, §Deliberately open](../../../../../../../../__design__/rdm/architecture.md) — what the skill must teach and the flows its usage patterns cover

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m8-cutover/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding: the usage patterns `SKILL.md` carries (author a campaign, pick and start the next ready leaf, record a step and finish a leaf, insert a milestone between two levels, split an oversized leaf, re-plan a subtree after new information, export a leaf for a human, migrate a legacy roadmap); which references survive, trimmed, and which go (grammar, CLI, schema validation, tier examples); the order of removal so that no commit leaves the plugin without a working tool; the migration of the existing campaigns, including this meta-roadmap and the invalid legacy one; the major version bump of the plugin; skills outside this repository that mention the old tooling (`umbod`), reported to Eliott and not edited.
Adversarial verification is required for the migration of the existing campaigns (they are kept as specification sources) and is done by a fresh-session trial for the skill text.
**Deliverables**: `__roadmap__/m8-cutover/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m8-cutover/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m8-cutover milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m8-cutover`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: the rewritten `skills/managing-roadmaps/SKILL.md` and references; the migrated `__roadmap__/`; updated `README.md`, `CONTRIBUTING.md`, `instructions/code-change-phases.instructions.md`, `skills/writing-reports/`
**Consistency Checks**: `uv run tools/quick_validate.py skills/managing-roadmaps` (expected: PASS)
**Commit**: `feat(managing-roadmaps)!: replace dirtree-rdm with the rdm MCP server`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m8-cutover/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M8` complete; the rework is complete. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m8-cutover/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/core/engine/interchange/consumers/delivery/cutover/README.md`
**Consistency Checks**: `rdm validate __roadmap__/roadmap-mcp-rework` (expected: PASS)
**Commit**: `docs(reports): close the m8-cutover milestone`
