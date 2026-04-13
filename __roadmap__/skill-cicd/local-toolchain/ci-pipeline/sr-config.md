# Semantic Release Config

**Goal**: Create root `package.json` and `.releaserc.js` so `multi-semantic-release` can analyze commits, compute per-skill version bumps, and publish GitHub Releases.
**Pre-conditions**:
- [ ] All 4 `skills/<name>/package.json` markers created (`skill-manifests` complete)
- [ ] `npm` available in PATH
- [ ] `GH_TOKEN` or `GITHUB_TOKEN` available for dry-run (can be a personal token with `repo` scope)
**Success Gates**:
- ✅ `npm install` exits 0 and creates `node_modules/` [run]
- ✅ `GITHUB_TOKEN=<token> npx multi-semantic-release --dry-run` exits 0 and prints expected tag format for at least one skill [run]
- ✅ `grep CHANGELOG.md .releaserc.js` returns a match [static]
**References**: [R01 §Deliverable 7-8](~/.claude/plans/binary-jumping-trinket.md) — exact package versions and plugin configuration

---

## Step 1: Create root package.json

**Goal**: Declare the monorepo and pin exact dev dependency versions.

**Implementation Logic**:
Create `package.json` at the repo root with `"private": true` (prevents accidental root publish), `"workspaces": ["skills/*"]` (registers each skill as a workspace package required by semantic-release-monorepo), and `"devDependencies"` with exact pinned versions: `semantic-release@25.0.3`, `semantic-release-monorepo@8.0.2`, `@semantic-release/changelog@6.0.3`, `@semantic-release/exec@7.1.0`, `@semantic-release/git@10.0.1`, `@semantic-release/github@12.0.6`. Use exact versions, not ranges, for reproducible CI builds.
**References**: [R01 §Deliverable 7](~/.claude/plans/binary-jumping-trinket.md) — exact pinned versions
**Deliverables**: `package.json` — `private`, `workspaces`, `devDependencies` fields with 6 pinned dependencies
**Consistency Checks**: `npm install` (expected: PASS)
**Commit**: `feat(release): add root package.json for semantic-release-monorepo`

---

## Step 2: Create .releaserc.js

**Goal**: Configure the semantic-release plugin chain: analyze commits, generate changelog, package artifact, publish to GitHub, commit version bump.

**Implementation Logic**:
Create `.releaserc.js` with `branches: ["main"]` and a `plugins` array in this order: `@semantic-release/commit-analyzer`, `@semantic-release/release-notes-generator`, `@semantic-release/changelog` (changelogFile: `skills/${name}/CHANGELOG.md`), `@semantic-release/exec` (prepareCmd: `uv run tools/package_skill.py skills/${name} dist/`), `@semantic-release/github` (assets: `dist/${name}.skill`), `@semantic-release/git` (assets: package.json + CHANGELOG.md, message with `[skip ci]`). The `${name}` token is interpolated by semantic-release-monorepo from each skill's `package.json` `name` field. Plugin order is load-bearing: exec (build) must precede github (upload).
**References**: [R01 §Deliverable 8](~/.claude/plans/binary-jumping-trinket.md) — plugin chain spec
**Deliverables**: `.releaserc.js` — `branches` field, `plugins` array with 6 plugin entries
**Consistency Checks**: `node -e "require('./.releaserc.js')"` (expected: PASS)
**Commit**: `feat(release): add .releaserc.js semantic-release plugin chain`
