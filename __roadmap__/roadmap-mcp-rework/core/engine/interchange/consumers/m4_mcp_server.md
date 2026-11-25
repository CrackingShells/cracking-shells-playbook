# M4 MCP server

**Goal**: `rdm-mcp` exposes the core to agents as a small set of stdio MCP tools, with no roadmap logic of its own.
**Pre-conditions**:
- [ ] `../m3_interchange.md` is `done`
- [ ] Reminder `skills-rdm-M4` read
**Success Gates**:
- ⬜ The server passes the MCP 2026-07-28 conformance suite over stdio
- ⬜ Every tool returns a text summary and `structuredContent` matching its `outputSchema`, which is the read-model type of `rdm-schema` wherever one exists; read tools are marked read-only; no tool requires user interaction (I11)
- ⬜ An author, execute, amend cycle on a campaign runs from an agent session with no shell call
- ⬜ Every tool handler maps to one core call (I2), and every call names its campaign (I10)
- ⬜ Tool results stay within a stated size budget on the largest existing campaign
**References**: [R01 §Tool surface (suggestion), §Contracts & Invariants](../../../../../../__design__/rdm/architecture.md) — the suggested tools and the constraints that are fixed; [M0 spike knowledge transfer](../../../../../../__reports__/m0-feasibility-spike/01-knowledge_transfer_v0.md) — the throwaway `crates/rdm-mcp` was deleted at M1; the `rmcp` 3.10 stdio wiring can be read back with `git show 58a4a30:crates/rdm-mcp/…`; [M1 knowledge transfer](../../../../../../__reports__/m1-core-foundation/04-knowledge_transfer_v0.md) — the items M1 carried to M4 (§Carried forward)

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m4-mcp-server/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding: the final tool list, names and order (R01's table is a suggestion: keep it small, and keep one tool for every re-branching and move and one for status including `done`); input schemas, which wrap the store operation request types M1 froze in `rdm-schema` (amendment A2), and the envelope around the read-model types; the error mapping from M1's error type to tool errors; output size budgets and pagination; which tools are safe to allow-list for unattended runs; how conformance is run in CI; what the server answers a host that lacks MCP Apps.
Adversarial verification is required for the error mapping and the stale-revision path only; the rest is checked by conformance.
**Deliverables**: `__roadmap__/m4-mcp-server/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m4-mcp-server/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m4-mcp-server milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m4-mcp-server`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: `crates/rdm-mcp` with the tool handlers, schemas and error mapping; the conformance job
**Consistency Checks**: `cargo test -p rdm-mcp` (expected: PASS)
**Commit**: `feat(rdm-mcp): land the MCP server`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m4-mcp-server/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M4` complete; `skills-rdm-M6`, `skills-rdm-M7` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m4-mcp-server/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/core/engine/interchange/consumers/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/core/engine/interchange/consumers/` (expected: PASS)
**Commit**: `docs(reports): close the m4-mcp-server milestone`
