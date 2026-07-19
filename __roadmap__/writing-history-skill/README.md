# writing-history-skill

## Context
Refactors the commit-only `committing-changes` skill into a new, broader **`writing-history`** skill spanning the whole git lifecycle — commit authoring against a *derived* (never hardcoded) vocabulary, convention/semver/changelog setup, rebase-then-`merge --no-ff` integration, and conditional worktree hygiene — applicable to code *and* scientific/document repos. Produces `skills/writing-history/` and archives the old skill unmodified. Consumed by all future git work and by skills that compose commit discipline.

## Reference Documents
- [R01 Source skill — git-workflow reference](../../skills/committing-changes/references/git-workflow.md) — durable commit-authoring content to migrate and generalize
- [R02 Source skill — SKILL.md](../../skills/committing-changes/SKILL.md) — current dispatcher whose durable directives carry over
- [R03 Release pipeline architecture](../../__reports__/skill_cicd/01-architecture_analysis_v0.md) — per-skill semantic-release/packaging machinery the new skill plugs into
- [R04 Shared release factory](../../release-config.js) — factory the skill's `.releaserc.js` delegates to

## Goal
Ship `skills/writing-history/` (SKILL.md + four references + release quartet), archive `committing-changes` unmodified, and verify release/packaging/triggers.

## Pre-conditions
- [ ] This dirtree campaign is authored and `dirtree-rdm validate` passes on every node
- [ ] `skills/committing-changes/` exists as the migration source
- [ ] `release-config.js` and `tools/package_skill.py` exist at repo root
- [ ] Node ≥20 and `uv` are available (for `multi-semantic-release` dry-run and packaging)

## Success Gates
- ✅ `skills/writing-history/` exists with a valid `SKILL.md` (`name: writing-history`, gerund, description ≤1024 chars, body ≤500 lines) delegating to `references/{commit-authoring,convention-setup,semver-changelog,branches}.md`
- ✅ No hardcoded `feat/fix/...` type table baked into the skill; the vocabulary-derivation precedence (machinery → history+docs → setup interview) is documented
- ✅ `committing-changes` moved to `archive/committing-changes/` unmodified; `colgrep -e 'committing-changes'` shows no live in-repo references
- ✅ `npx multi-semantic-release --dry-run` recognizes `writing-history` as a release unit; `uv run tools/package_skill.py skills/writing-history dist/` emits `dist/writing-history.skill`
- ✅ Every reference link in `SKILL.md`'s navigation table resolves

## Gotchas
- Root `package.json` `workspaces: ["skills/*"]` means a new `skills/writing-history/` **auto-registers** and moving `committing-changes` to `archive/` **auto-deregisters** — no `.github/workflows/release.yml` edit is needed (its paths-filter only gates managing-roadmaps' Rust binary build).
- Integration is **coordinator-only**: rebase each verified task branch onto the integration tip → re-verify → `git merge --no-ff`. Never fast-forward; never root-merge.
- `grep`/`rg` are blocked by the colgrep pre-hook — use `colgrep -e '<pattern>'` for regex/grep-parity searches.

## Status
```mermaid
graph TD
    scaffold_skill_package[Scaffold Skill Package]:::done
    author[Author Skill Content]:::done
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `scaffold_skill_package.md` | 📄 Leaf Task | ✅ Done |
| `author/` | 📁 Directory | ✅ Done |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
