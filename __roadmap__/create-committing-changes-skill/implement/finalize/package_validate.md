# Package and Validate Skill

**Goal**: Run `package_skill.py` to validate and package `committing-changes` into a distributable `.skill` file.

**Pre-conditions**:
- [ ] `~/.claude/skills/committing-changes/SKILL.md` passes manual review (correct frontmatter, lean body, no placeholder TODOs)
- [ ] `~/.claude/skills/committing-changes/references/git-workflow.md` passes manual review (all sections present, table of contents, no duplicate content with SKILL.md)
- [ ] Init-generated example files removed from `scripts/` and `assets/` (or those dirs deleted if unused)

**Success Gates**:
- ✅ `package_skill.py` exits with code 0 (validation passed)
- ✅ `committing-changes.skill` file exists in the output directory
- ✅ Unzipping the `.skill` file shows expected structure: `committing-changes/SKILL.md` + `committing-changes/references/git-workflow.md`

**References**: [R01 Synthesis Map](../../../../__reports__/create-committing-changes-skill/content-mapping.md) — skill content success criteria for pre-package review

---

## Step 1: Run package_skill.py; fix any validation errors; confirm output

**Goal**: Produce a valid `committing-changes.skill` distribution file.

**Implementation Logic**:

Run the packaging script targeting the skill directory. If validation errors are reported, fix them iteratively:

- **Frontmatter errors** (missing fields, name format violation, description too long) → edit `SKILL.md` frontmatter
- **Description quality errors** (vague, missing triggers, wrong person) → rewrite description per official guidance: third-person, ≤1024 chars, includes what + when
- **File reference errors** (broken links to references/) → verify file paths are correct
- **Naming convention errors** (uppercase, underscores in skill name) → verify `name: committing-changes` uses only lowercase letters and hyphens

After validation passes, confirm the output `.skill` file structure with `unzip -l`.

Deliver the `.skill` file to the user by reporting its path.

**Deliverables**: `committing-changes.skill` (output of package_skill.py, location determined by script default or `./dist/`)

**Consistency Checks**: `unzip -l committing-changes.skill | grep -q 'SKILL.md' && echo PASS || echo FAIL` (expected: PASS)

**Commit**: `chore(committing-changes): package and distribute skill`
