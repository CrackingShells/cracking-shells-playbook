# Claude Code CLI trial

CLI version 2.1.316, run with `claude -p` on 2026-11-16, using the project `.mcp.json` from `feat-m0-feasibility-spike`.

- **Tools:** `roadmap_view` is deferred and found through ToolSearch as `mcp__rdm__roadmap_view`. It was called with `{mode: meta, scale: 1}` and returned the 2026-07-28 summary (`node_count` 16). The server refuses any other protocol version, so the CLI speaks 2026-07-28. Cost: USD 0.089.
- **Allow rule:** `--allowedTools mcp__rdm__roadmap_view` let the call run without a prompt.
- **Rendering:** the CLI does not render MCP Apps. Only the tool call was tested.
