# Skills Intake Triplet — Cross-Skill Verification — Findings (v0)

Date: 2026-07-21

---
type: findings
topic: skills_intake_triplet
date: 2026-07-21
version: v0
prior-version: none
key-metric: checks passed: 15 / 15 (prior: N/A, delta: N/A)
decision-required: none
---

## Headline Result

metric: cross-skill verification battery pass rate
value: 15 / 15
unit: checks (5 checks × 3 skills)
prior: N/A
direction: new

All five deterministic checks in the verification battery pass for all three newly-intaked skills (`writing-prose`, `writing-design-canons`, `importing-design-figures`). The set is release-ready as a group; no remediation required.

## Results Tables

### Check matrix (skill × check)

| Skill | (a) origin-token sweep | (b) `package_skill.py` | (c) `.releaserc.js` require | (d) npm workspace census | (e) relative-link sweep |
|---|---|---|---|---|---|
| writing-prose | PASS | PASS | PASS | PASS | PASS |
| writing-design-canons | PASS | PASS | PASS | PASS | PASS |
| importing-design-figures | PASS | PASS | PASS | PASS | PASS |

### Exact commands and outputs

**(a) Origin-token sweep** — one combined invocation covers all three skill directories:

```
COLGREP_BYPASS=1 grep -rniE '\b(sift|glean|riken|ECL|VN1)\b' skills/writing-prose skills/writing-design-canons skills/importing-design-figures
```
Output: *(empty)* — exit code 1 (grep's "no lines matched"). **PASS** for all three.

**(b) Packaging** — run once per skill:

```
uv run tools/package_skill.py skills/writing-prose dist/               → exit 0, dist/writing-prose.skill
uv run tools/package_skill.py skills/writing-design-canons dist/       → exit 0, dist/writing-design-canons.skill
uv run tools/package_skill.py skills/importing-design-figures dist/    → exit 0, dist/importing-design-figures.skill
```
Each run logged `✅ Skill is valid!` from the bundled `quick_validate.py` pass before packaging. **PASS** for all three.

**(c) Release config** — run once per skill:

```
node -e "require('./skills/<skill>/.releaserc.js')"
```

| Skill | `.releaserc.js` contents | Resolved `github` asset path |
|---|---|---|
| writing-prose | `module.exports = require('../../release-config')('writing-prose');` | `../../dist/writing-prose.skill` |
| writing-design-canons | `module.exports = require('../../release-config')('writing-design-canons');` | `../../dist/writing-design-canons.skill` |
| importing-design-figures | `module.exports = require('../../release-config')('importing-design-figures');` | `../../dist/importing-design-figures.skill` |

All three `require()` calls exit 0. Each config threads its own skill name into `release-config.js`'s factory (`skills/<skill>/.releaserc.js` → shared `release-config(skillName)` in repo root), which parameterizes both the `@semantic-release/exec` packaging command and the `@semantic-release/github` asset `path`/`label` — confirmed no cross-skill leakage (each skill's asset path points to its own `.skill` file, not another skill's). **PASS** for all three.

**(d) npm workspace census** — `npm query .workspace` alone returned a **stale** result in this worktree: it reflects the *installed* `node_modules` symlink state in the main repo checkout (last populated 2026-04-13, before `writing-history` or any of the three intake skills existed), listing only `managing-roadmaps`, `writing-release`, `writing-reports`. This is an environment artifact of a `node_modules` install lagging behind `package.json`'s `workspaces` glob, not a defect in the skills themselves. Per the leaf spec's sanctioned alternative ("or `node -e` over workspaces"), the census was re-run by expanding the declared glob directly against the filesystem:

```js
const pkg = require('./package.json');           // workspaces: ["skills/*"]
fs.readdirSync('skills', { withFileTypes: true }) // enumerate skills/*
  .filter(d => d.isDirectory())
  .forEach(d => /* verify skills/<name>/package.json exists */);
```

Result — all 7 workspace members present, each with a valid `package.json`:

| Directory | package.json | Category |
|---|---|---|
| importing-design-figures | OK (`importing-design-figures@1.0.0`) | new (this intake) |
| managing-roadmaps | OK (`managing-roadmaps@1.0.0`) | existing |
| writing-design-canons | OK (`writing-design-canons@1.0.0`) | new (this intake) |
| writing-history | OK (`writing-history@1.0.0`) | existing |
| writing-prose | OK (`writing-prose@1.0.0`) | new (this intake) |
| writing-release | OK (`writing-release@1.0.0`) | existing |
| writing-reports | OK (`writing-reports@1.0.0`) | existing |

All three new skills sit alongside all four existing skills (`managing-roadmaps`, `writing-history`, `writing-release`, `writing-reports`). **PASS** for all three, via the sanctioned alternative method.

**(e) Relative-markdown-link resolution sweep** — every `.md` file under each skill directory was scanned for `[text](target)` links; non-HTTP, non-anchor, non-mailto targets were resolved relative to the containing file and checked for existence.

| Skill | `.md` files scanned | Relative links found | Unresolved |
|---|---|---|---|
| writing-prose | SKILL.md, references/register.md, references/reviews.md, references/config-and-harvest.md | 7 | 0 |
| writing-design-canons | SKILL.md, references/{identity,token-vocabulary,actionable-guidelines,consumption-guide,bertin-primer,chapter-splitting}.md | 13 | 0 |
| importing-design-figures | SKILL.md, references/{pipeline,config-and-harvest}.md, scripts/SETUP.md | 4 | 0 |

**PASS** for all three — every relative link resolves to an existing file.

## Observations

| Signal | Baseline / Expected | Observed [source] | Interpretation |
|---|---|---|---|
| Origin-token leakage (sift/glean/riken/ECL/VN1) | 0 matches across all 3 skill trees | 0 matches [grep, this report §(a)] | No provenance tokens from the source project leaked into the intake. |
| `.skill` packaging | Each skill packages independently, exit 0 | 3/3 exit 0, each producing a distinct `dist/<skill>.skill` [package_skill.py runs] | Skills are independently packageable; no shared-state coupling in the packager. |
| Release-config asset wiring | Each skill's GitHub release asset path points at its own `.skill` | 3/3 confirmed distinct, correctly-named paths [`node -e` require + assets inspection] | The shared `release-config.js` factory correctly parameterizes per-skill without copy-paste drift. |
| Workspace registration | `package.json` `workspaces: ["skills/*"]` glob picks up all 7 skills | Filesystem-level glob expansion confirms 7/7 [node -e over workspaces] | The *declared* workspace configuration is correct. The *installed* `node_modules` state in the main repo checkout is stale and under-reports — a housekeeping item for whoever next runs `npm install` at the repo root, not a blocker for this intake. |
| Cross-reference integrity | All relative `.md` links resolve | 24/24 links resolved across the 3 skills [node link-sweep script] | Reference structure is internally consistent; no dangling links introduced during intake. |

## Contradictions & Surprises

- `npm query .workspace` did not report the expected census on the first attempt — not because the workspace declaration is wrong, but because the main repo's `node_modules` (shared via the worktree's normal Node resolution) predates `writing-history` and all three intake skills. Confirmed via the sanctioned `node -e` alternative that the underlying declaration is correct; flagging here since it's a real observation about repo housekeeping, not a re-statement of a table row.

## Steering Questions

- [next run] Should `npm install` be re-run at the repo root (outside this worktree, on `main`) so `npm query .workspace` / `node_modules` symlinks reflect the current skill set for future dry-run/census checks? This intake's own checks did not depend on it, but future release tooling invocations that shell out through `node_modules/.bin` might.
- [later] Consider adding the workspace-census sweep (filesystem glob expansion, not `npm query`) as a repeatable script under `tools/`, since `npm query .workspace` is install-state-dependent and gave a false-negative-shaped result here.

## Pointers

- Leaf spec: `__roadmap__/skills_intake_triplet/finalize/verify_and_housekeep.md`
- `.gitignore` change (Step 1, same leaf): commit `chore(gitignore): ignore macOS Finder metadata`
- Skills verified: `skills/writing-prose/`, `skills/writing-design-canons/`, `skills/importing-design-figures/`
- Shared release config: `release-config.js` (repo root)
- Packager: `tools/package_skill.py`, `tools/quick_validate.py`
