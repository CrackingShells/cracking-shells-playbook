# Author commit-authoring Reference

**Goal**: Author `skills/writing-history/references/commit-authoring.md` by migrating the durable commit-authoring rules from the source skill, generalizing them to code *and* documents, and **replacing the hardcoded type table with vocabulary derivation**.
**Pre-conditions**:
- [ ] `scaffold_skill_package` merged (skill dir + `references/` exist)
- [ ] Source `skills/committing-changes/references/git-workflow.md` is readable
**Success Gates**:
- ⬜ `references/commit-authoring.md` exists and preserves the durable directives (WHY-over-WHAT, explicit-topic scope naming, scope-consistency narrative, history-aware style, sensitive-file advisory, dirty-file separation, commitlint validation)
- ⬜ It documents the vocabulary-derivation precedence and contains NO table presented as the skill's own authoritative `feat/fix/...` type set
- ⬜ It routes to `convention-setup.md` (greenfield) and `semver-changelog.md` (bump semantics)
**References**: [R01 git-workflow.md](../../../skills/committing-changes/references/git-workflow.md) — source content to migrate/generalize

## Step 1: Write commit-authoring.md with derived vocabulary

**Goal**: Produce the migrated, generalized, vocabulary-agnostic commit-authoring reference.

**Implementation Logic**:
Read `skills/committing-changes/references/git-workflow.md` and `skills/committing-changes/SKILL.md`. Author `skills/writing-history/references/commit-authoring.md` with these sections:
1. **Primary Directive** — WHY over WHAT (message answers why the change exists; diff shows what).
2. **Deriving the Vocabulary** (replaces the old fixed types table) — precedence:
   (a) **Project machinery** — if a commitlint / semantic-release config exists, its `type-enum`/`scope-enum` IS the authorized vocabulary (ground truth); read and obey it.
   (b) **Not-our-project** — when contributing to a project that isn't ours, derive conventions from existing `git log` history AND any `CONTRIBUTING`/commit-message guidelines in the project's docs.
   (c) **Greenfield** — if neither exists, route to `convention-setup.md` to run the setup interview.
3. **Scope Naming: Explicit Topic** — good/bad scope tables (migrate); scope answers "what is this about"; works for document topics too.
4. **Scope Consistency & Development Narrative** — related commits share a scope (migrate examples; add a document-authoring example).
5. **History-Aware Style Matching** — `git log --oneline -10` to match tense/casing/scope style.
6. **Sensitive-File Advisory** — migrate the do-not-commit list + halt procedure.
7. **Dirty-File Separation** — migrate.
8. **Validation** — if a commitlint config exists, validate (`npx commitlint`); note enforcement is repo-owner's job.
9. **Examples** — worked examples for BOTH a code repo and a document/scientific repo (each using its own derived vocabulary, not an imposed one).
Link sibling references by relative path.

**Deliverables**: `skills/writing-history/references/commit-authoring.md` (headings: `Primary Directive`, `Deriving the Vocabulary`, `Scope Naming`, `Scope Consistency`, `History-Aware Style Matching`, `Sensitive-File Advisory`, `Dirty-File Separation`, `Validation`, `Examples`)
**Consistency Checks**: `python3 -c "t=open('skills/writing-history/references/commit-authoring.md').read(); assert 'type-enum' in t and 'CONTRIBUTING' in t and 'convention-setup.md' in t and 'semver-changelog.md' in t and 'WHY' in t.upper(); print('PASS')"` (expected: PASS)
**Commit**: `feat(writing-history): add commit-authoring reference with derived vocabulary`
