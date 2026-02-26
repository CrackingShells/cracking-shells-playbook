# Write Architecture Reference

**Goal**: Create `references/software-architecture.md` with guidance and a flexible template for software architecture analysis reports.
**Pre-conditions**:
- [ ] Branch `task/write-architecture-reference` created from `milestone/create-reporting-skill`
- [ ] `skills/reporting/references/` directory exists (created by `write_skill_md` task)
**Success Gates**:
- ✅ `skills/reporting/references/software-architecture.md` exists
- ✅ File includes required artifacts list, contracts & invariants guidance, alternatives format, roadmap handoff note, and flexible template
**References**:
- [reporting-architecture.instructions.md](../../instructions/reporting-architecture.instructions.md) — Architecture reporting guidance
- [reporting-templates.instructions.md](../../instructions/reporting-templates.instructions.md) — Architecture analysis template

---

## Step 1: Write software-architecture.md

**Goal**: Author the architecture analysis reference file, extracting and rewriting content from source instructions.

**Implementation Logic**:
1. Open with framing: produce at the analysis stage, before writing implementation code
2. **Required artifacts** (4 items): (1) at least one Mermaid diagram — choose the type that best communicates the change; (2) contracts & invariants; (3) alternatives considered; (4) risk register. Drop the diagram type selection guide (graph TD vs sequenceDiagram etc.) — Claude already knows Mermaid
3. **Contracts & invariants section**: prefer interfaces over implementations; include input/output schemas, error model, invariants, pseudo-code for non-trivial logic only; explicitly forbid full class/module/function bodies
4. **Alternatives & decisions section**: for each major decision — options considered, why chosen approach wins, tradeoffs accepted
5. **Roadmap handoff section**: if complex or multi-phase, end the report with a handoff using the `managing-roadmaps` skill — replaces the broken `roadmap-generation.instructions.md` link from source
6. **Template section**: horizontal rule separator, label as flexible guidance ("Adapt sections as needed"), then the architecture analysis template: Executive Summary, Current State (mermaid), Proposed State (mermaid), Key Flows (mermaid), Contracts & Invariants, Alternatives Considered (table), Risks & Mitigations (table), Roadmap Recommendation

**References**:
- [reporting-architecture.instructions.md](../../instructions/reporting-architecture.instructions.md) — §1–5 all sections
- [reporting-templates.instructions.md](../../instructions/reporting-templates.instructions.md) — § Template: Architecture Analysis

**Deliverables**: `skills/reporting/references/software-architecture.md` (~80 lines)
**Consistency Checks**: `ls skills/reporting/references/software-architecture.md` (expected: PASS)
**Commit**: `feat(reporting): add software-architecture reference`
