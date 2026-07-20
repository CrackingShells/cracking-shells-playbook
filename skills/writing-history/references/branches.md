# Branch Integration Discipline

This document is the prescriptive contract for landing a finished branch (or worktree) onto its parent. It composes with [`commit-authoring.md`](commit-authoring.md)'s commit-authoring discipline: every commit replayed during integration must already satisfy that contract before it is allowed to merge.

## Table of Contents

1. [Integration Contract](#integration-contract)
2. [Rationale](#rationale)
3. [Prohibitions](#prohibitions)
4. [Worktrees](#worktrees)
5. [Parallel Sibling Work](#parallel-sibling-work)

---

## Integration Contract

Landing a child branch onto its parent always follows the same four-step sequence, in this order, no exceptions:

1. **Rebase the child onto the parent's current tip.**
   ```bash
   git fetch origin
   git rebase <parent>
   ```
   Replay the child's commits on top of whatever the parent looks like *right now* — not the parent as it was when the child branched off. If the parent has moved, the rebase is what catches the branch up.

   `git fetch origin` only applies when the project has a remote configured. A local-only repository has nothing to fetch — skip that line and rebase directly against the local parent branch.

2. **Resolve any conflicts.**
   Work through conflicts commit-by-commit as the rebase pauses on them. Do not resolve by bulk-diffing the two tips and squashing the result into one blob — each replayed commit should still make sense on its own after resolution.

3. **Re-verify.**
   Nothing merges unverified. After the rebase completes (with or without conflicts), re-run the test suite / consistency checks / whatever this project uses to establish "this branch works." A rebase rewrites commit parentage; it can silently change behavior even when no conflict was flagged. Treat post-rebase code as unverified until proven otherwise.

4. **Integrate with `git merge --no-ff`.**
   ```bash
   git checkout <parent>
   git merge --no-ff <child>
   ```
   Only after re-verification passes. The `--no-ff` flag forces a merge commit even when a fast-forward is possible, so the branch leaves behind one explicit, labeled boundary in history.

   That merge commit is the visible boundary this whole contract exists to leave behind — which means it is itself a historical record, and the skill's Primary Directive (WHY over WHAT, see [commit-authoring.md](commit-authoring.md)) applies to it exactly as it applies to any other commit. Do not accept the auto-generated `Merge branch '<child>' into <parent>` message as-is: name the unit of work being integrated (the feature, fix, or leaf task the branch represents) and, if the project applies its derived commit-message convention to merge commits, write the message in that vocabulary. This is a description of the expectation, not a fixed template — a rigid `type(scope):` shape is not mandated for merge commits, only a message that lets a future reader understand what landed and why, without re-reading every replayed commit.

This sequence — rebase, resolve, re-verify, `--no-ff` merge — is the whole contract. Skipping the re-verify step because "the rebase applied cleanly" is the most common shortcut taken and the most common source of regressions that slip through.

If the project maintains its version number and changelog by hand rather than through release automation, landing a merge is often the moment that update falls due — see [semver-changelog.md](semver-changelog.md) for how commits map to version bumps and changelog entries.

---

## Rationale

The contract optimizes for two properties at once, because either alone is insufficient:

- **Linear history.** Rebasing the child onto the parent's tip means the commit graph reads top-to-bottom as one coherent narrative — no interleaved, out-of-order commits from two lines of development tangled together. A reviewer or `git bisect` run later can follow the story without untangling parallel timelines.
- **A visible, reviewable per-branch boundary.** `--no-ff` refuses to let the merge disappear into a fast-forward. Every unit of work — a feature, a fix, a leaf task — leaves exactly one merge commit that names it. Stakeholders scanning `git log --graph` can see where each branch started and ended, and can revert or cherry-pick the whole unit by its merge commit.

Linear history without a visible boundary (fast-forward merges) hides *that* an integration happened at all. A visible boundary without linear history (merging without rebasing first, or merging across a criss-crossed graph) produces a history that is technically inspectable but practically unreadable. The contract requires both.

---

## Prohibitions

- **No history-hiding fast-forward.** Never merge with a plain `git merge` (or `--ff-only`) when the child branch represents a completed unit of work. A fast-forward advances the parent pointer with no merge commit, erasing the record that a branch existed and was integrated deliberately. Always use `--no-ff` for branch integration.
- **No criss-cross or same-root merges.** Do not merge two branches that both still share an un-rebased common ancestor and have diverged since (a criss-cross merge), and do not merge a branch back into the same root it forked from without rebasing it forward first. Both produce a tangled graph with multiple merge bases, which defeats linear history and makes `git bisect` unreliable.
- **No force-push to shared branches.** `git push --force` (or `--force-with-lease`) is permitted only on a branch that is exclusively yours — typically your own unshared task branch — and never on `main`, a parent integration branch, or any branch another agent or collaborator might be building on. Force-pushing a shared branch rewrites history out from under anyone who has already based work on it.

---

## Worktrees

**This section is conditional: if worktrees are in play** for a given piece of work, here's how they interact with the contract above. It is not a requirement to use them.

When worktrees are used for parallel work, the same integration contract above still governs how each worktree's branch lands on its parent. The worktree is just where the work happens; it does not change the rebase → resolve → re-verify → `--no-ff` merge sequence. On top of that, worktree usage typically adds:

- **Branch-per-worktree.** Each worktree checks out exactly one branch, and that branch belongs to exactly one worktree at a time. Do not point two worktrees at the same branch — Git will refuse it, and even if it didn't, it would defeat the point of isolation.
- **Path-scoping.** Each worktree lives at its own filesystem path so that parallel agents or contributors working in different worktrees never collide on working-directory state, even though they share the same underlying repository object store.
- **Init/cleanup lifecycle.** A worktree is created for the duration of a unit of work (`git worktree add <path> <branch>`) and removed once that branch has been integrated and is no longer needed (`git worktree remove <path>`), analogous to how a branch itself is deleted once merged. Leaving stale worktrees around after integration is the worktree-equivalent of leaving stale merged branches around — harmless but untidy, and worth cleaning up.

**Worktrees are optional, not mandated.** Some workflows deliberately avoid them — for example, a workflow where agents take turns working against a branch rather than editing it in parallel doesn't need worktree isolation, because there's no simultaneous parallel state to keep from colliding. Read this section as "here's how, if you're reaching for worktrees," not "you must reach for worktrees."

---

## Parallel Sibling Work

Independent sibling branches (whether or not they use worktrees) integrate **one at a time**, each under the full contract above:

1. Pick one sibling branch to land first — order by readiness (verified and rebase-clean) or by dependency, not arbitrarily.
2. Rebase that sibling onto the parent's current tip, resolve, re-verify, `--no-ff` merge. The parent's tip has now moved.
3. Rebase the *next* sibling onto the parent's **new** tip (which now includes the first sibling's merge) — not onto the tip it originally branched from. Resolve, re-verify, merge.
4. Repeat until all siblings have landed.

Each sibling always rebases onto whatever the parent looks like at the moment of its own integration, never onto a stale snapshot. This keeps the one-sibling-at-a-time sequence from silently becoming a criss-cross merge: by construction, each new merge base is a strict descendant of the previous one.
