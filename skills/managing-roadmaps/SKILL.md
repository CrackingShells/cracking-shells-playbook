---
name: managing-roadmaps
description: "Creates, executes, and amends graph-based campaign roadmaps. Use when: (1) planning a campaign or feature rollout as a structured roadmap, (2) authoring leaf tasks and step sequences, (3) executing work via BFS traversal, (4) tracking or updating node status (planned/inprogress/done/blocked), (5) restructuring roadmap nodes, (6) managing amendments, or (7) working with any __roadmap__/ directory."
---

---

# Managing Roadmaps

## The One Rule

**Never write or edit files under `__roadmap__/` by hand.** Use `dirtree-rdm` for every mutation:

```bash
bash skills/managing-roadmaps/scripts/dirtree-rdm.sh <command> [args]
```

Manual edits corrupt the Mermaid graph and Nodes table. `dirtree-rdm` validates the BNF grammar before and after every write — no silent corruption.

---

## What Are You Doing?

Read the listed files **before starting**. Each contains the file format, rules, and examples required to do the task correctly. Do not proceed on intuition.

| Task | Read before starting | Also useful |
|:-----|:---------------------|:------------|
| **Create a roadmap** — new campaign, milestone, or leaf tasks | [dirtree-authoring.md](references/dirtree-authoring.md) — structure rules, dependency signals, templates | [dirtree-tier-examples.md](references/dirtree-tier-examples.md) for Tier 1/2/3 worked examples |
| **Add, update, move, or insert nodes** (any mutation) | [dirtree-cli.md](references/dirtree-cli.md) — all `dirtree-rdm` commands with examples | — |
| **Execute tasks** — BFS traversal, step-by-step work, git workflow | [dirtree-execution.md](references/dirtree-execution.md) — traversal algorithm, progress tracking, failure handling | — |
| **Amend a roadmap** — add nodes due to scope change or gap | [amendments.md](references/amendments.md) — gap analysis, approval cycle, amendment log format | — |
| **Debug a validation failure** — `dirtree-rdm validate` reported errors | [dirtree-bnf.md](references/dirtree-bnf.md) — BNF grammar, line-by-line production names | — |
| **Understand the graph model** — node types, FSM, traversal contract | [graph-model.md](references/graph-model.md) — formal edge semantics, status machine, step model | — |

---

## Critical Concept: Depth Is Ordering — Not Mermaid Edges

Siblings are **parallel**. Sequential dependencies are encoded by **directory depth**, not by Mermaid `-->` edges.

`-->` edges between siblings are **forbidden by the BNF** and will cause `dirtree-rdm` to reject the file.

If task B must follow task A, place B one directory level deeper — not as a sibling with an arrow:

```
# Wrong — edges between siblings
task_a[Task A]:::planned --> task_b[Task B]:::planned

# Right — B is deeper; BFS guarantees A finishes before B's level runs
__roadmap__/campaign/task_a.md        ← leaf (runs first)
__roadmap__/campaign/next/task_b.md   ← one level deeper (runs after)
```

Read [graph-model.md § Edge Semantics](references/graph-model.md) before designing any roadmap structure.

---

## Status Values

| Value | Emoji | Meaning |
|:------|:------|:--------|
| `planned` | ⬜ | Not yet started |
| `inprogress` | 🔄 | Currently being executed |
| `done` | ✅ | Complete and success gates met |
| `amendment` | 🔵 | Birth state of nodes added via the amendment process |
| `blocked` | 🚫 | Terminal — work cannot proceed; never delete, mark blocked instead |

Full FSM with all valid transitions: [graph-model.md § Status State Machine](references/graph-model.md).
