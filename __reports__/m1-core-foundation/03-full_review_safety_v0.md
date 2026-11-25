# M1 full review, lens B (safety): false safety, sustainability, scalability, efficiency

Branch `feat/m1-core-foundation` at d9331cd. Read-only review of `crates/rdm-schema` and `crates/rdm-engine`
(src and tests), against `__design__/rdm/{store,contracts}.md`. Two mutations and two probes were run; the tree
was restored each time (`git diff --quiet` held, and a `store.proptest-regressions` file that proptest wrote
during the first mutation was deleted).

## Verdict

M1 is sound at its core. The crash, cache, lock and revision guards use oracles outside the code under test
(jq plus SHA-256, a recorded effect log replayed on copies, a cold open, a brute-force scan), and the one
guard I broke on purpose (`reverse_index_equals_brute_force_scan`) went red three times in three. There are
two things to fix before merging to main: one write rule that is documented as guarded and is not (S1), and one
documented fallback that does not exist (S2). Everything else can wait for M2 or M4.

## Findings, ranked by consequence

### S1. A step reference to a node that does not exist is accepted, and no test notices (before merge, effort S)

`crates/rdm-engine/src/write.rs:1703` (`take(s.get("references"))` in `introduced_references_exist`).
C28 says "a write never adds a defect", and the guard list names `requests_breaking_a_rule_are_refused` with a
`dangling_id` row. That row (`write.rs` test at `tests/store.rs`, the `dangling` edit) pushes onto
`l.references`, the leaf's own list. The loop over `steps[*].references` is not exercised by it.

Proof: I commented out line 1703 and ran the whole `store` test binary: 27 passed, 0 failed. Reverted.
The property test does not help either: it creates step references to `pick(b)` ids that may be gone, but it
compares the index to a scan, and a dangling link is the same in both.

Plausible bug missed: an agent writes a step whose reference names a typo'd or just-deleted node id. The write
succeeds, `Referrer` rows point at nothing, and `open` later reports it as damage on the next read.

Fix: add a row to `requests_breaking_a_rule_are_refused` (a replace that adds `steps/0/references` with an unknown
`node_id`, expect `dangling_id`), and a directory row (`references` on a directory). Mutate line 1703 to see it red.

### S2. Create-only publish has no fallback when the volume refuses hard links (before merge, effort S)

`crates/rdm-engine/src/publish.rs:35-48` against `store.md:445`. The design says: "If the volume refuses hard
links, the store falls back to 'check the name is free, then rename' under the lock". The code has no fallback:
`fs.hard_link(&tmp, target)` fails and the error returns as `Io`. On FAT or exFAT media, and on some bind or
FUSE mounts, every `create` (every new node, comment and amendment) fails, while `put` works.
The error is loud, so this is drift and portability, not silent loss.

Fix: on `ErrorKind::Unsupported` or `PermissionDenied` from `hard_link`, check `target` is absent and `rename`.
The writer lock is held, so the only race is an external writer, as the design says. Or edit `store.md:445`
to say there is no fallback and the platform is unsupported. Either way the two must agree.

### S3. The shape-walking code does not follow `LINKS`: three hand-written walkers (M2, effort M)

`links.rs:46-86` follows `LINKS` rows. Four other places hard-code field names and ignore it:
`write.rs:1534-1560` (`marks`, `preconditions`, `success_gates`), `write.rs:1683-1707` (`references`,
`steps/*/references`), `store.rs:232-262` (same marks walk, run for every node on every call) and
`write.rs:560-585` (`parent_id`, `order` of a snapshot). The guard `links_cover_every_id_field` checks `LINKS`
against the schema, so it keeps the reverse index honest, and nothing ties these walkers to `LINKS`.
The day a link field is added (a `Leaf.blocked_by`), the reverse index will find it and the dangling check of
S1 will not. All four also go through `serde_json::to_value(..).unwrap_or(Value::Null)`, so a failure there
would read as "no references, no marks" and pass.

Fix: one function `introduced_links(before, after) -> Vec<(row, pointer, target)>` over `LINKS` (reuse `follow`),
and let `introduced_references_exist` and the mark checks consume it. This also removes S1's class of bug.

### S4. An `Io` error leaves an intent record that the next writer completes, and no test injects one (M2, effort M)

`write.rs:398-404` and `apply_plan` (`write.rs:874-896`). When effect k of n fails (ENOSPC, EACCES, a Windows
sharing violation past the retry window), the caller gets `Io`, the intent record stays on disk, and the next
write or `recover()` rolls it forward. This is documented (`contracts.md:595`: "maybe a recoverable prefix") and
the crash tests cover the prefix states. What no test covers is the live path: an `Fs` that returns `Err` at the
k-th effect, then the same handle reads (it sees a prefix), writes again (recovery completes the first
operation, then the second applies), and a retry of the first request with its `op_id` is replayed. `Hooked`
(`tests/common/hooked.rs`) can fail nothing; it only hides or removes files. Also `write.rs:403` is
`let _ = self.refresh();`: keeping the first error is right, but if the refresh also fails, the handle
holds a state the next `read()` repairs only because `forget` bumped the generation. State this in a comment
or test it.

