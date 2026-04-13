# Local Toolchain

## Context
Sequential phase after the 4 parallel foundation tasks (`vendor-tools`, `dist-cleanup`, `skill-manifests`, `windows-dispatch`). Assembles the developer-facing tooling: the pre-commit hook that auto-builds `.skill` files on every commit, and the `Makefile` that bootstraps a contributor's environment in one command. Feeds into the CI pipeline phase.

## Reference Documents
- [R01 Implementation Plan](~/.claude/plans/binary-jumping-trinket.md) — Phase 1 deliverables §5-§6

## Goal
Wire together the vendored tools and Makefile into a working pre-commit hook that any contributor can activate with `make dev-setup`.

## Pre-conditions
- [ ] `tools/package_skill.py` and `tools/quick_validate.py` exist with PEP 723 inline deps
- [ ] `dist/` is gitignored
- [ ] All 4 `skills/<name>/package.json` markers created

## Success Gates
- ✅ `make dev-setup` completes without errors (installs hook, npm, uv)
- ✅ Staging any file under `skills/writing-reports/` and running `git commit` triggers hook and builds `dist/writing-reports.skill`
- ✅ Staging a `.rs` file under `skills/managing-roadmaps/` prints the Rust reminder without blocking the commit

## Status
```mermaid
graph TD
    hook-and-makefile[Pre-commit Hook and Makefile]:::planned
    ci-pipeline[CI Pipeline]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `hook-and-makefile.md` | 📄 Leaf Task | ⬜ Planned |
| `ci-pipeline/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
