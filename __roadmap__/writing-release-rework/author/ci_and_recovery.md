# CI Release Job and Recovery

**Goal**: Write the CI wiring patterns for gates and the release job (`ci-release-job.md`), and the operating and failure modes that follow from them (`recovery.md`).
**Pre-conditions**:
- [ ] The campaign README contract (C1–C7) is read in full
- [ ] GitHub Actions documentation (workflow triggers, concurrency, `GITHUB_TOKEN` permissions, branch protection and rulesets, GitHub App tokens) is reachable through Context7 or the official docs
**Success Gates**:
- ⬜ Both files carry exactly the C1 headings, are ≤350 lines, and contain no C5 residue
- ⬜ Every claim about GitHub behaviour (e.g. that a push made with `GITHUB_TOKEN` triggers no workflow, `pull_request` activity types, the rules for bypassing protection) cites the official doc it was checked against, in the leaf report (not in the file)
- ⬜ V1 (fact verifier) has reviewed the branch, and its findings are resolved before merge
**References**: [R01 §Candler](../../../__reports__/writing-release-rework/00-architecture_analysis_v0.md) — one worked pipeline and its failure lessons (source material only, C5)

## Step 1: The CI release job
**Goal**: Give tool-agnostic wiring patterns for a CI that gates PRs and pushes to `main` and cuts releases only after the gates pass.
**Implementation Logic**:
1. `## Triggers and Concurrency`: `pull_request` (including `edited`, so title changes are re-checked) and `push` to `main`; why topic-branch pushes alone don't run CI. Cancel in-progress runs on PRs but never on `main`, so a release is never cut short. `paths-ignore` for non-released paths (reports, roadmaps) on the heavy gates, which keeps the Direct lane cheap.
2. `## Job Graph`: gates → commit checks (range check over PR base..head or push before..after, plus the PR-title check) → release (`needs` both, `if` push to `main`). Show a small YAML skeleton with placeholders and no repo-specific names.
3. `## Permissions, Tokens, and Branch Protection`: minimal permissions (`contents: write` for the release job). With `main` unprotected, `GITHUB_TOKEN` can push the bump commit. Protection that requires PRs blocks that push, and also the Direct lane, unless a GitHub App token or ruleset bypass is used (the release bot plus maintainers). This section owns that tradeoff (C3). Note that pushes and PRs made with `GITHUB_TOKEN` trigger no workflow run.
4. `## The Bump Commit`: `chore(release): … [skip ci]` written by the bot; what it changes (versions, lock files, changelogs); why `[skip ci]`; and that it still falls inside the next push's commit-range check.
5. `## Hardening`: pin every action to an exact tag or SHA (a floating major tag can point at a broken release); give the runner a git `user.name` and `user.email` before any tool that reads them; never cancel on `main`.
6. `## Monorepos`: per-unit path filters, drift jobs that regenerate committed generated files and fail on a diff, and per-unit tag lines.
7. `## Idempotent Re-runs`: release tools that tag any untagged version can be safely re-run, and a run that loses the push race is recovered by the next run.
**Deliverables**: `skills/writing-release/references/ci-release-job.md` — sections `Contents`, `Triggers and Concurrency`, `Job Graph`, `Permissions, Tokens, and Branch Protection`, `The Bump Commit`, `Hardening`, `Monorepos`, `Idempotent Re-runs`; one placeholder YAML job-graph skeleton
**Consistency Checks**: `python3 -c "import re,sys;p=sys.argv[1];t=open(p).read();h=set(re.findall(r'^## (.+?)\s*$',t,re.M));m=[x for x in sys.argv[2:] if x not in h];n=t.count(chr(10));print(p,'missing:',m,'lines:',n);sys.exit(1 if m or n>350 else 0)" skills/writing-release/references/ci-release-job.md "Contents" "Triggers and Concurrency" "Job Graph" "Permissions, Tokens, and Branch Protection" "The Bump Commit" "Hardening" "Monorepos" "Idempotent Re-runs" && ! grep -nE '[Cc]andler|[Hh]atch|colgrep|[Pp]inion|[Pp]lumage|[Mm]urmuration|\bNest\b|py-repo-template|[Dd]iscord|[Rr]ework|[Pp]reviously|[Aa]s decided|old template' skills/writing-release/references/ci-release-job.md` (expected: PASS)
**Commit**: `feat(writing-release): wire the CI gates and release job`

## Step 2: Recovery
**Goal**: Give the symptom → cause → fix entries an agent needs when a release does not happen or fails.
**Implementation Logic**:
Each section follows symptom → how to confirm → fix → prevention, and links to the owning section in `ci-release-job.md` or `lifecycle.md` rather than restating it.
1. `## Nothing Was Released`: no release-type commit since the last tag, the path not part of a released unit, or a `[skip ci]` inherited by mistake.
2. `## The Bump Push Was Rejected`: the push lost a race to another push (non-fast-forward), so the next run releases; or protection blocks it (link to the tokens section).
3. `## First Release and Missing Tags`: how tools treat a repo without tags, and seeding a baseline tag deliberately.
4. `## Re-running a Release`: when a re-run is safe, and what to verify afterwards.
5. `## History Rewritten on main`: a force-push breaks before..after commit ranges and tag ancestry. Do not force-push `main`; recovering from one has its own steps.
6. `## Local main Is Stale`: the bump commit lands after the merge, so pull before branching again.
7. `## First Run of a New Release Job`: a checklist for watching the first release-type merge end to end (the bump commit, tag, release notes and changelog).
**Deliverables**: `skills/writing-release/references/recovery.md` — sections `Nothing Was Released`, `The Bump Push Was Rejected`, `First Release and Missing Tags`, `Re-running a Release`, `History Rewritten on main`, `Local main Is Stale`, `First Run of a New Release Job` (plus `Contents` if over ~100 lines)
**Consistency Checks**: `python3 -c "import re,sys;p=sys.argv[1];t=open(p).read();h=set(re.findall(r'^## (.+?)\s*$',t,re.M));m=[x for x in sys.argv[2:] if x not in h];n=t.count(chr(10));print(p,'missing:',m,'lines:',n);sys.exit(1 if m or n>350 else 0)" skills/writing-release/references/recovery.md "Nothing Was Released" "The Bump Push Was Rejected" "First Release and Missing Tags" "Re-running a Release" "History Rewritten on main" "Local main Is Stale" "First Run of a New Release Job" && ! grep -nE '[Cc]andler|[Hh]atch|colgrep|[Pp]inion|[Pp]lumage|[Mm]urmuration|\bNest\b|py-repo-template|[Dd]iscord|[Rr]ework|[Pp]reviously|[Aa]s decided|old template' skills/writing-release/references/recovery.md && uv run tools/package_skill.py skills/writing-release dist/` (expected: PASS)
**Commit**: `feat(writing-release): diagnose and recover failed releases`
