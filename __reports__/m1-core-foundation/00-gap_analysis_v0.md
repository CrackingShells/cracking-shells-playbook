# Gap Analysis: roadmap-mcp-rework — the M1 leaf predates M0 and M0b (v0)

Date: 2026-11-25

This report is the source of amendment A2 of `__roadmap__/roadmap-mcp-rework/`. Eliott approved the rulings below on 2026-11-25, in the session that planned `__roadmap__/m1-core-foundation/`. A2 adds no node. It edits the prose of the M1, M2, M4 and M7 leaves and of the meta-roadmap README.

## Problem Statement

The M1 leaf was last edited before the M0 host trials and the two M0b review rounds finished. Its gates and its planning list rest on three assumptions those rounds replaced:
- a tree of nested directories;
- the refusal of a newer format *major* only;
- schemas that are settled and only need consuming.

The leaf also defers three decisions to each other, so nobody owns them. M1 cannot be planned faithfully from the leaf as written.

## Evidence

| What | When (commit) |
|:--|:--|
| Last edit of `core/m1_core_foundation.md` | 2026-11-16 15:28 (`af8ad04`, the crate rename) |
| M0 host trials (desktop `_meta`, VS Code, CLI) | 2026-11-16 17:28–17:41 (`ff6cc8c` … `6f84ad5`) |
| M0 and M0b knowledge-transfer reports | 2026-11-16 18:33–18:34 (`ffc3b38`, `18b0615`) |
| M0b review: criteria enforced, three-part versions `0.X.Y` | 2026-11-16 18:35 (`9591a66`, `6843cc3`) |
| M0b second review: reasoned edits, stable sub-item ids | 2026-11-16 18:56–18:57 (`6a1bd41`, `265d873`, `c8b617f`) |

Ownership loops and gaps found by reading the leaf against the M0b design docs:

- **Id format.** `stored-schema.md` says the id format "belongs to M1". The M1 leaf says to mint ids "to the format M0b fixes". Neither owns it.
- **Newer formats.** The leaf's first gate refuses a newer *major*. The M0b review moved the format to `0.X.Y`, where a higher minor X must also be refused (`is_readable` in `crates/rdm-schema/src/lib.rs`).
- **Comments.** `_comments/` exists in the stored schema and in the read model ("a comment never changes a `rev`"). The M1 leaf names neither its layout nor its write path.
- **Testing lessons.** M0 learned to put a schema-validation test on every output. M0b learned to brief verifiers to challenge any relaxed rule. Neither lesson appears in the leaf's gates.
- **CI.** `.github/workflows/rust.yml` runs on `ubuntu-latest` only, with no MSRV job, while the leaf gates on three operating systems. It runs only on pull requests and pushes to `main`.
- **Lock files.** `.gitignore` has no `.rdm/` entry, but the lock and journal will live there.
- **Byte-for-byte round trip.** It holds only for canonical input. `serde_json` rewrites `1e5` and `\u` escapes, so a hand-edited file cannot round-trip byte for byte. The gate as written cannot be met for every input.
- **`op_id` replay.** `op_id` appears on stored records, but no leaf owns the rule for replaying an operation that was retried.
- Observed but outside A2: the meta README shows M0 and M0b `done`, while their leaves' criteria are still unticked. To be reconciled at M1 close-out, not here.

## Root Cause

The M1 leaf was written as a store-and-lock milestone over schemas that M0b would hand over finished. M0b did hand over the types. But the design questions that sit between the types and the files fell back to M1 without the leaf being rewritten. Those questions are: what a file holds, how a file is named, how versions evolve, and what is linked to what. The session that planned M1 then took design rulings that change the layout itself.

## Rulings (Eliott, 2026-11-25)

1. **Flat, id-addressed layout.** Each node is one file, `nodes/<id>.json`, holding its `name`, its `parent_id` and a sibling order key. The filesystem no longer mirrors the tree. Depth stays the ordering rule (I1), computed from the `parent_id` chain. A rename or a move writes one file and no directory is renamed. This overrules the earlier "names are paths" layout of R01: "The value of what you are suggesting over the value of keeping the depth in the file system is higher."
2. **Concurrency is a layout problem.** Readers need no coordination. The write path uses named, established algorithms, and their trade-off is argued.
3. **The Rust types and the data layout are the contract.** JSON and JSON Schema are derived from them. "Schema" means the data structures broadly: stored nodes, the campaign model and operation signatures, not only JSON Schema files.
4. **Schema evolution uses a chain of responsibility.**
   - Each format version is a layer that overrides only what changed.
   - Behaviour a version lacks defaults to "not in this version".
   - A field removed later is overridden back to that default.
   - Older files fall through the chain to the layer that reads them.
