# Hub Mode Reconciliation

**Goal**: Make hub mode coherent beyond the two marketplace files — record the hub's identity in the spec as an answered question rather than inferring it, fail loudly when it is missing, protect the hand-maintained dev README from `--force`, and move the example spec to hub mode so the regeneration guard survives colgrep-mcp relinquishing its marketplace.
**Pre-conditions**:
- [x] `generator_reshape` and `reference_docs` are done — this amends what they shipped
- [x] Amendment A1 approved by the maintainer
**Success Gates**:
- ⬜ The example spec declares hub mode, names the hub, and keeps both marketplace-name fields [static]
- ⬜ `spawn --dry-run` against colgrep-mcp writes and merges nothing, with `dev/README.md` the only divergence [run]
- ⬜ A hub-mode spec with no recorded hub is rejected at load with a message naming what to add [run]
- ⬜ `install_snippet` and the generated dev README both name the hub's slug, never the product repo's [run]
- ⬜ `spawn --force` leaves an existing `dev/README.md` untouched and reports it as kept [run]
- ⬜ `SKILL.md` instructs the reader to ask who owns the marketplace before writing a hub-mode spec [run]
- ⬜ `traps.md` carries a trap for a generator fix that cannot reach an already-generated hub-mode repo [run]
**References**: [R05 Gap Analysis](../../__reports__/agent_plugin_nest/00-gap_analysis_hub_mode_v0.md) — every finding with the file and line that verifies it

## Step 1: Move the example spec to hub mode

**Goal**: Stop the regeneration guard depending on colgrep-mcp keeping a marketplace it is about to delete.

**Implementation Logic**:
Add hub mode to `assets/examples/colgrep-mcp.spec.json` and **keep** `claude_marketplace.name` and `codex.marketplace_name` at `cracking-shells`. Those fields stop producing marketplace files and remain what they always were: the recorded answer to "which marketplace do these plugins belong to". Do not remove them.

Without this the guard goes red the moment colgrep-mcp's `relinquish_marketplace` leaf deletes its two marketplace files — the non-hub spec would try to create them, `w.written` becomes non-empty, and the guard fails for a reason unrelated to any drift it exists to catch. The manifest reshape alone does not break it: colgrep-mcp's `plugin.json` now carries `extensions`, so that divergence has already stopped appearing, which makes `ALLOWED_DIVERGENCE["plugin.json"]` inert. Drop that entry while re-baselining, leaving only the `dev/README.md` exemption.

Re-baseline deliberately and show before and after in the commit body.
**Deliverables**: `skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json` — hub mode declared with the hub named, name fields retained; `skills/spawning-agent-plugins/evals/test_regeneration.py` — `ALLOWED_DIVERGENCE` reduced to `{"dev/README.md": None}`
**Consistency Checks**: `uv run skills/spawning-agent-plugins/scripts/spawn_plugin.py --root /Users/hacker/Documents/src/CrackingShells/colgrep-mcp spawn --spec skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json --dry-run 2>&1 | grep -c "^kept" | grep -qx 1` (expected: PASS)
**Commit**: `fix(spawning-agent-plugins): move the example spec to hub mode`

## Step 2: Record the hub, do not infer it

**Goal**: Make the hub's identity an answered question stored in the spec, and refuse to guess when it is absent.

**Implementation Logic**:
Only two things need to know which marketplace a plugin belongs to, and both are documentation outputs: `dev_readme()` and `install_snippet`. No manifest a loader reads needs it. So the generator must never derive that identity — it reads a recorded value, and the skill asks for it. This is also what keeps `spawn` reproducible: an interactively supplied name would make the regeneration guard meaningless, because the guard re-runs `spawn` and compares.

Give the `marketplace` key an explicit shape that carries the hub repository — the value already accepts anything truthy, so a structured form costs no compatibility. Then **fail at spec load** when hub mode is set and no hub repository is recorded, with a message naming the exact key to add and what it is for. Do not fall back to `spec["repository"]`, which is the product repo and is precisely the wrong answer; do not print a hedged "cannot determine the hub" line either. A spec that cannot describe its own install path is a question for a human, and the load-time error is how the human gets asked.

