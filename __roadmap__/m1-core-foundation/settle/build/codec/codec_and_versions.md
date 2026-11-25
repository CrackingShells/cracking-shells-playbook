# Codec and versions

**Goal**: `rdm_engine::codec` encodes every stored type to one canonical text and decodes any valid text losslessly, reading the format version first and dispatching through the version chain.
**Pre-conditions**:
- [ ] `../schema_contracts.md` is done
**Success Gates**:
- ⬜ [run] `cargo test -p rdm-engine --test codec canonical_files_round_trip_byte_for_byte` passes over hand-written canonical fixtures, each reviewed line by line against `contracts.md`
- ⬜ [run] `cargo test -p rdm-engine --test codec decode_encode_is_identity` passes: a proptest over arbitrary stored values
- ⬜ [run] `cargo test -p rdm-engine --test codec noncanonical_valid_input_reads_and_reencodes_canonically` passes, with one row per real class: reordered keys, four-space indent, CRLF, no trailing newline, `\u` escapes, and the output of `git merge-file` on two edits of one node (the fixture records the command that produced it)
- ⬜ [run] `cargo test -p rdm-engine --test codec unknown_keys_survive_in_place_at_every_level` passes at the byte level, and decoding reports exactly one `unknown_key` diagnostic per kept key, naming the file and the key's JSON path (Eliott, 2026-11-25)
- ⬜ [run] `cargo test -p rdm-engine --test codec newer_file_refused_before_parsing` passes with two rows, a higher X (`0.2.0`, a known field changed type) and a higher Y (`0.1.1`): each returns `NewerFormat { found, supported }`, not a parse error. Every newer file is refused, whatever part is higher (Eliott, 2026-11-25)
- ⬜ [static] The value-level stored-type tests of `crates/rdm-schema/tests/stored.rs` that these byte-level guards supersede are culled, and the report lists them; the read-model tests stay, because M4 serialises those directly
- ⬜ [run] `! cargo test --workspace -- --list 2>/dev/null | grep confirm_` passes, and the report names, for each guard, the bug that turned it red
**References**: [contracts.md](../../../../../__design__/rdm/contracts.md) — Version chain, the canonical form; [stored-schema.md](../../../../../__design__/rdm/stored-schema.md) — field order, line arrays for prose, where `null` is allowed

## Step 1: Canonical encoding
**Goal**: One value has exactly one text.
**Implementation Logic**:
1. Write the canonical fixtures by hand from `contracts.md`, then the three guards (byte round trip, identity property with `proptest` as a dev-dependency that builds on 1.88, unknown keys in place). See them red against plain `serde_json::to_string_pretty`.
2. Implement `encode` and `decode`: struct field order, two-space indent, LF, a trailing newline, prose as line arrays, unknown keys after the known ones, `null` only where the contract has it (`Change.new`).
3. Cull the superseded value-level tests in `rdm-schema` and list them.
**Deliverables**: `crates/rdm-engine/src/codec.rs`: fns `encode`, `decode`; `crates/rdm-engine/tests/codec.rs`: the three guards; `crates/rdm-engine/tests/fixtures/canonical/*.json`
**Consistency Checks**: `cargo test -p rdm-engine --test codec` (expected: PASS)
**Commit**: `feat(rdm-engine): add the canonical node encoding`

## Step 2: Non-canonical input
**Goal**: Whatever a human, an editor or git writes still reads, and is rewritten canonically.
**Implementation Logic**:
Produce each non-canonical fixture by the real transformation it stands for (an editor's reformat, `unix2dos`-style CRLF, `git merge-file` on two edits) and record the command beside it. Write the parameterized guard first.
**Deliverables**: `crates/rdm-engine/tests/fixtures/noncanonical/*.json`; test `noncanonical_valid_input_reads_and_reencodes_canonically`
**Consistency Checks**: `cargo test -p rdm-engine --test codec noncanonical` (expected: PASS)
**Commit**: `test(rdm-engine): guard non-canonical input of node files`

## Step 3: Version chain
**Goal**: Old files read through their layer, newer files are refused before parsing.
**Implementation Logic**:
1. Write `newer_file_refused_before_parsing` first and see it red against a decoder that parses the whole struct first.
2. Implement `read_version`, the `FormatLayer` trait with the `NotInVersion` default, and the `V0_1` layer, as `contracts.md` specifies.
3. Exercise fall-through with a test-only older layer as a `confirm_` test while building, then cull it: no older format exists, so it would test data no code produces. The first real second layer brings that guard.
**Deliverables**: `crates/rdm-engine/src/version.rs`: fn `read_version`, trait `FormatLayer`, type `V0_1`; test `newer_file_refused_before_parsing`
**Consistency Checks**: `cargo test -p rdm-engine --test codec newer_file_refused_before_parsing` (expected: PASS)
**Commit**: `feat(rdm-engine): read node files through the format version chain`
