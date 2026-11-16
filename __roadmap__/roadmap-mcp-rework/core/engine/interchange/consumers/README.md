# Consumers

## Context
Depth 4. Runs after the core is complete. M4 (MCP server, for agents) and M5 (CLI, for humans) are independent thin layers over `rdm-core` and run in parallel.

## Goal
Agents and humans reach every core operation, through MCP tools and through `rdm`, with the same results.

## Pre-conditions
- [ ] `m3_interchange.md` is `done`

## Success Gates
- ⬜ `m4_mcp_server.md` and `m5_cli.md` are `done`

## Status
```mermaid
graph TD
    m4_mcp_server[M4 MCP server]:::planned
    m5_cli[M5 CLI]:::planned
    delivery[Delivery]:::planned
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `m4_mcp_server.md` | 📄 Leaf Task | ⬜ Planned |
| `m5_cli.md` | 📄 Leaf Task | ⬜ Planned |
| `delivery/` | 📁 Directory | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
