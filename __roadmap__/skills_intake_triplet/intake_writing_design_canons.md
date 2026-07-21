# Intake writing-design-canons

**Goal**: Land `writing-design-canons` on its task branch fully standalone (origin references purged) with full release wiring.
**Pre-conditions**:
- [ ] Pristine source present at `/Users/hacker/Documents/src/CrackingShells/cracking-shells-playbook/skills/writing-design-canons/` (primary checkout, untracked)
- [ ] Worktree on branch `task/intake-writing-design-canons` branched from `main`
**Success Gates**:
- ⬜ `grep -rniE 'sift|glean|riken|symlink' skills/writing-design-canons/` returns nothing `[run]`
- ⬜ `skills/writing-design-canons/README.md` does not exist (no sibling skill in this bank ships a README) `[static]`
- ⬜ Every relative link in `SKILL.md` and `references/*.md` resolves to a file on disk `[run]`
- ⬜ `uv run tools/package_skill.py skills/writing-design-canons dist/` exits 0 `[run]`
- ⬜ `python3 -c "import json;d=json.load(open('skills/writing-design-canons/package.json'));assert d['name']=='writing-design-canons' and d['version']=='1.0.0'"` passes `[run]`
**References**: [R01 Approved plan](/Users/hacker/.claude/plans/the-folder-contains-two-rustling-turing.md) — audit findings table (6 findings) and release-quartet spec

## Step 1: Import pristine skill directory
**Goal**: Bring the transferred skill content into version control unchanged, so the standalone fixes are reviewable as diffs against the import.
**Implementation Logic**:
Copy the pristine directory from the primary checkout into the worktree at `skills/writing-design-canons/`, excluding `.DS_Store` (e.g. `rsync -a --exclude='.DS_Store'`). README.md is imported as-is — its removal is Step 2, deliberately visible in history. One authorized edit (coordinator adjudication of implementer flag): the pristine SKILL.md frontmatter `description` is 1148 characters, over the 1024-character maximum enforced by `tools/quick_validate.py` via the pre-commit packaging hook, so a byte-faithful import is uncommittable; compress the `description` field to ≤1024 characters preserving every trigger phrase category and the four doc-type names, touching nothing else in the file.
**Deliverables**: `skills/writing-design-canons/SKILL.md` (frontmatter `name: writing-design-canons`, `description` ≤1024 chars); `skills/writing-design-canons/README.md` (temporarily); `skills/writing-design-canons/references/{identity.md,token-vocabulary.md,actionable-guidelines.md,bertin-primer.md,consumption-guide.md,chapter-splitting.md}`
**Consistency Checks**: `diff -r --exclude=.DS_Store --exclude=SKILL.md /Users/hacker/Documents/src/CrackingShells/cracking-shells-playbook/skills/writing-design-canons <worktree>/skills/writing-design-canons && uv run tools/package_skill.py skills/writing-design-canons dist/` (expected: PASS)
**Commit**: `feat(writing-design-canons): import generalized design-canon skill`

## Step 2: Purge origin-project references
**Goal**: Remove every dead pointer and alias to the origin authoring projects (audit findings 1, 2, 3, 5, 6).
**Implementation Logic**:
Delete `README.md` entirely — it documents a symlink-install/dev workflow from the origin single-skill repo and cites dead paths (`ro/sift-visual-identity-v0/`, `ro/glean-visual-identity-v0/`, `ro/riken-figure-styling/references/*.jsx`); SKILL.md already covers purpose/anatomy/output, and no sibling skill ships a README. In `references/token-vocabulary.md` (lines 51-52), replace the "(sift/glean style)" alias with a generic descriptor like "(narrative/poetic brand-identity projects)" and "(riken style)" with "(scientific/technical figure-styling projects)". In `references/identity.md` (line 47), rewrite the "prior canons" example phrases as clearly invented metaphors that no longer echo the origin themes verbatim.
**Deliverables**: deletion of `skills/writing-design-canons/README.md`; edited `skills/writing-design-canons/references/token-vocabulary.md` (generic register descriptors, no project aliases); edited `skills/writing-design-canons/references/identity.md` (invented example metaphors)
**Consistency Checks**: `! grep -rniE 'sift|glean|riken|symlink' skills/writing-design-canons/` (expected: PASS)
**Commit**: `fix(writing-design-canons): purge origin-project references`

## Step 3: Make chapter-splitting worked example self-contained
**Goal**: Replace the §B.6 worked example built on the origin canon's nine real `.jsx` files with a fully fictional, self-contained one (audit finding 4).
**Implementation Logic**:
Rewrite `references/chapter-splitting.md` §B.6 (plus its ToC entry near line 10 and the register-split aside near line 40) around an invented canon in the same spirit as the skill's existing fictional "Plumb-line" examples: invent a canon name and a nine-file split whose file names illustrate the same tier mapping (Atoms / system-level invariants / element-level rules / relation-level rules / cross-cutting modulation / compositional rules). Preserve the pedagogy (tier table, "every tier is present" lesson); change only the example's identity so a reader can follow it without access to any external project.
**Deliverables**: edited `skills/writing-design-canons/references/chapter-splitting.md` (§B.6 retitled, fictional canon file table, no `riken` token, no pointer to files outside the skill)
**Consistency Checks**: `! grep -qiE 'riken' skills/writing-design-canons/references/chapter-splitting.md` (expected: PASS)
**Commit**: `docs(writing-design-canons): make chapter-splitting example self-contained`

## Step 4: Scaffold release quartet
**Goal**: Wire the skill into the multi-semantic-release train (precedent: writing-history commit `70d34b7`).
**Implementation Logic**:
Add minimal `package.json`, one-liner `.releaserc.js` delegating to the shared factory, and seed `CHANGELOG.md` (`# Changelog`) so the npm workspace glob picks the skill up and CI tags `writing-design-canons@1.0.0`.
**Deliverables**: `skills/writing-design-canons/package.json` (`{"name":"writing-design-canons","version":"1.0.0"}`); `skills/writing-design-canons/.releaserc.js` (`module.exports = require('../../release-config')('writing-design-canons');`); `skills/writing-design-canons/CHANGELOG.md` (`# Changelog`)
**Consistency Checks**: `node -e "const c=require('./skills/writing-design-canons/.releaserc.js');process.exit(JSON.stringify(c.plugins).includes('writing-design-canons.skill')?0:1)"` (expected: PASS)
**Commit**: `feat(writing-design-canons): scaffold skill package and release quartet`
