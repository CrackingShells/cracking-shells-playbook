# Schema Validation Rules

This document provides complete validation requirements for roadmap documents.

## Contents

- [Document Types](#document-types)
- [Naming Conventions](#naming-conventions)
- [Status Requirements](#status-requirements)
- [1:1 Mapping Verification](#11-mapping-verification)
- [Step Validation Rules](#step-validation-rules)
- [Amendment Validation](#amendment-validation)
- [Progress Tracking Requirements](#progress-tracking-requirements)
- [Reference Document Requirements](#reference-document-requirements)
- [Tier Complexity Constraints](#tier-complexity-constraints)

---

## Document Types

### README.md (Directory Level)

**Required Sections:**

1. **title**: Descriptive title for this level
2. **context**: 1-2 sentences about parent campaign dependencies and context
3. **reference_documents**: Links to analysis reports in __reports__/
4. **goal**: One-line objective for this level
5. **preconditions**: Measurable entry criteria
6. **success_gates**: Measurable completion criteria
7. **status_graph**: Mermaid graph with nodes and class_defs
8. **nodes_table**: 1:1 mapping with filesystem entries and status graph
9. **amendment_log**: Log of all amendments (may be empty)
10. **progress**: Track branch names, commit counts per node

### Leaf Task (.md)

**Required Sections:**

1. **title**: Component or topic name
2. **goal**: One-line objective for the task
3. **preconditions**: Measurable entry criteria
4. **success_gates**: Measurable completion criteria for whole task
5. **references**: Task-level references to architecture reports
6. **steps**: Array of steps (1-5 max)

**Step Requirements:**

1. **number**: Sequential step number (1-N)
2. **title**: Short descriptive title
3. **goal**: Unique change intent
4. **implementation_logic**: Description of WHAT and WHY (constrains scope)
5. **deliverables**: File paths and LOC estimate
6. **consistency_checks**: Command with expected outcome (PASS|FAIL)
7. **commit**: Conventional commit message format
8. **references**: Optional step-level references (may be empty array)
9. **requires**: Optional (non-obvious prerequisites only)

---

## Naming Conventions

### File Extensions
- README.md: Required for every directory
- .md: Required for leaf task files only

### File Naming
**Directories:** `snake_case/` or `kebab-case/` naming components/concerns/phases

**Leaf Tasks:** `snake_case.md` naming specific work units

**Campaigns:** descriptive kebab-case or snake_case (e.g., `cli-ux-normalization` or `oauth_support`)

**Constraints:**
- **No numeric prefixes** (`T01_`, `M1.2_`, `issue-42_fix`) - ordering comes from tree depth
- No leading/double extensions (.md.md)

**Node IDs** (Mermaid graph):
- Must match filesystem names (without .md for files, without / for directories)
- Pattern: `^[a-z][a-z0-9_]*$`

---

## Status Requirements

### Status Values
`done`, `inprogress`, `planned`, `amendment`, `blocked`

### Status Mapping
```
✅ Done        → done
🔄 In Progress → inprogress
⬜ Planned      → planned
🔵 Amendment   → amendment
🚫 Blocked     → blocked
```

### Mermaid Coloring
All status colors must be defined:

```
classDef done       fill:#166534,color:#bbf7d0      # Dark green bg
classDef inprogress fill:#854d0e,color:#fef08a    # Dark amber bg
classDef planned    fill:#374151,color:#e5e7eb     # Dark gray bg
classDef amendment  fill:#1e3a5f,color:#bfdbfe     # Dark blue bg
classDef blocked    fill:#7f1d1d,color:#fecaca     # Dark red bg
```

### Edge Rules
**NO edges between siblings** - siblings are always parallel
Ordering comes only from tree structure (depth, leaves before subdirectories)

---

## 1:1 Mapping Verification

### README.md to Filesystem
For every directory level:

1. **Read `ls` output** of the directory
2. **Map each item**:
   - Files = node entries in status graph
   - Subdirectories = node entries in status graph
   - README.md = excluded from nodes (the container)
3. **Compare** with Nodes table:
   - Count must match
   - IDs must match exact filesystem names
   - Types must match (📄 Leaf Task vs 📁 Directory)

### README.md to Status Graph
Verify 1:1 mapping between:

1. **Nodes table**: Each row = one filesystem entry
2. **Status graph**: Each node = one filesystem entry
3. **Files/subdirectories**: Count must equal total rows

---

## Step Validation Rules

### Step Count Constraints
- **Minimum:** 1 step
- **Maximum:** 5 steps
- **Strict bijection:** 1 step = 1 commit

### Step Prerequisites
- **NOT required** for sequential reasoning (steps are sequential by definition)
- **Only include** for genuinely non-obvious needs (e.g., "running database instance")
- **Avoid:** "Previous step done" (tautological)

### Implementation Logic Requirements
- **Must describe WHAT and WHY**
- **NO code dumps**
- If not described, **NOT implemented**
- Use numbered pseudocode for complex logic

### Consistency Check Outcomes
- **PASS:** Expected outcome, commit after implementation
- **FAIL:** Expected outcome (e.g., TDD test step), commit after passing test
- **Mismatch:** Raises Level 1-2 failure in escalation ladder

### Commit Message Format
**Conventional commit**: `type(scope): description`

Allowed types: `feat`, `fix`, `test`, `docs`, `chore`, `refactor`, `style`, `perf`, `ci`, `build`, `revert`

**Required:** scope in parentheses

---

## Amendment Validation

### Amendment Entry Requirements
Every amendment in Amendment Log must include:

1. **id**: Sequential amendment ID (A1, A2, ...)
2. **date**: Approved date (YYYY-MM-DD)
3. **source**: Relative path in __reports__/
4. **nodes_added**: Array of filesystem names added
5. **rationale**: One-line explanation

### Amendment Rules
1. **Must have source document** in __reports__/
2. **Must be reviewed** by agent before plan updated
3. **Never rename/renumber** existing nodes
4. **Use `:::amendment`** styling until executed
5. **Place deeper** if depends on existing nodes
6. **Create subdirectory** if adds complexity

---

## Progress Tracking Requirements

### Progress Table Columns
1. **node**: Filesystem name
2. **branch**: Git branch name (task/<name>) or null/— for directories
3. **commits**: Number of commits (0+) or null/— if not applicable
4. **notes**: Free-text notes (e.g., ✅ Merged, 🔄 In progress)

### Branch Requirements
- **Task branches**: Named `task/<name>` flat off `milestone/<campaign>`
- **No nested branch hierarchy** - flat branching model
- **Number of commits** = exact count of commits on branch

---

## Reference Document Requirements

### Reference Format
```markdown
[R<nn> <title>](<relative path>) — <one-line description>
```

- **R<nn>**: Report ID (e.g., R01, R02)
- **path**: Relative to `__reports__/`
- **description**: What to find at this reference

### Reference Use Cases
- Leaf task references: Architecture design, methodology
- README references: Analysis reports informing each level
- Amendment sources: Gap analysis justifying changes

---

## Tier Complexity Constraints

### Tier 1: Patch
- **Depth:** Flat (single leaf task)
- **Parallelism:** None (sequential steps inside file)
- **When to use:** Bug fixes, chores

### Tier 2: Feature
- **Depth:** 1-2 levels of nesting
- **Parallelism:** Some parallel siblings
- **When to use:** New features, refactors

### Tier 3: Campaign
- **Depth:** 2+ levels of nesting
- **Parallelism:** Parallel groups at multiple depths
- **When to use:** Major initiatives

### Escalation Rules
- **Promote leaf:** When outgrows ~5 steps or 3+ sub-tasks
- **Natural progression:** Model doesn't change, only depth
