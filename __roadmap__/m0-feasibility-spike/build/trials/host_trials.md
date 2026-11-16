# Host trials

**Goal**: Every host-facing gate of M0 is observed on Claude Code Desktop, VS Code Copilot and the Claude Code CLI. The raw host data and timings are kept, and the run count is justified.
**Pre-conditions**:
- [ ] `../stub_server.md` and `../graph_view.md` are merged into `feat/m0-feasibility-spike`, and `cargo build --release -p rdm-mcp` passes there
- [ ] Eliott is at the keyboard (cold runs need a new session each)
**Success Gates**:
- ⬜ [behavioral] In Claude Code Desktop, the tool is listed and called, the page renders the fixture graph, the full view opens, one leaf's content opens, and the page makes no network request: there is no CSP violation in the host log, and the `check:offline` result is cited
- ⬜ [behavioral] From the Claude Code CLI, the three tools are listed and `roadmap_view` is called (the CLI does not render Apps; recorded as such)
- ⬜ [behavioral] In VS Code Copilot, a render is attempted and the outcome recorded, whatever it is (best effort, not a blocker)
- ⬜ [static] `__reports__/m0-feasibility-spike/trials/probe.jsonl` holds every run, with personal data removed or masked, and each host's protocol versions are noted
- ⬜ [static] The exact allow-rule string of `roadmap_view` under the `managing-roadmaps` plugin name is written down, copied from a real session
**References**: [R01 §Risks & Mitigations](../../../../__design__/rdm/architecture.md) — host protocol support and the allow-rule risk; [`../stub_server.md`](../stub_server.md) — the probe payload

## Step 1: Connect each host and read the allow-rule string
**Goal**: The stub is reachable from the three hosts, and the allow-rule string is known.
**Implementation Logic**:
1. Register the release binary as a stdio server named `rdm` in an uncommitted project `.mcp.json`, with `RDM_PROBE_DIR` set to the scratchpad. Use it in Claude Code Desktop and the CLI.
2. For VS Code, add the same server to VS Code's MCP configuration.
3. For the allow-rule string:
   - Create an uncommitted local copy of the `managing-roadmaps` plugin with an `mcpServers` entry for the stub, and load it with `claude --plugin-dir`.
   - Trigger the permission prompt and copy the rule string exactly as the session shows it, for example in the permissions list.
4. Record in one notes file per host: the host version, whether the tool is listed and called, whether the page renders, and what the full view and the leaf pane do.
5. Check every probe line for personal data before copying it: paths, user names, email addresses and machine names. Mask what is found and say so in the notes.
**Deliverables**: `__reports__/m0-feasibility-spike/trials/desktop.md`, `cli.md`, `vscode.md`, `allow-rule.md`
**Consistency Checks**: `ls __reports__/m0-feasibility-spike/trials/desktop.md __reports__/m0-feasibility-spike/trials/cli.md __reports__/m0-feasibility-spike/trials/vscode.md __reports__/m0-feasibility-spike/trials/allow-rule.md` (expected: PASS)
**Commit**: `docs(reports): record the m0 host connections and the allow-rule string`

## Step 2: Pilot the timing and fix the run count
**Goal**: The noise of time to display is known, and so is the number of runs needed to control it.
**Implementation Logic**:
1. Definitions:
   - **Time to display** is `t_view_ms - t_server_ms` from one probe line.
   - **A cold run** is the first render after the server process starts, in a new session.
   - **A warm run** is a later render in the same session.
2. In Claude Code Desktop, do 5 cold and 5 warm runs.
3. From the pilot, estimate the spread. Then choose N per kind so that the bootstrap 95% confidence interval of the median stays within ±10% of the median. The default is 20; the cap is 30 cold runs, because each costs Eliott a new session.
4. Write the pilot numbers and the chosen N, with the reason, in the notes.
**Deliverables**: `__reports__/m0-feasibility-spike/trials/timing-pilot.md`
**Consistency Checks**: `test -s __reports__/m0-feasibility-spike/trials/timing-pilot.md` (expected: PASS)
**Commit**: `docs(reports): pilot the m0 display timing and fix the run count`

## Step 3: Run the timed series
**Goal**: The cold and warm series are complete on Claude Code Desktop, and on VS Code if it renders.
**Implementation Logic**:
1. Run N cold and N warm on Claude Code Desktop, and the same series on VS Code only if it rendered in Step 1.
2. Copy the masked `probe.jsonl` into the trials directory.
**Deliverables**: `__reports__/m0-feasibility-spike/trials/probe.jsonl`
**Consistency Checks**: `uv run --no-project python -c "import json,sys; [json.loads(l) for l in open('__reports__/m0-feasibility-spike/trials/probe.jsonl')]"` (expected: PASS)
**Commit**: `docs(reports): record the m0 timed display runs`
