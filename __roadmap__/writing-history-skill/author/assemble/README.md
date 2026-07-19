# Assemble & Retire

## Context
Depth-2 assembly stage. Runs after the four references are authored and merged. Finalizes the `SKILL.md` dispatcher (navigation table + primary directive linking all four references) and archives the old `committing-changes` skill unmodified while repointing in-repo references. Its `verify/` subdirectory runs the end-to-end release/packaging/trigger gate last.

## Goal
Finalize `SKILL.md` and archive `committing-changes`, leaving the skill ready for verification.

## Pre-conditions
- [ ] All four `references/*.md` merged into the integration branch
- [ ] `skills/committing-changes/` still present (archive step consumes it after migration is confirmed)

## Success Gates
- ✅ `SKILL.md` navigation table links all four references and every link resolves
- ✅ `skills/committing-changes/` moved to `archive/committing-changes/` unmodified; in-repo references repointed to `writing-history`
- ✅ `dirtree-rdm validate` passes on this subtree

## Gotchas
- `skill_dispatcher.md` and `archive_committing_changes.md` are **parallel siblings** — they edit disjoint paths (`skills/writing-history/SKILL.md` vs. moving `skills/committing-changes/` + repointing external refs), so they integrate independently.
- `archive_committing_changes` must confirm the migrated content landed in `commit-authoring.md` before removing the source `git-workflow.md`.

## Status
```mermaid
graph TD
    skill_dispatcher[Finalize SKILL.md Dispatcher]:::planned
    archive_committing_changes[Archive committing-changes]:::planned
    verify[Verification Gate]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `skill_dispatcher.md` | 📄 Leaf Task | ⬜ Planned |
| `archive_committing_changes.md` | 📄 Leaf Task | ⬜ Planned |
| `verify/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
