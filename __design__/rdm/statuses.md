# rdm — Status semantics

What each status of a leaf means, how it may change, and what it does to the order of work. The
vocabulary and the editing rules are Eliott's decisions of 2026-11-16 (M0b README, Gotchas). Where
this document goes beyond them, a note says who decided it: Eliott or the coordinator, both on
2026-11-16. No question is open.

This document settles the meaning. The stored shape of the fields it names (`status`,
`status_reason`, `history`, and the comment files) is in [stored-schema.md](stored-schema.md). R01
(`architecture.md`) is not edited here; section `R01 deltas` lists what `r01_sync` must change.

## Statuses

Only a leaf has a stored status. A directory's status is derived (see `Subtrees and directories`).

| Status | Meaning | Finished | Edits |
|:--|:--|:--|:--|
| `planned` | The leaf is written and not started. No step is recorded done. | no | amendment note |
| `ongoing` | Work has started on the leaf. Steps may be recorded done. | no | amendment note; a correction of a recorded step needs a reason |
| `paused` | Work is held and can be resumed: a bug, a decision to wait, or a downstream branch held on purpose. | no | as `ongoing` |
| `cancelled` | The leaf is abandoned. It holds nothing back while it is cancelled. Whatever was done stays on record. A mistaken cancel can be undone: the leaf returns to the status it had just before the cancel. | yes | any edit, with a reason (a correction) |
| `done` | Every step is done with its commit recorded and every gate `met`. The normal end state. It can be reversed or corrected, each time with a reason. | yes | any edit, with a reason (a correction) |

`blocked` and `amendment` are not statuses. A "blocked" situation is `paused` (resumable) or
`cancelled` (abandoned). An "amendment" is an edit recorded in the amendment log, and a node
added by one starts as `planned`.

An unfinished leaf (`planned`, `ongoing`, `paused`) holds back deeper leaves. A finished leaf
(`done`, `cancelled`) holds back nothing. `done` is the normal end state, not a lock: a finished node can be edited or reversed with a reason (`Finished nodes`).

## Transitions

All 25 ordered pairs. A move out of `done` is a reversal: its history entry carries `correction: true` and a non-blank reason. "Who" is the actor kind recorded in `history[].actor`: `agent`, `human` or
`both`. "Reason" is the `reason` of the history entry (copied to `status_reason` when the new
status is `paused` or `cancelled`). "Ready set" says what the move does to the set defined in
`Ready set`: ENTER, LEAVE, RELEASE (deeper leaves may become ready), HOLD (a finished leaf is unfinished again, so deeper ready leaves stop being ready) or none.

