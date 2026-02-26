# Git Workflow Reference

Complete rule set for the `committing-changes` skill. Read this file before drafting any commit message.

## Table of Contents

1. [Conventional Commit Format](#1-conventional-commit-format)
2. [Scope Naming: Explicit Topic Identification](#2-scope-naming-explicit-topic-identification)
3. [Scope Consistency and Development Narrative](#3-scope-consistency-and-development-narrative)
4. [WHY-Focused Message Writing](#4-why-focused-message-writing)
5. [History-Aware Style Matching](#5-history-aware-style-matching)
6. [Sensitive File Advisory](#6-sensitive-file-advisory)
7. [Dirty File Separation](#7-dirty-file-separation)
8. [Commitlint Compliance](#8-commitlint-compliance)
9. [Body and Footer Guidelines](#9-body-and-footer-guidelines)
10. [Concrete Examples](#10-concrete-examples)

---

## 1. Conventional Commit Format

### Structure

```
type(scope): description

[optional body]

[optional footer(s)]
```

**Rules:**
- `scope` is **mandatory** — never omit it
- `description`: imperative mood, lowercase, ≤72 chars
- Blank line between subject and body; blank line between body and footers

### Types

| Type | Meaning | Version Bump |
|:-----|:--------|:-------------|
| `feat` | New feature | Minor |
| `fix` | Bug fix | Patch |
| `docs` | Documentation only | None |
| `refactor` | Code restructuring, no behavior change | None |
| `test` | Add or update tests | None |
| `chore` | Maintenance, dependencies, tooling | None |
| `ci` | CI/CD configuration changes | None |
| `perf` | Performance improvements | Patch |
| `style` | Formatting, whitespace (no logic change) | None |

### Subject Line Rules

- Imperative mood: "add", "fix", "remove" — not "added", "fixes", "removing"
- Lowercase after the colon
- No trailing period
- ≤72 characters total

---

## 2. Scope Naming: Explicit Topic Identification

### Core Principle

The scope must identify the **topic/subject** of the commit — not the action, not the layer, not the result. It answers: *"What is this commit primarily about?"*

### Good Scope Patterns

| Pattern | Examples |
|:--------|:---------|
| Feature names | `kiro`, `codex`, `auth`, `dashboard` |
| Component/module names | `UserService`, `api-client`, `database` |
| Concepts | `rate-limiting`, `unicode-handling`, `error-recovery` |
| Files/modules | `config.yml`, `main.py` |

### Bad Scope Patterns

| Anti-pattern | Why it fails |
|:-------------|:-------------|
| `core` | Too generic — no traceability |
| `refactor` | Describes action, not subject |
| `misc` | Zero information |
| `backend` | Architectural location, not topic |
| `update` | Describes action, not subject |

### Scope Discovery Process

1. Ask: *"What is this commit primarily about?"*
2. Use names from requirements, docs, existing codebase components
3. Check the commitlint config for allowed scopes
4. Pick the most specific name available

---

## 3. Scope Consistency and Development Narrative

### Principle

Related commits **must share the same scope** to form a coherent development story. Scope changes signal a shift in focus — use them intentionally.

### Good Narrative Flow

```
feat(kiro): add MCP server discovery
feat(kiro): implement configuration validation
fix(kiro): handle connection errors
docs(kiro): document configuration options
```

Reading left to right, the commit history tells a story: "We built the Kiro MCP integration."

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
- Consistent naming → traceable git blame and changelog

---

## 4. WHY-Focused Message Writing

### The Primary Directive

**The description must answer WHY this change exists — not just what changed.**

`what` is visible in the diff. `why` is only in the commit message. Future agents and developers reading the history need the intent, not the mechanic.

### Application

| Bad (describes what) | Good (explains why) |
|:---------------------|:--------------------|
| `fix(auth): update token check` | `fix(auth): prevent expired tokens from bypassing session validation` |
| `refactor(UserService): move validation` | `refactor(UserService): isolate validation to enable independent testing` |
| `feat(dashboard): add chart` | `feat(dashboard): expose throughput trend to support capacity planning` |

### When to Elaborate

- Simple change: subject line alone is sufficient
- Complex change: add body to explain context, tradeoffs, or alternatives considered
- Breaking change: body is mandatory — explain migration path

---

## 5. History-Aware Style Matching

### Principle

Before drafting a commit message, read the existing commit history to observe the tone, vocabulary, tense conventions, and scope naming already in use. Match that style.

### How to Do It

```bash
git log --oneline -10
```

Look for:
- **Scope names in use** — if the repo uses `auth` not `authentication`, use `auth`
- **Tense** — does the team use "add X" or "adds X"? Match it
- **Casing** — are scopes `PascalCase` (e.g., `UserService`) or `kebab-case`?
- **Description style** — terse or descriptive?

### Why It Matters

Prevents scope name drift. Without this check, an agent might introduce `auth-service` into a repo that consistently uses `auth`, breaking changelog generation and grep patterns.

---

## 6. Sensitive File Advisory

### Agent Awareness Rule

Before staging and committing, inspect the diff for sensitive content. **Do not commit:**

- `.env` or `.env.*` files
- Files containing patterns: `API_KEY=`, `SECRET=`, `PASSWORD=`, `PRIVATE_KEY`
- `*.pem`, `*.key`, `id_rsa`, `id_ed25519` private key files
- `credentials.json`, `service-account.json`, `aws-credentials`
- Any file with tokens, bearer credentials, or connection strings with passwords

### Procedure

```bash
git diff --staged
```

Scan the staged diff before committing. If sensitive content is found: warn, halt, and instruct the user to remove the sensitive data and add the file to `.gitignore`.

### Note on Enforcement

Enforcement is the **repo owner's responsibility** via pre-commit hooks (e.g., `detect-secrets`, `git-secrets`). This is an agent awareness rule — the agent is the last line of defense before `git commit` runs.

---

## 7. Dirty File Separation

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

## 8. Commitlint Compliance

### Mandatory Check

If a commitlint config exists in the repo, the commit message **must pass validation** before executing `git commit`.

### Config File Locations (check in order)

1. `commitlintrc.json`
2. `.commitlintrc.js`
3. `.commitlintrc.yaml`
4. `.commitlintrc.yml`
5. `commitlint.config.js`

### Validation Workflow

```bash
# After drafting message, validate it
echo "feat(auth): add OAuth2 authentication flow" | npx commitlint

# Or validate the last staged message
npx commitlint --edit
```

### Common Rules Enforced

- `type-enum`: only approved types allowed
- `scope-enum`: scopes must match project-defined list
- `subject-case`: typically `lower-case`
- `subject-max-length`: typically 72 chars
- `header-max-length`: typically 72 chars

### If Config Is Missing

Note it to the user but proceed with the conventional commits format. Do not create a commitlint config unless the user requests it.

---

## 9. Body and Footer Guidelines

### When to Include a Body

- Complex changes requiring context beyond what the subject captures
- Breaking changes (body is mandatory for migration context)
- Changes that close issues or reference external discussions
- Decisions where the "why" needs more than 72 chars

### Body Structure

```
feat(auth): add OAuth2 authentication flow

Add OAuth2 authorization code flow to replace the existing
basic auth implementation. Uses PKCE extension for public
clients to prevent authorization code interception.

Key changes:
- AuthProvider: new OAuth2Client with PKCE support
- TokenStore: secure in-memory token caching
- CLI: --auth-provider flag for provider selection
```

- Blank line between subject and body
- Wrap at 72 chars per line
- Use imperative mood consistently with subject

### Footer Format

```
Resolves #123
Fixes #456
Closes #789

BREAKING CHANGE: The --token flag is removed. Use --auth-provider oauth2 instead.
Migration guide: docs/migration/auth-v2.md
```

- `Resolves`, `Fixes`, `Closes` followed by issue reference
- `BREAKING CHANGE:` on its own line, followed by description
- One footer key per line

---

## 10. Concrete Examples

### feat — New feature with narrative scope

```
feat(kiro): add MCP server discovery protocol

Implements automatic discovery of MCP servers via .kiro/mcp.json
configuration. Enables agents to connect without manual server
registration for each session.
```

### fix — Bug fix with WHY focus

```
fix(codex): prevent duplicate API requests on token refresh

Token refresh triggered concurrent requests when multiple calls
hit a 401 simultaneously. Adds in-flight deduplication to ensure
only one refresh executes and all waiters receive the same token.
```

### docs — Documentation update

```
docs(auth): document OAuth2 PKCE flow and token lifecycle
```

### refactor — Structural change with intent

```
refactor(UserService): extract validation to enable unit testing

Validation logic was embedded in the service layer, making it
impossible to test without a full database connection. Extracted
to UserValidator for isolated testing.
```

### test — Test addition

```
test(auth): add OAuth2 token refresh race condition coverage
```

### chore — Maintenance

```
chore(deps): upgrade commitlint to v19 for Node 22 compatibility
```

### feat with breaking change

```
feat(api)!: replace REST endpoints with GraphQL schema

BREAKING CHANGE: All /api/v1/* REST endpoints are removed.
Clients must migrate to the /graphql endpoint.
Migration guide: docs/migration/graphql-v2.md

Resolves #234
```

### fix with issue reference

```
fix(dashboard): prevent chart overflow on narrow viewports

Resolves #567
```
