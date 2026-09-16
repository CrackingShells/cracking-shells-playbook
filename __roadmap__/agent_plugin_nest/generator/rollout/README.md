# Rollout

## Context
Sits below `generator/` because materialising five real plugin trees and wiring their releases needs
the finished, checked generator shape. Turns five playbook skills into five plugins and makes every
release write the version the loaders resolve. Produces the `plugins/` tree and the release wiring;
`verify/` consumes both.

## Goal
Five playbook skills exist as five installable plugins whose versions bump on every release.

## Pre-conditions
- [ ] `generator_reshape` is done and its checker passes
- [ ] `nest_catalogue` is done, so the entries these plugins must match already exist

## Success Gates
- ✅ `plugins/` holds exactly five plugin roots, each with `plugin.json`, `.claude-plugin/plugin.json` and `skills/<name>/SKILL.md` [static]
- ✅ `npx multi-semantic-release --dry-run` completes without error and names all five packages [run]
- ✅ Each plugin's `plugin.json` version matches its skill's `package.json` version [run]
- ✅ `skills/spawning-agent-plugins/` has `package.json`, `.releaserc.js` and `CHANGELOG.md` [static]

## Status
```mermaid
graph TD
    playbook_plugins[Playbook Plugins]:::inprogress
    release_wiring[Release Wiring]:::planned
    verify[Verification]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `playbook_plugins.md` | 📄 Leaf Task | 🔄 In Progress |
| `release_wiring.md` | 📄 Leaf Task | ⬜ Planned |
| `verify/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
