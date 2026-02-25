# Write SKILL.md

**Goal**: Create the `reporting` skill entry point with frontmatter, core contract, naming convention, and reference navigation table.
**Pre-conditions**:
- [ ] Branch `task/write-skill-md` created from `milestone/create-reporting-skill`
- [ ] `skills/reporting/` does not yet exist
**Success Gates**:
- ✅ `skills/reporting/SKILL.md` exists
- ✅ Frontmatter has `name: reporting` and a non-empty third-person `description`
- ✅ Navigation table links to all three reference file paths
**References**:
- [reporting.instructions.md](../../instructions/reporting.instructions.md) — Core contract, naming rules, and location patterns
- [reporting-structure.instructions.md](../../instructions/reporting-structure.instructions.md) — Naming patterns and directory conventions

---

## Step 1: Scaffold directory and write SKILL.md

**Goal**: Create `skills/reporting/` with `references/` subdirectory and write `SKILL.md`.

**Implementation Logic**:
1. Create `skills/reporting/references/` (`mkdir -p`)
2. Write `skills/reporting/SKILL.md` with:
   - **Frontmatter**: `name: reporting`; `description` in third person covering all three trigger contexts (architecture analysis, test definition, knowledge transfer) with explicit "Use when:" clause; max 1024 chars; no XML tags
   - **Core contract section**: model-first principle (lead with diagrams/schemas/contracts, never raw code or data); no raw dumps rule (no full class/module/function bodies); stakeholder-reviewable requirement (prefer tables, diagrams, short sections)
   - **File locations and naming section**: table mapping artifact type → location pattern (`__reports__/`, `__design__/`, `__roadmap__/`); naming pattern `<topic>/<round>-<name>_v<version>.md` with field definitions and a concrete example; auto-create rule; README convention per topic folder
   - **Navigation table**: three rows mapping situation → reference file path, each linking directly to `references/<filename>.md`
   - **Note**: for complex multi-phase efforts surfaced during analysis, use the `managing-roadmaps` skill

**References**:
- [reporting.instructions.md](../../instructions/reporting.instructions.md) — Core contract, naming rules, and location patterns
- [reporting-structure.instructions.md](../../instructions/reporting-structure.instructions.md) — Naming patterns and directory conventions

**Deliverables**: `skills/reporting/SKILL.md` (~55 lines), `skills/reporting/references/` (empty directory)
**Consistency Checks**: `ls skills/reporting/SKILL.md skills/reporting/references/` (expected: PASS)
**Commit**: `feat(reporting): scaffold skill directory and write SKILL.md`
