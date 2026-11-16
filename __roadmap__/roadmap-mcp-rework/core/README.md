# Core

## Context
Depth 1 of the meta-roadmap. Runs after `m0_feasibility_spike`, which proves the stack and creates the workspace. M1 builds the store every later milestone writes through.

## Goal
`rdm-engine` holds a roadmap as JSON files that many processes can read and write safely.

## Pre-conditions
- [ ] `m0_feasibility_spike.md` is `done` with a go
- [ ] `m0b_domain_model_spec.md` is `done`

## Success Gates
- ⬜ `m1_core_foundation.md` is `done`

## Status
```mermaid
graph TD
    m1_core_foundation[M1 Core foundation]:::planned
    engine[Engine]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `m1_core_foundation.md` | 📄 Leaf Task | ⬜ Planned |
| `engine/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
