# Schema contracts

**Goal**: Every type of `contracts.md` exists in `rdm-schema` under the same name, the stored types match the amended `stored-schema.md`, and the guards hold the contracts against their oracles.
**Pre-conditions**:
- [ ] `../r01_sync.md` is done
**Success Gates**:
- ⬜ [run] `cargo test -p rdm-schema --test schemas committed_schemas_match_the_types` passes (the existing drift guard covers the new types)
- ⬜ [run] `cargo test -p rdm-schema move_table_matches_statuses_doc` passes: the test parses the move table of `__design__/rdm/statuses.md` and compares it with `MOVES` cell for cell, in both directions
- ⬜ [run] `cargo test -p rdm-schema links_cover_every_id_field` passes: every property of the generated JSON Schemas that references an id newtype is declared in `Links`, and nothing else is
- ⬜ [run] `cargo test -p rdm-schema malformed_ids_are_refused` passes, with one row per class: wrong prefix, letter outside the alphabet, wrong length
- ⬜ [static] Every type and field in `contracts.md` and `stored-schema.md` exists in the crate under the same name, and the crate has none the documents lack
- ⬜ [run] `! cargo test --workspace -- --list 2>/dev/null | grep confirm_` passes, and the report names, for each new guard, the bug that turned it red
**References**: [contracts.md, stored-schema.md, statuses.md](../../../../__design__/rdm/) — the specification; [crates/rdm-schema](../../../../crates/rdm-schema/) — the current types and tests

## Step 1: Ids and stored types
**Goal**: The stored types match the amended schema.
**Implementation Logic**:
1. Add the id newtypes with validating `Deserialize` and their schemas, and the node fields `name`, `parent_id` and `order`.
2. Remove the ids `contracts.md` drops (checks and references) and make `Reference.target` derived when `node_id` is set.
3. Change the existing guards whose behaviour changed on purpose, before the code: `ids_are_unique_and_changes_name_items_by_id` and the reference fixtures. Cull `version_constants_and_strings` unless it is the only test of `version_string` formatting.
4. Write `malformed_ids_are_refused` first and see it red.
5. Regenerate the schemas with `RDM_SCHEMA_BLESS=1` and review the diff line by line.
**Deliverables**: `crates/rdm-schema/src/ids.rs`: types `NodeId`, `CriterionId`, `StepId`; `crates/rdm-schema/src/stored.rs` amended; `crates/rdm-schema/tests/stored.rs`: test `malformed_ids_are_refused`; `crates/rdm-schema/schemas/*.schema.json`
**Consistency Checks**: `cargo test -p rdm-schema` (expected: PASS)
**Commit**: `feat(rdm-schema)!: give nodes ids, parents and order keys`

## Step 2: Campaign model and moves
**Goal**: The campaign model and the move table exist as data.
**Implementation Logic**:
1. Add the campaign model types of `contracts.md` (node table by id; derived children, depth and path are computed by the engine, not stored).
2. Add `MOVES` as a constant table. Write `move_table_matches_statuses_doc` first: it reads `statuses.md` from the repository (path from `CARGO_MANIFEST_DIR`) and parses its table; no copy of the table lives in the test.
**Deliverables**: `crates/rdm-schema/src/campaign.rs`: type `Campaign` and the types `contracts.md` names; `crates/rdm-schema/src/moves.rs`: const `MOVES`, type `Move`; `crates/rdm-schema/tests/moves.rs`: test `move_table_matches_statuses_doc`
**Consistency Checks**: `cargo test -p rdm-schema --test moves` (expected: PASS)
**Commit**: `feat(rdm-schema): add the campaign model and the move table`

## Step 3: Operations, errors and links
**Goal**: The store-level operation types, the error type and the declared links exist.
**Implementation Logic**:
1. Add one request and one result type per store primitive and the error type, as `contracts.md` names them. No tool envelope.
2. Add the `Links` declaration. Write `links_cover_every_id_field` first: it walks the generated schemas for references to the id newtypes, so the schema, not a hand list, is the oracle.
**Deliverables**: `crates/rdm-schema/src/ops.rs`: the request and result types; `crates/rdm-schema/src/error.rs`: type `StoreError`; `crates/rdm-schema/src/links.rs`: trait or const `Links`; `crates/rdm-schema/tests/links.rs`: test `links_cover_every_id_field`
**Consistency Checks**: `cargo test -p rdm-schema` (expected: PASS)
**Commit**: `feat(rdm-schema): add the store operations, error type and declared links`
