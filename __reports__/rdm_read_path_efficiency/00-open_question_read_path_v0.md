# rdm read path: open question (v0)

Date: 2026-11-26

---
type: open-question
question: How must rdm store, read, revalidate and process campaign data so an agent's call costs what it touches, not the whole campaign and its history?
domain: code
date-opened: 2026-11-26
status: open
spawned-from: __reports__/rdm_read_path_efficiency/00-findings_read_path_v0.md (M1 soak cancelled on Windows)
closure-type: both
---

## The Question

How should rdm read, revalidate and process campaign data so that a call's cost follows what it touches, not campaign size and history?

## Handoff Contract

- Receiving agent is asked to: build the measurement first (throughput benchmarks and instrumented profiles of
  every hot path, on three OS), produce the numbers, then an architecture report that chooses a design from them.
- Receiving agent must NOT: change the stored format, the store's guarantees (I3, I4, I10, I12) or any public
  signature before Eliott approves the architecture report; nor tune constants (racy window, opt-level) as a fix.

## Context

What M1 built and what it costs (release build, macOS, from `01-cache_probe_v0.md` and the findings report):

| Path | Algorithm today | Cost grows with | Measured |
|:--|:--|:--|:--|
| Cold `open` | list three directories; `open`, `read`, SHA-256 and decode every file; build campaign, reverse index, op index | files | 12.4 MB/s; 42 µs per file |
| Warm `read` (every tool call) | list three directories; `stat` every file; re-read and hash files younger than 2 s | files, history included | 4.5 µs per file: 15 ms at 1 600 nodes |
| Single-node read | the full warm read, then a map lookup | files | same as warm read; lookup 22 ns |
| Write | refresh, re-read touched nodes, clone whole campaign and plan (S1), intent, publish, amendment, directory syncs | nodes (clone and plan), touched files (I/O) | about 1 ms of syncs per edit; the rest unmeasured |
| Shared handle | `read(&mut self)` behind one mutex (S2, S13) | concurrent agents | 66 reads/s at 1 600 nodes for the whole server |
| `rev` | `to_value`, then pretty-print the `Value`, then SHA-256 | record size | not isolated yet |

What is fixed and must hold for any design:

| Constraint | Source |
|:--|:--|
| Tracked campaign data stays text that git merges; a binary form may only be an untracked cache | `00-gap_analysis_v0.md`, plan rulings 2026-11-25 |
| A cached read equals a cold read for every external change class (I10), including in-place same-size edits within one mtime tick | `__design__/rdm/store.md` §Cache |
| Amendments and comments are append-only, never edited; a new write adds a new file | `__design__/rdm/store.md` §Layout (line 31), `stored-schema.md` (line 77) |
| MSRV 1.88; macOS, Linux and Windows | `rust.yml` |
| Many agents share one machine and one campaign | `~/.claude/CLAUDE.md`, "You are never alone" |

## Option Landscape

### Option A — Revalidate immutable records by name only
- Description: amendments (and any other append-only record) are checked by the set of names a directory lists, not by a `stat` per file.
- Benefit: warm read cost stops growing with history.
- Cost: an in-place edit of an amendment is no longer seen; I10's wording narrows to "every change a real actor makes".
- Unknown: whether any real actor rewrites an amendment in place (a merge, a rebase, a hand fix).

### Option B — Change notifications with the stat walk as fallback
- Description: a watcher (FSEvents, inotify, ReadDirectoryChangesW) marks changed paths; a read revalidates only those.
- Benefit: warm read cost follows what changed, not what exists.
- Cost: three OS back-ends, event overflow and lost-event handling, a dependency, and a test harness for each.
- Unknown: whether events arrive reliably for `git checkout` bursts and for network or synced folders.

### Option C — A cheaper unit of storage for history
- Description: history stored as fewer, larger text files (for example one append-only file per period) instead of one file per write.
- Benefit: fewer files to list, open and hash, on every path and every OS.
- Cost: a format change (version layer), and appends that git must merge on every branch.
- Unknown: how git merges concurrent appends to the same history file from two worktrees.

### Option D — Faster per-file processing
- Description: one pass per file (read, hash the bytes, decode typed without an intermediate `Value`), `rev` from bytes already canonical.
- Benefit: raises MB/s on cold open and on every re-read, with no format or guarantee change.
- Cost: touches the codec and `rev`, which the drift and jq-hex oracles guard.
- Unknown: the share of the 42 µs per file that is decode and hash rather than system calls.

## Comparison

| Option | Reach (which paths gain) | Format change | Guarantee change | Per-OS work | Cost to learn more |
|:--|:--|:--|:--|:--|:--|
| A | warm read | no | I10 wording | none | low: count real actors that edit amendments |
| B | warm read, write refresh | no | none if the fallback is exact | high | medium: prototype on three runners |
| C | all paths | yes | none | none | medium: merge trials in two worktrees |
| D | cold open, re-reads, `rev` | no | none | none | low: the stage-split profile |

*What this table does not settle:* the size of each gain. Without a per-stage profile (list, stat, open, read,
hash, decode, build, index) and throughput figures per OS, the options cannot be ranked, and they are not
exclusive.

## What Would Close This Question

[Both]: a benchmark suite that reports MB/s, files/s, per-operation latency and reads/s at 160, 1 600 and 16 000
nodes, with 0x and 10x history, on macOS, Linux and Windows. Also an instrumented per-stage profile of each hot
path. Then an architecture report choosing the design from those numbers, approved by Eliott. The question closes
when the soak runs on all three OS within the job's existing cap.

## Next Step to Reduce Uncertainty

Build the measurement: a benchmark crate or target with byte and file throughput, and span-level instrumentation
of `open`, `read`, `write` and `rev`, run once on an idle machine and on the three CI runners.
