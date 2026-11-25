# Campaign store

**Goal**: `rdm_engine::store::Store` opens a campaign from disk into the pure `rdm_schema::Campaign` model, validates it, mints ids, computes revisions, applies every store primitive through ordered writes under the writer lock, keeps a reverse link index, and recovers on lock acquisition.
**Pre-conditions**:
- [ ] `../codec_and_versions.md` is done
**Success Gates**:
- ✅ [run] `cargo test -p rdm-engine --test store stale_replace_changes_nothing` passes: after a refused stale replace, the node bytes, the amendments and `tree_rev` are unchanged (I4)
- ✅ [run] `cargo test -p rdm-engine --test store move_writes_one_file_and_keeps_ids` passes: a move or rename changes one file under `nodes/` (the amendment is written besides) and no id; the descendants' files are byte-identical while their derived paths change (I7)
- ✅ [run] `cargo test -p rdm-engine --test store rev_tracks_value_not_formatting` and `tree_rev_changes_on_node_ops_not_comments` pass
- ✅ [run] `cargo test -p rdm-engine --test store minting_retries_on_collision` passes, with an injected RNG that first returns an existing id
- ✅ [run] `cargo test -p rdm-engine --test store open_names_invalid_campaigns` passes, with one row per class a real actor produces. Per-file faults refuse `open`: a file name that differs from its inner id, a sub-item duplicate, an editor adding `target` beside `node_id`. Cross-file merge damage loads and is reported, one diagnostic naming file and rule, while the sound files load: a dangling `parent_id`, a `parent_id` cycle (two branches each moving one node under the other), a name collision, a root whose name is not the slug (every node unvalidated). A leftover temp file is ignored, not an error
- ✅ [run] `cargo test -p rdm-engine --test store reverse_index_equals_brute_force_scan` passes: a proptest over random operation sequences against a scan of the declared link fields; the same sequences compare children, depth and path with a rebuild
- ✅ [run] `cargo test -p rdm-engine --test store requests_breaking_a_rule_are_refused` passes, with rows `note_required`, `empty_edit`, a replace changing `parent_id` or `name` (`use_relocate`), create under or replace of an unvalidated node (`unvalidated_node`), and the repairs accepted: relocate into the tree, rename of a colliding sibling or of the root, delete
- ✅ [run] `cargo test -p rdm-engine --test store replayed_op_id_writes_nothing`, `failed_batch_writes_nothing` and `delete_reports_referrers_and_touches_only_the_subtree` pass
- ✅ [run] `cargo test -p rdm-engine --test store order_keys_sort_strictly_between_their_bounds` passes: a proptest over random bounds
- ✅ [run] The verifier's fix round added a cycle-member delete row (every file of the cycle and its subtree removed), rows refusing a rename of a sound root, `an_op_id_the_records_cannot_answer_is_refused_not_made_up` (`op_id_reused`), `an_intent_naming_a_file_outside_the_records_is_removed_not_redone`, `a_write_completes_an_interrupted_operation_first_and_keeps_the_report`, `an_intent_newer_than_this_binary_refuses_recovery_and_every_write`, `a_garbage_holder_record_gives_locked_with_no_holder_at_the_store`, `diagnostics_are_sorted_across_their_producers`, `every_write_stamps_the_current_format_version`, independent `tree_rev` and `SubtreeRev` values, and step references, batches and renames in the reverse-index generator; a re-check saw each of the nine mutants that had stayed green turn red
- ✅ [run] `! cargo test --workspace -- --list 2>/dev/null | grep confirm_` passes, and the report names, for each guard, the bug that turned it red
**References**: [store.md](../../../../../../__design__/rdm/store.md) — Layout, Ids, Revisions, Write order, Intent and recovery; [contracts.md](../../../../../../__design__/rdm/contracts.md) — Campaign model, Operations, Errors, Links

