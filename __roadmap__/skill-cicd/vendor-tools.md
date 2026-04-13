# Vendor Package Skill Tools

**Goal**: Vendor `package_skill.py` and `quick_validate.py` from the skill-creator plugin into `tools/` with three patches applied.
**Pre-conditions**:
- [ ] Skill-creator plugin accessible at `/Users/hacker/Library/Application Support/Claude/local-agent-mode-sessions/skills-plugin/` (the version with full exclusion logic, not the default agent install)
**Success Gates**:
- ✅ `uv run tools/package_skill.py skills/committing-changes dist/` exits 0 [run]
- ✅ `python3 -c "import zipfile; n=zipfile.ZipFile('dist/committing-changes.skill').namelist(); assert not any('package.json' in x or 'CHANGELOG.md' in x for x in n)"` exits 0 [run]
- ✅ `tools/package_skill.py` contains the PEP 723 `# /// script` block [static]
**References**: [R01 §Deliverable 1](~/.claude/plans/binary-jumping-trinket.md) — exact patch list

---

## Step 1: Vendor package_skill.py with three patches

**Goal**: Produce `tools/package_skill.py` that is self-contained, correctly excludes monorepo markers and changelogs, and resolves its own deps via `uv`.

**Implementation Logic**:
Read the canonical source from the session plugin path (the version with `ROOT_EXCLUDE_DIRS`, `EXCLUDE_DIRS`, `EXCLUDE_FILES` — NOT the default agent install at `~/.agents/`). Apply three patches:
1. PEP 723 inline script metadata at the very top (before any imports):
   ```python
   # /// script
   # dependencies = ["pyyaml"]
   # ///
   ```
2. Sibling import fix immediately after the standard imports (adapt if the source already uses a different resolution strategy):
   ```python
   import sys
   from pathlib import Path
   sys.path.insert(0, str(Path(__file__).parent))
   ```
3. Add `"package.json"` and `"CHANGELOG.md"` to `EXCLUDE_FILES`.

**Deliverables**: `tools/package_skill.py` — `package_skill(skill_path, output_dir)` function, `EXCLUDE_FILES` set (must contain `"package.json"`, `"CHANGELOG.md"`), `ROOT_EXCLUDE_DIRS` set, `EXCLUDE_DIRS` set
**Consistency Checks**: `uv run tools/package_skill.py skills/committing-changes dist/` (expected: PASS)
**Commit**: `feat(tools): vendor package_skill.py with pep723 deps and exclusion patches`

---

## Step 2: Vendor quick_validate.py with import fix

**Goal**: Produce `tools/quick_validate.py` that works as a sibling import resolved via the `sys.path.insert` added in Step 1.

**Implementation Logic**:
Read `quick_validate.py` from the same plugin source. The `yaml` dependency is already covered by the PEP 723 block in `package_skill.py`. No other changes needed — the `sys.path.insert` from Step 1 makes `quick_validate` importable as a sibling. Verify the import works end-to-end by running the packager against a skill that has a valid SKILL.md (the validate call is exercised internally).

**Deliverables**: `tools/quick_validate.py` — `quick_validate(path)` or equivalent function imported by `package_skill.py`
**Consistency Checks**: `uv run tools/package_skill.py skills/writing-reports dist/` (expected: PASS)
**Commit**: `feat(tools): vendor quick_validate.py as sibling to package_skill`
