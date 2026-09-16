# Verification

## Context
Sits below `rollout/` as verification after parallel siblings: it needs both the generated `plugins/`
tree and the release wiring. Exercises the one path that catches marketplace-install failures, which
no tree-level command reproduces. Produces the evidence that gates the colgrep-mcp campaign.

## Goal
Prove a real user can add Nest and install every playbook plugin in both harnesses.

## Pre-conditions
- [ ] `playbook_plugins` and `release_wiring` are both done
- [ ] The stale `cracking-shells` registration sourced from colgrep-mcp has been removed first

## Success Gates
- ✅ `claude plugin list` shows all five plugins with no `failed to load` line [run]
- ✅ `known_marketplaces.json` shows `cracking-shells` sourced from `CrackingShells/Nest` [run]
- ✅ Each installed plugin's skills appear in a fresh session [behavioral]

## Status
```mermaid
graph TD
    end_to_end[End To End Install]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `end_to_end.md` | 📄 Leaf Task | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
