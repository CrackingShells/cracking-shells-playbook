# Interchange

## Context
Depth 3. Runs after M2. M3 completes the core with markdown export and the legacy importer, so that both consumers start on a finished library.

## Goal
A roadmap can leave the store as markdown and a legacy markdown roadmap can enter it.

## Pre-conditions
- [ ] `m2_graph_engine.md` is `done`

## Success Gates
- ⬜ `m3_interchange.md` is `done`

## Status
```mermaid
graph TD
    m3_interchange[M3 Interchange]:::planned
    consumers[Consumers]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `m3_interchange.md` | 📄 Leaf Task | ⬜ Planned |
| `consumers/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
