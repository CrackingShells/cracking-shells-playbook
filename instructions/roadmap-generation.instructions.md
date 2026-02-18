---
applyTo: '**/*'
description: 'Directory-tree roadmap model: depth is ordering, siblings are parallel'
---

# Roadmap Generation Guidelines

## Overview

Roadmaps are **directory trees** under `__roadmap__/<campaign>/`. The filesystem structure IS the execution order:

- **Siblings** are always parallel (no exceptions)
- **Nesting depth** encodes sequential dependency (deeper = later)
- **Leaves before subdirectories** at every level
- **Breadth-first execution**: all of depth N completes before depth N+1

Every directory contains a `README.md` (status + context). Every leaf file contains sequential steps (1 step = 1 commit).

---

## 1. Core Invariants

### 1:1 Mapping

Every node in a `README.md` status visualization corresponds to exactly one sibling file or directory. Every sibling (except `README.md` itself) appears as a node. Verify by comparing `ls` output with the Nodes table.

### Depth IS Ordering

Within any directory:
1. Execute all **leaf files** (parallel)
2. Execute all **subdirectories** (parallel with each other)
3. Each subdirectory repeats recursively

Subdirectories represent work that depends on sibling leaves being complete.

### Diamond Dependency Resolution

When a node depends on multiple parents at different tree locations, place it **as deep as possible** — deeper than all parents. Breadth-first execution guarantees all parents complete first.

---

## 2. Directory Structure

### Root Convention

```
__roadmap__/<campaign_name>/
├── README.md
├── <leaf_task>.md                    # Parallel with other leaves
├── <another_leaf>.md                 # Parallel
└── <component_or_phase>/             # After all leaves at this level
    ├── README.md
    ├── <sub_leaf>.md
    └── <deeper_concern>/
        ├── README.md
        └── <deep_leaf>.md
```

### Naming Convention

- Directories: `snake_case/` — named after the component, concern, or phase
- Leaf task files: `snake_case.md` — named after the specific work unit
- Campaign names: descriptive kebab-case or snake_case (e.g., `cli-ux-normalization`)
- **No numeric prefixes** (`T01_`, `M1.2_`, etc.) — ordering comes from tree depth, not filenames

### When to Nest vs. Keep Flat

**Create a subdirectory** when:
- A concern decomposes into multiple parallel sub-tasks
- There is a sequential dependency (subdirectory contents depend on sibling leaves)
- The concern could be assigned to a different agent for parallel execution

**Keep as a leaf task file** when:
- The work is 1–5 sequential steps with no internal parallelism
- It represents a single logical unit (e.g., one TDD cycle)

**Promote a leaf to a subdirectory** when it outgrows ~5 steps or decomposes into 3+ independent sub-tasks.

---

## 3. The `README.md` Specification

Every directory under `__roadmap__/` contains exactly one `README.md` — the entry point for that level.

### Template

````markdown
# <Title>

## Context

<1–2 sentences: where this node fits in the parent campaign.
What it depends on, what it produces, who consumes it.>

## Reference Documents

- [R<nn> <title>](<relative path>) — <what this report covers>

## Goal

<One-line objective for this level>

## Pre-conditions

- [ ] <Measurable entry criteria>

## Success Gates

- ✅ <Measurable completion criteria>

## Gotchas

<Optional. Known issues, edge cases, or implementation notes.
Omit section entirely if nothing to flag.>

---

## Status

```mermaid
graph TD
    node_a[Node A]:::done
    node_b[Node B]:::inprogress
    node_c[Node C]:::planned

    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned fill:#374151,color:#e5e7eb
    classDef amendment fill:#1e3a5f,color:#bfdbfe
    classDef blocked fill:#7f1d1d,color:#fecaca
```

## Nodes

| Node | Type | Status |
|:-----|:-----|:-------|
| `node_a.md` | 📄 Leaf Task | ✅ Done |
| `node_b/` | 📁 Directory | 🔄 In Progress |
| `node_c.md` | 📄 Leaf Task | ⬜ Planned |

## Amendment Log

| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress

| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
````

### Status Visualization Rules

1. **Node IDs = filesystem names** (without `.md` for leaves, without `/` for directories)
2. **No edges between siblings** — siblings are always parallel; ordering comes from tree structure
3. **Status colors** (strong foreground/background contrast):

| Status | classDef | Visual |
|:-------|:---------|:-------|
| `done` | `fill:#166534,color:#bbf7d0` | Dark green bg, light green fg |
| `inprogress` | `fill:#854d0e,color:#fef08a` | Dark amber bg, light yellow fg |
| `planned` | `fill:#374151,color:#e5e7eb` | Dark gray bg, light gray fg |
| `amendment` | `fill:#1e3a5f,color:#bfdbfe` | Dark blue bg, light blue fg |
| `blocked` | `fill:#7f1d1d,color:#fecaca` | Dark red bg, light red fg |

4. **Amendments** add nodes (never rename/remove existing ones; mark abandoned as `:::blocked`)
5. For >12 nodes, use Mermaid **subgraphs** to group leaves vs. subdirectories as a readability aid

