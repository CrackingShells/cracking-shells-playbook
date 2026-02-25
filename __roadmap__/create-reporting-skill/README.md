# Create Reporting Skill

## Context

Produces the `reporting` skill for the cracking-shells-playbook from scratch. Skill structure and all content decisions were finalized in design conversation: a general reporting skill with a universal core contract in SKILL.md and three software-domain reference files. Source material lives in `instructions/reporting*.instructions.md`.

## Reference Documents

- [reporting.instructions.md](../../instructions/reporting.instructions.md) — Core reporting contract, naming, and location rules
- [reporting-structure.instructions.md](../../instructions/reporting-structure.instructions.md) — File naming patterns and directory conventions
- [reporting-architecture.instructions.md](../../instructions/reporting-architecture.instructions.md) — Architecture analysis reporting guidance
- [reporting-tests.instructions.md](../../instructions/reporting-tests.instructions.md) — Test definition reporting guidance
- [reporting-knowledge-transfer.instructions.md](../../instructions/reporting-knowledge-transfer.instructions.md) — Knowledge transfer reporting guidance
- [reporting-templates.instructions.md](../../instructions/reporting-templates.instructions.md) — Report templates for all three types

## Goal

Deliver a fully authored and packaged `reporting` skill: `SKILL.md` + three reference files + a distributable `dist/reporting.skill`.

## Pre-conditions

- [ ] Skill structure agreed: `skills/reporting/` with flat `references/` containing three files
- [ ] Source instruction files available in `instructions/` for content extraction

## Success Gates

- ✅ `skills/reporting/SKILL.md` exists and passes frontmatter validation
- ✅ All three reference files exist at correct paths
- ✅ `dist/reporting.skill` produced without validation errors

## Gotchas

`package_skill.py` is bundled with the skill-creator skill at:
`~/.claude/plugins/cache/anthropic-agent-skills/example-skills/1ed29a03dc85/skills/skill-creator/scripts/package_skill.py`

## Status

```mermaid
graph TD
    write_skill_md[write_skill_md.md]:::done
    write_architecture_reference[write_architecture_reference.md]:::done
    write_test_definition_reference[write_test_definition_reference.md]:::done
    write_knowledge_transfer_reference[write_knowledge_transfer_reference.md]:::done
    packaging[packaging/]:::done
    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned fill:#374151,color:#e5e7eb
    classDef amendment fill:#1e3a5f,color:#bfdbfe
    classDef blocked fill:#7f1d1d,color:#fecaca
```

## Nodes

| Node | Type | Status |
|:-----|:-----|:-------|
| `write_skill_md.md` | 📄 Leaf Task | ✅ Done |
| `write_architecture_reference.md` | 📄 Leaf Task | ✅ Done |
| `write_test_definition_reference.md` | 📄 Leaf Task | ✅ Done |
| `write_knowledge_transfer_reference.md` | 📄 Leaf Task | ✅ Done |
| `packaging/` | 📁 Directory | ✅ Done |

## Amendment Log

| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress

| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
| `write_skill_md.md` | `task/write-skill-md` | 1 | |
| `write_architecture_reference.md` | `task/write-architecture-reference` | 1 | |
| `write_test_definition_reference.md` | `task/write-test-definition-reference` | 1 | |
| `write_knowledge_transfer_reference.md` | `task/write-knowledge-transfer-reference` | 1 | |
| `packaging/` | `task/package-and-validate` | 1 | |
