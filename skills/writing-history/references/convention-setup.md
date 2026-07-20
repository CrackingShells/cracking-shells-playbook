# Convention Setup

An agent-run setup interview and bootstrap flow for standing up commit-convention, semantic-versioning, and changelog machinery in a project that has none. This reference is the greenfield branch referenced by [commit-authoring.md](commit-authoring.md)'s vocabulary-derivation precedence: when a project has neither machinery nor a derivable history/CONTRIBUTING pattern, run the flow below instead of guessing.

This flow is explicitly **tool-agnostic**. There is no one-size-fits-all stack: the right commit vocabulary, versioning scheme, and changelog tool depend on what the repository actually contains — its materials, languages, file types, and any tooling already present. The agent's job is to ask, look, and justify a recommendation from that context — never to reach for a default stack out of habit.

## Table of Contents

1. [When to Run Setup](#when-to-run-setup)
2. [Setup Interview](#setup-interview)
3. [Derive vs. Bootstrap](#derive-vs-bootstrap)
4. [Recommending a Stack](#recommending-a-stack)
5. [Bootstrap Checklist](#bootstrap-checklist)
6. [Delivering Recommendations](#delivering-recommendations)

---

## When to Run Setup

Run this setup flow only when **both** of the following hold:

1. **No machinery** — no commitlint config, no semantic-release config (`.releaserc*`, `release.config.*`), no equivalent tool config for the project's ecosystem (e.g. a Python `commitizen`/`bumpver` config, a `.changeset/` directory).
2. **No established pattern to derive from** — `git log` shows no consistent conventional-style history, and there is no `CONTRIBUTING.md` (or equivalent) documenting a commit-message convention.

This is the greenfield branch of the precedence described in [commit-authoring.md](commit-authoring.md): that reference always checks (a) project machinery, then (b) existing history/docs, before falling back to (c) this setup interview. If either (a) or (b) is present, do **not** run this flow — derive the vocabulary instead and skip straight to authoring commits.

If setup is triggered mid-project (e.g. a repo has *some* history but no formal convention), treat it as greenfield for the vocabulary decision, but surface the existing history to the user during the interview so the new convention doesn't contradict established scope names or tone.

## Setup Interview

Ask the user the following questions before recommending anything. Answers determine tooling — file types and materials drive tool choice, not the other way around. Do not skip ahead to a stack recommendation without these answers; a stack picked before understanding the repo is a guess, not a recommendation.

1. **What does the repo hold?** Code, prose/documentation, data, research artifacts, configuration, or a mix? A pure-code repo, a scientific-writing repo, and a mixed monorepo warrant different vocabularies and different bump semantics (see [semver-changelog.md](semver-changelog.md) for how "minor" vs "patch" differs by material type).
2. **Which languages and file types are involved?** (e.g. TypeScript/JavaScript, Python, Rust, Markdown/LaTeX, notebooks, YAML/JSON configs). This narrows the realistic tool candidates — a JS ecosystem has different native tooling than a pure-Python or pure-prose repo.
3. **Is there existing tooling already?** Git hooks, CI workflows, a release process of any kind (even manual), package manifests. Existing tooling is often the path of least resistance to extend rather than replace.
4. **Who commits?** Humans only, agents only, or both? Agent-authored commits change what's realistic to enforce locally (see the trade-off in "Bootstrap Checklist" below) — a repo committed to mostly by agents may prefer CI-only enforcement over a blocking local hook.
5. **What release cadence matters, and does a changelog matter to stakeholders?** Continuous releases, periodic tagged releases, or no formal releases at all? Do external consumers (users, collaborators, reviewers) read a changelog, or is version history purely internal bookkeeping?
6. **Is this repo a monorepo with independently-versioned units, or a single-versioned whole?** This determines whether the orchestrator needs path-scoped, per-unit release logic or a single top-level version.

Record the answers; they are the justification the agent must cite when it proposes a stack in the next step.

## Derive vs. Bootstrap

"When to Run Setup" above is the single gate for this flow; do not re-check it here. This section covers only what to do if one of those two conditions turns out not to hold after all:

- **Machinery turns up** — that machinery **is** the authorized vocabulary; nothing to bootstrap. Route back to [commit-authoring.md](commit-authoring.md) and derive from it directly.
- **A derivable history/`CONTRIBUTING` pattern turns up** — formalize it (write it down, optionally add lightweight tooling to enforce it) rather than replacing it with an unrelated stack, then route back to [commit-authoring.md](commit-authoring.md).

Bootstrapping over an already-derivable convention is a mistake: it discards working, human-legible history in favor of a stack that may not fit how the project already communicates change.

## Recommending a Stack

There is no single correct stack — the right choice is a function of the interview answers, not a default. Use the following as illustrative mappings from common contexts to sensible options, not as a menu to pick from blindly; any recommendation must be justified by citing the specific interview answers that motivated it.

| Repo context | Sensible option(s) | Why |
|---|---|---|
| JS/TS package or monorepo | Conventional Commits + `commitlint` + `semantic-release` (or `multi-semantic-release` for independently-versioned units) | Native to the ecosystem; widely known by contributors and agents alike |
| Python package | Conventional Commits + `commitizen`, `python-semantic-release`, or an equivalent | Mirrors the JS pattern without requiring a Node toolchain |
| Prose / scientific-writing / documentation repo | A lightweight, project-defined change-class vocabulary (e.g. `add`/`revise`/`fix`/`retract`) + a simple changelog generator, or even a hand-maintained `CHANGELOG.md` with a documented convention | Full semantic-release machinery is often overkill when there's no package to publish; the convention still needs to exist, just not the automation weight |
| Mixed monorepo (code + docs + data) | Path-scoped, per-unit release orchestration (e.g. `multi-semantic-release`) with a shared commit-type vocabulary that spans material types | Keeps independent units on independent version lifecycles while sharing one vocabulary |
| No release cadence / internal-only versioning | A documented commit convention alone, with changelog and version bumps deferred until they're actually needed | Avoids standing up machinery nobody consumes |

One concrete, fully worked example of a stack instantiated for a monorepo of independently-versioned units — npm workspaces + Conventional Commits + `multi-semantic-release` + GitHub Actions + a content-type-specific packaging script — is documented in [R03 release pipeline architecture analysis](../../../__reports__/skill_cicd/01-architecture_analysis_v0.md). Treat it as *one* worked option to study and adapt, not as the mandated answer for every repo; its own "Generalization Template" section walks through what changes per content type.

Whatever is proposed, state explicitly which interview answers led to it. "This repo is JS/TS with no existing tooling and wants an automated changelog for external users, so commitlint + semantic-release fits" is a recommendation; "use semantic-release" alone is not.

## Bootstrap Checklist

Once a stack is chosen (see "Delivering Recommendations" for how it gets approved), stand it up in this order:

1. **Define the vocabulary** — write down the commit-type set (and scope conventions, if any) as the project's own documented convention, sized to the repo's material types (see interview Q1–Q2).
2. **Add machinery config** — install and configure the chosen tool(s): commitlint config (`type-enum`, `scope-enum` if applicable), release-tool config (`.releaserc.js` or equivalent), and any per-unit manifest needed for path-scoped versioning.
3. **Wire changelog automation** — connect the release tool's changelog plugin/equivalent, or, for lighter-weight setups, document the manual changelog-update step.
4. **Decide local-hook vs. CI-only enforcement** — a blocking local hook (e.g. commitlint in a pre-commit hook) gives immediate feedback but can block agent-authored commits with unconventional but valid messages; CI-only enforcement is friendlier to mixed human/agent commit workflows at the cost of delayed feedback. Choose based on interview Q4 (who commits).
5. **Document in CONTRIBUTING** — write the convention, the bump mapping, and the enforcement model into `CONTRIBUTING.md` (or equivalent) so it becomes the derivable ground truth for future contributors and for [commit-authoring.md](commit-authoring.md)'s "not-our-project" derivation path.

The bump-mapping *principles* this checklist encodes — how commit classes map to major/minor/patch, and what changelog automation options exist — are covered in depth in [semver-changelog.md](semver-changelog.md); this checklist is the mechanical "how to stand it up," that reference is the "why it works this way."

## Delivering Recommendations

Present the interview answers, the derived recommendation, and its justification together as a single proposal — before creating any config file. The user must approve the stack (or redirect it) before the agent writes commitlint configs, release configs, CI workflow changes, or CONTRIBUTING edits.

A recommendation delivered for approval should include:

- A summary of the interview answers that drove the choice.
- The proposed vocabulary (types/scopes) and bump mapping.
- The proposed tool(s), with the one-line "why this fits this repo" justification.
- The bootstrap steps that will run once approved, so the user knows what's about to change.

Only after explicit approval does the agent proceed through the Bootstrap Checklist and create configuration.
