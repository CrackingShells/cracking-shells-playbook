# Pull Requests

A pull request (PR) in this manual is a future merge commit: its title becomes the merge subject and its body becomes the merge body. Write both for the permanent record in `git log`, not for the PR page alone.

## Contents

- [Opening a Pull Request](#opening-a-pull-request)
- [Title](#title)
- [Body](#body)
- [Landing](#landing)
- [Self-Merge and Its Fallback](#self-merge-and-its-fallback)
- [After the Merge](#after-the-merge)

## Opening a Pull Request

Open a PR only in the Reviewed PR and Self-merge PR lanes ([lifecycle.md](lifecycle.md#integration-lanes)); a direct change has no PR.

Before opening:

- [ ] The topic branch `<type>/<topic>` is pushed and its commits follow the repo's vocabulary (the `writing-history` skill, `references/commit-authoring.md`).
- [ ] Tests and the repo's local checks pass.
- [ ] The branch is rebased on `origin/main` ([Landing](#landing)).
- [ ] Title and body are drafted in files outside the working tree, so nothing is committed by accident and both can be reused when the PR is edited.

```bash
gh pr create --base main --head <type>/<topic> \
  --title "$(cat <scratch>/pr-title.txt)" --body-file <scratch>/pr-body.md
```

Revise later with `gh pr edit <pr> --title "$(cat <scratch>/pr-title.txt)" --body-file <scratch>/pr-body.md`. Editing the title re-runs the title check.

## Title

The title becomes the merge subject: `merge(<scope>): <what lands and why>`.

- **Vocabulary is the repo's own.** The `merge` type, the scope names and the length limit come from the repo's commit convention, derived as `writing-history` describes (`references/commit-authoring.md`, "Deriving the Vocabulary"). If the repo's vocabulary names merge commits differently, use its name.
- **Scope names the unit landing**, not the mechanism: `<unit>` or `<topic>`.
- **Say why, not only what.** The subject is what a reader scanning `git log --merges` sees.
- **Leave out version numbers.** The release job assigns the version; a version in the title is a guess that can be wrong.

| Weak | Better |
|:--|:--|
| `Update auth` | `merge(<unit>): refresh tokens before expiry so long sessions stay signed in` |
| `merge(<unit>): changes` | `merge(<unit>): add the <provider> login provider behind the shared interface` |

CI checks the title against the commit vocabulary, and again when it is edited ([ci-release-job.md](ci-release-job.md#triggers-and-concurrency)).

## Body

The body becomes the merge body. Lead with the reason, then what lands, then the evidence, then what is left. One template serves every PR; shrink it for a small change rather than switching templates. Omit a section that has nothing to say.

```markdown
<Why: the problem or need in one to three sentences. For a fix, state the root cause, not only the symptom.>

What lands:
- <change, one sentence; name new commands, fields or interfaces in code font>
- <change> (#<n>)   <- cross-reference constituent PRs or leaf work when this landing aggregates them
- <breaking change and its migration path, if any>

Verification:
- <command run and its result; tests added; manual check performed>

Follow-ups:
- <deferred work, known gaps, a decision the maintainer must make>

Co-Authored-By: <agent name and version> <noreply@example.com>
```

Guidance:

- **Plain labels, no `#` headings**, so the body reads the same in `git log` and on the PR page.
- **Verification is evidence**, not "tested": the check that ran and what it showed. A reviewer cannot approve what the body does not show.
- **A breaking change is declared in the commits' own footers** (the `writing-history` skill, `references/commit-authoring.md`, "Body and Footer Guidance"); the body repeats it so the reviewer sees it, with the migration path.
- **The trailer goes last, after a blank line**, so it parses as a trailer. Use it for agent-authored PRs, with the attribution line the session gives you.

## Landing

1. **Rebase onto the current `main`.**
   ```bash
   git fetch origin
   git rebase origin/main        # add --rebase-merges if the branch holds merge commits
   git push --force-with-lease
   ```
   Resolve conflicts commit by commit and re-verify afterwards; `writing-history` (`references/branches.md`) owns that contract. Force-pushing is acceptable here because the topic branch is yours alone; never force-push `main` or a branch others build on.
2. **Wait for green on the rebased head.** `gh pr checks <pr> --watch`. A green run on an older head says nothing about the one you will merge.
3. **Check before merging:**
   - [ ] Every required check is green on the current head.
   - [ ] Title and body still describe what the branch now contains, because they become the merge message.
   - [ ] Reviewed PR lane: the maintainer has approved (`gh pr view <pr> --json reviewDecision`). Green CI alone is not approval.
   - [ ] No review thread is left unresolved.
4. **Merge with a merge commit.**
   ```bash
   gh pr merge <pr> --merge
   ```
   Never `--squash` or `--rebase` ([lifecycle.md](lifecycle.md#invariants) gives the reasons), and never `--admin` to bypass a failing check. If the merge message does not come out as the title and body, the repository merge settings are off ([setting-up.md](setting-up.md#repository-settings)); pass them explicitly with `--subject "$(cat <scratch>/pr-title.txt)" --body-file <scratch>/pr-body.md`.

In the Reviewed PR lane, the merge follows the maintainer's approval: the maintainer merges, or the agent does once told to.

## Self-Merge and Its Fallback

In the Self-merge PR lane the agent rebases, waits for green, merges with a merge commit and reports the PR link ([Landing](#landing)).

If the harness or a permission classifier blocks the merge, **stop**. Do not retry it, and do not route around it: no other merge command or API call, no `--admin`, no direct push to `main`, no change to settings or branch protection. The lane is a delegation from the maintainer, and a block means that delegation does not cover this case right now. Working around it turns a control into an obstacle and spends trust the lane depends on.

Hand the PR over instead, in one message:

- the PR link, with CI green on the current head;
- what was blocked and the error as reported;
- what the maintainer needs to do: merge with a merge commit.

A blocked self-merge costs one maintainer click; a worked-around block costs the lane.

## After the Merge

1. **Pull `main`.** `git checkout main && git pull`. The release job may add a bump commit after the merge, so pull again once it finishes.
2. **Delete the topic branch.** `git branch -d <type>/<topic>`; the remote copy is deleted on merge when the repo is set to do so ([setting-up.md](setting-up.md#repository-settings)), otherwise `git push origin --delete <type>/<topic>`. If it lived in a worktree, `writing-history` (`references/branches.md`) covers cleanup.
3. **Confirm the release when the merge should have released something.** Watch the job (`gh run list --branch main --limit 3`), then check what exists, not what the job printed:
   - the bump commit on `main`;
   - the tag (`git fetch --tags && git tag --list --sort=-v:refname | head -3`);
   - the GitHub Release (`gh release view <tag>`): its notes list what the release contains.

   A job can print success for a step that failed later, so the tag and the Release are the evidence.
4. **If nothing was released**, check whether the merged commits included a release-type commit; if none did, that is correct. Otherwise, or if the job failed, go to [recovery.md](recovery.md#nothing-was-released).
