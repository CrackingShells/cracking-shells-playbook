---
applyTo: '**/*'
description: '3-stage code-change workflow: Analysis → Roadmap → Execution'
---

# Development Workflow

All code changes follow a 3-stage workflow: **Analysis → Roadmap → Execution**. The stages are sequential — each has clear deliverables and exit criteria before the next begins.

---

## Stage 1: Analysis (iterative, agent-reviewed)

Understand the problem, design the solution, define the tests.

```
Architecture Report (v0) → Agent Review → Architecture Report (v1) → ...
Test Definition (v0) → Agent Review → Test Definition (v1) → ...
```

**Agents**: Any combination of human stakeholders, LLM agents, or automated review tools. "Agent" is used generically throughout.

### Activities

1. **Codebase assessment** — code review, dependency analysis, existing patterns
2. **Requirements analysis** — functional/non-functional requirements, edge cases, constraints
3. **Architecture design** — proposed changes, contracts/invariants, alternatives, risks
4. **Test strategy** — risk-driven test matrix, fixtures strategy, regression set
5. **Iterative refinement** — stakeholder feedback, architecture adjustments

### Analysis Accuracy Verification

- **File references**: confirm all referenced paths exist and are spelled correctly
- **Code snippets**: verify against actual implementation (copy-paste, don't paraphrase)
- **Dependencies**: check actual versions in package files
- **Context timestamp**: document when analysis was performed (commit SHA or date)

### Deliverables

Versioned reports in `__reports__/<topic>/` following [reporting guidelines](./reporting.instructions.md):
- Architecture reports: [reporting-architecture.instructions.md](./reporting-architecture.instructions.md)
- Test definition reports: [reporting-tests.instructions.md](./reporting-tests.instructions.md)

### Exit Criteria

Reviewing agent approves the architecture and test strategy.

---

## Stage 2: Roadmap (plan the work)

Translate the approved design into an executable directory tree.

### Activities

1. Create `__roadmap__/<campaign>/` directory
2. Write root `README.md` with context, reference documents, goal, pre-conditions, success gates
3. Identify the dependency graph of the work
4. Map the dependency graph to a directory tree:
   - Parallel work → sibling leaves
   - Sequential dependencies → nesting depth (dependent work goes deeper)
   - Diamond dependencies → place dependent node deeper than all parents
5. For each leaf task: write Goal, Pre-conditions, Success Gates, Steps with Implementation Logic and References
6. Verify the 1:1 mapping invariant (README.md nodes = filesystem entries)

### Deliverables

Complete `__roadmap__/<campaign>/` directory tree per [roadmap generation guidelines](./roadmap-generation.instructions.md).

### Exit Criteria

Reviewing agent approves. All leaf tasks have Implementation Logic and References. 1:1 mapping holds. Tree structure correctly encodes all dependencies.

---

## Stage 3: Execution (breadth-first tree traversal)

Navigate the roadmap tree and implement.

### The Algorithm

1. Enter the directory, read `README.md`
2. Execute all **leaf files** at this level (they are parallel — can be assigned to parallel agents)
3. When all leaves are done, enter each **subdirectory** (parallel with each other)
4. Within each subdirectory, recurse from step 1
5. Within a leaf task, for each step:
   - Read Implementation Logic and References
   - Produce the deliverables
   - Run consistency checks
   - Commit with the prescribed message (**1 step = 1 commit**)
6. When a leaf task's success gates are met: merge task branch into milestone
7. When all nodes in a directory are done: mark the directory node as `done` in the parent `README.md`
8. If amendment needed: follow the amendment workflow in [roadmap-generation.instructions.md](./roadmap-generation.instructions.md)
9. Repeat until all nodes at all levels are `done`

### Git Integration

- `milestone/<campaign>` branch from `dev` — single integration branch
- `task/<name>` branches from milestone — one per leaf task, flat hierarchy
- Breadth-first merge order: all depth-d tasks merge before depth d+1 begins
- See [git-workflow-milestone.instructions.md](./git-workflow-milestone.instructions.md) for details

### Deliverables

Code, tests, documentation — all as prescribed by task files. Test logs for agent review.

### Exit Criteria

All nodes `done` at every level. All tests pass. Final agent review.

---

## What This Replaces

| Previous Model | Current Model | Why |
|:---|:---|:---|
| Phase 1: Architectural Analysis | Stage 1: Analysis | Same purpose, now explicitly iterative with agent review |
| Phase 2: Test Suite Development | Stage 1: Analysis (test definition reports) | Tests defined in analysis, implemented during execution |
| Phase 3: Core Implementation | Stage 3: Execution | Driven by breadth-first tree traversal, not a phase |
| Phase 4: Debugging to 100% | Stage 3: Execution (consistency checks per step) | Integrated into each step |
| Phase 5: Git Commits | Stage 3: Execution (commit per step) | Prescribed in task files |
| Phase 6: Documentation | Stage 3: Execution (doc tasks in tree) | Documentation tasks are leaf nodes |
| Phase 7: Doc Commits | Stage 3: Execution | No separate phase needed |

Everything after analysis is encoded in the roadmap directory tree. Tests, code, documentation, and commits are all steps in leaf task files, ordered by tree depth. The tree IS the execution plan.
