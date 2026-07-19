# Author branches Reference

**Goal**: Author `skills/writing-history/references/branches.md` — the prescriptive branch-integration contract (rebase → re-verify → `merge --no-ff`) plus a *conditional* worktree/parallel-work note.
**Pre-conditions**:
- [ ] `scaffold_skill_package` merged (skill dir + `references/` exist)
**Success Gates**:
- ⬜ `references/branches.md` exists and prescribes: rebase child onto the parent's current tip → resolve conflicts → re-verify → `git merge --no-ff`
- ⬜ It states the rationale (linear history + visible, reviewable per-branch boundary) and the prohibitions (no history-hiding fast-forward, no criss-cross/same-root merges, no force-push to shared branches)
- ⬜ The worktree section is framed conditionally ("if worktrees are in play") and explicitly notes some flows (e.g. umbod-style asymmetric two-agent work) intentionally avoid worktrees — guidance, not a mandate
**References**: [R02 committing-changes SKILL.md](../../../skills/committing-changes/SKILL.md) — commit discipline this integration contract composes with

## Step 1: Write branches.md (integration contract + conditional worktrees)

**Goal**: Produce the merge/rebase discipline reference with the conditional worktree note.

**Implementation Logic**:
Author `skills/writing-history/references/branches.md` with:
1. **Integration Contract** — the required sequence for landing a child branch/worktree onto its parent: (a) rebase the child onto the parent's current tip (`git rebase <parent>`) so commits replay linearly; (b) resolve any conflicts; (c) **re-verify** after conflict resolution (nothing merges unverified); (d) integrate with `git merge --no-ff` so the branch becomes one explicit, labeled merge commit.
2. **Rationale** — why: linear history AND a visible merge boundary per unit of work, readable by stakeholders; a silent fast-forward hides the branch, a same-root merge tangles history.
3. **Prohibitions** — no fast-forward that hides a branch; no criss-cross / divergent same-root merges; no force-push to shared branches (only your own task branch, and only when it is yours alone).
4. **Worktrees (conditional)** — "IF worktrees are in play": branch-per-worktree, path-scoping so parallel work does not collide, and an init/cleanup lifecycle. State plainly that worktrees are optional: some methodologies (e.g. `/umbod`'s asymmetric two-agent flow) deliberately avoid them, and that is fine — this section is a how-to for when they are used, not a requirement to use them.
5. **Parallel Sibling Work** — how independent sibling branches integrate one-by-one under the same contract.

**Deliverables**: `skills/writing-history/references/branches.md` (headings: `Integration Contract`, `Rationale`, `Prohibitions`, `Worktrees`, `Parallel Sibling Work`)
**Consistency Checks**: `python3 -c "t=open('skills/writing-history/references/branches.md').read(); tl=t.lower(); assert '--no-ff' in t and 'rebase' in tl and 'worktree' in tl and 'if worktree' in tl and ('umbod' in tl or 'not a mandate' in tl or 'optional' in tl); print('PASS')"` (expected: PASS)
**Commit**: `feat(writing-history): add branches reference for rebase and merge discipline`
