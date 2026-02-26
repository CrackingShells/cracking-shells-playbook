# Write git-workflow Reference File

**Goal**: Author `~/.claude/skills/committing-changes/references/git-workflow.md` — the detailed rule set synthesizing the org's git workflow conventions with community patterns.

**Pre-conditions**:
- [ ] `__reports__/create-committing-changes-skill/content-mapping.md` exists and is read
- [ ] `~/.claude/skills/committing-changes/references/` directory exists

**Success Gates**:
- ✅ `~/.claude/skills/committing-changes/references/git-workflow.md` exists
- ✅ File contains all sections defined in content-mapping report: conventional commits spec, scope naming, scope consistency narrative, WHY directive, history-aware style matching, sensitive file advisory, dirty file separation, commitlint compliance, body/footer templates, concrete examples
- ✅ File has a table of contents (>100 lines expected)
- ✅ No content duplicated between this file and SKILL.md body

**References**: [R01 Synthesis Map](../../../__reports__/create-committing-changes-skill/content-mapping.md) — authoritative content spec for this file

---

## Step 1: Read synthesis report and write references/git-workflow.md

**Goal**: Produce the detailed reference file covering all git workflow rules agents need when committing.

**Implementation Logic**:

Read `__reports__/create-committing-changes-skill/content-mapping.md` first to confirm the exact content spec. Then write `references/git-workflow.md` with the following sections (in order):

1. **Table of Contents** — required (file will exceed 100 lines)

2. **Conventional Commit Format** — structure (`type(scope): description`), scope is mandatory, types table (feat/fix/docs/refactor/test/chore/ci/perf/style), subject rules (imperative mood, lowercase, ≤72 chars)

3. **Scope Naming: Explicit Topic Identification** — core principle from org: scope must identify the *topic/subject* not the action. Good patterns (feature names, component names, concepts). Bad patterns (core, refactor, misc, backend). Scope discovery process (question: "What is this commit primarily about?").

4. **Scope Consistency and Development Narrative** — org's narrative principle: related commits must share scope to tell a coherent story. Good/bad narrative flow examples from org's instructions.

5. **WHY-Focused Message Writing** — elevated from community (atusy): the description field must answer WHY this change exists, not just what changed. Body is for elaboration; description itself should capture intent.

6. **History-Aware Style Matching** — community addition: before writing a message, read `git log --oneline -10` to observe existing tone, tense, vocabulary, and scope naming in use. Match the established style.

7. **Sensitive File Advisory** — community addition: before committing, inspect staged files for `.env`, credential files, private keys, or secrets. Do not commit these. Note: enforcement is the repo owner's responsibility via pre-commit hooks; this is an agent awareness rule.

8. **Dirty File Separation** — community addition (Aider pattern): if pre-existing uncommitted changes exist alongside new work, commit the pre-existing changes separately first to preserve history granularity. New work gets its own focused commit.

9. **Commitlint Compliance** — org requirement: locate and read `commitlintrc.json` / `.commitlintrc.js` / `.commitlintrc.yaml` if present. Follow all rules. Run `npx commitlint --edit` if available.

10. **Body and Footer Guidelines** — when to include body (complex changes, breaking changes, issue references), body structure, footer format (`Resolves #123`, `BREAKING CHANGE:`), 72-char wrap.

11. **Concrete Examples** — 6–8 examples covering feat, fix, docs, refactor, test, chore with scopes matching org naming conventions.

**Deliverables**: `~/.claude/skills/committing-changes/references/git-workflow.md` (~150–200 lines)

**Consistency Checks**: `wc -l ~/.claude/skills/committing-changes/references/git-workflow.md | awk '{print ($1 > 50) ? "PASS" : "FAIL"}'` (expected: PASS)

**Commit**: `feat(committing-changes): add git workflow reference with synthesized conventions`
