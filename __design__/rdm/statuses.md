# rdm — Status semantics

What each status of a leaf means, how it may change, and what it does to the order of work. The
vocabulary and the editing rules are Eliott's decisions of 2026-11-16 (M0b README, Gotchas). Where
this document goes beyond them, or disagrees with a default, the text says `OBJECTION` and Eliott
decides. One question is `OPEN`.

This document settles the meaning. The stored shape of the fields it names (`status`,
`status_reason`, `history`, `comments`) is in [stored-schema.md](stored-schema.md). R01
(`architecture.md`) is not edited here; section `R01 deltas` lists what `r01_sync` must change.

## Statuses

Only a leaf has a stored status. A directory's status is derived (see `Subtrees and directories`).

| Status | Meaning | Terminal | Sealed |
|:--|:--|:--|:--|
| `planned` | The leaf is written and not started. No step is recorded done. | no | no |
| `ongoing` | Work has started on the leaf. Steps may be recorded done. | no | steps recorded done only |
| `paused` | Work is held and can be resumed: a bug, a decision to wait, or a downstream branch held on purpose. | no | steps recorded done only |
| `cancelled` | The leaf is abandoned. It holds nothing back. Whatever was done stays on record. | yes | yes |
| `done` | Every step is done with its commit recorded. | yes | yes |

`blocked` and `amendment` are not statuses. A "blocked" situation is `paused` (resumable) or
`cancelled` (abandoned). An "amendment" is an edit recorded in the amendment log, and a node
added by one starts as `planned`.

A non-terminal leaf (`planned`, `ongoing`, `paused`) holds back deeper leaves. A terminal leaf
(`done`, `cancelled`) holds back nothing.

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
| `paused` | `planned` | allowed | both | required | No step is recorded done (OBJECTION 1: not in the default) | ENTER if nothing shallower holds it back |
| `paused` | `ongoing` | allowed | both | required | The leaf was `ongoing` before the pause (last pause entry has `from: ongoing`), or it is in the ready set. Otherwise an override with a reason | none |
| `paused` | `paused` | refused | | | No-op | none |
| `paused` | `cancelled` | allowed | both | required | none | RELEASE |
| `paused` | `done` | refused | | | Resume to `ongoing` first | none |
| `cancelled` | `planned` | refused | | | Terminal (OBJECTION 2) | none |
| `cancelled` | `ongoing` | refused | | | Terminal | none |
| `cancelled` | `paused` | refused | | | Terminal | none |
| `cancelled` | `cancelled` | refused | | | No-op | none |
| `cancelled` | `done` | refused | | | Terminal | none |
| `done` | `planned` | refused | | | Terminal: nothing reopens done work, new work is a new node | none |
| `done` | `ongoing` | refused | | | Terminal | none |
| `done` | `paused` | refused | | | Terminal | none |
| `done` | `cancelled` | refused | | | Terminal | none |
| `done` | `done` | refused | | | No-op | none |

Rules that hold for every row:

- No move is human-only. `history[].actor` records who made it.
- A move presents the status it expects to leave. A mismatch is refused (R01 §Concurrency).
- A move writes one history entry and never an amendment (I5).
- A move into `done` or `cancelled` may carry the final `progress` in the same operation. After it, only comments are accepted.
- The reason is required text, not blank, for `paused`, `cancelled`, `paused → planned` and `paused → ongoing`.

**OBJECTION 1: `paused → planned` needs "no step is recorded done".** The default allows the move
with a reason. A step recorded done is sealed with its commit, so a `planned` leaf holding a
recorded commit would contradict "not started". Default kept as stated, with this one added
precondition. Without it, the leaf must go to `ongoing` instead.

**OBJECTION 2: `cancelled` as terminal is a trap for a mistaken cancel.** The default makes
`cancelled` terminal, like `done`. But `done` is protected by a verified commit, while a cancel is
one call with a reason, and a directory cancel (below) is one call over many leaves. A wrong one
cannot be undone except by writing the leaves again as new nodes. Two cheaper options, neither
taken: a `human`-only `cancelled → planned` with a reason, allowed while no content was changed;
or `human`-only for the directory cancel. Default kept: terminal, both actors.

**Note: a cancel releases work without saying so.** Because `cancelled` holds nothing back, a
cancelled leaf can release a deeper leaf that really needed it. That is Eliott's intent. The
impact report of a cancel (M2) should list the leaves it makes ready, so the release is never
silent.

## Ready set

A leaf is ready when it is `planned` and every leaf that holds it back is `done` or `cancelled`.
So `cancelled` holds nothing back.

The depth of a leaf is the number of directories between the campaign root and the leaf. A leaf
directly under the campaign root has depth 0. The rule for which leaves hold a given leaf back
has two readings. It is `OPEN`.

