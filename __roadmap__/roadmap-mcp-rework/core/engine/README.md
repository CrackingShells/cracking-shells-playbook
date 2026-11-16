# Engine

## Context
Depth 2. Runs after M1's store. M2 adds everything that understands the tree: validation, order, the status machine and the operations that change it.

## Goal
`rdm-engine` can validate, order and restructure a campaign, with every edit recorded as an amendment.

## Pre-conditions
- [ ] `m1_core_foundation.md` is `done`

## Success Gates
- ⬜ `m2_graph_engine.md` is `done`

## Status
```mermaid
graph TD
    m2_graph_engine[M2 Graph engine]:::planned
    interchange[Interchange]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `m2_graph_engine.md` | 📄 Leaf Task | ⬜ Planned |
| `interchange/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
