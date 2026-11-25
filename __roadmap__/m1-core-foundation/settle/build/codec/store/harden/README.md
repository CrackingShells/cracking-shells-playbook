# Harden

## Context
Depth 5 of M1. Two independent pieces on the finished store: the crash test that guards I3, and the hot cache that guards I10. They touch different modules and test files.

## Goal
No kill loses an acknowledged write, and the long-lived process serves reads from memory without ever serving a stale one.

## Pre-conditions
- [ ] `../campaign_store.md` is done

## Success Gates
- ⬜ `cargo test --workspace --locked` passes after both merges, the must-run set in under 30 seconds
- ⬜ Integration review by the coordinator: the effects `RecordingFs` logs are the effects `RealFs` performs, in the same order
- ⬜ Eliott's go for the push of `feat/m1-core-foundation` is asked here, once, for the three-OS run

## Status
```mermaid
graph TD
    crash_test[Crash test]:::planned
    hot_cache[Hot cache]:::planned
    verify[Verify]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `crash_test.md` | 📄 Leaf Task | ⬜ Planned |
| `hot_cache.md` | 📄 Leaf Task | ⬜ Planned |
| `verify/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
