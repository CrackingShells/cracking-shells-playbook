# rdm-schema crate

**Goal**: `crates/rdm-schema` holds the stored types, the read-model types and the two version constants, with no store and no I/O. JSON Schemas are generated from the types and committed, and a test fails when they drift.
**Pre-conditions**:
- [ ] `../../read_model.md` is done and reviewed by Eliott
- [ ] The workspace skeleton is on the branch (root `Cargo.toml` with `members = ["crates/*"]`)
**Success Gates**:
- ⬜ [run] `cargo test -p rdm-schema` passes
- ⬜ [run] Editing any committed file under `crates/rdm-schema/schemas/` by hand makes `cargo test -p rdm-schema` fail, and regenerating with `RDM_SCHEMA_BLESS=1 cargo test -p rdm-schema` restores it. The verifier demonstrates this
- ⬜ [run] A JSON document with an unknown field at every object level deserialises and serialises back with the unknown fields intact (I12)
- ⬜ [static] Every type and field in `stored-schema.md` and `read-model.md` exists in the crate under the same name, and no field exists in the crate that the documents lack
**References**: [stored-schema.md, read-model.md, statuses.md](../../../../__design__/rdm/) — the specification these types implement; [R01 §Data Model](../../../../__design__/rdm/architecture.md) — forward compatibility rules

## Step 1: Stored types and versions
**Goal**: The stored schema exists as Rust types.
**Implementation Logic**:
1. **Crate:** `crates/rdm-schema`, version `0.1.0`, `publish = false` until M7, edition and `rust-version` taken from the workspace.
   - Dependencies: `serde` with derive, `serde_json` and `schemars = "1"`.
   - It has no I/O and no store.
2. **Types:** one type per document section, using the field names of `stored-schema.md` exactly.
3. **Status:** the `Status` enum has the five statuses from `statuses.md`, serialised in lowercase.
4. **Unknown fields:** every object type carries `#[serde(flatten)] extra: serde_json::Map<String, Value>`, so unknown fields round-trip.
5. **Version constants:** `FORMAT_VERSION: (u16, u16)` and `READ_MODEL_VERSION`.
6. **Tests:**
   - Each type deserialises the matching example from the spec documents.
   - The unknown-field round trip works at every object level.
**Deliverables**: `crates/rdm-schema/Cargo.toml`; `crates/rdm-schema/src/lib.rs`: consts `FORMAT_VERSION`, `READ_MODEL_VERSION`; `crates/rdm-schema/src/stored.rs`: types `Directory`, `Leaf`, `Step`, `Criterion`, `Reference`, `Commit`, `Check`, `Progress`, `Actor`, `History`, `Comment`, `Amendment`, `Change`, enum `Status` (names follow stored-schema.md); `crates/rdm-schema/tests/stored.rs`; `Cargo.lock`
**Consistency Checks**: `cargo test -p rdm-schema` (expected: PASS)
**Commit**: `feat(rdm-schema): add the stored types and the format version`

## Step 2: Read-model types
**Goal**: The read model exists as Rust types.
**Implementation Logic**:
1. Add one type per projection of `read-model.md`, with `Level { level, level_source }` where `level_source` is an enum holding only `Depth` for now.
2. The worked examples from `read-model.md`, saved as test fixtures, deserialise and re-serialise to equal JSON.
**Deliverables**: `crates/rdm-schema/src/read.rs`: types `Envelope`, `OutlineView`, `NodeView`, `ReadyView`, `AmendmentsView`, flat `level` and `level_source` fields, enum `LevelSource` (names follow read-model.md); `crates/rdm-schema/tests/read.rs`; `crates/rdm-schema/tests/fixtures/*.json`
**Consistency Checks**: `cargo test -p rdm-schema` (expected: PASS)
**Commit**: `feat(rdm-schema): add the read-model types`

## Step 3: Generated schemas and the drift test
**Goal**: The JSON Schemas are committed, and CI catches any drift.
**Implementation Logic**:
1. **Drift test:** `tests/schemas.rs` generates one JSON Schema per top-level stored type and per projection with schemars, as pretty JSON with a trailing newline, and compares it byte for byte with `schemas/<name>.schema.json`.
   - With `RDM_SCHEMA_BLESS=1` it writes the files instead.
   - Without that variable it fails, naming the file and the first differing line.
2. **CI:** the skeleton's `cargo test --workspace --locked` job already runs it. Do not edit the workflow.
3. **Proof the test can fail:** before you commit, edit one committed schema by hand and run the test to see it fail. Record that in your report.
**Deliverables**: `crates/rdm-schema/tests/schemas.rs`: functions `generated`, `check_or_bless`; `crates/rdm-schema/schemas/*.schema.json`
**Consistency Checks**: `cargo test -p rdm-schema` (expected: PASS)
**Commit**: `feat(rdm-schema): commit the generated json schemas with a drift test`
