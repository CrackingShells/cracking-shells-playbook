# R01 Content Mapping: committing-changes Skill Synthesis

**Date**: 2026-02-26
**Status**: Final — authoritative spec for skill file authoring

---

## 1. Source Materials

| Source | Path | Role |
|:-------|:-----|:-----|
| Org instructions | `instructions/git-workflow.instructions.md` | Primary — preserve all rules |
| Community research | Conversation context (previous session) | Gap fill — patterns to add |

---

## 2. Org Instruction Inventory

What the org's `git-workflow.instructions.md` already covers:

| Section | Strength | Preserve As-Is? |
|:--------|:---------|:----------------|
| Conventional Commit Format (type/scope/description) | Strong | ✅ Yes |
| Scope = explicit topic identification | **Distinctive** | ✅ Yes (centerpiece) |
| Scope consistency as development narrative | **Distinctive** | ✅ Yes (centerpiece) |
| Scope discovery process (Q: "What is this commit about?") | Strong | ✅ Yes |
| Types table (feat/fix/docs/refactor/test/chore/ci/perf/style) | Complete | ✅ Yes |
| Commitlint integration (mandatory config, `npx commitlint --edit`) | Strong | ✅ Yes |
| Body + footer structure with examples | Good | ✅ Yes |
| WHY directive (mentioned under "During Development") | Present but **understated** | ⚠️ Elevate |
| Pre-commit checklist | Present | ✅ Yes |
| Agent optimization notes | Present | ✅ Yes |
| Sensitive file advisory | **Absent** | ➕ Add |
| History-aware style matching | **Absent** | ➕ Add |
| Dirty file separation (Aider pattern) | **Absent** | ➕ Add |

---

## 3. Community Gap Analysis

Patterns identified from community research **not present** in org file:

### Gap 1 — Sensitive File Advisory
**Source**: commit-commands plugin, community security patterns
**Pattern**: Before committing, inspect staged diff for `.env`, credential files, private keys, and API tokens. Do not commit these; warn and halt if found.
**Framing**: Advisory only — enforcement is repo owner's responsibility via pre-commit hooks. This is an agent awareness rule.
**Add to**: `references/git-workflow.md` § Sensitive File Advisory

### Gap 2 — History-Aware Style Matching
**Source**: alexop.dev, BioErrorLog community patterns
**Pattern**: Before drafting any message, run `git log --oneline -10` to observe existing tone, tense, vocabulary, and scope naming conventions in use in this repo. Match the established style.
**Why it matters**: Prevents scope name drift (e.g., agent picks `auth-service` when the repo uses `auth`).
**Add to**: `references/git-workflow.md` § History-Aware Style Matching; `SKILL.md` workflow step 3

### Gap 3 — Elevated WHY Directive
**Source**: atusy's "why-focused commit messages" pattern
**Current org state**: WHY is mentioned under "During Development → Commit Message Quality → Explain the why not just the what" but is not operationalized as a primary directive.
**Elevation needed**: Move WHY to be the #1 principle of message construction — the description field itself must answer WHY this change exists. Body is elaboration; description captures intent.
**Add to**: `references/git-workflow.md` § WHY-Focused Message Writing (dedicated section); `SKILL.md` workflow primary directive

### Gap 4 — Dirty File Separation
**Source**: Aider auto-commit behavior
**Pattern**: If pre-existing uncommitted changes exist alongside new work when ready to commit, commit the pre-existing changes in a separate focused commit first. New work gets its own commit. This preserves history granularity.
**Add to**: `references/git-workflow.md` § Dirty File Separation; `SKILL.md` workflow step 1

---

## 4. Content-to-File Mapping

### What Goes in `SKILL.md` (lean body, ~60–80 lines)

- **Frontmatter**: `name: committing-changes`, third-person description ≤1024 chars with triggers
- **Primary directive**: WHY over WHAT — leading principle
- **7-step workflow** (imperative, lean — no rule details):
  1. Check for pre-existing dirty files → commit separately if found (dirty-file separation)
  2. Inspect staged diff for secrets/credentials → warn and halt if found (sensitive file advisory)
  3. Read `git log --oneline -10` → observe existing scope names, tone, style (history matching)
  4. Read `references/git-workflow.md` → load full rule set before drafting
  5. Draft commit message: `type(scope): WHY-focused description` (imperative, lowercase, ≤72 chars)
  6. Check for commitlint config → validate draft against it if present
  7. Execute: `git add <files>`, `git commit -m "<message>"`
- **Pointer**: "For complete scope naming rules, narrative consistency principles, and examples → `references/git-workflow.md`"

### What Goes in `references/git-workflow.md` (~150–200 lines)

All detail lives here. SKILL.md references this file with explicit load trigger in step 4.

| Section | Content | Source |
|:--------|:--------|:-------|
| Table of Contents | Required (file >100 lines) | — |
| Conventional Commit Format | Structure, types table, subject rules | Org |
| Scope Naming: Explicit Topic Identification | Core principle, good/bad patterns, discovery Q | Org |
| Scope Consistency and Development Narrative | Narrative principle, good/bad examples | Org |
| WHY-Focused Message Writing | Elevated: description = intent, not just action | Org (elevated) + atusy |
| History-Aware Style Matching | `git log --oneline -10`, match tone/vocab/scopes | Community (new) |
| Sensitive File Advisory | `.env`, credentials, private keys — warn + halt | Community (new) |
| Dirty File Separation | Pre-existing uncommitted work → separate commit | Community / Aider (new) |
| Commitlint Compliance | Locate config, validate, `npx commitlint --edit` | Org |
| Body and Footer Guidelines | When to use, structure, 72-char wrap, footers | Org |
| Concrete Examples | 6–8 examples: feat/fix/docs/refactor/test/chore | Org |

---

## 5. Description Draft

For SKILL.md frontmatter `description` field (third-person, ≤1024 chars, includes what + when):

> Generates and executes a git commit following Conventional Commits format and org narrative conventions. Enforces WHY-focused message writing, explicit-topic scope naming, scope consistency across related commits, and history-aware style matching. Use when the user asks to commit changes, create a commit, or stage and commit work — including slash-command invocation like `/commit` or `/committing-changes`. Also applies when the user has completed a feature, fix, or refactor and needs to record it with a semantically meaningful, commitlint-compliant message.

Character count: ~520 chars ✅

---

## 6. Success Criteria for Skill File Review (Pre-Package)

### SKILL.md
- [ ] `name: committing-changes` (gerund, lowercase, hyphens only, ≤64 chars)
- [ ] Description: third-person, ≤1024 chars, includes what + when
- [ ] Body: ≤500 lines, ≤100 lines preferred
- [ ] Step 4 explicitly says: "Read `references/git-workflow.md` before drafting"
- [ ] WHY directive is the leading principle in the body
- [ ] No detailed rule content duplicated from references/git-workflow.md

### references/git-workflow.md
- [ ] Has table of contents
- [ ] All 10 sections present (see table above)
- [ ] Sensitive file advisory is framed as agent awareness, not enforcement
- [ ] History-aware matching specifies `git log --oneline -10`
- [ ] Dirty file separation explains Aider pattern clearly
- [ ] WHY directive is a dedicated section, elevated from "During Development" note
- [ ] ≥100 lines (consistency check: `wc -l`)
