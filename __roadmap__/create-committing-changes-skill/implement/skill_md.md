# Write SKILL.md

**Goal**: Author `~/.claude/skills/committing-changes/SKILL.md` — lean frontmatter and workflow body that triggers the skill and guides the commit sequence.

**Pre-conditions**:
- [ ] `__reports__/create-committing-changes-skill/content-mapping.md` exists and is read
- [ ] `~/.claude/skills/committing-changes/SKILL.md` template exists (from init_skill.md)

**Success Gates**:
- ✅ Frontmatter: `name: committing-changes` (gerund form, ≤64 chars, lowercase + hyphens only)
- ✅ Frontmatter: `description` is in third person, ≤1024 chars, includes what the skill does AND when to use it
- ✅ Body: ≤500 lines, ≤100 lines preferred
- ✅ Body: references `references/git-workflow.md` with an explicit load trigger ("Read references/git-workflow.md before committing")
- ✅ Body: covers the 5-step workflow with WHY directive, dirty-file check, sensitive file advisory, history check, and commit execution
- ✅ No detailed rule content duplicated from references/git-workflow.md

**References**: [R01 Synthesis Map](../../../__reports__/create-committing-changes-skill/content-mapping.md) — content-to-file mapping, description language guidance

---

## Step 1: Read synthesis report then overwrite the template SKILL.md

**Goal**: Produce the production SKILL.md replacing the init_skill.py placeholder content.

**Implementation Logic**:

Read `__reports__/create-committing-changes-skill/content-mapping.md`. Then overwrite `~/.claude/skills/committing-changes/SKILL.md` with:

**Frontmatter**:
- `name: committing-changes` — gerund form per official best practices
- `description`: Third-person, includes triggers. Draft: *"Generates and executes a git commit following Conventional Commits format and org narrative conventions. Use when the user asks to commit changes, create a commit, or stage and commit work — including slash-command invocation like /commit or /committing-changes."*

**Body — Commit Workflow** (lean, ~50–70 lines):

Primary directive: WHY over WHAT — the commit message must answer why this change exists, not just describe what changed.

Workflow steps:
1. **Check for pre-existing dirty files** — if uncommitted changes exist alongside new work, commit the pre-existing changes separately first (dirty-file separation rule)
2. **Inspect staged files for secrets** — scan staged diff for `.env`, credential patterns, private keys; warn and halt if found (sensitive file advisory)
3. **Read commit history** — run `git log --oneline -10` to observe existing scope names, tone, and message style in this repo
4. **Read `references/git-workflow.md`** — load the full rule set before drafting any message
5. **Draft the commit message** — subject: `type(scope): WHY-focused description` (imperative, lowercase, ≤72 chars); add body if change is complex or breaking; add footers for issue refs or breaking changes
6. **Check commitlint config** — if `commitlintrc.json` / `.commitlintrc.js` / `.commitlintrc.yaml` exists, validate the draft message against it
7. **Execute** — `git add` the intended files, `git commit -m "<message>"`

Include a note: "For complete scope naming rules, narrative consistency principles, and examples → `references/git-workflow.md`"

**Deliverables**: `~/.claude/skills/committing-changes/SKILL.md` (~60–80 lines)

**Consistency Checks**: `python3 -c "import yaml; d=open('$HOME/.claude/skills/committing-changes/SKILL.md').read(); front=d.split('---')[1]; y=yaml.safe_load(front); assert y.get('name')=='committing-changes'; assert len(y.get('description',''))<=1024; print('PASS')"` (expected: PASS)

**Commit**: `feat(committing-changes): add SKILL.md with lean commit workflow`
