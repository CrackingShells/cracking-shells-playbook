# rdm — Traceability of the legacy grammars

Every production of `leaf.bnf` and `readme.bnf` has one row, with a disposition:

| Disposition | Meaning |
|:--|:--|
| `stored` | The fact is a field of the stored schema. The target is the field path in [stored-schema.md](stored-schema.md) |
| `derived` | The fact is computed on every read and never stored (I6). The target says from what |
| `dropped` | The fact is not kept. The target is the reason |

`uv run --no-project tools/check_rdm_traceability.py` prints each grammar with
`bash skills/managing-roadmaps/scripts/dirtree-rdm.sh grammar leaf` and `grammar readme`, and
fails unless every production appears in the matching table below with a valid disposition and a
non-empty target. It also fails on a row that names no production, so a renamed production cannot
leave a stale row.

Counts: `leaf.bnf` has 22 productions. `readme.bnf` has 57. The plan says 58; the printer's
header carries one more `::=` in a comment (`<rule> ::= production`), which defines nothing.
Four names occur in both grammars (`<h1-title>`, `<checkbox-item>`, `<gate-item>`,
`<non-blank-line>`) and have a row in each table.

Cells do not contain a pipe character, so the script can split rows on it. "Presentation" means
that the export (M3) re-creates the text from stored fields.

## Leaf grammar

| Production | Disposition | Target or reason |
|:--|:--|:--|
| `<leaf-task>` | `stored` | One `Leaf` file, `<name>.json`. The order of H1, header and steps is export layout |
| `<task-header>` | `stored` | Leaf `goal`, `preconditions`, `success_gates`, `references`. The grammar is order-insensitive, so no order is kept |
| `<header-field>` | `stored` | One of the four header fields below |
| `<field-goal>` | `stored` | `goal`, one line |
| `<field-preconditions>` | `stored` | `preconditions[]` |
| `<field-success-gates>` | `stored` | `success_gates[]` |
| `<field-references>` | `stored` | `references[]`, the line split on a semicolon and a space into `Reference` |
| `<checkbox-item>` | `stored` | `preconditions[].text` and `.met`; `[x]` is `true`, `[ ]` is `false` |
| `<gate-item>` | `stored` | `success_gates[].text` and `.met`; the check mark is `true`, the empty box is `false` |
| `<step>` | `stored` | One element of `steps[]` |
| `<step-heading>` | `stored` | `steps[].title` is the text after the colon; `steps[].number` is the digit (derived if OBJECTION 1 is accepted) |
| `<step-field>` | `stored` | One of the step fields below |
| `<step-field-goal>` | `stored` | `steps[].goal` |
| `<step-field-impl-logic>` | `stored` | `steps[].implementation_logic`, the body lines with blank lines kept. The header line and trailing blank lines are presentation |
| `<step-field-references>` | `stored` | `steps[].references[]` |
| `<step-field-deliverables>` | `stored` | `steps[].deliverables`, one element, not split |
| `<step-field-consistency>` | `stored` | `steps[].checks[].text` and `.expected`. The trailer `(expected: X)` becomes `expected`. `command` is not extracted: the check is prose (OBJECTION 2) |
| `<step-field-commit>` | `stored` | `steps[].commit.type`, `.scope`, `.breaking`, `.summary`. The backticks are presentation and are dropped |
| `<step-field-requires>` | `stored` | `steps[].requires`, prose lines |
| `<h1-title>` | `stored` | `title` |
| `<impl-body-line>` | `stored` | One element of `steps[].implementation_logic` |
| `<non-blank-line>` | `dropped` | A lexical helper. The lines it matches are stored through `implementation_logic[]` |

Constraints the grammar states outside its productions, and where each one goes:

| Constraint | Home |
|:--|:--|
| Step count is 1 to 5 | `validate` rule on `Leaf.steps` |
| Step numbers are sequential from 1 | `validate` rule on `steps[].number` (derived, if OBJECTION 1 is accepted) |
| At least one implementation line is not blank | `validate` rule on `Step.implementation_logic` |
| All four header fields and the five required step fields are present | Required fields of `Leaf` and `Step` |
| Header and step field order is free | Dropped: the canonical order is the field table's |
| Pre-conditions and gates have one or more items | `validate` rule on `Leaf` |

## README grammar

