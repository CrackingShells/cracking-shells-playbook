# M1 Core foundation

## Context
Milestone M1 of the rdm meta-roadmap (`__roadmap__/roadmap-mcp-rework/core/m1_core_foundation.md`), rescoped by amendment A2. It freezes the contracts every later layer consumes and builds the store under them: flat, id-addressed node files that readers use without coordination, ordered writes that survive a kill, and a hot cache in the long-lived server. M2 builds the graph semantics on this layer; M4 wraps its operation types as tools.

## Reference Documents
- [R01 Architecture](../../__design__/rdm/architecture.md) — §Data Model, §Concurrency, §Contracts & Invariants; superseded where A2 says so, synced by `settle/r01_sync.md`
- [R02 A2 gap analysis](../../__reports__/m1-core-foundation/00-gap_analysis_v0.md) — Eliott's rulings of 2026-11-25 this campaign implements
- [R03 M0b design](../../__design__/rdm/) — `stored-schema.md`, `read-model.md`, `statuses.md`: the settled meanings and types
- [R04 M0 and M0b knowledge transfer](../../__reports__/m0-feasibility-spike/) and [`__reports__/m0b-domain-model-spec/`](../../__reports__/m0b-domain-model-spec/) — lessons carried into the gates (a schema test for every output, gate on exit codes, verifiers challenge relaxed rules)

## Goal
`rdm-schema` holds the contracts (stored types, campaign model, `MOVES`, store operation and error types, declared links, version chain), and `rdm-engine` stores a campaign as flat JSON node files that survive any kill, served hot by a revalidated cache.

## Pre-conditions
- [x] Plan approved by Eliott (2026-11-25), with amendment A2 landed on the meta-roadmap
- [x] `feat/m1-core-foundation` is cut from `claude/m1-core-foundation-json-33306d`
- [x] The shared target directory `$CARGO_TARGET_DIR` is set for every agent (see Gotchas)

## Success Gates
- ✅ Eliott has reviewed `__design__/rdm/store.md` and `__design__/rdm/contracts.md`, and R01 matches them in the same merge
- ✅ `cargo test --workspace --locked` and `cargo +1.88 check --workspace --locked` pass on `feat/m1-core-foundation`
- ✅ The three-OS CI matrix (macOS, Linux, Windows) and the MSRV job are green on `feat/m1-core-foundation`, soak included (**not met for the Windows soak**, cancelled at the 45-minute cap; Eliott ruled on 2026-11-26 that M1 merges with this recorded as a limit, and the read path is redesigned in its own PR: `__reports__/rdm_read_path_efficiency/`)
- ✅ `! cargo test --workspace -- --list 2>/dev/null | grep confirm_` passes: no confirm is left
- ✅ Every leaf's guards were seen red before green, each with its named bug, as recorded in the leaf reports and checked by `settle/build/codec/store/harden/verify/adversarial_review.md` (I7 and I12 have no recorded mutation; Eliott ruled the mutation checks sufficient, 2026-11-25)
- ✅ The rescoped gates of the M1 leaf hold: contracts, version refusal, I3, I4, I7, I10, I12, declared links

## Gotchas
- **Rulings (Eliott, 2026-11-25), not to be reopened by an implementer:** flat `nodes/<id>.json` with `name`, `parent_id`, an order key, and depth computed from `parent_id`; Rust types are the contract and JSON is derived; a version chain of responsibility; a file newer than the binary is refused with `NewerFormat`; declared links, never a stored value derivable from an id; tracked data stays text; the cache is never the source of truth. A leaf that finds one of these wrong reports it as a finding. **Review rulings (Eliott, 2026-11-25, reply 5):** every newer file is refused, a higher X or a higher Y; unknown keys survive and each one yields an `unknown_key` diagnostic; `open` refuses per-file faults but loads and loudly reports cross-file merge damage, anchored on the root named by the expected slug, and refuses writes that build on an unvalidated node except repairs; `op_id` replay is the store's; `FORMAT_VERSION` stays `0.1.0` until M1 merges to `main`; `Reference.target` is relative to the root and `Change` holds no paths.
- **Test roles** (designing-tests). A *guard* is a Success Gate, stays in the suite, and is named as a sentence. A *confirm* is scaffolding with the `confirm_` name prefix, culled at leaf done by the cull gate above. A *probe* stays outside the suite: its answer goes into the design doc or a report, and its code is deleted.
- **Fixtures come only from real producers:** hand-written canonical files reviewed line by line against `contracts.md`; the store's own writes and their prefixes; transformations a real actor performs (an editor writing in place, `git merge-file`, a CRLF checkout, `git checkout`). No synthetic old-format files. The 160-node benchmark campaign is built through the store's create API.
- **Expected values are independent of the code:** `statuses.md` is the oracle for `MOVES`, the generated JSON Schema for `Links`, a brute-force scan for the reverse index, a fresh cold open for the cache, the full or absent operation for crash prefixes. No snapshot is blessed unreviewed.
- **Randomness is faked, the filesystem is real:** seeded RNG for minting and proptest; real files, processes, `rename` and locks. `proptest-regressions/` is ignored by git; a shrunk failure becomes a named row only if it is a new class.
- **Probes** (question, leaf that answers it, decision it feeds):
  - Lock crates on 1.88, and lock release when the holder is killed: `store_design` (macOS scratch; Linux and Windows through the `lock_survives_holder_kill` guard in CI). Decides the lock crate.
  - Windows `rename` over a target another program holds open: `store_design` (Win32 docs; the `replace_while_target_open` guard in CI). Decides the retry count and back-off.
  - Directory mtime granularity, and whether an in-place edit changes it: `store_design` (docs, macOS scratch). Decides the fast path, the per-file fallback, the racy-clean window and `notify`.
  - fsync cost per write (file and directory; `F_FULLFSYNC`): `store_design` (macOS scratch). Decides the fsync policy.
  - CRLF on a Windows checkout: `ci_matrix` (`git ls-files --eol` on the runner). Confirms the decision: the decoder accepts CRLF, and this repo pins its JSON fixtures to LF.
  - Cold vs warm load of 160 nodes: `hot_cache` (scratch bench). Decides whether an rkyv cache is built at all.
