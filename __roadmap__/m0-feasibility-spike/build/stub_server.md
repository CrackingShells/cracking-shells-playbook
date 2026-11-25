# Stub server

**Goal**: A throwaway `rdm-mcp` stub on `rmcp =3.10.0` speaks MCP 2026-07-28 over stdio. It serves one model-visible tool with structured output, one `ui://` page and two app-only tools, over a fixture cut from a frozen roadmap.
**Pre-conditions**:
- [ ] `../workspace_skeleton.md` is done (root `Cargo.toml` present on the branch)
**Success Gates**:
- ⬜ `cargo test -p rdm-mcp` passes. Its stdio integration test lists the three tools, finds `_meta.ui.resourceUri` on `roadmap_view` and `_meta.ui.visibility == ["app"]` on the two app-only tools, calls `roadmap_view` and gets `structuredContent` that validates against its `outputSchema`, and reads `ui://rdm/graph` with MIME type `text/html;profile=mcp-app`
- ⬜ The server's capabilities carry `extensions["io.modelcontextprotocol/ui"]`, and the server refuses any protocol version other than 2026-07-28 (asserted by the test)
- ⬜ `cargo build -p rdm-mcp` passes whether or not `view/dist/index.html` exists
**References**: [R01 §Proposed State, §Graph view](../../../__design__/rdm/architecture.md) — the pinned stack and how the page is embedded; [M0 README Gotchas](../README.md) — the rmcp calls that set `_meta`, the extension and the MIME type

## Step 1: Freeze the fixture
**Goal**: A frozen copy of the meta-roadmap exists as JSON in the shape both the server and the view use.
**Implementation Logic**:
The spike needs a real, large, deep roadmap that does not move while it runs. The meta-roadmap `__roadmap__/roadmap-mcp-rework/` has 10 leaves and a depth of 6, so it is frozen from the base commit `af8ad04`.

A stdlib-only script, run as `uv run --no-project crates/rdm-mcp/fixtures/freeze.py <commit>`, reads the tree with `git show`/`git ls-tree` at that commit. It takes each node's status from its parent README's Nodes table, and writes two outputs.

`crates/rdm-mcp/fixtures/roadmap.json` is the fixed contract with `graph_view`; do not change it without the coordinator:
```
{ "campaign": "roadmap-mcp-rework", "frozen_at": "<sha>",
  "nodes": [ { "path": "core/m1_core_foundation.md", "parent": "core", "kind": "leaf" | "dir",
               "title": "M1 Core foundation", "status": "planned" | "inprogress" | "done" | "amendment" | "blocked",
               "depth": 1 } ] }
```
- `depth` is the number of directory segments above the node, so a root leaf has depth 0.
- `parent` is `null` at the root.
- Nodes are listed in breadth-first order.

`crates/rdm-mcp/fixtures/leaves/<path>.md` holds each leaf's raw markdown.

The fixture uses the legacy vocabulary on purpose. It does not depend on M0b.
**Deliverables**: `crates/rdm-mcp/fixtures/freeze.py`: functions `read_tree`, `parse_nodes_table`, `main`; `crates/rdm-mcp/fixtures/roadmap.json`; `crates/rdm-mcp/fixtures/leaves/**.md`
**Consistency Checks**: `uv run --no-project crates/rdm-mcp/fixtures/freeze.py af8ad04 && git diff --exit-code crates/rdm-mcp/fixtures/` (expected: PASS)
**Commit**: `test(rdm-mcp): freeze the meta-roadmap as the spike fixture`

## Step 2: Serve the tools and the page
**Goal**: The stub server runs over stdio with its tools and its resource.
**Implementation Logic**:
1. Crate and dependencies:
   - The crate is `crates/rdm-mcp`, binary `rdm-mcp`, with `publish = false` while it is a spike.
   - Dependencies: `rmcp = { version = "=3.10.0", features = ["server", "macros", "transport-io"] }`, `tokio`, `serde`, `serde_json` and `schemars = "1"`.
2. `build.rs` embeds `view/dist/index.html` when it exists. Otherwise it embeds a placeholder page that says the view is not built. `cargo:rerun-if-changed` is set on the view file.
3. Tools:
   - **`roadmap_view`** (model-visible):
     - Its `outputSchema` is generated from a `RoadmapView` struct.
     - Its `structuredContent` is the fixture plus `t_server_ms`, which is `SystemTime` milliseconds taken just before the result is returned, and `payload_bytes`, the byte length of the serialised fixture.
     - It also returns a one-line text summary.
     - Its `_meta` is `{"ui": {"resourceUri": "ui://rdm/graph"}}`.
   - **`leaf_get(path)`** (app-only, `_meta.ui.visibility = ["app"]`): returns `{path, markdown}` from `fixtures/leaves/`, embedded at build time.
   - **`probe_record(record)`** (app-only):
     - It takes `{run_kind: "cold" | "warm", t_server_ms, t_view_ms, apps_protocol_announced, apps_protocol_host, host_capabilities, host_context, display_mode}`.
     - It adds `mcp_protocol` (the version negotiated with this host) and `t_recorded_ms`.
     - It appends one JSON line to `$RDM_PROBE_DIR/probe.jsonl`. If the variable is unset, it logs a warning to stderr and records nothing.
4. Resource `ui://rdm/graph`: MIME type `text/html;profile=mcp-app`, and the contents are the embedded HTML.
5. Capabilities: insert `extensions["io.modelcontextprotocol/ui"] = {"mimeTypes": ["text/html;profile=mcp-app"]}` by hand. rmcp has no helper for it.
6. Accept only protocol 2026-07-28. Find how rmcp 3.10.0 exposes version negotiation (`ProtocolVersion`, `NO_INITIALIZE`). If refusing older versions is impossible without patching rmcp, stop and report the defect; do not work around it.
7. The server process keeps no state between calls.
**Deliverables**: `crates/rdm-mcp/Cargo.toml`; `crates/rdm-mcp/build.rs`; `crates/rdm-mcp/src/main.rs`: `struct RdmSpike` with tool handlers `roadmap_view`, `leaf_get` and `probe_record`; `struct RoadmapView`; `struct ProbeRecord`; const `UI_URI`; const `APPS_MIME`; `Cargo.lock`
**Consistency Checks**: `cargo build -p rdm-mcp` (expected: PASS)
**Commit**: `feat(rdm-mcp): serve the spike tools and the ui page over stdio`

## Step 3: Test it over stdio
**Goal**: An integration test proves the server's contract without a host.
**Implementation Logic**:
1. `tests/stdio.rs` spawns the built binary as a child process. It uses an rmcp client, through the `client` and `transport-child-process` features as dev-dependencies.
2. It asserts each item of the first two Success Gates.
3. It asserts that `probe_record` with `RDM_PROBE_DIR` set to a temporary directory writes exactly one valid JSON line holding `mcp_protocol`.
4. For the version refusal, it attempts a connection or request at 2025-11-25 and expects a protocol error.
**Deliverables**: `crates/rdm-mcp/tests/stdio.rs`: tests `lists_three_tools_with_ui_meta`, `roadmap_view_structured_content_matches_schema`, `reads_ui_resource_with_apps_mime`, `declares_apps_extension`, `refuses_older_protocol` and `probe_record_appends_one_line`
**Consistency Checks**: `cargo test -p rdm-mcp` (expected: PASS)
**Commit**: `test(rdm-mcp): check the spike server contract over stdio`
