# Integration Lifecycle

How a change travels from a topic branch to `main`, and how a release follows from it. This file owns the lane table, the decision procedure and the invariants; the other references link here rather than restate them.

## Contents

- [Integration Lanes](#integration-lanes)
- [Choosing a Lane](#choosing-a-lane)
- [Mapping Lanes in a Repository](#mapping-lanes-in-a-repository)
- [The Lifecycle](#the-lifecycle)
- [Invariants](#invariants)
- [Where Each Piece Lives](#where-each-piece-lives)

## Integration Lanes

Merging to `main` is what releases: the release job runs on every push to `main` and cuts a release when the new commits warrant one. So the line to draw is not "code versus docs" but **what the change would release**.

A pull request (PR) buys two things: CI runs on the change before it reaches `main`, and a maintainer gets a point to review. Both are worth their latency only when the change risks something. A change that cannot release anything and cannot break how the repo builds, verifies or releases gains nothing from either, so it takes a lighter lane.

Two terms the lanes rely on, defined per repository in [Mapping Lanes in a Repository](#mapping-lanes-in-a-repository):

- **Released path**: a path whose change ships in a release (source, packaged assets, a published unit's files).
- **Release-type commit**: a commit whose type the release config maps to a version bump.

| Lane | When | How | Maintainer approval |
|:--|:--|:--|:--|
| Reviewed PR | Lands a roadmap campaign or a multi-leaf feature branch; carries a `BREAKING CHANGE`; changes CI, release machinery or repository settings; changes a persisted contract (schemas, public interfaces others pin); or the agent is uncertain | Topic branch → PR → CI green → maintainer approves → merge commit | Yes |
| Self-merge PR | Would release something (a release-type commit touching a released path) and is self-contained: a fix, a small feature in one unit, a dependency bump | Topic branch → PR → CI green → the agent rebases, merges with a merge commit, reports the link. If the harness or a permission classifier blocks the merge, the agent does not retry or work around it: it hands the green PR to the maintainer and says why | No (a blocked self-merge falls back to the maintainer) |
| Direct | Releases nothing and changes nothing about how the repo builds, verifies or releases: reports, roadmap nodes and status, docs outside released units, typo or link fixes, regenerated artifacts | `writing-history` branch-to-parent integration if branched → pull → push to `main` | No |

The direct lane needs `main` unprotected, or protected with a bypass for maintainers and the release bot; [ci-release-job.md](ci-release-job.md#permissions-tokens-and-branch-protection) owns that tradeoff. Branch-to-parent integration for a branched direct change belongs to the `writing-history` skill (`references/branches.md`), not to this manual.

## Choosing a Lane

First match wins:

1. Does any Reviewed-PR condition hold (campaign or multi-leaf landing, `BREAKING CHANGE`, CI, release machinery or settings, a persisted contract, or doubt)? Reviewed PR.
2. Otherwise, would merging release something (a release-type commit touching a released path)? Self-merge PR.
3. Otherwise: Direct.

**Doubt is a Reviewed-PR condition.** When two lanes both seem plausible, take the heavier one.

**A wrong lane is cheap.** CI still releases a direct push that turns out to be release-type, and the commit-range check runs on every push to `main`, so a misjudged change is caught and released by the same machinery that handles every other one. The costs are asymmetric: an over-cautious lane costs only latency, an under-cautious one costs a release nobody reviewed. Hence the tie-break above.

For the PR lanes, [pull-requests.md](pull-requests.md) covers opening, titling, landing and the self-merge fallback.

## Mapping Lanes in a Repository

The lane table is generic; each repository states which paths and commits it means. Derive them with the same precedence the `writing-history` skill uses for commit vocabulary (`references/commit-authoring.md`, "Deriving the Vocabulary"):

1. **Machinery is ground truth.** Read the release config and the CI release workflow:
   - the release tool's commit-analyzer or type-to-bump mapping gives the release-type commits;
   - per-unit manifests, package directories, the workflow's path filters and the tool's file globs give the released paths;
   - workflow files, the release config and settings-as-code are the machinery whose change forces a Reviewed PR.
2. **Docs next.** Where there is no machinery, or it is silent, read the contributing guide and README for what ships.
3. **Interview last.** Ask the maintainer which paths ship, which commit types release, and which files are persisted contracts others pin. Write the answers down so the next agent derives them instead of asking again.

Record the result in the repository's contributing guide as an "Integration lanes" section ([setting-up.md](setting-up.md#the-contributing-section) covers the rest of that section). Template:

```markdown
## Integration lanes

Released paths: `<path-glob>`, `<unit>/**`
Release-type commits: `<type>` (<bump>), `<type>` (<bump>), any `BREAKING CHANGE` footer (<bump>)

| Lane | This repository |
|:--|:--|
| Reviewed PR | Campaign or multi-leaf landings; `BREAKING CHANGE`; `<ci-workflow-path>`, `<release-config-path>`, repository settings; persisted contracts at `<schema-or-interface-path>`; doubt |
| Self-merge PR | A release-type commit touching a released path, self-contained: a fix, a small feature in one `<unit>`, a dependency bump |
| Direct | `<reports-dir>`, `<roadmap-dir>`, `<docs-dir>` outside released units, typo and link fixes, regenerated artifacts |
```

Keep the template's wording in step with the table above; thresholds are not tuned per repository, only the paths and types they refer to.

## The Lifecycle

```mermaid
flowchart TD
    A["topic branch &lt;type&gt;/&lt;topic&gt;"] --> B{Choose a lane}
    B -->|Reviewed PR or Self-merge PR| C[Push and open PR]
    C --> D["CI gates: tests, commit-range check, PR-title check"]
    D --> E["Rebase onto origin/main, push --force-with-lease, CI green on the new head"]
    E -->|Reviewed PR| F[Maintainer approves]
    E -->|Self-merge PR| G[Agent merges]
    F --> H[Merge commit]
    G --> H
    B -->|Direct| I["Branch-to-parent integration (writing-history), pull, push to main"]
    H --> J["CI release job on push to main"]
    I --> J
    J --> K["Bump commit, tag, GitHub Release, only if the commits warrant it"]
    K --> L[Pull main]
```

Checklist:

1. Branch from an up-to-date `main` as `<type>/<topic>`; author commits per the `writing-history` skill.
2. Choose the lane ([Choosing a Lane](#choosing-a-lane)).
3. PR lanes: push and open the PR ([pull-requests.md](pull-requests.md#opening-a-pull-request)). Direct lane: integrate per `writing-history`, pull, push.
4. CI gates run on the PR: tests, the commit-range check, and the PR-title check, which re-runs when the title is edited.
5. Rebase onto `origin/main` and `git push --force-with-lease` whenever `main` has moved. The head that merges must be the head CI went green on.
6. Merge with a merge commit: after maintainer approval (Reviewed PR) or by the agent (Self-merge PR).
7. The release job runs on the push to `main`. With no release-type commit in range it finds nothing to release, which is correct ([recovery.md](recovery.md#nothing-was-released)). Triggers, job graph and permissions are in [ci-release-job.md](ci-release-job.md).
8. Pull `main`; the release job may have added a bump commit.

Releases happen on `main` only; adding a second channel is a deliberate decision with a cost ([channels.md](channels.md#one-channel-by-default)).

## Invariants

These hold in every lane that has the thing they govern. Each is here with its reason, so that an exception can be judged rather than guessed.

| Invariant | Why |
|:--|:--|
| Rebase before merge | CI's verdict then describes what `main` will contain, and the branch's commits read linearly under a single merge commit. Mechanics and rationale for branch-to-parent integration are in the `writing-history` skill (`references/branches.md`). A branch that holds merge commits needs `git rebase --rebase-merges`, or the rebase flattens them. |
| Merge commit only; never squash, never rebase-merge | The release tool and the changelog read one commit at a time, so a squash replaces several commits (a `fix` and a `feat`, say) with one, and the bump and the changelog lose entries. A rebase-merge leaves no merge record, which is the one boundary that names the unit of work and lets it be reverted whole. |
| PR title = merge subject | The merge subject starts with the PR title (the host may append ` (#<n>)`, which the commit check must accept). The merge commit lands on `main` and is read by the same tooling as any commit, so its subject must satisfy the repo's commit vocabulary. CI checks the title for that reason. |
| PR body = merge body | The reasoning survives in `git log`, not only on a hosting page that can change or vanish. |
| Pull `main` after merge | The release job may have pushed a bump commit; a stale local `main` makes the next push fail or the next branch start behind ([recovery.md](recovery.md#local-main-is-stale)). |
| Release only from CI; no release PRs; no local release commands | One release ceremony is easier to sustain than several, and CI alone has the gates, the credentials and a clean checkout. A local release skips the gates, depends on one machine, and can race the job. The bump commit CI makes carries `[skip ci]` so it does not retrigger CI ([ci-release-job.md](ci-release-job.md#the-bump-commit)). |

The first four govern PR lanes; the direct lane uses `writing-history`'s `--no-ff` integration in their place. The last two hold in every lane.

## Where Each Piece Lives

Each piece is a role; the file name varies by repository and tooling.

| Piece | Holds | Details |
|:--|:--|:--|
| Contributing guide | The "Integration lanes" section, the PR conventions, the commit vocabulary pointer | [setting-up.md](setting-up.md#the-contributing-section) |
| CI workflow | Gates, commit-range check, PR-title check, the release job | [ci-release-job.md](ci-release-job.md) |
| Release config | Type-to-bump mapping, tag format, changelog, channels | [setting-up.md](setting-up.md#choosing-tooling-from-contents), [channels.md](channels.md) |
| Commit-check config | Allowed types and scopes, enforced by the commit-range and title checks | the `writing-history` skill (`references/convention-setup.md`) |
| Repository settings | Allowed merge methods, merge-message defaults, branch protection, branch deletion on merge | [setting-up.md](setting-up.md#repository-settings) |
| Version and changelog files | Written by the release job; contributors do not edit them | [ci-release-job.md](ci-release-job.md#the-bump-commit) |
