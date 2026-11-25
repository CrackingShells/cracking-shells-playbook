# Cache probe: cold open against warm revalidated read, and the binary cache (v0)

Date: 2026-11-25

This report answers the `hot_cache` leaf's step 2 and the decision `store.md` §Cache (Persisted form) leaves to it: whether to build a binary cache in `.rdm/cache/`. The answer is no, for M1.

## Problem Statement

A long-lived process now serves reads from decoded entries it revalidates on every read (`CachedCampaign::read`). A binary form of the campaign (rkyv, in the untracked `.rdm/cache/`) would only be worth building if the cold path were slow enough to hurt, or if it shortened the warm one. The probe measures both at the size the design expects, 160 nodes, and at ten times that.

## Method

- **Campaign.** Built through the store's own create API (`CachedCampaign::create`, default `Fsync` tier): one root, 7 directories, the rest leaves of about 1 KB each (160 nodes, 163 KB; 1 600 nodes, 1.68 MB). Every create also writes an amendment, so the campaign directory holds as many amendment files as nodes, and a revalidation stats both directories: 320 and 3 200 files.
- **Cold open.** `Store::open` on the directory, median of 15, in-process, after the files were in the page cache. It reads, decodes and hashes every file and builds the campaign, reverse index and op index.
- **Warm read, aged.** `CachedCampaign::read`, median of 200, taken 3 s after the last write and after one read that cleared the racy flags: one `readdir` and one `metadata` per file (it follows symlinks), nothing decoded.
- **Warm read, all racy.** A fresh copy of the campaign (every file written within the last instant), opened and read 9 times within the 2 s window: every entry is read and hashed again each time, none is decoded because the bytes are unchanged.
- **Single-node read.** `read()` followed by `campaign().get(id)`.
- **Raw floor.** `readdir` plus `fs::metadata` of every file of `nodes/`, `_amendments/` and `_comments/`, no cache, median of 200.
- **Build.** `cargo test --release`, a throw-away test file in the crate, deleted afterwards. Four runs, minimum and median of each series.
- **Machine.** Apple M3 Pro, 12 cores, macOS 26.6.2 (25G83), APFS. The machine was shared with other agents during every run (load average 20 to 108 on 12 cores). Minimums are the figures to read; medians carry the scheduling noise, and some runs are inverted by it (see the findings).

## Findings

Minimum of the series over four runs (median range in brackets):

| Measure | 160 nodes (320 files) | 1 600 nodes (3 200 files) |
|:--|--:|--:|
| Cold open | 12.6 to 17.8 ms (14 to 55 ms) | 135 to 144 ms (140 to 148 ms) |
| Warm read, aged | 1.34 to 1.41 ms (1.4 to 1.7 ms) | 15.1 to 15.6 ms (17 to 18 ms) |
| Warm read, all racy | 2.7 to 6.1 ms (3.2 to 6.2 ms) | 10.8 to 12.7 ms (11 to 17 ms) |
| `read()` plus `get` of one node | 1.31 to 1.49 ms | 15.1 to 17.2 ms |
| `get` of one node, no revalidation | 20 to 30 ns | 22 ns |
| Raw `readdir` plus `metadata` of every file | 1.18 to 1.22 ms | 13.1 to 13.7 ms |

- **A warm read costs about a tenth of a cold open** at both sizes, and it is within 10 to 15 % of the raw `readdir` plus `metadata` floor. The cache adds almost nothing to the syscalls it cannot avoid without trusting the directory (`store.md`: an in-place write does not change the directory mtime).
- **The overhead the first version of this probe could not explain was the amendment directory.** The first floor counted `nodes/` only, and every revalidation also lists and stats `_amendments/` (one file per create). With the floor over the three directories the gap closes. The walk of the second version (one ordered merge of the sorted listing and the sorted entries, ids validated without allocating, one clock read per pass) did not change the warm read measurably: it was syscall-bound already.
- **The racy re-read is visible at 160 nodes and not resolvable at 1 600.** Right after writes, a read re-reads and re-hashes every entry the writes touched: 2 to 5 times an aged read at 160 nodes (2.7 to 6.1 ms against 1.3 to 1.4 ms). At 1 600 nodes the all-racy minimum (10.8 to 12.7 ms) is below the aged one (15.1 to 15.6 ms), which no code path explains, so the difference between the two lies inside the noise of a shared machine. The earlier figure in `store.md` (36 ms all racy against 12 ms aged at 1 600) came from the first version of this probe, which ran under a load average of 36 to 51; it is not reproduced. The cost lasts `RACY_WINDOW` after the last change of each file and is paid by the few files a tool call wrote, not by the campaign.
- **A single node costs the same as the whole revalidation**: the lookup is nanoseconds, the revalidation is the price of any read.
- **The design's earlier figure** (0.25 ms for 160 files from Python) was not reproduced from Rust: the raw floor is 0.6 ms for the 160 node files and 1.2 ms with the amendments. It is still small next to a tool call.
- **A write reads the nodes it touches from the disk**, not from the cache: a few files per request, plus the siblings a rebalance renumbers, found when the plan names them. This costs one stat, read and decode per touched file and is not in the figures above.

## Decision

**No binary cache in M1.**

- A binary cache only shortens the cold open (decode and hash), which a server pays once at start. 13 to 18 ms at 160 nodes is not a cost anyone will see; even 1 600 nodes start in about 150 ms.
- It does nothing for the warm read, which is the path every tool call takes and which is bound by one `metadata` call per file, because a persisted cache must be validated by the same signatures.
- It would add a second state to keep equal to the files, with its own header, format version and discard rules, for a gain below what the probe can resolve.

Revisit it if a campaign of several thousand nodes has to start inside a request. The first move is then to measure again on an idle machine, then to fix the warm path (the per-file work), not to persist the cold one.
