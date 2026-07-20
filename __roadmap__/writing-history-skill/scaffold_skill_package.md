# Scaffold Skill Package

**Goal**: Create the `skills/writing-history/` package skeleton (directory, `references/`, a minimal valid `SKILL.md`, and the release quartet) so downstream leaves can add content and `multi-semantic-release` recognizes the unit.
**Pre-conditions**:
- [ ] `release-config.js` and `tools/package_skill.py` exist at repo root
- [ ] Root `package.json` has `workspaces: ["skills/*"]`
**Success Gates**:
- ⬜ `skills/writing-history/{package.json,.releaserc.js,CHANGELOG.md,SKILL.md}` all exist
- ⬜ `SKILL.md` frontmatter is valid: `name: writing-history` (gerund, lowercase+hyphens, ≤64), `description` ≤1024 chars
- ⬜ `skills/writing-history/references/` exists in git (tracked)
**References**: [R04 release-config.js](../../release-config.js) — the factory `.releaserc.js` delegates to; [R02 committing-changes SKILL.md](../../skills/committing-changes/SKILL.md) — quartet shape to mirror

## Step 1: Create package skeleton, release quartet, and minimal SKILL.md

**Goal**: Establish a release-recognized, BNF-valid skill package that later leaves fill in.

**Implementation Logic**:
Create `skills/writing-history/` with:
1. `package.json` — exactly `{"name":"writing-history","version":"1.0.0"}` (mirrors the per-skill version-marker convention).
2. `.releaserc.js` — exactly `module.exports = require('../../release-config')('writing-history');`.
3. `CHANGELOG.md` — empty placeholder (semantic-release manages it; a single `# Changelog` heading is fine).
4. `references/.gitkeep` — so the `references/` directory is tracked before content leaves add files.
5. `SKILL.md` — minimal but VALID: frontmatter `name: writing-history` + a third-person `description` (≤1024 chars) that states scope and triggers (commit authoring against a *derived* vocabulary; convention/semver/changelog setup; rebase-then-`merge --no-ff` integration; conditional worktree hygiene; for code and documents; supersedes committing-changes). Body: a one-line primary directive (WHY over WHAT) and a placeholder note that the navigation table is finalized by the `skill_dispatcher` leaf. Keep body <100 lines.

Do NOT author the four reference files here — that is the `author/` stage.

**Deliverables**: `skills/writing-history/package.json`, `skills/writing-history/.releaserc.js`, `skills/writing-history/CHANGELOG.md`, `skills/writing-history/references/.gitkeep`, `skills/writing-history/SKILL.md` (frontmatter keys `name`, `description`; body with `Primary Directive` heading)
**Consistency Checks**: `python3 -c "import yaml; d=open('skills/writing-history/SKILL.md').read(); f=yaml.safe_load(d.split('---')[1]); assert f['name']=='writing-history'; assert len(f['description'])<=1024; import os; [os.stat(p) for p in ['skills/writing-history/package.json','skills/writing-history/.releaserc.js','skills/writing-history/CHANGELOG.md','skills/writing-history/references/.gitkeep']]; print('PASS')"` (expected: PASS)
**Commit**: `feat(writing-history): scaffold skill package and release quartet`
