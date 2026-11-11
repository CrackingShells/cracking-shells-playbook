# M0 Feasibility spike

**Goal**: The stack is proven against real hosts before anything depends on it: a stub server on the pinned `rmcp` speaks MCP 2026-07-28 over stdio and its Svelte `ui://` page renders.
**Pre-conditions**:
- [ ] R01 read (`__design__/rdm/architecture.md`)
- [ ] Reminder `skills-rdm-M0` read
**Success Gates**:
- ⬜ A stub `rdm-mcp` with one tool and one `ui://` resource lists and calls its tool from this desktop app and from the Claude Code CLI
- ⬜ The `ui://` page, a Svelte build inlined into one HTML file, renders in this desktop app with zero network requests
- ⬜ The exact allow-rule string of the tool, read from a real session, is written down
- ⬜ The crate names `rdm-core`, `rdm-mcp` and `rdm-cli` are confirmed free on crates.io, or replacements are agreed with Eliott
- ⬜ A go/no-go is written; on no-go the later milestones are amended before they start
**References**: [R01 §Proposed State, §Graph view, §Risks & Mitigations](../../__design__/rdm/architecture.md) — the pinned stack, the Apps mechanics and the risks this spike settles

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m0-feasibility-spike/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding: what the stub contains and nothing more (one tool with structured output, one `ui://` page calling one app-only tool); how each host is tested and what counts as rendering; how the `rmcp` version is pinned and which of its calls set tool `_meta`, the Apps MIME type and the `io.modelcontextprotocol/ui` capability; whether the Apps client library fits a small bundle or a hand-written bridge is needed; where the workspace skeleton lives (`crates/`, an empty CI job); what is done if a host does not speak 2026-07-28 (the maintainer wants 2026-07-28 conformance only, so a failing host is a finding to bring back, not a reason to add a fallback).
No adversarial verification: the gates are observations on real hosts, made with Eliott.
**Deliverables**: `__roadmap__/m0-feasibility-spike/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m0-feasibility-spike/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m0-feasibility-spike milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m0-feasibility-spike`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: the Cargo workspace skeleton under `crates/` with a stub `rdm-mcp`; the spike report with the go/no-go under `__reports__/m0-feasibility-spike/`
**Consistency Checks**: `cargo build --workspace` (expected: PASS)
**Commit**: `feat(rdm-mcp): land the feasibility spike`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m0-feasibility-spike/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M0` complete; `skills-rdm-M1` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m0-feasibility-spike/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/` (expected: PASS)
**Commit**: `docs(reports): close the m0-feasibility-spike milestone`
