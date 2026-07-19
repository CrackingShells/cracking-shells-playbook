# Finalize SKILL.md Dispatcher

**Goal**: Finalize `skills/writing-history/SKILL.md` — a lean dispatcher with a primary directive and a navigation table routing each situation to one of the four references.
**Pre-conditions**:
- [ ] All four `references/*.md` (`commit-authoring`, `convention-setup`, `semver-changelog`, `branches`) merged into the integration branch
**Success Gates**:
- ⬜ `SKILL.md` body is lean (≤100 lines preferred, ≤500 hard) and delegates depth to `references/`
- ⬜ A navigation table maps situations → all four reference files, and every linked file exists
- ⬜ Frontmatter remains valid (`name: writing-history`, description ≤1024 chars, third person)
**References**: [R02 committing-changes SKILL.md](../../../skills/committing-changes/SKILL.md) — dispatcher shape (navigation-table-first, "read before acting" pointers)

## Step 1: Rewrite SKILL.md body with navigation table

**Goal**: Replace the scaffold placeholder body with the production dispatcher.

**Implementation Logic**:
Overwrite the body of `skills/writing-history/SKILL.md` (keep/tighten the frontmatter from scaffold). Include:
1. **Primary Directive** — WHY over WHAT; and the meta-rule that the commit vocabulary is derived, never assumed.
2. **Navigation table** — `| Situation | Reference |` rows routing:
   - author/stage/commit a change → `references/commit-authoring.md`
   - no commit conventions / changelog / versioning yet → `references/convention-setup.md`
   - how versions & changelog derive from commits → `references/semver-changelog.md`
   - merge, rebase, integrate a branch, or parallel/worktree work → `references/branches.md`
3. A brief "read the routed reference before acting" instruction consistent with house style.
Keep it under 100 lines; do not duplicate reference content.

**Deliverables**: `skills/writing-history/SKILL.md` (body headings: `Primary Directive`, a navigation table linking all four `references/*.md`)
**Consistency Checks**: `python3 -c "import yaml,os; d=open('skills/writing-history/SKILL.md').read(); f=yaml.safe_load(d.split('---')[1]); assert f['name']=='writing-history' and len(f['description'])<=1024; body=d.split('---',2)[2]; refs=['commit-authoring.md','convention-setup.md','semver-changelog.md','branches.md']; assert all(r in body for r in refs); assert all(os.path.exists('skills/writing-history/references/'+r) for r in refs); assert body.count(chr(10))<=500; print('PASS')"` (expected: PASS)
**Commit**: `feat(writing-history): finalize SKILL.md dispatcher and navigation table`
