# Review and Close

## Context
Level 4, the close. V2 attacks the whole skill and runs the evals. Then the coordinator regenerates the generated plugin trees, runs the campaign's Verification gates, and writes the verification report. This produces the branch that is proposed to `main` through a Reviewed PR.

## Goal
Prove the finished skill is coherent, stand-alone and releasable, and record the proof.

## Pre-conditions
- [ ] `standalone.md` is merged

## Success Gates
- ✅ `review_and_evals.md` is done, and every V2 finding and failed eval is resolved or adjudicated in `01-verification_v0.md`
- ✅ A coordinator commit (with the hook firing) regenerates `plugins/writing-release/`; `uv run tools/spawn_plugin.py spawn --force` is re-run if `plugin.spec.json` changed; `uv run tools/check_plugin.py --root . --spec plugin.spec.json` passes; `uv run tools/assemble_plugin.py writing-release` leaves `git status --porcelain plugins/` empty
- ✅ Every campaign-README success gate is checked and recorded in `01-verification_v0.md`, including the git-log release analysis over `main..HEAD`

## Gotchas
- The multi-semantic-release dry run only proves a package is recognised on a non-`main` branch, so the git-log release analysis replaces it.
- V2's fixes go back to the agent that owns the files (via SendMessage), not to a fresh agent.

## Status
```mermaid
graph TD
    review_and_evals[Adversarial Review and Evals]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `review_and_evals.md` | 📄 Leaf Task | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
