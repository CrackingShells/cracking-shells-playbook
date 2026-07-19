# Verify Release, Packaging & Triggers

**Goal**: Prove the refactor is release-ready and self-consistent, and record the evidence in a verification report.
**Pre-conditions**:
- [ ] `skill_dispatcher` and `archive_committing_changes` merged into the integration branch
- [ ] `npm ci` has been run (so `node_modules/` exists); `uv` available
**Success Gates**:
- ⬜ `multi-semantic-release --dry-run` recognizes `writing-history`; `package_skill.py` emits `dist/writing-history.skill`
- ⬜ Every reference in `SKILL.md`'s navigation table resolves; no link points at the removed `git-workflow.md`
- ⬜ `archive/committing-changes/` intact; `skills/committing-changes/` gone; no live `committing-changes` references remain
- ⬜ A verification report capturing the command outputs exists
**References**: [R03 release pipeline architecture](../../../../__reports__/skill_cicd/01-architecture_analysis_v0.md) — expected release-unit behavior

## Step 1: Run gates and record the verification report

**Goal**: Execute every campaign-level gate and capture PASS/FAIL evidence in one report.

**Implementation Logic**:
Run, capturing output:
1. `npx multi-semantic-release --dry-run 2>&1 | python3 -c "import sys; assert 'writing-history' in sys.stdin.read(); print('recognized')"` — confirm the unit is seen with no config error.
2. `uv run tools/package_skill.py skills/writing-history dist/ && test -f dist/writing-history.skill` — confirm packaging.
3. Link-check: every `references/*.md` named in `SKILL.md` exists; nothing references `git-workflow.md`.
4. Archive integrity + no live `committing-changes` references (reuse the check from `archive_committing_changes` Step 2).
Write `__reports__/writing-history-skill/00-verification_v0.md` recording each command, its output, and PASS/FAIL. If any gate fails, raise a flag to the coordinator (do not paper over it) — the report must reflect reality.

**Deliverables**: `__reports__/writing-history-skill/00-verification_v0.md` (sections: `Release Dry-Run`, `Packaging`, `Link-Check`, `Archive Integrity`, `Verdict`)
**Consistency Checks**: `uv run tools/package_skill.py skills/writing-history dist/ && test -f dist/writing-history.skill && python3 -c "import os; d=open('skills/writing-history/SKILL.md').read(); refs=['commit-authoring.md','convention-setup.md','semver-changelog.md','branches.md']; assert all(os.path.exists('skills/writing-history/references/'+r) for r in refs); assert 'git-workflow.md' not in d; assert os.path.isdir('archive/committing-changes') and not os.path.exists('skills/committing-changes'); print('PASS')"` (expected: PASS)
**Commit**: `docs(writing-history): record release, packaging, and trigger verification`
