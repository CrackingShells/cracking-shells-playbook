# Verification Gate

## Context
Depth-3 verification gate — the campaign's final integration check. Runs after `SKILL.md` is finalized and `committing-changes` is archived. Confirms the new skill is a valid release unit, packages cleanly, has no dangling links, and left no live references to the archived skill.

## Goal
Prove the refactor is release-ready and self-consistent end to end.

## Pre-conditions
- [ ] `skill_dispatcher` and `archive_committing_changes` merged into the integration branch
- [ ] Node ≥20 and `uv` available; `npm ci` run once so `node_modules/` exists

## Success Gates
- ✅ `npx multi-semantic-release --dry-run` lists `writing-history` as a package with no config errors
- ✅ `uv run tools/package_skill.py skills/writing-history dist/` emits `dist/writing-history.skill`
- ✅ Every `references/*.md` named in `SKILL.md` exists; no link resolves to the removed `git-workflow.md`
- ✅ `archive/committing-changes/` is intact and byte-identical to the pre-move skill; `skills/committing-changes/` is gone; `colgrep -e 'committing-changes'` finds no live references

## Gotchas
- The dry-run computes from `main` tags; on a feature branch it mainly confirms `writing-history` is *recognized* and its `.releaserc.js`/config load without error — treat "recognized + no error" as the gate, not a specific next version.

## Status
```mermaid
graph TD
    package_and_triggers[Verify Release, Packaging & Triggers]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `package_and_triggers.md` | 📄 Leaf Task | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