| Production | Disposition | Target or reason |
|:--|:--|:--|
| `<readme>` | `stored` | One `Directory` file, `_dir.json`. Section order is export layout |
| `<h1-title>` | `stored` | `title` |
| `<section-context>` | `stored` | `context` |
| `<h2-context>` | `dropped` | Heading literal. The field name is `context` |
| `<context-body>` | `stored` | `context`, the lines |
| `<section-reference-documents>` | `stored` | `references[]`. The section is optional; an absent section is an empty array |
| `<h2-reference-documents>` | `dropped` | Heading literal |
| `<reference-list>` | `stored` | `references[]` |
| `<reference-item>` | `stored` | `references[].label` is the `R` and two digits, `.title`, `.target`, `.note` from the text after the dash |
| `<section-goal>` | `stored` | `goal` |
| `<h2-goal>` | `dropped` | Heading literal |
| `<goal-body>` | `stored` | `goal`, exactly one line |
| `<section-preconditions>` | `stored` | `preconditions[]` |
| `<h2-preconditions>` | `dropped` | Heading literal |
| `<checkbox-list>` | `stored` | `preconditions[]` |
| `<checkbox-item>` | `stored` | `preconditions[].text` and `.met` |
| `<section-success-gates>` | `stored` | `success_gates[]` |
| `<h2-success-gates>` | `dropped` | Heading literal |
| `<gate-list>` | `stored` | `success_gates[]` |
| `<gate-item>` | `stored` | `success_gates[].text` and `.met` |
| `<section-gotchas>` | `stored` | `gotchas`, verbatim lines. The section is optional; absent is an empty array |
| `<h2-gotchas>` | `dropped` | Heading literal |
| `<section-status>` | `derived` | The status view of the node's children, from their `status` or derived status |
| `<h2-status>` | `dropped` | Heading literal |
| `<mermaid-block>` | `derived` | The graph of the node's children, rebuilt on each export |
| `MERMAID_OPEN` | `dropped` | Markup; the export emits it |
| `MERMAID_CLOSE` | `dropped` | Markup; the export emits it |
| `<mermaid-graph-decl>` | `dropped` | The fixed line `graph TD`; the export emits it |
| `<mermaid-node-decl>` | `derived` | One per child: the id is the child's path name, the label is its `title`, the class is its status. 13 of 95 real labels differ from the child's H1; the title wins |
| `<mermaid-classdef-done>` | `dropped` | Colours are presentation; M6 owns the palette |
| `<mermaid-classdef-inprogress>` | `dropped` | Colours are presentation. The class follows the status `ongoing` |
| `<mermaid-classdef-planned>` | `dropped` | Colours are presentation |
| `<mermaid-classdef-amendment>` | `dropped` | Colours are presentation. The status `amendment` no longer exists (`statuses.md`) |
| `<mermaid-classdef-blocked>` | `dropped` | Colours are presentation. The status `blocked` no longer exists (`statuses.md`) |
| `<section-nodes>` | `derived` | The table of the node's children, from the directory listing |
| `<h2-nodes>` | `dropped` | Heading literal |
| `<nodes-table>` | `derived` | One row per child. Row order carries no meaning: siblings are parallel, so readers order by depth, kind and name. The authored row order is not kept |
| `<table-header-nodes>` | `dropped` | Fixed header; the export emits it |
| `<table-separator>` | `dropped` | Markdown table syntax |
| `<nodes-table-row>` | `derived` | A leaf row or a directory row |
| `<nodes-table-leaf-row>` | `derived` | Name from the path, type from `kind`, status from `Leaf.status`. The emoji labels are presentation |
| `<nodes-table-dir-row>` | `derived` | Name from the path, status derived from the leaves under it (`statuses.md`). The emoji labels are presentation |
| `<section-amendment-log>` | `stored` | `Amendment` files. The log of a directory is the amendments whose changes name its nodes |
| `<h2-amendment-log>` | `dropped` | Heading literal |
| `<amendment-table>` | `stored` | `Amendment` files, one per row |
| `<table-header-amendment>` | `dropped` | Fixed header; the export emits it |
| `<amendment-row>` | `stored` | `Amendment.legacy_id`, `.ts`, `.source`, `.changes[]` (one add per filename), `.description` |
| `<section-progress>` | `stored` | `Leaf.progress` and `Directory.notes` |
| `<h2-progress>` | `dropped` | Heading literal |
| `<progress-table>` | `derived` | Rendered from the progress of every leaf and the notes of every directory |
| `<table-header-progress>` | `dropped` | Fixed header; the export emits it |
| `<progress-row>` | `stored` | Leaf row: `Leaf.progress.branch`, `.commits`, `.notes`. Directory row: `Directory.notes`; its branch and commit count are dropped as copies of its leaves' |
| `<non-blank-line>` | `dropped` | A lexical helper. The lines it matches are stored through `context`, `gotchas` and `goal` |
| `<status-value>` | `stored` | `Leaf.status`, through the mapping in `statuses.md`, `Legacy mapping` |
| `<node-id>` | `derived` | The path segment of the node. The stored `id` is new and minted by M1 |
| `<node-name-file>` | `derived` | The path segment of a leaf plus `.json` |
| `<node-name-dir>` | `derived` | The path segment of a directory |

