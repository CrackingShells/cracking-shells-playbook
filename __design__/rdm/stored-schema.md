# rdm — Stored schema

Every fact `rdm` stores, with its name, type, whether it is required, and its meaning. This is the
settled form of R01's provisional schema sketch (`architecture.md` §Data Model). The legacy
grammars it must cover are traced production by production in [traceability.md](traceability.md).
Status meanings are in [statuses.md](statuses.md). The read model that consumers receive is a
separate document (`read-model.md`).

Where a default of the M0b plan is defective, the text gives the default as the field table says
it, and then an `OBJECTION` with the alternative. The `rdm-schema` crate builds the default unless
Eliott accepts the objection. Fields new against R01 are marked **new**.

Notation:

| Type | Meaning |
|:--|:--|
| `string` | One line, no line break |
| `lines` | Array of strings, each one line. Used for all multi-line prose, so git merges it line by line. The API accepts and returns plain strings and splits or joins on `\n` |
| `ts` | UTC instant, RFC 3339, `"2026-11-16T10:20:30Z"` |
| `ulid` | A ULID, 26 characters of Crockford base32 |
| `T[]` | Array of `T` |
| `T?` | Optional: the key is absent when there is no value, never `null` |

## Files

```
__roadmap__/<campaign>/
  _dir.json                   campaign root              (Directory, kind: campaign)
  _amendments/<ulid>.json     one file per amendment     (Amendment)
  <leaf>.json                 leaf node                  (Leaf)
  <dir>/_dir.json             directory node             (Directory, kind: directory)
  <dir>/<leaf>.json
  .rdm/                       untracked scratch: lock, journal/, staging/
```

| Rule | Detail |
|:--|:--|
| Names are paths | A node's name is its path segment, `^[a-z][a-z0-9_-]*$`. It is not stored in the file |
| Identity | Every node has an immutable `id`. The id format and its minting belong to M1. The schema types it as `string` |
| Version stamp | Every file carries `schema_version` (see `Format version`) |
| Unknown fields | Every object type below keeps the keys it does not know, in order, and writes them back after the known ones. This holds for nested objects too (steps, criteria, references, commits, history and comment entries, changes). It is the stored half of I12 |
| Canonical order | The order of the rows in each field table is the canonical field order. Two-space indent, LF, trailing newline. M1 owns the serialiser |
| Derived, never stored (I6) | Directory status, the nodes table, the mermaid graph, the progress table, waves, the ready set, a step's number if OBJECTION 1 is accepted, the display id of an amendment, a leaf's commit count when the engine recorded the steps |

## Directory

`_dir.json`, for the campaign root and for every directory node. A directory has no `status`: it
is derived (`statuses.md`, `Subtrees and directories`).

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `schema_version` | `string` | yes | Format version, `"MAJOR.MINOR"` |
| `id` | `string` | yes | Immutable node id |
| `kind` | `"campaign"` or `"directory"` | yes | `campaign` for the root file only |
| `title` | `string` | yes | The H1 of the legacy README |
| `context` | `lines` | yes | Where the node fits; legacy `## Context`, one or more lines |
| `references` | `Reference[]` | yes | Legacy `## Reference Documents`. May be empty (the section is optional in the legacy grammar) |
| `goal` | `string` | yes | One line. The legacy grammar allows exactly one line |
| `preconditions` | `Criterion[]` | yes | Entry criteria, one or more |
| `success_gates` | `Criterion[]` | yes | Completion criteria, one or more |
| `gotchas` | `lines` | yes | Legacy `## Gotchas`, verbatim markdown lines. May be empty (the section is optional) |
| `notes` | `lines` | yes | **new**: the `Notes` cell of the directory's own row in its parent's Progress table. May be empty |
| `comments` | `Comment[]` | yes | **new**: append-only comments. May be empty |

Rules for `validate`: `goal` has no line break; `preconditions` and `success_gates` each have at
least one entry; `kind` is `campaign` exactly when the file is at the campaign root.

A directory carries no branch and no commit count. The legacy Progress row of a directory repeats
the branch of one of its leaves and a count of commits; both are dropped (see `traceability.md`,
`Stretched legacy data`).

## Leaf

`<name>.json`.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `schema_version` | `string` | yes | Format version |
| `id` | `string` | yes | Immutable node id |
| `kind` | `"leaf"` | yes | Discriminates a leaf from a directory when a reader holds only the JSON |
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
| `comments` | `Comment[]` | yes | **new**: append-only comments |

