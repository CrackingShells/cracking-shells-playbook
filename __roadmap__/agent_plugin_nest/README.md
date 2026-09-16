# Agent Plugin Marketplace: Nest

## Context
`CrackingShells` is about to publish agent plugins from more than one repository, but `colgrep-mcp`
already ships a marketplace named `cracking-shells` inside its own repo. Two repositories claiming
one marketplace name is a **silent replace** in Claude Code and a hard error in Codex. This campaign
moves that catalogue into a dedicated hub, `CrackingShells/Nest`, extends the
`spawning-agent-plugins` generator to emit several plugins per repository, and turns five playbook
skills into five installable plugins without disturbing their per-skill release lines. It produces
the Nest catalogue, the extended generator, and a `plugins/` tree; `__roadmap__/nest_migration/` in
the colgrep-mcp repository consumes it.

## Reference Documents
- [R01 Implementation Plan](~/.claude/plans/good-news-overall-it-s-gleaming-wreath.md) — settled decisions, equivalence matrix, critical files
- [R02 Manifests, field by field](../../skills/spawning-agent-plugins/references/manifests.md) — what each ecosystem's manifest may contain
- [R03 Versioning](../../skills/spawning-agent-plugins/references/versioning.md) — where the version is written and who bumps it
- [R04 Traps](../../skills/spawning-agent-plugins/references/traps.md) — symptom/cause/fix for the failures that cost a cycle

## Goal
Publish every CrackingShells plugin from one `cracking-shells` marketplace owned by `CrackingShells/Nest`, with five playbook skills installable in Claude Code and Codex and conformant to Agent Plugins 1.0.

## Pre-conditions
- [x] Implementation plan reviewed and approved (`~/.claude/plans/good-news-overall-it-s-gleaming-wreath.md`)
- [x] `CrackingShells/Nest` exists and is cloned at `/Users/hacker/Documents/src/CrackingShells/Nest`
- [x] `skills/spawning-agent-plugins/` is committed — a worktree cannot check out untracked files
- [x] `uv` resolves on PATH; no mamba environment is required by this campaign

## Success Gates
- ✅ `uv run skills/spawning-agent-plugins/scripts/check_plugin.py` reports no problems for each of the five generated plugin roots [run]
- ✅ Regenerating `plugins/` leaves `git status --porcelain plugins/` empty [run]
- ✅ `claude plugin validate .` passes for the Nest marketplace and each `plugins/*/.claude-plugin/plugin.json` [run]
- ✅ After `claude plugin marketplace remove cracking-shells` and adding Nest, `known_marketplaces.json` shows `cracking-shells` sourced from `CrackingShells/Nest` [run]
- ✅ `claude plugin list` shows all five plugins installed with no `failed to load` line [run]
- ✅ Each plugin's `plugin.json` version equals its skill's `package.json` version after a release [static]

## Status
```mermaid
graph TD
    nest_catalogue[Nest Catalogue]:::done
    regeneration_guard[Regeneration Guard]:::done
    plugin_assembly_tool[Plugin Assembly Tool]:::done
    generator[Generator Extensions]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `nest_catalogue.md` | 📄 Leaf Task | ✅ Done |
| `regeneration_guard.md` | 📄 Leaf Task | ✅ Done |
| `plugin_assembly_tool.md` | 📄 Leaf Task | ✅ Done |
| `generator/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
