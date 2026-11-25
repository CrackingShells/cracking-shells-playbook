# rdm — Read model

The shape every consumer receives. Four projections over the stored files. Each one has a field
list, the consumer that uses it, and a worked example computed by hand from
`__roadmap__/roadmap-mcp-rework/`.

The stored facts are in [stored-schema.md](stored-schema.md). The meaning of each status, the
ready rule and the derivation of a directory's status are in [statuses.md](statuses.md). The
legacy campaign is read through its legacy mapping first: `inprogress` is `ongoing`, `planned` is
`planned`.

## Projections

| Projection | Section | Consumers | Answers |
|:--|:--|:--|:--|
| Campaign outline with waves | `Campaign outline` | M4 `roadmap_view` and the app-only tool behind it (R01 `ui_snapshot`), M5 `--json`, M6 view | What is in the campaign, in what status, in which wave |
| Node detail | `Node detail` | M4 `roadmap_get`, M5 `--json`, M6 detail pane | Everything about one node |
| Ready set | `Ready set` | M4 `roadmap_next`, M5 `--json` | What may start now, and what holds the rest back |
| Amendment log | `Amendment log` | M4 `roadmap_amendments`, M5 `--json`, M6 timeline | What was changed, by whom, why |

M4 owns the final tool names; R01's table is a suggestion. The campaign list and the diagnostics
of `validate` are not projections of this document. M2 owns the diagnostics.

**Every projection field has a source.** The `Source` column of each field table names a stored
field (path in `stored-schema.md`) or a derivation. A field with neither is a defect.

**Envelope.** Every projection starts with these fields.

| Field | Type | Source |
|:--|:--|:--|
| `read_model_version` | `string` | The constant `READ_MODEL_VERSION` |
| `format_version` | `string` | The highest `schema_version` among the files read |
| `projection` | `"outline"`, `"node"`, `"ready"` or `"amendments"` | Fixed by the projection |
| `campaign` | `{id, path, title}` | `id` and `title` from the campaign root node. `path` is the argument of the call |
| `tree_rev` | `string` | Opaque. Computed by M1 over the campaign |

**Revisions.** `rev` (one node) and `tree_rev` (the campaign) are opaque strings. A consumer
compares them for equality and sends them back as `expected_rev` and `expected_tree_rev`. It
never parses them. They are defined in [store.md](store.md) §Revisions. A comment never changes a `rev`
(`stored-schema.md`, `Comment`).

**Order.** Siblings carry a stored `order` key (contracts.md §Node fields), a position for
presentation that never expresses a dependency (I1). Every list below orders nodes by `level`, then
siblings by `(order, id)`, as `Campaign::children` does. Lists of amendments and comments order by ULID, oldest first.

**Types in `rdm-schema`.** The crate leaf holds these as types:

| Type | Holds |
|:--|:--|
| `Envelope` | The envelope fields, flattened into each view |
| `OutlineView`, `NodeView`, `ReadyView`, `AmendmentsView` | The four projections |
| `LevelSource` | Enum, one value today: `depth` |
| `StatusCounts` | `{planned, ongoing, paused, cancelled, done}`, five integers, all always present |
| `Holder` | `{path, title, status, status_reason?}`, the value of a `holders` map |
| `WaveView`, `OutlineNode`, `StepView`, `AmendmentEntry`, `ChangeView` | Parts named in the field tables |

**Examples.** Ids are stand-ins (`n_m1`): M1 mints the real ones. Revisions are stand-ins
(`rev-m1`). A string ending in `[cut]` or a string element `"[cut: …]"` marks text left out. It is
not part of the shape.

## Campaign outline

One call returns the whole tree, its waves, and the flags a graph needs.

| Field | Type | Source |
|:--|:--|:--|
| `root` | `OutlineNode` of kind `directory`, `is_campaign: true` | The campaign root, see below. The envelope's `campaign` holds only the identity |
| `waves` | `WaveView[]` | Derived: the leaves grouped by `level`, ascending, one entry per level that has a leaf |

`OutlineNode`, directory:

| Field | Type | Source |
|:--|:--|:--|
| `id`, `title` | `string` | `Directory.id`, `.title` |
| `path` | `string` | Derived: the path segments from the campaign root. `""` for the root |
| `kind` | `"directory"` | `Directory.kind`, with `campaign` shown as `is_campaign: true` |
| `is_campaign` | `bool` | `Directory.kind == "campaign"` |
| `goal` | `string` | `Directory.goal`. Root only |
| `status` | status | Derived, `statuses.md` `Subtrees and directories` |
| `counts` | `StatusCounts` | Derived: the leaves under the directory, any depth, counted by `status`. Needed because a directory shows `done` even when some leaves are cancelled |
| `amendment_count` | `integer` | Derived: the `Amendment` files. Root only |
| `rev` | `string` | Opaque, M1 |
| `children` | `OutlineNode[]` | Derived from the directory listing, in the order above |

