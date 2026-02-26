# Write Test Definition Reference

**Goal**: Create `references/software-test-definition.md` with guidance, workflow checklist, and template for software test definition reports.
**Pre-conditions**:
- [ ] Branch `task/write-test-definition-reference` created from `milestone/create-reporting-skill`
- [ ] `skills/reporting/references/` directory exists
**Success Gates**:
- ✅ `skills/reporting/references/software-test-definition.md` exists
- ✅ File opens with a table of contents (required — file exceeds 100 lines)
- ✅ File includes workflow checklist, core principles, anti-explosion strategy, self-review feedback loop, inline test tier definitions, and flexible template
- ✅ No references to "Wobble" or `testing.instructions.md`
**References**:
- [reporting-tests.instructions.md](../../instructions/reporting-tests.instructions.md) — Test definition reporting guidance
- [reporting-templates.instructions.md](../../instructions/reporting-templates.instructions.md) — Test definition template

---

## Step 1: Write software-test-definition.md

**Goal**: Author the test definition reference file, extracting and rewriting content from source instructions with required additions.

**Implementation Logic**:
1. **Table of contents** (required — target ~130 lines): link to all major sections
2. **Workflow checklist**: copyable block ("Copy this checklist and check off items as you work") with 6 steps: (1) list top risks 3–8, (2) map 1–3 tests per risk, (3) apply equivalence classes and boundary sets, (4) self-review each proposed test, (5) draft report sections, (6) save to `__reports__/<topic>/`
3. **Core principles** (condensed from §2): test what we own; risk-driven over exhaustive; observable behavior over implementation details; consolidate aggressively
4. **What to test / what not to test** (condensed from §3): healthy targets as a bullet list; over-testing sources as a bullet list; ownership check one-liner ("If I removed the new/changed code, would this fail?")
5. **Anti-explosion strategy** (condensed from §4): start from risks not functions; equivalence classes + boundary sets; pairwise over cartesian products; parameterize by default; size heuristics (bug fix ~2–3, new feature ~4–6, refactor ~1–2 tests per changed unit)
6. **Self-review feedback loop** (from §5): frame explicitly as "apply to each proposed test; revise any that fail a check before submitting"; present as a checkbox list
7. **Report format** (condensed from §7): executive summary, scope, test matrix table with column headers, fixtures/test data strategy, minimal must-run regression set
8. **Test tiers inline** (replacing broken `testing.instructions.md` reference): development (temporary scaffolding to drive feature work), regression (permanent coverage for critical behaviors), integration (cross-component and end-to-end); include scope labels for integration tests (component | service | end_to_end)
9. **Template**: horizontal rule, flexible guidance label, then the test definition template. Remove all "Wobble" references — generalize to "capture test execution output as log files"

**References**:
- [reporting-tests.instructions.md](../../instructions/reporting-tests.instructions.md) — §2–10 all sections
- [reporting-templates.instructions.md](../../instructions/reporting-templates.instructions.md) — § Template: Test Definition

**Deliverables**: `skills/reporting/references/software-test-definition.md` (~130 lines)
**Consistency Checks**: `wc -l skills/reporting/references/software-test-definition.md` (expected: PASS)
**Commit**: `feat(reporting): add software-test-definition reference`
