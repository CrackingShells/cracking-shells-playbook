# Roadmap Generation Template

## Instructions

This template follows the exact structure of the Stichotrope v1.0.0 Implementation Roadmap, which has proven effective for complex multi-phase projects. Use this template to create comprehensive roadmaps that integrate with the organization's git workflow, versioning strategy, and development practices.

**How to use this template**:
1. Copy the entire template section below
2. Replace all placeholders (e.g., `[Project Name]`, `[X.Y.Z]`) with your project-specific values
3. Adjust the number of phases, milestones, and tasks to match your project scope
4. Save to `__design__/<project>_roadmap_v<version>.md`

**Key principles**:
- **Phases** → **Milestones** → **Tasks** hierarchy
- Semantic versioning aligned with roadmap structure
- Git workflow integration (main → dev → milestone/X.X → task/X.X.X)
- Clear success gates at each level
- Topological ordering for parallel development

---

# TEMPLATE STARTS HERE

---

# [Project Name] v[X.0.0] – Implementation Roadmap (v[N] - [Descriptor])

**Project**: [Project Name] – [Brief Description]
**Roadmap Date**: YYYY-MM-DD
**Phase**: [Planning | Implementation | Release]
**Source**: [e.g., Synthesis v1 + Stakeholder Decisions + Workflow Alignment]
**Prototype Version**: v[X.Y.Z] ([status, e.g., validated, GO decision, kept as baseline])
**Target Version**: v[X.0.0] ([e.g., production release, MVP, beta])
**Timeline**: [X-Y weeks/months] ([key approach, e.g., documentation & tests-driven, architecture-first])

---

## Executive Summary

[Brief overview of the roadmap and its purpose]

**Key Changes from v[N-1]** (if this is a roadmap update):
- ✅ **[Change Category 1]**: [Description of change and rationale]
- ✅ **[Change Category 2]**: [Description of change and rationale]
- ✅ **[Change Category 3]**: [Description of change and rationale]

**Maintained from v[N-1]** (if this is a roadmap update):
- ✅ **[Maintained Item 1]**: [Description]
- ✅ **[Maintained Item 2]**: [Description]

**Stakeholder Decisions**:
- ✅ [Decision 1]: [Choice made and rationale]
- ✅ [Decision 2]: [Choice made and rationale]
- ✅ [Decision 3]: [Choice made and rationale]

**Architectural Decision**:
[Key architectural decision and rationale. Example: "Complete redesign of X architecture from scratch to integrate Y as a first-class concern from the ground up. This avoids retrofitting Y support after implementation, which creates technical debt and maintenance challenges."]

---

## Versioning Strategy

**Semantic Versioning (SemVer)**: `Major.Minor.Patch`

**Version Progression**:
- **Major Version**: [Starting major] ([status]) → [Target major] ([target status])
  - Major=[Starting] enforced until v[Target].0.0 release
  - Breaking changes [allowed/not allowed] during Major=[Starting] phase
  - Major=[Target] signals [what it signals, e.g., production-ready, stable API]

- **Minor Version**: Increments per Phase completion
  - Phase 1 complete → v[X].[Y].0 ([Phase 1 Name])
  - Phase 2 complete → v[X].[Y+1].0 ([Phase 2 Name])
  - Phase 3 complete → v[X].[Y+2].0 ([Phase 3 Name])
  - Final release → v[Target].0.0 ([Final Release Name])

- **Patch Version**: Increments per Milestone completion within a Phase
  - Milestone 1.1 complete → v[X].[Y].0 (first milestone of Phase 1)
  - Milestone 1.2 complete → v[X].[Y].1 (second milestone of Phase 1)
  - Milestone 1.3 complete → v[X].[Y].2 (third milestone of Phase 1)
  - Milestone 2.1 complete → v[X].[Y+1].0 (first milestone of Phase 2)
  - Milestone 2.2 complete → v[X].[Y+1].1 (second milestone of Phase 2)

- **Tasks**: Do NOT increment version (tasks are sub-units of milestones)

