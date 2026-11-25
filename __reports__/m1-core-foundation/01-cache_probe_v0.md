# Cache probe: cold open against warm revalidated read, and the binary cache (v0)

Date: 2026-11-25

This report answers the `hot_cache` leaf's step 2 and the decision `store.md` §Cache (Persisted form) leaves to it: whether to build a binary cache in `.rdm/cache/`. The answer is no, for M1.

## Problem Statement

A long-lived process now serves reads from decoded entries it revalidates on every read (`CachedCampaign::read`). A binary form of the campaign (rkyv, in the untracked `.rdm/cache/`) would only be worth building if the cold path were slow enough to hurt, or if it shortened the warm one. The probe measures both at the size the design expects, 160 nodes, and at ten times that.

## Method

- **Campaign.** Built through the store's own create API (`CachedCampaign::create`, default `Fsync` tier): one root, 7 directories, the rest leaves of about 1 KB each (160 nodes, 163 KB; 1 600 nodes, 1.68 MB). Nodes are spread over the 7 directories.
- **Cold open.** `Store::open` on the directory, median of 30, in-process, after the files were in the page cache. It reads, decodes and hashes every file and builds the campaign, reverse index and op index.
- **Warm revalidated read.** `CachedCampaign::read`, median of 200, taken 3 s after the last write and after one read that cleared the racy flags, so every entry is clean: one `readdir`, one `stat` per file, nothing decoded.
- **Single-node read.** `read()` followed by `campaign().get(id)`, median of 200; and `get` alone, median of 200 batches of 1 000.
- **Raw floor.** `readdir` plus `fs::metadata` of each file, no cache, median of 200.
- **Two more cases.** A read right after writes (every entry racy, so every file is read and decoded again), and a read after one file was changed in place.
- **Build.** `cargo test --release`, a throw-away test file in the crate, deleted afterwards.
- **Machine.** Apple M3 Pro, 12 cores, macOS 26.6.2 (25G83), APFS. The machine was shared with other agents during every run: load average 36 to 51 on 12 cores. The numbers are upper bounds and noisy; the ranges below are across runs, not confidence intervals.

## Findings

| Measure | 160 nodes | 1 600 nodes |
|:--|--:|--:|
| Cold open | 16 to 30 ms (5 runs) | 240 to 390 ms |
| Warm revalidated read, all entries clean | 1.9 to 2.8 ms | 21 to 29 ms |
| `read()` plus `get` of one node | 2.1 to 3.3 ms | 21 to 25 ms |
| `get` of one node, no revalidation | 20 to 30 ns | 22 ns |
| Raw `readdir` plus `stat` of every file | 0.6 to 1.2 ms | 5.2 to 5.9 ms |
| Read right after writes (all entries racy) | 8.6 to 19 ms | 29 ms (partial: only the last-written files were racy) |
| Read after one file changed in place | 4.1 to 6.3 ms | 39 to 50 ms |

- **A warm read costs 7 to 15 times less than a cold open** at both sizes, and it is dominated by the `stat` loop, which no cache can avoid without trusting the directory (`store.md`: an in-place write does not change the directory mtime).
- **Above the floor**, the warm read spends about 1 ms at 160 nodes and about 15 ms at 1 600 nodes that the syscalls do not explain. The probe did not locate it. With the machine this loaded, it may be scheduling and not code. It is the one number worth measuring again on an idle machine before M2 depends on it.
- **A single node costs the same as the whole revalidation**: the lookup is nanoseconds, the revalidation is the price of any read.
- **The racy window is visible.** Within 2 s of a write, a read decodes every file the write touched again. In the probe that is every file, because the campaign was just built; in use it is the few files a tool call wrote.
- **The design's earlier figure** (0.25 ms for 160 files from Python) was not reproduced from Rust under this load; the raw floor here is 0.6 to 1.2 ms. It is still small next to a tool call.

## Decision

**No binary cache in M1.**

- A binary cache only shortens the cold open (decode and hash), which a server pays once at start. 16 to 30 ms at 160 nodes is not a cost anyone will see; even 1 600 nodes starts in under half a second.
- It does nothing for the warm read, which is the path every tool call takes and which is bound by one `stat` per file, because a persisted cache must be validated by the same signatures.
- It would add a second state to keep equal to the files, with its own header, format version and discard rules, for a gain below what the probe can resolve.

Revisit it if a campaign of several thousand nodes has to start inside a request, or if the warm overhead above the floor turns out to be the code and not the load. In both cases the first move is to measure again on an idle machine, then to fix the warm path (the per-file work), not to persist the cold one.