`OutlineNode`, leaf:

| Field | Type | Source |
|:--|:--|:--|
| `id`, `title`, `status` | | `Leaf.id`, `.title`, `.status` |
| `status_reason` | `string?` | `Leaf.status_reason` |
| `path` | `string` | Derived, as above |
| `kind` | `"leaf"` | `Leaf.kind` |
| `level`, `level_source` | `integer`, `"depth"` | `Level and its source` |
| `ready` | `bool` | Derived: the leaf is in the ready set (`Ready set`) |
| `steps` | `{done, total}` | Derived: count of `Leaf.steps[].done`, and of `Leaf.steps` |
| `added_by` | `string?` | Derived: the display id of the amendment with a change `{op: add}` naming this node. Absent when none |
| `rev` | `string` | Opaque, M1 |

`WaveView`:

| Field | Type | Source |
|:--|:--|:--|
| `level`, `level_source` | `integer`, `"depth"` | `Level and its source` |
| `leaf_ids` | `string[]` | Derived: ids of the leaves at this level, in the order above |
| `counts` | `StatusCounts` | Derived: those leaves counted by `status` |

A wave is a display group. It is not a readiness fact: readiness is in `Ready set`.

### Worked example

Counts and statuses of `roadmap-mcp-rework`: ten leaves. M0 and M0b are `ongoing` (legacy
`inprogress`). M1 to M8 are `planned`. Every leaf has three steps, none done. The one amendment,
legacy `A1`, added `m0b_domain_model_spec`. The ready set is empty (see `Ready set`), so every
`ready` is `false`. Directory statuses: the root has an `ongoing` leaf, so `ongoing` (row 4). The
six directories hold only `planned` leaves, so `planned` (row 7).

