# Package and Validate Skill

**Goal**: Run the skill packaging script, fix any validation errors, and produce `dist/reporting.skill`.
**Pre-conditions**:
- [ ] Branch `task/package-and-validate` created from `milestone/create-reporting-skill`
- [ ] All four root-level authoring tasks complete
- [ ] `skills/reporting/SKILL.md` and all three reference files exist
**Success Gates**:
- ✅ `package_skill.py` runs without validation errors
- ✅ `dist/reporting.skill` produced
**References**:
- Skill-creator skill — SKILL.md validation rules and packaging command

---

## Step 1: Run package script and resolve any validation errors

**Goal**: Produce a valid, distributable `dist/reporting.skill` file.

**Implementation Logic**:
1. Run the packaging script:
   `python ~/.claude/plugins/cache/anthropic-agent-skills/example-skills/1ed29a03dc85/skills/skill-creator/scripts/package_skill.py skills/reporting/ dist/`
2. If validation fails, read the error output and identify the violated rule. Most likely causes:
   - `name`: max 64 chars, lowercase letters/numbers/hyphens only, no reserved words
   - `description`: max 1024 chars, no XML tags, must be written in third person
   - Extraneous files in skill directory (no README.md, CHANGELOG.md, etc. at root of skill)
3. Fix the offending file (most likely `skills/reporting/SKILL.md` frontmatter) and re-run
4. Repeat until packaging succeeds and `dist/reporting.skill` is produced

**References**:
- Skill-creator skill — packaging script path and SKILL.md frontmatter validation rules

**Deliverables**: `dist/reporting.skill`
**Consistency Checks**: `ls dist/reporting.skill` (expected: PASS)
**Commit**: `feat(reporting): package and distribute reporting skill`
