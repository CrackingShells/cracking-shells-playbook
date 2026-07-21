# Intake importing-design-figures

**Goal**: Land `importing-design-figures` on its task branch fully standalone — origin constants parameterized out of the render harness — with full release wiring.
**Pre-conditions**:
- [ ] Pristine source present at `/Users/hacker/Documents/src/CrackingShells/cracking-shells-playbook/skills/importing-design-figures/` (primary checkout, untracked)
- [ ] Worktree on branch `task/intake-importing-design-figures` branched from `main`
- [ ] `node` available in the worktree (syntax checks on `.mjs`)
**Success Gates**:
- ⬜ `grep -rniE '\b(ECL|VN1|sift|glean|riken)\b' skills/importing-design-figures/` returns nothing — coordinator adjudication: word-boundary form, since the bare pattern substring-matches ordinary English ("declining", "declaring") and can never pass on normal prose `[run]`
- ⬜ `shoot.mjs` computes true-mm height from a `--target-width-mm` parameter — no literal `168` remains in its computation, and no hardcoded `Palatino`/`Helvetica Neue`/`rgb(252, 251, 247)`/`rgb(232, 228, 218)` literals remain `[static]`
- ⬜ `node --check skills/importing-design-figures/scripts/shoot.mjs && node --check skills/importing-design-figures/scripts/decode-dc.mjs && bash -n skills/importing-design-figures/scripts/render.sh` exit 0 `[run]`
- ⬜ `references/config-and-harvest.md` names a concrete author root (`~/.config/importing-design-figures/`); SKILL.md or SETUP.md documents the `DesignSync` prerequisite and a fallback when it is absent `[static]`
- ⬜ `uv run tools/package_skill.py skills/importing-design-figures dist/` exits 0 `[run]`
- ⬜ `python3 -c "import json;d=json.load(open('skills/importing-design-figures/package.json'));assert d['name']=='importing-design-figures' and d['version']=='1.0.0'"` passes `[run]`
**References**: [R01 Approved plan](/Users/hacker/.claude/plans/the-folder-contains-two-rustling-turing.md) — audit findings table (8 findings) and release-quartet spec

## Step 1: Import pristine skill directory
**Goal**: Bring the transferred skill content into version control unchanged, so the generalization work is reviewable as diffs against the import.
**Implementation Logic**:
Copy the pristine directory from the primary checkout into the worktree at `skills/importing-design-figures/`, excluding `.DS_Store` (e.g. `rsync -a --exclude='.DS_Store'`). No content edits in this step.
**Deliverables**: `skills/importing-design-figures/SKILL.md` (frontmatter `name: importing-design-figures`); `skills/importing-design-figures/references/{pipeline.md,config-and-harvest.md}`; `skills/importing-design-figures/scripts/{SETUP.md,render.sh,shoot.mjs,decode-dc.mjs}`
**Consistency Checks**: `diff -r --exclude=.DS_Store /Users/hacker/Documents/src/CrackingShells/cracking-shells-playbook/skills/importing-design-figures <worktree>/skills/importing-design-figures` (expected: PASS)
**Commit**: `feat(importing-design-figures): import generalized figure-import skill`

## Step 2: Parameterize render-harness constants
**Goal**: Replace the origin design system's compiled-in constants with per-project CLI parameters so the harness is correct on any project (audit findings 2, 3, 4).
**Implementation Logic**:
In `scripts/shoot.mjs`: (a) add a `--target-width-mm <n>` argument and use it wherever the literal `168` is used to compute the printed true-mm height and DPI notes — with no built-in project default, require it or print heights only in relative terms when absent; (b) add `--fonts "<Family>[,<Family>...]"` and run the `document.fonts.check` probe against the supplied families instead of hardcoded Palatino / Helvetica Neue, skipping the probe (with a note) when not supplied; (c) drop the two hardcoded fallback field colors — rely on the page's field CSS token or an explicit `--field <css-color>` argument. Thread the same parameters through `scripts/render.sh` (pass-through flags, updated usage comment). Keep the argument parsing style already used by the scripts.
**Deliverables**: edited `skills/importing-design-figures/scripts/shoot.mjs` (`--target-width-mm`, `--fonts`, `--field` argument handling; no literal `168`, no `Palatino`/`Helvetica Neue` literals, no hardcoded `rgb(...)` fallbacks); edited `skills/importing-design-figures/scripts/render.sh` (forwarding flags + usage text)
**Consistency Checks**: `node --check skills/importing-design-figures/scripts/shoot.mjs && ! grep -nE 'rgb\(252, 251, 247\)|Helvetica Neue|168 \*' skills/importing-design-figures/scripts/shoot.mjs` (expected: PASS)
**Commit**: `feat(importing-design-figures): parameterize render harness constants`

