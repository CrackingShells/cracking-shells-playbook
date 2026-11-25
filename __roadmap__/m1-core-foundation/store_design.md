# Store design

**Goal**: `__design__/rdm/store.md` specifies how a campaign lives on disk and how it is read, written, recovered and cached, naming each technique with a verified source and binding every invariant to the guard that will hold it.
**Pre-conditions**:
- [ ] Amendment A2 is on the branch (`8b4a4e7`) and the A2 gap analysis is read
- [ ] `__design__/rdm/stored-schema.md` and `read-model.md` are read
**Success Gates**:
- ✅ [static] `store.md` covers the layout, id format and minting, the write algorithm with one write-order row per store primitive (create, replace, move or rename, subtree delete, comment, batch), recovery, the fsync policy, the lock and its holder record, `rev` and `tree_rev`, cache revalidation and Windows rename semantics
- ✅ [static] Each technique names its source, checked by web research: Soft Updates (McKusick & Ganger, 1999), Maildir delivery, ARIES redo (Mohan et al., 1992), Pillai et al. OSDI 2014 (ALICE), git's racy-git note, the Win32 `MoveFileEx` and `ReplaceFile` documentation. A claim the source does not support is dropped or marked as this design's own
- ✅ [static] Every invariant row has a "Guarded by" cell naming a guard from the campaign README (for example `every_write_prefix_loads_and_recovers_to_pre_or_post`), or "not tested" with the reason
- ✅ [static] The four probes assigned here (lock crate, Windows rename, directory mtime, fsync cost) have their answer and decision recorded in `store.md`; no probe code is committed
**References**: [A2 gap analysis](../../__reports__/m1-core-foundation/00-gap_analysis_v0.md) — the rulings; [R01 §Concurrency](../../__design__/rdm/architecture.md) — the lock and journal sketch this replaces; [read-model.md](../../__design__/rdm/read-model.md) — what `rev` and `tree_rev` promise consumers

## Step 1: Probes and sources
**Goal**: The questions the design depends on are answered before it is written.
**Implementation Logic**:
1. **Sources:** dispatch a web-research subagent (Sonnet) to read each source named in the gates and report what it supports, with links. Record the links in a Sources section.
2. **Probes,** in the scratchpad, never in the repository:
   - `fs4` and `fd-lock` built with `cargo +1.88`: does a second process see the lock, and is it released after `kill -9` of the holder?
   - On APFS: does a rename into `nodes/` change the directory mtime, and does an in-place write of an existing file? At what granularity?
   - The cost of `fsync` on a file and on its directory, and of `F_FULLFSYNC`.
   - From the Win32 documentation: when does `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING` fail on a target another program holds open, and with which error?
3. Write a Probe answers section: question, answer, decision. Delete the scratch code.
**Deliverables**: `__design__/rdm/store.md`: sections Sources, Probe answers
**Consistency Checks**: `grep -q '^## Probe answers' __design__/rdm/store.md && test -z "$(git status --porcelain -- crates)"` (expected: PASS)
**Commit**: `docs(design): record the rdm store probes and sources`

## Step 2: Layout, ids and revisions
**Goal**: What a campaign is on disk, and how its versions are named.
**Implementation Logic**:
1. **Layout:** `nodes/<id>.json` (one file per node, holding `name`, `parent_id` and the order key), `_amendments/`, `_comments/`, and `.rdm/` holding `lock`, `intent/` and `cache/` (untracked). Temp files are named so the loader never reads them.
2. **Sibling order:** a fractional-indexing key, so inserting between two siblings rewrites only the inserted node. State how a key is generated between two keys and how a run of inserts is kept short.
3. **Ids:** a kind prefix, then short base32 (the alphabet, the length and why that length is enough at the expected campaign size). Minted from an injected RNG, retried on collision, never a counter (counters collide across worktrees).
4. **Revisions:** `rev` is a hash of the node's canonical encoding (so a whitespace-only edit keeps it), and `tree_rev` a hash over the sorted `(id, rev)` pairs of all nodes. Choose the hash and say what is counted. A comment never changes a `rev`.
**Deliverables**: `__design__/rdm/store.md`: sections Layout, Order key, Ids, Revisions
**Consistency Checks**: `grep -q '^## Revisions' __design__/rdm/store.md` (expected: PASS)
**Commit**: `docs(design): specify the rdm flat layout, ids and revisions`

## Step 3: Writes, recovery and the lock
**Goal**: No kill leaves a campaign that fails to load or loses an acknowledged write.
**Implementation Logic**:
1. **Readers take no lock.** Each node file is replaced whole (temp, fsync, rename: Maildir delivery), so a reader sees the old or the new file, never a torn one.
2. **Writers are serialised** by one advisory lock on `.rdm/lock` (crate chosen by the probe), with a holder record (pid, host, start time) that is advisory only. `Locked { holder }` is returned after a timeout.
3. **Ordering:** apply the three soft-updates rules to each primitive. Write the table: for each primitive, the ordered list of file effects and what a reader sees after each prefix.
4. **Intent:** a multi-file operation writes a redo-only intent record first. The amendment serves as the intent where the operation writes one. Recovery runs under the lock on acquisition: it redoes an incomplete intent to completion and removes completed ones and stray temp files.
5. **fsync policy:** what a process kill guarantees and what power loss does not, and why the chosen policy fits a git-tracked store.
**Deliverables**: `__design__/rdm/store.md`: sections Reads, Lock, Write order (table), Intent and recovery, Durability
**Consistency Checks**: `grep -q '^## Write order' __design__/rdm/store.md` (expected: PASS)
**Commit**: `docs(design): specify rdm write ordering, recovery and the writer lock`

## Step 4: Cache revalidation and Windows
**Goal**: A cached read returns what a cold read would at that moment, and the design holds on Windows.
**Implementation Logic**:
1. **Revalidation:** a directory `stat` as the fast path, a per-file `(mtime, size)` check where the probe says the directory mtime misses a change, and git's racily-clean rule: an entry whose mtime is not older than the snapshot is re-read. Optional `notify` only as an accelerator, never as the only signal.
2. **Write-through:** a write returns only after the files are written; the cache is updated from the written value.
3. **Windows:** rename semantics, the sharing-violation retry from the probe, and long paths.
4. **Guard table:** every invariant of the document with its "Guarded by" cell.
**Deliverables**: `__design__/rdm/store.md`: sections Cache, Windows, Invariants and guards
**Consistency Checks**: `grep -q 'cached_read_equals_cold_read' __design__/rdm/store.md` (expected: PASS)
**Commit**: `docs(design): specify the rdm cache revalidation and windows semantics`
