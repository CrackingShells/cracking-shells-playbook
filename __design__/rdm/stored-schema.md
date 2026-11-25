# rdm — Stored schema

Every fact `rdm` stores, with its name, type, whether it is required, and its meaning. This is the
settled form of R01's provisional schema sketch (`architecture.md` §Data Model). The legacy
grammars it must cover are traced production by production in [traceability.md](traceability.md).
Status meanings are in [statuses.md](statuses.md). The read model that consumers receive is a
separate document (`read-model.md`).

Where this document departs from a default of the M0b plan, a one-line note says who decided it
(Eliott or the coordinator, 2026-11-16). Two recorded objections stand, with the default kept.
Fields new against R01 are marked **new**.

**Revision, 2026-11-25 (A2, Eliott's rulings).** The layout is flat and id-addressed, a node holds
its own `name`, `parent_id` and `order`, and the Rust types in [contracts.md](contracts.md) are the
contract this document describes. Checks and references lose their ids (`Stable identity`), and the
id types, the id string format and the order key are named in contracts.md and
[store.md](store.md). Edited sections carry a "Revised 2026-11-25" note.

Notation:

| Type | Meaning |
|:--|:--|
| `string` | One line, no line break |
| `lines` | Array of strings, each one line. Used for all multi-line prose, so git merges it line by line. The API accepts and returns plain strings and splits or joins on `\n` |
| `ts` | UTC instant, RFC 3339, `"2026-11-16T10:20:30Z"` |
| `ulid` | A ULID, 26 characters of Crockford base32 |
| `T[]` | Array of `T` |
| `T?` | Optional: the key is absent when there is no value. A writer never writes `null`. A reader treats `null` as absent. The exception is `Change.old` and `Change.new`, where `null` is a value |

## Files

```
__roadmap__/<campaign>/
  nodes/<id>.json             one file per node          (Directory or Leaf)
  _amendments/<ulid>.json     one file per amendment     (Amendment)
  _comments/<ulid>.json       one file per comment       (Comment)
  .rdm/                       untracked: lock, intent/, cache/ (store.md §Layout)
```

Revised 2026-11-25: the files no longer mirror the tree. A node is `nodes/<id>.json`, whatever its
kind, and its place in the tree is data inside it (`name`, `parent_id`, `order`). The campaign root
is the node whose `kind` is `campaign`. A rename or a move writes one file and renames no directory.

| Rule | Detail |
|:--|:--|
| Names are paths | A node's `name` is its path segment, `^[a-z][a-z0-9_-]*$`, unique among the children of one parent. It is stored in the node file (revised 2026-11-25: it used to be the file's own name). The path is derived from the `parent_id` chain and never stored. The campaign root has no name |
| Identity | Every node, criterion and step has an immutable `id`, typed `NodeId`, `CriterionId` and `StepId` in [contracts.md](contracts.md) §Id types. The string format and the minting are [store.md](store.md)'s §Ids. Checks and references have none: `Stable identity` |
| Version stamp | Every file carries `schema_version` (see `Format version`) |
| Unknown fields | Every object type below keeps the keys it does not know, in order, and writes them back after the known ones. This holds for nested objects too (steps, criteria, references, commits, history and comment entries, changes). It is the stored half of I12. Keys keep their order, so the crate enables `serde_json`'s `preserve_order`. Numbers outside the `i64`, `u64` and `f64` range are not preserved exactly, because `arbitrary_precision` is not enabled. `Envelope` (read model) has no `extra` of its own: it is flattened into each view, whose `extra` keeps the unknown keys |
| Canonical order | The order of the rows in each field table is the canonical field order. Two-space indent, LF, trailing newline. M1 owns the serialiser |
| Derived, never stored (I6) | Directory status, the nodes table, the mermaid graph, the progress table, waves, the ready set, a step's number, the display id of an amendment, a leaf's commit count when the engine recorded the steps, a node's children, depth and path, and a `Reference.target` when `Reference.node_id` is set. No stored field holds a value that can be derived from an id |

## Stable identity

Eliott, 2026-11-16: the data structures must be aligned so that mutations cannot break references.
If reordering criteria made a mark record point at the wrong criterion, the data model would be
wrong.

**Invariant.** No record refers to a sub-item by position, and every mutation (insert, delete,
reorder, move) leaves existing references valid.

