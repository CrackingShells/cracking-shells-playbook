# Roadmap Authoring Guide

## Contents

- [Directory Structure Rules](#directory-structure-rules)
- [File Structure Specifications](#file-structure-specifications)
- [CRUD: Create](#crud-create)
- [CRUD: Read](#crud-read)
- [Key References](#key-references)

---

## Directory Structure Rules

**Create a subdirectory when:**
- Concern decomposes into 3+ independent sub-tasks
- Sequential dependencies exist between sibling work
- Task could be assigned to different agents for parallel execution

**Keep as a leaf task when:**
- 1-5 sequential steps with no internal parallelism
- Single logical unit (e.g., one TDD cycle)

**Promote when:** Outgrows ~5 steps or 3+ sub-tasks

**Tier Complexity Levels:**
- Tier 1 (Patch): Flat, single leaf task
- Tier 2 (Feature): 1-2 levels of nesting, some parallelism
- Tier 3 (Campaign): 2+ levels, parallel groups at multiple depths

See [dirtree-tier-examples.md](dirtree-tier-examples.md) for worked examples at each tier.

### Dependency Signals

Use these heuristics when deciding placement — the implementation context determines which apply:

| Signal | Structure |
|:-------|:----------|
| No shared files, no cross-references | Siblings |
| Both tasks edit the same files | Sequential — merge conflicts at branch integration |
| B's steps cannot execute without A's output existing | Sequential — runtime dependency |

After parallel siblings that produce cross-references (imports, shared interfaces), add a verification leaf at next depth to check mutual consistency.

**Anti-pattern — sequential by default:**

```
# Wrong: depth-nesting independent tasks
auth.md  →  next/billing.md  →  next/final/notifications.md

# Right: siblings + verification gate
auth.md  billing.md  notifications.md    (parallel leaves)
integration/verify_contracts.md          (depth +1, after leaves)
```

---

## File Structure Specifications

### README.md (Every Directory)

**Required Sections:**

```markdown
# <Title>

## Context
<1-2 sentences: where this node fits in parent campaign, what it depends on, what it produces, who consumes it>

## Reference Documents
- [R<nn> <title>](<relative path>) — <what this covers>

## Goal
<One-line objective for this level>

## Pre-conditions
- [ ] <Measurable entry criteria>

## Success Gates
- ✅ <Measurable completion criteria>

## Gotchas
<Optional. Known issues or implementation notes>

## Status
```mermaid
graph TD
    node_a[Node A]:::done
    node_b[Node B]:::inprogress
    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned fill:#374151,color:#e5e7eb
    classDef amendment fill:#1e3a5f,color:#bfdbfe
    classDef blocked fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `node_a.md` | 📄 Leaf Task | ✅ Done |
| `node_b/` | 📁 Directory | 🔄 In Progress |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
```

**Status Visualization Rules:**
- Node IDs = filesystem names (no `.md` or `/`)
- No edges between siblings
- All 5 statuses must be defined in `classDef`: done, inprogress, planned, amendment, blocked
- >12 nodes → use Mermaid subgraphs for readability

### Leaf Task (.md)

**Required Structure:**

```markdown
# <Title>

**Goal**: <One-line objective>
**Pre-conditions**:
- [ ] <Sibling leaves complete, environmental state, etc.>
**Success Gates**:
- ✅ <Measurable gate>
**References**: [R<nn> §<section>](<path>) — <what to find>

## Step 1: <Title>
**Goal**: <Unique change intent>
**Implementation Logic**: <WHAT and WHY. Describe scope. Use numbered pseudocode if needed.>
**References**: [R<nn> §<section>](<path>) — <what to find>
**Deliverables**: <file paths> (~LOC estimate)
**Consistency Checks**: `<command>` (expected: PASS|FAIL)
**Commit**: `<type>(<scope>): <description>`
```

**Step Requirements:**
- Numbered sequence (1-N), maximum 5 steps
- 1 step = 1 commit (strict bijection)
- Implementation Logic must describe WHAT and WHY — if not described here, it should not be implemented
- TDD pattern recommended: Test (FAIL) → Implementation (PASS) → Verification (PASS)

---

## CRUD: Create

1. **Identify campaign** and understand requirements
2. **Choose tier level** (patch vs feature vs campaign)
3. **Analyze dependencies**: Partition work into parallel siblings vs. sequential depths using dependency signals (see [Dependency Signals](#dependency-signals))
4. **Create root directory**: `__roadmap__/<campaign_name>/`
5. **Write root README.md** with context, references, and status graph
6. **Create leaf tasks** (1-5 steps each) or subdirectories
7. **For subdirectories**: Add README.md with goal, pre-conditions, success gates
8. **For multiple depth levels**: Follow breadth-first required order
9. **No numeric prefixes** in names

---

## CRUD: Read

1. **Check merit todo command** (creates `__roadmap__/TODO.md`)
2. **Read rollout announcements** in `__reports__/` for context
3. **Navigate to task**: List sibling files, then subdirectories
4. **Check status**: Look at Mermaid diagram and Nodes table
5. **Review branches**: Check Progress table for which tasks have branches

---

## Key References

- [dirtree-schema-validation.md](dirtree-schema-validation.md) — Full naming rules, node ID patterns, step validation, and compliance requirements
- [dirtree-tier-examples.md](dirtree-tier-examples.md) — Worked examples for Tier 1 (patch), Tier 2 (feature), and Tier 3 (campaign)
