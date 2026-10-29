# Stand-Alone Pass

## Context
Level 3: the stand-alone pass (C5) over writing-release and the writing-history handoff edits. Then `review/` attacks the finished skill and closes the campaign.

## Goal
Leave no meta-discourse, project-specific name or single-data-point advice in any shipped file.

## Pre-conditions
- [ ] `best_practices.md` is merged

## Success Gates
- ✅ `standalone.md` is merged with its removed-passage list in the leaf report
- ✅ `review/` is closed

## Gotchas
- R01 serves as a blocklist here, not as source content.

## Status
```mermaid
graph TD
    standalone[Stand-Alone Pass]:::inprogress
    review[Review and Close]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `standalone.md` | 📄 Leaf Task | 🔄 In Progress |
| `review/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