| From | To | Allowed | Who | Reason | Precondition | Ready set |
|:--|:--|:--|:--|:--|:--|:--|
| `planned` | `planned` | refused | | | A move to the same status is a no-op and writes no history | none |
| `planned` | `ongoing` | allowed | both | only for an override | Every pre-condition is `met`, and the leaf is in the ready set. Outside the set, an override with a reason is needed, and the entry records `override: true`. The override skips the ready set, never the pre-conditions (Eliott, 2026-11-16) | LEAVE |
| `planned` | `paused` | allowed | both | required | none | LEAVE |
| `planned` | `cancelled` | allowed | both | required | none | LEAVE, RELEASE |
| `planned` | `done` | refused | | | Start the leaf first: `done` needs the steps recorded (I8) and the gates `met` | none |
| `ongoing` | `planned` | refused | | | Use `paused`, then `planned` while no step is done | none |
| `ongoing` | `ongoing` | refused | | | No-op | none |
| `ongoing` | `paused` | allowed | both | required | none | none |
| `ongoing` | `cancelled` | allowed | both | required | none. Steps already done stay recorded | RELEASE |
| `ongoing` | `done` | allowed | both | none | Every step is done with its commit recorded (I8), and every success gate is `met` (Eliott, 2026-11-16). See `Criteria` | RELEASE |
| `paused` | `planned` | allowed | both | required | No step is recorded done (coordinator, 2026-11-16) | ENTER if nothing shallower holds it back |
| `paused` | `ongoing` | allowed | both | required | The leaf was `ongoing` before the pause (last pause entry has `from: ongoing`), or it is in the ready set. Otherwise an override with a reason | none |
| `paused` | `paused` | refused | | | No-op | none |
| `paused` | `cancelled` | allowed | both | required | none | RELEASE |
| `paused` | `done` | refused | | | Resume to `ongoing` first | none |
| `cancelled` | `planned` | allowed | both | required | Undo: the status before the cancel, read from history (the `from` of the last cancel entry), is `planned`. Eliott's intent 2026-11-16, ruling (coordinator, 2026-11-16) | ENTER if nothing shallower holds it back |
| `cancelled` | `ongoing` | allowed | both | required | Undo: the status before the cancel was `ongoing`. Any other target is refused (coordinator, 2026-11-16) | none |
| `cancelled` | `paused` | allowed | both | required | Undo: the status before the cancel was `paused`. Any other target is refused (coordinator, 2026-11-16) | none |
| `cancelled` | `cancelled` | refused | | | No-op | none |
| `cancelled` | `done` | allowed | both | required | Undo only: the status before the cancel was `done` (a leaf cancelled by a reversal out of `done`). Flagged `correction: true`. Any other route into `done` from `cancelled` is refused (coordinator, 2026-11-16) | RELEASE |
| `done` | `planned` | refused | | | The leaf has steps recorded done, and `planned` means not started. Reverse to `ongoing` | none |
| `done` | `ongoing` | allowed | both | required | A reversal (Eliott, 2026-11-16). Recorded with `correction: true`. No gate or step is changed: they stay as recorded | HOLD: deeper leaves that were ready stop being ready; deeper leaves already started are not stopped |
| `done` | `paused` | allowed | both | required | A reversal, `correction: true` | HOLD |
| `done` | `cancelled` | allowed | both | required | A reversal, `correction: true`. Steps and gates stay as recorded | none |
| `done` | `done` | refused | | | No-op | none |

Rules that hold for every row:

- No move is human-only. `history[].actor` records who made it.
- A move presents the status it expects to leave. A mismatch is refused (R01 §Concurrency).
- A move writes one history entry and never an amendment (I5). A criterion mark writes a mark record, see `Criteria`.
- A move into `done` or `cancelled` may carry the final `progress` in the same operation. After it, every edit is still accepted, with a reason (`Finished nodes`).
- The reason is required text, not blank, for `paused`, `cancelled`, `paused → planned`, `paused → ongoing`, every undo out of `cancelled` and every move out of `done`.

**`paused → planned` needs "no step is recorded done".** A step recorded done has its
commit on record, so a `planned` leaf holding a recorded commit would contradict "not started". A leaf with a recorded step goes to `ongoing`
instead. Decided 2026-11-16 (coordinator).

**`cancelled` is undoable.** A cancel is one call with a reason, and a directory cancel is one
call over many leaves, so a mistake must be reversible. Undoing a cancel restores the status the
node had just before the cancel, with a required reason, recorded in history. That status is read
from history: the `from` of the last cancel entry. It may be `planned`, `ongoing`, `paused` or, for a leaf reversed out of `done` and then cancelled, `done`. Any
other target out of `cancelled` is refused. A leaf with no cancel entry in its history (one imported from legacy `blocked`) returns to `planned`, as the importer writes no steps done. A leaf cancelled from `done` is restored to `done`, as the table says.
Eliott's intent 2026-11-16 ("undo a mistaken cancel with a reason"); the restore rule is the
coordinator's ruling, 2026-11-16.

**Closed gap: a cancelled leaf with a recorded step.** The first form of the undo moved only to
`planned`, which a leaf with a done step cannot take, so such a leaf could never be resumed. The
restore rule fixes it: a leaf cancelled from `ongoing` or `paused` returns there.

