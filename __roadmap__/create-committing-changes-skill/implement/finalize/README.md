# Finalize and Package

## Context

Executed after `git_workflow_ref.md` and `skill_md.md` are both complete. Both skill
files must exist and pass manual review before packaging. Produces the distributable
`committing-changes.skill` file.

## Reference Documents

- [R01 Synthesis Map](../../../../__reports__/create-committing-changes-skill/content-mapping.md) — success criteria for skill content; used to verify files before packaging

## Goal

Package the `committing-changes` skill into a valid `.skill` distribution file.

## Pre-conditions

- [ ] `~/.claude/skills/committing-changes/SKILL.md` exists with valid frontmatter
- [ ] `~/.claude/skills/committing-changes/references/git-workflow.md` exists
- [ ] No example placeholder files remain in `scripts/` or `assets/` (cleaned in `init_skill.md`)

## Success Gates

- ✅ `package_skill.py` validation passes with zero errors
- ✅ `committing-changes.skill` file produced in output directory
- ✅ `.skill` file is a valid zip containing expected directory structure

## Gotchas

- Validation checks YAML frontmatter format, naming conventions, description completeness — fix errors before re-running
- If `scripts/` or `assets/` directories still contain init-generated example files, packaging may warn; delete them first

## Status

```mermaid
graph TD
    package_validate[package_validate.md]:::done
    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned fill:#374151,color:#e5e7eb
    classDef amendment fill:#1e3a5f,color:#bfdbfe
    classDef blocked fill:#7f1d1d,color:#fecaca
```

## Nodes

| Node | Type | Status |
|:-----|:-----|:-------|
| `package_validate.md` | 📄 Leaf Task | ✅ Done |

## Amendment Log

| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress

| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
| `package_validate.md` | `task/package-validate` | 1 | Validation PASS; dist/committing-changes.skill produced |