```json
{
  "read_model_version": "0.1.0", "format_version": "0.1.0", "projection": "outline",
  "campaign": { "id": "c_rework", "path": "__roadmap__/roadmap-mcp-rework",
                "title": "rdm rework — meta-roadmap" },
  "tree_rev": "tree-1",
  "root": {
    "id": "c_rework", "path": "", "kind": "directory", "is_campaign": true,
    "title": "rdm rework — meta-roadmap",
    "goal": "`managing-roadmaps` ships as a plugin whose tools are a published Rust MCP server and CLI over one core, with JSON roadmaps, full CRUD under an amendment log, and a graph view.",
    "status": "ongoing",
    "counts": { "planned": 8, "ongoing": 2, "paused": 0, "cancelled": 0, "done": 0 },
    "amendment_count": 1, "rev": "rev-root",
    "children": [
      { "id": "n_m0", "path": "m0_feasibility_spike", "kind": "leaf", "title": "M0 Feasibility spike",
        "status": "ongoing", "level": 0, "level_source": "depth", "ready": false,
        "steps": { "done": 0, "total": 3 }, "rev": "rev-m0" },
      { "id": "n_m0b", "path": "m0b_domain_model_spec", "kind": "leaf", "title": "M0b Domain model spec",
        "status": "ongoing", "level": 0, "level_source": "depth", "ready": false,
        "steps": { "done": 0, "total": 3 }, "added_by": "A1", "rev": "rev-m0b" },
      { "id": "d_core", "path": "core", "kind": "directory", "is_campaign": false, "title": "Core",
        "status": "planned",
        "counts": { "planned": 8, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 }, "rev": "rev-core",
        "children": [
          { "id": "n_m1", "path": "core/m1_core_foundation", "kind": "leaf", "title": "M1 Core foundation",
            "status": "planned", "level": 1, "level_source": "depth", "ready": false,
            "steps": { "done": 0, "total": 3 }, "rev": "rev-m1" },
          { "id": "d_engine", "path": "core/engine", "kind": "directory", "is_campaign": false, "title": "Engine",
            "status": "planned",
            "counts": { "planned": 7, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 }, "rev": "rev-engine",
            "children": [
              { "id": "n_m2", "path": "core/engine/m2_graph_engine", "kind": "leaf", "title": "M2 Graph engine",
                "status": "planned", "level": 2, "level_source": "depth", "ready": false,
                "steps": { "done": 0, "total": 3 }, "rev": "rev-m2" },
              { "id": "d_interchange", "path": "core/engine/interchange", "kind": "directory",
                "is_campaign": false, "title": "Interchange", "status": "planned",
                "counts": { "planned": 6, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 },
                "rev": "rev-interchange",
                "children": [
                  { "id": "n_m3", "path": "core/engine/interchange/m3_interchange", "kind": "leaf",
                    "title": "M3 Interchange", "status": "planned", "level": 3, "level_source": "depth",
                    "ready": false, "steps": { "done": 0, "total": 3 }, "rev": "rev-m3" },
                  { "id": "d_consumers", "path": "core/engine/interchange/consumers", "kind": "directory",
                    "is_campaign": false, "title": "Consumers", "status": "planned",
                    "counts": { "planned": 5, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 },
                    "rev": "rev-consumers",
                    "children": [
                      { "id": "n_m4", "path": "core/engine/interchange/consumers/m4_mcp_server", "kind": "leaf",
                        "title": "M4 MCP server", "status": "planned", "level": 4, "level_source": "depth",
                        "ready": false, "steps": { "done": 0, "total": 3 }, "rev": "rev-m4" },
                      { "id": "n_m5", "path": "core/engine/interchange/consumers/m5_cli", "kind": "leaf",
                        "title": "M5 CLI", "status": "planned", "level": 4, "level_source": "depth",
                        "ready": false, "steps": { "done": 0, "total": 3 }, "rev": "rev-m5" },
                      { "id": "d_delivery", "path": "core/engine/interchange/consumers/delivery",
                        "kind": "directory", "is_campaign": false, "title": "Delivery", "status": "planned",
                        "counts": { "planned": 3, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 },
                        "rev": "rev-delivery",
                        "children": [
                          { "id": "n_m6", "path": "core/engine/interchange/consumers/delivery/m6_apps_view",
                            "kind": "leaf", "title": "M6 Apps view", "status": "planned", "level": 5,
                            "level_source": "depth", "ready": false, "steps": { "done": 0, "total": 3 },
                            "rev": "rev-m6" },
                          { "id": "n_m7", "path": "core/engine/interchange/consumers/delivery/m7_distribution",
                            "kind": "leaf", "title": "M7 Distribution", "status": "planned", "level": 5,
                            "level_source": "depth", "ready": false, "steps": { "done": 0, "total": 3 },
                            "rev": "rev-m7" },
                          { "id": "d_cutover", "path": "core/engine/interchange/consumers/delivery/cutover",
                            "kind": "directory", "is_campaign": false, "title": "Cutover", "status": "planned",
                            "counts": { "planned": 1, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 },
                            "rev": "rev-cutover",
                            "children": [
                              { "id": "n_m8",
                                "path": "core/engine/interchange/consumers/delivery/cutover/m8_cutover",
                                "kind": "leaf", "title": "M8 Cutover", "status": "planned", "level": 6,
                                "level_source": "depth", "ready": false, "steps": { "done": 0, "total": 3 },
                                "rev": "rev-m8" } ] } ] } ] } ] } ] } ] } ]
  },
  "waves": [
    { "level": 0, "level_source": "depth", "leaf_ids": ["n_m0", "n_m0b"],
      "counts": { "planned": 0, "ongoing": 2, "paused": 0, "cancelled": 0, "done": 0 } },
    { "level": 1, "level_source": "depth", "leaf_ids": ["n_m1"],
      "counts": { "planned": 1, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 } },
    { "level": 2, "level_source": "depth", "leaf_ids": ["n_m2"],
      "counts": { "planned": 1, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 } },
    { "level": 3, "level_source": "depth", "leaf_ids": ["n_m3"],
      "counts": { "planned": 1, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 } },
    { "level": 4, "level_source": "depth", "leaf_ids": ["n_m4", "n_m5"],
      "counts": { "planned": 2, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 } },
    { "level": 5, "level_source": "depth", "leaf_ids": ["n_m6", "n_m7"],
      "counts": { "planned": 2, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 } },
    { "level": 6, "level_source": "depth", "leaf_ids": ["n_m8"],
      "counts": { "planned": 1, "ongoing": 0, "paused": 0, "cancelled": 0, "done": 0 } }
  ]
}
```

## Node detail

One node, with everything stored about it and what is derived. `kind` says which of two shapes.

Common to both kinds:

