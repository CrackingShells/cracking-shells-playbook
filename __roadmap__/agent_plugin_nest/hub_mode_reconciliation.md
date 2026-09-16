# Hub Mode Reconciliation

**Goal**: Make hub mode coherent beyond the two marketplace files — correct the documentation that breaks the generated dev README, teach the install snippet where the hub lives, protect the hand-maintained dev README from `--force`, and move the example spec to hub mode so the regeneration guard survives colgrep-mcp relinquishing its marketplace.
**Pre-conditions**:
- [x] `generator_reshape` and `reference_docs` are done — this amends what they shipped
- [ ] Amendment A1 approved by the maintainer
**Success Gates**:
- ⬜ The example spec declares hub mode and keeps both marketplace-name fields [static]
- ⬜ `spawn --dry-run` against colgrep-mcp writes and merges nothing, with `dev/README.md` the only divergence [run]
- ⬜ A hub-mode spec's `install_snippet` names the hub's slug in the `marketplace add` line, not the product repo's [run]
- ⬜ `spawn --force` leaves an existing `dev/README.md` untouched, or says plainly that it will not [run]
- ⬜ `references/manifests.md` no longer tells a hub-bound spec to drop the marketplace-name fields [run]
- ⬜ `traps.md` carries a trap for a generator fix that cannot reach an already-generated hub-mode repo [run]
**References**: [R05 Gap Analysis](../../__reports__/agent_plugin_nest/00-gap_analysis_hub_mode_v0.md) — every claim with the file and line that verifies it

## Step 1: Move the example spec to hub mode

**Goal**: Stop the regeneration guard from depending on colgrep-mcp keeping a marketplace it is about to delete.

**Implementation Logic**:
Add `"marketplace": "hub"` to `assets/examples/colgrep-mcp.spec.json` and **keep** `claude_marketplace.name` and `codex.marketplace_name` at `cracking-shells` — they stop producing marketplace files and become documentation inputs only, which is exactly what step 3 corrects the docs to say. Do not remove them.

Without this the guard goes red the moment colgrep-mcp's `relinquish_marketplace` leaf deletes its two marketplace files: the non-hub spec would try to create them, `w.written` becomes non-empty, and the guard fails for a reason unrelated to any drift it exists to catch. Note the manifest reshape alone does not break it — colgrep-mcp's `plugin.json` now carries `extensions`, so that divergence has already stopped appearing, which also makes `ALLOWED_DIVERGENCE["plugin.json"]` inert. Drop that entry while re-baselining, leaving only the `dev/README.md` exemption.

Re-baseline deliberately and show the before and after in the commit body.
**Deliverables**: `skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json` — top-level `marketplace` key added, name fields retained; `skills/spawning-agent-plugins/evals/test_regeneration.py` — `ALLOWED_DIVERGENCE` reduced to `{"dev/README.md": None}`
**Consistency Checks**: `uv run skills/spawning-agent-plugins/scripts/spawn_plugin.py --root /Users/hacker/Documents/src/CrackingShells/colgrep-mcp spawn --spec skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json --dry-run 2>&1 | grep -c "^kept" | grep -qx 1` (expected: PASS)
**Commit**: `fix(spawning-agent-plugins): move the example spec to hub mode`

## Step 2: Teach the generator where the hub is, and what not to clobber

**Goal**: Two independent corrections in the same file, both of which currently fail silently.

**Implementation Logic**:
First, make `install_snippet` hub-aware. It derives its slug from `spec["repository"]`, so in hub mode it emits `marketplace add <product-repo>` — backwards, because the marketplace lives in the hub. The `marketplace` key already accepts any truthy value, so a spec can carry the hub's slug in it directly: `"marketplace": "CrackingShells/Nest"`. When the value looks like an `owner/repo` slug, use it for the `marketplace add` lines; when it is a bare sentinel like `"hub"`, the snippet cannot know the hub and must say so rather than printing a confidently wrong command. The `plugin install <name>@<market>` lines already use the marketplace *name* and stay correct either way.

Second, stop `--force` destroying `dev/README.md`. The guard exempts that file as hand-maintained prose, but `put()` under `force=True` overwrites it anyway — the exemption records the divergence without protecting it. It cost real content in colgrep-mcp: the dev plugin's eval-case location, the skill-creator attribution, and the CONTRIBUTING framing, recovered only because a reviewer diffed. Write it when absent, leave it alone when present even under `--force`, and report it as kept so the operator sees why.

Both changes are in `spawn_plugin.py`, so they are one step rather than two parallel ones.
**Deliverables**: `skills/spawning-agent-plugins/scripts/spawn_plugin.py` — `install_snippet` reading a hub slug from the `marketplace` key, and the `dev/README.md` write made create-only
**Consistency Checks**: `uv run skills/spawning-agent-plugins/scripts/spawn_plugin.py --root /Users/hacker/Documents/src/CrackingShells/colgrep-mcp install-snippet --spec skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json 2>&1 | grep -q "marketplace add" ` (expected: PASS)
**Commit**: `fix(spawning-agent-plugins): make install-snippet hub-aware and protect dev/README.md`

## Step 3: Correct the documentation the code contradicts

**Goal**: Stop the reference telling readers to do the thing that breaks their generated dev README.

**Implementation Logic**:
`references/manifests.md` says a hub-bound spec "carries no `claude_marketplace` or `codex.marketplace_name` section at all". That is wrong and the failure is silent: both fields still feed `dev_readme` and `install_snippet`, each falling back to `f"{spec['name']}-marketplace"`, so following the advice makes the generated dev README advertise a marketplace that does not exist. Rewrite the paragraph to say hub mode keys only off the top-level `marketplace` key, that the two name fields remain as documentation inputs, and that a hub slug can be carried in the key's value.

Then add a trap to `traps.md`: a generator fix cannot reach an already-generated hub-mode repository, because its marketplace files are never regenerated once the key is set. The live instance is the `authentication: "NONE"` fix — colgrep-mcp's file had to be repaired by hand, and any repo generated before that fix carries the same latent breakage without self-healing. Symptom, cause, and the manual repair.
**Deliverables**: `skills/spawning-agent-plugins/references/manifests.md` — hub-mode paragraph corrected; `skills/spawning-agent-plugins/references/traps.md` — trap `{#hub-mode-no-retro-repair}`
**Consistency Checks**: `COLGREP_BYPASS=1 grep -q "hub-mode-no-retro-repair" skills/spawning-agent-plugins/references/traps.md` (expected: PASS)
**Commit**: `docs(spawning-agent-plugins): correct the hub-mode advice and add the retro-repair trap`