**Note: a cancel releases work without saying so.** Because `cancelled` holds nothing back, a
cancelled leaf can release a deeper leaf that really needed it. That is Eliott's intent. The
impact report of a cancel (M2) should list the leaves it makes ready, so the release is never
silent.

## Criteria

Eliott, 2026-11-16: verified pre- and post-conditions are cornerstones of AI planning, an
unmaintained checklist is pointless, and a wrong check must be traceable. So criteria are
enforced, and every mark leaves a trace.

- **Pre-conditions gate the start.** `planned → ongoing` needs every pre-condition `met`. This holds for an override too: an override skips the ready set, never the pre-conditions. The same holds for `paused → ongoing` by override.
- **Success gates gate the finish.** `ongoing → done` needs every success gate `met`, in addition to I8.
- **Every mark is traced.** A change of a criterion's `met` mark, on a leaf or a directory, writes a mark record: `ts`, `actor`, the criterion's `criterion_id` (the stable id of the criterion, never its position), the new value, `evidence?` (free text: a command, its result, a commit), and on a finished node `reason` and `correction: true`. Leaves and directories keep the records in their own `marks` array, apart from `history` (stored-schema.md).
- **`met` is a state mark, not content.** Setting it makes no amendment. On a `done` or `cancelled` node a mark needs a non-blank reason and carries `correction: true` (`Finished nodes`). A wrong mark is corrected by another mark record, at any time; both records stay.
- **Legacy data.** 30 of 48 done leaves under `__roadmap__/` carry open gates. The importer never invents a tick: it imports them as they are, and `validate` reports each `done` node with an unmet criterion as an error to reconcile. Whoever reconciles ticks the criterion with an ordinary reasoned mark (`correction: true`, `reason` or `evidence`), which writes a mark record. The count of such corrections is data (`Finished nodes`).

## Ready set

A leaf is ready when it is `planned` and every leaf that holds it back is `done` or `cancelled`.
So `cancelled` holds nothing back while it is cancelled. Undoing a cancel returns the leaf to its earlier status; if that is
`planned`, `ongoing` or `paused` it holds back again; deeper leaves that already started are not stopped.

**The rule (scoped, the legacy rule; Eliott, 2026-11-16).** An unfinished leaf holds back only
the deeper leaves under its own parent. Precisely: leaf L is held back by every unfinished leaf
that is a leaf child of the parent of A, for A = L's own directory and each ancestor directory of
L below the campaign root. Leaves in an unrelated branch never hold L back, and sibling leaves are
parallel.

Equivalently: a leaf directly under directory P holds back every leaf found in the directories
under P, at any depth, and nothing else. A leaf directly under the campaign root holds back every
leaf in the campaign's directories.

The same rule applies to `planned`, `ongoing` and `paused`: a `paused` leaf holds back exactly what
a `planned` or `ongoing` one does. To hold a branch on purpose, pause its directory.