Then have both consumers read it: `install_snippet`'s `marketplace add` lines and `dev_readme()`'s install instructions name the hub's slug, while the `plugin install <name>@<market>` lines keep using the marketplace *name*, which was already correct.
**Deliverables**: `skills/spawning-agent-plugins/scripts/spawn_plugin.py` — `load_spec` rejecting hub mode with no recorded hub, `install_snippet` and `dev_readme` reading the hub slug
**Consistency Checks**: `uv run skills/spawning-agent-plugins/scripts/spawn_plugin.py --root /Users/hacker/Documents/src/CrackingShells/colgrep-mcp install-snippet --spec skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json 2>&1 | grep -q "CrackingShells/Nest"` (expected: PASS)
**Commit**: `fix(spawning-agent-plugins): record the hub repository instead of inferring it`

## Step 3: Stop `--force` destroying hand-maintained prose

**Goal**: Make the guard's exemption protective rather than merely descriptive.

**Implementation Logic**:
`ALLOWED_DIVERGENCE["dev/README.md"] = None` documents that this file deliberately diverges from `dev_readme()` because a maintainer edits it. But it is written through `put()`, so `--force` regenerates it anyway — the exemption records the divergence without protecting it. It cost real content in colgrep-mcp: the dev plugin's eval-case location, the skill-creator attribution for the progressive-disclosure layout, and the CONTRIBUTING framing, recovered only because a reviewer diffed before committing.

Write it when absent; leave it alone when present, even under `--force`; and report it as kept so the operator can see why it was skipped rather than wondering whether the run worked. `--force` exists to overwrite generated manifests, and this file is the one output the generator seeds once and a human then owns.
**Deliverables**: `skills/spawning-agent-plugins/scripts/spawn_plugin.py` — the `dev/README.md` write made create-only regardless of `force`
**Consistency Checks**: `uv run skills/spawning-agent-plugins/scripts/spawn_plugin.py --root /Users/hacker/Documents/src/CrackingShells/colgrep-mcp spawn --spec skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json --force --dry-run 2>&1 | grep -q "dev/README.md"` (expected: PASS)
**Commit**: `fix(spawning-agent-plugins): never regenerate an existing dev README`

## Step 4: Make the documentation ask the question

**Goal**: Put the human question in the workflow, and correct the advice that currently breaks the generated dev README.

**Implementation Logic**:
`references/manifests.md` says a hub-bound spec "carries no `claude_marketplace` or `codex.marketplace_name` section at all". That is wrong and silent: both fields still feed the two documentation outputs, each falling back to `f"{spec['name']}-marketplace"`, so following the advice advertises a marketplace that does not exist. Rewrite it to say hub mode suppresses only the marketplace *files*, that the name fields remain the recorded answer, and that the hub repository must be recorded too.

Then add the question to `SKILL.md`'s spec-writing step: before writing a hub-mode spec, ask who owns the marketplace, what it is called, and which repository holds it — and record the answers. State the principle, because it generalises past this one key: the generator reads recorded answers and never infers human-facing identity, which is why the load-time error in step 2 is a feature rather than a nuisance.

Finally add a trap to `traps.md`: once a repo declares hub mode its marketplace files are never regenerated, so a generator fix to their contents can never reach an already-generated repo. The live instance is the `authentication: "NONE"` fix — colgrep-mcp's file had to be repaired by hand, and any repo generated before that fix carries the same latent breakage without self-healing.
**Deliverables**: `skills/spawning-agent-plugins/references/manifests.md` — hub-mode paragraph corrected; `skills/spawning-agent-plugins/SKILL.md` — the hub question added to the spec-writing step; `skills/spawning-agent-plugins/references/traps.md` — trap `{#hub-mode-no-retro-repair}`
**Consistency Checks**: `COLGREP_BYPASS=1 grep -q "hub-mode-no-retro-repair" skills/spawning-agent-plugins/references/traps.md` (expected: PASS)
**Commit**: `docs(spawning-agent-plugins): ask who owns the marketplace, and add the retro-repair trap`
