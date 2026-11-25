# Verify

## Context
Depth 6 of M1, the last level. A fresh verifier attacks the whole milestone after everything is merged; per-leaf verifiers have already reviewed their leaves.

## Goal
The milestone's guards are shown to catch the bugs they claim, and the three-OS evidence is recorded.

## Pre-conditions
- [ ] `../crash_test.md` and `../hot_cache.md` are done

## Success Gates
- ⬜ Every finding is fixed by the leaf's original implementer or recorded with Eliott's decision

## Status
```mermaid
graph TD
    adversarial_review[Adversarial review]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `adversarial_review.md` | 📄 Leaf Task | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
