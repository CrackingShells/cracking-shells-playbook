# Codec

## Context
Depth 3 of M1. It needs the contract types of `schema_contracts`. The codec turns those types into the canonical text that git diffs and merges, and reads every format version through the chain.

## Goal
Bytes in and out of a node file are canonical, lossless and version-aware.

## Pre-conditions
- [ ] `../schema_contracts.md`, `../fs_primitives.md` and `../ci_matrix.md` are done

## Success Gates
- ⬜ `cargo test --workspace --locked` passes after the merge

## Gotchas
- One implementer builds `codec_and_versions` and then, resumed, `store/campaign_store.md`: the store is the codec's first consumer.

## Status
```mermaid
graph TD
    codec_and_versions[Codec and versions]:::done
    store[Store]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `codec_and_versions.md` | 📄 Leaf Task | ✅ Done |
| `store/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
