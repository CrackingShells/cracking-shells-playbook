# Graph view

**Goal**: A Svelte 5 page, built into one HTML file with every script and style inlined, draws the fixture as a depth-ordered graph inside an MCP Apps host. It opens a full view and one leaf's content, and records its own display time and the host's data through the app-only tools.
**Pre-conditions**:
- [ ] `../workspace_skeleton.md` is done
- [ ] The fixture shape and tool names in `stub_server.md` Step 1 and Step 2 are read: they are the contract
**Success Gates**:
- ⬜ `npm --prefix crates/rdm-mcp/view ci && npm --prefix crates/rdm-mcp/view run build` produces `crates/rdm-mcp/view/dist/index.html` as one self-contained file, and the size is printed
- ⬜ `npm --prefix crates/rdm-mcp/view run check:offline` passes. It is a script that fails if the built file has any `<script src>`, `<link href>`, `import(`, or `http://`/`https://` string other than inside comments the builder keeps (each allowed exception is listed and justified)
- ⬜ Loaded through `view/dev/mock-host.html` in a browser, the page draws every node of `fixtures/roadmap.json`, opens a leaf's markdown through a mocked `leaf_get`, requests full view, and posts one `probe_record`. A screenshot is in the leaf's report
**References**: [R01 §Graph view](../../../__design__/rdm/architecture.md) — waves as columns, colour by status, one inlined HTML file; [M0 README Gotchas](../README.md) — Apps protocol version, MIME type, pins

## Step 1: Scaffold the page and its bridge
**Goal**: A Svelte 5 and Vite project builds a single inlined HTML file that completes the MCP Apps handshake.
**Implementation Logic**:
1. The project is `crates/rdm-mcp/view/`, with exact pins: `svelte@5`, `vite`, `@sveltejs/vite-plugin-svelte`, `vite-plugin-singlefile@2.3.4` and `@modelcontextprotocol/ext-apps` at its current `latest` dist-tag. Every version is exact, with no caret, and `package-lock.json` is committed.
2. Use the `App` class of ext-apps for the bridge. If the built page grows past 250 KB because of it, report that rather than writing your own bridge.
3. On mount, the page:
   - connects through `ui/initialize`, announcing the Apps `protocolVersion` the library uses (expected `2026-01-26`)
   - keeps the host's `hostCapabilities`, `hostContext` and protocol version
   - waits for `ui/notifications/tool-result` to receive the `roadmap_view` structured content
4. The `vite.config.ts` uses `viteSingleFile()` and `build.assetsInlineLimit` at its maximum.
5. `node_modules/` stays untracked.
**Deliverables**: `crates/rdm-mcp/view/package.json`: scripts `build` and `check:offline`; `crates/rdm-mcp/view/package-lock.json`; `crates/rdm-mcp/view/vite.config.ts`; `crates/rdm-mcp/view/src/main.ts`; `crates/rdm-mcp/view/src/bridge.ts`: functions `connect` and `callAppTool`; `crates/rdm-mcp/view/scripts/check-offline.mjs`
**Consistency Checks**: `npm --prefix crates/rdm-mcp/view ci && npm --prefix crates/rdm-mcp/view run build` (expected: PASS)
**Commit**: `feat(rdm-mcp): scaffold the svelte graph view and its apps bridge`

## Step 2: Draw the graph, the full view and a leaf
**Goal**: The page shows the roadmap and lets the user open one leaf and the full view.
**Implementation Logic**:
1. **Graph:** columns by `depth`. Directories are group headers, and each leaf is a card coloured by status, using the five legacy classDef colours as constants.
2. **Leaf content:** clicking a leaf calls `leaf_get` through `callAppTool` and shows the markdown as preformatted text in a side pane. Markdown rendering is not needed.
3. **Full view:**
   - A "Full view" button calls `ui/request-display-mode` with `fullscreen` when `availableDisplayModes` includes it.
   - Otherwise it switches the page to an inline outline of the whole campaign, nested by path.
   - Record which of the two ran in `display_mode`.
4. **Probe:**
   - On first paint, the page takes `t_view_ms = Date.now()` in the first `requestAnimationFrame` after the graph mounts.
   - It calls `probe_record` once, with `run_kind` read from a `?run=cold|warm` query flag or else `unknown`, `t_server_ms` from the structured content, both Apps protocol versions, `hostCapabilities`, `hostContext` and `display_mode`.
   - It also calls `probe_record` again when the full view is opened, so that event is recorded.
5. **Theme:** follow `hostContext.theme` (light or dark) and nothing else.
**Deliverables**: `crates/rdm-mcp/view/src/App.svelte`; `crates/rdm-mcp/view/src/Graph.svelte`; `crates/rdm-mcp/view/src/LeafPane.svelte`; `crates/rdm-mcp/view/src/Outline.svelte`; `crates/rdm-mcp/view/src/probe.ts`: function `recordFirstPaint`
**Consistency Checks**: `npm --prefix crates/rdm-mcp/view run build` (expected: PASS)
**Commit**: `feat(rdm-mcp): draw the roadmap graph, full view and leaf pane`

## Step 3: Build, check offline and commit the page
**Goal**: The built page is committed and proven self-contained, and a mock host shows it working without a real host.
**Implementation Logic**:
1. `dev/mock-host.html` is a plain HTML file, not built and not shipped. It iframes `../dist/index.html`, answers `ui/initialize` with a fixed `hostContext`, sends the fixture as a tool result, and answers `tools/call` for `leaf_get` and `probe_record` from `../../fixtures/`. It logs every message.
2. Open it through a local static server, for example `python3 -m http.server` in `crates/rdm-mcp`. Then check the third Success Gate in a browser and save a screenshot path in your report.
3. Commit `dist/index.html`, because the server embeds it and crates.io consumers have no Node.
4. Delete `node_modules/` when done.
**Deliverables**: `crates/rdm-mcp/view/dist/index.html`; `crates/rdm-mcp/view/dev/mock-host.html`
**Consistency Checks**: `npm --prefix crates/rdm-mcp/view run check:offline` (expected: PASS)
**Commit**: `build(rdm-mcp): commit the built graph view`