Fix: a `FailAt(k)` wrapper over `RealFs` in `tests/common`, and one test over a multi-effect op (a delete).

### S5. Replay hands out the current revision, not the first result's, and does not compare the request (M2, effort S-M; keep the limit, narrow it)

`write.rs:493-540`, `written()` at `write.rs:548`. Deliberate and stated (`contracts.md`, limit 2: "the store holds
no fingerprint of the request"). Two consequences are worth the numbers:
(a) the same node under the same `op_id` with other content is answered `replayed: true` and writes nothing.
`an_op_id_the_records_cannot_answer_is_refused_not_made_up` covers another node and another record kind, never
the same node with another payload. (b) `Written.rev` is the current `rev`. After a second actor edits the node,
a replayed first request returns a `rev` that includes the other edit, and a caller that treats it as the
result of its own write then replaces on it and passes the stale check over an edit it never saw.
Cheap partial mitigation: for a comment, compare `text` with the stored comment; for a replace, return
`Stale`-style information when the current `rev` differs from the `after` of the amendment. Not worth a
fingerprint in M1.

### S6. Unknown integers beyond 64 bits are rewritten as floats (M2, effort S)

`rdm-engine/Cargo.toml:21` and `rdm-schema/Cargo.toml:13` (`preserve_order`, `float_roundtrip`, no
`arbitrary_precision`). Probe on `canonical/campaign.json` with unknown keys added, through `decode` then `encode`:

    in : "zz_big": 18446744073709551617   out: "zz_big": 1.8446744073709552e+19
    in : "zz_neg": -9223372036854775809   out: "zz_neg": -9.223372036854776e+18
    in : "zz_exp": 1E3                    out: "zz_exp": 1000.0        (documented in store.md)
    in : "zz_dec": 0.1000000000000000055511151231257827   out: 0.1

I12 says unknown values are kept as written, and `store.md:157` documents only the `1e5` case. The proptest
(`tests/codec.rs`, `json_value`) draws `i64`, `u64` and floats within 1e15, so it cannot reach this. Any later
edit of such a node rewrites the unknown value. Fix: enable `serde_json/arbitrary_precision` (check the
`schemars` and untagged-enum interplay) or state the limit beside the `1e5` one.

### S7. `diagnostics()` serialises every node on every call (M4, effort S)

`store.rs:228-262`. `mark_diagnostics` runs `serde_json::to_value(node)` for every node of the campaign, plus a
`removed` set over all amendments, on each call. It depends only on the state, so it belongs in `build()` next
to `file_diags`. At 10k nodes that is 10k `Value` trees per call, on top of the known O(N) revalidation.
(Estimated, not measured.) Smaller O(N) per call: `Store::tree_rev()` (`store.rs:296`) rehashes all revs on each
`Receipt` and each replay; cache the `TreeRev` in `State`.

### S8. A refused recovery wedges every write and the error does not name the record (M2/M4, effort S)

`write.rs:228-236`, test `recovery_does_not_overwrite_a_create_whose_destination_holds_other_bytes`. By design
the record stays "for a person to resolve" and every write fails with `Invalid duplicate_id` naming
`nodes/<id>.json`. A server operator sees a node file, not `.rdm/intent/<ulid>.json`, which is the thing to remove.
Fix: put the record name in `detail`; M4 may want a `discard_intent` tool.

### S9. Smaller items

- `fs.rs:276` (`list_dir`): a non-UTF-8 file name is dropped without a trace; `cache.rs:140` (`listed`) drops every
  name that is not `<valid id>.json`. A node file saved as `n_0000000a.JSON`, or with a `.bak` copy beside it, is
  invisible, not reported. `store.md` should say so, or `unknown_file` should be a diagnostic. (M2, S)
- `rev.rs:31,37` and `links.rs:46`: `unwrap_or(Value::Null)` / `unwrap_or_default()` on a serialisation that cannot
  fail. If it ever did, every node would share one `rev` and the stale check would pass for all. Use `expect`, as
  `codec.rs:98` already does. (M2, S)
- `lock.rs:108`: the holder record write is best effort and a failure is swallowed (documented, `store.md:201`).
  After a `kill -9` the previous holder's `holder.json` stays, so a failed write names a dead pid. Remove stale
  holder on acquire before writing. (M4, S)
- `fs.rs:204` `sync_dir` is skipped on Windows even at `Fsync`/`Full`; stated in `store.md:337` ("stated, not
  tested"). Keep, deliberate. Opening the directory with `FILE_FLAG_BACKUP_SEMANTICS` and `FlushFileBuffers` is
  possible through `OpenOptionsExt::custom_flags` if that claim matters later.
- `publish.rs:20-26`: `put` creates `nodes/` then syncs only that directory, not the campaign root, so the first
  write's new directory is not durable. Power loss only (S17 is "not tested"). Keep, deliberate.
- `Open` on a path that does not exist succeeds with one `root_count` diagnostic, and the first lock-taking call
  (`recover`, any write) creates the whole path plus `.rdm/{lock,.gitignore,tmp}` (probe, run once). A typo in
  the root yields a stray directory tree. Fix: `Store::open` returns `Io(NotFound)` when `root` is not a directory,
  or `WriterLock::acquire` uses `create_dir` for `.rdm` and not `create_dir_all`. (before merge is not needed;
  M2, S)
- `intent.rs:97` `effect_path_is_ours` (`write.rs:1650`): on Windows a recorded `nodes/x.json:stream` is a
  `Normal` component. The records are written by this binary and `.rdm/` is ignored by git, so the exposure is
  an intent file planted by hand. Keep; note it.
- Tests: `replace_while_target_open` passes trivially on macOS and Linux (its own comment), so only the Windows
  job guards S12. Keep, deliberate. `rev_tracks_value_not_formatting` needs `jq` on the machine and panics when
  it is missing, which is loud; say so in the test or the README.
- `fs.rs` reader test spins a reader for up to 50 s; the main loop bounds at 20 s. Fine, but it is a busy loop
  on a shared machine; add a `sleep(1ms)` every 25 reads.
- Sorted `Vec` tables in `cache.rs:70-110`: `insert`/`remove` are O(N) per row, so a checkout that interleaves
  thousands of new names into a 10k table does O(N x changes) moves of large `Entry` rows. Rebuild the table when
  changes exceed a threshold. (M4, S; not measured.)

## What I checked and found sound (so nobody re-checks it)

- `links_cover_every_id_field`, `move_table_matches_statuses_doc`: oracle is the schema and the markdown table,
  not a copy. `links.rs` does not check `scope` and `on_delete` of each row: a flip of `Report` to `Keep` is
  guarded only by the delete test (C11).
- `rev_tracks_value_not_formatting`, `tree_rev_changes_on_node_ops_not_comments`: hex computed outside the code.
- `stale_replace_changes_nothing`: both rows (another handle; same-size in-place edit under a frozen stat) are
  real actors and assert the file bytes are unchanged.
- `unknown_keys_survive_in_place_at_every_level`: injection is generated from the fixtures, and the expected
  pointers are computed independently. The `required` shape list is hand-kept: a new nested struct with `extra`
  that no fixture contains will not be noticed, and `scan_*` in `codec.rs:290-370` is a hand list of the same.
- `reverse_index_equals_brute_force_scan`: mutation 1, removing the reverse-index update of a deleted node
  (`write.rs:933`), is red 3 of 3 runs, so the generator reaches deletes that succeed.
- Duplicate JSON keys are refused at every level (`codec.rs:160-215`); CRLF and a BOM are accepted and not part of
  the `rev`; the CI matrix runs Linux, macOS and Windows on stable and Linux on 1.88.
- Every guard name in `store.md` and `contracts.md` exists as a test function (28 of 28 named, listed by `fn` name).
  Rows marked "not tested" are marked, with a reason.
- `#[non_exhaustive]` is on `StoreError` and `Rule`. Other public enums in `rdm-schema` are not; the crates are
  `publish = false` and move in lock step, so keep.

## rdm-mcp (M0 spike)

Depends on neither `rdm-schema` nor `rdm-engine`; it serves its own fixture (`fixtures/roadmap.json`) and pulls in
`rmcp =3.10.0`, `tokio`, `jsonschema` and a `build.rs`. `cargo test --workspace` builds all of that on three
operating systems and the MSRV job. Nothing in it should be deleted for M1 correctness. I would delete the crate
at M1 merge (history keeps it, and M4 rewrites it over `CachedCampaign`), or, if M4 wants it as a reference,
exclude it from the CI job and say so. Decision for the owner.

## Not reached (stopped at the tool budget)

`write.rs` 1100-1500 (relocate, rename, rebalance, `check_whole`) beyond the guards that name them; `read.rs` and
`campaign.rs` in rdm-schema (only validation rules read); `crash_kill.rs` and `lock.rs` test bodies (guard exists,
oracle not read); `order.rs` midpoint arithmetic; id-space collision odds (`store.md` S16: 40-bit node ids make
"under 2e-9 per mint" true only up to about 2,200 deleted ids); clippy and a timing run at 10k nodes.
