---
name: writing-reports
description: >
  Guides production of stakeholder-reviewable reports across coding, research, data analysis, and writing work. Use when producing any of these seven report types: (1) observation — peripheral finding for cold-start agent hand-off; (2) findings — data-first results briefing with tables, charts, and steering questions; (3) notice — lightweight improvement idea captured outside current task scope; (4) open-question — open question framing for fresh-agent delegation; (5) architecture — pre-implementation analysis with diagrams, contracts, and risk register; (6) test-definition — risk-driven test plan with test matrix and fixtures strategy; (7) knowledge-transfer — post-cycle retrospective covering wins, pain points, and next-cycle changes. Applies a model-first, no-code-dump contract: lead with diagrams and schemas, never include full implementations. Saves reports under __reports__/TOPIC/ using round-versioned naming.
---

# Reporting Skill

## Navigation

Read this table first. Pick the report type that matches your situation, then open the linked reference.

| Situation | Report type | Reference |
|:---|:---|:---|
| Spotted a peripheral issue, inconsistency, or anomaly mid-work; need to capture enough context for a fresh agent thread to pick it up cold | `observation` | [references/observation.md](references/observation.md) |
| Have results, measurements, data, or test outcomes to report; need a concise briefing to steer the project | `findings` | [references/findings.md](references/findings.md) |
| Noticed an improvement idea or enhancement that is outside the current task's scope; want to capture it without interrupting the current thread | `notice` | [references/notice.md](references/notice.md) |
| Have an open question that needs to be framed and delegated; no direction chosen yet | `open-question` | [references/open-question.md](references/open-question.md) |
| Analyzing a proposed change before implementation | `architecture` | [references/software-architecture.md](references/software-architecture.md) |
| Defining a test plan | `test-definition` | [references/software-test-definition.md](references/software-test-definition.md) |
| Capturing post-cycle retrospective notes | `knowledge-transfer` | [references/software-knowledge-transfer.md](references/software-knowledge-transfer.md) |

> For complex multi-phase efforts surfaced during analysis, use the `managing-roadmaps` skill to produce a formal roadmap.

---

## Core Contract

### Model-first
Lead every report with diagrams, schemas, and contracts. Never open with raw code or data dumps.

### No raw dumps
Reports must not include full implementations.
Allowed: signatures, schemas, pseudo-code, and small snippets strictly necessary to clarify a contract or invariant.

### Stakeholder-reviewable
Prefer tables, diagrams, and short sections. Avoid enumerating exhaustive permutations; use equivalence classes, boundary sets, and consolidation.

---

## File Locations and Naming

| Artifact type | Location |
|:---|:---|
| All reports (iterative) | `__reports__/<topic>/` |
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
