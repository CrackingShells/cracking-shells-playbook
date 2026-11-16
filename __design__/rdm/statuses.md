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

| Status | Meaning | Finished | Sealed |
|:--|:--|:--|:--|
| `planned` | The leaf is written and not started. No step is recorded done. | no | no |
| `ongoing` | Work has started on the leaf. Steps may be recorded done. | no | steps recorded done only |
| `paused` | Work is held and can be resumed: a bug, a decision to wait, or a downstream branch held on purpose. | no | steps recorded done only |
| `cancelled` | The leaf is abandoned. It holds nothing back while it is cancelled. Whatever was done stays on record. A mistaken cancel can be undone: the leaf returns to the status it had just before the cancel. | yes | yes, while cancelled |
| `done` | Every step is done with its commit recorded. The only terminal status. | yes | yes, for good |

`blocked` and `amendment` are not statuses. A "blocked" situation is `paused` (resumable) or
`cancelled` (abandoned). An "amendment" is an edit recorded in the amendment log, and a node
added by one starts as `planned`.

An unfinished leaf (`planned`, `ongoing`, `paused`) holds back deeper leaves. A finished leaf
(`done`, `cancelled`) holds back nothing. Only `done` is terminal.

## Transitions

All 25 ordered pairs. "Who" is the actor kind recorded in `history[].actor`: `agent`, `human` or
`both`. "Reason" is the `reason` of the history entry (copied to `status_reason` when the new
status is `paused` or `cancelled`). "Ready set" says what the move does to the set defined in
`Ready set`: ENTER, LEAVE, RELEASE (deeper leaves may become ready) or none.

| From | To | Allowed | Who | Reason | Precondition | Ready set |
|:--|:--|:--|:--|:--|:--|:--|
| `planned` | `planned` | refused | | | A move to the same status is a no-op and writes no history | none |
| `planned` | `ongoing` | allowed | both | only for an override | The leaf is in the ready set. Outside it, an override with a reason is needed, and the entry records `override: true` | LEAVE |
| `planned` | `paused` | allowed | both | required | none | LEAVE |
| `planned` | `cancelled` | allowed | both | required | none | LEAVE, RELEASE |
| `planned` | `done` | refused | | | Start the leaf first: `done` needs the steps recorded (I8) | none |
| `ongoing` | `planned` | refused | | | Use `paused`, then `planned` while no step is done | none |
| `ongoing` | `ongoing` | refused | | | No-op | none |
| `ongoing` | `paused` | allowed | both | required | none | none |
| `ongoing` | `cancelled` | allowed | both | required | none. Steps already done stay sealed and recorded | RELEASE |
| `ongoing` | `done` | allowed | both | none | Every step is done with its commit recorded (I8). Gate marks are not checked, see `Sealing` | RELEASE |
| `paused` | `planned` | allowed | both | required | No step is recorded done (coordinator, 2026-11-16) | ENTER if nothing shallower holds it back |
| `paused` | `ongoing` | allowed | both | required | The leaf was `ongoing` before the pause (last pause entry has `from: ongoing`), or it is in the ready set. Otherwise an override with a reason | none |
| `paused` | `paused` | refused | | | No-op | none |
| `paused` | `cancelled` | allowed | both | required | none | RELEASE |
| `paused` | `done` | refused | | | Resume to `ongoing` first | none |
| `cancelled` | `planned` | allowed | both | required | Undo: the status before the cancel, read from history (the `from` of the last cancel entry), is `planned`. Eliott's intent 2026-11-16, ruling (coordinator, 2026-11-16) | ENTER if nothing shallower holds it back |
| `cancelled` | `ongoing` | allowed | both | required | Undo: the status before the cancel was `ongoing`. Any other target is refused (coordinator, 2026-11-16) | none |
| `cancelled` | `paused` | allowed | both | required | Undo: the status before the cancel was `paused`. Any other target is refused (coordinator, 2026-11-16) | none |
| `cancelled` | `cancelled` | refused | | | No-op | none |
| `cancelled` | `done` | refused | | | Only an undo leaves `cancelled`, and never to `done` | none |
| `done` | `planned` | refused | | | Terminal: nothing reopens done work, new work is a new node | none |
| `done` | `ongoing` | refused | | | Terminal | none |
| `done` | `paused` | refused | | | Terminal | none |
| `done` | `cancelled` | refused | | | Terminal | none |
| `done` | `done` | refused | | | No-op | none |

Rules that hold for every row:

- No move is human-only. `history[].actor` records who made it.
- A move presents the status it expects to leave. A mismatch is refused (R01 §Concurrency).
- A move writes one history entry and never an amendment (I5).
- A move into `done` or `cancelled` may carry the final `progress` in the same operation. After it, only comments are accepted, and from `cancelled` only the move to `planned`.
- The reason is required text, not blank, for `paused`, `cancelled`, `paused → planned`, `paused → ongoing` and every undo out of `cancelled`.

