# M6 Apps view

**Goal**: A host that supports MCP Apps shows the roadmap as a graph: waves, statuses, leaf details and the amendment log.
**Pre-conditions**:
- [ ] `../m4_mcp_server.md` is `done` (the tree also waits for `../m5_cli.md`; the reminder needs only M4)
- [ ] Reminder `skills-rdm-M6` read
**Success Gates**:
- ⬜ The view renders in this desktop app from the installed server, for the largest existing campaign
- ⬜ The page makes zero network requests and stays under the size budget set in the plan
- ⬜ The view writes nothing: it only calls the app-only read tool
- ⬜ The built HTML is committed, and CI rebuilds it and fails on a difference
- ⬜ A host without MCP Apps receives the same information as text and structured content
**References**: [R01 §Graph view, §Tool surface (suggestion)](../../../../../../../__design__/rdm/architecture.md) — what the view shows, how it is built and how it degrades; [M0 spike knowledge transfer](../../../../../../../__reports__/m0-feasibility-spike/01-knowledge_transfer_v0.md) — the throwaway `crates/rdm-mcp` was deleted at M1; the Svelte graph view and its MCP Apps bridge can be read back with `git show 58a4a30:crates/rdm-mcp/…`

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m6-apps-view/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding: the layout (waves as columns, directories as groups, the detail pane, the amendment timeline) and how it reads on a narrow panel; the data contract of the app-only tool and how TypeScript types are generated from the `rdm-schema` JSON Schemas (the app-only tool returns the read model); refresh (polling interval, manual refresh); theming from the host; the Svelte and Vite setup producing one inlined file, the pinned Node version and a deterministic build; the size budget; the headless smoke test.
No adversarial verification; Eliott reviews the view by eye.
**Deliverables**: `__roadmap__/m6-apps-view/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m6-apps-view/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m6-apps-view milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m6-apps-view`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: the Svelte source under `crates/rdm-mcp/ui/`; the built `crates/rdm-mcp/assets/view.html`; the app-only tool; the smoke test and the rebuild check in CI
**Consistency Checks**: `cargo test -p rdm-mcp` (expected: PASS)
**Commit**: `feat(rdm-mcp): land the roadmap graph view`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m6-apps-view/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M6` complete; `skills-rdm-M8` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m6-apps-view/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/core/engine/interchange/consumers/delivery/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/core/engine/interchange/consumers/delivery/` (expected: PASS)
**Commit**: `docs(reports): close the m6-apps-view milestone`
