# Skill CI/CD Pipeline

## Context
Top-level campaign to automate skill distribution. Currently `dist/` is manually synced and committed; this campaign removes it from git, introduces per-skill semantic versioning via `semantic-release-monorepo`, and publishes `.skill` artifacts to GitHub Releases on every merge to `main`. Architecture analysis and decisions are captured in `__reports__/skill_cicd/`.

## Reference Documents
- [R01 Architecture Analysis](__reports__/skill_cicd/00-architecture_analysis_v0.md) — contracts, diagrams, alternatives, risk register
- [R02 Knowledge Transfer](__reports__/skill_cicd/00-knowledge_transfer_v0.md) — pain points, root causes, open questions
- [R03 Implementation Plan](~/.claude/plans/binary-jumping-trinket.md) — full deliverable list with decisions

## Goal
Fully automate `.skill` packaging, versioning, and GitHub Release publication with no manual `dist/` management.

## Pre-conditions
- [ ] Architecture analysis reviewed and approved (`__reports__/skill_cicd/`)
- [ ] Implementation plan reviewed and approved (`~/.claude/plans/binary-jumping-trinket.md`)
- [ ] GitHub repo has `contents: write` permission available to the default `GITHUB_TOKEN`

## Success Gates
- ✅ `dist/` is gitignored; `git status` shows no tracked files under `dist/`
- ✅ `uv run tools/package_skill.py skills/committing-changes dist/` succeeds; artifact contains no `package.json` or `CHANGELOG.md`
- ✅ Pre-commit hook auto-builds `dist/<name>.skill` on any staged skill change without blocking commits
- ✅ `npx multi-semantic-release --dry-run` completes without errors
- ✅ All 4 per-skill GitHub Actions workflows exist and pass CI lint (`actionlint`)
- ✅ `CONTRIBUTING.md` and `README.md` installation guide committed

## Status
```mermaid
graph TD
    vendor-tools[Vendor Package Skill Tools]:::planned
    dist-cleanup[Remove dist/ from Git Tracking]:::planned
    skill-manifests[Add Per-Skill package.json Markers]:::planned
    windows-dispatch[Add Windows PowerShell Dispatch Wrapper]:::planned
    local-toolchain[Local Toolchain]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `vendor-tools.md` | 📄 Leaf Task | ⬜ Planned |
| `dist-cleanup.md` | 📄 Leaf Task | ⬜ Planned |
| `skill-manifests.md` | 📄 Leaf Task | ⬜ Planned |
| `windows-dispatch.md` | 📄 Leaf Task | ⬜ Planned |
| `local-toolchain/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
