# Crash test

**Goal**: Every prefix of every store operation's writes loads and recovers to the state before or after it, and real processes killed at random lose no acknowledged write, on three operating systems (I3).
**Pre-conditions**:
- [ ] `../campaign_store.md` is done
**Success Gates**:
- ✅ [run] `cargo test -p rdm-engine --test crash_prefix every_write_prefix_loads_and_recovers_to_pre_or_post` passes: for each store primitive, each prefix of the effects `RecordingFs` logged is applied to a copy, then opened and recovered, and equals the full state or the absent state; one row also cuts recovery's own effects and recovers again (store.md S18)
- ✅ [run] `cargo test -p rdm-engine --test crash_kill killed_workers_lose_no_acknowledged_write` passes with the default short, seeded run; with `RDM_SOAK_ITERS` set it runs the soak
- ✅ [static] The two guards were seen red: the prefix guard with node files removed before the amendment is written, the kill guard with the lock released between the revision check and the write
- ✅ [run] `! cargo test --workspace -- --list 2>/dev/null | grep confirm_` passes
- ✅ [run] The verifier's fix round added a check that no intent or temp file is left after the last effect, facts of each row's result derived from plain JSON and a `parent_id` walk, `recovery_does_not_overwrite_a_create_whose_destination_holds_other_bytes`, and one random stream per kill worker; a re-check saw the two mutants that had stayed green turn red. The lock released after the check is red on about one default run in three and reliably only under the soak. Deletes are ordered by depth along the on-disk parent chain (the prefix guard's finding, `store.md` delete row). The three-OS run is the harden gate
**References**: [store.md](../../../../../../../__design__/rdm/store.md) — Write order, Intent and recovery, Durability; Pillai et al., OSDI 2014 — the prefix-replay method

## Step 1: Every prefix loads
**Goal**: Ordering bugs are caught deterministically.
**Implementation Logic**:
Run each primitive once on `RecordingFs` to get its ordered effects. For each prefix length, replay that prefix on a copy of the starting campaign with `RealFs`, then `open` and `recover`. Compare with the states from running the operation fully and not at all, computed separately. The test is deterministic and belongs to the must-run set.
**Deliverables**: `crates/rdm-engine/tests/crash_prefix.rs`: test `every_write_prefix_loads_and_recovers_to_pre_or_post`
**Consistency Checks**: `cargo test -p rdm-engine --test crash_prefix` (expected: PASS)
**Commit**: `test(rdm-engine): guard that every write prefix loads and recovers`

## Step 2: Killed workers
**Goal**: Concurrency bugs are caught with real processes.
**Implementation Logic**:
The test re-executes its own binary as N workers (selected by an environment variable). Each runs random seeded operations on one campaign and appends an acknowledgement to a side file after each operation returns. The parent kills workers at random (`Child::kill`: SIGKILL on Unix, `TerminateProcess` on Windows) and restarts them. At the end, every acknowledged write is present and the campaign loads. The default run takes seconds; `RDM_SOAK_ITERS` raises it for the CI soak.
**Deliverables**: `crates/rdm-engine/tests/crash_kill.rs`: test `killed_workers_lose_no_acknowledged_write`
**Consistency Checks**: `cargo test -p rdm-engine --test crash_kill` (expected: PASS)
**Commit**: `test(rdm-engine): guard that killed writers lose no acknowledged write`
