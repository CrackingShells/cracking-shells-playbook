# Initialize Skill Directory

**Goal**: Scaffold the `committing-changes` skill directory at `~/.claude/skills/committing-changes/` using `init_skill.py`.

**Pre-conditions**:
- [ ] `~/.claude/skills/skill-creator/scripts/init_skill.py` exists

**Success Gates**:
- ✅ `~/.claude/skills/committing-changes/` directory exists
- ✅ `SKILL.md` template file present (will be overwritten in implement/)
- ✅ `scripts/`, `references/`, `assets/` example subdirectories created by init script

**References**: [R01 Synthesis Map](../../__reports__/create-committing-changes-skill/content-mapping.md) — not required for this step; directory scaffold is independent of content decisions

---

## Step 1: Run init_skill.py to scaffold the skill directory

**Goal**: Create the standard skill directory layout with template files.

**Implementation Logic**:

Run `init_skill.py` with the skill name `committing-changes` and output path `~/.claude/skills/`. The script creates:
- `committing-changes/SKILL.md` with YAML frontmatter TODO placeholders
- `committing-changes/scripts/` with example script
- `committing-changes/references/` with example reference file
- `committing-changes/assets/` with example asset

After running, delete the example files in `scripts/`, `assets/`, and the example `references/` file — this skill uses only `references/git-workflow.md` (authored in implement/). Keep the generated `SKILL.md` as a starting template for the implement/ phase.

**Deliverables**: `~/.claude/skills/committing-changes/` directory (~5 template files from init script)

**Consistency Checks**: `ls ~/.claude/skills/committing-changes/SKILL.md && echo PASS || echo FAIL` (expected: PASS)

**Commit**: `chore(committing-changes): initialize skill directory scaffold`
