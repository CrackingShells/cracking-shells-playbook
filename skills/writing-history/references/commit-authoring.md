# Commit Authoring Reference

Complete rule set for authoring a single commit message. Read this file before drafting any commit.

## Table of Contents

1. [Primary Directive](#1-primary-directive)
2. [Deriving the Vocabulary](#2-deriving-the-vocabulary)
3. [Scope Naming: Explicit Topic Identification](#3-scope-naming-explicit-topic-identification)
4. [Scope Consistency and Development Narrative](#4-scope-consistency-and-development-narrative)
5. [History-Aware Style Matching](#5-history-aware-style-matching)
6. [Sensitive-File Advisory](#6-sensitive-file-advisory)
7. [Dirty-File Separation](#7-dirty-file-separation)
8. [Validation](#8-validation)
9. [Examples](#9-examples)

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
- `description`: imperative mood, lowercase, ≤72 chars, WHY-focused
- Blank line between subject and body; blank line between body and footers

### Subject Line Rules

- Imperative mood: "add", "fix", "remove" — not "added", "fixes", "removing"
- Lowercase after the colon
- No trailing period
- ≤72 characters total

---

## 2. Deriving the Vocabulary

**This skill does not impose a fixed `feat`/`fix`/`docs`/... type set.** The set of valid `type` values (and, where applicable, valid `scope` values) is never hardcoded here — it is *derived* from the project being committed to, in the following order of precedence.

### (a) Project machinery is ground truth

If the repository has commit-linting or release-automation configuration — a commitlint config, a `semantic-release` config, or equivalent — its `type-enum` and `scope-enum` rules **are the authorized vocabulary**. Read the config and obey it exactly; do not substitute a type or scope the config does not list, even if it looks conventional.

Config file locations to check, in order:

1. `commitlintrc.json`
2. `.commitlintrc.js` / `.commitlintrc.cjs` / `.commitlintrc.yaml` / `.commitlintrc.yml`
3. `commitlint.config.js`
4. A `.releaserc*` or `release.config.js` file whose plugin list references `@commitlint/*` or defines commit-analyzer rules

If a `type-enum` rule is present, extract its allowed list and use only those values. If a `scope-enum` rule is present, do the same for scopes.

### (b) Not-our-project: derive from history and docs

When contributing to a project that is not the agent's or user's own — a third-party repository, an external open-source project, a client's existing codebase — and no machinery is found, **do not invent a vocabulary**. Derive it from what the project already does:

- Run `git log --oneline -30` (or more) and tabulate the `type(scope):` prefixes actually in use. Treat the observed set as authoritative, including any project-specific types (e.g. a documentation project using `revise` or `errata` instead of `fix`).
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

The scope must identify the **topic/subject** of the commit — not the action, not the layer, not the result. It answers: *"What is this commit primarily about?"* This holds for code changes and for document changes alike: a scope names the topic a manuscript section, dataset, or protocol addresses, just as it names a feature or module in code.

### Good Scope Patterns

| Pattern | Code examples | Document examples |
|:--------|:---------------|:-------------------|
| Feature/topic names | `kiro`, `codex`, `auth`, `dashboard` | `methods`, `results-fig3`, `discussion` |
| Component/module names | `UserService`, `api-client`, `database` | `protocol`, `dataset-schema`, `appendix-b` |
| Concepts | `rate-limiting`, `unicode-handling` | `statistics`, `citation-style`, `figure-captions` |
| Files/modules | `config.yml`, `main.py` | `manuscript.tex`, `README.md` |

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
2. Use names from requirements, docs, existing codebase components, or — for document repos — section/figure/dataset names
3. Check the derived vocabulary (Section 2) for an allowed-scope list (`scope-enum`) before inventing a new one
4. Pick the most specific name available

---

## 4. Scope Consistency and Development Narrative

### Principle

Related commits **must share the same scope** to form a coherent development story. Scope changes signal a shift in focus — use them intentionally.

### Good Narrative Flow (code)

```
feat(kiro): add MCP server discovery
feat(kiro): implement configuration validation
fix(kiro): handle connection errors
docs(kiro): document configuration options
```

Reading left to right, the commit history tells a story: "We built the Kiro MCP integration."

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
feat(kiro): add MCP server discovery
feat(config): update settings format
fix(connection): handle timeout
chore: update dependencies
```

The story is fragmented — no coherent narrative about what was built.

### Key Rules

- Related commits → same scope
- Scope changes → intentional shift in focus
- Inconsistent scopes → broken development story
- Consistent naming → traceable git blame and changelog, for code and prose alike

---

## 5. History-Aware Style Matching

### Principle

Before drafting a commit message, read the existing commit history to observe the tone, vocabulary, tense conventions, and scope naming already in use. Match that style — this is what Section 2(b) calls deriving from history, applied at the level of phrasing rather than vocabulary set.

### How to Do It

```bash
git log --oneline -10
```

Look for:
- **Scope names in use** — if the repo uses `auth` not `authentication`, use `auth`; if a document repo uses `methods` not `methodology`, use `methods`
- **Tense** — does the project use "add X" or "adds X"? Match it
- **Casing** — are scopes `PascalCase` (e.g., `UserService`) or `kebab-case`?
- **Description style** — terse or descriptive?

### Why It Matters

Prevents scope name drift. Without this check, an agent might introduce `auth-service` into a repo that consistently uses `auth`, or `methodology` into a repo that consistently uses `methods` — breaking changelog generation and grep patterns either way.

---

## 6. Sensitive-File Advisory

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

## 7. Dirty-File Separation

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
# New work: feat(auth): add OAuth2 flow

git add README.md
git commit -m "docs(readme): fix typo in installation section"

git add src/auth/
git commit -m "feat(auth): add OAuth2 authentication flow"
```

---

## 8. Validation

### Mandatory Check When Machinery Exists

If a commitlint (or equivalent) config exists in the repo — per the Section 2(a) precedence — the commit message **must pass validation** before executing `git commit`.

### Validation Workflow

```bash
# After drafting message, validate it
echo "feat(auth): add OAuth2 authentication flow" | npx commitlint

# Or validate the last staged message
npx commitlint --edit
```

### Common Rules Enforced

- `type-enum`: only the config's approved types allowed
- `scope-enum`: scopes must match the project-defined list
- `subject-case`: typically `lower-case`
- `subject-max-length`: typically 72 chars
- `header-max-length`: typically 72 chars

### If Config Is Missing

Fall back to Section 2(b)/2(c): derive from history/`CONTRIBUTING`, or route to `convention-setup.md`. Note the absence of machinery to the user, but proceed with a defensible, derived vocabulary. Do not create a commitlint config unless the user requests it — enforcement setup is the concern of [`convention-setup.md`](convention-setup.md), not of authoring an individual commit.

---

## 9. Examples

### Example set A — code repository (vocabulary from project machinery)

Project has a commitlint config with `type-enum: [feat, fix, docs, refactor, test, chore, ci, perf, style]`.

```
feat(kiro): add MCP server discovery protocol

Implements automatic discovery of MCP servers via .kiro/mcp.json
configuration. Enables agents to connect without manual server
registration for each session.
```

```
fix(codex): prevent duplicate API requests on token refresh

Token refresh triggered concurrent requests when multiple calls
hit a 401 simultaneously. Adds in-flight deduplication to ensure
only one refresh executes and all waiters receive the same token.
```

```
refactor(UserService): extract validation to enable unit testing

Validation logic was embedded in the service layer, making it
impossible to test without a full database connection. Extracted
to UserValidator for isolated testing.
```

```
feat(api)!: replace REST endpoints with GraphQL schema

BREAKING CHANGE: All /api/v1/* REST endpoints are removed.
Clients must migrate to the /graphql endpoint.
Migration guide: docs/migration/graphql-v2.md

Resolves #234
```

### Example set B — scientific/document repository (vocabulary derived from history + CONTRIBUTING)

Project has no commitlint config. `CONTRIBUTING.md` states: "Use `write`, `revise`, `correct`, `cite`, or `format` as the commit type, scoped to the manuscript section." `git log --oneline -10` confirms this pattern is actually followed.

```
write(results): report primary endpoint effect size

Adds the effect-size estimate and 95% CI for the primary endpoint
now that the blinded analysis is complete.
```

```
correct(methods): fix transposed sample-size figures

Sample sizes for the treatment and control arms were swapped in
the power-analysis paragraph, understating the study's power.
```

```
cite(discussion): add reference to 2025 replication study

The replication study directly supports the claim in paragraph 3
and was published after the initial draft was written.
```

```
revise(abstract): tighten framing per reviewer 2 feedback

Reviewer 2 noted the original framing overstated generalizability;
narrows the claim to the studied population.
```

Both example sets use a vocabulary *derived* per Section 2 — the code repo's from its `type-enum` config, the document repo's from its `CONTRIBUTING.md` and observed history — rather than a vocabulary imposed by this reference. When neither source exists, route to [`convention-setup.md`](convention-setup.md) before drafting any message. For how these types map to version bumps once derived, see [`semver-changelog.md`](semver-changelog.md).
