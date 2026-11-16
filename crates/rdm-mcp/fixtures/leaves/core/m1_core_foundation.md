# M1 Core foundation

**Goal**: `rdm-engine` stores a roadmap as JSON, one file per node, and N processes can read and write one campaign without losing or tearing a write.
**Pre-conditions**:
- [ ] `../m0_feasibility_spike.md` is `done` with a go
- [ ] `../m0b_domain_model_spec.md` is `done`
- [ ] Reminder `skills-rdm-M1` read
**Success Gates**:
- ⬜ `rdm-engine` depends on `rdm-schema` for every stored type and holds none of its own; a file with a newer format major is refused with an upgrade message
- ⬜ Canonical serialisation round-trips byte for byte, and unknown fields survive a read and a write (I12)
- ⬜ A multi-process test with kill injection at every journal stage passes on macOS, Linux and Windows: no lost update, no state that fails to load (I3)
- ⬜ A write presenting a stale revision is refused and changes nothing (I4)
- ⬜ A node's `id` is unchanged by a rename or a move (I7)
**References**: [R01 §Data Model, §Concurrency, §Contracts & Invariants](../../../__design__/rdm/architecture.md) — the file layout, the schema sketch, the lock, the journal and the revisions

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m1-core-foundation/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding: the id minting, to the format M0b fixes; canonical JSON (field order, line arrays for prose) over the `rdm-schema` types; how the store reads the format version; the lock crate (std's file lock needs a newer toolchain than the MSRV); the journal format and its recovery rule; the revision hash; how the multi-process test injects failures and how it runs on three operating systems; the error type consumers will map.
Adversarial verification is required for the store, lock and journal (silent data loss). The schemas were settled in M0b.
**Deliverables**: `__roadmap__/m1-core-foundation/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m1-core-foundation/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m1-core-foundation milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m1-core-foundation`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: `crates/rdm-engine` modules for the store (lock, atomic writes, journal, revisions) over the `rdm-schema` types; the multi-process test
**Consistency Checks**: `cargo test -p rdm-engine` (expected: PASS)
**Commit**: `feat(rdm-engine): land the core foundation`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m1-core-foundation/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M1` complete; `skills-rdm-M2` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m1-core-foundation/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/core/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/core/` (expected: PASS)
**Commit**: `docs(reports): close the m1-core-foundation milestone`
