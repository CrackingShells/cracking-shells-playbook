---
name: reporting
description: >
  Guides production of stakeholder-reviewable reports at key stages of software development work: architecture analysis, test definition, and knowledge transfer. Use when: (1) analyzing a proposed code change and producing an architecture report with Mermaid diagrams, contracts, alternatives, and risks; (2) defining a test plan with a risk-driven test matrix, fixtures strategy, and must-run regression set; (3) capturing post-cycle knowledge transfer notes covering wins, pain points, root causes, and next-cycle changes. Applies an architecture-first, no-code-dump contract: lead with diagrams and schemas, never include full class, module, or function bodies. Saves reports under __reports__/<topic>/ using round-versioned naming.
---

# Reporting Skill

## Core Contract

### Model-first
Lead every report with diagrams, schemas, and contracts. Never open with raw code or data dumps.

### No raw dumps
Reports must not include full implementations — no full class, module, or function bodies.
Allowed: signatures, schemas, pseudo-code, and small snippets strictly necessary to clarify a contract or invariant.

### Stakeholder-reviewable
Prefer tables, diagrams, and short sections. Avoid enumerating exhaustive permutations; use equivalence classes, boundary sets, and consolidation.

---

## File Locations and Naming

| Artifact type | Location |
|:---|:---|
| Analysis and test reports (iterative) | `__reports__/<topic>/` |
| Durable architectural decisions | `__design__/` |
| Living implementation roadmaps | `__roadmap__/<campaign>/` |

**Naming pattern**: `__reports__/<topic>/<round>-<name>_v<version>.md`

| Field | Format | Meaning |
|:---|:---|:---|
| `<topic>` | `snake_case` | Descriptive folder name for a work session |
| `<round>` | `00`, `01`, `02` … | Increments per user prompt or work session |
| `<name>` | `snake_case` | Describes the report |
| `<version>` | `v0`, `v1`, `v2` … | Increments per iteration of the same report |

**Example**: `__reports__/auth_refactor/01-test_definition_v0.md`

**Auto-create**: auto-create `__reports__/<topic>/` if it does not exist.

**README convention**: each `__reports__/<topic>/` should include a `README.md` listing documents in chronological order, marking the latest versions, and providing a short status section.

---

## Navigation

| Situation | Reference |
|:---|:---|
| Producing an architecture analysis report | [references/software-architecture.md](references/software-architecture.md) |
| Defining a test plan | [references/software-test-definition.md](references/software-test-definition.md) |
| Capturing knowledge transfer notes | [references/software-knowledge-transfer.md](references/software-knowledge-transfer.md) |

> For complex multi-phase efforts surfaced during analysis, use the `managing-roadmaps` skill to produce a formal roadmap.
