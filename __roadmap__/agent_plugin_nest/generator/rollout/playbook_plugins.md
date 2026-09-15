# Playbook Plugins

**Goal**: Turn five playbook skills into five plugin roots under `plugins/`, each with its own manifests and an assembled copy of its skill.
**Pre-conditions**:
- [ ] `generator_reshape` is done and `check_plugin.py` passes on its output
- [ ] `plugin_assembly_tool` is done
- [ ] `nest_catalogue` is done, so the entries these plugins must match already exist
**Success Gates**:
- ⬜ `plugins/` holds exactly five roots: `managing-roadmaps`, `writing-history`, `writing-release`, `writing-reports`, `spawning-agent-plugins` [run]
- ⬜ Each root has `plugin.json`, `.claude-plugin/plugin.json` and `skills/<name>/SKILL.md` [run]
- ⬜ No root contains a `.codex-plugin/` directory [run]
- ⬜ `check_plugin.py` reports no problems for any root [run]
- ⬜ Each plugin's `name` matches its Nest entry name exactly [run]
**References**: [R01 §Scope](~/.claude/plans/good-news-overall-it-s-gleaming-wreath.md) — which five skills, and which are excluded and why

## Step 1: Write the multi-plugin spec

**Goal**: Describe all five plugins in one spec the generator can consume.

**Implementation Logic**:
Author `plugin.spec.json` at the repo root with a `plugins` array of five entries. Each entry sets `name` to the skill name, `dir` to `plugins/<name>`, a `description` drawn from the skill's `SKILL.md` frontmatter, and `version_from: "package.json:skills/<name>/package.json"` so each plugin tracks its own skill's release line. Set the hub marketplace mode so no marketplace file is written into this repo — Nest owns the `cracking-shells` name, and a second file claiming it would silently replace Nest's on any client that added both.

Note `spawning-agent-plugins` has no `package.json` yet; `release_wiring` creates it. Until then its `version_from` resolves to the `0.1.0` fallback. That is expected, not a defect — but if the generator errors instead of falling back, report it rather than working around it.
**Deliverables**: `plugin.spec.json` — `plugins[]` with five entries, each carrying `name`, `dir`, `description`, `version_from`, and per-ecosystem presentation blocks
**Consistency Checks**: `python3 -c "import json;d=json.load(open('plugin.spec.json'));assert len(d['plugins'])==5;assert {p['name'] for p in d['plugins']}=={'managing-roadmaps','writing-history','writing-release','writing-reports','spawning-agent-plugins'}"` (expected: PASS)
**Commit**: `feat(plugins): declare the five playbook plugins in one spec`

## Step 2: Generate the plugin roots

**Goal**: Materialise the trees Nest points at.

**Implementation Logic**:
Run the assembly tool for each of the five skills to populate `plugins/<name>/skills/<name>/`, then run `spawn_plugin.py` against the spec to write each root's `plugin.json` and `.claude-plugin/plugin.json`. Commit the whole `plugins/` tree: unlike `dist/`, it is tracked, because Nest references these paths by git source and a sparse clone can only find what the ref actually contains. Verify each root with `check_plugin.py` before committing, and confirm every generated `name` matches the corresponding Nest entry — a mismatch there is invisible until an install fails.
**Deliverables**: `plugins/managing-roadmaps/`, `plugins/writing-history/`, `plugins/writing-release/`, `plugins/writing-reports/`, `plugins/spawning-agent-plugins/` — each with `plugin.json`, `.claude-plugin/plugin.json` and `skills/<name>/`
**Consistency Checks**: `test $(ls -d plugins/*/ | wc -l) -eq 5 && for p in plugins/*/; do test -f "$p/plugin.json" && test -f "$p/.claude-plugin/plugin.json" || exit 1; done` (expected: PASS)
**Commit**: `feat(plugins): generate the five playbook plugin roots`

## Step 3: Confirm the tree is reproducible

**Goal**: Prove the committed tree is exactly what regeneration produces, so the drift gate has a true baseline.

**Implementation Logic**:
Re-run assembly and generation over the committed tree and confirm `git status --porcelain plugins/` is empty. This is the first real exercise of the drift gate `plugin_assembly_tool` installed, and it is the check that makes a tracked generated directory safe to keep in git. If it is not empty, the difference is either non-determinism in the tools (file ordering, timestamps) or a hand edit that crept in — report which, rather than committing the diff to make the check pass.
**Deliverables**: no new files — the reproducibility evidence goes in the commit body
**Consistency Checks**: `test -z "$(git status --porcelain plugins/)"` (expected: PASS)
**Commit**: `chore(plugins): confirm the assembled plugin tree regenerates identically`
