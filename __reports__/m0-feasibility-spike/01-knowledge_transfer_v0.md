# M0 feasibility spike: Knowledge Transfer (v0)

Date: 2026-11-16

## Executive Summary

| Item | Outcome |
|:--|:--|
| Shipped | `rdm-mcp` spike server, Svelte graph view, trial notes for Desktop, VS Code and CLI, findings report |
| Result | The stack works on 3 of 3 hosts, but only with a changed data path: view data rides in the result `_meta` |
| Decisions | M6 goes inline from `_meta` (Eliott, 2026-11-16). See [00-findings_spike_v0.md](00-findings_spike_v0.md) |
| Budget | USD 30 for M0 and M0b. The counter reached USD 30.39. Eliott topped up USD 10 |

## Wins

| Win | Why it matters |
|:--|:--|
| Result-`_meta` channel, found by testing | Desktop refuses page-to-server calls. `_meta` reached the page (439 KB) and cost the model 203 characters |
| Per-host matrix | Capabilities advertised by a host proved unreliable (`serverResources`). The matrix records what works |
| Allow-rule string read from a real session | Both forms are pinned for M7 instead of guessed |

## Pain Points and Fixes

| Pain point | Fix for next time |
|:--|:--|
| A binary overwritten in place was SIGKILLed on macOS (exit 137) | Install as `rm` then `cp`, so the file gets a new inode |
| `structuredContent` failed `outputSchema` because `frozen_at` was dropped. A test chain still committed, because `grep` succeeded | Add a schema-validation test for every tool output. Gate commits on the test exit code, not on a `grep` |
| A project `.mcp.json` with the same server name masked the plugin server | Check which config wins before reading a rule string. Use a distinct name per trial |
| zsh `==` expansion broke a command | Quote such words, or avoid them in zsh |
| The CLI OAuth had expired | Check CLI login before scheduling a CLI trial |
| A user-attention gap was read as host latency. Desktop creates the page frame only when the message is shown (670,660 ms) | Separate data-to-paint from time-to-attention in every timing table |
| The planned 20-run cold and warm series was replaced, at Eliott's steer, by a channel and payload study | Fine: the study answered the real question. Confirm the replacement in the plan before the trial |
| The coordinator first dismissed Eliott's standalone viewer idea | Record a user idea as a design option with evidence for and against before dismissing it |
| Budget: the coordinator missed the per-level spend reports and the USD 24 checkpoint | Report spend at every level, and stop at the checkpoint |

## Root Causes

- Host trials came late, so host limits surfaced after the build.
- Tool outputs had no schema test, so a drift shipped.
- The budget rule existed but nothing forced the report.

## Next-Cycle Changes

| Area | Change |
|:--|:--|
| Workflow | Spend report at every level, with a checkpoint at 80% of budget |
| Testing | A schema-validation test for every tool output |
| Workflow | Set up host trials earlier, in parallel with the build |

## Artifacts to Preserve

- [00-findings_spike_v0.md](00-findings_spike_v0.md) with the steering answers
- [trials/](trials/): desktop, vscode, cli, allow-rule

## Open Questions

- M6: keep ext-apps 2.0.4 or take the 54 KB hand-written bridge (untested on any host)?
- Re-test the Desktop stream, since the capability list now shows the domain.
