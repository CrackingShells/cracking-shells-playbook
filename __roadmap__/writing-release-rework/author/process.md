# Process: Lifecycle, Pull Requests, Dispatcher

**Goal**: Write the integration-lane and lifecycle core (`lifecycle.md`), the PR guide (`pull-requests.md`), and the new `SKILL.md` dispatcher, then retire the four Hatch-era references in the same commit as the dispatcher.
**Pre-conditions**:
- [ ] The campaign README contract (C1–C7) is read in full; the lane table C3 is used verbatim
- [ ] The four current references exist under `skills/writing-release/references/` (sources for step 2)
**Success Gates**:
- ⬜ `lifecycle.md` and `pull-requests.md` carry exactly the C1 headings, are ≤350 lines, and contain no C5 residue
- ⬜ `SKILL.md` passes `uv run --with pyyaml tools/quick_validate.py skills/writing-release`, is ≤500 lines, and its Navigation table names all six C1 references
- ⬜ `git ls-files skills/writing-release/references` lists none of `pr-fork-to-dev.md`, `pr-dev-to-main.md`, `discord-dev-release.md`, `discord-main-release.md`
- ⬜ Exactly one commit on the branch carries `BREAKING CHANGE:`, and it is step 3's
**References**: [R01 §Cross-repo survey](../../../__reports__/writing-release-rework/00-architecture_analysis_v0.md) — the patterns behind the lanes and invariants (source material only, C5)

## Step 1: Lifecycle and integration lanes
**Goal**: State the lanes, how to choose one, and the end-to-end lifecycle with each invariant's reason, as the owner of those facts (C2).
**Implementation Logic**:
1. `## Integration Lanes`: what a PR buys (CI before `main`, a review point), and why merging to `main` being a release makes the release boundary the line. Then the C3 table verbatim.
2. `## Choosing a Lane`: the C3 decision procedure as a numbered checklist, plus why a wrong lane is cheap.
3. `## Mapping Lanes in a Repository`: how a repo derives its released paths and release-type commits from its own release config (machinery → docs → interview), plus a template for a CONTRIBUTING "Integration lanes" table with placeholder rows.
4. `## The Lifecycle`: topic branch `<type>/<topic>` → push → PR (or direct) → CI gates (tests, commit-range check, PR-title check) → rebase onto `main` + `git push --force-with-lease` → merge commit → CI release job → pull `main`. Include a mermaid or ASCII flow covering both PR lanes and the direct lane.
5. `## Invariants`: each C2 invariant with its WHY. Squash merging loses the per-commit history the changelog is generated from, and rebase-merging loses the merge record. The bump commit carries `[skip ci]` (detail lives in `ci-release-job.md`). There are no release PRs and no local release commands, because one release ceremony is easier to sustain than several.
6. `## Where Each Piece Lives`: CONTRIBUTING sections, the CI workflow file, the release config, and the commit-check config, each as a generic role rather than a filename from a specific repo.
Link to `pull-requests.md`, `ci-release-job.md` and `channels.md` for the details they own. Reference the `writing-history` skill by name for branch-to-parent integration (C2).
**Deliverables**: `skills/writing-release/references/lifecycle.md` — sections `Contents`, `Integration Lanes`, `Choosing a Lane`, `Mapping Lanes in a Repository`, `The Lifecycle`, `Invariants`, `Where Each Piece Lives`; the C3 lane table; a CONTRIBUTING lanes template table
**Consistency Checks**: `python3 -c "import re,sys;p=sys.argv[1];t=open(p).read();h=set(re.findall(r'^## (.+?)\s*$',t,re.M));m=[x for x in sys.argv[2:] if x not in h];n=t.count(chr(10));print(p,'missing:',m,'lines:',n);sys.exit(1 if m or n>350 else 0)" skills/writing-release/references/lifecycle.md "Contents" "Integration Lanes" "Choosing a Lane" "Mapping Lanes in a Repository" "The Lifecycle" "Invariants" "Where Each Piece Lives" && ! grep -nE '[Cc]andler|[Hh]atch|colgrep|[Pp]inion|[Pp]lumage|[Mm]urmuration|\bNest\b|py-repo-template|[Dd]iscord|[Rr]ework|[Pp]reviously|[Aa]s decided|old template' skills/writing-release/references/lifecycle.md` (expected: PASS)
**Commit**: `feat(writing-release): define integration lanes and the release lifecycle`

