# Tooling: Setting Up and Channels

**Goal**: Write how a repo adopts the lifecycle, choosing release tooling from what it contains (`setting-up.md`), and why one channel is the default and when to add another (`channels.md`).
**Pre-conditions**:
- [ ] The campaign README contract (C1–C7) is read in full
- [ ] Context7 (`resolve-library-id` / `query-docs`) or official documentation is reachable for semantic-release, multi-semantic-release, python-semantic-release, commitizen, release-plz, cocogitto and release-please
**Success Gates**:
- ⬜ Both files carry exactly the C1 headings, are ≤350 lines, and contain no C5 residue
- ⬜ Every tool claim in the matrix (what it computes, whether it writes the changelog, tags, creates GitHub Releases, publishes, handles multi-unit repos, prerelease support) cites the official doc it was checked against, in the leaf report (not in the file)
- ⬜ V1 (fact verifier) has reviewed the branch, and its findings are resolved before merge
**References**: [R01 §Cross-repo survey](../../../__reports__/writing-release-rework/00-architecture_analysis_v0.md) — which contents map to which tools in practice (source material only, C5); [R05 §Recommending a Stack](../../../skills/writing-history/references/convention-setup.md) — rows that move here

## Step 1: Setting up a repository
**Goal**: Let an agent adopt the lifecycle in any repo by deriving the release pipeline from the repo's contents.
**Implementation Logic**:
1. `## Capabilities a Release Pipeline Needs`: the checklist (version from commits since the last tag, changelog, tag, GitHub Release, optional registry publish, one tag line per independently versioned unit), and which are optional when nothing is published.
2. `## Choosing Tooling from Contents`: a decision procedure (inspect existing machinery first; identify the units and the main language; prefer tooling native to that language so CI needs no second toolchain, but treat this as a preference, not a rule: sustainability and leanness win) plus a matrix by contents: single JS/TS package or workspace; single Python package; Python with uv; Rust crate or cargo workspace with version groups; skill/plugin monorepo with per-unit releases; content-only, schema or catalogue repos (tag-only or none). Each row gives candidate tools, what each covers from the checklist, and its tag format. Use generic examples only (C5). Commit-message enforcement tools belong to `writing-history`; name them here only as the companion that checks messages.
3. `## Repository Settings`: merge commits only (squash and rebase merges off), merge commit title from the PR title and message from the PR body, delete the branch on merge. Give the `gh api -X PATCH repos/<owner>/<repo>` field names. Branch protection is decided in `ci-release-job.md`; link to it.
4. `## The CONTRIBUTING Section`: the sections to write (commit messages → `writing-history`; Integration lanes → `lifecycle.md#mapping-lanes-in-a-repository`; Versioning; Branches and integration), each with a one-line purpose.
5. `## Adoption Checklist`: an ordered checklist from audit to first verified release, including the step that the first release-type merge is the real test.
**Deliverables**: `skills/writing-release/references/setting-up.md` — sections `Contents`, `Capabilities a Release Pipeline Needs`, `Choosing Tooling from Contents`, `Repository Settings`, `The CONTRIBUTING Section`, `Adoption Checklist`; the tooling-by-contents matrix
**Consistency Checks**: `python3 -c "import re,sys;p=sys.argv[1];t=open(p).read();h=set(re.findall(r'^## (.+?)\s*$',t,re.M));m=[x for x in sys.argv[2:] if x not in h];n=t.count(chr(10));print(p,'missing:',m,'lines:',n);sys.exit(1 if m or n>350 else 0)" skills/writing-release/references/setting-up.md "Contents" "Capabilities a Release Pipeline Needs" "Choosing Tooling from Contents" "Repository Settings" "The CONTRIBUTING Section" "Adoption Checklist" && ! grep -nE '[Cc]andler|[Hh]atch|colgrep|[Pp]inion|[Pp]lumage|[Mm]urmuration|\bNest\b|py-repo-template|[Dd]iscord|[Rr]ework|[Pp]reviously|[Aa]s decided|old template' skills/writing-release/references/setting-up.md` (expected: PASS)
**Commit**: `feat(writing-release): choose release tooling from a repository's contents`

## Step 2: Channels
**Goal**: Argue the single-channel default and give the escalation path without building it.
**Implementation Logic**:
1. `## One Channel by Default`: `main` is what gets released. Explain why: one integration target, no drift between branches, every merge is shippable, and nothing to promote.
2. `## When to Add a Channel`: observable adoption triggers (a user base large enough that regressions reach many people before a fix; downstream consumers pinning versions and asking for a soak period; repeated hotfix churn after releases), and what does *not* justify a channel (wanting a place to park unfinished work, which belongs on topic branches).
3. `## The Lagged Release-Candidate Pattern`: `main` releases `-rc.N` prereleases; a scheduled job (for example weekly) promotes the newest rc that has soaked at least N days (for example 14) to stable. Cover what it needs (prerelease support in the tool, a promotion job, a policy for hotfixes during the soak). It is a sketch, not a build instruction.
4. `## Prereleases per Tool`: how each matrix tool expresses prerelease channels, linking to `setting-up.md#choosing-tooling-from-contents`.
5. `## The Cost of a Long-Lived Development Branch`: double integration, a drifting branch, a backlog that never releases, and prerelease tags nobody consumes, stated as a general pattern (C5).
**Deliverables**: `skills/writing-release/references/channels.md` — sections `One Channel by Default`, `When to Add a Channel`, `The Lagged Release-Candidate Pattern`, `Prereleases per Tool`, `The Cost of a Long-Lived Development Branch` (plus `Contents` if over ~100 lines)
**Consistency Checks**: `python3 -c "import re,sys;p=sys.argv[1];t=open(p).read();h=set(re.findall(r'^## (.+?)\s*$',t,re.M));m=[x for x in sys.argv[2:] if x not in h];n=t.count(chr(10));print(p,'missing:',m,'lines:',n);sys.exit(1 if m or n>350 else 0)" skills/writing-release/references/channels.md "One Channel by Default" "When to Add a Channel" "The Lagged Release-Candidate Pattern" "Prereleases per Tool" "The Cost of a Long-Lived Development Branch" && ! grep -nE '[Cc]andler|[Hh]atch|colgrep|[Pp]inion|[Pp]lumage|[Mm]urmuration|\bNest\b|py-repo-template|[Dd]iscord|[Rr]ework|[Pp]reviously|[Aa]s decided|old template' skills/writing-release/references/channels.md && uv run tools/package_skill.py skills/writing-release dist/` (expected: PASS)
**Commit**: `feat(writing-release): default to one release channel`
