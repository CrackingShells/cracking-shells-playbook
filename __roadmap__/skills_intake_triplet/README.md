# Skills Intake Triplet

## Context
Three skills authored in other projects (`writing-prose`, `writing-design-canons`, `importing-design-figures`) were generalized and transferred into this bank as untracked directories. Sonnet 5 standalone audits found `writing-prose` clean and the other two carrying dead origin references (sift/glean/riken paths and aliases; ECL/VN1/168mm/font/color constants); this campaign fixes them and wires all three into the per-skill semantic-release train (release quartet per skill, precedent `writing-history@1.0.0`), ending with a push of `main` that auto-tags the three `<skill>@1.0.0` releases.

## Goal
Land the three transferred skills on `main` fully standalone and release-wired, with CI cutting `writing-prose@1.0.0`, `writing-design-canons@1.0.0`, and `importing-design-figures@1.0.0`.

## Pre-conditions
- [ ] Pristine untracked skill dirs present in the primary checkout at `skills/writing-prose/`, `skills/writing-design-canons/`, `skills/importing-design-figures/`
- [ ] Standalone audit findings recorded (plan file + leaf prose) for both NEEDS-FIXES skills
- [ ] `main` clean of tracked changes; `npm ci`/`node_modules` and `uv` available for packaging checks

## Success Gates
- ✅ `grep -rniE 'sift|glean|riken|ECL|VN1' skills/<each>/` returns no origin hits in all three skill dirs `[static]`
- ✅ Each skill has `package.json` (name matches dir, version 1.0.0), `.releaserc.js` requiring `../../release-config`, and a seed `CHANGELOG.md` `[static]`
- ✅ `uv run tools/package_skill.py skills/<each> dist/` exits 0 for all three `[run]`
- ✅ `.DS_Store` gitignored repo-wide; no stray `.DS_Store` tracked `[static]`
- ✅ After push: release workflow green and tags `writing-prose@1.0.0`, `writing-design-canons@1.0.0`, `importing-design-figures@1.0.0` exist `[behavioral]`

## Gotchas
The three skill dirs are untracked in the primary checkout: implementer worktrees branch from `main` and must copy the pristine source from the primary checkout path; before each merge the coordinator must move the untracked original aside or `git merge` refuses to overwrite untracked files.

## Status
```mermaid
graph TD
    intake_writing_prose[Intake writing-prose]:::planned
    intake_writing_design_canons[Intake writing-design-canons]:::planned
    intake_importing_design_figures[Intake importing-design-figures]:::planned
    finalize[Finalize Intake]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `intake_writing_prose.md` | 📄 Leaf Task | ⬜ Planned |
| `intake_writing_design_canons.md` | 📄 Leaf Task | ⬜ Planned |
| `intake_importing_design_figures.md` | 📄 Leaf Task | ⬜ Planned |
| `finalize/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
