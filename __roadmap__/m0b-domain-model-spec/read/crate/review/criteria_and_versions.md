# Enforced criteria and 0.X.Y versions

**Goal**: The m0b specs, the `rdm-schema` crate and R01 enforce pre-conditions and gates, trace every criterion mark, and use three-part `0.X.Y` versions.
**Pre-conditions**:
- [ ] `../rdm_schema_crate.md` and `../r01_sync.md` are done
**Success Gates**:
- ⬜ [static] `statuses.md`, `stored-schema.md`, `traceability.md` and `read-model.md` state that `planned → ongoing` needs every pre-condition `met` and `ongoing → done` needs every gate `met`, and no sentence says `met` gates nothing
- ⬜ [static] The four documents and the crate define a trace record for each criterion mark, on leaves and on directories
- ⬜ [static] Format, read-model and crate versions are `0.X.Y`, and bumps are derived from commit messages by the existing release setup, with M7 wiring the crate
- ⬜ [dynamic] `cargo test -p rdm-schema --locked` and `uv run --no-project tools/check_rdm_traceability.py` pass
**References**: [R01](../../../../../__design__/rdm/architecture.md) — sections to sync; [the spec documents](../../../../../__design__/rdm/) — the text to amend; [release-config.js](../../../../../release-config.js) — the existing release setup

## Step 1: Amend the specs
**Goal**: The four documents state the enforced criteria and the 0.X.Y rule.
**Implementation Logic**:
1. Make criteria gate `planned → ongoing` (override path included) and `ongoing → done`; a directory is `done` only when its leaves are finished and its own gates are met.
2. Add a mark record (`ts`, `actor`, kind, index, value, `evidence?`) to the leaf history and a matching log on `Directory`; `met` stays a state mark, not content.
3. Report each `done` node with an unmet criterion as a `validate` error; the importer never invents a tick.
4. Make the format and read-model versions `0.X.Y`, with bumps derived from commit types and scopes.
**Deliverables**: `__design__/rdm/statuses.md`, `__design__/rdm/stored-schema.md`, `__design__/rdm/traceability.md`, `__design__/rdm/read-model.md`: criteria rule, mark records, legacy-import rule, version rule, Decisions rows
**Consistency Checks**: `! grep -n 'Gate marks do not gate' __design__/rdm/statuses.md` (expected: PASS)
**Commit**: `docs(design): enforce rdm criteria and three-part versions`

## Step 2: Amend the crate
**Goal**: The types, versions, tests and schemas match the amended specs.
**Implementation Logic**:
1. Add the mark record type and the directory log to `stored.rs`; make `Version` a `(u16,u16,u16)` and update `parse_version` and `is_readable`.
2. Update tests, then regenerate every schema with `RDM_SCHEMA_BLESS=1`.
**Deliverables**: `crates/rdm-schema/src/stored.rs`, `crates/rdm-schema/src/read.rs`, `crates/rdm-schema/src/lib.rs`: `Version`, `FORMAT_VERSION`, `READ_MODEL_VERSION`, `parse_version`, `is_readable`, the mark record type; `crates/rdm-schema/schemas/*.schema.json`
**Consistency Checks**: `cargo test -p rdm-schema --locked` (expected: PASS)
**Commit**: `feat(rdm-schema)!: enforce criteria and use three-part versions`

## Step 3: Sync R01
**Goal**: R01 restates the I8 neighbourhood, the version rule and the schema sketch lines.
**Implementation Logic**:
1. Touch only the lines these rulings change, and neither §Proposed State nor §Risks & Mitigations.
**Deliverables**: `__design__/rdm/architecture.md`: I8, the launch version rule, the schema sketch criterion and history lines
**Consistency Checks**: `uv run --no-project tools/check_rdm_traceability.py` (expected: PASS)
**Commit**: `docs(design): sync r01 with the m0b review`