| Field | Type | Source |
|:--|:--|:--|
| `id`, `title`, `kind` | | `id`, `title`, `kind` of the file |
| `path` | `string` | Derived: path segments from the campaign root |
| `parent` | `string` | Derived: the path of the parent directory. `""` for a node under the root |
| `rev` | `string` | Opaque, M1 |
| `finished` | `bool` | Derived from `statuses.md` `Finished nodes`: `true` when an edit needs a reason and is a correction (leaf `cancelled` or `done`, directory with derived status `cancelled` or `done`) |
| `correction_count` | `integer` | Derived, never stored: the number of records with `correction: true` that name this node (its amendments, its `marks`, its `history`). It measures how often corrections and reversals happen. `0` for a node never corrected |
| `comments` | `Comment[]` | The `_comments/` files whose `node_id` is this node, oldest first. The stored `Comment` shape |
| `amendments` | `string[]` | Derived: display ids of the amendments whose `changes` name this node |
| `added_by` | `string?` | As in `Campaign outline` |

Leaf only:

| Field | Type | Source |
|:--|:--|:--|
| `status`, `status_reason?` | | `Leaf.status`, `.status_reason` |
| `status_since` | `ts?` | Derived: `ts` of the last `Leaf.history` entry. Absent when `history` is empty, as for an imported leaf |
| `level`, `level_source` | | `Level and its source` |
| `ready` | `bool` | Derived, `Ready set` |
| `held_by` | `string[]` | Derived: ids of the unfinished leaves that hold this leaf back, by the ready rule. Empty when none. Also set for a leaf that is not `planned`, so a reader sees what a paused leaf waits for |
| `goal`, `preconditions`, `success_gates`, `references` | | The stored fields of `Leaf` |
| `steps` | `StepView[]` | `StepView`: every stored `Step` field, plus `number`. A consumer addresses a step by `id`, never by `number` |
| `progress` | | `Leaf.progress`, as stored |
| `commit_count` | `integer?` | Derived: the number of steps with a `commit_sha`; else `progress.commits`; else absent |
| `step_progress` | `{done, total}` | Derived, as `steps` in `Campaign outline` |
| `history` | `History[]` | `Leaf.history`, as stored, oldest first |
| `marks` | `Mark[]` | `Leaf.marks`, as stored, oldest first: the trace of every criterion mark |

`StepView.number` is derived for display: the position in `Leaf.steps` plus one. It is not a reference: a reorder changes it, and `Step.id` does not. Criteria, references and checks are likewise addressed by their `id` (`stored-schema.md`, `Stable identity`).

Directory only:

| Field | Type | Source |
|:--|:--|:--|
| `is_campaign` | `bool` | `Directory.kind == "campaign"` |
| `status` | status | Derived, `statuses.md` |
| `counts` | `StatusCounts` | Derived, as in `Campaign outline` |
| `context`, `goal`, `preconditions`, `success_gates`, `references`, `gotchas`, `notes` | | The stored fields of `Directory` |
| `marks` | `Mark[]` | `Directory.marks`, as stored, oldest first |
| `children` | `{id, path, kind, title, status}[]` | Derived from the directory listing, in the order above |

Both kinds: a top-level `holders` map, id to `Holder`, holds every id named in `held_by`, so the
caller needs no second read.

### Worked example

`core/m1_core_foundation`, a `planned` leaf held back by M0 and M0b. Its three-step file is cut to
step 1.

