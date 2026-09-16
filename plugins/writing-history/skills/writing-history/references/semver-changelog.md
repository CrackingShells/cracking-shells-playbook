# Semver & Changelog Reference

Tool-agnostic principles tying structured commit messages to automated semantic-version bumps and changelog generation. Read this after [commit-authoring.md](commit-authoring.md) — that file tells you *how* to write a commit against a derived vocabulary; this file explains *why* that vocabulary matters beyond the commit itself, and how it flows downstream into a version number and a changelog entry.

## Table of Contents

1. [Ground Truth: The Vocabulary Is Derived](#ground-truth-the-vocabulary-is-derived)
2. [From Commits to Versions](#from-commits-to-versions)
3. [Changelog Automation](#changelog-automation)
4. [Documents, Not Just Code](#documents-not-just-code)
5. [Relationship to Setup](#relationship-to-setup)

---

## Ground Truth: The Vocabulary Is Derived

The same ground-truth rule [commit-authoring.md](commit-authoring.md) applies to writing a commit message applies here to reading one back out: **the authorized commit vocabulary, and the version-bump semantics attached to it, are never a hardcoded universal table.** They come from the project itself, in this order of precedence:

1. **The project's machinery** — a commitlint config (`type-enum`/`scope-enum`), a semantic-release config's commit-analyzer preset or rules, or an equivalent automation file. If it exists, it *is* the authorized vocabulary and its bump mapping — read it and obey it, don't infer.
2. **Absent machinery, the project's history + docs** — `git log` conventions actually in use, plus any `CONTRIBUTING`/versioning guidelines. If a project has been tagging `feat:` as minor and `fix:` as patch for 200 commits without a config file to say so, that convention is still the ground truth; it's just implicit rather than codified.
3. **Neither exists** — this is a greenfield project. There is no vocabulary to derive yet; route to [convention-setup.md](convention-setup.md) to stand one up rather than guessing.

A skill (or an agent using one) never imposes a `feat/fix/docs/chore/...` table as *the* answer for every project. That table is one popular convention among many — useful only insofar as a given project's own machinery or history actually adopts it.

## From Commits to Versions

Once a vocabulary is established (by whichever precedence step applies), it maps each commit's type/class to a semver bump: major, minor, or patch. That mapping is a **project-defined contract**, not a language universal. The classes that exist, and which bump each one triggers, differ per project and even per material type within a project (see "Documents, Not Just Code" below).

A worked example, framed generically rather than tied to any one project: suppose a project's machinery defines three classes — `feat`, `fix`, and `chore` — and maps `feat` to a minor bump, `fix` to a patch bump, and `chore` to no bump at all. A commit-analysis step (whatever tool implements it — a commit-analyzer-style preset is one common shape, not the only one) reads each commit's `type(scope):` prefix since the last release, looks up its class in that mapping, and takes the *highest* bump found across the whole batch. Two `fix` commits and one `feat` commit since the last tag yield a minor bump overall, regardless of how many patch-level commits rode along. Nothing here should be read as "this three-class mapping is universally correct" — it is correct only for a project whose own config or documented convention actually defines it that way. A different project's mapping could remap `perf` to `minor`, drop `revert` bumps entirely, or invent a `data`-type class scoped to dataset commits — all equally valid, because the mapping lives in that project's machinery, not in a skill's assumptions.

The pattern to internalize, not the specific numbers:

- Enumerate the classes the project's machinery (or history) actually recognizes.
- For each class, find its bump (major / minor / patch / none) in that same machinery — or, absent machinery, treat the implicit pattern in history as the working hypothesis and confirm it against any versioning docs.
- Never backfill a missing class with a "standard" bump from memory; a class with no defined bump is a signal to ask, not to assume.

## Changelog Automation

A changelog is the human-readable byproduct of the same structured history: once commits carry a reliable type/scope, a generator can group them (features, fixes, breaking changes, ...) and emit release notes without manual transcription. The mechanism is tool-agnostic — pick options based on what the project's machinery already establishes rather than what a skill would prefer:

- **Node/JS ecosystems**: `semantic-release` orchestrating `@semantic-release/commit-analyzer` (bump decision), `@semantic-release/release-notes-generator` (notes), and `@semantic-release/changelog` (writing `CHANGELOG.md` to disk) is one common combination.
- **Python ecosystems**: equivalents such as `python-semantic-release` or `commitizen` play the same role — analyze structured commits, compute a bump, emit notes.
- **Language-agnostic / CI-native**: tools like `release-please` or `standard-version` achieve the same pipeline outside any single language's package manager.

None of these is *the* answer; each is interchangeable machinery implementing the same principle. As a concrete example already presented above — the `feat`/`fix`/`chore` mapping into minor/patch/no-bump classification — the same structured commit data that decided the version bump also drives the changelog: a generator groups the same commits by type into sections (Features, Fixes, ...) and renders release notes from them without manual transcription, one worked instance of "structured commits in, changelog and version out," not a template to copy verbatim into unrelated projects.

## Documents, Not Just Code

Commit-to-version-to-changelog automation is not a code-only concept. A repository of scientific manuscripts, protocols, or other prose deliverables can run the identical pipeline once *that* project defines what its own change classes mean:

- A **major** bump might mean a manuscript's conclusions or methodology changed materially — something a downstream reader must know invalidates prior citations of it.
- A **minor** bump might mean a new section was added (new content, no retraction of prior claims).
- A **patch** bump might mean a wording fix, citation correction, or typo — no substantive change to what the document claims.

These mappings are that document repository's own vocabulary, defined the same way a code repo's is: by its machinery if it has any (a changelog-generator config keyed off commit types), or by its history and any authoring guidelines if it doesn't. A skill helping with a document repo must ask the same ground-truth question — "what does this project's own convention say a `feat`-equivalent commit means here?" — rather than assuming code semantics ("new feature" = "new function") transfer unchanged to prose ("new feature" = "new section" is a plausible project choice, but it is a choice that project makes, not a universal fact).

## Relationship to Setup

This file explains the *principles* — why the vocabulary is derived, how it maps to bumps, and how that mapping feeds changelog automation. It assumes a vocabulary already exists to derive, whether codified in machinery or implicit in history and docs.

When no machinery exists and no established history/docs pattern can be derived — a truly greenfield project — [convention-setup.md](convention-setup.md) is where that vocabulary and its bump mapping get stood up in the first place: it runs the setup interview, helps choose tooling appropriate to the project's materials, and bootstraps the config that this file's principles then apply to going forward. Read [convention-setup.md](convention-setup.md) for the setup flow; read [commit-authoring.md](commit-authoring.md) for how the resulting vocabulary is applied commit-by-commit.