## Stretched legacy data

Data that the grammar accepts, or that the validator lets through, and that real campaigns bend.
Counts are over `__roadmap__/` at the date of this document.

| Item | Evidence | Home or drop | Importer rule |
|:--|:--|:--|:--|
| Checked pre-conditions | 16 `[x]` pre-conditions outside the M0 and M0b campaigns; 150 gate check marks; 30 of 48 `done` leaves still carry an unmet gate | `stored`: `Criterion.met`. A `done` leaf may keep unmet gates, so `met` never gates a move | `[x]` and the check mark give `true`. Nothing is inferred from the leaf's status |
| Progress rows on directories | 3 rows: `implement/`, `finalize/` (both with dashes) and `packaging/` (branch, count 1, empty notes). R01 puts progress on leaves only | `stored`: `Directory.notes`. Branch and commit count are `dropped`, copies of the leaves' | A directory row sets `notes` and nothing else |
| Dash cells | 2 rows use an em dash (U+2014). The validator accepts only `-`, `--` and `---` | `dropped` as a value: a dash means absent | `—`, `–`, `-`, `--` and `---` all mean no value |
| Rows the validator never reads | Its row loop stops silently at the first row that does not match. Only 1 of 14 real Progress rows matches: 11 write the branch in backticks, 2 use the em dash, and 5 have an empty Notes cell, which its pattern cannot match either | No fact lost in the schema | The importer reads every row, strips backticks around the branch, and accepts an empty Notes cell. A row it cannot read is kept as text in `Directory.notes` with a warning, never skipped |
| Commit count above the step count | `dirtree-rdm-cli-ux/refine-output-rendering.md`: 5 commits for 4 steps ("4 step commits + 1 clippy fixup") | `stored`: `Leaf.progress.commits`, import only. For an engine-recorded leaf the count is derived from `commit_sha` | A numeric cell is copied. `-` leaves it absent |
| Done leaves without commit shas | Legacy markdown holds no per-step commit | `Step.commit_sha` is absent on imported steps. I8 is checked on the move to `done`, not on stored state (`stored-schema.md`, `Changes against R01`) | Imported steps of a `done` leaf are `done: true` with no sha |
| The `Requires` step field | 0 uses in any campaign | `stored`: `Step.requires`, prose lines, never ids (I1) | The line is copied as one element |
| Amendment log | 2 rows, both `A1`: `agent_plugin_nest` and `roadmap-mcp-rework`. `Nodes Added` holds filenames, written `["x.md"]` or `[ "x.md" ]`. `Source` holds a report path | `stored`: `Amendment` with `legacy_id` | Both list spellings are read. The date becomes midnight UTC. The actor is `import` |
| Amendment log per directory | Each README has its own log; R01 has one `_amendments/` per campaign | `stored` once, at the campaign root | The importer collects every row of every README |
| Node added by an amendment | The one such node, `hub_mode_reconciliation.md`, is `done`. No node carries the status `amendment` | The origin is the amendment's `add` change | Status maps as written; nothing else |
| Statuses in use | 26 planned, 4 in progress, 65 done; 0 blocked; 0 amendment | Mapping in `statuses.md` | None beyond the mapping |
| Reference items without a label | `create-reporting-skill/README.md` lists a reference with no `R01` label, so its validation fails at that line | `Reference.label` is optional in the type | The label is absent and the importer warns. `validate` requires a label in a directory |
| Mermaid labels | 13 of 95 differ from the child's H1 | `dropped`: derived from `title` | The H1 wins; the importer warns on a difference |
| Sibling order | Nodes-table row order | `dropped`: siblings are parallel, order carries no meaning | Readers order by depth, kind and name |
| Step count | 23 leaves with 1 step, 16 with 2, 23 with 3, 3 with 4, 2 with 5 | `validate` rule: 1 to 5 | None needed; none exceeds 5 |