```json
{
  "read_model_version": "0.1.0", "format_version": "0.1.0", "projection": "node",
  "campaign": { "id": "c_rework", "path": "__roadmap__/roadmap-mcp-rework",
                "title": "rdm rework — meta-roadmap" },
  "tree_rev": "tree-1",
  "node": {
    "id": "n_m1", "path": "core/m1_core_foundation", "parent": "core", "kind": "leaf",
    "rev": "rev-m1", "title": "M1 Core foundation",
    "status": "planned", "level": 1, "level_source": "depth",
    "ready": false, "held_by": ["n_m0", "n_m0b"], "finished": false, "correction_count": 0,
    "goal": "`rdm-engine` stores a roadmap as JSON, one file per node, and N processes can read and write one campaign without losing or tearing a write.",
    "preconditions": [
      { "id": "k_1", "text": "`../m0_feasibility_spike.md` is `done` with a go", "met": false },
      { "id": "k_2", "text": "`../m0b_domain_model_spec.md` is `done`", "met": false },
      { "id": "k_3", "text": "Reminder `skills-rdm-M1` read", "met": false } ],
    "success_gates": [
      { "id": "k_4", "text": "`rdm-engine` depends on `rdm-schema` for every stored type and holds none of its own; a file with a newer format major is refused with an upgrade message", "met": false },
      "[cut: 4 more gates, same shape]" ],
    "references": [
      { "id": "r_1", "label": "R01", "title": "§Data Model, §Concurrency, §Contracts & Invariants",
        "target": "../../../__design__/rdm/architecture.md",
        "note": "the file layout, the schema sketch, the lock, the journal and the revisions" } ],
    "steps": [
      { "id": "s_m1_1", "number": 1, "title": "Plan the milestone",
        "goal": "The campaign `__roadmap__/m1-core-foundation/` exists and Eliott has approved it.",
        "implementation_logic": [
          "In plan mode, author the campaign with `managing-roadmaps`, deciding: the id minting, to the format M0b fixes; … [cut]",
          "[cut: 1 more line]" ],
        "deliverables": [ "`__roadmap__/m1-core-foundation/README.md` and its leaves" ],
        "references": [],
        "checks": [ { "id": "ck_1", "text": "`bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m1-core-foundation/`",
                      "expected": "PASS" } ],
        "commit": { "type": "chore", "scope": "roadmap", "breaking": false,
                    "summary": "plan the m1-core-foundation milestone" },
        "done": false },
      "[cut: steps 2 and 3, same shape]" ],
    "progress": { "notes": [] },
    "step_progress": { "done": 0, "total": 3 },
    "history": [], "marks": [], "comments": [], "amendments": []
  },
  "holders": {
    "n_m0":  { "path": "m0_feasibility_spike",  "title": "M0 Feasibility spike",  "status": "ongoing" },
    "n_m0b": { "path": "m0b_domain_model_spec", "title": "M0b Domain model spec", "status": "ongoing" }
  }
}
```

The imported leaf has no `history`, no `commit_count`, no `status_since` and no `branch`: the
legacy file held none of them, and its Progress table is empty.

## Ready set

**The rule** is the scoped rule of `statuses.md` (Eliott, 2026-11-16). A leaf is ready when it is
`planned` and no leaf holds it back. An unfinished leaf (`planned`, `ongoing`, `paused`) holds
back only the deeper leaves under its own parent. In steps: for a leaf L, take every directory
above L's own directory, up to and including the campaign root. Every unfinished leaf that sits
directly in one of those directories holds L back. Sibling leaves are parallel. `done` and
`cancelled` leaves hold nothing back.

| Field | Type | Source |
|:--|:--|:--|
| `ready` | `ReadyLeaf[]` | Derived: the leaves that are `planned` and held by none |
| `stalled` | `StalledLeaf[]` | Derived: the `planned` leaves held by at least one leaf |
| `holders` | map id to `Holder` | Derived: every leaf named in a `held_by` |

`ReadyLeaf`: `id`, `path`, `title` (stored or derived as in the outline), `level` and
`level_source`. `StalledLeaf`: the same, plus `held_by`, the ids of the unfinished leaves that hold
it, in the order above. A leaf that is `ongoing`, `paused` or finished is in neither list. A
`paused` leaf shows in `holders` with its `status_reason`.

### Worked example: the real campaign

Statuses: M0, M0b `ongoing`; M1 to M8 `planned`. The campaign is a chain: each leaf sits in a
directory under the previous leaf's directory, so each holds back everything deeper. M0 and M0b
sit at the root and hold back every directory leaf. No `planned` leaf is free: `ready` is empty.
M1 is held by M0 and M0b. M2 adds M1, and so on down the chain.

