# M1 Core foundation

**Goal**: `rdm-schema` holds the contracts every layer consumes (stored types, campaign model, move table as data, store operation types, declared links, version chain), and `rdm-engine` stores a campaign as flat, id-addressed JSON files that N processes read without coordination and write without losing or tearing a write, served hot from memory by a long-lived process.
**Pre-conditions**:
- [x] `../m0_feasibility_spike.md` is `done` with a go
- [x] `../m0b_domain_model_spec.md` is `done`
- [x] Reminder `skills-rdm-M1` read
**Success Gates**:
- ✅ `__design__/rdm/store.md` and `__design__/rdm/contracts.md` are reviewed by Eliott, and R01 matches them in the same merge
- ✅ `rdm-engine` holds no stored or contract type of its own and depends on `rdm-schema`; `MOVES` matches the move table of `statuses.md` and the declared links cover every id field
- ✅ Every older format version reads through the version chain; a newer one is refused with `NewerFormat` before anything else is parsed
- ✅ Canonical files round-trip byte for byte, valid non-canonical input re-encodes canonically, and unknown fields survive (I12)
- ✅ Every write prefix of every store operation loads and recovers to the state before or after it, and a multi-process kill test on macOS, Linux and Windows loses no acknowledged write (I3)
- ✅ A write presenting a stale revision is refused and changes nothing (I4); a rename or move writes one file and changes no `id` (I7); the reverse link index always equals a full scan
- ✅ A cached read equals a cold read at that moment for every kind of external change (I10)
**References**: [R01 §Data Model, §Concurrency, §Contracts & Invariants](../../../__design__/rdm/architecture.md) — superseded where amendment A2 says so; [A2 gap analysis](../../../__reports__/m1-core-foundation/00-gap_analysis_v0.md) — the rulings of 2026-11-25 this leaf implements

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m1-core-foundation/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps` from the A2 gap analysis: two design leaves first (`store.md`: flat layout, id format and minting, ordered writes and their recovery, lock, revisions, cache revalidation; `contracts.md`: id inventory, campaign model, move table as data, operation and error types, declared links, version chain), reviewed by Eliott before any code; then the contracts, the filesystem primitives and the CI matrix; the codec and version layer; the store; the crash test and the hot cache. Each leaf's gates are the guards of the campaign's test design.
Adversarial verification is required for the store, the lock, the intent record and the cache (silent data loss). Tool input and output stay with M4.
**Deliverables**: `__roadmap__/m1-core-foundation/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m1-core-foundation/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m1-core-foundation milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m1-core-foundation`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: the contract types in `crates/rdm-schema`; `crates/rdm-engine` modules for the filesystem primitives, codec and version chain, store and cache; the crash and multi-process tests; the three-OS CI matrix
**Consistency Checks**: `cargo test --workspace --locked` (expected: PASS)
**Commit**: `feat(rdm-engine): land the core foundation`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m1-core-foundation/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M1` complete; `skills-rdm-M2` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m1-core-foundation/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/core/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/core/` (expected: PASS)
**Commit**: `docs(reports): close the m1-core-foundation milestone`
