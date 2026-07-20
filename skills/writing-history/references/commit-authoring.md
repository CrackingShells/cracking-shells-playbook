# Commit Authoring Reference

Complete rule set for authoring a single commit message. Read this file before drafting any commit.

## Table of Contents

1. [Primary Directive](#1-primary-directive)
2. [Deriving the Vocabulary](#2-deriving-the-vocabulary)
3. [Scope Naming: Explicit Topic Identification](#3-scope-naming-explicit-topic-identification)
4. [Scope Consistency and Development Narrative](#4-scope-consistency-and-development-narrative)
5. [Body and Footer Guidance](#5-body-and-footer-guidance)
6. [History-Aware Style Matching](#6-history-aware-style-matching)
7. [Sensitive-File Advisory](#7-sensitive-file-advisory)
8. [Dirty-File Separation](#8-dirty-file-separation)
9. [Validation](#9-validation)
10. [Examples](#10-examples)

---

## 1. Primary Directive

**WHY over WHAT.** The commit message description must answer *why* this change exists — not just what changed. The diff shows what; only the message captures intent. Future agents and developers reading the history need the reasoning, not a restatement of the mechanic.

This directive applies identically whether the changed material is source code, configuration, or prose (a manuscript section, a protocol document, a dataset README). A commit is a historical record of a decision; the record must justify the decision.

### Structure

```
type(scope): description

[optional body]

[optional footer(s)]
```

**Rules:**
- `scope` is **mandatory** — never omit it
- Blank line between subject and body; blank line between body and footers

### Subject Line Rules Are a Fallback, Not a Law

The rules below (length, case, mood) apply **only when the project's own machinery or documentation defines none of its own** (see Section 2). If a commitlint config sets `subject-max-length` to 100, or a `CONTRIBUTING.md` says descriptions may run to a full sentence, that project's rule wins. These fallback rules exist purely so an agent has *something* defensible to apply when nothing project-specific has been found — not because 72 characters or lowercase is universally correct.

Fallback rules, used only absent any project-specific rule:
- Imperative mood: "add", "fix", "remove" — not "added", "fixes", "removing"
- Lowercase after the colon
- No trailing period
- ≤72 characters total

**Gate command** — before committing, run this against the drafted subject to check it against whichever length limit applies (the project's own documented limit if one exists, 72 as fallback otherwise):

```bash
printf '%s' "<drafted subject line>" | wc -c
```

Read the project's own limit first (Section 2); only fall back to 72 characters when nothing else says otherwise. This command is a pre-write gate an agent runs on its own draft — it is not a substitute for running the project's actual linter when one exists (Section 9 covers that).

---

## 2. Deriving the Vocabulary

**This skill does not impose a fixed `feat`/`fix`/`docs`/... type set.** The set of valid `type` values (and, where applicable, valid `scope` values) is never hardcoded here — it is *derived* from the project being committed to, in the following order of precedence.

### (a) Project machinery is ground truth

If the repository has commit-linting or release-automation configuration of any kind, its rules **are the authorized vocabulary**. Read the config and obey it exactly; do not substitute a type or scope the config does not list, even if it looks conventional.

The pattern to apply — not a list to match: look for *any* configuration that governs commit messages, releases, or changelog generation. That configuration is typically either a dedicated dotfile or a section embedded inside a manifest the project already has (a package/project file, a build config). Open whatever you find, and look for the shape common to nearly all such tools: an enumerated list of allowed types (and often scopes), plus a rule mapping each type to a version-bump class. The field names differ per tool; the shape recurs.

To make that shape concrete — purely illustrative, and explicitly **non-exhaustive** — this can look like a commitlint-style config with a `type-enum` array, a semantic-release-style config whose analyzer preset defines the mapping, a Python project's `pyproject.toml` table for a commit/versioning tool, a dedicated changelog-generator config, or a Rust project's release-tooling config. Plenty of projects use tooling not resembling any of these, including tools invented in-house or ones neither this reference nor the agent has ever seen. The rule is **discover and read whatever configuration actually exists**, never "check whether it matches a known tool's name." If something config-shaped turns up under an unfamiliar name, read it the same way: extract whatever type/scope rules it defines and treat those as authoritative.

If a type-enumeration rule is present (one common name for it is `type-enum`; other tools name the equivalent differently), extract its allowed list and use only those values. Do the same for any scope-enumeration rule.

### (b) Not-our-project: derive from history and docs

When contributing to a project that is not the agent's or user's own — a third-party repository, an external open-source project, a client's existing codebase — and no machinery is found, **do not invent a vocabulary**. Derive it from what the project already does:

- Run `git log --oneline -30` (a wide sample, deliberately larger than the sample used for style-matching in Section 6, because vocabulary breadth requires seeing rarely-used but valid types, not just the most recent handful) and tabulate the `type(scope):` prefixes actually in use. Treat the observed set as authoritative, including any project-specific types (e.g. a documentation project using `revise` or `errata` instead of `fix`).
- Read any `CONTRIBUTING.md`, `CONTRIBUTING.rst`, `CODE_OF_CONDUCT.md` commit-message section, or repository wiki page that documents commit conventions. These docs take precedence over inferred patterns when they conflict, since they represent the maintainers' explicit intent.
- If history is sparse or inconsistent, prefer whatever `CONTRIBUTING` says; if both are silent, fall back to the closest observed convention and note the ambiguity to the user rather than silently picking a convention of your own.

### (c) Greenfield: route to setup

If the repository has **neither** machinery **nor** an established history/`CONTRIBUTING` pattern (e.g. the very first few commits, or a brand-new repo), do not guess at a vocabulary. Route to [`convention-setup.md`](convention-setup.md) to run the setup interview and stand up a vocabulary deliberately, in collaboration with the user.

### Precedence summary

```
config exists?        --yes--> use its type-enum / scope-enum (ground truth)
   |no
history/CONTRIBUTING?  --yes--> derive vocabulary from git log + docs
   |no
greenfield              --------> convention-setup.md (setup interview)
```

This same precedence is the one [`semver-changelog.md`](semver-changelog.md) relies on when mapping commit types to version bumps — the vocabulary is derived once, then reused for both authoring and bump semantics.

---

## 3. Scope Naming: Explicit Topic Identification

### Core Principle

The scope must identify the **topic/subject** of the commit — not the action, not the layer, not the result. It answers: *"What is this commit primarily about?"* This holds for code changes and for every other kind of tracked file: a scope names the topic a manuscript section, dataset, configuration surface, or design asset addresses, just as it names a feature or module in code.

### Good Scope Patterns

| Pattern | Code examples | Prose/document examples | Other artifact examples |
|:--------|:---------------|:--------------------------|:---------------------------|
| Feature/topic names | `auth`, `search-index`, `onboarding` | `methods`, `results-fig3`, `discussion` | `pricing-model` (dataset), `brand-palette` (design asset) |
| Component/module names | `api-client`, `database`, `worker-pool` | `protocol`, `dataset-schema`, `appendix-b` | `nda-terms` (legal doc), `ci-pipeline` (config) |
| Concepts | `rate-limiting`, `unicode-handling` | `statistics`, `citation-style`, `figure-captions` | `retention-policy` (data governance), `token-budget` (config) |
| Files/modules | `config.yml`, `main.py` | `manuscript.tex`, `README.md` | `release.toml`, `grant-budget.xlsx` |

### Bad Scope Patterns

| Anti-pattern | Why it fails |
|:-------------|:-------------|
| `core` | Too generic — no traceability |
| `refactor` | Describes action, not subject |
| `misc` | Zero information |
| `backend` | Architectural location, not topic |
| `update` | Describes action, not subject |
| `paper` (when the whole repo is one paper) | Too generic to distinguish sections |

### Scope Discovery Process

1. Ask: *"What is this commit primarily about?"*
2. Use names from requirements, docs, existing codebase components, or — for non-code repos — section/figure/dataset/asset names
3. Check the derived vocabulary (Section 2) for an allowed-scope list before inventing a new one
4. Pick the most specific name available

---

## 4. Scope Consistency and Development Narrative

### Principle

Related commits **must share the same scope** to form a coherent development story. Scope changes signal a shift in focus — use them intentionally.

### Good Narrative Flow (code)

```
feat(search-index): add incremental reindexing on file save
feat(search-index): implement query result ranking
fix(search-index): handle stale index after bulk delete
docs(search-index): document reindexing trigger configuration
```

Reading left to right, the commit history tells a story: "We built the search-index feature."

### Good Narrative Flow (document/scientific repo)

```
docs(methods): describe cohort selection criteria
docs(methods): add power-analysis justification
fix(methods): correct sample-size formula
docs(methods): cross-reference supplementary table
```

The story is equally coherent: "We wrote and corrected the Methods section."

### Bad Narrative Flow

```
feat(search-index): add incremental reindexing
feat(config): update settings format
fix(connection): handle timeout
chore: update dependencies
```

The story is fragmented — no coherent narrative about what was built.

### Key Rules

- Related commits → same scope
- Scope changes → intentional shift in focus
- Inconsistent scopes → broken development story
- Consistent naming → traceable git blame and changelog, for code and every other tracked material alike

---

## 5. Body and Footer Guidance

### When the Body Is Needed

A subject line is one line — often too short to hold the WHY that Section 1 demands. Per the primary directive, a body is not a rare addition reserved for exceptional cases; **strongly favor writing one**, since the reasoning behind a change routinely doesn't fit in ~72 characters. Treat a subject-only commit as the exception, appropriate only when the change is small and self-evident enough that no further reasoning needs recording.

A body is **mandatory**, not merely encouraged, when:

- The change is breaking — the migration path must be documented
- The rationale involves trade-offs, alternatives considered, or context a future reader cannot reconstruct from the diff alone
- The change closes an issue or references an external discussion that needs a durable pointer

### Body Structure

```
<type>(<scope>): <imperative summary>

<paragraph(s) explaining why this change was necessary — the
context, constraints, or trade-offs that motivated it>

Key changes:
- <component/aspect>: <what changed and why>
- <component/aspect>: <what changed and why>
```

- Blank line between subject and body
- Wrap at roughly 72 chars per line, or the project's own documented wrap convention if it has one
- Use imperative mood consistently with the subject

### Footer Format

```
Resolves #123
Fixes #456
Closes #789

BREAKING CHANGE: <description of what breaks and how to migrate>
Migration guide: <path or link, if one exists>
```

- `Resolves`, `Fixes`, `Closes` followed by an issue reference
- `BREAKING CHANGE:` on its own line, followed by a description
- One footer key per line

Body and footer discipline applies identically regardless of what kind of file changed — a manuscript revision with a substantive rationale needs a body exactly as much as a breaking API change does; a dataset correction that invalidates a prior export needs a `BREAKING CHANGE:`-equivalent footer just as much as a removed function signature does.

---

## 6. History-Aware Style Matching

### Principle

Before drafting a commit message, read the existing commit history to observe the tone, vocabulary, tense conventions, and scope naming already in use. Match that style — this is what Section 2(b) calls deriving from history, applied at the level of phrasing rather than vocabulary set.

### How to Do It

```bash
git log --oneline -10
```

A smaller, more recent sample than the one used for vocabulary derivation in Section 2(b) — style matching cares about how the project writes *right now*, not every type it has ever used, so recency matters more than breadth here.

Look for:
- **Scope names in use** — if the repo uses `auth` not `authentication`, use `auth`; if a document repo uses `methods` not `methodology`, use `methods`
- **Tense** — does the project use "add X" or "adds X"? Match it
- **Casing** — are scopes `kebab-case`, or `PascalCase` matching a class/component name?
- **Description style** — terse or descriptive?

### Why It Matters

Prevents scope name drift. Without this check, an agent might introduce `auth-service` into a repo that consistently uses `auth`, or `methodology` into a repo that consistently uses `methods` — breaking changelog generation and grep patterns either way.

---

## 7. Sensitive-File Advisory

### Agent Awareness Rule

Before staging and committing, inspect the diff for sensitive content. **Do not commit:**

- `.env` or `.env.*` files
- Files containing patterns: `API_KEY=`, `SECRET=`, `PASSWORD=`, `PRIVATE_KEY`
- `*.pem`, `*.key`, `id_rsa`, `id_ed25519` private key files
- `credentials.json`, `service-account.json`, `aws-credentials`
- Any file with tokens, bearer credentials, or connection strings with passwords
- For document repos: unpublished participant-identifying data, embargoed datasets, or reviewer comments not meant for the public history

### Procedure

```bash
git diff --staged
```

Scan the staged diff before committing. If sensitive content is found: warn, halt, and instruct the user to remove the sensitive data and add the file to `.gitignore`.

### Note on Enforcement

Enforcement is the **repo owner's responsibility** via pre-commit hooks (e.g., `detect-secrets`, `git-secrets`). This is an agent awareness rule — the agent is the last line of defense before `git commit` runs.

---

## 8. Dirty-File Separation

### Principle

If pre-existing uncommitted changes exist alongside new work when it is time to commit, **commit them separately** before committing the new work.

### Procedure

1. Before committing new work, run `git status`
2. If there are pre-existing modified files unrelated to the current change:
   - Stage and commit those first with their own focused commit message
   - Then stage and commit the new work
3. This preserves history granularity: each commit addresses exactly one logical change

### Why It Matters

Mixing pre-existing changes with new work conflates unrelated intentions in a single commit. Future `git blame` and `git bisect` sessions become harder to reason about. Focused commits enable precise rollback.

### Example

```bash
# Pre-existing: README.md has minor typo fix
# New work: feat(search-index): add incremental reindexing

git add README.md
git commit -m "docs(readme): fix typo in installation section"

git add src/search/
git commit -m "feat(search-index): add incremental reindexing on file save"
```

---

## 9. Validation

### Validate Against Whatever Machinery Was Discovered

Section 2(a) already identified whether the project has commit-linting or release machinery, and — if so — what shape it takes. If it does, validate the drafted message against that same machinery before running `git commit`; don't assume the message is compliant just because it looks conventional.

The pattern: whatever tool was discovered almost always exposes some way to check a message, or preview a release, without actually committing — either a lint subcommand that accepts a message via stdin or a flag, or a dry-run/preview mode that shows how the next commit would be classified. Find that command in the tool's own help or docs and run it against the draft before committing.

To ground this concretely — again illustrative and **non-exhaustive**, not a checklist to match — this can look like piping a subject into a commitlint-style CLI, running a conventional-commit toolkit's own `check`/`verify` subcommand, or invoking a release tool's `--dry-run` flag to see how it would classify the commit. The exact invocation depends entirely on which tool the project actually uses, including tools neither this reference nor the agent has ever seen — read that tool's own docs for its equivalent rather than assuming one of the examples above applies.

Where a config defines lint rule *names*, treat those names as specific to that tool, not universal — a name like `type-enum` (or `scope-enum`, `subject-case`, `subject-max-length`) is one illustration among many possible vocabularies. A different tool encodes the same ideas under different names, or doesn't lint them at all and relies on a dry-run preview instead.

### If No Machinery Exists

Fall back to the pre-write gate command from Section 1: measure and cap subject length yourself, since nothing else will.

```bash
printf '%s' "<drafted subject line>" | wc -c
```

Compare the result against whatever limit applies — the project's own documented limit (Section 2b) if one exists, or the 72-character fallback (Section 1) if it doesn't. Note the absence of machinery to the user, but proceed with a defensible, derived vocabulary. Do not create a commitlint config or equivalent unless the user requests it — enforcement setup is the concern of [`convention-setup.md`](convention-setup.md), not of authoring an individual commit.

---

## 10. Examples

Each path below shows the *process* — discover, read, apply — rather than a gallery of types to memorize.

### Path (a) — vocabulary from project machinery

Discovery: the repo has a release-tooling config at its root whose type-mapping table defines an `ops` type alongside the conventional set. (This is one possible shape a project's machinery can take, chosen here only to make the example concrete — the discovery step is identical no matter which tool the config belongs to.)

Applying the discovered vocabulary directly (no invented types, no guessing):

```
ops(ci-pipeline): pin runner image to avoid upstream breaking change

The floating `latest` tag pulled in a runner update that dropped
Python 3.8 support mid-build. Pinning to a digest keeps builds
reproducible until the project's own Python floor moves.
```

### Path (b) — vocabulary derived from history + CONTRIBUTING

Discovery: no machinery config anywhere. `CONTRIBUTING.md` states: "Use `write`, `revise`, `correct`, `cite`, or `format`, scoped to the manuscript section or dataset affected." `git log --oneline -30` (the wide sample from Section 2b, for vocabulary breadth) confirms every recent commit actually follows this pattern.

Applying the derived vocabulary:

```
correct(cohort-dataset): fix transposed sample-size figures

Sample sizes for the treatment and control arms were swapped in
the exported summary table, understating the study's power. The
source dataset was correct; only the export step had the bug.
```

### Path (c) — greenfield

No worked example is given here: per Section 2(c), a project with neither machinery nor a derivable pattern doesn't get a guessed vocabulary — it gets routed to [`convention-setup.md`](convention-setup.md) to stand one up deliberately, in collaboration with the user. Only once that interview produces a vocabulary does authoring resume here.

Both worked paths above apply the same process — discover what the project already does (config, or history plus docs), read the vocabulary it defines, then write the commit against that vocabulary — rather than reteaching a canonical type list. For how these types map to version bumps once derived, see [`semver-changelog.md`](semver-changelog.md).