- **When to run what:**
  - Each edit, implementer: `cargo test -p <crate> --test <file> <guard>` and its confirms.
  - Leaf done, implementer: the crate's tests, `cargo clippy -p <crate> --all-targets`, the cull gate, each new guard seen red (reported).
  - Level merge, coordinator: `cargo test --workspace --locked` and `cargo +1.88 check --workspace --locked`.
  - Every change, everyone, the must-run set in under 30 s: schema drift, codec properties, newer-file refusal, I4, I7, reverse index, every-prefix-loads, cached equals cold, the short kill run.
  - Soak, CI matrix on the gated push: kill harness and proptest at high counts (`RDM_SOAK_ITERS`, `PROPTEST_CASES`), MSRV, CRLF probe.
- **Deliberately not tested:** power-loss reordering (needs a fault-injecting filesystem; the fsync policy is argued in `store.md`); serde and std behaviour; chain fall-through until a second real layer exists; tool I/O (M4); the newer-binary lookup (M7).
- **Execution model:** the main thread coordinates, integrates and reviews the seams after each level; it writes no feature code. Every subagent is dispatched with `model: "sonnet"`. One implementer per leaf on `task/m1-<leaf>` in its own worktree under `/Users/hacker/Documents/tmp/claude-worktrees/cracking-shells-playbook/`; `codec_and_versions` and `campaign_store` share one implementer, resumed with `SendMessage`. Fixes go back to the implementer that did the work. Fresh Sonnet verifiers attack `fs_primitives`, `codec_and_versions`, `campaign_store`, `crash_test` and `hot_cache`; `schema_contracts`, `ci_matrix` and `r01_sync` merge on the implementer's report. Every brief says: report a spec defect as a finding instead of building it.
- **Environment:** Rust with local `cargo` (stable 1.99; toolchain 1.88 installed for the MSRV check). Every agent uses `CARGO_TARGET_DIR=/private/tmp/claude-501/-Users-hacker-Documents-tmp-claude-worktrees-cracking-shells-playbook-roadmap-mcp-m0-milestones-8bc366/2632a31d-93c8-44fb-9192-907e230b2d13/scratchpad/m1-target`, never a private one; it is removed when the milestone lands. Gate on exit codes, not on terminal text. Probe code lives in the scratchpad and is deleted after its answer is recorded. Search with colgrep scoped to the agent's own worktree path.
- **Git:** commits follow `writing-history`. The coordinator rebases each verified branch onto the tip of `feat/m1-core-foundation`, re-verifies after any conflict, and merges with `--no-ff`, messages passed with `-F`. No bare `git stash`. Nothing is pushed until Eliott says so: the coordinator asks once, when `ci_matrix` and `crash_test` are merged, for the push that runs the three-OS matrix.
- **Budget:** USD 60 for the campaign. Spend is reported at every level close; at USD 48 the coordinator stops and asks Eliott.

## Status
```mermaid
graph TD
    store_design[Store design]:::done
    contracts_design[Contracts design]:::done
    settle[Settle]:::done
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `store_design.md` | 📄 Leaf Task | ✅ Done |
| `contracts_design.md` | 📄 Leaf Task | ✅ Done |
| `settle/` | 📁 Directory | ✅ Done |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
