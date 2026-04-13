# Skill CI/CD Pipeline — Knowledge Transfer (v0)

Date: 2026-04-13

## Executive Summary

- **What happened:** Audited and repaired `dist/` (3 skills stale, 1 missing, 1 misnamed). Designed a full CI/CD pipeline for per-skill versioning and automated GitHub Releases using `semantic-release-monorepo`. Implementation not yet started.
- **Primary outcomes:** `dist/` is now in sync. Architecture analysis and implementation plan are documented. Decision log is captured below.

---

## Wins

- The `writing-release/evals/evals.json` file is correctly excluded by the existing packager (`ROOT_EXCLUDE_DIRS = {"evals"}`) — no custom logic needed.
- The `managing-roadmaps/scripts/` tree (shell wrapper + Rust binaries) is correctly included by default, because the packager uses `rglob('*')` with only a small exclusion set. The user explicitly confirmed scripts must ship.
- `semantic-release-monorepo` was identified as the right fit quickly: same plugin ecosystem as the rest of the org, path-based commit attribution solves the scope-drift problem, and the tag format (`<name>@<version>`) is conventional.

---

## Pain Points

- **Two different versions of `package_skill.py` exist** in the local environment. The version at `/Users/hacker/.agents/skills/skill-creator/scripts/package_skill.py` (the default agent install) has no exclusion rules at all — `rglob('*')`, include everything. The session-specific version at the plugin path `/Users/hacker/Library/Application Support/Claude/local-agent-mode-sessions/skills-plugin/.../scripts/package_skill.py` has the full exclusion logic. This caused the Explore agent to report incorrect information during planning.
  - **Implication:** vendoring the script into `tools/` is urgent — there must be one canonical version in the repo.

- **`package.json` is not excluded by either packager version.** Without the fix in the vendored `tools/package_skill.py`, the monorepo marker ships inside every `.skill` artifact. This was caught during planning, not after implementation — but only because the question was asked directly.

- **`reporting.skill` diverged from the skill rename.** The skill was renamed `writing-reports` (SKILL.md `name:` field updated, directory renamed) but `dist/reporting.skill` was never regenerated, and its internal ZIP paths still used `reporting/` as the prefix. No automated check caught this.

---

## Root Causes

- **No canonical packager in the repo.** The packager lives in an external plugin, making it session-dependent and version-ambiguous. Any contributor or CI job without the exact plugin installed would use a different (or no) packager.
- **No automation linking `skills/` changes to `dist/` updates.** The gap is structural: nothing enforces that a skill change implies a dist rebuild. The pre-commit hook and CI pipeline close this gap.
- **`dist/reporting.skill` naming mismatch** arose because the rename of `writing-reports` was committed without regenerating `dist/`. A pre-commit hook would have caught this immediately.

---

## Next-Cycle Changes

- **Instruction changes:** Add a `CONTRIBUTING.md` note that explains the pre-commit hook, the `make dev-setup` requirement, and the initial-release behavior (all skills release at once on first SR run).
- **Workflow changes:** After `make dev-setup`, contributors never need to think about `dist/` — the hook handles it. Document this explicitly so contributors do not manually run the packager.
- **Review process changes:** PR review should check that `dist/<name>.skill` is updated whenever `skills/<name>/` is changed. Once the hook is in place, this is automatic and the check becomes "does the hook work?" rather than "did the contributor remember?"

---

## Artifacts to Preserve

| Artifact | Location | Notes |
|----------|----------|-------|
| Architecture analysis | [`__reports__/skill_cicd/00-architecture_analysis_v0.md`](00-architecture_analysis_v0.md) | Contracts, diagrams, alternatives, risk register |
| Implementation plan | [`~/.claude/plans/woolly-enchanting-flamingo.md`](~/.claude/plans/woolly-enchanting-flamingo.md) | Full deliverable list with file paths |
| Canonical packager (source) | skill-creator plugin, session path | Must be vendored to `tools/` as first implementation step |

---

## Open Questions

- Does the org have a shared `GITHUB_TOKEN` secret configuration, or does each repo need its own? Affects `.github/workflows/release.yml` permissions block.
- Should `@semantic-release/changelog` write a per-skill `CHANGELOG.md` inside `skills/<name>/`? If yes, that file would be committed (via `@semantic-release/git`) and also packaged into the `.skill` artifact — potentially desirable for consumers.
- Should the initial release tag all skills at `1.0.0`, or start at `0.1.0` to signal pre-stable? Conventional for SR: first meaningful release is `1.0.0` if a `feat:` commit exists, else `0.0.0` with no release. Consider seeding with an explicit `feat:` commit per skill to trigger clean `1.0.0` tags.
- `managing-roadmaps.skill` is ~2 MB (includes Rust binaries). Is this within GitHub Release asset size limits for the org's plan? (GitHub free plan: 2 GB per release, so fine — but worth confirming.)