`Progress` is an object, not a separate type section:

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `branch` | `string?` | no | The task branch, for example `task/write-skill-md`. Absent when the legacy cell is `-`, `--`, `---` or `—` |
| `commits` | `integer?` | no | **new**: a commit count written by the importer for a leaf whose steps carry no `commit_sha`. The legacy count can exceed the step count (a fixup commit). The engine never writes it: for an engine-recorded leaf the count is derived from the steps |
| `notes` | `lines` | yes | The legacy `Notes` cell. May be empty |

Rules for `validate`:

- `steps` has one to five entries, and entry *i* has `number == i + 1`.
- `goal` has no line break.
- `references` has at least one entry (the legacy grammar requires a non-empty `**References**` line). OBJECTION 6.
- `status == planned` implies every step has `done: false`.
- `status == done` implies every step has `done: true`.
- `status_reason` is present exactly when `status` is `paused` or `cancelled`, and equals the `reason` of the last `history` entry when `history` is not empty.
- A leaf imported as `done` has steps with `done: true` and no `commit_sha`. I8 is checked when a move to `done` is made (see `Changes against R01`).

## Step

An element of `Leaf.steps`.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `number` | `integer` | yes | 1 to 5, sequential from 1. OBJECTION 1 |
| `title` | `string` | yes | The text after `## Step N:` |
| `goal` | `string` | yes | One line: the unique change intent of the step |
| `implementation_logic` | `lines` | yes | Free prose and lists, blank lines allowed. At least one line is not blank |
| `deliverables` | `lines` | yes | Legacy: one line. Import stores it as one element, unsplit. A new step may hold several elements, one per deliverable. M3's export joins them with `; ` |
| `references` | `Reference[]` | yes | Step-level documents. May be empty (optional in the legacy grammar) |
| `checks` | `Check[]` | yes | One or more. The legacy grammar has exactly one per step, so import yields one. OBJECTION 2 |
| `commit` | `Commit` | yes | The commit message the step must produce |
| `requires` | `lines?` | no | Prose prerequisites that are not obvious. **Never an id or an edge**: I1 forbids a stored dependency between siblings. Never used in real campaigns, kept so the grammar's field has a home |
| `done` | `bool` | yes | The step is recorded done. A step recorded done is sealed (`statuses.md`, `Sealing`) |
| `commit_sha` | `string?` | no | The commit that completed the step. Set with `done: true` by the engine |
| `done_at` | `ts?` | no | **new**: when the step was recorded done |

`Check`:

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `text` | `string` | yes | **new**: the check as written, without the `(expected: …)` trailer. Prose or a command line |
| `expected` | `"PASS"` or `"FAIL"` | yes | The outcome the check expects |
| `command` | `string?` | no | A shell command, when the whole check is one runnable command. Set only by the author of a new check, never guessed by the importer |

**OBJECTION 1: store a step `id`, derive `number`.** The default stores `number`. A number stored
beside the array position can disagree with it, which is the problem R01 avoided for names by
making the name the path. R01's composite "reorder steps" would have to renumber, and comments or
amendment `changes` that point into a step by number would then point at another step. R01's
sketch had a step `id`, and the plan's list drops it. Alternative: keep an immutable step `id`
(minted like node ids), derive `number` from position, and let `Leaf.steps` order be the only
order. Then the validator rule "numbered in sequence" cannot be violated. If accepted, the field
`number` leaves the table, `id` joins it, and in `traceability.md` the `<step-heading>` row reads `title` as stored and the number as derived.

