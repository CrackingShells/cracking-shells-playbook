# rdm read path efficiency

How fast rdm reads, revalidates and processes a campaign, and the redesign that makes a call cost what it touches.
Opened when the M1 soak could not finish on Windows within 45 minutes (2026-11-26).

## Documents

### Round 00 (2026-11-26)

- [00-findings_read_path_v0.md](00-findings_read_path_v0.md) (latest): what the soak, the profile and the M1
  cache probe show, read as throughput.
- [00-open_question_read_path_v0.md](00-open_question_read_path_v0.md) (latest): the question, the constraints, four
  options and the measurement that must come first.

## Status

Open. M1 merges with this as a known limit of its implementation (Eliott, 2026-11-26). Before any redesign comes
the measurement: throughput benchmarks (MB/s, files/s, per-operation latency, reads/s) and instrumented
per-stage profiles on macOS, Linux and Windows. This supersedes the "no cost anyone will see" reading in
`__reports__/m1-core-foundation/01-cache_probe_v0.md`.
