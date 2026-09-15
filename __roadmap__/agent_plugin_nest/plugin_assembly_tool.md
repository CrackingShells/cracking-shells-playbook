# Plugin Assembly Tool

**Goal**: Assemble `plugins/<name>/skills/<name>/` from the canonical `skills/<name>/`, and keep that tracked copy from drifting via the pre-commit hook and a CI gate.
**Pre-conditions**:
- [ ] `tools/package_skill.py` exposes reusable exclusion logic (`should_exclude`, `EXCLUDE_DIRS`, `EXCLUDE_FILES`, `ROOT_EXCLUDE_DIRS`)
- [ ] `.githooks/pre-commit` is the active hooks path (`git config core.hooksPath` returns `.githooks`)
**Success Gates**:
- ⬜ `uv run tools/assemble_plugin.py <name>` produces `plugins/<name>/skills/<name>/SKILL.md` [run]
- ⬜ The assembled tree contains no `package.json`, `CHANGELOG.md`, `.releaserc.js` or `__pycache__` [run]
- ⬜ Re-running assembly on an unchanged skill leaves `git status --porcelain plugins/` empty [run]
- ⬜ Staging a change under `skills/<name>/` and committing refreshes `plugins/<name>/` automatically [behavioral]
- ⬜ CI fails when `plugins/` is out of sync with `skills/` [run]
**References**: [R01 §Layout, and the artifact that must be tracked](~/.claude/plans/good-news-overall-it-s-gleaming-wreath.md) — why `plugins/` is committed while `dist/` is not

## Step 1: Write the assembly tool

**Goal**: One command that materialises a plugin's skills subtree from its canonical source.

**Implementation Logic**:
Add `tools/assemble_plugin.py` taking a skill name and copying `skills/<name>/` to `plugins/<name>/skills/<name>/`. Import and reuse `should_exclude` from `tools/package_skill.py` rather than restating the exclusion rules — the two must never disagree about what belongs in a distributed skill, and `package_skill.py` already filters `package.json`, `CHANGELOG.md`, `.DS_Store`, `__pycache__`, `*.pyc` and a root-level `evals/`. Also exclude `.releaserc.js`, which `package_skill.py` does not list because a `.skill` zip never contained one. Make the copy idempotent: remove the destination subtree first, then rewrite it, so a deleted source file disappears from the assembled copy instead of lingering. Do not write manifests here — `spawn_plugin.py` owns those.

Give the script a PEP 723 header declaring `dependencies = ["pyyaml"]`. This is not optional and not obvious: importing `package_skill` transitively imports `quick_validate`, which imports `yaml` at module level, so `uv run tools/assemble_plugin.py` fails with `ModuleNotFoundError: No module named 'yaml'` without it. Measured on 2026-09-16.

Report a spec defect rather than inventing behaviour if `should_exclude`'s signature makes reuse awkward: its `rel_path` is relative to the skill's *parent*, which the caller must reproduce.
**Deliverables**: `tools/assemble_plugin.py` — functions `assemble(skill_name, repo_root)` and `main()`, constant `EXTRA_EXCLUDE_FILES = {".releaserc.js"}`
**Consistency Checks**: `uv run --with pyyaml python3 -c "import sys;sys.path.insert(0,'tools');import package_skill;assert callable(package_skill.should_exclude)"` (expected: PASS)
**Commit**: `feat(tools): assemble a plugin skills subtree from the canonical skill`

## Step 2: Regenerate on commit

**Goal**: Make the tracked copy refresh itself, the way `dist/` already does.

**Implementation Logic**:
Extend `.githooks/pre-commit`. It already derives the set of changed skills with `git diff --cached --name-only | grep '^skills/' | cut -d/ -f2 | sort -u` and rebuilds `dist/<skill>.skill` for each. Add a second action in the same loop: when `plugins/<skill>/` exists, re-run `assemble_plugin.py` for it and `git add plugins/<skill>` so the refreshed copy joins the commit being made. Guard on directory existence so skills that are not plugins are untouched, and so the hook is a no-op until `playbook_plugins` creates the trees. Keep the hook non-blocking in spirit — it repairs drift rather than rejecting the commit, matching the existing Rust-source warning that informs without failing.
**Deliverables**: `.githooks/pre-commit` — an assembly branch inside the existing `for skill in $changed` loop, guarded by `[ -d "plugins/$skill" ]`
**Consistency Checks**: `test "$(git config core.hooksPath)" = ".githooks" && test -x .githooks/pre-commit` (expected: PASS)
**Commit**: `feat(hooks): refresh the assembled plugin tree on staged skill changes`

## Step 3: Fail CI on drift

**Goal**: Catch the case the hook cannot — a commit made with hooks bypassed or from another machine.

**Implementation Logic**:
Add a `drift` job to `.github/workflows/release.yml` that checks out the repo, installs `uv`, re-runs `assemble_plugin.py` for every directory under `plugins/`, and fails if `git status --porcelain plugins/` is non-empty. Run it before the `release` job and make `release` depend on it, so a drifted tree never publishes: an installed plugin whose skills lag its source is exactly the silent failure this campaign is trying to remove. Print the offending diff on failure — a bare non-zero exit tells a reader nothing about which skill drifted.
**Deliverables**: `.github/workflows/release.yml` — job `drift` with steps `actions/checkout@v4`, `astral-sh/setup-uv@v5`, a re-assembly loop and a `git status --porcelain plugins/` gate; `release.needs` extended to include `drift`
**Consistency Checks**: `python3 -c "import yaml,sys;d=yaml.safe_load(open('.github/workflows/release.yml'));assert 'drift' in d['jobs'];assert 'drift' in d['jobs']['release']['needs']"` (expected: PASS)
**Commit**: `ci(release): fail the pipeline when the assembled plugin tree drifts`
