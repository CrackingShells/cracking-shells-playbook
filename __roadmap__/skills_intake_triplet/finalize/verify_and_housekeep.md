# Verify and Housekeep

**Goal**: Confirm the three merged skills are release-ready as a set, and stop macOS Finder metadata from ever entering the repo.
**Pre-conditions**:
- [ ] All three intake branches merged into `main` (skills present and tracked at `skills/writing-prose/`, `skills/writing-design-canons/`, `skills/importing-design-figures/`)
- [ ] Worktree on branch `task/finalize-intake` branched from post-merge `main`
**Success Gates**:
- ⬜ `.gitignore` contains a `.DS_Store` rule; `git check-ignore skills/.DS_Store` exits 0 `[run]`
- ⬜ Verification report exists at `__reports__/skills_intake_triplet/` and records PASS for every check in Step 2 `[static]`
- ⬜ `git ls-files | grep -c '\.DS_Store'` is 0 `[run]`
**References**: [R01 Approved plan](/Users/hacker/.claude/plans/the-folder-contains-two-rustling-turing.md) — Verification section (per-leaf and end-to-end gates)

## Step 1: Ignore macOS Finder metadata
**Goal**: Prevent stray `.DS_Store` files (observed at `skills/.DS_Store` and inside transferred skill dirs) from being committed in the future.
**Implementation Logic**:
Append a `.DS_Store` rule to the repo root `.gitignore` under a short comment, consistent with the file's existing section style (`# External repo checkouts`, `# Node.js`, ...).
**Deliverables**: edited `.gitignore` (new `.DS_Store` entry with `# macOS` section comment)
**Consistency Checks**: `git check-ignore skills/.DS_Store` (expected: PASS)
**Commit**: `chore(gitignore): ignore macOS Finder metadata`

## Step 2: Cross-skill verification report
**Goal**: Record deterministic evidence that the intake set is coherent before the release push.
**Implementation Logic**:
Run and capture: (a) `grep -rniE 'sift|glean|riken|ECL|VN1' skills/writing-prose skills/writing-design-canons skills/importing-design-figures` → empty; (b) `uv run tools/package_skill.py skills/<each> dist/` → exit 0 ×3; (c) `node -e "require('<each>/.releaserc.js')"` → exit 0 ×3 with matching skill names; (d) npm workspace census showing all three new skills alongside existing ones; (e) relative-link resolution sweep over each skill's `.md` files. Write the outcomes as a findings-style report per the writing-reports skill's conventions under `__reports__/skills_intake_triplet/`.
**Deliverables**: `__reports__/skills_intake_triplet/` report file (round-versioned name per writing-reports convention) containing a check matrix with PASS/FAIL per skill per check
**Consistency Checks**: `ls __reports__/skills_intake_triplet/*.md` (expected: PASS)
**Commit**: `docs(skills-intake): record cross-skill intake verification report`
