# writing-release rework — Architecture Analysis (v0)

Date: 2026-10-29

## Executive Summary

- **Problem:** `skills/writing-release` (1.0.0) drafts PR bodies and Discord posts for a fork → `dev` → `main` model that no Org repo now runs. It has never been used. The Org has no operational manual for its release cycle, and each repo has reinvented one.
- **Proposed change:** rewrite the skill as a release-cycle manual. It covers integration lanes into `main`, PRs whose title and body become the merge commit, releases cut by CI as a consequence of merging, release tooling chosen from what the repo contains, a single channel by default, and recovery. It ships as `writing-release@2.0.0`.
- **Non-goals:**
  - adopting the lifecycle in any repo;
  - changing repo settings;
  - announcement channels, which a possible future skill may cover (see the notice report).
- **Biggest risks:**
  - the skill gets overfitted to the one repo that triggered the rework;
  - meta-discourse from the refactor leaks into shipped files;
  - factual errors about release tools or GitHub permissions fail silently in a future repo.
- **Validation approach:** two fact and adversarial verifiers, a best-practices pass, a stand-alone pass, five acceptance evals, and the packaging and drift gates.

This report is **source material for the campaign only**. Nothing below may be quoted or named in shipped skill files: they describe patterns, never individual repos.

---

## Cross-repo survey (2026-10-29)

| Repo | Contents | Release tool | Trigger | Tag | Integration into `main` |
|---|---|---|---|---|---|
| Hatch | Python package | semantic-release + uv plugin, `dev` prerelease channel | push to `main`/`dev` | `v*`, `v*-dev.N` | GitHub PRs from forks, fork → `dev` → `main` |
| Hatch-Validator | Python package | semantic-release + uv plugin | push to `main`/`dev` | `v*` | PRs plus local merges into `dev` |
| Hatchling | Python package | semantic-release, `dev` channel | push to `main`/`dev` | `v*-dev.N` | local merges into `dev`; 177 unpushed commits, no stable release since `v0.5.0` |
| Hatch-MCP-Server | Python package | semantic-release (version bumped by hand) | push | `v*` | PRs; dormant since 2025-09 |
| py-repo-template | Python template | semantic-release + poetry plugin on a setuptools backend | push | `v*` | local merges |
| Hatch-Schemas | JSON schemas | none; tagged by an action | push to `main` (paths) | `schemas-*-vX` | PRs |
| Hatch-Registry | data catalogue | none; daily cron tag | cron | date | bot commits; dormant |
| Candler | Rust workspace | release-plz (bump, changelog, tag, GitHub Release) + cocogitto (message checks only) | push to `main`, gated on CI | `v*` + `schemas@*` | moving from local `--no-ff` to PRs ([PR #1](https://github.com/CrackingShells/Candler/pull/1)) |
| colgrep-mcp | Python + uv | commitizen `cz bump`, run locally; tag push publishes to PyPI | tag push | `v*` | **GitHub PRs** (Org-native) |
| Pinion, Plumage, playbook | skill/plugin monorepos (Node tooling + uv scripts) | multi-semantic-release, per-skill units | push to `main` (`skills/**`) | `<skill>@*` | local `--no-ff` then push |
| Murmuration | Rust + Python plugin | none (version edited by hand) | — | none | local merges |
| Nest | marketplace catalogue | none | — | none | local merges |

**Patterns**

- **Common:**
  - Conventional Commits;
  - a CI-made bump commit `chore(release): … [skip ci]` on `main`;
  - a tag and a GitHub Release.
- **The tool follows the language:** semantic-release for Node and most Python, release-plz for Rust, commitizen for Python with uv.
- **Pushing past branch protection:** the Python family mints a GitHub App token for the bump push. The monorepos and Candler push with `GITHUB_TOKEN` because `main` is unprotected.
- **Prerelease channels:** the `dev`-channel repos are the stale or broken ones. Hatchling has a large unreleased backlog on `dev`, and Hatch-Validator's `dev` sits ahead of `main` with nothing due. That is evidence that the channel cost more than it returned at this scale.

## Candler (2026-10-29), as one data point

- **Branching:** `main` only, with topic branches `<type>/<topic>`.
- **PR flow** ([PR #1](https://github.com/CrackingShells/Candler/pull/1), not yet merged):
  - merge commits only;
  - `merge_commit_title=PR_TITLE` and `merge_commit_message=PR_BODY`;
  - delete branch on merge;
  - CI checks the PR title `merge(<scope>): …`;
  - rebase plus `--force-with-lease` before merging;
  - pull `main` after merging.
- **CI in one `ci.yml`:**
  - jobs run gates → commits (`cog check` over the range, plus a title check) → release (push to `main` only, never cancelled);
  - `release-plz update` writes the version, lock-file and changelog edits locally (it does not commit); the CI job commits and pushes them (corrected after the docs check: the release-plz docs describe `update` as local updates without committing);
  - `release-plz release` creates the tags and GitHub Releases;
  - no registry publish.
- **Lessons:**
  - `cog verify` needs a git `user.name` **and** `user.email` on the runner;
  - floating action tags (`@v4`) pointed at a broken release, so actions get exact pins.
- **Decisions:**
  - no release PR, to keep a single release ceremony across the Org;
  - release-plz over multi-semantic-release, because it is native to Rust and handles workspaces and version groups;
  - merge-commit-only, because squash loses the commits the changelog is built from and rebase-merge loses the merge record.
- **Not yet proven in CI:** the bump push and GitHub Release creation. The first `feat` or `fix` to land will be the real test.

## Decisions for the skill

They are recorded in the campaign contract at [`__roadmap__/writing-release-rework/README.md`](../../__roadmap__/writing-release-rework/README.md):
- the integration lanes;
- single channel by default;
- tooling derived from the repo's contents;
- the boundary with writing-history;
- Discord retired to a possible future skill;
- the stand-alone rule.

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Content overfitted to Candler or named repos | High (the trigger case is fresh) | Skill misleads in other repos | Stand-alone rule in the contract; `standalone` pass; residue gate |
| Factual errors about tools, tokens or protection | Medium | Silent CI failure in a future adoption | V1 checks claims against official docs |
| Three parallel authors contradict one another | Medium | Incoherent manual | Owner table + pinned lane wording in the contract; V2 |
| Generated `plugins/` conflicts across branches | High without a rule | Rebase churn | `--no-verify`, never stage `plugins/`; the coordinator regenerates once |
| Wrong release bump | Low | 2.0.0 not cut, or other skills released | Single BREAKING footer; git-log release analysis |
| Self-merge lane blocked by a permission classifier | Medium | Friction | The lane falls back to the maintainer merging |
