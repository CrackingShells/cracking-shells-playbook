# Remove dist/ from Git Tracking

**Goal**: Stop tracking `dist/` in git; CI publishes `.skill` files as GitHub Release assets instead.
**Pre-conditions**:
- [ ] No open PRs that depend on `dist/` being tracked (check git log)
**Success Gates**:
- ✅ `git ls-files dist/` returns empty output [run]
- ✅ `cat .gitignore | grep dist/` returns `dist/` [static]
- ✅ `git status` after running the packager shows `dist/` as untracked, not staged [run]
**References**: [R01 §Deliverable 2](~/.claude/plans/binary-jumping-trinket.md) — dist cleanup decision

---

## Step 1: Add dist/ to .gitignore and untrack

**Goal**: Make `dist/` invisible to git so contributors never accidentally commit stale artifacts.

**Implementation Logic**:
Two operations in one commit: (1) Append `dist/` to `.gitignore` (create if absent; check for conflicting `!dist/` overrides). (2) Run `git rm -r --cached dist/` to untrack all files under `dist/` without deleting them from disk. Local `.skill` files remain for dev use and are rebuilt by the hook on the next commit; they are never staged back into git.

**References**: [R01 §Deliverable 2](~/.claude/plans/binary-jumping-trinket.md) — dist cleanup decision
**Deliverables**: `.gitignore` — `dist/` entry
**Consistency Checks**: `[ -z "$(git ls-files dist/)" ]` (expected: PASS)
**Commit**: `chore(dist): remove dist/ from git tracking, add to gitignore`
