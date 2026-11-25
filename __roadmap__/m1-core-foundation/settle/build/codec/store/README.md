# Store

## Context
Depth 4 of M1. The store combines the codec and the filesystem primitives into the campaign operations every consumer calls.

## Goal
A campaign opens, validates, and changes through ordered, revision-checked writes.

## Pre-conditions
- [ ] `../codec_and_versions.md` is done

## Success Gates
- ⬜ `cargo test --workspace --locked` passes after the merge
- ⬜ Integration review by the coordinator: `rev` is computed from the codec's canonical encoding, the same bytes the cache will compare

## Status
```mermaid
graph TD
    campaign_store[Campaign store]:::done
    harden[Harden]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `campaign_store.md` | 📄 Leaf Task | ✅ Done |
| `harden/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
