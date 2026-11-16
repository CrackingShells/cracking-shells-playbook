# Delivery

## Context
Depth 5. Runs after the consumers. M6 (the graph view) needs only M4; M7 (publication and launch) needs M4 and M5. They are independent and run in parallel.

## Goal
The server shows the roadmap as a graph and reaches consumers as a published, self-updating binary.

## Pre-conditions
- [ ] `m4_mcp_server.md` and `m5_cli.md` are `done`

## Success Gates
- ⬜ `m6_apps_view.md` and `m7_distribution.md` are `done`

## Status
```mermaid
graph TD
    m6_apps_view[M6 Apps view]:::planned
    m7_distribution[M7 Distribution]:::planned
    cutover[Cutover]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `m6_apps_view.md` | 📄 Leaf Task | ⬜ Planned |
| `m7_distribution.md` | 📄 Leaf Task | ⬜ Planned |
| `cutover/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
