# M0 feasibility spike: Findings (v0)

Date: 2026-11-16

---
type: findings
topic: m0-feasibility-spike
date: 2026-11-16
version: v0
prior-version: none
key-metric: data-to-paint (Desktop, 160 nodes, 438,707 B): 18 ms (prior: N/A, delta: N/A)
decision-required: confirm
---

<!-- Step 1 content: the timing section. Steps below are added in the next commit. -->

## Timing

Trial notes: [desktop.md](trials/desktop.md), [vscode.md](trials/vscode.md). Every figure is one sample, from the run named in its row. There are no medians and no confidence intervals.

> **DEVIATION from the leaf.** The leaf asked for `timing.py` over a `probe.jsonl`, with a 20-run cold and warm series and bootstrap CIs. Neither exists.
> - Claude Code Desktop does not let the page call the server ([desktop.md](trials/desktop.md), Trial 1: `IPC not available for MCP tool call`). No stamp could be recorded server-side. Probes were read off an on-screen panel.
> - At Eliott's steer, the spike measured data channels and payload sizes instead of a repeated series.
> - The single samples still answer the question. The number that matters is data-to-paint, and it is tens of ms in every run. The large figures are host-side delays that follow user attention: Desktop creates the page frame only when the message is on screen ([desktop.md](trials/desktop.md), Trial 4 scale 10: 670,660 ms, Eliott scrolled back after about 11 minutes). No repetition would shrink a delay that depends on the reader. No `timing.py` was written.

### Data to paint, per host and channel (one sample each)

| Host | Channel | Payload [source] | Data reached page | First paint | Complete |
|:--|:--|:--|:--|:--|:--|
| Desktop | `_meta`, scale 1 | 43,715 B, 16 nodes [desktop.md, Trial 4] | 318 ms after server stamp | 371 ms | 372 ms |
| Desktop | `_meta`, scale 10 | 438,707 B, 160 nodes [desktop.md, Trial 4 scale 10] | 18 ms from data to first paint | same as data | same as first paint |
| VS Code | `result` x1 | 43,715 B, 16 nodes [vscode.md, Trial 2] | 24 ms | 41 ms | 41 ms |
| VS Code | `chunks` x100 | 4,391,237 B, 1,600 nodes, 65 `resources/read` [vscode.md, Trial 2] | 39 ms | 64 ms | 597 ms |
| VS Code | `stream` x100 (SSE) | 4,391,237 B, 1,600 nodes [vscode.md, Trial 2] | 28 ms | 70 ms | 157 ms |
| VS Code | `_meta` x1 | 43,715 B, 16 nodes [vscode.md, Trial 2] | 23 ms | 39 ms | 40 ms |

The Desktop scale-1 and scale-10 rows are not on the same clock. Scale 1 is server stamp to paint. Scale 10 is data arrival to first paint, because the server-to-page leg was lost to the user's delay (see next table).

### Host-side delays that are not display time (one sample each)

| Host | Run | Delay | Cause [source] |
|:--|:--|:--|:--|
| Desktop | `result` scale 1, Trial 2 | 46,970 ms | Eliott's copy delay. The panel showed the latest probe stamp, not first paint. Eliott saw the page paint at once [desktop.md, Trial 2] |
| Desktop | `_meta` scale 10, Trial 4 | 670,660 ms | Frame created when the message was on screen; Eliott scrolled back after about 11 min [desktop.md, Trial 4] |
| VS Code | `result` scale 1, Trial 1 | 73,679 ms | Unexplained: chat turn, trust prompt or off-screen page. Did not recur; later results arrived in 18 to 24 ms [vscode.md, Trial 1, Trial 2] |
| Desktop | `result` scale 1, Trial 3 | 2.3 s | Tool round trip, call to result in the transcript [desktop.md, Trial 3] |

### Why one sample is enough here

| Question | Answer from the data |
|:--|:--|
| Is display slow at the sizes M6 needs? | No. 438,707 B paints 18 ms after arrival on Desktop; 4,391,237 B starts painting at 64 to 70 ms on VS Code |
| Does noise hide a difference that matters? | The spread between the fast runs is tens of ms. The delays that look large are one to three orders of magnitude bigger and come from attention, not from the host or the page |
| What would a series add? | Tighter CIs on tens of ms. It would not change a go or no-go |