```json
{
  "read_model_version": "0.1.0", "format_version": "0.1.0", "projection": "ready",
  "campaign": { "id": "c_rework", "path": "__roadmap__/roadmap-mcp-rework",
                "title": "rdm rework — meta-roadmap" },
  "tree_rev": "tree-1",
  "ready": [],
  "stalled": [
    { "id": "n_m1", "path": "core/m1_core_foundation", "title": "M1 Core foundation", "level": 1, "level_source": "depth",
      "held_by": ["n_m0", "n_m0b"] },
    { "id": "n_m2", "path": "core/engine/m2_graph_engine", "title": "M2 Graph engine", "level": 2, "level_source": "depth",
      "held_by": ["n_m0", "n_m0b", "n_m1"] },
    { "id": "n_m3", "path": "core/engine/interchange/m3_interchange", "title": "M3 Interchange", "level": 3,
      "level_source": "depth", "held_by": ["n_m0", "n_m0b", "n_m1", "n_m2"] },
    { "id": "n_m4", "path": "core/engine/interchange/consumers/m4_mcp_server", "title": "M4 MCP server", "level": 4,
      "level_source": "depth", "held_by": ["n_m0", "n_m0b", "n_m1", "n_m2", "n_m3"] },
    { "id": "n_m5", "path": "core/engine/interchange/consumers/m5_cli", "title": "M5 CLI", "level": 4,
      "level_source": "depth", "held_by": ["n_m0", "n_m0b", "n_m1", "n_m2", "n_m3"] },
    { "id": "n_m6", "path": "core/engine/interchange/consumers/delivery/m6_apps_view", "title": "M6 Apps view",
      "level": 5, "level_source": "depth",
      "held_by": ["n_m0", "n_m0b", "n_m1", "n_m2", "n_m3", "n_m4", "n_m5"] },
    { "id": "n_m7", "path": "core/engine/interchange/consumers/delivery/m7_distribution", "title": "M7 Distribution",
      "level": 5, "level_source": "depth",
      "held_by": ["n_m0", "n_m0b", "n_m1", "n_m2", "n_m3", "n_m4", "n_m5"] },
    { "id": "n_m8", "path": "core/engine/interchange/consumers/delivery/cutover/m8_cutover", "title": "M8 Cutover",
      "level": 6, "level_source": "depth",
      "held_by": ["n_m0", "n_m0b", "n_m1", "n_m2", "n_m3", "n_m4", "n_m5", "n_m6", "n_m7"] }
  ],
  "holders": {
    "n_m0":  { "path": "m0_feasibility_spike", "title": "M0 Feasibility spike", "status": "ongoing" },
    "n_m0b": { "path": "m0b_domain_model_spec", "title": "M0b Domain model spec", "status": "ongoing" },
    "n_m1":  { "path": "core/m1_core_foundation", "title": "M1 Core foundation", "status": "planned" },
    "n_m2":  { "path": "core/engine/m2_graph_engine", "title": "M2 Graph engine", "status": "planned" },
    "n_m3":  { "path": "core/engine/interchange/m3_interchange", "title": "M3 Interchange", "status": "planned" },
    "n_m4":  { "path": "core/engine/interchange/consumers/m4_mcp_server", "title": "M4 MCP server", "status": "planned" },
    "n_m5":  { "path": "core/engine/interchange/consumers/m5_cli", "title": "M5 CLI", "status": "planned" },
    "n_m6":  { "path": "core/engine/interchange/consumers/delivery/m6_apps_view", "title": "M6 Apps view", "status": "planned" },
    "n_m7":  { "path": "core/engine/interchange/consumers/delivery/m7_distribution", "title": "M7 Distribution", "status": "planned" }
  }
}
```

### Worked example: where the scoped and the global rule differ

