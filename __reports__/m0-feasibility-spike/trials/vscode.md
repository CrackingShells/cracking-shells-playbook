# VS Code Copilot trials

Host: VS Code Copilot, observed by Eliott on 2026-11-16. The server is the same stub, which speaks 2026-07-28 only. VS Code connected and called it, so it negotiated 2026-07-28. The open issue about VS Code not negotiating that version no longer applies.

## Trial 1: `result`, scale 1 (diagnostics panel)
- **Render:** the page renders 16 of 16 nodes, receiving 43,715 bytes.
- **Timing:** `t_result − t_server` is 73,679 ms. First paint and complete paint are both 24 ms after the data arrived. The 73.7 s before the result reached the page is not yet explained: it may be the host waiting for the chat turn, a trust prompt, or the page being off-screen. To be checked with Eliott.
- **`hostCapabilities`:**
  - `openLinks` and `downloadFile`
  - **`serverTools {listChanged}`** and **`serverResources {listChanged}`**
  - `logging`
  - **`sandbox.csp.connectDomains ["http://127.0.0.1:60608"]`** (VS Code adopted the domain the page declared), and `sandbox.permissions.clipboardWrite`
  - `updateModelContext {image, audio, resource, resourceLink, structuredContent}`
  - **There is no `message`.**
- **`hostContext`:** theme (dark), styles (mapped to VS Code theme variables), `displayMode` inline, **`availableDisplayModes ["inline"]` only**, `containerDimensions` 902×945, locale `en-us`, platform desktop, and `toolCall {toolCallId, toolName: "rdm-roadmap_view"}`.
- **Protocol:** the Apps protocol was `2026-01-26` both ways. `ui/message` was skipped, because the host does not advertise it. `update-model-context` returned ok. There were no tool errors.

## What this means
VS Code advertises every channel that Claude Code Desktop lacks: tool calls from the page, resource reads and the localhost stream. It does not offer fullscreen or `ui/message`.

## Trial 2: every channel (2026-11-16, Eliott, one session, in order)
| Mode | Payload | Data reached page | First paint | Complete |
|:--|:--|:--|:--|:--|
| `result` ×1 | 43,715 B, 16 nodes | 24 ms | 41 ms | 41 ms |
| `chunks` ×100 | 4,391,237 B, 1,600 nodes (65 resources/read) | 39 ms | 64 ms | 597 ms |
| `stream` ×100 (SSE from `rdm-mcp` on 127.0.0.1) | 4,391,237 B, 1,600 nodes | 28 ms | 70 ms | 157 ms |
| `meta` ×1 (`_meta`) | 43,715 B, 16 nodes from `_meta` | 23 ms | 39 ms | 40 ms |

- All times are measured from the server's stamp to the page, on one sample each.
- No tool errors. The full view fell back to the inline outline (`display mode: outline`), because VS Code offers no fullscreen.
- The 73.7 s of Trial 1 did not recur: every result reached the page within 18 to 24 ms.
- In VS Code all three channels work, and painting starts before the data is complete: 1,600 nodes paint first at 64 to 70 ms.