## Step 2: Pull requests as future merge commits
**Goal**: Guide opening, titling, writing and landing a PR whose title and body become the merge commit, generalising what is durable in the two `pr-*` sources.
**Implementation Logic**:
1. Read `references/pr-fork-to-dev.md` and `references/pr-dev-to-main.md`. Keep only what generalises: a summary first, a root cause for each fix, verification evidence, and cross-references to constituent work. Drop the fork, `dev` and version-in-title framing.
2. `## Opening a Pull Request`: when (Reviewed or Self-merge lane, linking to `lifecycle.md#integration-lanes`), from a pushed topic branch, `gh pr create` with the title and body taken from files.
3. `## Title`: the title becomes the merge subject `merge(<scope>): <what lands and why>`, in the repo's own vocabulary (the `writing-history` skill, `references/commit-authoring.md`). CI checks it, including when the title is edited.
4. `## Body`: the body becomes the merge body. Give a fill-in template: why → what lands → verification → follow-ups, ending with the `Co-Authored-By` trailer for agent-authored PRs. No separate PR-type templates.
5. `## Landing`: `git fetch` + `git rebase origin/main` + `git push --force-with-lease`; wait for green; `gh pr merge --merge` (never `--squash` or `--rebase`); what to check before merging.
6. `## Self-Merge and Its Fallback`: the C3 fallback, and why retrying or working around a block is wrong.
7. `## After the Merge`: pull `main` (the release job may have added a bump commit), delete the local branch, confirm the release from the release notes rather than from the echoed output.
**Deliverables**: `skills/writing-release/references/pull-requests.md` — sections `Opening a Pull Request`, `Title`, `Body`, `Landing`, `Self-Merge and Its Fallback`, `After the Merge` (plus `Contents` if over ~100 lines); a PR body template block
**Consistency Checks**: `python3 -c "import re,sys;p=sys.argv[1];t=open(p).read();h=set(re.findall(r'^## (.+?)\s*$',t,re.M));m=[x for x in sys.argv[2:] if x not in h];n=t.count(chr(10));print(p,'missing:',m,'lines:',n);sys.exit(1 if m or n>350 else 0)" skills/writing-release/references/pull-requests.md "Opening a Pull Request" "Title" "Body" "Landing" "Self-Merge and Its Fallback" "After the Merge" && ! grep -nE '[Cc]andler|[Hh]atch|colgrep|[Pp]inion|[Pp]lumage|[Mm]urmuration|\bNest\b|py-repo-template|[Dd]iscord|[Rr]ework|[Pp]reviously|[Aa]s decided|old template' skills/writing-release/references/pull-requests.md` (expected: PASS)
**Commit**: `feat(writing-release): write pull requests as future merge commits`

## Step 3: Dispatcher and retirement of the Hatch-era references
**Goal**: Replace `SKILL.md` with the release-cycle dispatcher and delete the four Hatch-era references, in one breaking commit.
**Implementation Logic**:
1. Frontmatter: `name: writing-release`, and a third-person `description` (≤1024 chars) saying what the skill does and when to use it. Triggers: opening, titling or landing a PR to `main`; deciding whether a change needs a PR; setting up releases or a CI release job; choosing release tooling; "why didn't it release" or "the release job failed"; adding a prerelease, dev or rc channel; "cut a release".
2. `# Writing Release` → `## Primary Directives`, one short paragraph each, saying WHY: (a) `main` is the only channel, and a change reaches it by the lightest lane that still gates what it risks; (b) a release is a consequence of merging, not a ceremony; (c) release tooling is derived from what the repo contains, never assumed; (d) composition with the `writing-history` skill by name.
3. `## Navigation`: a situation → reference table covering all six C1 references. Then "read the routed reference before acting".
4. `git rm` the four files listed in the Success Gates.
5. The commit message body explains the change of purpose and ends with the footer `BREAKING CHANGE: writing-release no longer drafts fork-to-dev PR bodies or Discord announcements; it is the release-cycle manual (integration lanes, CI releases, tooling, channels, recovery).`
The Navigation links to `setting-up.md`, `ci-release-job.md`, `channels.md` and `recovery.md` resolve only after the sibling leaves merge. Do not create stubs for them; the coordinator's integration review checks them.
**Deliverables**: `skills/writing-release/SKILL.md` — frontmatter `name`, `description`; sections `Primary Directives` (4 directives), `Navigation` (6 rows); deletions of the four Hatch-era reference files
**Consistency Checks**: `uv run --with pyyaml tools/quick_validate.py skills/writing-release && test $(wc -l < skills/writing-release/SKILL.md) -le 500 && test -z "$(git ls-files skills/writing-release/references/pr-fork-to-dev.md skills/writing-release/references/pr-dev-to-main.md skills/writing-release/references/discord-dev-release.md skills/writing-release/references/discord-main-release.md)" && uv run tools/package_skill.py skills/writing-release dist/` (expected: PASS)
**Commit**: `feat(writing-release)!: turn the skill into the release-cycle manual`
