# rdm read path: findings (v0)

Date: 2026-11-26

---
type: findings
topic: rdm_read_path_efficiency
date: 2026-11-26
version: v0
prior-version: none
key-metric: cold-open throughput: 12.4 MB/s (prior: N/A, delta: N/A)
decision-required: intervene
---

## Headline Result

metric: campaign bytes a cold `open` processes per second (1 600 nodes, 1.68 MB, release build, macOS)
value: 12.4
unit: MB/s
prior: N/A (first measurement framed as throughput)
direction: new

The store moves campaign data at about 12 MB/s cold, and every warm read pays one `stat` per file of the
campaign, history included. The M1 soak could not finish on Windows within 45 minutes. Eliott warned during the
architecture setup that reading JSON files would be expensive; the coordinator argued it would not matter and
never measured the claim against agent throughput. This report records what the numbers show.

## Results Tables

### Soak run 45621003595 at 813e42f (`PROPTEST_CASES=10000`, `RDM_SOAK_ITERS=1000`)

| Job | `store` test binary | Kill harness | Job |
|:--|--:|--:|:--|
| ubuntu-latest | 702 s | 95 s | success |
| macos-latest | 1 127 s | 139 s | success |
| windows-latest | not finished at the cap | 100 s | cancelled at `timeout-minutes: 45` |

In the `store` binary, `reverse_index_equals_brute_force_scan` takes nearly all the time: 240 cases take 17.4 s
locally, and the other 28 tests take about 1 s.

### Where one proptest case spends its time (macOS `sample`, debug build, 8 s)

| Part | Share of the test thread | What it is |
|:--|--:|:--|
| Test helper: one `store.read()` per candidate node | about 40 % | each read revalidates every file (test defect, fixed in a73b07b) |
| Cold `open` after each step | about 17 % | the oracle "a cold build agrees" |
| The six store writes | about 13 % | intent, publish, amendment, directory syncs |
| Brute-force scan and `held()` reads | about 9 % | the oracle, plus a revalidation |
| Set-up, tear-down, proptest | about 20 % | temporary campaigns created and removed per case |
| **System calls, after the helper fix** | **80 %** | `open` alone 28 %; decode and hash under 20 % |

### Experiments (240 cases, local; each was reverted except the first)

| Change | Time | Per case |
|:--|--:|--:|
| As committed at 6bf8b92 | 17.4 s | 73 ms |
| Helper reads once per step (a73b07b) | 10.9 s | 45 ms |
| plus `RACY_WINDOW` cut from 2 s to 20 ms | 10.1 s | 42 ms |
| plus dependencies built at `opt-level = 2` | 9.8 s | 41 ms |

### The M1 cache probe, read as throughput (`01-cache_probe_v0.md`, release, macOS)

| Measure | 160 nodes, 320 files, 163 KB | 1 600 nodes, 3 200 files, 1.68 MB |
|:--|--:|--:|
| Cold open | 12.6 ms: 12.9 MB/s, 39 µs per file | 135 ms: 12.4 MB/s, 42 µs per file |
| Warm read (aged) | 1.34 ms: 4.2 µs per file | 15.1 ms: 4.7 µs per file |
| Reads per second for the whole server (one read mutex) | about 750 | about 66 |

## Observations

| Signal | Baseline / Expected | Observed [source] | Interpretation |
|:--|:--|:--|:--|
| Warm read cost | Bounded by what a call needs (one node: nanoseconds) | One `stat` per file of the campaign, nodes, comments and every amendment [source: `cache.rs:237` `sync_kind`, `01-cache_probe_v0.md`] | Read cost grows with campaign size and with history. History only grows: every write adds an amendment. |
| Amendment revalidation | Amendments and comments are append-only, never edited (`store.md` §Layout) | Stat-ed, and re-read when racy, on every read [source: profile, `load_file` under `CachedCampaign::read`] | Wasted work that grows with history. |
| Server throughput | Many agents in parallel against one campaign | `read` takes `&mut self` behind one mutex [source: `cache.rs:600-611`]: 66 reads/s at 1 600 nodes | Agents queue behind each other; per-read cost becomes the fleet's rate limit. |
| Cold throughput | Data processing at disk or parser speed | 12.4 MB/s, flat across 10x size [source: probe] | Per-file overhead (`open`, `stat`, `read`, `close`, hash, decode) dominates, not bytes. Not yet split by stage. |
| `rev` computation | One hash over the canonical bytes | Profile shows `rev` via `to_value` then `to_string_pretty` of the `Value` [source: sample, `rev::rev` → `codec::json_text`] | Encodes each record twice to hash it; to be measured in isolation. |
| Windows | Same algorithm, same order of time | The open-heavy test exceeds the cap; the process-heavy kill harness runs at Linux speed (100 s vs 95 s) [source: CI logs] | Per-file `open` is costlier on Windows (NTFS, real-time scanning of temporary directories), so per-file algorithms hurt most there. |

## Charts & Visualizations

Test job duration at soak counts, per OS (seconds, build included; the Windows bar is a lower bound, cut by the
45-minute cap with the `store` binary unfinished):

```
ubuntu   ████████████████▉                         896
macOS    ██████████████████████████▋              1407
windows  ███████████████████████████████████████▶ >2 700
         0        500       1000      1500      2000      2500   s
```

Warm read cost against campaign files (release, macOS, minimum of four runs): linear, about 4.5 µs per file.

```
ms
15 |                                             ● 1 600 nodes (3 200 files): 15.1 ms
   |
 1 |  ● 160 nodes (320 files): 1.34 ms
   +----------------------------------------------- files stat-ed per read
      320                                      3 200
```

## Contradictions & Surprises

- The probe that should have caught this did measure it: 15 ms per read at 1 600 nodes, linear in files. Its
  decision weighed that against "a tool call" for one agent, not against a fleet of agents behind one mutex, nor
  against history growth, and it ran on a machine at load average 20 to 108 on 12 cores.
- The racy window (2 s) and the debug build together explain under 10 % of the time. The cost is per-file system
  calls, not parsing.
- A single-node read costs the same as revalidating the whole campaign.

## Steering Questions

- [now] Merge M1 with the Windows soak recorded as a known limit of this implementation (Eliott, 2026-11-26), and
  open a dedicated efficiency milestone before M2 builds on the read path?
- [now] Is the metric set right: MB/s for cold load and decode, files/s for revalidation, latency per operation
  kind, reads/s for the shared handle, each at 160, 1 600 and 16 000 nodes and at 0x and 10x history, on all
  three OS?
- [next run] Instrumented profiling of the hot paths (spans per stage: list, stat, read, hash, decode, build, index)
  on an idle machine and on the three CI runners, before choosing any redesign.
- [later] Does history stay in per-write amendment files, or does its storage form change (it is tracked text that
  git merges, so this is a format question)?

## Pointers

- `__reports__/rdm_read_path_efficiency/00-open_question_read_path_v0.md`
- `__reports__/m1-core-foundation/01-cache_probe_v0.md`
- `__reports__/m1-core-foundation/02-sustainability_review_src_v0.md` (S1, S2, S13)
- https://github.com/CrackingShells/cracking-shells-playbook/actions/runs/45621003595
- `crates/rdm-engine/src/cache.rs` (`sync_kind`, `load_file`, `RACY_WINDOW`, `CachedCampaign::read`)
- commit a73b07b (the test helper fix)
