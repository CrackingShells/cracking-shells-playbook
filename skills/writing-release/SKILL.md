---
name: writing-release
description: Operational manual for how work reaches `main` and turns into a release in any repository. Covers choosing an integration lane (reviewed PR, self-merge PR, or direct push), writing pull-request titles and bodies that become merge commits, setting up CI-driven releases with tooling derived from what the repo contains, deciding whether a prerelease channel is warranted, and recovering from missing or failed releases. Use when opening, titling or landing a pull request to main; deciding whether a change needs a PR; setting up releases or a CI release job; choosing release tooling; investigating "why didn't it release" or "the release job failed"; adding a prerelease, dev or rc channel; or asked to "cut a release".
---

# Writing Release

## Primary Directives

**One channel, and the lightest lane that still gates what the change risks.** `main` is the only place releases come from, because every extra channel is a second integration to keep in sync. A change reaches `main` by a reviewed PR, a self-merge PR or a direct push, chosen by what merging would release and what it could break. A heavier lane than needed costs latency; a lighter one costs a release nobody reviewed.

**A release is a consequence of merging, not a ceremony.** CI cuts it after a change lands: no release PRs, no local release commands, no hand-made tags. One path is easier to sustain, audit and recover than several. Asked to "cut a release", land a release-type commit through the right lane; do not tag by hand.

**Tooling is derived from what the repo contains, never assumed.** Language, layout, existing config and conventions choose the release tool. Prefer the main language's toolchain and the leanest setup that works, so CI needs nothing extra to maintain.

**This skill composes with `writing-history`.** Commit vocabulary and messages, version-bump semantics, branch-to-parent integration and worktrees belong to the `writing-history` skill. This manual says what happens to that work on its way to `main` and after; it never restates it.

## Navigation

Read the routed reference before acting; the table says where a rule lives, not what it says.

| Situation | Reference |
|:--|:--|
| Deciding whether a change needs a PR, choosing a lane, mapping lanes in a repo, understanding the lifecycle and its invariants, or asked to "cut a release" | [`references/lifecycle.md`](references/lifecycle.md) |
| Opening, titling, writing or landing a PR; a self-merge was blocked; what to do after merging | [`references/pull-requests.md`](references/pull-requests.md) |
| Setting up releases in a repo: what a pipeline needs, choosing tooling from the repo's contents, repository settings, the contributing-guide section | [`references/setting-up.md`](references/setting-up.md) |
| Writing or changing the CI release job: triggers, concurrency, permissions and tokens, branch protection, the bump commit, monorepos, re-runs | [`references/ci-release-job.md`](references/ci-release-job.md) |
| Asked for a prerelease, dev or rc channel, or a long-lived development branch | [`references/channels.md`](references/channels.md) |
| No release appeared, the release job failed, a bump push was rejected, the first release or tags are missing, history was rewritten on `main`, or local `main` is stale | [`references/recovery.md`](references/recovery.md) |