## Step 1: Open, validate and mint
**Goal**: A campaign on disk becomes a validated model.
**Implementation Logic**:
1. Write `open_names_invalid_campaigns` and `minting_retries_on_collision` first; produce the invalid campaigns with the store's own writes followed by the real actor's change (for the cycle, two git branches merged).
2. Implement `rdm_schema::Campaign::open` and its derivations (`parent`, `children`, `depth`, `path`, `resolve`, `subtree`, `insert`, `remove_subtree`) in `rdm-schema`, pure and without I/O, exactly as `contracts.md` §Campaign model gives them; `open` becomes the only constructor and the interim `Campaign::from_parts` of `schema_contracts` is removed. Then `Store::open` in the engine reads `nodes/`, decodes through the codec, calls `Campaign::open`, builds the reverse index, and refuses per-file faults with `Invalid` naming the file and the rule. The per-node rules neither serde nor the codec enforces (`root_shape`, `reference_target_derived`, `sub_item_id_unique`, and the per-file rules of `stored-schema.md` such as `goal_line`) are checked here. The codec's diagnostics come in walk order; `open` sorts them.
3. Implement `mint_id` over an injected RNG.
**Deliverables**: `crates/rdm-schema/src/campaign.rs`: fn `Campaign::open` and the derivations; `crates/rdm-engine/src/store.rs`: type `Store`, fn `open`; `crates/rdm-engine/src/ids.rs`: fn `mint_id`; `crates/rdm-engine/tests/store.rs`: the two guards
**Consistency Checks**: `cargo test -p rdm-engine --test store open_names_invalid_campaigns` (expected: PASS)
**Commit**: `feat(rdm-engine): open and validate a flat campaign`

## Step 2: Revisions and replace
**Goal**: A stale write is refused and changes nothing.
**Implementation Logic**:
Write the three revision guards first. Compute `rev` from the canonical encoding and `tree_rev` from the sorted `(id, rev)` pairs, as `store.md` says. `replace` takes the lock, replays a known `op_id` from the op index, re-reads the node from disk, compares `expected_rev`, applies the request rules (`diff` decides whether an amendment is needed), then writes, setting `schema_version` to `FORMAT_VERSION` before `encode` (the codec stamps nothing). Write `requests_breaking_a_rule_are_refused` and `replayed_op_id_writes_nothing` first.
**Deliverables**: `crates/rdm-engine/src/rev.rs`: fns `rev`, `tree_rev`, `subtree_rev`; `crates/rdm-schema/src/diff.rs`: fn `diff` (`contracts.md` §The amendment rule, pure); fn `Store::replace`; the op index; the five guards
**Consistency Checks**: `cargo test -p rdm-engine --test store rev` (expected: PASS)
**Commit**: `feat(rdm-engine): check revisions on replace`

## Step 3: Structural primitives and the reverse index
**Goal**: Create, move, delete, comment and batch keep every link resolvable.
**Implementation Logic**:
1. Write `move_writes_one_file_and_keeps_ids`, `reverse_index_equals_brute_force_scan`, `failed_batch_writes_nothing`, `delete_reports_referrers_and_touches_only_the_subtree` and `order_keys_sort_strictly_between_their_bounds` first.
2. Implement `create`, `move_node`, `delete_subtree` (amendment with its snapshot first, then the node files bottom-up), `comment` and `batch` (through the intent record), each in the order of the write-order table of `store.md`. Update the reverse index on every change.
**Deliverables**: fns `Store::create`, `move_node`, `delete_subtree`, `comment`, `batch`; `crates/rdm-engine/src/links.rs`: type `ReverseIndex`, and type `Record` (a borrowed node, comment or amendment, which `contracts.md` names in `replace_record` without defining); the five guards
**Consistency Checks**: `cargo test -p rdm-engine --test store` (expected: PASS)
**Commit**: `feat(rdm-engine): apply the store primitives through ordered writes`

## Step 4: Recovery
**Goal**: An interrupted operation is completed by the next writer.
**Implementation Logic**:
Implement recovery on lock acquisition, as `store.md` specifies: redo incomplete intents, remove completed ones and stray temp files. Use confirms only: the behaviour is observable only under interruption, and `harden/crash_test.md` guards it.
**Deliverables**: fn `Store::recover`
**Consistency Checks**: `cargo test -p rdm-engine` (expected: PASS)
**Commit**: `feat(rdm-engine): recover interrupted operations on lock acquisition`