**OBJECTION 2: a check is prose, not `{command, expected}`.** The plan's default makes `command`
required. The legacy check is a prose line that ends in `(expected: PASS)` or `(expected: FAIL)`.
Real checks are commands in backticks, several commands joined by `;`, shell loops, or a
sentence. No parser extracts a `command` from all of them without loss or invention. Decision
recorded: `text` is required and lossless, `expected` is parsed from the trailer (the one part
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
| `text` | `string` | yes | The criterion, one line. Tags such as `[static]` or `[behavioral]` stay in the text |
| `met` | `bool` | yes | Pre-condition: `[x]` is `true`. Gate: `✅` is `true`. A new criterion is `false` |

`met` is a state mark, not content: setting it produces no amendment and is allowed until the
node is sealed. A `done` leaf may carry unmet gates; `met` informs and gates nothing
(`statuses.md`, `Sealing`). Real campaigns hold 16 checked pre-conditions and 150 `✅` gate marks.

## Reference

An element of `references`, at directory, leaf and step level. One shape for all three.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `label` | `string?` | no | `R01`, `R02`, … The label other text cites. Unique within one `references` array. The legacy directory section requires it, so `validate` requires it there |
| `title` | `string` | yes | The link text without the label |
| `target` | `string?` | no | The link target as written: a path relative to the node file, or a URL. Absent for a reference written as plain text. Not resolved by the schema |
| `note` | `string?` | no | The text after ` — `, one line |

Legacy forms and how they split:

| Legacy text | `label` | `title` | `target` | `note` |
|:--|:--|:--|:--|:--|
| `- [R01 Architecture](../a.md) — the design` (directory) | `R01` | `Architecture` | `../a.md` | `the design` |
| `[R01 §Data Model](../a.md) — the sketch` (leaf, in a `; `-separated line) | `R01` | `§Data Model` | `../a.md` | `the sketch` |
| `[Gap analysis](g.md) — why` | none | `Gap analysis` | `g.md` | `why` |
| a segment that is not a link | none | the whole segment | none | none |

The importer splits a leaf's one-line `**References**` on `; ` outside brackets and parentheses.
A label is the prefix `R` and two digits followed by a space at the start of the link text. A
segment it cannot parse becomes a reference with only `title`, so nothing is dropped.
Note: a relative `target` breaks when a node moves. M2 may normalise targets to repository-relative
paths on `move`; the stored form is as written.

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
| `actor` | `Actor` | yes | Who made it. OBJECTION 3 |
| `reason` | `string?` | no | Required for moves into `paused` and `cancelled`, and for `paused → planned` and `paused → ongoing` (`statuses.md`) |
| `override` | `bool?` | no | `true` when the leaf was started outside the ready set |
| `op_id` | `string?` | no | **new**: shared by every entry that one operation writes, so a directory pause or cancel is recognisable as one operation |

`Actor` (also used by `Comment` and `Amendment`):

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `kind` | `"agent"`, `"human"` or `"import"` | yes | The actor kind that `statuses.md` names in its `Who` column |
| `name` | `string?` | no | Free identifier: a user, an agent session, a tool |

**OBJECTION 3: `actor` is an object, not a string.** The plan's list gives `actor` with no type,
and R01 gives none. `statuses.md` states who may set each move, `agent`, `human` or `both`. A
bare string cannot be checked against that. `kind` is the part a rule can test, and `name` is the
part a person reads. If a plain string is wanted, the kind must then be a separate field.

## Comment

An element of `comments` on a leaf or a directory. Append-only. The one change a sealed node
accepts. Never an amendment.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `ts` | `ts` | yes | When it was added |
| `actor` | `Actor` | yes | Who added it |
| `text` | `lines` | yes | The comment, one or more lines (it is prose, so it follows the lines rule) |

Entries are never edited or removed. A correction is a new comment.

**OBJECTION 4: comments in the node file will conflict across branches.** R01 chose one file per
leaf so task branches never touch the same file. A comment on a done leaf, added on two branches,
appends at the end of one array on both sides and git reports a conflict. `history` has the same
property, but only the branch that works a leaf writes it. Comments are written by anyone, on
finished nodes. Alternative: one file per comment, `_comments/<ulid>.json` at the campaign root,
as for amendments, with `node_id` in the entry. A node's comments are then a query, and a comment
never changes the node's `rev`. The default stays in the field tables; M1 should weigh the
alternative before the store fixes the layout.

## Amendment

`_amendments/<ulid>.json`. One file per content or structure edit. Append-only.

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `schema_version` | `string` | yes | Format version |
| `id` | `ulid` | yes | Identity and order. The display id is derived (below) |
| `legacy_id` | `string?` | no | **new**: the legacy `A1`, `A2`, … of an imported amendment |
| `ts` | `ts` | yes | When it was made. For a legacy row: the row's day at `T00:00:00Z` |
| `actor` | `Actor` | yes | Who made it |
| `description` | `string` | yes | Non-empty (I5). The legacy `Rationale` |
| `source` | `string?` | no | The report that justified it. Legacy: a path under `__reports__/` |
| `op_id` | `string?` | no | The retry key of the write that made it |
| `changes` | `Change[]` | yes | What changed, one or more |
| `snapshot` | `object?` | no | The removed subtree of a delete, as stored JSON keyed by path, so it can be restored |

`Change`:

| Field | Type | Req | Meaning |
|:--|:--|:--:|:--|
| `node_id` | `string` | yes | The node the change names. A node's amendment log is the set of amendments whose `changes` name its id |
| `op` | `"add"`, `"remove"`, `"set"`, `"rename"` or `"move"` | yes | R01's primitives. Composite operations expand to these. M2 owns any addition |
| `path_before` | `string?` | no | For `rename`, `move`, `remove` |
| `path_after` | `string?` | no | For `rename`, `move`, `add` |
| `field` | `string?` | no | A JSON pointer into the node, for `set` |
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
| Field | `schema_version`, a string `"MAJOR.MINOR"`, in every file |
| First value | `"0.1"`. The crate stays at `0.x` until M7, so the format does too: a format `1.0` would force crate `1.0` |
| Constant | `FORMAT_VERSION` in `rdm-schema` |
| Bump: minor | A new optional field or a new enum value that old readers can ignore. Adding an optional edge field later is a minor bump |
| Bump: major | A field removed, renamed or given another meaning, or a field made required |
| Reader | Refuses a file with a higher major, with an upgrade message. While the major is `0`, it refuses a higher minor too, since `0.x` minors may break. From `1.0`, a higher minor is read and its unknown fields kept (I12) |
| Writer | Stamps the version it writes. One campaign may hold files of different minors |
| Relation to the crate | A format major forces a crate major, a format minor a crate minor (M0b plan). The read-model version is a separate constant, in `read-model.md` |

## Changes against R01

| Area | R01 sketch | This schema |
|:--|:--|:--|
| Criteria | `preconditions[]`, `success_gates[]` as plain text | `Criterion {text, met}`, at leaf and directory level |
| Step | `{id, goal, implementation_logic, deliverables[], checks[], commit, references[], requires[], done, commit_sha?}` | adds `number`, `title`, `done_at?`; `implementation_logic` and `deliverables` are `lines`; `requires` is optional prose; `id` is dropped, see OBJECTION 1 |
| Check | `{command, expected}` | `{text, expected, command?}`, OBJECTION 2 |
| Commit | `{type, scope, summary}` | `{type, scope?, breaking, summary}` |
| Reference | `references[]`, no shape | `{label?, title, target?, note?}` |
| Status | `blocked_reason?` | `status_reason?`; vocabulary per `statuses.md` |
| History | `{ts, from, to, actor, note?, override?}` | `note` is `reason`; adds `op_id?`; `actor` is an object |
| Comments | none | `comments[]` on leaves and directories |
| Amendment | `{… id, …}` | adds `legacy_id?`; the id is a ULID with a derived display id |
| Directory | no notes | adds `notes` and `comments` |
| Progress | `{branch?, notes}` | adds `commits?` (import only); `notes` is `lines` |
| Gotchas, context | strings or lines unstated | `lines` |
| Unknown fields | I12 stated | Every type keeps them, nested ones too |
| Format version | "every file carries `schema_version`" | `"MAJOR.MINOR"`, first value `"0.1"`, reader rules above |

**Finding against R01 (invariant I8).** R01 says a leaf is `done` only when every step is done
with its commit recorded. A leaf imported from legacy markdown is `done` and has no `commit_sha`
on any step: the grammar never stored one. If I8 were checked on stored state, every imported done
leaf would fail. Resolution: I8 is checked when the move to `done` is made, and the stored-state
rule is "`done` leaf has every step `done`". The import records the legacy count in
`progress.commits`. `r01_sync` rewrites I8 to say so. This was not in `statuses.md`, whose `R01
deltas` row for I8 says "unchanged"; this document supersedes that row.

**OBJECTION 5: `status_reason` is a copy of the last history reason.** It is derived data stored
beside its source, which I6 forbids. The default keeps it because an imported `blocked → cancelled`
leaf has no history entry to hold the reason. Alternative: the importer writes one `history` entry
with `actor.kind: "import"` and `status_reason` is derived. Default kept, with the consistency
rule in `Leaf`.

**OBJECTION 6: a leaf's `references` need not be non-empty.** The legacy grammar demands a
non-empty `**References**` line, and this document carries that as a `validate` rule. A leaf that
has nothing to cite then has to invent a reference. A warning, not an error, would be kinder.
Left to M2's diagnostics, which assign severities.
