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
**Deliverables**: <file paths, each annotated with the named symbols they will contain: function/method names, type/struct/interface names, endpoint paths, exported identifiers, component names — whatever the mode produces. Name the artifact, not its size.>
**Consistency Checks**: `<command>` (expected: PASS|FAIL)
**Commit**: `<type>(<scope>): <description>`
```

**Step Requirements:**
- Numbered sequence (1-N), maximum 5 steps
- 1 step = 1 commit (strict bijection)
- Implementation Logic must describe WHAT and WHY — if not described here, it should not be implemented
- TDD pattern recommended: Test (FAIL) → Implementation (PASS) → Verification (PASS)

**Writing effective Deliverables and Success Gates:**

Deliverables are a checklist for verification — name the symbols, not the size. A reader who hasn't seen the code should be able to search for each named artifact and confirm it exists. If you can't name a specific symbol, the step's Implementation Logic probably needs more precision first.

Success Gates describe observable outcomes. Prefer gates that can be confirmed by running a command or searching for a named artifact. If a gate requires a running system, a rendered interface, or human observation, note what setup is needed — otherwise a verifier has no way to distinguish "untested" from "passing." Optionally tag each gate `[run]`, `[static]`, or `[behavioral]` to signal how it should be checked.

A consistency check that would pass even if the step's deliverable was never implemented is tautological and provides false confidence. A workspace build passing does not confirm a type was added — it confirms nothing broke. Prefer commands that directly exercise the new artifact: a test that imports the new type, a command that invokes the new endpoint, a type-check that uses the new interface at a real call site.

---

## CRUD: Create

> **`__roadmap__/` must NOT have its own README.md.** It is a plain container directory. Only campaign subdirectories have README.md files.

1. **Identify campaign** and understand requirements
2. **Choose tier level** (patch vs feature vs campaign)
3. **Analyze dependencies**: Partition work into parallel siblings vs. sequential depths using dependency signals (see [Dependency Signals](#dependency-signals))
4. **Bootstrap the campaign** using `dirtree-rdm init` — creates the directory and a BNF-valid template README.md without requiring a parent README:
   ```bash
   bash skills/managing-roadmaps/scripts/dirtree-rdm.sh init __roadmap__/<campaign_name>
   ```
5. **Fill in the campaign README.md** prose sections — `init` creates the scaffold; author the Context, Goal, Pre-conditions, and Success Gates. Do not touch the Mermaid block, Nodes table, or Amendment Log by hand.
6. **Add nodes using `dirtree-rdm`** — do not hand-edit README.md Nodes tables or Mermaid blocks:
   ```bash
   bash skills/managing-roadmaps/scripts/dirtree-rdm.sh add __roadmap__/<campaign>/node.md --title "Title"
   bash skills/managing-roadmaps/scripts/dirtree-rdm.sh add __roadmap__/<campaign>/subdir --type dir --title "Title"
   ```
7. **For multiple depth levels**: Follow breadth-first required order
8. **No numeric prefixes** in names

See [dirtree-cli.md](dirtree-cli.md) for full `dirtree-rdm` command reference.

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
