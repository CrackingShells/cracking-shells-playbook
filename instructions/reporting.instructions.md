---
applyTo: '**/*'
description: 'Reporting core contract (architecture-first, no code dumps)'
---

# Reporting Core Contract

This is the **always-apply** reporting contract. It is intentionally short to preserve working context.

Reports exist to align stakeholders on **architecture** and **validation**, not to duplicate implementation code.

## When to produce reports
- Generate reports at key phases (especially Phase 1 and Phase 2) as described in [code-change-phases.instructions.md](./code-change-phases.instructions.md), or when explicitly requested.

## Where reports go
- Default location:
  - `__reports__/<topic>/<round>-<descriptive_name>_v<version>.md`
- Use `__design__/` for durable design/roadmaps.

Details (structure, naming, README conventions):
- [reporting-structure.instructions.md](reporting-structure.instructions.md)

## Content rules (architecture-first)

### 1) No code dumps
Reports must not include full implementations (no full modules/classes, no full function bodies).

Allowed:
- Signatures, schemas, pseudo-code, and small snippets strictly necessary to clarify a contract or invariant.

### 2) Default artifacts
- Phase 1: Mermaid diagrams + contracts/invariants + alternatives + risks.
- Phase 2: Risk-driven test matrix + fixtures strategy + minimal must-run regression set.

### 3) Keep reports reviewable
- Prefer tables, diagrams, and short sections.
- Avoid enumerating exhaustive permutations; use equivalence classes, boundary sets, and consolidation.

## Specialized reporting guidance
- Phase 1 architecture guidance: [reporting-architecture.instructions.md](reporting-architecture.instructions.md)
- Phase 2 test definition reports: [reporting-tests.instructions.md](reporting-tests.instructions.md)
- Knowledge transfer (LLM cycle learnings): [reporting-knowledge-transfer.instructions.md](reporting-knowledge-transfer.instructions.md)
- Templates: [reporting-templates.instructions.md](reporting-templates.instructions.md)

## Roadmaps (when needed)
If Phase 1 reveals a complex multi-phase effort, prefer creating a formal roadmap in `__design__/` using:
- [roadmap-generation.instructions.md](roadmap-generation.instructions.md)

