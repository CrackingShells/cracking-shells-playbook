# Pre-commit Hook and Makefile

**Goal**: Create `.githooks/pre-commit` and `Makefile` so contributors can activate local auto-packaging with a single command.
**Pre-conditions**:
- [ ] `tools/package_skill.py` vendored and verified (`vendor-tools` complete)
- [ ] `dist/` gitignored (`dist-cleanup` complete)
- [ ] `uv` available or installable via the Makefile
**Success Gates**:
- ✅ `make dev-setup` exits 0 and sets `core.hooksPath` to `.githooks` [run]
- ✅ Staging any file under `skills/writing-reports/` and running `git commit` builds `dist/writing-reports.skill` and prints `[hook] Built dist/writing-reports.skill` [behavioral]
- ✅ Staging `skills/managing-roadmaps/scripts/dirtree-rdm/src/main.rs` prints the Rust reminder without blocking the commit [behavioral]
- ✅ `.githooks/pre-commit` is executable (`ls -la .githooks/pre-commit` shows `-rwxr-xr-x`) [run]
**References**: [R01 §Deliverable 5-6](~/.claude/plans/binary-jumping-trinket.md) — hook logic and Makefile targets

---

## Step 1: Create .githooks/pre-commit

**Goal**: Produce a hook that auto-builds `.skill` files locally on every commit touching `skills/`, without recompiling Rust or staging anything to git.

**Implementation Logic**:
Create `.githooks/pre-commit` and `chmod +x` it. The script uses `git diff --cached --name-only` to get staged files. First, grep for `^skills/managing-roadmaps/scripts/dirtree-rdm/.*\.(rs|toml)$` — if matched, echo the reminder to run `make build-rust-local` but do NOT exit non-zero. Then compute `changed=$(git diff --cached --name-only | grep '^skills/' | cut -d/ -f2 | sort -u)`. Loop over each skill name: if `[ -d "skills/$skill" ]` is true, run `uv run tools/package_skill.py "skills/$skill" dist/` and echo `[hook] Built dist/$skill.skill`. Do NOT call `git add` — `dist/` is gitignored and must not be staged.
**References**: [R01 §Deliverable 5](~/.claude/plans/binary-jumping-trinket.md) — hook logic
**Deliverables**: `.githooks/pre-commit` — Rust-change detection block, `changed` variable, per-skill packaging loop
**Consistency Checks**: `bash -n .githooks/pre-commit` (expected: PASS)
**Commit**: `feat(dev): add pre-commit hook for local skill auto-packaging`

---

## Step 2: Create Makefile

**Goal**: Produce a `Makefile` that bootstraps a contributor's environment and provides a local Rust build shortcut.

**Implementation Logic**:
Create `Makefile` at the repo root. Declare `.PHONY: dev-setup build-rust-local`. The `dev-setup` target runs: `git config core.hooksPath .githooks`, `chmod +x .githooks/pre-commit`, uv install via the official shell installer if not present, and `npm install`. The `build-rust-local` target `cd`s into `skills/managing-roadmaps/scripts/dirtree-rdm` and calls `bash build.sh local`. All recipe lines must be indented with tabs (Makefile requirement, not spaces).
**References**: [R01 §Deliverable 6](~/.claude/plans/binary-jumping-trinket.md) — Makefile targets
**Deliverables**: `Makefile` — `dev-setup` target, `build-rust-local` target
**Consistency Checks**: `make --dry-run dev-setup` (expected: PASS)
**Commit**: `feat(dev): add Makefile with dev-setup and build-rust-local targets`
