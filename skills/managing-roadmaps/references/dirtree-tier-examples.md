# Tier Examples

This document provides worked examples at each complexity tier for understanding roadmap composition.

## Contents

- [Tier 1: Patch (Bug Fix)](#tier-1-patch-bug-fix)
- [Tier 2: Feature (OAuth2 Support)](#tier-2-feature-oauth2-support)
- [Tier 3: Campaign (CLI-UX Normalization)](#tier-3-campaign-cli-ux-normalization)

---

## Tier 1: Patch (Bug Fix)

**Complexity:** Simple, single leaf task, flat structure
**Use Case:** Bug fixes, chores, minor improvements

### Directory Tree

```
__roadmap__/fix-threading-deadlock/
├── README.md
└── fix_deadlock.md
```

### Root README.md

```markdown
# Fix Threading Deadlock

## Context
Standalone bug fix. `ThreadedFileWriter` hangs during shutdown due to missing timeout on thread join.

## Reference Documents
- [Issue #42](link) — Original bug report with reproduction steps

## Goal
Prevent hang during shutdown of the `ThreadedFileWriter`.

## Pre-conditions
- [ ] Deadlock reproduced manually

## Success Gates
- ✅ `tests/test_shutdown.py` passes 50/50 iterations

## Status
```mermaid
graph TD
    fix_deadlock[Fix Deadlock]:::planned
    classDef planned fill:#374151,color:#e5e7eb
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `fix_deadlock.md` | 📄 Leaf Task | ⬜ Planned |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
```

### Leaf Task File: fix_deadlock.md

```markdown
# Fix Deadlock

**Goal**: Prevent hang during shutdown of the `ThreadedFileWriter`.
**Pre-conditions**:
- [ ] Branch `task/fix-deadlock` created from `milestone/fix-threading-deadlock`
**Success Gates**:
- ✅ `tests/test_shutdown.py` passes 50/50 iterations

---

## Step 1: Write reproduction test

**Goal**: Establish a failing test that reproduces the deadlock.

**Implementation Logic**:
Write a test that starts a `ThreadedFileWriter`, writes several entries,
then calls shutdown. The test should timeout after 5 seconds — if shutdown
hangs, the test fails by timeout.

**References**:
- [Issue #42](link) — Original bug report

**Deliverables**: `tests/test_shutdown.py` (~30 LOC)
**Consistency Checks**: `pytest tests/test_shutdown.py` (expected: FAIL)
**Commit**: `test(core): add reproduction case for writer shutdown deadlock`

---

## Step 2: Implement timeout join

**Goal**: Add timeout to thread join to prevent hanging.

**Implementation Logic**:
In `ThreadedFileWriter.shutdown()`, replace `self._thread.join()` with
`self._thread.join(timeout=5.0)`. If the thread is still alive after timeout,
log a warning and proceed with cleanup.

**References**:
- [Fix #42](link) — Proposed solution approach

**Deliverables**: `src/core/writer.py` (~5 LOC changed)
**Consistency Checks**: `pytest tests/test_shutdown.py` (expected: PASS)
**Commit**: `fix(core): add timeout to thread join in writer shutdown`

---

## Step 3: Full regression

**Goal**: Verify no regressions from the fix.

**Implementation Logic**:
Run the full test suite to confirm nothing else broke.

**Consistency Checks**: `pytest` (expected: PASS)
**Commit**: `chore(core): verify no regressions from deadlock fix`
```

**Execution Order:**
1. Enter `__roadmap__/fix-threading-deadlock/`
2. Read README.md
3. Execute `fix_deadlock.md` (Step 1 → commit, Step 2 → commit, Step 3 → commit)
4. Mark node as done
5. Merge task branch

---

## Tier 2: Feature (OAuth2 Support)

**Complexity**: Multiple directories, sequential dependencies between groups
**Use Case**: New features, refactors with intermediate states
**Parallelism**: Internal parallelism within subdirectories

### Directory Tree

```
__roadmap__/oauth2-support/
├── README.md
└── setup/
    ├── README.md
    ├── database_migration.md            # Depth 1 leaf: done first
    └── providers/                       # Depth 1 subdir: after database_migration
        ├── README.md
        ├── github.md                    # Depth 2 leaf: parallel with google
        ├── google.md                    # Depth 2 leaf: parallel with github
        └── validation/                  # Depth 2 subdir: after both providers
            ├── README.md
            └── integration_tests.md     # Depth 3 leaf
```

### Execution Order (Breadth-First)

1. **Root Level**: Setup phase (depth 1)
2. **Setup/read README** → Execute `database_migration.md` (leaf)
3. **Enter providers/** (depth 2)
   - Execute `github.md` ∥ `google.md` (parallel leaves)
4. **Enter validation/** (depth 3)
   - After both providers done → Execute `integration_tests.md` (leaf)
5. Return to root

### Root README.md

```markdown
# OAuth2 Support

## Context
Foundation work for OAuth2 support enabling GitHub and Google authentication.

## Reference Documents
- [R01 §2](path) — OAuth2 architecture design
- [Security Audit](path) — Authentication requirements

## Goal
Establish database schema and provider implementations for OAuth2 support.

## Pre-conditions
- [ ] Architecture report approved
- [ ] Database access granted

## Success Gates
- ✅ Both providers functional
- ✅ Integration tests pass
- ✅ Authentication working for both login flows

## Status
```mermaid
graph TD
    setup[Setup Phase]:::inprogress
    setup_completed[Setup Completed]:::planned
    providers_running[Running Providers]:::planned
    validators_working[Validation Complete]:::planned

    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned fill:#374151,color:#e5e7eb
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `setup/` | 📁 Directory | 🔄 In Progress |
| `providers/` | 📁 Directory | ⬜ Planned |
| `validation/` | 📁 Directory | ⬜ Planned |
```

<!-- setup/README.md, providers/README.md, validation/README.md, and all leaf task files omitted; follows the same structure as dirtree-authoring.md § Leaf Task -->

**Key Patterns:**
- **Depth indicates dependency**: `providers/` must wait for `database_migration.md`
- **Parallel siblings**: `github.md` and `google.md` execute in parallel
- **Leaf-first logic**: Leaves execute before subdirectories within each level
- **Progressive complexity**: Each level builds on previous work

---

## Tier 3: Campaign (CLI-UX Normalization)

**Complexity**: Multiple nested levels, parallel groups at every depth
**Use Case**: Major initiatives, multi-phased transformations
**Parallelism**: Parallel groups at multiple depths

### Directory Tree

```
__roadmap__/cli-ux-normalization/
├── README.md
├── test_setup.md                         # Depth 0 leaf (parallel)
├── fixtures.md                           # Depth 0 leaf (parallel)
└── core_rendering/                       # Depth 0 subdir → after foundation
    ├── README.md
    ├── color_system.md                   # Depth 1 leaf (parallel)
    ├── table_formatter.md                # Depth 1 leaf (parallel)
    └── advanced_components/              # Depth 1 subdir → after depth 1 leaves
        ├── README.md
        ├── reporter.md                   # Depth 2 leaf (parallel)
        ├── conversion_bridge.md          # Depth 2 leaf (parallel)
        ├── list_commands.md              # Depth 2 leaf (parallel)
        └── consumers/                    # Depth 2 subdir → after depth 2 leaves
            ├── README.md
            ├── mcp_handlers.md           # Depth 3 leaf (parallel)
            ├── package_handlers.md       # Depth 3 leaf (parallel)
            ├── env_system_handlers.md    # Depth 3 leaf (parallel)
            ├── show_commands.md          # Depth 3 leaf (parallel)
            └── finalization/             # Depth 3 subdir → after depth 3 leaves
                ├── README.md
                ├── deprecation_flags.md  # Depth 4 leaf (parallel)
                └── mcp_list_fix.md       # Depth 4 leaf (amendment A1)
```

### Parallelism at Every Depth

- **Depth 0 leaves**: `test_setup` ∥ `fixtures`
- **Depth 1 leaves**: `color_system` ∥ `table_formatter`
- **Depth 2 leaves**: `reporter` ∥ `conversion_bridge` ∥ `list_commands`
- **Depth 3 leaves**: `mcp_handlers` ∥ `package_handlers` ∥ `env_system_handlers` ∥ `show_commands`
- **Depth 4 leaves**: `deprecation_flags` ∥ `mcp_list_fix`

### Diamond Dependency Resolution

`deprecation_flags` depends on:
- Handler work (depth 3: `mcp_handlers`, `package_handlers`, etc.)
- Output work (depth 2: `reporter`, `conversion_bridge`, etc.)

**Solution**: Place `deprecation_flags` at depth 4
**Guarantee**: Breadth-first execution ensures both dependencies complete first

### Root README.md

```markdown
# CLI-UX Normalization

## Context
Complete transformation of CLI interface for improved usability and maintainability.
Transforms from legacy command pattern to unified CLI hub architecture.

## Reference Documents
- [UX Research Report](path) — User testing and feedback
- [Architecture Design](path) — Unified hub architecture
- [Component Inventory](path) — Existing components to refactor

## Goal
Normalize all CLI output and commands into a comprehensive, consistent interface.

## Success Gates
- ✅ All commands implement new colors, formatting, and show functionality
- ✅ MCP and package handlers unified
- ✅ Deprecation flags working for all old commands
- ✅ Full backward compatibility maintained

## Status
```mermaid
graph TD
    setup[Foundation Phase]:::inprogress
    core_rendering[Core Rendering]:::planned
    final_milestones[Milestones]:::planned

    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned fill:#374151,color:#e5e7eb
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `test_setup.md` | 📄 Leaf Task | 🔄 In Progress |
| `fixtures.md` | 📄 Leaf Task | 🔄 In Progress |
| `core_rendering/` | 📁 Directory | ⬜ Planned |
| `finalization/` | 📁 Directory | ⬜ Planned |
```

### Execution Flow

**Iterative Process:**

1. **Depth 0 Execution**
   - Create branches for `test_setup` and `fixtures` (parallel)
   - Execute parallel leaves, then return when both complete
   - Mark root `setup` as done

2. **Depth 1 Execution**
   - Enter `core_rendering/` directory
   - Create branches for `color_system` and `table_formatter`
   - Execute parallel leaves one task at a time
   - Return to root when done

3. **Depth 2 Execution**
   - Enter `core_rendering/advanced_components/`
   - Create branches for reporter, conversion_bridge, list_commands
   - Execute parallel leaves

4. **Depth 3 Execution**
   - Enter `consumers/` directory
   - Create branches for all handlers
   - Execute parallel leaves (4 independent handlers)

5. **Depth 4 Execution**
   - Enter `finalization/` directory
   - Execute `deprecation_flags` and any amendments
   - Final parallel leaves before completion

6. **Milestone Merge**
   - Verify all branches merged
   - Root README.md all-green
   - Milestone branch merged to `dev`

**Key Insight:**
- **Siblings are ALWAYS parallel** - maintain this discipline
- **Depth placement resolves dependencies** - deeper nodes execute later
- **Leaves execute before subdirectories** - every level follows this rule
- **No edges needed** - tree structure is sufficient execution spec
