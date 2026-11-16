# M0 Feasibility spike

## Context
Milestone M0 of the rdm meta-roadmap (`__roadmap__/roadmap-mcp-rework/m0_feasibility_spike.md`), run in parallel with M0b. It proves the pinned stack on real hosts before M1 builds on it: a stub `rdm-mcp` on `rmcp`, speaking MCP 2026-07-28 over stdio, with a Svelte `ui://` page that draws a throwaway fixture. Its report gives M4, M6 and M7 a go or no-go per host.

## Reference Documents
- [R01 Architecture](../../__design__/rdm/architecture.md) — §Proposed State, §Graph view and §Risks & Mitigations: the pinned stack, the Apps mechanics and the risks this spike settles
- [R02 Meta-roadmap leaf](../roadmap-mcp-rework/m0_feasibility_spike.md) — the milestone gates this campaign must meet
- [R03 Domain model gap analysis](../../__reports__/rdm_domain_model_review/01-gap_analysis_v0.md) — why M0 is reduced to a feasibility check and is independent of the schema

## Goal
Show, on the real hosts, that the stub server and its `ui://` page work: the page renders, opens a full view and one leaf's content, and makes no network request. Record each host's capabilities and context and the time to display, and give a go or no-go per host.

## Pre-conditions
- [x] Plan approved by Eliott (2026-11-16)
- [ ] `workspace_skeleton.md` is done on the shared base before `feat/m0-feasibility-spike` is cut

## Success Gates
- ⬜ `cargo build --workspace` and `cargo test -p rdm-mcp` pass on `feat/m0-feasibility-spike`
- ⬜ Every gate of the meta-roadmap leaf is recorded in `__reports__/m0-feasibility-spike/` as PASS, DEVIATION or BLOCKED, with its evidence
- ⬜ The report gives a go or no-go for Claude Code Desktop, VS Code Copilot and the Claude Code CLI

## Gotchas
- **Decisions already taken (2026-11-16, Eliott):**
  - The core crate is `rdm-engine`, because `rdm-core` is taken.
  - The primary host is Claude Code Desktop, this app. VS Code Copilot is best effort and is not a blocker. The CLI does not render MCP Apps, so it is tested for tool calls only.
  - The server speaks 2026-07-28 only, with no fallback and no diagnostic flag. A host that fails is a finding.
- **Pins:**
  - `rmcp = "=3.10.0"` with the features `server`, `macros` and `transport-io`. 3.10.0 came out on 2026-11-12, after R01's 3.9.0.
  - `@modelcontextprotocol/ext-apps` is pinned exactly, at 2.0.4 or later, with `vite-plugin-singlefile` 2.3.4.
  - Node 26 and npm 11 are local.
- **What rmcp does and does not type:** rmcp has no typed MCP Apps API. The server builds these values by hand:
  - the tool's `_meta.ui.resourceUri`, through `Tool::with_meta`
  - `_meta.ui.visibility = ["app"]` on the app-only tools
  - `capabilities.extensions["io.modelcontextprotocol/ui"] = {"mimeTypes":["text/html;profile=mcp-app"]}`
  - the resource MIME type `text/html;profile=mcp-app`
- **The view's protocol:** the view announces Apps `protocolVersion` `2026-01-26` in `ui/initialize`.
- **Known host issues before the spike:**
  - VS Code has an open issue saying it does not negotiate 2026-07-28.
  - The desktop app has an open report (2026-11-07) that local stdio apps are not rendered.
- **`basic-host` is not a baseline.** The `basic-host` of ext-apps has no stdio transport, and HTTP is a non-goal.
- **The fixture shape is the contract** between `stub_server` and `graph_view`. It is fixed in `build/stub_server.md` Step 1, and neither leaf changes it alone.
- **Shared resources:**
  - One shared `CARGO_TARGET_DIR` holds the builds of both M0 and M0b.
  - `node_modules/` is deleted once `view/dist/index.html` is committed.
  - Probe data goes to `$RDM_PROBE_DIR`, outside the repository. It is checked for personal data before any of it is copied into a report.
- **Budget:** the spike shares a USD 30 budget with M0b. It uses three Sonnet dispatches (`stub_server`, `graph_view`, `spike_report`), and the coordinator runs `host_trials` with Eliott.

## Status
```mermaid
graph TD
    workspace_skeleton[Workspace skeleton]:::planned
    build[Build the stub]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `workspace_skeleton.md` | 📄 Leaf Task | ⬜ Planned |
| `build/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
