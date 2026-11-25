# Hot cache

**Goal**: A long-lived process keeps a campaign in memory, writes through to the files, and revalidates every read so that a cached read equals a cold read at that moment (I10).
**Pre-conditions**:
- [ ] `../campaign_store.md` is done
**Success Gates**:
- ⬜ [run] `cargo test -p rdm-engine --test cache cached_read_equals_cold_read` passes, with one row per real external change: a store write from another process; an editor writing in place at the same size within one mtime tick; `git checkout` replacing many files; a file added; a file deleted. The oracle is a fresh `Campaign::open`, and its diagnostics (`unknown_key` and merge damage) and op index must equal the warm ones. Write step 3 runs on a revalidation made after the lock is taken, or `op_id` replay breaks (store.md §Cache)
- ⬜ [run] `cargo test -p rdm-engine --test cache write_through_visible_to_cold_reader_on_return` passes
- ⬜ [static] The guards were seen red: the first against revalidation by directory mtime alone, the second against a buffered write
- ⬜ [static] The cold vs warm probe is answered in `__reports__/m1-core-foundation/01-cache_probe_v0.md` (numbers, machine, decision on a binary cache), and its bench code is deleted
- ⬜ [run] `! cargo test --workspace -- --list 2>/dev/null | grep confirm_` passes
**References**: [store.md](../../../../../../../__design__/rdm/store.md) — Cache (revalidation, racily-clean rule, write-through)

## Step 1: Revalidated cache
**Goal**: Reads come from memory and are never stale.
**Implementation Logic**:
Write both guards first, producing each external change with the real actor (a second process using the store, an in-place `write` at the same length, a `git checkout` in a temp repository). Implement the cache over `Campaign` as `store.md` specifies: directory stat, per-file `(mtime, size)`, the racily-clean re-read, write-through.
**Deliverables**: `crates/rdm-engine/src/cache.rs`: type `CachedCampaign`, fns `open`, `read`, and the write operations delegating to `Campaign`; `crates/rdm-engine/tests/cache.rs`: the two guards
**Consistency Checks**: `cargo test -p rdm-engine --test cache` (expected: PASS)
**Commit**: `feat(rdm-engine): serve a campaign from a revalidated cache`

## Step 2: Cold vs warm probe
**Goal**: Decide whether a binary cache is worth building.
**Implementation Logic**:
In the scratchpad, build a 160-node campaign through the store's create API and time a cold open, a warm revalidated read and a single-node read. Write the numbers and the decision in a short findings report (writing-reports). Delete the bench.
**Deliverables**: `__reports__/m1-core-foundation/01-cache_probe_v0.md`
**Consistency Checks**: `test -f __reports__/m1-core-foundation/01-cache_probe_v0.md` (expected: PASS)
**Commit**: `docs(reports): record the rdm cache probe`