A hypothetical campaign with two branches. Global rule: a leaf waits for every unfinished leaf of
a smaller level, in any branch (R01's first reading).

| Path | Level | Status |
|:--|:--|:--|
| `a_setup` | 0 | `done` |
| `core/m1_core` | 1 | `paused` |
| `docs/d1_docs` | 1 | `done` |
| `core/engine/m2_engine` | 2 | `planned` |
| `docs/guide/g1_guide` | 2 | `planned` |

| Leaf | Held by, scoped | Held by, global |
|:--|:--|:--|
| `m2_engine` | `m1_core` (leaf of `core/`, above `core/engine/`) | `m1_core` |
| `g1_guide` | none: `d1_docs` is `done`, `a_setup` is `done` | `m1_core` (paused, in the unrelated branch `core/`) |

Scoped: `ready` is `[g1_guide]`, `stalled` is `[m2_engine]`. Global: `ready` is empty. The paused
leaf in `core/` halts the whole campaign under the global rule and only its own branch under the
scoped one. The scoped projection:

```json
{ "ready":   [ { "id": "n_g1", "path": "docs/guide/g1_guide", "title": "G1 Guide", "level": 2, "level_source": "depth" } ],
  "stalled": [ { "id": "n_m2", "path": "core/engine/m2_engine", "title": "M2 Engine", "level": 2, "level_source": "depth",
                 "held_by": ["n_m1"] } ],
  "holders": { "n_m1": { "path": "core/m1_core", "title": "M1 Core", "status": "paused",
                         "status_reason": "waiting for the schema review" } } }
```

## Amendment log

The amendments of a campaign, oldest first, optionally filtered. One entry per `Amendment` file.

Request parameters: `node` (an id or a path: keep amendments whose `changes` name that node),
`since` and `until` (a `ts` range, inclusive). All optional. No parameter gives the whole log.

| Field | Type | Source |
|:--|:--|:--|
| `amendments` | `AmendmentEntry[]` | The `_amendments/` files that pass the filters, ordered by ULID |
| `total` | `integer` | Derived: the count of all amendments of the campaign, before filtering |

`AmendmentEntry`:

| Field | Type | Source |
|:--|:--|:--|
| `id` | `ulid` | `Amendment.id` |
| `display_id` | `string` | Derived: `Amendment.legacy_id` when present, else `A-` and the shortest ULID suffix, at least 6 characters, unique in the campaign (`stored-schema.md`, `Amendment`) |
| `legacy_id` | `string?` | `Amendment.legacy_id` |
| `ts`, `actor`, `description` | | `Amendment.ts`, `.actor`, `.description` |
| `source` | `string?` | `Amendment.source` |
| `op_id` | `string?` | `Amendment.op_id` |
| `correction` | `bool?` | `Amendment.correction`: `true` for an edit that needed a reason on a finished node or a recorded step |
| `has_snapshot` | `bool` | Derived: `Amendment.snapshot` is present. The snapshot itself is not returned |
| `changes` | `ChangeView[]` | `Amendment.changes`, each as below |

`ChangeView` carries the stored `Change` fields (`node_id`, `op`, `path_before?`, `path_after?`,
`item_id?`, `field?`, `old?`, `new?`) and two derived ones:

| Field | Type | Source |
|:--|:--|:--|
| `node_path` | `string?` | Derived: the current path of `node_id`. Absent when the node no longer exists |
| `node_title` | `string?` | Derived: the current `title` of `node_id`. Absent when the node no longer exists |

A display id can lengthen when a later amendment shares its suffix. Cite `id` for a stable
reference.

### Worked example

The whole log of `roadmap-mcp-rework`: one amendment, imported from the legacy row `A1`.

```json
{
  "read_model_version": "0.1.0", "format_version": "0.1.0", "projection": "amendments",
  "campaign": { "id": "c_rework", "path": "__roadmap__/roadmap-mcp-rework",
                "title": "rdm rework — meta-roadmap" },
  "tree_rev": "tree-1",
  "total": 1,
  "amendments": [
    { "id": "am_a1", "display_id": "A1", "legacy_id": "A1",
      "ts": "2026-11-16T00:00:00Z", "actor": { "kind": "import" },
      "description": "No milestone owned the domain model; also edits the planned leaves m0, m1, m2, m4, m6, m7 and this README, and Eliott chose to keep the label M0b",
      "source": "__reports__/rdm_domain_model_review/01-gap_analysis_v0.md",
      "has_snapshot": false,
      "changes": [ { "node_id": "n_m0b", "op": "add", "node_path": "m0b_domain_model_spec",
                     "node_title": "M0b Domain model spec" } ] }
  ]
}
```

With `node=n_m0b` the call returns the same entry. With `node=n_m1` it returns an empty list and
`total` stays 1. `ts` is the row's date at midnight UTC, `actor.kind` is `import` and the change
has no `path_after`, as `stored-schema.md` says of a legacy row.

## Level and its source

`level` is the wave of a leaf. Today it is the depth: the number of directories between the
campaign root and the leaf. A leaf in the root has level 0. It is derived from the path and never
stored.

Every leaf in every projection carries both fields:

```json
{ "level": 2, "level_source": "depth" }
```

| Field | Type | Meaning |
|:--|:--|:--|
| `level` | `integer` | The position of the leaf in the order, from 0 |
| `level_source` | `LevelSource` | The rule that produced `level`. One value today: `depth` |

Why the source is reported: depth is the only ordering rule, and the ordering open question
(R04) keeps other rules possible. If a rule other than depth is added later, `level` changes
meaning and `level_source` says which. A consumer groups by `level` only for a `level_source` it
knows, and shows a flat list for another. A new `level_source` value is a Y change of the
read-model version (`Versions`). The ready rule does not read `level`: it reads the directory
tree, so a change of `level_source` alone cannot change the ready set.

Directories have no level.

## Versions

Two version numbers, two constants, both in `rdm-schema`. Eliott, 2026-11-16: both are
three-part, `0.X.Y`.

| Item | Format version | Read-model version |
|:--|:--|:--|
| Constant | `FORMAT_VERSION` | `READ_MODEL_VERSION` |
| Form | `"0.X.Y"` string | `"0.X.Y"` string |
| First value | `"0.1.0"` | `"0.1.0"` |
| Where it appears | `schema_version` in every stored file | `read_model_version` in every projection |
| Format of what it covers | The stored files (`stored-schema.md`) | The four projections of this document |
| Bump Y | A new optional stored field, or a new enum value old readers can ignore; a compatible fix | A new optional field, a new projection, a new request parameter, a new `level_source` value; a compatible fix |
| Bump X (breaking, while the major is 0) | A stored field removed, renamed, retyped, given another meaning, or made required | A projection field removed, renamed, retyped or given another meaning; a changed derivation (the ready rule, the order, a status derivation); a required request parameter added |
| Reader | Refuses every file newer than it, the whole triple compared, so a higher major, X or Y (Eliott, 2026-11-25), with an upgrade message. Reads an older file through the layer of its version and keeps unknown fields (I12), warning once per key | The same rule: a consumer refuses every payload newer than it, a higher Y included (Eliott, 2026-11-25), and reads an older one. It ignores unknown fields |
| Crate rule | A breaking format change forces a crate X bump. A compatible one forces at least a crate Y bump | A breaking read-model change forces a crate X bump. A compatible one forces at least a crate Y bump |

**Bumps come from commit messages.** The crate's version is decided from commit types and scopes
by the repository's existing multi-semantic-release setup (`@semantic-release/commit-analyzer`,
`release-config.js`, `.github/workflows/release.yml`, which release `skills/**` only today). A
`!` or BREAKING commit gives X and `feat` or `fix` gives Y, while the crate stays `0.x`. Wiring
the crate into that release job, with a rule that keeps a breaking change at `0.x` (semantic-release
goes to 1.0.0 by default), is M7's job. No CI check is added. "Forces at least a Y bump" means the
crate may bump more for its own reasons. While the crate is `0.x`, an X bump is the breaking one
in Rust's own rules.

