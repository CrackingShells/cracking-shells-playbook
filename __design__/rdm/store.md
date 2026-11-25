# rdm — Store

How a campaign lives on disk and how it is read, written, recovered and cached. It replaces
R01's `§Concurrency` (lock, staging, journal) and the "names are paths" layout of `§Data Model`
(`architecture.md`), under the rulings of the A2 gap analysis (2026-11-25).

What this document owns: the on-disk layout, the id string format and minting, the order key, `rev`
and `tree_rev`, the writer lock and its holder record, the write order of each primitive, the
intent and recovery, durability, cache revalidation and Windows. What it does not own, and only
references: the Rust types, the id newtypes and the record inventory, the operation requests and
results, the error type, declared links, `MOVES` and the version chain, all in
[contracts.md](contracts.md). Errors are named by their variant (`Stale { current }`,
`Locked { holder }`, `NewerFormat { found, supported }`, `Invalid`). The stored fields are in
[stored-schema.md](stored-schema.md), the meaning of `rev` and `tree_rev` for consumers in
[read-model.md](read-model.md).

Rulings this document builds on and does not reopen: flat `nodes/<id>.json` holding `name`,
`parent_id` and an order key, depth computed from `parent_id`; readers take no lock; Rust types are
the contract and JSON is derived; a file newer than the binary is refused with `NewerFormat`;
declared links; tracked data stays text; the cache is write-through, revalidated on every read,
never the source of truth.

Where a claim is this design's own and not a source's, it says so. Guard names are those of the
[campaign README](../../__roadmap__/m1-core-foundation/README.md).

## Probe answers

Four probes were assigned to this leaf. Probe code lived in a scratch directory and is deleted;
nothing of it is committed. Machine: macOS 26 (Darwin 25.6), APFS, local `cargo` stable 1.99 and
`cargo +1.88`.