| Sub-item | Id field | Pointed at by |
|:--|:--|:--|
| Criterion (`preconditions[]`, `success_gates[]`) | `Criterion.id` | `Mark.criterion_id`, `Change.item_id` |
| Step | `Step.id` | `Change.item_id` (an edit or a correction of the step) |

Revised 2026-11-25. The rule for an id is "it is pointed at from outside, and its natural key can
change" ([contracts.md](contracts.md) §Id inventory). A check and a reference are pointed at by no
record, so they have no id. Their edits are recorded at the array level: a `set` change on the
owner's `/checks` or `/references`, with the whole old and new array. A reference that names a
node points *out* through `node_id`; nothing points in.

- **Minting.** An id is minted when the sub-item is created, by the engine, or by the importer at import, with the format and the minter of [store.md](store.md) §Ids. It is unique within its node file, and is immutable.
- **Never reused, never derived.** An id is not computed from position, text or a counter, and the id of a deleted sub-item is never given to another. A delete keeps the item in its amendment's `snapshot`, so a record that still names the id resolves there.
- **Order is separate.** The order of a sub-item is the order of its array, and a reorder changes that order only. Nothing stored holds an order as an identity. A step's `number` is derived from the array for display, and is never a reference.
- **No records point by position.** A record that names a sub-item carries its id: `Mark` uses `criterion_id`, and `Change` uses `item_id` with a `field` pointer relative to that item. A node is named by `node_id`, never by its path.
- **Append-only logs** (`history`, `marks`, comments, amendments) are never reordered or edited, and their entries are not pointed at by position. The one reading of order, "the last cancel entry", reads the end of an append-only log.
- **Not addressable.** `Commit` and `Progress` are single objects reached through their step or leaf, so they have no id. Neither have a check, a reference, a history entry or a mark.

**Audit of positional and path-based references**, fixed the same way:

| Was | Broke on | Now |
|:--|:--|:--|
| `Mark.index` and `Mark.kind` | reorder, or a move of a criterion between `preconditions` and `success_gates` | `Mark.criterion_id`; the kind is derived by looking up the criterion |
| `Change.field` as a JSON pointer with an array index (`/steps/1/goal`) | insert, delete or reorder of steps and criteria | `Change.item_id` names the sub-item, and `field` is a pointer inside it (`/goal`) |
| `Amendment.snapshot` keyed by path | a move or rename of the removed subtree's nodes | keyed by node id |
| `Reference.target`, a relative path, when it names a node of the campaign | a move of the node | optional `Reference.node_id`; when it is set `target` is absent and derived on read (revised 2026-11-25) |

## Directory

`_dir.json`, for the campaign root and for every directory node. A directory has no `status`: it
is derived (`statuses.md`, `Subtrees and directories`).

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `schema_version` | `string` | yes | Format version, `"0.X.Y"` |
| `id` | `NodeId` | yes | Immutable node id |
| `kind` | `"campaign"` or `"directory"` | yes | `campaign` for the root only, which is the node with no `parent_id` |
| `name` | `Name?` | no | **new**: the path segment. Absent exactly for the campaign root. Unique among the children of one parent |
| `parent_id` | `NodeId?` | no | **new**: the directory this node is in. Absent exactly for the campaign root |
| `order` | `OrderKey?` | no | **new**: position among siblings, an opaque key (store.md §Order key). Absent exactly for the campaign root |
| `title` | `string` | yes | The H1 of the legacy README |
| `context` | `lines` | yes | Where the node fits; legacy `## Context`, one or more lines |
| `references` | `Reference[]` | yes | Legacy `## Reference Documents`. May be empty (the section is optional in the legacy grammar) |
| `goal` | `string` | yes | One line. The legacy grammar allows exactly one line |
| `preconditions` | `Criterion[]` | yes | Entry criteria, one or more |
| `success_gates` | `Criterion[]` | yes | Completion criteria, one or more |
| `gotchas` | `lines` | yes | Legacy `## Gotchas`, verbatim markdown lines. May be empty (the section is optional) |
| `notes` | `lines` | yes | **new**: the `Notes` cell of the directory's own row in its parent's Progress table. May be empty |
| `marks` | `Mark[]` | yes | **new**: the trace of every change of a criterion mark of this directory, oldest first. Empty on a new or imported directory |

