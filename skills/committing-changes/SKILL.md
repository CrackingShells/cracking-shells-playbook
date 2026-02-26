---
name: committing-changes
description: Generates and executes a git commit following Conventional Commits format and org narrative conventions. Enforces WHY-focused message writing, explicit-topic scope naming, scope consistency across related commits, and history-aware style matching. Use when the user asks to commit changes, create a commit, or stage and commit work — including slash-command invocation like /commit or /committing-changes. Also applies when the user has completed a feature, fix, or refactor and needs to record it with a semantically meaningful, commitlint-compliant message.
---

# Committing Changes

## Primary Directive

**WHY over WHAT.** The commit message description must answer *why* this change exists — not just what changed. The diff shows what; only the message captures intent.

## Commit Workflow

Read `references/git-workflow.md` before drafting any commit message. Then execute these steps in order:

**Step 1 — Check for dirty pre-existing files**
Run `git status`. If modified files unrelated to the current change are present, commit them first in a separate focused commit before staging new work. Each commit must address exactly one logical change.

**Step 2 — Inspect staged diff for secrets**
Run `git diff --staged` and scan for `.env` files, API keys, passwords, private key material (`*.pem`, `*.key`, `id_rsa`), or credential files. If found: warn, halt, and advise the user to remove the sensitive content and add the file to `.gitignore`.

**Step 3 — Read commit history**
Run `git log --oneline -10`. Observe the scope names, tense, casing, and description style already in use. Match that style — do not introduce new scope naming patterns without checking existing conventions first.

**Step 4 — Read `references/git-workflow.md`**
Load the full rule set before drafting. This file contains: scope naming rules, scope consistency principles, WHY directive, sensitive file advisory, dirty file separation, commitlint compliance, and concrete examples.

**Step 5 — Draft the commit message**
Format: `type(scope): WHY-focused description`
- Scope is **mandatory** and must identify the explicit topic (feature name, component, concept)
- Description: imperative mood, lowercase, ≤72 chars
- Add body if the change is complex, breaking, or needs context beyond 72 chars
- Add footers for issue references (`Resolves #123`) or breaking changes (`BREAKING CHANGE:`)

**Step 6 — Check commitlint config**
Look for `commitlintrc.json`, `.commitlintrc.js`, `.commitlintrc.yaml`, or `commitlint.config.js` in the project root. If found, validate the draft: `echo "<message>" | npx commitlint`. Fix any violations before proceeding.

**Step 7 — Execute**
```bash
git add <intended files>
git commit -m "<message>"
```

---

For complete scope naming rules, narrative consistency principles, sensitive file advisory details, dirty file separation procedure, and examples → `references/git-workflow.md`