| Question | Answer | Decision |
|:--|:--|:--|
| Lock crate: do `fs4` and `fd-lock` build on 1.88, does a second process see the lock, is it released by `kill -9` of the holder? | `fs4` 1.1.0 (declares MSRV 1.75) and `fd-lock` 4.0.4 both build with `cargo +1.88`. With both, a second process got `BUSY` while the holder lived and acquired the lock right after `kill -9` of the holder (the lock file stays on disk, unlocked). A second `open` of the lock file inside the holder's own process also got `BUSY` (per-open-file-description `flock`, not per-process) | `fs4`. Same semantics as `fd-lock`, a plain `lock`/`try_lock` on the `File` (the names std adopted in 1.89, so dropping the crate later is mechanical), and no guard that borrows the lock owner. std's own `File::lock` is not usable at the 1.88 MSRV |
| Does a rename into `nodes/` change the directory mtime; does an in-place write of an existing file? | Create, rename to a new name, rename over an existing name, and unlink each changed the directory mtime. An in-place write (`r+`, or truncate and rewrite) did not (delta 0), though it changed the file's own mtime. A rename over an existing file also changes its inode, and its mtime even at equal size | The directory mtime cannot certify "nothing changed": an editor writing in place is missed. It is not a fast path. Revalidation is one `readdir` plus a `stat` per entry (see Cache) |
| Granularity of the mtimes? | Nanoseconds: 300 file writes gave 300 distinct `mtime_ns`, none a multiple of 1 000; consecutive renames into one directory gave distinct directory mtimes (156 µs and 277 µs apart) | APFS needs no coarse-timestamp care, but the cache does not rely on it: a conservative racy window (see Cache) covers FAT (2 s), HFS+ (1 s) and the coarse kernel clock of ext4 |
| Cost of `fsync` on a file, on its directory, and `F_FULLFSYNC`? (300 iterations, 1.5 KB file, write + rename) | No fsync: median 0.148 ms, p95 0.255. `fsync` file: 0.157 / 0.222. `fsync` file and directory: 0.157 / 0.289. `F_FULLFSYNC` file: 4.99 / 6.04. `F_FULLFSYNC` file and directory: 9.97 / 10.98. On macOS plain `fsync` does not flush the drive's cache, which is why it is as cheap as no fsync | Default tier `Fsync` (file and directory); `Full` is an opt-in (see Durability). `F_FULLFSYNC` costs about 65 times a plain write and would turn a 160-node import into 1.6 s of waiting |
| What does a revalidation scan cost? (extra probe, feeds the cache decision) | `scandir` plus `stat` of every entry: median 0.25 ms (p95 0.35) for 160 files, 2.7 ms (p95 3.4) for 2 000 files, measured from Python, so an upper bound for Rust. Stat of the directory alone: 1 µs | A full scan on every read is affordable at the expected size. No fast path is built, and `notify` is not adopted in M1 |
| Which hash for `rev`: do `sha2` and `blake3` build on 1.88? | `sha2` 0.11.1 and `blake3` 1.8.7 both build. `blake3` declares MSRV 1.88, exactly the floor, and its default build compiles C and assembly (`cc`) | SHA-256 (see Revisions) |
| Win32 `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING`, and `ReplaceFile`, against a target another program holds open without `FILE_SHARE_DELETE`: which error, when? | **The Microsoft pages do not say.** Neither page names `ERROR_ACCESS_DENIED` (5) or `ERROR_SHARING_VIOLATION` (32) for this case, neither promises atomicity, and `MoveFileEx` says nothing about `FILE_SHARE_DELETE`. What the pages do say: `MoveFileEx` fails if the new name is an existing directory; `ReplaceFile` opens the replaced file with read, delete and synchronize access and share mode read, write and delete, and the replacement with no sharing; `ReplaceFile`'s `REPLACEFILE_WRITE_THROUGH` is documented as not supported; a failed `ReplaceFile` can leave the replaced file renamed or absent (error 1175, 1176, 1177). Rust's `OpenOptions` opens with `FILE_SHARE_READ`, `WRITE` and `DELETE` by default, so this program's own readers never block a replace. Rust's `fs::rename` calls `MoveFileExW(REPLACE_EXISTING)` from Rust 1.86 and falls back to a POSIX-semantics `SetFileInformationByHandle` only on `ERROR_ACCESS_DENIED` (1.85 used the POSIX path first) | The replace is `std::fs::rename`, not `ReplaceFile`: its failure states are not documented as atomic and it leaves the replaced file absent in some of them. Retry on 5 and 32 (the two codes a held-open target is reported with in practice, which is this design's own claim) with a bounded back-off, then fail. Numbers under Windows. `replace_while_target_open` settles the codes on a real Windows runner |

### What the probes did not settle

- Linux and Windows lock release on kill: only macOS was run. `lock_survives_holder_kill` runs on all three operating systems in CI.
- The Windows error code of a held-open target and whether `std::fs::rename` is atomic there: from documentation only, as the table says. `replace_while_target_open` and `killed_workers_lose_no_acknowledged_write` on the Windows runner are the evidence.
- Power loss: no probe can show it on this machine. See Durability.

## Sources

Each source was read, by a research subagent for the PDFs and pages, and by this author for the
Rust `OpenOptionsExt` page. The third column is what the source supports; a claim it does not
support is dropped or marked as this design's own wherever it would have appeared.

| Technique | Source read | What it supports | What it does not, and what this design does with that |
|:--|:--|:--|:--|
| Ordering of dependent writes | [McKusick and Ganger, "Soft Updates", USENIX ATC 1999](https://www.usenix.org/legacy/publications/library/proceedings/usenix99/full_papers/mckusick/mckusick.pdf), §2 | Three rules: never point to a structure before it is initialised; never re-use a resource before nullifying every earlier pointer to it; never reset the old pointer to a live resource before the new one is set (its example is a rename: do not drop the old name before the new one is written) | Soft Updates is about a file system's own metadata and a per-block dependency tracker. This design borrows the rules for the order of files, not the tracker |
| Publish by write-then-link | [Bernstein, Maildir](https://cr.yp.to/proto/maildir.html) and [maildir(5)](https://manpages.debian.org/testing/qmail/maildir.5.en.html) | Write the message under `tmp/`, then `link()` it into `new/`; a reader never sees a partial message; unique names built from time, process id, host and a counter need no lock; the man page requires `write`, `fsync`, `close` each checked before the `link()`; `tmp/`, `new/`, `cur/` on one file system | Bernstein's page gives the rationale and the name scheme but not the delivery steps; those and the fsync are in maildir(5). Maildir publishes with `link()`, not `rename()`. This design uses `link()` for create-only files (it fails when the name exists, which a rename does not) and `rename()` for replacement, which Maildir never does |
| Redo from a durable log | [Mohan et al., ARIES, ACM TODS 17(1), 1992](https://web.stanford.edu/class/cs345d-01/rl/aries.pdf) | Write-ahead logging; the redo pass repeats history; compensation log records are redo-only | Redo idempotence by page LSN: only partly confirmed from the OCR copy read. A "redo-only intent record that recovery rolls forward" is not a construct of ARIES. This design's intent is its own application of log-before-data, with whole-file after-images, so redo is idempotent by construction (putting the same bytes twice) and needs no LSN |
| What a crash can expose | [Pillai et al., "All File Systems Are Not Created Equal", OSDI 2014](https://www.usenix.org/system/files/conference/osdi14/osdi14-paper-pillai.pdf) | Applications assume rename is atomic; most of the vulnerabilities it found sit in missing `fsync` of a parent directory or of data before a rename; whether an append persists before a later rename differs between file systems and even between configurations of one; a single-sector write is not atomic on every file system | The claim "an unsynced temp file renamed into place can leave a zero-length file" was not found in those words and is not cited. This design fsyncs the temp file before the rename and the directory after it because the paper shows the behaviour varies, not because it measured that outcome |
| Racily clean entries | [git, racy-git](https://git-scm.com/docs/racy-git) | An entry whose mtime is not older than the index's own timestamp may have changed within one timestamp tick and must be compared by content; git ignores the nanosecond fields by default; coarse file systems stay problematic | Git also zeroes the cached size so a later stat check fails. This design has no index file to timestamp, so it keeps a per-entry flag and re-reads flagged entries (see Cache): an adaptation, not git's mechanism |
| Atomic replace on Windows | [`MoveFileExW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw), [`ReplaceFileW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew), [`std::fs::rename`](https://doc.rust-lang.org/std/fs/fn.rename.html), [`OpenOptionsExt::share_mode`](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html) | See the probe answer above | Neither Microsoft page promises atomicity or names the error for a held-open target. Atomic replace on NTFS is therefore an observed property this design relies on and tests (`killed_workers_lose_no_acknowledged_write`, `replace_while_target_open`), not a documented guarantee |
| Fractional indexing | [Figma, "Realtime editing of ordered sequences"](https://www.figma.com/blog/realtime-editing-of-ordered-sequences/), [rocicorp/fractional-indexing](https://github.com/rocicorp/fractional-indexing) (Greenspan's design; the article was unreachable, the README states it is based on it) | A key between two keys is a string midpoint; keys compare with plain lexicographic order over an ascending digit alphabet (base 62 by default in rocicorp, base 95 in Figma's post); Figma lists growth of the key length as a known drawback and not a practical concern; `generateNKeysBetween` is advised for bulk inserts | Neither source names a rebalancing step, and jitter appears only as advice to reduce concurrent collisions. The rebalance threshold and the choice of tie-break instead of jitter are this design's own |
| Advisory locks | [`fs4` 1.1.0](https://docs.rs/fs4/1.1.0/), [`fd-lock` 4.0.4](https://docs.rs/fd-lock/4.0.4/) | Crate behaviour as probed above | Not a literature source. The kernel's release of a `flock` on process death is shown by the probe on macOS only |