Worked on `__roadmap__/roadmap-mcp-rework/`, a chain: M5 sits in `consumers/`, so pausing it holds
back M6, M7 and M8 in `consumers/delivery/` and below. A global rule would give the same set
there. The two differ with a second branch. Hypothetical: a directory `docs/guide/` at the
campaign root, with a leaf at depth 2, next to `core/`. Pausing M1 (parent `core/`) holds back M2
to M8, which sit under `core/`, and does not hold back the docs leaf. The global rule (R01's)
would have.

An `ongoing` leaf is not in the ready set, because it is not `planned`. A leaf started by an
override (`planned → ongoing` outside the set) is recorded and allowed.

## Subtrees and directories

**Pausing or cancelling a directory** is one operation over its leaves.

- `pause` sets every `planned` and `ongoing` leaf under the directory to `paused`.
- `cancel` sets every `planned`, `ongoing` and `paused` leaf under it to `cancelled`.
- Leaves already finished are left alone. A leaf where the move is refused is skipped, never an error for the whole operation.
- One reason, one timestamp and one `op_id` are shared by every history entry the operation writes. The operation is one call and one transaction, so it is "recorded once".
- It is a snapshot, not a standing flag. A leaf added to the directory later starts `planned`.
- `resume` of a directory sets each `paused` leaf under it to the `from` of its last pause entry (`planned` or `ongoing`). A leaf whose last pause came from another operation is resumed too, each by its own rule. The reason is required.
- `uncancel` of a directory restores every `cancelled` leaf whose last history entry carries the `op_id` of the cancel being undone. Each leaf returns to the `from` of that entry, so a leaf may come back `planned`, `ongoing` or `paused`. One reason and one new `op_id` are shared. Leaves cancelled by another operation are left alone.

**A directory's status is derived on every read and never stored (I6).** It is computed from all
leaves under it, at any depth. The first matching row wins.

| # | Leaves under the directory | Derived status |
|:--|:--|:--|
| 1 | none | `planned` |
| 2 | all `cancelled` | `cancelled` |
| 3 | all finished, at least one `done`, and the directory's own gates all `met` | `done` |
| 3b | all finished, at least one `done`, and a gate of the directory not `met` | `ongoing` |
| 4 | at least one `ongoing` | `ongoing` |
| 5 | at least one `paused` | `paused` |
| 6 | at least one `done`, the others `planned` or `cancelled` | `ongoing` |
| 7 | otherwise (`planned`, with or without `cancelled`) | `planned` |

Row 3 reads a directory whose leaves are all finished as `done` even if some were cancelled, once its own gates are met. Until then it is `ongoing` (row 3b): the directory's gates are the last thing left to check. A
reader that must tell them apart uses the counts per status, which the read model carries.

## Finished nodes

Eliott, 2026-11-16: much of the data is best effort and should not be modified once set, but for
convenience anything may be edited as long as a reason or comment is given, and the count of such
reasons is data in itself: it measures how often corrections and reversals happen. So nothing is
locked. A node is *finished* when it is `done` or `cancelled` (a directory: derived status `done`
or `cancelled`). Every edit of a finished node needs a non-blank reason and is recorded as a
**correction**.

| Status | Content edit | Comment | Status move | Progress | Criterion mark |
|:--|:--|:--|:--|:--|:--|
| `planned` | allowed, amendment note required | allowed | per `Transitions` | allowed | allowed |
| `ongoing` | allowed, amendment note required. Correcting a step recorded done is a correction | allowed | per `Transitions` | allowed | allowed |
| `paused` | as `ongoing` | allowed | per `Transitions` | allowed | allowed |
| `cancelled` | allowed, as a correction | allowed | the undo, to the pre-cancel status | allowed, as a correction | allowed, as a correction |
| `done` | allowed, as a correction | allowed | to `ongoing`, `paused` or `cancelled`, as a reversal | allowed, as a correction | allowed, as a correction |

- **Content** is every stored field except `status`, `status_reason`, `history`, `marks`, `progress` and the state marks. The state marks are a criterion's `met` with its mark record, and a step's `done`, `commit_sha` and `done_at`.
- **The amendment rule is uniform.** Any content edit to a `planned`, `ongoing` or `paused` node carries the amendment note, whatever the status. Those edits are unchanged and are not corrections.
- **A correction is identifiable.** It carries `correction: true` on its record: the amendment (content edit, progress edit, or correction of a recorded step), the mark record (a mark on a finished node), or the history entry (a reversal out of `done`). Its reason is the amendment `description`, the mark `reason` (or `evidence`), or the history `reason`. The reason is not blank, and the engine refuses a blank one.
- **Corrections are counted.** The node detail carries `correction_count`, derived on read and never stored: the number of records with `correction: true` that name the node (amendments, marks and history entries). It measures how often corrections and reversals happen.
- **Correcting a recorded step.** Recording a step done is a state mark and writes no amendment. Correcting it afterwards (a changed `commit_sha`, a cleared `done`, an edit of its content) is an amendment with `correction: true` and a `set` change that names the step by `item_id`, on any node, finished or not. The step keeps its id, so records that point at it stay valid. Steps and criteria can be inserted, removed and reordered; stable ids keep every reference valid (`stored-schema.md`, `Stable identity`).
- **Status moves, progress, comments and state marks never produce an amendment (I5)** on an unfinished node. Setting `met` is a state change like a move, and writes a mark record.
- **Reversal.** A move out of `done` to `ongoing`, `paused` or `cancelled` needs a reason and is recorded in history with `correction: true`. Steps and gates stay as recorded. `done → planned` stays refused.
- **Directories follow their derived status.** An edit of the content of a directory (or the campaign root) whose derived status is `done` or `cancelled` is a correction. A directory in any other derived status takes edits with an amendment note. Adding or removing a child is a structure edit of the child, not a correction of the directory.
- **Criteria gate the moves.** `done` needs I8 and every gate `met`; the start needs every pre-condition `met` (see `Criteria`). A directory is `done` only when its leaves are finished and its own gates are met. A reversal out of `done` ignores both: the leaf is already past them.
- **Structure edits on finished nodes.** Deleting a `done` or `cancelled` leaf keeps its acknowledgement (I9) and its snapshot in the amendment. Moving a finished leaf changes no content. Whether a move that changes the ready set of other leaves needs the acknowledgement is left to M2's impact analysis.

## Legacy mapping

For the importer (M3). Counts are from `__roadmap__/` at the date of this document.

| Legacy status | New status | Reason and provenance | In real campaigns |
|:--|:--|:--|:--|
| `planned` | `planned` | Same meaning | 26 rows |
| `inprogress` | `ongoing` | Rename | 4 rows, all in campaigns being planned now |
| `done` | `done` | Same meaning. Legacy `done` was "immutable; reopen via a new amendment node"; a `done` node can now be corrected or reversed with a reason | 65 rows |
| `blocked` | `cancelled` | The legacy skill defined `blocked` as terminal within a campaign, never auto-resolved, which is how it was used to abandon work. The legacy file holds no reason, so the importer writes `status_reason: "imported from legacy status blocked"` and warns | 0 rows |
| `amendment` | `planned` | Legacy `amendment` was the initial status of a node added by an amendment, never a state an existing node moved to. The provenance stays in the amendment: the importer writes the `A<n>` row as an amendment (`legacy_id`) with a change `op: add` for each filename in its "Nodes Added" cell | 0 rows. The one node added by an amendment (`hub_mode_reconciliation.md`, A1) is `done` |

Legacy files carry no timestamps and no history of moves. The importer writes no history entries:
the status is set directly and `history` starts empty.

## R01 deltas

For `r01_sync`. R01 is not edited by this leaf.

| R01 location | Change |
|:--|:--|
| §Graph operations, Status FSM diagram | Replace with the five-status machine of `Transitions`: `planned → ongoing, paused, cancelled`; `ongoing → paused, cancelled, done`; `paused → planned, ongoing, cancelled`; `cancelled → planned, ongoing, paused, done` (only the pre-cancel status); `done → ongoing, paused, cancelled` (reversals with a reason). Remove `blocked`, `amendment` and the `done → amendment` edge |
| §Graph operations, Order | "not done" becomes "`planned`"; "every leaf of smaller depth is done" becomes "`done` or `cancelled`". The ready rule becomes the scoped rule of `Ready set`. The sentence "Wave semantics are global: a leaf also waits for shallower leaves in unrelated branches" is replaced: a leaf waits only for shallower leaves under its own ancestors' parents. "Waves" stay a display of depth (`level`), not a readiness fact |
| §Graph operations, after the FSM | "Starting a leaf outside the ready set is refused unless an override with a reason is given" stays. Add the directory pause and cancel operation and the derivation table |
| §Graph operations, Impact analysis | "done work reopened" stays, now for a reversal out of `done`. Add "leaves made ready by a cancel" |
| §Contracts, I5 | Add: comments (separate append-only files) and state marks never produce an amendment. A content edit of a finished node needs a reason and is a correction |
| §Contracts, I8 | Keep I8. Add: `done` also needs every success gate `met`, and the start needs every pre-condition `met`; every mark is traced |
| §Contracts, I9 | "Deleting or reopening done work, or moving work in progress, needs an explicit acknowledgement" becomes "Deleting finished work, or moving work in progress, needs an explicit acknowledgement. Reopening done work needs a reason and is counted as a correction" |
| §Data Model, schema sketch | criteria carry an `id` and `met`, with mark records (`criterion_id`, `correction?`) in `marks`; `blocked_reason?` becomes `status_reason?`; history `note?` becomes `reason?`; comments are not a node field: they are files `_comments/<ulid>.json` (details in stored-schema.md), so the §Data Model tree gains `_comments/` |
| §Concurrency, last bullet | "records `inprogress`" becomes "records `ongoing`" |
| §Graph operations, after the FSM | Add a directory `uncancel` through the `op_id` of the cancel |
| §Key Flows, Execute | `inprogress` becomes `ongoing` |
| §Alternatives Considered, Mutation policy | "full CRUD with guards on done work" is now "full CRUD, finished nodes edited with a reason" |

## Decisions

Nothing is open. Who decided what:

| # | Decision | Decided |
|:--|:--|:--|
| 1 | Ready rule: an unfinished leaf holds back only the deeper leaves under its own parent (legacy rule). Applies to `planned`, `ongoing` and `paused` alike | Eliott, 2026-11-16 |
| 2 | Comments are one file each, `_comments/<ulid>.json`, append-only | Eliott, 2026-11-16 |
| 3 | A mistaken cancel can be undone with a required reason; `done` stays the normal end state | Eliott, 2026-11-16 |
| 4 | `paused → planned` needs no step recorded done | coordinator, 2026-11-16 |
| 5 | A mistyped `commit_sha` or a wrongly recorded step is corrected by an amendment with a reason and `correction: true` (replaces the earlier permanent `commit_sha` and the recorded objection) | Eliott, 2026-11-16 |
| 6 | Undoing a cancel restores the status the node had just before the cancel, read from history, with a required reason: `planned`, `ongoing` or `paused`. A directory undo restores each leaf through the cancel's `op_id`. `done` is reversible with a reason (row 9). Replaces the earlier `cancelled → planned` only form and closes its gap (coordinator, 2026-11-16) | coordinator, 2026-11-16 |
| 7 | Criteria are enforced: `planned → ongoing` needs every pre-condition `met` (override included), `ongoing → done` needs every success gate `met` plus I8, and a directory is `done` only when its leaves are finished and its own gates are met. Every mark leaves a mark record (`ts`, `actor`, kind, index, value, `evidence?`). Legacy done leaves with open gates are imported as they are and reported by `validate`. Reverses the earlier rule that gate marks do not gate `done` | Eliott, 2026-11-16 |
| 8 | Reconciling a legacy done leaf's open gate is an ordinary reasoned mark (`reason` or `evidence`, `correction: true`). Closes the earlier open defect | Eliott, 2026-11-16 |
| 9 | Sealing becomes a reason requirement. `done` and `cancelled` nodes accept every edit (content as an amendment, marks as mark records, a recorded step as a correction amendment) and status reversals out of `done` (to `ongoing`, `paused`, `cancelled`), each with a non-blank reason and flagged `correction: true`. Unfinished nodes are unchanged. Replaces "sealed", "terminal" and "nothing reopens done work" | Eliott, 2026-11-16 |
| 10 | The count of corrections is data: the node detail carries a derived `correction_count`, never stored | Eliott, 2026-11-16 |
| 11 | A cancel made by a reversal out of `done` can be undone back to `done`, with a reason, so the undo rule has no dead end | coordinator, 2026-11-16 (gap closed, flagged) |
