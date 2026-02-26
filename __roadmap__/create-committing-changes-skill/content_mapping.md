# Content Mapping Analysis

**Goal**: Produce a written synthesis specification mapping `instructions/git-workflow.instructions.md` against community git commit skill patterns, defining exactly what each skill file must contain.

**Pre-conditions**:
- [ ] `instructions/git-workflow.instructions.md` is readable
- [ ] Community git skill research findings are available in context (catalogue of Claude Code, MCP, Aider, aicommits, opencommit, LangChain, SKILL.md standard)

**Success Gates**:
- ✅ `__reports__/create-committing-changes-skill/content-mapping.md` exists
- ✅ Report documents: org strengths to preserve, community gaps to add, content-to-file mapping (what goes in SKILL.md vs references/git-workflow.md)
- ✅ Report explicitly calls out any org rules that conflict with or supersede community patterns

**References**: [R01 Synthesis Map](../../__reports__/create-committing-changes-skill/content-mapping.md) — output of this task

---

## Step 1: Map org instructions against community patterns and write synthesis report

**Goal**: Produce the content-mapping report that serves as the authoritative specification for all subsequent skill authoring.

**Implementation Logic**:

Read `instructions/git-workflow.instructions.md` in full. Cross-reference each section against the five gap areas identified in research:

1. **Sensitive file advisory** — org file has no mention; community skills (commit-commands plugin) block `.env`, credentials, secrets. Org owns pre-commit hook enforcement; skill adds a WHY warning reminding agents to inspect staged files for secrets before committing.

2. **History-aware style matching** — org file establishes scope consistency as a narrative principle but does not instruct agents to read recent `git log` at commit time. Community pattern (alexop.dev, BioErrorLog): inject `git log --oneline -10` dynamically to match existing tone, tense, and style. Add to skill workflow.

3. **WHY-focused message directive** — org file buries "explain the why not just the what" under Commit Message Best Practices §Body Content. Community (atusy): makes WHY the primary directive for the description field, not just the body. Elevate in SKILL.md as the top instruction.

4. **Precondition framing** — org file's Pre-Commit Checklist is a manual checklist. Community (atusy): enforces tests-pass and lint-clean as hard gates. User decision: tests/lint are repo-owner responsibility via pre-commit hooks; skill adds advisory framing, not enforcement.

5. **Dirty file separation** — org file has no mention. Aider pattern: pre-existing uncommitted changes should be committed separately before new work, preserving history granularity. Add as an agent decision rule in skill workflow.

Content-to-file mapping:
- **SKILL.md body**: Lean workflow (5–8 steps), top-level WHY directive, sensitive file check note, dirty-file separation rule, pointer to references/git-workflow.md
- **references/git-workflow.md**: Full conventional commits format spec, scope naming rules, scope consistency/narrative principle, history-aware matching instruction, commitlint compliance, body/footer templates, examples

Produce the report at `__reports__/create-committing-changes-skill/content-mapping.md`.

**Deliverables**: `__reports__/create-committing-changes-skill/content-mapping.md` (~80–120 lines)

**Consistency Checks**: `test -f __reports__/create-committing-changes-skill/content-mapping.md && echo PASS || echo FAIL` (expected: PASS)

**Commit**: `docs(committing-changes): add content mapping analysis for skill synthesis`
