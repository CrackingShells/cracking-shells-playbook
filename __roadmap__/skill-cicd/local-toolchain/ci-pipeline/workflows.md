# Per-Skill GitHub Actions Workflows

**Goal**: Create 4 workflow files — one per skill — with precise path triggers; `managing-roadmaps` includes the Rust cross-compilation matrix, the others are simple release-only jobs.
**Pre-conditions**:
- [ ] `.github/workflows/` directory exists (or create it)
- [ ] `sr-config` complete (`.releaserc.js` and root `package.json` present)
- [ ] `astral-sh/setup-uv` action version confirmed (check latest at github.com/astral-sh/setup-uv/releases)
**Success Gates**:
- ✅ All 4 workflow files exist under `.github/workflows/` [static]
- ✅ Each workflow triggers only on changes under its own `skills/<name>/**` path [static]
- ✅ `actionlint .github/workflows/*.yml` exits 0 (install via `brew install actionlint` if needed) [run]
- ✅ `release-managing-roadmaps.yml` has a `build-rust-binaries` job with 3 matrix entries (macos, ubuntu, windows) [static]
**References**: [R01 §Deliverable 9](~/.claude/plans/binary-jumping-trinket.md) — full workflow YAML

---

## Step 1: Create release-managing-roadmaps.yml

**Goal**: Produce the most complex workflow — two-job pipeline that cross-compiles Rust across 3 OS runners then releases.

**Implementation Logic**:
Create `.github/workflows/release-managing-roadmaps.yml`. Trigger: `push` to `main`, `paths: ["skills/managing-roadmaps/**"]`. Job `build-rust-binaries` uses a 3-entry matrix: `macos-latest` builds `aarch64-apple-darwin x86_64-apple-darwin` and uploads artifact `dirtree-rdm-darwin`; `ubuntu-latest` builds `x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu` (installs `cross` crate for arm64 cross-compilation) and uploads `dirtree-rdm-linux`; `windows-latest` builds `x86_64-pc-windows-msvc` and uploads `dirtree-rdm-windows`. All runners use `dtolnay/rust-toolchain@stable` and `bash build.sh <targets>` with `shell: bash` (required for Windows Git Bash). Job `release` declares `needs: build-rust-binaries`, downloads all 3 artifacts to `skills/managing-roadmaps/scripts/dirtree-rdm/bin/`, then runs `npm ci` + `npx multi-semantic-release` with `GITHUB_TOKEN` and `permissions: contents: write, issues: write, pull-requests: write`.
**References**: [R01 §Deliverable 9](~/.claude/plans/binary-jumping-trinket.md) — workflow YAML spec
**Deliverables**: `.github/workflows/release-managing-roadmaps.yml` — `build-rust-binaries` 3-entry matrix job, `release` job with `needs` and 3 artifact download steps
**Consistency Checks**: `actionlint .github/workflows/release-managing-roadmaps.yml` (expected: PASS)
**Commit**: `feat(ci): add release-managing-roadmaps workflow with Rust cross-compilation matrix`

---

## Step 2: Create the 3 remaining skill workflows

**Goal**: Produce `release-committing-changes.yml`, `release-writing-release.yml`, `release-writing-reports.yml` — identical single-job structure, no Rust build.

**Implementation Logic**:
Each workflow is a single-job file. Trigger: `push` to `main`, `paths: ["skills/<name>/**"]` where `<name>` is the skill directory name. The `release` job runs on `ubuntu-latest` with `permissions: contents: write, issues: write, pull-requests: write`. Steps: `actions/checkout@v4` (fetch-depth: 0, persist-credentials: false), `actions/setup-node@v4` (node 20), `astral-sh/setup-uv@v5`, `npm ci`, `npx multi-semantic-release` with `GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}`. Substitute the correct skill name in the workflow `name:` field, `paths:` trigger, and file name for all 3 files.
**References**: [R01 §Deliverable 9](~/.claude/plans/binary-jumping-trinket.md) — simple workflow structure
**Deliverables**: `.github/workflows/release-committing-changes.yml`, `.github/workflows/release-writing-release.yml`, `.github/workflows/release-writing-reports.yml`
**Consistency Checks**: `actionlint .github/workflows/release-committing-changes.yml .github/workflows/release-writing-release.yml .github/workflows/release-writing-reports.yml` (expected: PASS)
**Commit**: `feat(ci): add release workflows for committing-changes, writing-release, writing-reports`
