# Desktop trials (Claude Code Desktop)

Host: Claude Code Desktop. The user agent reports `Claude/5.11492.5`, Electron 45.2.1 and Chrome 156. Server: the `rdm-mcp` stub on rmcp =3.10.0, 2026-07-28 only, registered as the project stdio server `rdm`. Observed by Eliott on 2026-11-16.

## Trial 1: render and app-only tools
- `roadmap_view` is listed and called, and the page renders "rather fast": **PASS**.
- Clicking a leaf (`leaf_get`, app-only) fails with `ProtocolError: IPC not available for MCP tool call`.
- No `probe_record` line reached the server: every call from the page to the server failed.

## Trial 2: host data (diagnostics panel)
- **`hostCapabilities`:** `openLinks`, `serverResources`, `logging`, `sandbox`, `updateModelContext {text, image}` and `message {text}`. **There is no `serverTools`.** Under the Apps spec a page may proxy tool calls only when the host advertises that, so the IPC error is the host declining by design.
- **Apps protocol:** the page announced `2026-01-26` and the host returned `2026-01-26`.
- **Display:** `displayMode` is `inline`, and `availableDisplayModes` is `["inline", "fullscreen"]`.
- **`hostContext` keys:** `toolInfo`, `theme` (dark), `styles` (host CSS variables, plus `css.fonts`, which loads fonts from `assets.claude.ai`), `displayMode`, `availableDisplayModes`, `containerDimensions` (width 731, maxHeight 5000), `locale`, `timeZone`, `userAgent`, `platform` (desktop), `deviceCapabilities` (hover, no touch) and `safeAreaInsets`. The raw dump is in [desktop-host.json](desktop-host.json), with `styles` left out.
- **`updateModelContext`:** returns ok, but nothing appears in the session transcript.
- **Timing:** the panel showed 46,970 ms, which was Eliott's copy delay. The panel displayed the stamp of the latest probe, not of first paint. Eliott saw the page paint immediately.
- **Allow-rule string (project server):** `mcp__rdm__roadmap_view`, read from the session transcript.
- **macOS install trap:** the trial binary had been overwritten in place. macOS killed it at start (exit 137), so the host reported "Couldn't reconnect rdm: Connection closed". The fix is to install onto a new inode (`rm` then `cp`).

## Trial 3: data channels (mode × scale)
| Channel | Result |
|:--|:--|
| `result`, scale 1 (43,715 B payload) | Renders. The tool result lands in the model context in full (45,034 chars in the transcript, about 11k tokens). Tool round trip 2.3 s (call 08:14:25.573, result 08:14:27.871). |
| `resources/read` from the page (`rdm://leaf/…`, `rdm://chunk/…`) | `ProtocolError: Resource unavailable: read_failed` for every URI, although `serverResources` is advertised |
| `stream` (SSE from `127.0.0.1:<port>`, declared in `_meta.ui.csp.connectDomains`) | `CSP violation: connect-src blocked http://127.0.0.1:60021/…`. The host does not honour the declared domain |
| Tool call from the page | Still `IPC not available` |
| `ui/message` probe | No `RDM_PROBE` message in the transcript |

## What this means
In this host the inline page can only show what the tool result carries, and the model's context pays for every byte of it. Leaf content, large roadmaps, streaming and live refresh cannot reach an inline page here. A standalone viewer has none of these limits: Svelte in Tauri, or in a local browser page, talking to the Rust core over IPC or RPC (Eliott's suggestion).

## Trial 4: `meta` mode (data in the tool result's `_meta`)
Run on 2026-11-16 after the fix that keeps `frozen_at` (commit `7981e5c`). The first attempt failed because the host validates `structuredContent` against the tool's `outputSchema`, and the summary lacked the required `frozen_at`.

| Scale | Model saw (transcript tool_result) | Page got | t_result − t_server | first paint | complete |
|:--|:--|:--|:--|:--|:--|
| 1 (43,715 B) | 200 chars (the summary only) | 16 of 16 nodes from `_meta` | 318 ms | 371 ms | 372 ms |
| 10 (~439 KB) | 203 chars (the summary only) | panel not captured yet | | | |

- **The finding:** Claude Code Desktop hands the result-level `_meta` to the page and does NOT put it in the model context. It does put `structuredContent` in the model context, contrary to the Apps spec's "not added to model context". So the full roadmap can reach the inline page at no token cost, if it travels in `_meta`.
- **The host validates `structuredContent` against `outputSchema`,** so every required field must be present in every mode.
- **Capabilities that changed on this run:** `ui/message` returned ok this time, but no `RDM_PROBE` text was found in the transcript. `hostCapabilities.sandbox.csp.connectDomains` now lists the declared `http://127.0.0.1:<port>`, yet Trial 3's stream was CSP-blocked. Re-test the stream before concluding.
