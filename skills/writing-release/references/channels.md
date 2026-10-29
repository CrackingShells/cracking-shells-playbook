# Release Channels

Why a repository releases from one branch by default, which observable signs justify a second channel, and how to add one without building a second integration branch. Read it before anyone proposes a `dev`, `next` or `beta` branch, or a prerelease series. Tool choice and tag formats are in [setting-up.md](setting-up.md#choosing-tooling-from-contents); the CI job that runs the tool is in [ci-release-job.md](ci-release-job.md).

## Contents

- [One Channel by Default](#one-channel-by-default)
- [When to Add a Channel](#when-to-add-a-channel)
- [The Lagged Release-Candidate Pattern](#the-lagged-release-candidate-pattern)
- [Prereleases per Tool](#prereleases-per-tool)
- [The Cost of a Long-Lived Development Branch](#the-cost-of-a-long-lived-development-branch)

## One Channel by Default

A channel is a stream of versions that consumers opt into: a prerelease tag series, a registry dist-tag, or a branch that releases separately. The default is a single channel: `main` is what gets released, and a release-type merge to `main` releases it ([lifecycle.md](lifecycle.md#the-lifecycle)).

Why one channel is the right default:

- **One integration target.** Every change lands once, in one place, and the history of what shipped is the history of `main`.
- **No drift between branches.** There is no second branch to fall behind, to be back-merged into, or to disagree about what the latest state is.
- **Every merge is shippable.** CI gates the merge and the lane decides whether a maintainer reviews it ([lifecycle.md](lifecycle.md#integration-lanes)), so what is on `main` is already what you would release.
- **Nothing to promote.** No promotion step means no promotion failures, no "which commit was blessed" question and one tag line per unit.
- **Recovery is plain.** A bad release is fixed forward by the next merge, or reverted by a merge; both use the normal path.

What one channel gives up: a release per release-type merge, and no batching. Five small fixes make five releases, and a merge's release cannot be held back. Batch by landing related work on a campaign branch and merging it once, or use the lagged pattern below.

A consumer who wants unreleased work installs from a commit or a topic branch. That is not a reason for a channel.

## When to Add a Channel

Add a channel only on observed evidence, and only with the maintainer's approval: it changes release machinery, so it travels the reviewed lane ([lifecycle.md](lifecycle.md#choosing-a-lane)).

| Trigger | Observable signal | Why a channel helps |
|:--|:--|:--|
| Regressions reach many users before a fix | Issue reports arrive faster than a fix can ship; several releases in a row each needed a follow-up | A soak period lets early adopters find the regression first |
| Downstream consumers pin versions and ask for a soak | Consumers say they pin to tested versions, or request a candidate to validate against | They get something to test without taking untested stable releases |
| Repeated hotfix churn after releases | A pattern of `fix` releases within days of a `feat` release | Time-delayed promotion absorbs the churn before stable |
| Maintaining an older major line (backports) | Users stay on a major version and need fixes after a newer major ships | A maintenance branch per supported line, released from that branch (the tool must support it; `semantic-release` does). A different case from a dev channel: the line takes backported fixes only, never new work |

Not a reason for a channel:

- **A place to park unfinished work.** That belongs on topic branches; the unit of integration is a finished change.
- **A wish to review later.** Review belongs in the lane that applies to the change, before the merge.
- **Many contributors.** More contributors need more CI and clearer lanes, not a second branch.
- **A big feature in flight.** Merge it behind a flag, or in pieces that are each safe to release.

Cheaper first: smaller releases, a faster revert path, and tighter CI. If a trigger persists after those, add the lagged pattern below rather than a second branch, and record it in the CONTRIBUTING Versioning section ([setting-up.md](setting-up.md#the-contributing-section)).

## The Lagged Release-Candidate Pattern

`main` still releases on every release-type merge, but the version is a release candidate; a scheduled job promotes a candidate to stable once it has soaked.

```
merge to main -> release job -> X.Y.Z-rc.N          (prerelease tag, prerelease GitHub Release)
                                   |
        scheduled promotion job: newest rc at least <soak> days old, no blocking defect
                                   v
                                X.Y.Z               (stable tag and GitHub Release, on the soaked commit or a bump commit after it)
```

A sketch with example numbers: promote weekly, with a 14-day soak. Choose both from the consumers' pace, not from convention.

What it needs:

- **Prerelease support in the tool**, and a way to turn a soaked rc into the stable version ([Prereleases per Tool](#prereleases-per-tool)). Verify both with the tool's dry-run mode before relying on them.
- **A promotion that ships exactly the soaked code.** Only tag-only versioning, where no version file is committed, lets the promotion tag the rc commit itself. A tool that writes versions into files makes a new bump commit and tag after the rc, and on `main` that commit sits on top of everything merged since, including unsoaked work. Either run the promotion from a pointer branch positioned at the soaked commit ([The Cost of a Long-Lived Development Branch](#the-cost-of-a-long-lived-development-branch)), or promote only when the newest rc is the soaked one.
- **A version-order rule.** `X.Y.Z` outranks every `X.Y.Z-rc.N`, so after a promotion the next cut is the next version's first rc (`X.Y.(Z+1)-rc.1` or whatever the next bump calculates). Release candidates of `X.Y.Z` newer than the promoted one are superseded and never promoted under that number.
- **A promotion job**: a scheduled workflow plus a manual trigger. It selects the newest rc old enough, skips it if a blocking defect is recorded against it, and releases the stable version. Like any release job, it must be safe to re-run ([ci-release-job.md](ci-release-job.md#idempotent-re-runs)); a failed promotion is handled in [recovery.md](recovery.md#re-running-a-release).
- **A hotfix policy during the soak.** A fix merged to `main` produces a newer rc with its own clock. Write down two rules: an rc with a blocking defect is never promoted (record it with a label or a deny list the job reads, and fix or revert on `main`), and an urgent fix may be promoted early by a manual trigger the maintainer approves.
- **Consumer guidance.** Stable consumers follow stable tags; rc consumers opt in explicitly. Say which in the README.
- **Registry fit.** If the repository publishes, check that the registry can hold prerelease versions that stable users do not receive by default. If not, publish stable only and keep the rc as tag and GitHub Release.

This is a sketch to size the idea, not a build instruction; the promotion job is repository-specific.

## Prereleases per Tool

How each tool in [the matrix](setting-up.md#choosing-tooling-from-contents) expresses a prerelease and returns to stable. Confirm the exact flow with the tool's dry-run mode in a scratch clone before the first real run.

| Tool | Prerelease | Back to stable | Caveats |
|:--|:--|:--|:--|
| `semantic-release` | A property of a branch entry: `{ name: "beta", prerelease: true }` uses the branch name as the identifier, a string value such as `"rc"` sets it. Prereleases publish to a channel (npm dist-tag) of that name, not the default one | Merge the prerelease branch into the stable branch; the next run releases the stable version | Prerelease is tied to a branch, so the lagged pattern needs a pointer branch for stable. The tool reads the branch from the CI ref, and scheduled workflows run on the default branch, so the schedule dispatches the stable run with `--ref stable`. A bump commit from `@semantic-release/git` would diverge the pointer: do not commit on it, or reset it. Verify with a dry run first |
| `multi-semantic-release` | Inherits the branch configuration of `semantic-release` | As `semantic-release` | Verify per-package prerelease behaviour with a dry run before relying on it |
| `python-semantic-release` | A branch group with `prerelease = true` and `prerelease_token = "rc"`, or flags on any run: `version --as-prerelease --prerelease-token rc`. Produces `X.Y.Z-rc.N` | Run `version` without the prerelease flag; the next version is computed from the latest full release plus the commits | Preview with `version --print`. The `--prerelease` flag is a forced bump that increments the revision, which differs from `--as-prerelease` |
| `commitizen` | `cz bump --prerelease rc` (`alpha`, `beta` or `rc`) | A plain `cz bump` | Spelling follows `version_scheme`: PEP 440 by default (`1.1.0rc0`); `version_scheme = "semver2"` gives `1.1.0-rc.0` |
| `release-plz` | No prerelease command or channel. The version in `Cargo.toml` is the source; `release-plz set-version <version>` sets one. Its version calculation increments the last numeric identifier of an existing prerelease. `git_release_type = "auto"` marks the GitHub Release as a prerelease when the version has a prerelease part | `release-plz set-version` to the stable version | Verify the automatic rc counter with a dry run |
| `release-please` | `"versioning": "prerelease"` with `prerelease-type`, and `"prerelease": true` to mark GitHub Releases as prereleases | Release with the prerelease setting off, or `release-as` the stable version | Works through a release PR, outside the default pipeline. `"prerelease": true` also marks pre-1.0 stable versions as GitHub prereleases, and `release-as` stays in force until removed or raised after its PR merges |
| `cocogitto` | `cog bump --pre "<identifier>.*"` sets or increments a prerelease; for example `cog bump --major --pre "beta.*"` gives `2.0.0-beta.1` | `cog bump --auto` without `--pre` | Confirm the combination of `--auto` and `--pre` with `--dry-run` |

Whatever the tool, the tag shape must keep prereleases distinguishable from stable tags (`<unit>@<version>-rc.<n>`; `commitizen` needs `version_scheme = "semver2"` for that spelling), so the "last release" lookup that decides the next version can tell them apart. See the tag-format table in [setting-up.md](setting-up.md#choosing-tooling-from-contents).

Sorting caveat: a plain version sort ranks `1.0.0-rc.2` above `1.0.0`; any job that picks the latest release by sorting tags must rank prereleases below their stable version ([ci-release-job.md](ci-release-job.md#monorepos) has the command).

## The Cost of a Long-Lived Development Branch

A long-lived `dev` branch is the usual way a second channel is built, and the pattern it creates is a reason to prefer the lagged pattern above.

- **Double integration.** Every change lands twice: topic branch to `dev`, then `dev` to `main`. Conflicts are resolved twice, reviews happen twice or are skipped once, and the PR and merge-commit conventions have to be applied on both hops.
- **A drifting branch.** Hotfixes land on `main` and must be merged back, and anything that rewrites `dev` history strands the tags already cut from it ([recovery.md](recovery.md#history-rewritten-on-main)). `dev` ends up ahead of, behind or diverged from `main`, and "what is released" has no single answer.
- **A backlog that never releases.** Work accumulates on `dev` while `main` waits for a promotion nobody schedules. The eventual `dev`-to-`main` merge is a large, untested-as-a-whole release, the opposite of what the soak was meant to deliver.
- **Prerelease tags nobody consumes.** Every merge to `dev` cuts a prerelease no one installs, so no feedback returns and the channel adds cost without lowering risk.
- **Config that depends on branch topology.** Branch-based prerelease tools need the branch history to line up with their tags; a repository that merges and rebases loosely breaks their version calculation.

If a branch-based tool forces a second branch, keep it a pointer: a `stable` branch that only the promotion job advances to a soaked commit, and that never receives topic branches or direct pushes. That keeps one integration target, `main`, and one place where work lands. The run must use the `stable` ref and must not commit to it; the semantic-release row in [Prereleases per Tool](#prereleases-per-tool) lists the caveats, and a dry run comes first.
