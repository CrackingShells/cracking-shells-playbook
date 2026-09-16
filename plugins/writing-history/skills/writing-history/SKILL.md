---
name: writing-history
description: Guides the full git-discipline lifecycle for a project's commit history, for both code and documents. Covers deriving a project-specific commit vocabulary from existing history, setting up commit conventions together with semantic versioning and changelog automation, authoring individual commits against that derived vocabulary, integrating finished branches via rebase-then-merge-with-no-ff, and conditionally cleaning up worktrees once integration is complete. Use when the user asks to set up commit or versioning conventions, derive or apply a commit vocabulary, author or draft a commit message, integrate or merge a branch, rebase work before merging, or clean up a git worktree after a feature lands — including cases mixing code changes with documentation changes.
---

# Writing History

## Primary Directive

**WHY over WHAT.** A commit and a merge are historical records — each must be justified by the reasoning behind it, not merely by the mechanics of what changed. A worktree teardown leaves no such record; it is housekeeping that keeps parallel work clean once a branch has landed, not a claim on the historical record. Every action this skill takes on that record should leave behind a history a future reader can trust and reconstruct intent from.

**The commit vocabulary is derived, never assumed.** No reference in this skill hardcodes a `feat`/`fix`/`docs`/... type set as universal truth. The authorized types, scopes, and version-bump mapping always come from the project itself — its own convention/versioning machinery (whatever form that takes) if it has any, else its own history and `CONTRIBUTING` docs, else a setup interview run to stand one up deliberately. Every reference below shares this precedence; none of them substitutes a convention of its own.

## Navigation

Read the routed reference before acting — do not proceed from this table alone; the table only tells you where the rule set lives, not what it says.

| Situation | Reference |
|---|---|
| Author, stage, or commit a change (code or prose) | [`references/commit-authoring.md`](references/commit-authoring.md) |
| Project has no commit conventions, changelog, or versioning set up yet | [`references/convention-setup.md`](references/convention-setup.md) |
| Understand how versions and changelog entries derive from commits | [`references/semver-changelog.md`](references/semver-changelog.md) |
| Merge, rebase, integrate a branch, or work in parallel/worktrees | [`references/branches.md`](references/branches.md) |
