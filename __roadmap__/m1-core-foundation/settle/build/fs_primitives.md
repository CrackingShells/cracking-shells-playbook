# Filesystem primitives

**Goal**: A new `rdm-engine` crate provides atomic file replacement, the writer lock with its holder record, and the intent record, written against a small filesystem trait so `crash_test` can record every effect, and each primitive survives real processes.
**Pre-conditions**:
- [ ] `../r01_sync.md` is done
**Success Gates**:
- ⬜ [run] `cargo test -p rdm-engine --test fs reader_sees_old_or_new_never_partial` passes: a reader in another process never sees a torn or empty file while `atomic_replace` runs
- ⬜ [run] `cargo test -p rdm-engine --test lock` passes, with `second_writer_gets_locked_with_holder`, `lock_survives_holder_kill` (the holder is killed with SIGKILL or `TerminateProcess`, then the lock is acquired) and `garbage_holder_record_does_not_break_locking`
- ⬜ [run] `cargo test -p rdm-engine --test fs replace_while_target_open` passes: a replace succeeds while another handle holds the target open for reading
- ⬜ [run] `cargo +1.88 check -p rdm-engine` passes
- ⬜ [static] The primitives go through a `Fs` trait with a real implementation and a recording one that logs each effect in order (write temp, fsync, rename, remove, create directory), so `crash_test` needs no change to the primitives
- ⬜ [run] `! cargo test --workspace -- --list 2>/dev/null | grep confirm_` passes, and the report names, for each guard, the bug that turned it red (for example a truncate-then-write, or a pid-file lock with `O_EXCL`)
**References**: [store.md](../../../../__design__/rdm/store.md) — Reads, Lock, Intent and recovery, Windows, and the chosen lock crate

## Step 1: Crate, filesystem trait and atomic replace
**Goal**: A node file is always replaced whole.
**Implementation Logic**:
1. Create `crates/rdm-engine` (workspace edition and `rust-version`, `publish = false`, depends on `rdm-schema`).
2. Define the `Fs` trait over the effects the store needs, with `RealFs` and `RecordingFs`.
3. Write the two guards first, each spawning the test binary itself as the child process. See them red against a naive in-place write.
4. Implement `atomic_replace`: temp file in `.rdm/tmp/` (`store.md` §Layout; same file system as the target), write, fsync, rename, directory fsync per the fsync policy, and the Windows sharing-violation retry from `store.md`. On macOS the `Fsync` tier calls `fsync(2)` through `rustix`, because std's `sync_all` issues `F_FULLFSYNC` there (`store.md` §Durability).
5. Add `.rdm/` and `proptest-regressions/` to `.gitignore`.
**Deliverables**: `crates/rdm-engine/Cargo.toml`; `crates/rdm-engine/src/lib.rs`; `crates/rdm-engine/src/fs.rs`: trait `Fs`, types `RealFs`, `RecordingFs`, `Effect`, fn `atomic_replace`; `crates/rdm-engine/tests/fs.rs`: tests `reader_sees_old_or_new_never_partial`, `replace_while_target_open`; `.gitignore`; `Cargo.lock`
**Consistency Checks**: `cargo test -p rdm-engine --test fs` (expected: PASS)
**Commit**: `feat(rdm-engine): add atomic file replacement over a recordable filesystem`

## Step 2: Writer lock
**Goal**: Writers are serialised across processes, and a killed holder never leaves the campaign locked.
**Implementation Logic**:
1. Write the three lock guards first, with real child processes, and see them red.
2. Implement `WriterLock::acquire(root, timeout, holder_record: &[u8])` with `fs4`, as `store.md` §Lock says: the lock file `.rdm/lock` stays empty, and the caller's holder bytes replace `.rdm/holder.json` through `atomic_replace` after the lock is held, and are removed before release. The record is informational: the lock state never depends on it.
3. On timeout return `LockError::Busy { holder: Option<Vec<u8>> }`, the holder file's bytes as read (`None` when absent). The primitives stay generic over bytes: `LockHolder` and `StoreError::Locked` are `rdm-schema`'s contract types, and `campaign_store` parses the bytes into them (an unparsable record is an unknown holder). `rdm-engine` defines no contract type.
**Deliverables**: `crates/rdm-engine/src/lock.rs`: types `WriterLock`, `LockError`, fn `acquire`; `crates/rdm-engine/tests/lock.rs`: the three guards
**Consistency Checks**: `cargo test -p rdm-engine --test lock` (expected: PASS)
**Commit**: `feat(rdm-engine): add the writer lock with a holder record`

## Step 3: Intent record
**Goal**: A multi-file operation can be redone after a kill.
**Implementation Logic**:
Implement the intent record of `store.md` (begin, complete, list pending) on the `Fs` trait. It gets confirms only: write-then-read is set-then-get, and its real behaviour (visible after a kill, redone by recovery) is guarded by `crash_test`.
**Deliverables**: `crates/rdm-engine/src/intent.rs`: type `Intent`, fns `begin`, `complete`, `pending`
**Consistency Checks**: `cargo test -p rdm-engine` (expected: PASS)
**Commit**: `feat(rdm-engine): add the redo intent record`
