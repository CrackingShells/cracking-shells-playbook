# M5 CLI

**Goal**: `rdm-cli` (binary `rdm`) gives a human the same operations as the MCP tools: peek at a roadmap, export markdown and make updates, with no roadmap logic of its own.
**Pre-conditions**:
- [ ] `../m3_interchange.md` is `done`
- [ ] Reminder `skills-rdm-M5` read
**Success Gates**:
- ⬜ Every core operation is reachable from a `rdm` verb
- ⬜ `rdm` and the MCP tools give the same result on the same input, checked by a shared test (I2); before M4 lands the test compares against the core directly
- ⬜ Every verb has a `--json` output, and a content or structure edit refuses to run without an amendment description (I5)
- ⬜ `rdm export` prints the markdown of any node, and `rdm` works from any directory given a campaign path
**References**: [R01 §Proposed State, §Key Flows, §Contracts & Invariants](../../../../../../__design__/rdm/architecture.md) — the consumer rule and the flows a human runs

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m5-cli/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding: the verbs and their arguments, mirroring the core operations; the human output (tree, waves, colours, `NO_COLOR`) and the `--json` output; how an amendment description is passed; how a structural batch is given (a file, standard input); what `rdm` reuses from the old `dirtree-rdm` rendering; how the shared CLI and MCP equivalence test is built so that neither consumer waits for the other.
No adversarial verification: the gates are mechanical.
**Deliverables**: `__roadmap__/m5-cli/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m5-cli/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m5-cli milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m5-cli`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: `crates/rdm-cli` with the `rdm` binary and its verbs; the equivalence test
**Consistency Checks**: `cargo test -p rdm-cli` (expected: PASS)
**Commit**: `feat(rdm-cli): land the rdm CLI`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m5-cli/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M5` complete; `skills-rdm-M7` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m5-cli/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/core/engine/interchange/consumers/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/core/engine/interchange/consumers/` (expected: PASS)
**Commit**: `docs(reports): close the m5-cli milestone`