5. **A file newer than the binary is refused.**
   - The binary refuses a file whose format it does not support. No layer exists that could interpret what changed, and old code must never misread a field whose meaning changed.
   - Making this transparent is M7's concern, not M1's (see M7's open question below).
   - Eliott at first proposed that a higher X need not be refused. He agreed to the refusal after the case above was raised.
6. **Links are declared, never left to prose.** Every key-value link between records is declared in the contract, so an update or a delete reaches every reference automatically. A value that can be derived from an id is never stored.
7. **The tracked campaign data stays text.**
   - Per-node merging is why the layout exists, and binary files neither merge nor show in a PR diff.
   - Derived artefacts may come from CI: JSON Schemas, and docs rendered from the types. Campaign data is merged by git before CI runs, so CI cannot produce it.
   - A binary form of the campaign, if one is ever built, is an untracked cache in `.rdm/`.
8. **A hot in-memory campaign in the long-lived server process.**
   - It is a cache, revalidated against the files on every read.
   - Writes go straight through to the files.
   - It is never the source of truth, because other processes (another worktree, the CLI, an editor, git) change the files under it.
9. **Scope.**
   - M1 freezes: the stored types, the campaign-level model, the status move table as data, and the store-level operation requests, results and errors.
   - Tool input and output stay with M4.
   - M2 consumes this layer, adding the operations' semantics, the FSM and impact analysis.
   - Contract design leaves M4.
   - Budget for the M1 campaign: USD 60, with a checkpoint at USD 48.

## Impact Assessment

- **Scope:** no new node in the meta-roadmap. Prose edits:
  - **M1:** goal, gates, Step 1.
  - **M2:** inputs.
  - **M4:** tool inputs wrap M1's operation types.
  - **M7:** the newer-format question.
  - **Meta README:** the Gotchas ownership line.
  - R01 is synced inside the M1 campaign (leaf `r01_sync`), in the same merge as the code.
- **Dependencies:** M2 now starts from M1's operation signatures and move table instead of designing them, and M4 from M1's operation types. M7 inherits a refusal that it may make transparent.
- **Risk if not addressed:**
  - M1 builds a nested store whose renames touch whole subtrees.
  - The id format stays ownerless.
  - Old binaries misread newer files.
  - Every consumer re-implements links by hand, which is the class of bug Eliott has seen in agent-written CRUD.

## Answers recorded for later milestones

- **M7, the newer-format hand-off.**
  - It is not a hot auto-update. A running server never swaps itself mid-session: it refuses the affected campaign with `NewerFormat{found, supported}` and keeps serving the others.
  - The realistic transparent path, for M7 to decide: at launch, look for a binary that supports `found` and is already on the machine. The plugin cache keeps versioned directories, and the newer file usually came from a newer plugin version that is installed. If one is found, start it.
  - Downloading one is an install and needs the user's consent.
  - Otherwise the user sees the message naming the version needed.
- **M4.** Tool inputs wrap the operation request types of `rdm-schema`, and tool errors map from M1's error type.

## Proposed Solution

| Change | Detail |
|:--|:--|
| M1 leaf | **Goal:** contracts, the version chain, the flat store with ordered writes, and the hot cache. <br>**Gates:** the rescoped gates of the M1 campaign, as below. <br>**Step 1 Implementation Logic:** points to this report and to the campaign. |
| M2 leaf | **Pre-condition and planning list:** takes M1's operation signatures, `MOVES` and declared links as given; decides semantics, not shapes. |
| M4 leaf | **Planning list:** input schemas wrap M1's operation request types; the error mapping starts from M1's error type. |
| M7 leaf | **Planning list:** gains the newer-format hand-off (look up an installed binary that supports the file's format, never download without consent). |
| Meta README | **Gotchas ownership line:** M1 owns the contracts beyond the stored types (campaign model, move table as data, operation types, declared links), the id format and minting, the error type and the version chain. <br>**Amendment Log:** A2 row. |

The rescoped M1 gates:

- `__design__/rdm/store.md` and `__design__/rdm/contracts.md` are reviewed by Eliott, and R01 matches them in the same merge.
- `rdm-engine` holds no stored or contract type of its own; it depends on `rdm-schema`.
- `MOVES` matches the move table of `statuses.md`, and the declared links cover every id field.
- Every older format version reads through the version chain. A newer one is refused with `NewerFormat` before anything else is parsed.
- Canonical files round-trip byte for byte. Valid non-canonical input reads and re-encodes canonically. Unknown keys survive (I12).
- Every write prefix of every store operation loads and recovers to the state before or after it. A multi-process kill test on macOS, Linux and Windows loses no acknowledged write (I3).
- A stale replace changes nothing (I4). A move writes one file and changes no id (I7). The reverse link index always equals a full scan.
- A cached read equals a cold read for every kind of external change (I10, reworded).

## Recommendations

1. Land A2 on the meta-roadmap with this report as its source, then plan `__roadmap__/m1-core-foundation/`.
2. Write the Amendment Log row of A2 by hand, as A1's was, and check it with `dirtree-rdm validate`. The legacy tool still has no amendment writer.
3. The reminder `skills-rdm-M1` was estimated (150+150 minutes) before this rescoping. Propose a re-estimate to Eliott; apply it only on his yes, through the scheduling-work writer.
4. `op_id` replay: M1's `contracts_design` leaf decides whether the store or M2 owns it, and records the decision.