**`paused → planned` needs "no step is recorded done".** A step recorded done is sealed with its
commit, so a `planned` leaf holding a recorded commit would contradict "not started". A leaf with a recorded step goes to `ongoing`
instead. Decided 2026-11-16 (coordinator).

**`cancelled` is undoable.** A cancel is one call with a reason, and a directory cancel is one
call over many leaves, so a mistake must be reversible. Undoing a cancel restores the status the
node had just before the cancel, with a required reason, recorded in history. That status is read
from history: the `from` of the last cancel entry. It may be `planned`, `ongoing` or `paused`. Any
other target out of `cancelled` is refused. A leaf with no cancel entry in its history (one imported from legacy `blocked`) returns to `planned`, as the importer writes no steps done. Done work stays sealed: nothing leaves `done`.
Eliott's intent 2026-11-16 ("undo a mistaken cancel with a reason"); the restore rule is the
coordinator's ruling, 2026-11-16.

**Closed gap: a cancelled leaf with a recorded step.** The first form of the undo moved only to
`planned`, which a leaf with a done step cannot take, so such a leaf could never be resumed. The
restore rule fixes it: a leaf cancelled from `ongoing` or `paused` returns there.

**Note: a cancel releases work without saying so.** Because `cancelled` holds nothing back, a
cancelled leaf can release a deeper leaf that really needed it. That is Eliott's intent. The
impact report of a cancel (M2) should list the leaves it makes ready, so the release is never
silent.

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
| 3 | all finished, at least one `done` | `done` |
| 4 | at least one `ongoing` | `ongoing` |
| 5 | at least one `paused` | `paused` |
| 6 | at least one `done`, the others `planned` or `cancelled` | `ongoing` |
| 7 | otherwise (`planned`, with or without `cancelled`) | `planned` |

Row 3 reads a directory whose leaves are all finished as `done` even if some were cancelled. A
reader that must tell them apart uses the counts per status, which the read model carries.

## Sealing

A node is sealed when its content can no longer change. A sealed node accepts one change: an
append-only comment (a file `_comments/<ulid>.json` whose `node_id` names the node).

| Status | Content edit | Comment | Status move | Progress |
|:--|:--|:--|:--|:--|
| `planned` | allowed, amendment note required | allowed | per `Transitions` | allowed |
| `ongoing` | allowed except sealed steps, amendment note required | allowed | per `Transitions` | allowed |
| `paused` | allowed except sealed steps, amendment note required | allowed | per `Transitions` | allowed |
| `cancelled` | refused | allowed | only the undo, to the pre-cancel status | refused |
| `done` | refused | allowed | refused | refused |

- **Content** is every stored field except `status`, `status_reason`, `history`, `progress` and the state marks. The state marks are a criterion's `met`, and a step's `done`, `commit_sha` and `done_at`.
- **The amendment rule is uniform.** Any content edit to a `planned`, `ongoing` or `paused` node carries the amendment note, whatever the status. The stricter rule applies everywhere, as Eliott decided.
- **Status moves, progress, comments and state marks never produce an amendment (I5).** Setting `met` is a state change like a move, allowed until the leaf is `done` or `cancelled`.
- **A step recorded done inside an `ongoing` or `paused` leaf is sealed.** Its commit is on record, so its content (goal, logic, deliverables, checks, commit, references) cannot change and its `done` mark cannot be cleared. The other steps and the leaf's own fields stay editable. A sealed step cannot be removed or reordered.
- **Directories follow their derived status.** A directory (or the campaign root) whose derived status is `done` or `cancelled` is sealed: its own content (title, context, goal, criteria, references, gotchas, notes) cannot change, and it accepts comments. Adding or removing a child is a structure edit of the child, not a content edit of the directory, so it stays allowed with an amendment. A directory in any other derived status takes edits with an amendment note. Weak point: a new `planned` child moves the derived status off `done`, which unseals the directory. The amendment log still records both edits.
- **A cancelled node stays sealed while it is cancelled.** Undoing the cancel unseals it, in the status it had before. A `done` node never unseals.
- **Nothing reopens done work.** A new piece of work is a new node. There is no `done → ongoing`.
- **Gate marks do not gate `done`.** Real campaigns leave open (`⬜`) gates on most done leaves: 30 of 48 done leaves under `__roadmap__/` carry at least one. So the only precondition of `done` is I8, as in R01. Criteria `met` is information, not a check.
- **Structure edits on sealed nodes.** Deleting a `done` or `cancelled` leaf keeps its acknowledgement (I9) and its snapshot in the amendment. Moving a sealed leaf changes no content. Whether a move that changes the ready set of other leaves needs the acknowledgement is left to M2's impact analysis.

**Recorded objection (coordinator kept the default): a wrong step record is permanent.** A mistyped `commit_sha` on a sealed step cannot be
corrected. The remedy today is a comment on the leaf. If this bites, a human-only correction with
an amendment is the least invasive fix. Not decided here, since it weakens the seal.

