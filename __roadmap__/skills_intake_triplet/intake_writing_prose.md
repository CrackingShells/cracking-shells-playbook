# Intake writing-prose

**Goal**: Land the audited-clean `writing-prose` skill on its task branch with full release wiring.
**Pre-conditions**:
- [ ] Pristine source present at `/Users/hacker/Documents/src/CrackingShells/cracking-shells-playbook/skills/writing-prose/` (primary checkout, untracked)
- [ ] Worktree on branch `task/intake-writing-prose` branched from `main`
**Success Gates**:
- ⬜ `skills/writing-prose/` tracked on the branch, byte-identical to the pristine source minus `.DS_Store` and the Step-2 release quartet (`diff -r --exclude=.DS_Store --exclude=package.json --exclude=.releaserc.js --exclude=CHANGELOG.md`) — coordinator adjudication: the original exact-diff gate structurally conflicted with Step 2's deliverables landing in the same directory; import fidelity is additionally pinned by Step 1's pre-quartet consistency check `[run]`
- ⬜ `node -e "require('./skills/writing-prose/.releaserc.js')"` exits 0 and config names `writing-prose` `[run]`
- ⬜ `uv run tools/package_skill.py skills/writing-prose dist/` exits 0 `[run]`
- ⬜ `python3 -c "import json;d=json.load(open('skills/writing-prose/package.json'));assert d['name']=='writing-prose' and d['version']=='1.0.0'"` passes `[run]`
**References**: [R01 Approved plan](/Users/hacker/.claude/plans/the-folder-contains-two-rustling-turing.md) — audit verdict (STANDALONE) and release-quartet spec

## Step 1: Import pristine skill directory
**Goal**: Bring the generalized skill content into version control unchanged.
**Implementation Logic**:
Copy the pristine directory from the primary checkout into the worktree at `skills/writing-prose/`, excluding `.DS_Store` (e.g. `rsync -a --exclude='.DS_Store'`). No content edits in this step — the audit found the skill standalone, so the import commit is a faithful record of the transferred artifact.
**Deliverables**: `skills/writing-prose/SKILL.md` (frontmatter `name: writing-prose`); `skills/writing-prose/references/{register.md,reviews.md,config-and-harvest.md}`; `skills/writing-prose/scripts/` tree (`INSTALL.md`, `pyproject.toml`, `uv.lock`, `.gitignore`, `writing_prose/{__init__.py,__main__.py,cli.py}`)
**Consistency Checks**: `diff -r --exclude=.DS_Store /Users/hacker/Documents/src/CrackingShells/cracking-shells-playbook/skills/writing-prose <worktree>/skills/writing-prose` (expected: PASS)
**Commit**: `feat(writing-prose): import generalized prose-review skill`

## Step 2: Scaffold release quartet
**Goal**: Wire the skill into the multi-semantic-release train (precedent: writing-history commit `70d34b7`).
**Implementation Logic**:
Add the three release files so the npm workspace glob picks the skill up and CI can tag `writing-prose@1.0.0`: minimal `package.json`, one-liner `.releaserc.js` delegating to the shared factory, seed `CHANGELOG.md` (single line `# Changelog`; CI rewrites it on release).
**Deliverables**: `skills/writing-prose/package.json` (`{"name":"writing-prose","version":"1.0.0"}`); `skills/writing-prose/.releaserc.js` (`module.exports = require('../../release-config')('writing-prose');`); `skills/writing-prose/CHANGELOG.md` (`# Changelog`)
**Consistency Checks**: `node -e "const c=require('./skills/writing-prose/.releaserc.js');process.exit(c.plugins.some(p=>Array.isArray(p)&&JSON.stringify(p).includes('writing-prose.skill'))?0:1)"` (expected: PASS)
**Commit**: `feat(writing-prose): scaffold skill package and release quartet`