| Reading | Leaves that hold leaf L back |
|:--|:--|
| A. Global (R01) | Every leaf of smaller depth, anywhere in the campaign |
| B. Scoped (legacy, `graph-model.md`) | Every leaf of smaller depth that is a leaf child of the parent of L or of any ancestor directory of L |

Under B, a leaf also waits for the leaf children that sit above its own directory, and so on up
to the campaign root. Leaves in an unrelated branch never hold it back.

Both readings use the same rule for the three non-terminal statuses: a `paused` leaf holds back
exactly what a `planned` or `ongoing` one does. The choice is about the ready rule as a whole, not
about `paused` alone. The status is not special: Eliott asked whether a halted upper leaf must
block everything downstream, and that is the same question for any leaf that is not finished.

An `ongoing` leaf is not in the ready set, because it is not `planned`. A leaf started by an
override (`planned → ongoing` outside the set) is recorded and allowed.

## Subtrees and directories

**Pausing or cancelling a directory** is one operation over its leaves.

- `pause` sets every `planned` and `ongoing` leaf under the directory to `paused`.
- `cancel` sets every `planned`, `ongoing` and `paused` leaf under it to `cancelled`.
- Leaves already terminal are left alone. A leaf where the move is refused is skipped, never an error for the whole operation.
- One reason, one timestamp and one `op_id` are shared by every history entry the operation writes. The operation is one call and one transaction, so it is "recorded once".
- It is a snapshot, not a standing flag. A leaf added to the directory later starts `planned`.
- `resume` of a directory sets each `paused` leaf under it to the `from` of its last pause entry (`planned` or `ongoing`). A leaf whose last pause came from another operation is resumed too, each by its own rule. The reason is required.

**A directory's status is derived on every read and never stored (I6).** It is computed from all
leaves under it, at any depth. The first matching row wins.

| # | Leaves under the directory | Derived status |
|:--|:--|:--|
| 1 | none | `planned` |
| 2 | all `cancelled` | `cancelled` |
| 3 | all terminal, at least one `done` | `done` |
| 4 | at least one `ongoing` | `ongoing` |
| 5 | at least one `paused` | `paused` |
| 6 | at least one `done`, the others `planned` or `cancelled` | `ongoing` |
| 7 | otherwise (`planned`, with or without `cancelled`) | `planned` |

Row 3 reads a directory whose leaves are all finished as `done` even if some were cancelled. A
reader that must tell them apart uses the counts per status, which the read model carries.

## Sealing

A node is sealed when its content can no longer change. A sealed node accepts one change: an
append-only comment (`comments[]`).

| Status | Content edit | Comment | Status move | Progress |
|:--|:--|:--|:--|:--|
| `planned` | allowed, amendment note required | allowed | per `Transitions` | allowed |
| `ongoing` | allowed except sealed steps, amendment note required | allowed | per `Transitions` | allowed |
| `paused` | allowed except sealed steps, amendment note required | allowed | per `Transitions` | allowed |
| `cancelled` | refused | allowed | refused | refused |
| `done` | refused | allowed | refused | refused |

- **Content** is every stored field except `status`, `status_reason`, `history`, `progress`, `comments` and the state marks. The state marks are a criterion's `met`, and a step's `done`, `commit_sha` and `done_at`.
- **The amendment rule is uniform.** Any content edit to a `planned`, `ongoing` or `paused` node carries the amendment note, whatever the status. The stricter rule applies everywhere, as Eliott decided.
- **Status moves, progress, comments and state marks never produce an amendment (I5).** Setting `met` is a state change like a move, allowed until the leaf is terminal.
- **A step recorded done inside an `ongoing` or `paused` leaf is sealed.** Its commit is on record, so its content (goal, logic, deliverables, checks, commit, references) cannot change and its `done` mark cannot be cleared. The other steps and the leaf's own fields stay editable. A sealed step cannot be removed or reordered.
- **Directories follow their derived status.** A directory (or the campaign root) whose derived status is `done` or `cancelled` is sealed: its own content (title, context, goal, criteria, references, gotchas, notes) cannot change, and it accepts comments. Adding or removing a child is a structure edit of the child, not a content edit of the directory, so it stays allowed with an amendment. A directory in any other derived status takes edits with an amendment note. Weak point: a new `planned` child moves the derived status off `done`, which unseals the directory. The amendment log still records both edits.
- **Nothing reopens done work.** A new piece of work is a new node. There is no `done → ongoing`.
- **Gate marks do not gate `done`.** Real campaigns leave open (`⬜`) gates on most done leaves: 30 of 48 done leaves under `__roadmap__/` carry at least one. So the only precondition of `done` is I8, as in R01. Criteria `met` is information, not a check.
- **Structure edits on sealed nodes.** Deleting a `done` or `cancelled` leaf keeps its acknowledgement (I9) and its snapshot in the amendment. Moving a sealed leaf changes no content. Whether a move that changes the ready set of other leaves needs the acknowledgement is left to M2's impact analysis.