**Version Management**:
- Configured in [package file, e.g., pyproject.toml, package.json] with [tool, e.g., python-semantic-release, semantic-release]
- Automated version bumping based on conventional commits
- Git tags created automatically on version increments
- [Publishing platform, e.g., PyPI, npm] publishing triggered on v[Target].0.0 tag

---

## Git Workflow

**[Project Name]-Specific Branching Strategy** (Project Variation)

**Branch Hierarchy**:
```
main (production, v[Target].0.0 release only)
  └── dev (development integration branch)
      ├── milestone/1.1-[short-description]
      │   ├── task/1.1.1-[short-description]
      │   ├── task/1.1.2-[short-description]
      │   └── task/1.1.3-[short-description]
      ├── milestone/1.2-[short-description]
      │   ├── task/1.2.1-[short-description]
      │   ├── task/1.2.2-[short-description]
      │   └── task/1.2.3-[short-description]
      ├── milestone/2.1-[short-description]
      │   ├── task/2.1.1-[short-description]
      │   ├── task/2.1.2-[short-description]
      │   └── task/2.1.3-[short-description]
      └── ...
```

**Workflow Rules**:

1. **All work from `dev` branch** (not `main`)
   - `main` is production-only, receives merges only at v[Target].0.0 release
   - `dev` is the integration branch for all development work

2. **Milestone branches from `dev`**
   - Branch naming: `milestone/<milestone-id>-<short-description>`
   - Example: `milestone/2.1-[example-description]`
   - Created when milestone work begins
   - Deleted after merge back to `dev`

3. **Task branches from milestone branches**
   - Branch naming: `task/<task-id>-<short-description>`
   - Example: `task/2.1.1-[example-description]`
   - Created when task work begins
   - Deleted after merge back to milestone branch

4. **Merge Hierarchy**:
   - Task branches → Milestone branch (when task complete)
   - Milestone branch → `dev` (when ALL milestone tasks complete)
   - `dev` → `main` (when milestone in `dev` passes ALL tests: regression, unit, integration, performance)

5. **Merge Criteria**:
   - **Task → Milestone**: Task success gates met, task tests pass
   - **Milestone → dev**: All milestone tasks complete, all milestone tests pass, no regressions
   - **dev → main**: ALL tests pass (regression, unit, integration, performance), ready for release

6. **Conventional Commits**:
   - Follow organization's conventional commit format
   - Enables automated semantic versioning
   - See `git-workflow.md` for commit message standards

**Note**: This is a [Project Name]-specific variation of the organization's standard git workflow, optimized for milestone-based development with strict quality gates.

---

## Phase 1: [Phase Name] (Weeks [X-Y] or [Duration])

**Version Target**: v[X].[Y].0 (after all Phase 1 milestones complete)

**Objective**: [High-level objective for this phase]

### Milestone 1.1: [Milestone Name] ([Duration])

**Version Target**: v[X].[Y].0 (first milestone of Phase 1)

**Tasks**:

**1.1.1 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task, or "None"]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion 3]
  - [Success criterion N]

**1.1.2 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**1.1.3 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]
  - **NOTE**: [Any important notes or clarifications]

### Milestone 1.2: [Milestone Name] ([Duration])

**Version Target**: v[X].[Y].1 (second milestone of Phase 1)

**Tasks**:

**1.2.1 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**1.2.2 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**1.2.3 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

### Milestone 1.3: [Milestone Name] ([Duration])

**Version Target**: v[X].[Y].2 (third milestone of Phase 1)

**Tasks**:

**1.3.1 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**1.3.2 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**1.3.3 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

[Add more milestones as needed for Phase 1]

---

## Phase 2: [Phase Name] (Weeks [X-Y] or [Duration])

**Version Target**: v[X].[Y+1].0 (after all Phase 2 milestones complete)

**Objective**: [High-level objective for this phase]

### Milestone 2.1: [Milestone Name] ([Duration])

**Version Target**: v[X].[Y+1].0 (first milestone of Phase 2)

**Architectural Decision** (if applicable): [Key architectural decision for this milestone]

**Tasks**:

