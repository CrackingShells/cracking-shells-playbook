# Generator Extensions

## Context
Sits below the campaign root, after the assembly tool exists, because the generator's per-plugin
loop invokes it. Teaches `spawn_plugin.py` to emit several sibling plugins from one spec and to
target a marketplace owned by another repository, and teaches `check_plugin.py` to validate that
shape. Produces the generator every downstream leaf depends on; `rollout/` consumes it.

## Goal
Make the generator and checker capable of multi-plugin, hub-aware, Agent-Plugins-first output without breaking colgrep-mcp's existing manifests beyond the one deliberate reshape.

## Pre-conditions
- [ ] `plugin_assembly_tool` is done — the per-plugin loop calls it
- [ ] `regeneration_guard` is done and green on the pre-change tree, so any drift is attributable

## Success Gates
- ✅ `uv run skills/spawning-agent-plugins/scripts/spawn_plugin.py spawn --spec <multi-plugin spec> --dry-run` writes one manifest set per declared plugin [run]
- ✅ The regeneration guard passes against its deliberately re-baselined colgrep-mcp expectation [run]
- ✅ `check_plugin.py` reports no problems on a repo holding two sibling plugins at different versions [run]
- ✅ No call path passes `force=True` to `Writer.merge_marketplace` [static]

## Status
```mermaid
graph TD
    generator_reshape[Generator Reshape]:::done
    reference_docs[Reference Documentation]:::done
    rollout[Rollout]:::done
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `generator_reshape.md` | 📄 Leaf Task | ✅ Done |
| `reference_docs.md` | 📄 Leaf Task | ✅ Done |
| `rollout/` | 📁 Directory | ✅ Done |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
