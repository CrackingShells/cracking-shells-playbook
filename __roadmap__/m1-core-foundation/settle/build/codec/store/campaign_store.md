# Campaign store

**Goal**: `rdm_engine::store::Campaign` opens and validates a campaign, mints ids, computes revisions, applies every store primitive through ordered writes under the writer lock, keeps a reverse link index, and recovers on lock acquisition.
**Pre-conditions**:
- [ ] `../codec_and_versions.md` is done
**Success Gates**:
- ⬜ [run] `cargo test -p rdm-engine --test store stale_replace_changes_nothing` passes: after a refused stale replace, the node bytes, the amendments and `tree_rev` are unchanged (I4)
- ⬜ [run] `cargo test -p rdm-engine --test store move_writes_one_file_and_keeps_ids` passes: a move or rename changes one file under `nodes/` and no id; the descendants' files are byte-identical while their derived paths change (I7)
- ⬜ [run] `cargo test -p rdm-engine --test store rev_tracks_value_not_formatting` and `tree_rev_changes_on_node_ops_not_comments` pass
- ⬜ [run] `cargo test -p rdm-engine --test store minting_retries_on_collision` passes, with an injected RNG that first returns an existing id
- ⬜ [run] `cargo test -p rdm-engine --test store open_names_invalid_campaigns` passes, with one row per class a real actor produces: a dangling `parent_id`, a `parent_id` cycle (two branches each moving one node under the other), a file name that differs from its inner id, and a leftover temp file (ignored, not an error)
- ⬜ [run] `cargo test -p rdm-engine --test store reverse_index_equals_brute_force_scan` passes: a proptest over random operation sequences against a scan of the declared link fields
- ⬜ [run] `! cargo test --workspace -- --list 2>/dev/null | grep confirm_` passes, and the report names, for each guard, the bug that turned it red
**References**: [store.md](../../../../../../__design__/rdm/store.md) — Layout, Ids, Revisions, Write order, Intent and recovery; [contracts.md](../../../../../../__design__/rdm/contracts.md) — Campaign model, Operations, Errors, Links

## Step 1: Open, validate and mint
**Goal**: A campaign on disk becomes a validated model.
**Implementation Logic**:
1. Write `open_names_invalid_campaigns` and `minting_retries_on_collision` first; produce the invalid campaigns with the store's own writes followed by the real actor's change (for the cycle, two git branches merged).
2. Implement `Campaign::open`: read `nodes/`, decode through the codec, derive children, depth and path, build the reverse index, and refuse with `Invalid` naming the file and the rule.
3. Implement `mint_id` over an injected RNG.
**Deliverables**: `crates/rdm-engine/src/store.rs`: type `Campaign`, fn `open`; `crates/rdm-engine/src/ids.rs`: fn `mint_id`; `crates/rdm-engine/tests/store.rs`: the two guards
**Consistency Checks**: `cargo test -p rdm-engine --test store open_names_invalid_campaigns` (expected: PASS)
**Commit**: `feat(rdm-engine): open and validate a flat campaign`

## Step 2: Revisions and replace
**Goal**: A stale write is refused and changes nothing.
**Implementation Logic**:
Write the three revision guards first. Compute `rev` from the canonical encoding and `tree_rev` from the sorted `(id, rev)` pairs, as `store.md` says. `replace` takes the lock, re-reads the node from disk, compares `expected_rev`, then writes.
**Deliverables**: `crates/rdm-engine/src/rev.rs`: fns `rev`, `tree_rev`; fn `Campaign::replace`; the three guards
**Consistency Checks**: `cargo test -p rdm-engine --test store rev` (expected: PASS)
**Commit**: `feat(rdm-engine): check revisions on replace`

## Step 3: Structural primitives and the reverse index
**Goal**: Create, move, delete, comment and batch keep every link resolvable.
**Implementation Logic**:
1. Write `move_writes_one_file_and_keeps_ids` and `reverse_index_equals_brute_force_scan` first.
2. Implement `create`, `move_node`, `delete_subtree` (amendment with its snapshot first, then the node files bottom-up), `comment` and `batch` (through the intent record), each in the order of the write-order table of `store.md`. Update the reverse index on every change.
**Deliverables**: fns `Campaign::create`, `move_node`, `delete_subtree`, `comment`, `batch`; `crates/rdm-engine/src/links.rs`: type `ReverseIndex`; the two guards
**Consistency Checks**: `cargo test -p rdm-engine --test store` (expected: PASS)
**Commit**: `feat(rdm-engine): apply the store primitives through ordered writes`

## Step 4: Recovery
**Goal**: An interrupted operation is completed by the next writer.
**Implementation Logic**:
Implement recovery on lock acquisition, as `store.md` specifies: redo incomplete intents, remove completed ones and stray temp files. Use confirms only: the behaviour is observable only under interruption, and `harden/crash_test.md` guards it.
**Deliverables**: fn `Campaign::recover`
**Consistency Checks**: `cargo test -p rdm-engine` (expected: PASS)
**Commit**: `feat(rdm-engine): recover interrupted operations on lock acquisition`