---

## 4. Leaf Task File Template

A leaf task is the atomic unit of work — sequential steps, each targeting exactly one commit.

````markdown
# <Component/Topic Name>

**Goal**: <One-line objective>
**Pre-conditions**:
- [ ] <Sibling leaves complete, environmental state, etc.>
**Success Gates**:
- ✅ <Measurable gate>
**References**: [R<nn> §<section>](<path>) — <what to find>

---

## Step 1: <Step Title>

**Goal**: <Unique change intent>

**Implementation Logic**:
<Natural language description of what to build/test/change.
NO code dumps — describe WHAT and WHY, not HOW at syntax level.
Use numbered pseudocode for non-trivial logic.
Add context-specific sections as needed (Parser Flags, Test Cases, etc.)>

**References**:
- [R<nn> §<section>](<path>) — <what to find>

**Deliverables**: <file paths> (~<LOC estimate> LOC)

**Consistency Checks**: `<command>` (expected: <PASS|FAIL>)

**Commit**: `<type>(<scope>): <description>`

---

## Step 2: <Step Title>
...
````

### Key Rules

- **Step-level Pre-conditions are NOT required.** Steps are sequential by definition. Only add a `Requires:` note for genuinely non-obvious prerequisites (e.g., "Requires: running database instance").
- **Implementation Logic** is the most important section — it constrains scope. If it's not described here, it shouldn't be implemented.
- **1 step = 1 commit.** Strict bijection.
- Steps may add context-specific sections (Parser Flags, Test Cases, numbered pseudocode) as needed — the template is a minimum, not a maximum.

### The TDD Pattern (recommended for code-producing tasks)

1. **Test step** — failing test defining the behavioral contract (expected: FAIL)
2. **Implementation step** — code that makes the test pass (expected: PASS)
3. **Verification step** (optional) — broader validation (full suite, linting)

Not mandatory for all tasks. Documentation, configuration, and deprecation tasks may have implementation-only steps.

---

## 5. Complexity Tiers

The tree depth scales naturally with complexity:

| Tier | When to use | Directory depth | Parallelism |
|:-----|:------------|:---------------|:------------|
| **Tier 1: Patch** | Bug fixes, chores | Flat (1 leaf task) | None (sequential steps inside) |
| **Tier 2: Feature** | New features, refactors | 1–2 levels of nesting | Parallel siblings appear |
| **Tier 3: Campaign** | Major initiatives | 2+ levels of nesting | Parallel groups at multiple depths |

**Tier escalation is natural**: promote a leaf to a subdirectory when it outgrows ~5 steps or decomposes into 3+ independent sub-tasks. The model doesn't change — only the depth.

See [roadmap-composition-examples.annex.md](./roadmap-composition-examples.annex.md) for worked examples at each tier.

---

## 6. Amendment Workflow

Amendments evolve a roadmap after initial creation.

### The Cycle

1. Executing agent discovers a gap or receives an enhancement request
2. Agent produces a gap analysis report in `__reports__/`
3. Agent produces an architecture report if scope warrants
4. Reviewing agent (human, LLM, or program) reviews the analysis
5. On approval: create new task files at the appropriate depth, update `README.md` (add nodes + amendment log entry)
6. Continue execution per tree structure

### Rules

1. Every amendment MUST have a source document in `__reports__/`
2. Every amendment MUST be reviewed by an agent before the plan is updated
3. New nodes use descriptive names. **Never rename or renumber existing nodes.**
4. The amendment is logged in the `README.md` at the level where the change occurs
5. New nodes use `:::amendment` styling until executed, then transition to `:::done`
6. New nodes must respect depth-ordering: if the new work depends on existing nodes, place it deeper
7. If an amendment adds enough complexity, create a new subdirectory with its own `README.md`

---

## 7. Where Roadmaps Fit

| Item | Location | Rationale |
|:-----|:---------|:----------|
| Implementation plan (directory tree) | `__roadmap__/<campaign>/` | Living plan, evolves with amendments |
| Analysis reports, test definitions | `__reports__/<topic>/` | Iterative analysis, informs the roadmap |
| Durable architectural decisions | `__design__/` | Outlives any single campaign |

A campaign in `__roadmap__/` typically has a corresponding topic in `__reports__/` with the same or similar name.

**Cross-references**:
- Leaf task References point to `__reports__/` for architecture context
- `README.md` Reference Documents link to reports that inform each level
- Gap analyses in `__reports__/` reference `__roadmap__/` for the plan they amend

---

## Cross-References

- [roadmap-execution.instructions.md](./roadmap-execution.instructions.md) — how to navigate and execute against a roadmap tree
- [roadmap-composition-examples.annex.md](./roadmap-composition-examples.annex.md) — worked examples at each tier
- [code-change-phases.instructions.md](./code-change-phases.instructions.md) — the 3-stage workflow
- [git-workflow-milestone.instructions.md](./git-workflow-milestone.instructions.md) — flat branching model