**2.1.0 – [Task Name]** (if you need a task 0 for evaluation/planning)
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - **Aspect 1: [Aspect Name]** - [Description]:
    - [Option 1 description]
    - [Option 2 description]
    - [Option 3 description]
    - Pros/cons analysis for each approach ([criteria 1], [criteria 2], [criteria 3])
  - **Aspect 2: [Aspect Name]** - [Description]:
    - [Option 1 description]
    - [Option 2 description]
    - [Option 3 description]
    - Pros/cons analysis for each approach ([criteria 1], [criteria 2], [criteria 3])
  - **Recommended Approach**: Clear selection with rationale for both aspects
  - **Trade-off Analysis**: [Trade-offs] documented
  - **[Comparison Point]**: How recommended approach differs from [baseline] documented
  - **Evaluation Review**: Stakeholder review completed and approved

**2.1.1 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**2.1.2 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**2.1.3 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**2.1.4 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

### Milestone 2.2: [Milestone Name] ([Duration])

**Version Target**: v[X].[Y+1].1 (second milestone of Phase 2)

**Tasks**:

**2.2.1 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**2.2.2 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**2.2.3 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**2.2.4 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

[Add more milestones as needed for Phase 2]

---

## Phase 3: [Phase Name] (Weeks [X-Y] or [Duration])

**Version Target**: v[X].[Y+2].0 (after all Phase 3 milestones complete) → v[Target].0.0 ([final release name])

**Objective**: [High-level objective for this phase]

### Milestone 3.1: [Milestone Name] ([Duration])

**Version Target**: v[X].[Y+2].0 (first milestone of Phase 3)

**Tasks**:

**3.1.1 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**3.1.2 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**3.1.3 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

[Add more tasks as needed]

### Milestone 3.2: [Milestone Name] ([Duration])

**Version Target**: v[Target].0.0 ([final release name])

**Tasks**:

**3.2.1 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**3.2.2 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**3.2.3 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

**3.2.4 – [Task Name]**
- **Goal**: [What this task accomplishes]
- **Pre-conditions**: [What must be complete before starting this task]
- **Success Gates**:
  - [Success criterion 1]
  - [Success criterion 2]
  - [Success criterion N]

---

## Deferred Features (v[Next Version]+)

**[Feature Name 1]**:
- Rationale: [Why deferred]
- Timeline: v[Next Version] release
- Benefit: [What this feature provides]
- Implementation: [High-level implementation approach]

**[Feature Name 2]**:
- Rationale: [Why deferred]
- Timeline: [When it will be implemented]
- Benefit: [What this feature provides]
- Implementation: [High-level implementation approach]

**[Feature Name 3]**:
- Rationale: [Why deferred]
- Timeline: [When it will be implemented]
- Benefit: [What this feature provides]
- Implementation: [High-level implementation approach]

---

## Success Criteria for v[Target].0.0

**Functional Requirements**:
- ✅ [Functional requirement 1]
- ✅ [Functional requirement 2]
- ✅ [Functional requirement 3]
- ✅ [Functional requirement N]

**Quality Requirements**:
- ✅ [Quality requirement 1, e.g., Test coverage >90%]
- ✅ [Quality requirement 2, e.g., Performance ≤X% overhead]
- ✅ [Quality requirement 3, e.g., Platform support: Windows, Linux, macOS]
- ✅ [Quality requirement 4, e.g., Documentation complete]
- ✅ [Quality requirement N]

---

## Topological Ordering for Parallel Development

**Critical Path** (must complete in order):
1. [Phase/Milestone sequence 1]
2. [Phase/Milestone sequence 2]
3. [Phase/Milestone sequence 3]

**Parallel Opportunities**:
- [Milestone X] and [Milestone Y] can start simultaneously
- [Milestone A] and [Milestone B] can start after [Milestone C]
- [Milestones D, E, F] can run in parallel after [Milestone G] implementation complete
- [Phase X] and [Phase Y] can run in parallel

---

**Report Version**: v[N] ([Descriptor])
**Status**: [Ready for implementation | In progress | Complete]
**Next Steps**: [What happens next, e.g., "Create GitHub Issues for each task, assign to team members, begin Phase 1"]

