# Contracts design

**Goal**: `__design__/rdm/contracts.md` specifies every contract type M1 freezes, with its Rust name, and `stored-schema.md` is edited to match, so that `schema_contracts` can be implemented from the documents alone.
**Pre-conditions**:
- [ ] Amendment A2 is on the branch (`8b4a4e7`) and the A2 gap analysis is read
- [ ] `__design__/rdm/stored-schema.md`, `read-model.md` and `statuses.md` are read
**Success Gates**:
- ⬜ [static] The id inventory gives, for every stored record, the verdict (id or no id) and the reason, by the rule "it is pointed at from outside, and its natural key can change"; `stored-schema.md` matches it
- ⬜ [static] Every contract type is listed with its fields and Rust name: the id newtypes, the node fields (`name`, `parent_id`, `order`), the campaign model and what it derives, `MOVES`, the store operation request and result types, the error type, the `Links` declaration and the `FormatLayer` trait
- ⬜ [static] The version chain states the trait, the `NotInVersion` default, how a layer overrides, that a newer file is refused with `NewerFormat { found, supported }` before anything else is parsed, the write-version rule, and whether `FORMAT_VERSION` stays `0.1.0`
- ⬜ [static] No stored field holds a value derivable from an id (`Reference.target` is derived when `node_id` is set); the owner of `op_id` replay (store or M2) is decided and recorded
- ⬜ [static] Every invariant names its guard from the campaign README, or "not tested" with the reason
**References**: [A2 gap analysis](../../__reports__/m1-core-foundation/00-gap_analysis_v0.md) — the rulings and the scope (no tool I/O); [stored-schema.md](../../__design__/rdm/stored-schema.md) — the types to amend; [statuses.md](../../__design__/rdm/statuses.md) — the move table `MOVES` encodes; [crates/rdm-schema/src/stored.rs](../../crates/rdm-schema/src/stored.rs) — the current types

## Step 1: Ids and node fields
**Goal**: What is identified, and how a node names its place.
**Implementation Logic**:
1. Apply the rule to each record: nodes, criteria, steps, checks, references, amendments, comments, history entries, marks. The proposed verdict is that nodes, criteria and steps keep ids, while checks and references do not and are recorded at the array level in a change. Confirm or overturn it with reasons.
2. A node gains `name`, `parent_id` and `order`. The id format is `store.md`'s; this document gives the newtypes (`NodeId`, `CriterionId`, `StepId`) and their validation.
3. Edit `stored-schema.md` to match, keeping its revision note style.
**Deliverables**: `__design__/rdm/contracts.md`: sections Id inventory, Node fields; `__design__/rdm/stored-schema.md` edited
**Consistency Checks**: `grep -q 'Id inventory' __design__/rdm/contracts.md` (expected: PASS)
**Commit**: `docs(design): specify the rdm id inventory and node fields`

## Step 2: Campaign model, moves and links
**Goal**: The shapes M2 consumes instead of designing them.
**Implementation Logic**:
1. **Campaign model:** a node table by id; children, depth and path are derived, never stored.
2. **`MOVES`:** the move table of `statuses.md` as data (from, to, allowed, who, reason, precondition, ready-set effect). The document is the oracle, so `MOVES` is tested against it.
3. **Links:** a `Links` declaration listing each id-reference field of each type, and the reverse index derived from it on load. State what a delete of a referenced node does (refused, or reported to M2).
**Deliverables**: `__design__/rdm/contracts.md`: sections Campaign model, Moves, Links
**Consistency Checks**: `grep -q 'MOVES' __design__/rdm/contracts.md` (expected: PASS)
**Commit**: `docs(design): specify the rdm campaign model, moves and links`

## Step 3: Operations and errors
**Goal**: The store-level operation signatures and the one error type every consumer maps.
**Implementation Logic**:
1. One request and result type per store primitive (create, replace with `expected_rev`, move or rename, subtree delete, comment, batch). Tool envelopes are M4's, not here.
2. The error type with at least `Stale { current }`, `Locked { holder }`, `NewerFormat { found, supported }` and `Invalid` naming the file and the rule.
3. Decide who owns `op_id` replay and record it.
**Deliverables**: `__design__/rdm/contracts.md`: sections Operations, Errors
**Consistency Checks**: `grep -q 'NewerFormat' __design__/rdm/contracts.md` (expected: PASS)
**Commit**: `docs(design): specify the rdm store operations and error type`

## Step 4: Version chain
**Goal**: Old files keep reading as the format evolves, and newer ones are never misread.
**Implementation Logic**:
1. The `FormatLayer` trait: one layer per format version, each overriding only what changed; behaviour a version lacks returns `NotInVersion`; a removed field is overridden back to that default.
2. Reading: the version is read first; an older version falls through to its layer; a newer one is refused with `NewerFormat`.
3. The write-version rule: whether an edit rewrites an old file in the current version or keeps its version.
4. Whether `FORMAT_VERSION` stays `0.1.0` (nothing is stored yet) and what bumps it from now on.
5. The guard table for the document.
**Deliverables**: `__design__/rdm/contracts.md`: sections Version chain, Invariants and guards
**Consistency Checks**: `grep -q 'Guarded by' __design__/rdm/contracts.md` (expected: PASS)
**Commit**: `docs(design): specify the rdm format version chain`
