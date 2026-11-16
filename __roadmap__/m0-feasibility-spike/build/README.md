# Build the stub

## Context
Depth 1 of the spike. It runs after the workspace skeleton exists. Two independent leaves build the stub server, with its fixture, and the graph page; they share only the fixture shape fixed in `stub_server.md`. Their output is what `trials/` puts in front of real hosts.

## Goal
A stub `rdm-mcp` that serves the built graph page and passes its stdio integration test.

## Pre-conditions
- [ ] `../workspace_skeleton.md` is done

## Success Gates
- ⬜ `cargo test -p rdm-mcp` passes with the real `view/dist/index.html` embedded, not the placeholder
- ⬜ The coordinator's integration review finds that the server and the view agree on the fixture shape, the tool names and the probe payload

## Gotchas
- **The two leaves never write each other's files.** `stub_server` owns `crates/rdm-mcp/` except `view/`, and `graph_view` owns `crates/rdm-mcp/view/`.
- **`build.rs` embeds a placeholder** until `view/dist/index.html` exists. Each leaf can therefore build and land alone.

## Status
```mermaid
graph TD
    stub_server[Stub server]:::inprogress
    graph_view[Graph view]:::inprogress
    trials[Host trials]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `stub_server.md` | 📄 Leaf Task | 🔄 In Progress |
| `graph_view.md` | 📄 Leaf Task | 🔄 In Progress |
| `trials/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
