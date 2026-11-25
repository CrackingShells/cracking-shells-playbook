# Reasoned edits and stable ids

**Goal**: Finished rdm nodes accept every edit with a non-blank reason that is counted, and every addressable sub-item has a stable id that records point to.
**Pre-conditions**:
- [ ] `criteria_and_versions.md` is done
**Success Gates**:
- ⬜ [static] The four documents say that `done` and `cancelled` nodes accept edits, status reversals and corrections with a non-blank reason, each marked as a correction, and no sentence calls done work sealed or terminal
- ⬜ [static] The four documents and the crate give every sub-item a stable id, records point by id, and no record refers to a sub-item by position
- ⬜ [dynamic] `cargo test -p rdm-schema --locked` and `uv run --no-project tools/check_rdm_traceability.py` pass
**References**: [R01](../../../../../__design__/rdm/architecture.md) — sections to sync; [the spec documents](../../../../../__design__/rdm/) — the text to amend

## Step 1: Amend the specs
**Goal**: The four documents state the reasoned-edit rule (Eliott's review, 2026-11-16) and the stable-id invariant.
**Implementation Logic**:
1. Replace sealing by a reason requirement on finished nodes, flag each correction and add a derived `correction_count` to the node detail.
2. Give every sub-item a stable id, point records by id, and fix any other positional reference.
**Deliverables**: `__design__/rdm/statuses.md`, `__design__/rdm/stored-schema.md`, `__design__/rdm/traceability.md`, `__design__/rdm/read-model.md`: finished-node rule, correction flag, `correction_count`, stable-id invariant, Decisions rows
**Consistency Checks**: `! grep -n 'Nothing reopens done work' __design__/rdm/statuses.md` (expected: PASS)
**Commit**: `docs(design): allow reasoned edits of finished rdm nodes and give sub-items stable ids`

## Step 2: Amend the crate
**Goal**: The types, tests and schemas match the amended specs.
**Implementation Logic**:
1. Add ids, `criterion_id` and the correction flag to `stored.rs`, and `correction_count` to `read.rs`.
2. Update tests, then regenerate every schema with `RDM_SCHEMA_BLESS=1`.
**Deliverables**: `crates/rdm-schema/src/stored.rs`, `crates/rdm-schema/src/read.rs`: the id fields, `Mark`, the correction flag, `correction_count`; `crates/rdm-schema/schemas/*.schema.json`
**Consistency Checks**: `cargo test -p rdm-schema --locked` (expected: PASS)
**Commit**: `feat(rdm-schema)!: add reasoned corrections and stable sub-item ids`

## Step 3: Sync R01
**Goal**: R01 restates the finished-node rule and the schema sketch lines.
**Implementation Logic**:
1. Touch only the invariants, FSM and sketch lines these rulings change.
**Deliverables**: `__design__/rdm/architecture.md`: I5, I8, I9, the FSM, the schema sketch criterion, step and mark lines
**Consistency Checks**: `uv run --no-project tools/check_rdm_traceability.py` (expected: PASS)
**Commit**: `docs(design): sync r01 with reasoned edits and stable ids`
