# Build

## Context
Depth 2 of M1. Three independent pieces start in parallel on the reviewed design: the contract types in `rdm-schema`, the filesystem primitives in a new `rdm-engine`, and the CI matrix. They touch different directories (`crates/rdm-schema`, `crates/rdm-engine`, `.github`), so their branches do not conflict.

## Goal
The contracts exist as types, the primitives survive real processes on three operating systems, and CI can show it.

## Pre-conditions
- [ ] `../r01_sync.md` is done

## Success Gates
- ✅ `cargo test --workspace --locked` passes after the three merges
- ✅ Integration review by the coordinator: the error type `fs_primitives` returns is the one `schema_contracts` defines (`StoreError` and its constructors live in rdm-schema, confirmed by `03-full_review_harmony_v0.md`)

## Gotchas
- `fs_primitives` needs the error type from `schema_contracts`. Both start together: `fs_primitives` uses a local placeholder error enum with a `TODO(schema_contracts)` comment and switches to the `rdm_schema` type at rebase, before its merge.

## Status
```mermaid
graph TD
    schema_contracts[Schema contracts]:::done
    fs_primitives[Filesystem primitives]:::done
    ci_matrix[CI matrix]:::done
    codec[Codec]:::done
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `schema_contracts.md` | 📄 Leaf Task | ✅ Done |
| `fs_primitives.md` | 📄 Leaf Task | ✅ Done |
| `ci_matrix.md` | 📄 Leaf Task | ✅ Done |
| `codec/` | 📁 Directory | ✅ Done |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