**Risk: a wrong step record is permanent.** A mistyped `commit_sha` on a sealed step cannot be
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
| §Graph operations, Status FSM diagram | Replace with the five-status machine of `Transitions`: `planned → ongoing, paused, cancelled`; `ongoing → paused, cancelled, done`; `paused → planned, ongoing, cancelled`. Remove `blocked`, `amendment` and the `done → amendment` edge |
| §Graph operations, Order | "not done" becomes "`planned`"; "every leaf of smaller depth is done" becomes "`done` or `cancelled`". The sentence "Wave semantics are global" stands or changes with Eliott's answer in `Open for Eliott` |
| §Graph operations, after the FSM | "Starting a leaf outside the ready set is refused unless an override with a reason is given" stays. Add the directory pause and cancel operation and the derivation table |
| §Graph operations, Impact analysis | Remove "done work reopened". Add "leaves made ready by a cancel" |
| §Contracts, I5 | Add: comments and state marks never produce an amendment. A sealed node accepts no content edit |
| §Contracts, I8 | Unchanged. Add: it is the only precondition of `done` |
| §Contracts, I9 | "Deleting or reopening done work, or moving work in progress, needs an explicit acknowledgement" becomes "Deleting sealed work, or moving work in progress, needs an explicit acknowledgement. Nothing reopens done work". Reopening is replaced by sealing |
| §Data Model, schema sketch | `blocked_reason?` becomes `status_reason?`; history `note?` becomes `reason?`; add `comments[]` (details in stored-schema.md) |
| §Concurrency, last bullet | "records `inprogress`" becomes "records `ongoing`" |
| §Key Flows, Execute | `inprogress` becomes `ongoing` |
| §Alternatives Considered, Mutation policy | "full CRUD with guards on done work" is now "full CRUD, with done and cancelled nodes sealed" |

## Open for Eliott

### OPEN: how far does a held leaf hold back deeper leaves?

Whether a leaf that is not finished (`paused`, and equally `planned` or `ongoing`) holds back every
deeper leaf in the campaign (A, R01's rule) or only those that sit under its own parent (B, the
legacy rule). See `Ready set` for the exact rule.

**What the real meta-roadmap shows.** `__roadmap__/roadmap-mcp-rework/` is a chain, not a tree:

| Depth | Leaves |
|:--|:--|
| 0 | M0, M0b |
| 1 | M1 (`core/`) |
| 2 | M2 (`core/engine/`) |
| 3 | M3 (`core/engine/interchange/`) |
| 4 | M4, M5 (`.../consumers/`) |
| 5 | M6, M7 (`.../consumers/delivery/`) |
| 6 | M8 (`.../delivery/cutover/`) |

Every deeper leaf sits under the parent of every shallower one, so A and B agree on every leaf
today. The readings only differ once a directory has a second branch. Example, per reading:

- **A. Global.** M5 is paused for a bug. M6, M7 and M8 are all held back. M6 needs only M4 (the meta-roadmap Gotcha: "M6 needs only M4 but also waits on M5"), yet it waits.
- **B. Scoped.** M5 is paused. Its parent is `consumers/`, so the leaves held back are those in `consumers/delivery/` and `consumers/delivery/cutover/`: M6, M7, M8. Same set as A here. The difference needs a hypothetical branch. Add a directory `docs/guide/` at the campaign root with a leaf at depth 2, next to `core/`. Pause M1 (depth 1, parent `core/`). Under A the docs leaf waits for M1. Under B it does not, because it is not under `core/`.

**Recommendation: B.**

- It is what the legacy skill did, so the imported campaigns keep the readiness they had. Under A, independent branches become sequenced on import.
- It answers Eliott's remark that a halted upper leaf need not block everything downstream, without a new status or an edge field. To hold a branch on purpose, pause the directory.
- It removes the M6-after-M5 over-sequencing at no cost to the "depth is the only ordering rule" decision.
- It costs a more careful ready computation (ancestors, not a global depth index) and the "wave" of the view becomes a display of `level`, not a readiness fact. R01's sentence "Wave semantics are global" changes, and I1 stays true.

A mixed reading (global for `planned` and `ongoing`, scoped for `paused` only) is possible and
rejected here: it gives two ordering rules for one tree.

### Objections raised against the defaults

| # | Default | Objection | Taken |
|:--|:--|:--|:--|
| 1 | `paused → planned` allowed with a reason | Add: no step recorded done, or `planned` contradicts a sealed commit | Added to `Transitions` |
| 2 | `cancelled` terminal | A mistaken cancel, above all a directory cancel, cannot be undone. Options in the objection text | Default kept, Eliott decides |
| 3 | A step recorded done is sealed (recommended: yes) | A mistyped commit record is permanent | Default kept, risk noted in `Sealing` |