## Step 3: Purge origin design-system references from docs and examples
**Goal**: Remove origin identifiers and align every document with the parameterized model (audit findings 1, 4, 5).
**Implementation Logic**:
Scrub the undefined "ECL" acronym from `references/pipeline.md:74` and the `shoot.mjs` comment — describe the 1429 px = 168 mm pairing, if kept at all, as a worked illustration of the width→scale arithmetic, not as an authoritative default. Update `pipeline.md:92`, `SETUP.md:37` sample output, `render.sh:11` DPI comment, and `SKILL.md:53` placement-height formula to reference the `--target-width-mm` parameter (sourced from the project's `config.md`) instead of a fixed 168 mm. Replace the "VN1 Vision Figure.dc.html" example filename in `render.sh:42` with an unambiguous generic like "Figure 1 Overview.dc.html".
**Deliverables**: edited `skills/importing-design-figures/references/pipeline.md`, `scripts/SETUP.md`, `scripts/render.sh`, `SKILL.md` (parameter-based wording, no `ECL`, no `VN1`)
**Consistency Checks**: `! grep -rniE 'ECL|VN1' skills/importing-design-figures/` (expected: PASS)
**Commit**: `fix(importing-design-figures): purge origin design-system references`

## Step 4: Document author root and DesignSync prerequisite
**Goal**: Close the two setup-context gaps that assume the origin environment (audit findings 6, 7).
**Implementation Logic**:
In `references/config-and-harvest.md` (lines 51, 66), pin "the author root" to a concrete location — `~/.config/importing-design-figures/` — mirroring the sibling `writing-prose` skill's `~/.config/writing-prose/` pattern. In `SKILL.md` (near the DesignSync workflow step) or `scripts/SETUP.md`, add a short prerequisite note: `DesignSync` is a deferred MCP tool that must be connected in the host session, and state the fallback when it is absent (manual export of the design source into the render pipeline's input format).
**Deliverables**: edited `skills/importing-design-figures/references/config-and-harvest.md` (concrete `~/.config/importing-design-figures/` author root); edited `skills/importing-design-figures/SKILL.md` and/or `scripts/SETUP.md` (DesignSync prerequisite + fallback note)
**Consistency Checks**: `grep -q 'config/importing-design-figures' skills/importing-design-figures/references/config-and-harvest.md && grep -rqi 'DesignSync' skills/importing-design-figures/scripts/SETUP.md skills/importing-design-figures/SKILL.md` (expected: PASS)
**Commit**: `docs(importing-design-figures): document author root and DesignSync prerequisite`

## Step 5: Scaffold release quartet
**Goal**: Wire the skill into the multi-semantic-release train (precedent: writing-history commit `70d34b7`).
**Implementation Logic**:
Add minimal `package.json`, one-liner `.releaserc.js` delegating to the shared factory, and seed `CHANGELOG.md` (`# Changelog`) so the npm workspace glob picks the skill up and CI tags `importing-design-figures@1.0.0`.
**Deliverables**: `skills/importing-design-figures/package.json` (`{"name":"importing-design-figures","version":"1.0.0"}`); `skills/importing-design-figures/.releaserc.js` (`module.exports = require('../../release-config')('importing-design-figures');`); `skills/importing-design-figures/CHANGELOG.md` (`# Changelog`)
**Consistency Checks**: `node -e "const c=require('./skills/importing-design-figures/.releaserc.js');process.exit(JSON.stringify(c.plugins).includes('importing-design-figures.skill')?0:1)"` (expected: PASS)
**Commit**: `feat(importing-design-figures): scaffold skill package and release quartet`
