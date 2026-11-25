# R01 sync

**Goal**: R01 describes the flat, id-addressed store, its write algorithm and the contracts, with I3, I7 and I10 reworded to what M1 guards.
**Pre-conditions**:
- [ ] Eliott has reviewed `__design__/rdm/store.md` and `__design__/rdm/contracts.md`
**Success Gates**:
- ⬜ [run] `grep -c 'Names are paths' __design__/rdm/architecture.md` prints 0
- ⬜ [static] §Data Model describes `nodes/<id>.json` with `parent_id` and the order key; §Concurrency describes lock-free readers, the writer lock, ordered writes and the intent record, pointing to `store.md`
- ⬜ [static] I3, I7 and I10 read as guarded (I10: "a cached read returns what a cold read would at that moment"); §Deliberately open no longer lists the id format, minting or the journal format; §Risks drops the directory-rename risk on Windows
- ⬜ [static] `read-model.md` notes that `rev` and `tree_rev` are defined in `store.md`
**References**: [store.md, contracts.md](../../../__design__/rdm/) — the reviewed design; [R01](../../../__design__/rdm/architecture.md) — §Data Model, §Concurrency, §Contracts & Invariants, §Risks & Mitigations, §Deliberately open

## Step 1: Sync R01 and the read model
**Goal**: The architecture document matches the reviewed design.
**Implementation Logic**:
The coordinator does this leaf itself: it is a document sync with no code. Edit each section named in the gates, keeping R01's voice and its decision dates. Add a revision note naming amendment A2.
**Deliverables**: `__design__/rdm/architecture.md`; `__design__/rdm/read-model.md`
**Consistency Checks**: `test "$(grep -c 'Names are paths' __design__/rdm/architecture.md)" = 0` (expected: PASS)
**Commit**: `docs(design): sync r01 with the m1 store and contracts`
