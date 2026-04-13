# CI Pipeline

## Context
Sequential phase after the pre-commit hook is working. Wires `semantic-release-monorepo` to GitHub Actions with one workflow per skill. `sr-config` and `workflows` are independent and run in parallel. Documentation runs after both are verified. Produces automated GitHub Releases with `.skill` assets on every push to `main`.

## Reference Documents
- [R01 Implementation Plan](~/.claude/plans/binary-jumping-trinket.md) — Phase 2 §7-§9, Phase 3 §10-§11

## Goal
Publish versioned `.skill` artifacts to GitHub Releases automatically on every skill change merged to `main`.

## Pre-conditions
- [ ] Pre-commit hook and Makefile working (`local-toolchain` complete)
- [ ] GitHub repo `GITHUB_TOKEN` has `contents: write` permission

## Success Gates
- ✅ `npx multi-semantic-release --dry-run` exits 0 with expected tag format (`<name>@1.0.0`)
- ✅ All 4 workflow files pass `actionlint` with no errors
- ✅ `CONTRIBUTING.md` and `README.md` installation guide committed to `main`

## Status
```mermaid
graph TD
    sr-config[Semantic Release Config]:::planned
    workflows[Per-Skill GitHub Actions Workflows]:::planned
    documentation[Documentation]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `sr-config.md` | 📄 Leaf Task | ⬜ Planned |
| `workflows.md` | 📄 Leaf Task | ⬜ Planned |
| `documentation/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
