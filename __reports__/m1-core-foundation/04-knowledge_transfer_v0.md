# M1 core foundation: knowledge transfer, v0

M1 froze the contracts in `rdm-schema` and built the store in `rdm-engine`, on `feat/m1-core-foundation`:
- flat `nodes/<id>.json` files;
- a canonical codec and a version chain that refuses newer files;
- ordered writes with a redo-only intent record and recovery under the lock;
- a revalidated hot cache behind one public handle, `CachedCampaign`.

Every gate in the campaign README holds. The three-OS matrix, MSRV 1.88, lint and the soak are green at 825ea5d
(CI evidence in `02-adversarial_review_v0.md`). The Windows soak passed only after a test-helper fix, in 33 minutes
against a 45-minute cap; the cost behind that margin is the store's read path, which Eliott ruled on 2026-11-26 is
redesigned in its own PR (`__reports__/rdm_read_path_efficiency/`). The milestone is not yet on
`main`: landing it is Step 2 of the M1 leaf, and `FORMAT_VERSION` stays `0.1.0` until then (ruling R5).

## What worked

- **Independent oracles.** jq plus SHA-256 for `rev`, a fresh cold `open` for the cache, a brute-force scan for the
  reverse index, the prefix-or-full state for crashes, and `statuses.md` parsed for `MOVES`. Every guard that was
  mutated went red.
- **Depth as ordering.** Siblings ran in parallel worktrees, and each level merged `--no-ff` after a fresh verifier.
- **Open reviews late.** The two full-review lenses (harmony, safety) found real defects the mutation passes could
  not: unchecked step references, a fallback the design promised and the code lacked, and four silent serialisation
  fallbacks.

## What hurt, and the change for M2

| Pain | Cost | Change |
|:--|:--|:--|
| Verifiers added a test row per surviving mutant | about 500 lines of cache tests, culled later | Triage survivors by real actor and class before briefing; a mutant is never a row |
| Agents re-read big files and dumped cargo output | up to 300k tokens per leaf | Fresh agent per round; reads by line range; filtered cargo output; batched calls; a tool-call cap in every brief |
| A temp worktree built into the shared target dir | rdm-schema tests ran with stale `CARGO_MANIFEST_DIR` paths | A worktree at another path gets its own target, or `cargo clean -p <crate>` afterwards |
| Windows only seen at push time | three red rounds: backslash snapshot keys, jq CRLF output, autocrlf on committed schemas | `--no-fail-fast` on CI; `.gitattributes` `eol=lf` on every committed JSON that a test compares byte for byte |
| Helpers forked per test file | each cross-cutting fix had to be made 3-4 times | One `tests/common` from the start of M2 |
| Eliott's warning that reading JSON files would be expensive was answered by argument, and the cache probe judged 15 ms per read at 1 600 nodes against one tool call, on a machine at load 20-108 | the soak could not finish on Windows until a test-helper fix, and still takes 33 of its 45 minutes there; the read path needs its own redesign PR | A performance claim is a measurement: throughput (MB/s, files/s, reads/s) and a per-stage profile, at 10x the expected size and with history, on an idle machine and the three CI runners, judged against a fleet of agents |

## Carried forward

### To M2 (graph engine)

| Id | Item | Source |
|:--|:--|:--|
| S1 | Whole-campaign clone and O(N) plan and derive per write | `02-sustainability_review_src_v0.md` |
| S9 | ULIDs round-trip through `String` | same |
| S10 | `write.rs` (about 1700 lines) to split by concern | same |
| S12 | "Move" means a status transition and a reparent (`Move`, `move_node`, `op_relocate`) | same, H14 |
| H6 | `Version` is a tuple alias with free functions; one newtype | `03-full_review_harmony_v0.md` |
| H11 | Three hand-written newtype shapes (about 90 duplicated lines) | same |
| H12, H13 | Ownership in signatures, `# Errors` sections, small idioms | same |
| sS3 | Four JSON walkers do not follow `LINKS`; one `introduced_links` | `03-full_review_safety_v0.md` |
| sS4 | No test injects an `Io` error mid-apply; `let _ = self.refresh()` at write.rs | same |
| sS5 | Replay returns the current `rev` and never compares the request payload | same |
| sS6 | Unknown integers beyond 64 bits are rewritten as floats (breaks I12) | same |
| sS9 | Non-`<id>.json` names silently ignored; stale `holder.json` after a kill | same |
| T8 | One red row hides the rest in multi-row tests | `02-sustainability_review_tests_v0.md` |
| T9 | `crash_prefix` copies the whole tree per prefix | same |
| `op_id` | M2 chooses the `op_id`, sets `History.op_id`, and owns the directory `uncancel` lookup; the store owns replay (R4) | `00-gap_analysis_v0.md` §4 |
| Nits | `OpResult` is `#[non_exhaustive]` but mirrors the closed `Op`; `debug_assert!` on the decode at write.rs:905; the hard-link fallback replaces a dangling symlink | fix-round verifier |
| `RecordPath` | The logical-path newtype | sustainability round A |

### To M4 (MCP server)

| Id | Item | Source |
|:--|:--|:--|
| S2 | O(N) revalidation per read, history included, single read mutex: moved to the read-path redesign PR, before M2 | `02-sustainability_review_src_v0.md`, `__reports__/rdm_read_path_efficiency/` |
| S13 | Synchronous API and sleep-poll lock in an async server | same |
| sS7 | `diagnostics()` serialises every node per call; `tree_rev` rehashed per receipt | `03-full_review_safety_v0.md` |
| sS8 | A refused recovery wedges writes; a `discard_intent` tool may be wanted (the error now names the intent file) | same |
| F8 (F-d) | Typed ids in the read model | earlier design review, `settle` |
| `rdm-mcp` | The M0 spike was deleted at M1 (6bf8b92); its `rmcp` wiring and Svelte bridge are at 58a4a30 | M4 and M6 leaves |
| `rmcp` pin | Moves from M0 to M4 with the crate | meta README ownership line |

## Owner decisions taken during M1

Recorded in the leaf gates and `__design__/rdm/{store,contracts}.md`: R1-R6 and F-c "refuse all" (Eliott).
The coordinator decided these within those rulings, and Eliott has not yet confirmed them:
- the root-family exemption;
- a rename of a sound root is refused;
- `op_id_reused`;
- the `Current` change;
- the cycle-member delete rule;
- the reading of the expected slug.
