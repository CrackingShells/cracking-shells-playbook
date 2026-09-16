# End To End Install

**Goal**: Prove a real user can add Nest and install all five playbook plugins, and be explicit about which half of that is exercised versus merely validated.
**Pre-conditions**:
- [ ] `playbook_plugins` and `release_wiring` are both done
- [ ] `plugins/` has been pushed, since a git source resolves against the remote, not the working tree
**Success Gates**:
- ⬜ `known_marketplaces.json` shows `cracking-shells` sourced from `CrackingShells/Nest` [run]
- ⬜ `claude plugin list` shows all five plugins with no `failed to load` line [run]
- ⬜ Each installed plugin's skills are offered in a fresh session [behavioral]
- ⬜ `claude plugin validate` passes on the Nest marketplace and each plugin root [run]
- ⬜ The report states plainly that Codex was validated statically, not exercised [static]
**References**: [R04 §hooks-manifest-duplicate](../../../../../skills/spawning-agent-plugins/references/traps.md) — why only a marketplace install catches this class of failure

## Step 1: Replace the stale marketplace registration

**Goal**: Make the test measure Nest, not whatever catalogue was registered first.

**Implementation Logic**:
`cracking-shells` is already registered on this machine from `CrackingShells/colgrep-mcp`. Adding Nest under the same name would silently replace it, and reading the plugin list afterwards would not tell you which catalogue answered. So remove the registration first with `claude plugin marketplace remove cracking-shells`, then add `CrackingShells/Nest`, then assert the `source` recorded in `known_marketplaces.json` is the Nest repo — not merely that some marketplace named `cracking-shells` exists.

Checking the plugin list without checking the source is the marketplace-level twin of running a test suite against the wrong checkout: it fails by passing.
**Deliverables**: no files — evidence captured in the step's commit body and in the campaign's report
**Consistency Checks**: `python3 -c "import json,pathlib;d=json.loads((pathlib.Path.home()/'.claude/plugins/known_marketplaces.json').read_text());assert d['cracking-shells']['source']['repo']=='CrackingShells/Nest'"` (expected: PASS)
**Commit**: `chore(verify): repoint the cracking-shells marketplace at Nest`

## Step 2: Install all five and read the load state

**Goal**: Exercise the only path that reproduces marketplace-install failures.

**Implementation Logic**:
Install each of the five plugins with `claude plugin install <name>@cracking-shells`, then read `claude plugin list` and assert no entry reports `failed to load`. This path matters because `--plugin-dir`, `plugin details` and `plugin validate` all accept trees that fail a real marketplace install — the "Duplicate hooks file detected" class of failure appears nowhere else. Then open a fresh session and confirm each plugin's skills are actually offered, since a plugin can load without its components resolving.
**Deliverables**: no files — the observed `claude plugin list` output goes in the commit body
**Consistency Checks**: `claude plugin list 2>&1 | grep -c "failed to load" | grep -qx 0` (expected: PASS)
**Commit**: `chore(verify): install all five playbook plugins from Nest`

## Step 3: Report what was proven and what was not

**Goal**: Leave an honest verification record, and open the gate for the colgrep-mcp campaign.

**Implementation Logic**:
Write a findings report under `__reports__/agent_plugin_nest/`. Record what was exercised end to end in Claude Code and what was only validated statically: no Codex CLI is available here, so every Codex claim rests on its published manifest format and the loader source, not on a live run — the same distinction `manifests.md` already draws in its verification-status table, and it must not quietly blur now that more is riding on it. State that this leaf's completion is the gate the `nest_migration` campaign in colgrep-mcp waits on, so whoever picks that up knows the precondition is genuinely met.
**Deliverables**: `__reports__/agent_plugin_nest/00-findings_install_verification_v0.md` — sections Exercised, Validated only, Open risks, Gate status
**Consistency Checks**: `test -f __reports__/agent_plugin_nest/00-findings_install_verification_v0.md` (expected: PASS)
**Commit**: `docs(reports): record the Nest install verification and its limits`
