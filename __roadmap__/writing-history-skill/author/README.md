# Author Skill Content

## Context
Depth-1 authoring stage. Runs after `scaffold_skill_package` is merged (skill directory, release quartet, and a minimal valid `SKILL.md` exist). Produces the four `references/*.md` files that hold the skill's substance. The `assemble/` subdirectory (finalize dispatcher + archive old skill + verify) runs after these four land.

## Reference Documents
- [R01 git-workflow reference (source)](../../../skills/committing-changes/references/git-workflow.md) — durable commit-authoring rules to migrate/generalize into `commit-authoring.md`

## Goal
Author the four reference files: `commit-authoring.md`, `convention-setup.md`, `semver-changelog.md`, `branches.md`.

## Pre-conditions
- [ ] `scaffold_skill_package` merged into the integration branch
- [ ] `skills/writing-history/references/` directory exists

## Success Gates
- ✅ All four `references/*.md` exist with the content specified in their leaf tasks
- ✅ Cross-references between the four files (e.g. `commit-authoring.md` → `convention-setup.md`/`semver-changelog.md`) point to real files
- ✅ No file re-hardcodes a `feat/fix/...` vocabulary; vocabulary is derived per `semver-changelog.md`

## Gotchas
- The four reference leaves are **parallel siblings** — independent files, dispatchable to parallel implementer subagents once the scaffold is merged.
- `SKILL.md` is **not** edited at this stage; the `skill_dispatcher` leaf (in `assemble/`) finalizes the navigation table after these files exist.
- Cross-file link consistency is gated downstream by `skill_dispatcher` and `verify/`.

## Status
```mermaid
graph TD
    commit_authoring[Author commit-authoring Reference]:::inprogress
    convention_setup[Author convention-setup Reference]:::done
    semver_changelog[Author semver-changelog Reference]:::done
    branches[Author branches Reference]:::done
    assemble[Assemble & Retire]:::done
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `commit_authoring.md` | 📄 Leaf Task | 🔄 In Progress |
| `convention_setup.md` | 📄 Leaf Task | ✅ Done |
| `semver_changelog.md` | 📄 Leaf Task | ✅ Done |
| `branches.md` | 📄 Leaf Task | ✅ Done |
| `assemble/` | 📁 Directory | ✅ Done |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
