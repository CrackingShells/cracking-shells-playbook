# Recovery

What to do when a release does not happen, fails halfway, or leaves `main` in a state the next change trips over. Each entry runs symptom, confirm, fix, prevention. Confirm before changing anything: the commands are read-only, and nothing here force-pushes `main` except the one deliberate case in [History Rewritten on main](#history-rewritten-on-main). Deleting a published tag or Release is the maintainer's decision, not the agent's.

## Contents

- [Nothing Was Released](#nothing-was-released)
- [The Bump Push Was Rejected](#the-bump-push-was-rejected)
- [First Release and Missing Tags](#first-release-and-missing-tags)
- [Re-running a Release](#re-running-a-release)
- [History Rewritten on main](#history-rewritten-on-main)
- [Local main Is Stale](#local-main-is-stale)
- [First Run of a New Release Job](#first-run-of-a-new-release-job)

Placeholders: `<unit>` is a released unit (or the whole repository when it has one), `<tag-glob>` its tag pattern (`v*`, `<unit>@*`), `<sha>` a commit.

## Nothing Was Released

**Symptom.** A change merged, CI finished (or never started), and `main` has no bump commit, tag or Release for it.

**Confirm**, in this order; the first match is the cause:

1. **No run started.** `gh run list --branch main --commit <merge-sha>` is empty. Look for a skip marker in the pushed head commit's message, which includes the merge commit's body, i.e. the PR body:

   ```bash
   git log -1 --format=%B <merge-sha> | grep -iE '\[(skip ci|ci skip|no ci|skip actions|actions skip)\]|skip-checks'
   ```

   Other reasons: Actions disabled, or a trigger filter that excludes the push ([ci-release-job.md](ci-release-job.md#triggers-and-concurrency)).
2. **The run did not reach `release`.** `gh run view <run-id>` shows `gates` or `commits` failed (the release job is skipped on purpose), or the run was cancelled.
3. **No release-type commit on a released path since the last tag.**

   ```bash
   LAST=$(git -c versionsort.suffix=- tag --list '<tag-glob>' --sort=-version:refname | head -1)
   git log "$LAST"..origin/main --format='%h %s' -- <released path>
   ```

   Empty output, or only non-release types (docs, chore, test), means nothing is due, by design ([lifecycle.md](lifecycle.md#integration-lanes); what releases: `writing-history` → `references/semver-changelog.md`). Commits that touch only reports, roadmaps or docs outside a unit never release.
4. **The tool found nothing because it lost its baseline.** The last tag is not an ancestor of `main`: `git merge-base --is-ancestor "$LAST" origin/main || echo orphaned`. Go to [History Rewritten on main](#history-rewritten-on-main). A shallow checkout hides tags from the tool too: [First Release and Missing Tags](#first-release-and-missing-tags).

**Fix.**

| Cause | Fix |
|:--|:--|
| Skip marker | Start a fresh run on the current tip: `gh workflow run <workflow-file> --ref main`. Skip markers apply only to `push` and `pull_request` runs, so a manual run is not affected. Then remove the marker's source. |
| A gate failed on `main` | Fix forward through the normal lanes; the release job runs on the next push (or a manual run) and releases everything since the last tag. Do not skip or weaken the gate. |
| Cancelled run | The workflow cancels runs on `main`, which it should not: [ci-release-job.md](ci-release-job.md#triggers-and-concurrency). Fix the concurrency expression, then start a manual run. |
| Nothing due | Nothing to fix. If the change should have released, its commit type was wrong. Land a correctly typed follow-up (`writing-history` → `references/commit-authoring.md`) instead of rewriting `main`. |

**Prevention.** Never copy a bump commit's message, or a skip marker, into a PR body ([pull-requests.md](pull-requests.md#body)): the body becomes the merge commit message. See [ci-release-job.md](ci-release-job.md#the-bump-commit).

## The Bump Push Was Rejected

**Symptom.** The release job fails at the step that pushes the bump commit.

**Confirm.** Read the failing step's log:

| Message shape | Meaning |
|:--|:--|
| `! [rejected] … (fetch first)` or `non-fast-forward` | Race: another push reached `main` after the job checked out. |
| `GH006: Protected branch update failed`, followed by a reason (a required PR or a required status check) | Protection: the pushing identity may not bypass. |

For a race, compare the run's commit with the tip: `gh run view <run-id> --json headSha` against `git log origin/main --oneline -3`. Then see what the failed run left behind:

```bash
git ls-remote --tags origin '<tag-glob>'     # tags newer than the last good release?
gh release list --limit 5                    # a Release with no bump commit?
# and, if the repository publishes: does the registry already hold the new version?
```

**Fix.**

- **Race, no new tag or Release.** Nothing was published. Do not re-run the failed run (it replays the old commit): start a fresh run on `main` (`gh workflow run <workflow-file> --ref main`), or let the next push's run do it. It releases the accumulated commits together.
- **Race, a tag or Release exists.** Check whether the tag's commit is reachable from `main` (`git merge-base --is-ancestor <tag> origin/main`). Most tools tag the bump commit itself, so a tag whose commit is not on `main` means the push that carried it never landed: the maintainer deletes the tag and its Release (`gh release delete <tag> --cleanup-tag`), then a fresh run releases the same version cleanly. A tag that is reachable from `main` without a bump commit can only come from a tool that tags another commit; ask the maintainer how that tool continues. Never publish the same version twice.
- **Race, a registry already holds the version.** A registry refuses a version it has, so the fresh run fails at publish: [Re-running a Release](#re-running-a-release) covers registry-first publishes.
- **Protection.** The push identity is not allowed past the rule: give the release job an app token and put the app on a ruleset's bypass list in "always" mode ([ci-release-job.md](ci-release-job.md#permissions-tokens-and-branch-protection)). With required status checks, a ruleset bypass actor (or an admin with "Do not allow bypassing" off) is needed, because the bump commit has no checks; do not assume the classic bypass-PR list waives them. Verify on a scratch repository.

**Prevention.** Concurrency serialises runs but cannot stop a person's push, so a race is possible whenever someone lands while a release is running; the design absorbs it ([ci-release-job.md](ci-release-job.md#idempotent-re-runs)). Settle protection before the first release: [setting-up.md](setting-up.md#adoption-checklist).

## First Release and Missing Tags

**Symptom.** The first run in a repository proposes a version far from the published one (often `1.0.0`), a changelog listing the repository's whole history, or an error about a missing previous release. The same happens later when the tool "forgets" tags.

**Confirm.**

```bash
git tag --list '<tag-glob>'        # locally and, in CI, in the job's log
git describe --tags --abbrev=0 origin/main
git branch -r --contains <last-release-sha>   # is the last release inside main's history?
```

A checkout fetches one commit and no history by default, so a CI job can see no tags in a repository that has them. Fetch the full history (`fetch-depth: 0`), as in the skeleton at [ci-release-job.md](ci-release-job.md#job-graph).

**How tools treat a repository with no tags.** Tools that anchor on tags treat "no matching tag" as "never released". One such tool starts at `1.0.0` and, when adopted on an existing project, needs the last released commit tagged in its configured format. Another looks the version up in a package registry when the tag is missing, and in its git-only mode treats a package with no matching tag as an initial release. Read the tool's rule before the first run; it differs per tool ([setting-up.md](setting-up.md#choosing-tooling-from-contents)).

**Fix: seed a baseline deliberately.** Tag the commit that matches what the world already has, in the tool's exact tag format, and push only that tag:

```bash
git tag <unit>@<current-version> <sha>     # or v<current-version> for a single unit
git push origin <unit>@<current-version>
```

- Pick `<sha>` on `main`'s history. Tagging the current tip releases only what lands after adoption; tagging an older commit makes the first changelog list everything since.
- Take the version from the manifest or the registry, not a guess. A repository that never released chooses its first version on purpose (and sets it in the manifest if the tool reads it there).
- Seed one tag per unit in a monorepo.
- A pushed tag is a push: check for workflows triggered by `tags:` first, so it does not publish anything.
- Confirm with a manual run: it should report no release, or the next version you expect.

**Prevention.** The seed is an item on the adoption checklist: [setting-up.md](setting-up.md#adoption-checklist).

## Re-running a Release

**When a re-run is safe.** The release tool derives its work from tags, so a second run finds either nothing or the same commits ([ci-release-job.md](ci-release-job.md#idempotent-re-runs)), and the failure was before the effects (a runner or network failure, a registry timeout) or the failed step is repeatable.

| Situation | Do |
|:--|:--|
| Transient failure, `main` unmoved since the run's commit | `gh run rerun <run-id> --failed` |
| `main` has moved | A fresh run on the tip: `gh workflow run <workflow-file> --ref main`. A re-run replays the original commit and ref, so it would rebuild from stale history and its bump push would be rejected. |
| Re-run budget spent or the run is older than 30 days | A fresh run, for the same reason: re-runs are limited in number and age. |
| A registry already accepted the version, but the tag or Release is missing | Not a plain re-run: the registry will refuse the same version. Tag the published commit, create the Release for it by hand (maintainer), and fix the ordering that let the publish go first. |

**Verify afterwards**, every time:

```bash
git fetch --tags origin && git log origin/main --first-parent --oneline -5
git -c versionsort.suffix=- tag --list '<tag-glob>' --sort=-version:refname | head -3
gh release list --limit 3
```

- Exactly one bump commit per version, authored by the bot, with `[skip ci]`.
- One tag per version, reachable from `main` (`git merge-base --is-ancestor <tag> origin/main`).
- A Release for that tag, with notes and assets.
- The registry shows the version, if the repository publishes.
- No second run started by the bump commit.

## History Rewritten on main

**Symptom.** `main` was force-pushed or rewritten (a rebase of `main`, a filter, a reset). Afterwards: range checks fail on an unknown `before` commit; tags are not on `main`; the tool proposes an old version again, or releases everything; changelogs repeat entries.

**Why it breaks.** A push's `before..after` range assumes `before` is an ancestor of `after`. Release tools find "the last release" by walking from the tip to a tag, so a tag on a commit no longer in history is a lost baseline. At least one tool also keeps release-channel records in git notes, which are lost the same way, and the repair is to recreate the tags (and notes) on the new commits.

**Do not force-push `main`.** It is blocked on protected branches by default; if `main` is unprotected, block it with a ruleset anyway ([ci-release-job.md](ci-release-job.md#permissions-tokens-and-branch-protection)). The topic-branch rule (`--force-with-lease` after a rebase) does not extend to `main`: `writing-history` → `references/branches.md`.

**Confirm.**

```bash
git fetch origin --tags
git tag --list '<tag-glob>' | while read t; do
  git merge-base --is-ancestor "$t" origin/main || echo "orphaned: $t"
done
```

**Fix.** Stop other landings first; this is a maintainer-led repair.

1. **Prefer restoring the old tip.** If the previous tip is known (a clone that still has it, or the commit of the last good run in `gh run list --branch main`), the maintainer restores it with one pinned force push: `git push --force-with-lease=main:<bad-tip> origin <old-tip>:main`. A rule that blocks non-fast-forward pushes refuses this too, so the maintainer lifts it (or bypasses it) just for the push and restores it straight after. Tags and ranges are consistent again, and the rewritten-away work re-lands through the normal lanes.
2. **Otherwise accept the new history and re-anchor.** For each orphaned tag, find the equivalent commit in the new history (same subject or patch: `git log origin/main --grep=<subject>`, `git cherry`), then recreate the tag there (`git tag -f <tag> <new-sha>`, pushed by the maintainer after deleting the remote tag). Check each GitHub Release still points at its tag. Recreate any tool-specific records the tool's documentation names.
3. **Verify** with a fresh run on `main`: it must report no release, or exactly the version you expect, and publish no version that already exists.

**Prevention.** A rule that blocks force pushes and deletions on `main`, with no bypass for it; rebase only topic branches ([lifecycle.md](lifecycle.md#invariants)).

## Local main Is Stale

**Symptom.** A Direct-lane `git push` is rejected as non-fast-forward; a topic branch cut from local `main` conflicts in version files or the changelog; a PR shows changes to bot-managed files you never made.

**Cause.** The bump commit lands after the merge, a minute or more later, made by CI. Local `main` does not have it until you pull. The same applies to every other maintainer's landing.

**Confirm.**

```bash
git fetch origin
git status -sb                                  # "behind N"
git log HEAD..origin/main --format='%h %s'      # includes chore(release) commits
```

**Fix.**

- Local `main` has no commits of its own: `git switch main && git pull --ff-only`.
- Local `main` has commits that are not on `origin/main` (someone committed on it): save them first with `git switch -c <topic>`, then reset local `main` to `origin/main`, and re-land the topic by its lane ([lifecycle.md](lifecycle.md#integration-lanes)). Do not `git reset --hard` before the commits are on a branch.
- A topic branch cut from stale `main`: rebase it onto `origin/main` (`writing-history` → `references/branches.md`). If the rebase conflicts in versions, lock files or the changelog, take the `origin/main` side: those files are bot-managed and a topic never needs to edit them.

**Prevention.** After merging a release-type change, wait for its run to push the bump commit (`gh run watch <run-id>`) before pulling and branching again. Pull `main` after every merge, and again before cutting a branch: [lifecycle.md](lifecycle.md#invariants).

## First Run of a New Release Job

Watch the first release-type merge end to end, once, before trusting the job. Copy the list below and tick each item against the real run. Start from [setting-up.md](setting-up.md#adoption-checklist) (baseline tag seeded, protection and token settled), and pick a small release-type change: a fix in one unit, through the Self-merge or Reviewed lane.

- [ ] **The run starts.** A run appears for the merge commit's SHA (`gh run list --branch main --commit <sha>`). None: [Nothing Was Released](#nothing-was-released).
- [ ] **The graph is right.** `changes`, `gates` and `commits` pass; `release` is not skipped.
- [ ] **The bump commit lands.** On `origin/main`, subject `chore(release): … [skip ci]`, authored by the bot. `git show --stat <bump-sha>` lists only version fields, lock files and the changelog.
- [ ] **It starts nothing.** No run exists for the bump commit's SHA.
- [ ] **The tag exists** in the expected format, is reachable from `main`, and sits on the commit the tool is expected to tag. `git ls-remote --tags origin '<tag-glob>'`.
- [ ] **The Release exists**, with notes that list the commits of the merged branch (not only the merge commit) and the expected assets.
- [ ] **The changelog entry matches.** Versions in the manifest and lock files equal the tag.
- [ ] **Registry** (if the repository publishes): the version is there once.
- [ ] **Pull `main`** and check the local tree has the bump commit ([Local main Is Stale](#local-main-is-stale)).
- [ ] **Land a Direct-lane change** (a doc fix). The commit-range check must run, nothing may release, and with protection on, the maintainer's push must be accepted.
- [ ] **Learn the half-failure behaviour** while nothing is at stake: read the tool's documentation for what it does when the push step fails, so the "tag or Release exists?" question in [The Bump Push Was Rejected](#the-bump-push-was-rejected) is not new during an incident.

If any item fails, fix it before the next release-type merge; a failed first run is cheaper than a wrong tag on a real release.
