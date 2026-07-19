# Archive committing-changes

**Goal**: Move `skills/committing-changes/` wholesale into `archive/committing-changes/` unmodified, then repoint live in-repo references to `writing-history`.
**Pre-conditions**:
- [ ] `commit_authoring` merged — the durable content of `git-workflow.md` has been migrated into `references/commit-authoring.md` (so the source can be safely archived)
**Success Gates**:
- ⬜ `archive/committing-changes/` exists and is byte-identical to the pre-move skill; `skills/committing-changes/` no longer exists
- ⬜ The move is recorded as pure renames (no content edits) in git
- ⬜ No *live* in-repo reference to `committing-changes` remains — residual matches occur only in archival locations (`archive/`, `__roadmap__/`, `__reports__/`, `CHANGELOG.md`)
**References**: [R02 committing-changes SKILL.md](../../../skills/committing-changes/SKILL.md) — the artifact being archived

## Step 1: git mv the skill into archive/ unmodified

**Goal**: Relocate the old skill without editing it, dropping it from the `skills/*` release glob.

**Implementation Logic**:
Create `archive/` at repo root if absent. Run `git mv skills/committing-changes archive/committing-changes` so the whole tree (SKILL.md, references/git-workflow.md, package.json, .releaserc.js, CHANGELOG.md) moves intact. Do NOT edit any moved file. Confirm the staged change is renames only.

**Deliverables**: `archive/committing-changes/` (moved tree: `SKILL.md`, `references/git-workflow.md`, `package.json`, `.releaserc.js`, `CHANGELOG.md`); `skills/committing-changes/` removed
**Consistency Checks**: `python3 -c "import os; assert os.path.isdir('archive/committing-changes') and os.path.isfile('archive/committing-changes/SKILL.md') and not os.path.exists('skills/committing-changes'); print('PASS')"` (expected: PASS)
**Commit**: `chore(writing-history): archive committing-changes skill unmodified`

## Step 2: Repoint live references to writing-history

**Goal**: Update active references so users/composers are directed to `writing-history`, leaving archival history untouched.

**Implementation Logic**:
Find references with `colgrep -e 'committing-changes'` (plain grep/rg are hook-blocked). For each hit, decide: **update** live references (skill indexes like `README.md`, any skill that composes committing-changes, active docs) to `writing-history`; **leave** archival/historical references (`archive/`, `__roadmap__/`, `__reports__/`, `CHANGELOG.md`, `.skill` assets) as-is. Mentions inside `skills/writing-history/` itself are permitted ONLY as supersede/migration notes (e.g. "Supersedes committing-changes"), never as live pointers directing users to use the old skill. Note in the Progress table which files were repointed.

**Deliverables**: updated live-reference files (e.g. `README.md` skill index and any composing skill manifests) pointing to `writing-history`
**Consistency Checks**: `python3 -c "import os,glob; allow=('archive/','__roadmap__/','__reports__/','dist/','node_modules/','.git/','skills/writing-history/'); bad=[p for p in glob.glob('**/*',recursive=True) if os.path.isfile(p) and not p.startswith(allow) and os.path.basename(p)!='CHANGELOG.md' and 'committing-changes' in open(p,'rb').read().decode('utf-8','ignore')]; assert not bad, bad; print('PASS')"` (expected: PASS)
**Commit**: `docs(writing-history): repoint live references from committing-changes to writing-history`