## Legacy mapping

For the importer (M3). Counts are from `__roadmap__/` at the date of this document.

| Legacy status | New status | Reason and provenance | In real campaigns |
|:--|:--|:--|:--|
| `planned` | `planned` | Same meaning | 26 rows |
| `inprogress` | `ongoing` | Rename | 4 rows, all in campaigns being planned now |
| `done` | `done` | Same meaning. Legacy `done` was "immutable; reopen via a new amendment node", which is the sealing rule | 65 rows |
| `blocked` | `cancelled` | The legacy skill defined `blocked` as terminal within a campaign, never auto-resolved, which is how it was used to abandon work. The legacy file holds no reason, so the importer writes `status_reason: "imported from legacy status blocked"` and warns | 0 rows |
| `amendment` | `planned` | Legacy `amendment` was the initial status of a node added by an amendment, never a state an existing node moved to. The provenance stays in the amendment: the importer writes the `A<n>` row as an amendment (`legacy_id`) with a change `op: add` for each filename in its "Nodes Added" cell | 0 rows. The one node added by an amendment (`hub_mode_reconciliation.md`, A1) is `done` |

Legacy files carry no timestamps and no history of moves. The importer writes no history entries:
the status is set directly and `history` starts empty.

## R01 deltas

For `r01_sync`. R01 is not edited by this leaf.

| R01 location | Change |
|:--|:--|
| §Graph operations, Status FSM diagram | Replace with the five-status machine of `Transitions`: `planned → ongoing, paused, cancelled`; `ongoing → paused, cancelled, done`; `paused → planned, ongoing, cancelled`; `cancelled → planned, ongoing, paused` (only the pre-cancel status). Remove `blocked`, `amendment` and the `done → amendment` edge |
| §Graph operations, Order | "not done" becomes "`planned`"; "every leaf of smaller depth is done" becomes "`done` or `cancelled`". The ready rule becomes the scoped rule of `Ready set`. The sentence "Wave semantics are global: a leaf also waits for shallower leaves in unrelated branches" is replaced: a leaf waits only for shallower leaves under its own ancestors' parents. "Waves" stay a display of depth (`level`), not a readiness fact |
| §Graph operations, after the FSM | "Starting a leaf outside the ready set is refused unless an override with a reason is given" stays. Add the directory pause and cancel operation and the derivation table |
| §Graph operations, Impact analysis | Remove "done work reopened". Add "leaves made ready by a cancel" |
| §Contracts, I5 | Add: comments (separate append-only files) and state marks never produce an amendment. A sealed node accepts no content edit |
| §Contracts, I8 | Unchanged. Add: it is the only precondition of `done` |
| §Contracts, I9 | "Deleting or reopening done work, or moving work in progress, needs an explicit acknowledgement" becomes "Deleting sealed work, or moving work in progress, needs an explicit acknowledgement. Nothing reopens done work". Reopening is replaced by sealing |
| §Data Model, schema sketch | `blocked_reason?` becomes `status_reason?`; history `note?` becomes `reason?`; comments are not a node field: they are files `_comments/<ulid>.json` (details in stored-schema.md), so the §Data Model tree gains `_comments/` |
| §Concurrency, last bullet | "records `inprogress`" becomes "records `ongoing`" |
| §Graph operations, after the FSM | Add a directory `uncancel` through the `op_id` of the cancel |
| §Key Flows, Execute | `inprogress` becomes `ongoing` |
| §Alternatives Considered, Mutation policy | "full CRUD with guards on done work" is now "full CRUD, with done and cancelled nodes sealed" |

## Decisions

Nothing is open. Who decided what:

| # | Decision | Decided |
|:--|:--|:--|
| 1 | Ready rule: an unfinished leaf holds back only the deeper leaves under its own parent (legacy rule). Applies to `planned`, `ongoing` and `paused` alike | Eliott, 2026-11-16 |
| 2 | Comments are one file each, `_comments/<ulid>.json`, append-only | Eliott, 2026-11-16 |
| 3 | A mistaken cancel can be undone with a required reason; `done` stays terminal and sealed | Eliott, 2026-11-16 |
| 4 | `paused → planned` needs no step recorded done | coordinator, 2026-11-16 |
| 5 | A step recorded done is sealed, so a mistyped `commit_sha` is permanent. Recorded objection: a human-only correction with an amendment would be the least invasive fix | default kept, coordinator, 2026-11-16 |
| 6 | Undoing a cancel restores the status the node had just before the cancel, read from history, with a required reason: `planned`, `ongoing` or `paused`. A directory undo restores each leaf through the cancel's `op_id`. `done` stays the only terminal status. Replaces the earlier `cancelled → planned` only form and closes its gap (coordinator, 2026-11-16) | coordinator, 2026-11-16 |
