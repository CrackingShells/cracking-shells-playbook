# Create committing-changes Skill

## Context

Standalone skill creation campaign. Produces a packaged `committing-changes.skill` file
that synthesizes the org's git workflow conventions with the best patterns identified from
community and official git commit agent skills research.

## Reference Documents

- [R01 Community Git Skills Catalogue](../../__reports__/create-committing-changes-skill/content-mapping.md) — Synthesis map: org instructions vs community patterns, gap analysis, content spec for each skill file

## Goal

Build, validate, and package the `committing-changes` Agent Skill.

## Pre-conditions

- [ ] `~/.claude/skills/skill-creator/scripts/init_skill.py` exists and is executable
- [ ] `~/.claude/skills/skill-creator/scripts/package_skill.py` exists and is executable
- [ ] `instructions/git-workflow.instructions.md` exists in this repo
- [ ] Community git skill research findings are available in conversation context

## Success Gates

- ✅ `~/.claude/skills/committing-changes/` directory exists with valid structure
- ✅ `references/git-workflow.md` contains synthesized conventions from org + community
- ✅ `SKILL.md` has valid frontmatter (gerund name, ≤1024-char description) and ≤500-line body
- ✅ `package_skill.py` passes validation and emits `committing-changes.skill`

## Status

```mermaid
graph TD
    content_mapping[content_mapping.md]:::done
    init_skill[init_skill.md]:::done
    implement[implement/]:::planned
    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned fill:#374151,color:#e5e7eb
    classDef amendment fill:#1e3a5f,color:#bfdbfe
    classDef blocked fill:#7f1d1d,color:#fecaca
```

## Nodes

| Node | Type | Status |
|:-----|:-----|:-------|
| `content_mapping.md` | 📄 Leaf Task | ✅ Done |
| `init_skill.md` | 📄 Leaf Task | ✅ Done |
| `implement/` | 📁 Directory | ⬜ Planned |

## Amendment Log

| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress

| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
| `content_mapping.md` | `task/content-mapping` | 1 | Synthesis report produced |
| `init_skill.md` | `task/init-skill` | 1 | Scaffolded at skills/committing-changes/ |
| `implement/` | — | — | — |
