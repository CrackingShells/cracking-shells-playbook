---
name: managing-roadmaps
description: "Creates, executes, and amends graph-based campaign roadmaps. Use when: (1) planning a campaign or feature rollout as a structured roadmap, (2) authoring leaf tasks and step sequences, (3) executing work via BFS traversal, (4) tracking or updating node status (planned/inprogress/done/blocked), (5) restructuring roadmap nodes, (6) managing amendments, or (7) working with any __roadmap__/ directory."
---

---

# Managing Roadmaps

This skill provides complete lifecycle guidance for roadmap management: creation, execution, and amendment.

## Navigation

| Intent | File |
|:-------|:-----|
| Understand the graph model (node types, FSM, traversal contract) | [references/graph-model.md](references/graph-model.md) |
| Create a roadmap (structure, templates, naming rules) | [references/dirtree-authoring.md](references/dirtree-authoring.md) |
| Execute tasks (BFS traversal, step discipline, git workflow) | [references/dirtree-execution.md](references/dirtree-execution.md) |
| Manage amendments (cycle, gap analysis, approval) | [references/amendments.md](references/amendments.md) |
| Validate compliance (naming, schema, step rules) | [references/dirtree-schema-validation.md](references/dirtree-schema-validation.md) |
| See worked examples (Tier 1/2/3) | [references/dirtree-tier-examples.md](references/dirtree-tier-examples.md) |
| Mutate nodes without corrupting README.md (add/update/move/insert) | [references/dirtree-cli.md](references/dirtree-cli.md) |
| Debug a validation failure (BNF grammar spec) | [references/dirtree-bnf.md](references/dirtree-bnf.md) |

---

## Roadmap Architecture

### Core Concept

Roadmaps are **rooted DAGs** where depth encodes execution ordering, siblings are parallel, and leaves execute before branches at each level.

The default (and only implemented) backend is the **directory tree** under `__roadmap__/`.

### Core Invariants

1. **Generic Agent Model**: "Agent" instructions apply universally to any entity (human, LLM, automated program, team role) capable of executing work. The roadmap does not specify WHO performs work, only WHAT must be done and in what order.
2. **1:1 Representation Mapping**: Every node has exactly one representation in the backend store; the status view is the authoritative inventory.
3. **Depth IS Ordering**: A node at depth N+1 depends on completion of all nodes at depth N within the same parent.
4. **Sibling Parallelism**: Children of the same parent carry no mutual ordering — always execute concurrently.
5. **Leaf-Before-Branch**: Leaf nodes execute before branch nodes at each level.
6. **Flat Execution Contexts**: Task contexts branch flat off milestone context, not hierarchically.
7. **Step-Commit Bijection**: Each step produces exactly one git commit; never batch two steps or skip one silently.
8. **Stable Node Identity**: Node IDs are immutable after creation; naming convention is backend-defined.
9. **Status Completeness**: Every branch node exposes a status view of all children.

---

## Status Lifecycle

`planned → inprogress → done` (normal path)
`amendment → inprogress → done` (newly-added nodes introduced via amendment)
`planned | inprogress → blocked` (terminal within a campaign)

See [references/graph-model.md](references/graph-model.md) for the complete FSM with all valid transitions.

---

## Key Principles

1. **Parallelize First**: Default to siblings; use sequential depth only when a concrete dependency requires it (see [references/dirtree-authoring.md § Dependency Signals](references/dirtree-authoring.md)).
2. **Consistency over Convenience**: The status view is the single source of truth for graph state.
3. **Execution Transparency**: Every step snapshot recorded, every status updated immediately.
4. **Progressive Disclosure**: Core invariants in SKILL.md, detailed specs in backend reference files.
5. **Iterative Improvement**: Amended graphs share the same model — only nodes are added, never removed.
6. **Detection before Correction**: Exhaust the failure escalation ladder before raising an amendment.