Rules for `validate`: `goal` has no line break; `preconditions` and `success_gates` each have at
least one entry; `kind` is `campaign` exactly when `parent_id`, `name` and `order` are all absent
(revised 2026-11-25: it used to be "the file is at the campaign root"). A directory's children are
the nodes whose `parent_id` names it, ordered by `order`; the list is derived and never stored.

A directory carries no branch and no commit count. The legacy Progress row of a directory repeats
the branch of one of its leaves and a count of commits; both are dropped (see `traceability.md`,
`Stretched legacy data`).

## Leaf

`<name>.json`.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `schema_version` | `string` | yes | Format version, `"0.X.Y"` |
| `id` | `NodeId` | yes | Immutable node id |
| `kind` | `"leaf"` | yes | Discriminates a leaf from a directory when a reader holds only the JSON |
| `name` | `Name` | yes | **new**: the path segment. Unique among the children of one parent |
| `parent_id` | `NodeId` | yes | **new**: the directory this leaf is in |
| `order` | `OrderKey` | yes | **new**: position among siblings, opaque (store.md §Order key). Ties break by `id` |
| `title` | `string` | yes | The H1 of the legacy leaf file |
| `status` | `"planned"`, `"ongoing"`, `"paused"`, `"cancelled"` or `"done"` | yes | Meanings and moves in `statuses.md`. A new leaf is `planned` |
| `status_reason` | `string?` | no | **new** (replaces R01's `blocked_reason`). The reason of the move into the current status. Present only while the status is `paused` or `cancelled` |
| `goal` | `string` | yes | One line |
| `preconditions` | `Criterion[]` | yes | Entry criteria, one or more |
| `success_gates` | `Criterion[]` | yes | Completion criteria, one or more |
| `references` | `Reference[]` | yes | Supporting documents of the leaf |
| `steps` | `Step[]` | yes | One to five steps, in order |
| `progress` | `Progress` | yes | Where the work happened |
| `history` | `History[]` | yes | Status moves, oldest first. Empty on a new leaf, and on a leaf imported from legacy markdown |
| `marks` | `Mark[]` | yes | **new**: the trace of every change of a criterion mark of this leaf, oldest first. Empty on a new or imported leaf. Kept apart from `history` so that the last `history` entry stays the last status move |

`Progress` is an object, not a separate type section:

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `branch` | `string?` | no | The task branch, for example `task/write-skill-md`. Absent when the legacy cell is `-`, `--`, `---` or `—` |
| `commits` | `integer?` | no | **new**: a commit count written by the importer for a leaf whose steps carry no `commit_sha`. The legacy count can exceed the step count (a fixup commit). The engine never writes it: for an engine-recorded leaf the count is derived from the steps |
| `notes` | `lines` | yes | The legacy `Notes` cell. May be empty |

Rules for `validate`:

- `steps` has one to five entries. A step's number is its position plus one, so no rule can break it.
- `goal` has no line break.
- `references` has at least one entry (the legacy grammar requires a non-empty `**References**` line). An empty array is a warning, not an error. Decided 2026-11-16 (coordinator).
- `status == planned` implies every step has `done: false`.
- `status == done` implies every step has `done: true`.
- `status_reason` is present exactly when `status` is `paused` or `cancelled`, and equals the `reason` of the last `history` entry when `history` is not empty.
- A leaf imported as `done` has steps with `done: true` and no `commit_sha`. I8 is checked when a move to `done` is made (see `Changes against R01`).
- A `done` leaf with a criterion that is not `met` is an **error to reconcile** (`marks`, `Criterion`). The importer never ticks a criterion, so every legacy done leaf with an open gate reports it. The same holds for a directory whose derived status is `done`.
- For each criterion, when `marks` holds a record for it, `met` equals the `met` of its last record. A legacy tick that has no record is allowed.
- Every `id` of a sub-item is unique within the file. A `Mark.criterion_id` names a criterion of the file, or a criterion that an amendment removed (its snapshot holds it).
- A record with `correction: true` has a non-blank reason: the `reason` of a mark or history entry, the `description` of an amendment. A history entry with `correction: true` has `from: done`.

## Step

An element of `Leaf.steps`.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `id` | `StepId` | yes | **new** (R01 had it): immutable step id, minted like node ids (`Stable identity`). Anything that points into a step uses it |
| `title` | `string` | yes | The text after `## Step N:` |
| `goal` | `string` | yes | One line: the unique change intent of the step |
| `implementation_logic` | `lines` | yes | Free prose and lists, blank lines allowed. At least one line is not blank |
| `deliverables` | `lines` | yes | Legacy: one line. Import stores it as one element, unsplit. A new step may hold several elements, one per deliverable. M3's export joins them with `; ` |
| `references` | `Reference[]` | yes | Step-level documents. May be empty (optional in the legacy grammar) |
| `checks` | `Check[]` | yes | One or more. The legacy grammar has exactly one per step, so import yields one |
| `commit` | `Commit` | yes | The commit message the step must produce |
| `requires` | `lines?` | no | Prose prerequisites that are not obvious. **Never an id or an edge**: I1 forbids a stored dependency between siblings. Never used in real campaigns, kept so the grammar's field has a home |
| `done` | `bool` | yes | The step is recorded done. A correction of a step recorded done needs a reason (`statuses.md`, `Finished nodes`) |
| `commit_sha` | `string?` | no | The commit that completed the step. Set with `done: true` by the engine |
| `done_at` | `ts?` | no | **new**: when the step was recorded done |

`Check` (revised 2026-11-25: no `id`; nothing points at a check, and an edit is a `set` of the step's `/checks`):

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `text` | `string` | yes | **new**: the check as written, without the `(expected: …)` trailer. Prose or a command line |
| `expected` | `"PASS"` or `"FAIL"` | yes | The outcome the check expects |
| `command` | `string?` | no | A shell command, when the whole check is one runnable command. Set only by the author of a new check, never guessed by the importer |

**A step has an `id`, and its number is derived.** The number is the step's position plus one, 1
to 5, never stored. A stored number could disagree with the array position, and R01's composite
"reorder steps" would have to renumber it, so references into a step by number would point at
another step. The export writes the number from the position. In `traceability.md` the
`<step-heading>` row stores `title` and derives the number. Decided 2026-11-16 (coordinator).

**A check is prose, not `{command, expected}`.** The plan's default made `command`
required. The legacy check is a prose line that ends in `(expected: PASS)` or `(expected: FAIL)`.
Real checks are commands in backticks, several commands joined by `;`, shell loops, or a
sentence. No parser extracts a `command` from all of them without loss or invention. Decided
2026-11-16 (coordinator): `text` is required and lossless, `expected` is parsed from the trailer (the one part
the grammar fixes), and `command` is optional and filled only by an author. Export (M3) writes
`text`, a space, and `(expected: X)`. An agent that wants to run a check uses `command` when it
exists and reads `text` otherwise.

Rules for `validate`: `title` is not blank; `implementation_logic` has a non-blank line; a step
with `done: true` has `done_at` when the engine recorded it; `commit_sha` and `done_at` appear
only when `done` is `true`.

## Criterion

An element of `preconditions` or `success_gates`, at leaf and directory level.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `id` | `CriterionId` | yes | **new**: immutable criterion id (`Stable identity`). A `Mark` points at it, never at the criterion's position |
| `text` | `string` | yes | The criterion, one line. Tags such as `[static]` or `[behavioral]` stay in the text |
| `met` | `bool` | yes | Pre-condition: `[x]` is `true`. Gate: `✅` is `true`. A new criterion is `false` |

`met` is a state mark, not content: setting it produces no amendment and is allowed on every node, with a reason on a finished one (`statuses.md`, `Finished nodes`). It is enforced (Eliott, 2026-11-16): `planned → ongoing` needs every
pre-condition `met`, `ongoing → done` needs every success gate `met`, and a directory is `done`
only when its leaves are finished and its own gates are `met` (`statuses.md`, `Criteria`). Every
change of `met` writes a `Mark`. Real campaigns hold 16 checked pre-conditions and 150 `✅` gate
marks, and 30 of 48 done leaves carry an open gate: the importer imports them as they are, and
`validate` reports each `done` node with an unmet criterion as an error to reconcile.

## Mark

An element of `Leaf.marks` or `Directory.marks`: one change of one criterion's `met`. Written by
the operation that sets the mark; never an amendment (I5).

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `ts` | `ts` | yes | When the mark was changed |
| `actor` | `Actor` | yes | Who changed it |
| `criterion_id` | `CriterionId` | yes | The `id` of the criterion, in `preconditions` or `success_gates`. Never a position |
| `met` | `bool` | yes | The new value |
| `evidence` | `string?` | no | Free text: a command, its result, a commit |
| `reason` | `string?` | no | **new** (Eliott, 2026-11-16): why the mark was made. Required, non-blank, when `correction` is `true` |
| `correction` | `bool?` | no | **new**: `true` when the mark was made on a `done` or `cancelled` node. Counted in `correction_count` (`read-model.md`) |

A wrong check stays traceable: correcting it writes a new record, and the old one stays. A
reorder, an insert or a delete of criteria changes no mark, because a mark names its criterion by
id.

## Reference

An element of `references`, at directory, leaf and step level. One shape for all three.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `label` | `string?` | no | `R01`, `R02`, … The label other text cites. Unique within one `references` array. The legacy directory section requires it, so `validate` requires it there |
| `title` | `string` | yes | The link text without the label |
| `target` | `string?` | no | A URL, or a path relative to the campaign root directory, as written. Absent for a reference written as plain text, and **absent whenever `node_id` is set**: it is derived on read from the path of that node. Not resolved by the schema. Revised 2026-11-25: the base was the node file, and a node file no longer sits where its node is in the tree, so a tree-relative path would break on a move. The importer (M3) rewrites a legacy path to this base |
| `note` | `string?` | no | The text after ` — `, one line |
| `node_id` | `NodeId?` | no | **new**: the id of the campaign node the reference names, when it names one. A path breaks when the node moves, the id does not. It excludes `target`: a reference holds one or the other, and a record that holds both is `Invalid` (rule `reference_target_derived`). The importer sets `node_id` when a legacy target resolves to an imported node. If that node is later deleted the reference dangles: a warning, never an error (contracts.md §Links) |

Legacy forms and how they split:

| Legacy text | `label` | `title` | `target` | `note` |
|:--|:--|:--|:--|:--|
| `- [R01 Architecture](../a.md) — the design` (directory) | `R01` | `Architecture` | `../a.md` | `the design` |
| `[R01 §Data Model](../a.md) — the sketch` (leaf, in a `; `-separated line) | `R01` | `§Data Model` | `../a.md` | `the sketch` |
| `[Gap analysis](g.md) — why` | none | `Gap analysis` | `g.md` | `why` |
| a segment that is not a link | none | the whole segment | none | none |

The importer splits a leaf's one-line `**References**` on `; ` outside brackets and parentheses.
A `**References**:` line followed by bullet lines (`- [text](target) — note`) is the bulleted form: each bullet is one `Reference`, split like a directory item.
A label is the prefix `R` and two digits followed by a space at the start of the link text. A
segment it cannot parse becomes a reference with only `title`, so nothing is dropped.
Note (revised 2026-11-25): a `target` relative to the campaign root does not depend on where a node
sits in the tree, so a move leaves it valid and M2 need not normalise it.

## Commit

The commit message a step must produce. Field of `Step.commit`.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `type` | `string` | yes | Project-defined (`docs`, `feat`, …). The legacy shape is `[^\s()`!:]+` |
| `scope` | `string?` | no | Project-defined. **The legacy grammar requires a scope**; the default makes it optional |
| `breaking` | `bool` | yes | **new**: the `!` mark. `false` when absent |
| `summary` | `string` | yes | The text after `: `. May contain colons |

The backticks around the legacy line are presentation and are dropped. The message has no body or
trailer; the actor adds trailers at commit time.

Consequence for M3: a commit with no `scope` cannot be exported to a line the legacy validator
accepts. Export either writes `type: summary`, which the old tool rejects, or refuses. M3 decides.

## History

An element of `Leaf.history`: one status move. Written by `transition`; never an amendment (I5).

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `ts` | `ts` | yes | When the move was made |
| `from` | status | yes | The status left |
| `to` | status | yes | The status entered |
| `actor` | `Actor` | yes | Who made it |
| `reason` | `string?` | no | Required for moves into `paused` and `cancelled`, and for `paused → planned`, `paused → ongoing` and every undo out of `cancelled` (`statuses.md`) |
| `override` | `bool?` | no | `true` when the leaf was started outside the ready set |
| `op_id` | `OpId?` | no | **new**: shared by every entry that one operation writes, so a directory pause or cancel is recognisable as one operation |
| `correction` | `bool?` | no | **new** (Eliott, 2026-11-16): `true` for a reversal out of `done`. `reason` is then required and non-blank |

`Actor` (also used by `Comment` and `Amendment`):

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `kind` | `"agent"`, `"human"` or `"import"` | yes | The actor kind that `statuses.md` names in its `Who` column |
| `name` | `string?` | no | Free identifier: a user, an agent session, a tool |

**`actor` is an object, not a string.** The plan's list gave `actor` with no type,
and R01 gives none. `statuses.md` states who may set each move, `agent`, `human` or `both`. A
bare string cannot be checked against that. `kind` is the part a rule can test, and `name` is the
part a person reads. Decided 2026-11-16 (coordinator).

## Comment

`_comments/<ulid>.json`, at the campaign root. One file per comment, append-only, like an
amendment. A comment on a leaf, a directory or the campaign root. Allowed on every node, finished or not, with no reason. Never an amendment. A node's comments are the set of comment files whose `node_id` names
it, so a comment never changes the node's `rev`. Decided 2026-11-16 (Eliott): this replaces a
`comments[]` array in the node file, which two branches would both append to, and conflict.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `schema_version` | `string` | yes | Format version, `"0.X.Y"` |
| `id` | `CommentId` (`ulid`) | yes | Identity and order |
| `node_id` | `NodeId` | yes | The node commented on |
| `ts` | `ts` | yes | When it was added |
| `actor` | `Actor` | yes | Who added it |
| `text` | `lines` | yes | The comment, one or more lines (it is prose, so it follows the lines rule) |
| `op_id` | `OpId?` | no | The retry key of the write that made it |

Files are never edited or removed. A correction is a new comment. Deleting a node does not delete
its comments; the removing amendment's `snapshot` keeps them with it.

## Amendment

`_amendments/<ulid>.json`. One file per content or structure edit. Append-only.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `schema_version` | `string` | yes | Format version, `"0.X.Y"` |
| `id` | `AmendmentId` (`ulid`) | yes | Identity and order. The display id is derived (below) |
| `legacy_id` | `string?` | no | **new**: the legacy `A1`, `A2`, … of an imported amendment |
| `ts` | `ts` | yes | When it was made. For a legacy row: the row's day at `T00:00:00Z` |
| `actor` | `Actor` | yes | Who made it |
| `description` | `string` | yes | Non-empty (I5). The legacy `Rationale` |
| `source` | `string?` | no | The report that justified it. Legacy: a path under `__reports__/` |
| `op_id` | `OpId?` | no | The retry key of the write that made it |
| `correction` | `bool?` | no | **new** (Eliott, 2026-11-16): `true` for an edit of a finished node, of a step recorded done, or of progress on a finished node. `description` is then its reason and is non-blank |
| `changes` | `Change[]` | yes | What changed, one or more |
| `snapshot` | `object?` | no | The removed subtree of a delete, as stored JSON keyed by `NodeId`, so it can be restored. The keys are links (contracts.md §Links); the values are historical copies and are not scanned. A removed sub-item is not here: its `remove` change holds it in `old` |

`Change`:

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `node_id` | `NodeId` | yes | The node the change names. A node's amendment log is the set of amendments whose `changes` name its id |
| `op` | `"add"`, `"remove"`, `"set"`, `"rename"` or `"move"` | yes | R01's primitives. Composite operations expand to these. M2 owns any addition. With `item_id`, `add` and `remove` insert or delete the sub-item (a `remove` holds the removed sub-item in `old`, so a mark that names it resolves), and `move` reorders it: `old` and `new` hold its array position before and after, which is order data and not a reference. Without `item_id`, on a node: `add` and `remove` create and delete the node (a `remove` keeps the subtree in `snapshot`), `rename` holds the old and new `name` in `old` and `new`, and `move` holds `{"parent_id", "order"}` in `old` and `new` |
| `item_id` | `ItemId?` | no | **new**: the id of the sub-item (a criterion or a step) the change names. Absent for a change of the node itself. A check or a reference has no id: its edit is a `set` with `item_id` the step (or none, for the node) and `field` `/checks` or `/references`, `old` and `new` the whole arrays |
| `field` | `string?` | no | A JSON pointer for `set`: into the node, or, with `item_id`, into that sub-item (`/commit_sha` of a step). Never an array index of a sub-item array |
| `old` | JSON value? | no | The value before, for `set` |
| `new` | JSON value? | no | The value after, for `set` |

**Display id.** A sequential id (`A1`, `A2`) collides when two branches mint the next number. The
display id is derived on read: `A-` followed by the shortest suffix of the ULID that is unique in
the campaign, at least 6 characters (`A-7QX3NM`). An imported amendment shows its `legacy_id`
instead (`A1`). The display id is never stored.

**Legacy rows.** A legacy log row `| A1 | 2026-11-16 | <source> | ["x.md"] | <why> |` becomes one
amendment: `legacy_id: "A1"`, `ts` from the date, `actor.kind: "import"`, `source`, `description`
from the rationale, and one `{node_id, op: "add"}` change per filename in `Nodes Added`, resolved
to node ids after the nodes are imported. The cell is a JSON-like list, written as `["a.md"]` or
`[ "a.md" ]`; the importer reads both. A legacy log sits in the README of the directory that added
the nodes; the amendment is stored once, at the campaign root, and `changes` name the nodes.

## Format version

| Item | Decision |
|:--|:--|
| Field | `schema_version`, a string `"0.X.Y"`, in every file |
| First value | `"0.1.0"`. The crate stays at `0.x` until M7, so the format does too. Revised 2026-11-25: `0.1.0` is redefined by [contracts.md](contracts.md) (flat nodes, no ids on checks and references); nothing was stored in the old shape, so no layer reads it |
| Constant | `FORMAT_VERSION` in `rdm-schema` |
| Bump X (`0.X.y`) | A breaking change while the major is `0`: a field removed, renamed or given another meaning, or a field made required |
| Bump Y (`0.x.Y`) | A compatible addition or fix: a new optional field, or a new enum value that old readers can ignore |
| Reader | Reads the `schema_version` first. A file newer than the binary (a higher major, or at major 0 a higher X) is refused with `NewerFormat { found, supported }` before anything else of it is parsed. It reads a higher Y and keeps unknown fields (I12), and reads an older file through the layer of its version (`Layer`) |
| Layer | One `FormatLayer` per format line `(major, minor)`, each overriding only what changed; a behaviour a version lacks is `NotInVersion`. The chain, the reading order and the write-version rule are in contracts.md §Version chain |
| Writer | An edit rewrites the files it touches in the current version, stamped `FORMAT_VERSION`; a read never writes. One campaign may hold files of different versions |
| Where bumps come from | The commit messages (see `Release`): the crate's version is decided from commit types and scopes by the repository's existing multi-semantic-release setup, not by hand |
| Relation to the crate | A breaking format change forces a crate X bump, a compatible one at least a Y bump. The read-model version is a separate constant, in `read-model.md` |

### Release

The crate's version is decided from commit types and scopes by `@semantic-release/commit-analyzer`
(`release-config.js`, `.github/workflows/release.yml`, which release `skills/**` only today). A
`!` or BREAKING commit gives X and `feat` or `fix` gives Y, while the crate stays `0.x`. Wiring
the crate into that release job, with a rule that keeps a breaking change at `0.x` (semantic-release
goes to 1.0.0 by default), is M7's job. No CI check is added for versions.

## Changes against R01

| Area | R01 sketch | This schema |
|:--|:--|:--|
| Criteria | `preconditions[]`, `success_gates[]` as plain text | `Criterion {id, text, met}`, at leaf and directory level, enforced, with `Mark` records in `marks` that point by `criterion_id` |
| Sub-item ids | `id` on a step only | an `id` on criteria and steps only, and no record pointing by position. Checks and references have none (`Stable identity`, revised 2026-11-25) |
| Layout and place | one file per node in a directory tree; the name is the path segment | `nodes/<id>.json`; `name`, `parent_id` and `order` in the file; children, depth and path derived (A2) |
| Corrections | none | `correction?` on `Amendment`, `Mark` and `History`, with a reason; the count is derived |
| Step | `{id, goal, implementation_logic, deliverables[], checks[], commit, references[], requires[], done, commit_sha?}` | adds `title`, `done_at?`; `implementation_logic` and `deliverables` are `lines`; `requires` is optional prose; `id` is kept and the number is derived |
| Check | `{command, expected}` | `{text, expected, command?}` |
| Commit | `{type, scope, summary}` | `{type, scope?, breaking, summary}` |
| Reference | `references[]`, no shape | `{label?, title, target?, note?, node_id?}`; `target` is absent when `node_id` is set |
| Status | `blocked_reason?` | `status_reason?`; vocabulary per `statuses.md` |
| History | `{ts, from, to, actor, note?, override?}` | `note` is `reason`; adds `op_id?`; `actor` is an object |
| Comments | none | one file each, `_comments/<ulid>.json`, naming the node |
| Amendment | `{… id, …}` | adds `legacy_id?`; the id is a ULID with a derived display id. `Change` drops `path_before` and `path_after` (a path is derived from ids and goes stale on a move; a rename or a move holds its old and new `name` or `parent_id` and `order`) and `item_id` names a criterion or a step only (revised 2026-11-25) |
| Directory | no notes | adds `notes` |
| Progress | `{branch?, notes}` | adds `commits?` (import only); `notes` is `lines` |
| Gotchas, context | strings or lines unstated | `lines` |
| Unknown fields | I12 stated | Every type keeps them, nested ones too |
| Format version | "every file carries `schema_version`" | `"0.X.Y"`, first value `"0.1.0"`, reader rules above; a version chain of layers (A2) |

**Finding against R01 (invariant I8).** R01 says a leaf is `done` only when every step is done
with its commit recorded. A leaf imported from legacy markdown is `done` and has no `commit_sha`
on any step: the grammar never stored one. If I8 were checked on stored state, every imported done
leaf would fail. Resolution: I8 is checked when the move to `done` is made, and the stored-state
rule is "`done` leaf has every step `done`". The import records the legacy count in
`progress.commits`. `r01_sync` rewrites I8 to say so. This was not in `statuses.md`, whose `R01
deltas` row for I8 says "unchanged"; this document supersedes that row.

**Recorded objection (coordinator kept the default): `status_reason` is a copy of the last history reason.** It is derived data stored
beside its source, which I6 forbids. The default keeps it because an imported `blocked → cancelled`
leaf has no history entry to hold the reason. Alternative: the importer writes one `history` entry
with `actor.kind: "import"` and `status_reason` is derived. Default kept, with the consistency
rule in `Leaf`. Decided 2026-11-16 (coordinator).

## Decisions

| # | Decision | Decided |
|:--|:--|:--|
| 1 | Criteria are enforced: `met` gates the start (pre-conditions) and `done` (gates), and gates a directory's `done`. Reverses the earlier rule that `met` gates nothing | Eliott, 2026-11-16 |
| 2 | Every change of a criterion mark writes a `Mark` (`ts`, `actor`, `criterion_id`, `met`, `evidence?`, `reason?`, `correction?`), in `marks` on `Leaf` and on `Directory` | Eliott, 2026-11-16 |
| 3 | Legacy done nodes with an open criterion are imported as they are, and reported by `validate` as errors to reconcile. The importer never ticks | Eliott, 2026-11-16 |
| 4 | Versions are `0.X.Y`; a breaking change bumps X, a compatible one Y; bumps come from commit messages through the existing release setup; M7 wires the crate | Eliott, 2026-11-16 |
| 5 | The mark records live in a `marks` array beside `history`, not inside it, so the last `history` entry stays the last status move (`status_reason` rule) | coordinator, 2026-11-16 |
| 6 | Sealing becomes a reason requirement: `done` and `cancelled` nodes accept every edit with a non-blank reason, flagged `correction: true` on the `Amendment`, `Mark` or `History` record, and a reversal out of `done` is a history entry with `correction: true`. A `correction_count` is derived in the read model | Eliott, 2026-11-16 |
| 7 | Every addressable sub-item has a stable id, minted at creation and never reused or derived from position: criteria, references, checks, steps (narrowed by row 10: criteria and steps only). `Mark.criterion_id` replaces `index` and `kind`; `Change.item_id` replaces array-index pointers; the importer mints the ids. Invariant: no record refers to a sub-item by position, and every mutation leaves existing references valid | Eliott, 2026-11-16 |
| 8 | The audit also moved `Amendment.snapshot` to node-id keys and added `Reference.node_id` for node targets | coordinator, 2026-11-16 (follows from row 7) |
| 9 | Flat layout: a node holds `name`, `parent_id` and `order`; the filesystem no longer mirrors the tree; depth is computed from the `parent_id` chain | Eliott, 2026-11-25 (A2) |
| 10 | An id is given by the rule "pointed at from outside, and its natural key can change". Row 7 is narrowed: checks and references lose their ids, and their edits are recorded at the array level. `Reference.target` is absent when `node_id` is set | contracts_design, 2026-11-25, under Eliott's rulings 3 and 6 (A2) |
