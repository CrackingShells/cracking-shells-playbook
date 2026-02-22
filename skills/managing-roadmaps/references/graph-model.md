# Graph Model

Formal semantics for roadmap graphs. All backends implement these contracts; backend-specific authoring and execution details live in the `dirtree-*` reference files.

## Contents

- [Node Types](#node-types)
- [Edge Semantics](#edge-semantics)
- [Status State Machine](#status-state-machine)
- [Traversal Contract](#traversal-contract)
- [Step Model](#step-model)
- [Amendment Algebra](#amendment-algebra)
- [Failure Escalation Ladder](#failure-escalation-ladder)
- [Backend Contract](#backend-contract)

---

## Node Types

Two node types exist. Every node is one or the other — never both.

**Branch node** — contains child nodes, no direct steps:

| Field | Required | Description |
|:------|:--------:|:------------|
| `id` | ✓ | Stable identifier; pattern `^[a-z][a-z0-9_]*$` |
| `title` | ✓ | Descriptive name for this level |
| `context` | ✓ | Where this node fits in the parent campaign |
| `goal` | ✓ | One-line objective |
| `preconditions` | ✓ | Measurable entry criteria |
| `success_gates` | ✓ | Measurable completion criteria |
| `status_view` | ✓ | Visualization of all children and their statuses |
| `nodes_table` | ✓ | 1:1 mapping of children to backend entries |
| `amendment_log` | ✓ | Log of all amendments (may be empty) |
| `progress` | ✓ | Per-child execution progress |
| `reference_documents` | ✓ | Links to analysis reports; required before any roadmap creation or amendment |

**Leaf node** — atomic unit of work, no children:

| Field | Required | Description |
|:------|:--------:|:------------|
| `id` | ✓ | Stable identifier |
| `title` | ✓ | Component or topic name |
| `goal` | ✓ | One-line objective |
| `preconditions` | ✓ | Measurable entry criteria |
| `success_gates` | ✓ | Measurable completion criteria for whole task |
| `steps` | ✓ | Ordered list of 1–5 steps |
| `references` | ✓ | Architecture reports or supporting documents informing this task |

---

## Edge Semantics

- **Parent → child**: scoped dependency. A child cannot begin until its parent's siblings at the same depth are complete.
- **Depth encodes ordering**: nodes at depth N+1 execute after all nodes at depth N within the same parent.
- **Siblings are parallel**: children of the same parent carry no mutual ordering — execute concurrently.
- **No sibling edges**: never add an explicit dependency between siblings; use depth instead.

---

## Status State Machine

Valid statuses: `planned`, `inprogress`, `done`, `blocked`, `amendment`.

```
planned ──→ inprogress ──→ done
   │              │
   └──→ amendment ┘
         (returns to inprogress on approval)

planned ──→ blocked
inprogress ──→ blocked
```

Rules:
- `amendment` is a transient state; node returns to `inprogress` after amendment is approved.
- `blocked` is terminal within a campaign; never auto-resolve.
- `done` is immutable; reopen only via a new amendment node.

---

## Traversal Contract

BFS, leaves before branches at every level:

```
ENTER node
  IDENTIFY leaf children and branch children
  EXECUTE all leaf children (parallel)
  WAIT for all leaf children to complete
  EXECUTE all branch children (parallel — each recurses this algorithm)
  WAIT for all branch children to complete
  MARK node as done
EXIT node
```

Invariant: a branch node is marked `done` only after every descendant is `done`.

---

## Step Model

Each leaf contains 1–5 steps. Steps are strictly sequential.

| Field | Required | Description |
|:------|:--------:|:------------|
| `number` | ✓ | Sequential integer (1–5) |
| `title` | ✓ | Short descriptive title |
| `goal` | ✓ | Unique change intent for this step |
| `implementation_logic` | ✓ | What and why (no code dumps; pseudocode for complex logic) |
| `deliverables` | ✓ | Concrete outputs with scope |
| `consistency_checks` | ✓ | Verifiable check with expected outcome (PASS or FAIL) |
| `commit` | ✓ | Git commit message (conventional format: `type(scope): description`); every step must produce exactly one commit |
| `references` | — | Step-level supporting documents |
| `requires` | — | Non-obvious prerequisites only |

**Behavioral rule**: 1 step = 1 git commit. Never batch two steps into one commit; never skip a step silently.

---

## Amendment Algebra

Amendments are **additive only** — existing nodes are never renamed, renumbered, or removed.

Amendment log entry fields:

| Field | Required | Description |
|:------|:--------:|:------------|
| `id` | ✓ | Sequential: `A1`, `A2`, … |
| `date` | ✓ | Approval date (YYYY-MM-DD) |
| `source` | ✓ | Reference to gap analysis in reports store |
| `nodes_added` | ✓ | Array of node IDs added: `["node_a", "node_b"]` |
| `rationale` | ✓ | One-line explanation |

Placement rule: if new work depends on existing nodes, place it deeper in the graph so BFS guarantees dependencies complete first.

---

## Failure Escalation Ladder

Before raising an amendment, exhaust all five levels:

| Level | Name | Action |
|:------|:-----|:-------|
| 1 | Self | Re-read the step, check your own logic and output |
| 2 | Downstream | Verify later steps still make sense with current output |
| 3 | Upstream | Check if a previous step produced wrong input |
| 4 | Prove | Produce a minimal reproduction demonstrating the problem |
| 5 | Amend | Gap analysis + formal review (only after levels 1–4 exhausted) |

---

## Backend Contract

Any backend must implement the following operations against the schema-defined node shapes:

| Operation | Signature | Notes |
|:----------|:----------|:------|
| `create_node` | `(id, type, fields)` | type = branch \| leaf |
| `update_status` | `(node_id, status)` | validates state machine transitions |
| `list_children` | `(node_id) → [ids]` | returns direct children only |
| `traverse_bfs` | `(root_id) → order` | leaves before branches at each level |
| `record_snapshot` | `(node_id, step_num, data)` | 1:1 with step execution |
| `store_report` | `(path, content)` | persists gap analysis or architecture report |
| `get_status_view` | `(node_id) → summary` | branch node's view of all children statuses |
