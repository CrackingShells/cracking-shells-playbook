---
name: managing-roadmaps
description: "Guides roadmap creation, execution, and amendment for graph-based campaign planning. Supports multiple backends (directory tree, graph database). Use when: (1) defining a campaign roadmap graph (including __roadmap__/ directory trees), (2) authoring leaf tasks and step sequences, (3) executing a campaign via BFS traversal, (4) updating roadmap status, or (5) managing amendments."
---

---

# Managing Roadmaps

This skill provides complete lifecycle guidance for roadmap management: creation, execution, and amendment.

## Quick Use Cases

- **Understanding the graph model**: See [references/graph-model.md](references/graph-model.md) for node types, status FSM, traversal contract, and backend operations
- **Creating a roadmap** (directory tree): See [references/dirtree-authoring.md](references/dirtree-authoring.md) for structure, templates, and CRUD Create
- **Executing tasks** (directory tree): See [references/dirtree-execution.md](references/dirtree-execution.md) for BFS traversal, step discipline, and git workflow
- **Making changes**: See [references/amendments.md](references/amendments.md) for the amendment cycle
- **Validating compliance** (directory tree): See [references/dirtree-schema-validation.md](references/dirtree-schema-validation.md) for naming rules and schema requirements
- **Worked examples** (directory tree): See [references/dirtree-tier-examples.md](references/dirtree-tier-examples.md) for Tier 1/2/3 examples

---

## Roadmap Architecture

### Core Concept

Roadmaps are **rooted DAGs** where depth encodes execution ordering, siblings are parallel, and leaves execute before branches at each level.

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
`planned → amendment → inprogress → done` (after an approved amendment)
`planned | inprogress → blocked` (terminal within a campaign)

See [references/graph-model.md](references/graph-model.md) for the complete FSM with all valid transitions.

---

## Backend Strategies

| Backend | Status | Authoring | Execution | Validation | Examples |
|:--------|:-------|:----------|:----------|:-----------|:---------|
| Directory Tree | **default** | [dirtree-authoring.md](references/dirtree-authoring.md) | [dirtree-execution.md](references/dirtree-execution.md) | [dirtree-schema-validation.md](references/dirtree-schema-validation.md) | [dirtree-tier-examples.md](references/dirtree-tier-examples.md) |
| ArangoDB | TBD | — | — | — | — |
| JSON | TBD | — | — | — | — |

---

## Cross-References

| Reference | What it covers |
|:----------|:---------------|
| [references/graph-model.md](references/graph-model.md) | Abstract graph semantics: node types, edge rules, status FSM, BFS traversal, step model, amendment algebra, failure escalation, backend contract |
| [references/amendments.md](references/amendments.md) | Full amendment cycle: discovery, gap analysis, review, roadmap update, tracking |
| [references/dirtree-authoring.md](references/dirtree-authoring.md) | Directory structure rules, README and leaf task templates, CRUD Create and Read |
| [references/dirtree-execution.md](references/dirtree-execution.md) | BFS traversal algorithm, step execution, git workflow, progress tracking, failure handling, CRUD Update and Delete |
| [references/dirtree-schema-validation.md](references/dirtree-schema-validation.md) | Naming conventions, node ID patterns, step validation, status values, compliance rules |
| [references/dirtree-tier-examples.md](references/dirtree-tier-examples.md) | Worked examples for Tier 1 (patch), Tier 2 (feature), Tier 3 (campaign) |

---

## Key Principles

1. **Parallelize First**: Default to siblings; use sequential depth only when a concrete dependency requires it (see [references/dirtree-authoring.md § Dependency Signals](references/dirtree-authoring.md)).
2. **Consistency over Convenience**: The status view is the single source of truth for graph state.
3. **Explicit Dependencies**: Use graph depth, not ad-hoc edges or node names, for ordering.
4. **Breadth-First Discipline**: Complete all siblings before descending to the next depth level.
5. **Execution Transparency**: Every step snapshot recorded, every status updated immediately.
6. **Living Documentation**: Status views evolve with the work as each node completes.
7. **Progressive Disclosure**: Core invariants in SKILL.md, detailed specs in backend reference files.
8. **Iterative Improvement**: Amended graphs share the same model — only nodes are added, never removed.
9. **Detection before Correction**: Exhaust the failure escalation ladder before raising an amendment.