A format change can leave the read model alone: a new optional stored field that no projection
shows is a format Y bump and no read-model change. A breaking format change needs a read-model review,
because a projection may read the field that changed. The read-model version moves only if a
projection field does.

`format_version` in a projection is not the read-model version. It tells a consumer which stored
format the data came from, as the highest `schema_version` among the files read.

## Gaps

No stored field a projection needs is missing. Two limits, neither one blocking:

| Limit | Effect |
|:--|:--|
| An imported leaf has no history | `status_since` is absent and `commit_count` comes from `progress.commits`. Consumers must treat both as optional |
| A display id is not stable | It can lengthen when a later amendment shares its ULID suffix. `id` is the stable key |

## Decisions

| # | Decision | Decided |
|:--|:--|:--|
| 1 | The ready set uses the scoped rule: an unfinished leaf holds back only the deeper leaves under its own parent | Eliott, 2026-11-16 |
| 2 | `level` is reported as `{level, level_source: "depth"}` | Eliott, 2026-11-16 |
| 3 | The read-model version is separate from the format version, with its own bump rule and constant. A breaking read-model change forces a crate X bump, a compatible one at least a Y bump, as the format does | coordinator, 2026-11-16 |
| 4 | `held_by` and `holders`: a ready projection names what stalls each leaf by id, with one map of holders to avoid repeating them | coordinator, 2026-11-16 |
| 5 | Sibling order is not stored. Projections order by level, then leaves before directories, then path (`traceability.md`, `Sibling order`) | Eliott, 2026-11-16 |
| 6 | Versions are three-part `0.X.Y`, first value `0.1.0`, for the format, the read model and the crate. A reader refuses a different major or a higher X and reads a higher Y (superseded by decision 10). Bumps come from commit messages through the existing release setup; M7 wires the crate | Eliott, 2026-11-16 |
| 7 | The node projections carry `marks`, the trace of criterion marks. `ready` is unchanged: it does not read pre-conditions, which gate the start separately (`statuses.md`, `Criteria`) | Eliott, 2026-11-16 (rule), coordinator (projection) |
| 8 | Sealing becomes a reason requirement, so the node projections carry `finished` (replacing `sealed`) and a derived `correction_count`, the number of records with `correction: true` that name the node. It is never stored | Eliott, 2026-11-16 |
| 9 | Projections address sub-items by `id`; `StepView.number` and `level` are display positions and never references | Eliott, 2026-11-16 |
| 10 | One version rule for stored files and projections: every version newer than the reader, a higher Y included, is refused (`is_newer`, the whole triple). A Y bump still promises additive meaning, not that older readers accept it | Eliott, 2026-11-25 (review) |
