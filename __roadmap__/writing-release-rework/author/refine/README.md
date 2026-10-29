# Refine Content

## Context
Level 2: two sequential content passes over every shipped file. The best-practices pass runs first; the stand-alone pass runs second, so nothing re-contaminates the files after it. Both edit the same files, which is why they are depth-ordered rather than siblings.

## Goal
Make the manual best-practice compliant and stand-alone, without changing a rule or a fact.

## Pre-conditions
- [ ] `author/` closed: all three leaves are merged and the integration review has passed
- [ ] `chores.md` is merged

## Success Gates
- ✅ `best_practices.md` is merged with its findings table in the leaf report
- ✅ `standalone/` is closed

## Gotchas
- A content pass that wants to change a rule, a threshold or a fact must flag it rather than make the change; the coordinator adjudicates.

## Status
```mermaid
graph TD
    best_practices[Best-Practices Pass]:::done
    standalone[Stand-Alone Pass]:::done
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `best_practices.md` | 📄 Leaf Task | ✅ Done |
| `standalone/` | 📁 Directory | ✅ Done |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
