# Host trials

## Context
Depth 2 of the spike. It runs after the stub and the page are built. The coordinator runs it with Eliott at the keyboard, because every gate here is an observation on a real host. Its notes and probe data feed `report/`.

## Goal
Every host-facing gate of M0 is observed on Claude Code Desktop, VS Code Copilot and the Claude Code CLI, and the raw data is kept.

## Pre-conditions
- [ ] `../stub_server.md` and `../graph_view.md` are done and merged into `feat/m0-feasibility-spike`

## Success Gates
- ⬜ `__reports__/m0-feasibility-spike/trials/` holds one notes file per host, the probe JSONL (checked for personal data) and the allow-rule string

## Gotchas
- **No adversarial verifier:** the gates are observations made with Eliott.
- **Local registration only:** the stub is registered through an uncommitted project `.mcp.json`, and the allow-rule string is read through an uncommitted dev copy of the `managing-roadmaps` plugin. Neither file is committed, because M7 owns every manifest.

## Status
```mermaid
graph TD
    host_trials[Host trials]:::planned
    report[Spike report]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `host_trials.md` | 📄 Leaf Task | ⬜ Planned |
| `report/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
