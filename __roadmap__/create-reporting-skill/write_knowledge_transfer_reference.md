# Write Knowledge Transfer Reference

**Goal**: Create `references/software-knowledge-transfer.md` with guidance and a flexible template for post-cycle knowledge transfer reports.
**Pre-conditions**:
- [ ] Branch `task/write-knowledge-transfer-reference` created from `milestone/create-reporting-skill`
- [ ] `skills/reporting/references/` directory exists
**Success Gates**:
- ✅ `skills/reporting/references/software-knowledge-transfer.md` exists
- ✅ File includes purpose, recommended sections, focus areas, what-not-to-do, and flexible template
**References**:
- [reporting-knowledge-transfer.instructions.md](../../instructions/reporting-knowledge-transfer.instructions.md) — KT reporting guidance
- [reporting-templates.instructions.md](../../instructions/reporting-templates.instructions.md) — Knowledge Transfer template

---

## Step 1: Write software-knowledge-transfer.md

**Goal**: Author the knowledge transfer reference file.

**Implementation Logic**:
1. **Purpose statement**: a KT report captures what happened during a development cycle and what to do differently next time; keep short and practical (~1–2 pages)
2. **Recommended sections** (numbered 1–7): executive summary (what shipped/changed, primary outcomes); wins; pain points (review friction, failure modes, tooling gaps); root causes (why it happened, not just symptoms); next-cycle changes (instruction updates, workflow changes, checklist updates); artifacts to preserve (diagrams, test matrices, decision tables, scripts); open questions (unknowns to validate next cycle)
3. **Focus areas**: prompting and instructions; review process; testing (where did the test plan expand beyond scope and why); documentation (what was unclear or outdated)
4. **What not to do**: do not paste large code blocks; do not rewrite the full architecture — link to analysis stage artifacts instead
5. **Template**: horizontal rule, flexible guidance label ("Adapt sections as needed. Aim for ~1–2 pages"), then the Knowledge Transfer template

**References**:
- [reporting-knowledge-transfer.instructions.md](../../instructions/reporting-knowledge-transfer.instructions.md) — §all
- [reporting-templates.instructions.md](../../instructions/reporting-templates.instructions.md) — § Template: Knowledge Transfer

**Deliverables**: `skills/reporting/references/software-knowledge-transfer.md` (~60 lines)
**Consistency Checks**: `ls skills/reporting/references/software-knowledge-transfer.md` (expected: PASS)
**Commit**: `feat(reporting): add software-knowledge-transfer reference`
