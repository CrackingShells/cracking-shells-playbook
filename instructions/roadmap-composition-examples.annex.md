# Roadmap Composition Examples

Worked examples at each complexity tier. See [roadmap-generation.instructions.md](./roadmap-generation.instructions.md) for the full model.

---

## 1. Tier 1: Patch (Bug Fix)

A single leaf task. Sequential work lives as steps inside one file.

### Directory Tree

```
__roadmap__/fix-threading-deadlock/
├── README.md
└── fix_deadlock.md
```

### `README.md`

```markdown
# Fix Threading Deadlock

## Context

Standalone bug fix. `ThreadedFileWriter` hangs during shutdown due to missing
timeout on thread join.

## Reference Documents

- [Issue #42](link) — Original bug report with reproduction steps

## Goal

Prevent hang during shutdown of the `ThreadedFileWriter`.

## Pre-conditions

- [ ] Deadlock reproduced manually

## Success Gates

- ✅ `tests/test_shutdown.py` passes 50/50 iterations

---

## Status

​```mermaid
graph TD
    fix_deadlock[Fix Deadlock]:::planned
    classDef planned fill:#374151,color:#e5e7eb
​```

## Nodes

| Node | Type | Status |
|:-----|:-----|:-------|
| `fix_deadlock.md` | 📄 Leaf Task | ⬜ Planned |
```

### `fix_deadlock.md`

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

---

## 2. Tier 2: Feature (OAuth2 Support)

Nesting appears when there are sequential dependencies between groups of work.

### Directory Tree

```
__roadmap__/oauth2-support/
├── README.md
└── setup/
    ├── README.md
    ├── database_migration.md            # Leaf: done first
    └── providers/                       # Subdir: after database_migration
        ├── README.md
        ├── github.md                    # Leaf: parallel with google
        ├── google.md                    # Leaf: parallel with github
        └── validation/                  # Subdir: after both providers
            ├── README.md
            └── integration_tests.md     # Leaf
```

### Execution Order (breadth-first)

1. Enter `setup/`: execute `database_migration.md` (leaf)
2. Enter `providers/`: execute `github.md` ∥ `google.md` (parallel leaves)
3. Enter `validation/`: execute `integration_tests.md` (leaf)

### `setup/README.md`

```markdown
# Setup

## Context

Foundation work for OAuth2 support. Database tables must exist before
provider implementations can begin.

## Reference Documents

- [R01 §2](path) — OAuth2 architecture design

## Goal

Establish database schema and provider implementations for OAuth2.

## Pre-conditions

- [ ] Architecture report approved

## Success Gates

- ✅ Both providers functional
- ✅ Integration tests pass

---

## Status

​```mermaid
graph TD
    database_migration[Database Migration]:::done
    providers[Providers]:::inprogress

    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
​```

## Nodes

| Node | Type | Status |
|:-----|:-----|:-------|
| `database_migration.md` | 📄 Leaf Task | ✅ Done |
| `providers/` | 📁 Directory | 🔄 In Progress |
```

### `providers/README.md`

```markdown
# Providers

## Context

OAuth2 provider implementations. GitHub and Google can be implemented in
parallel — they share the database schema from `database_migration.md`
but have no dependency on each other.

## Goal

Implement GitHub and Google OAuth2 providers.

## Success Gates

- ✅ Both `/login/github` and `/login/google` functional

---

## Status

​```mermaid
graph TD
    github[GitHub]:::done
    google[Google]:::inprogress
    validation[Validation]:::planned

    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned fill:#374151,color:#e5e7eb
​```

## Nodes

| Node | Type | Status |
|:-----|:-----|:-------|
| `github.md` | 📄 Leaf Task | ✅ Done |
| `google.md` | 📄 Leaf Task | 🔄 In Progress |
| `validation/` | 📁 Directory | ⬜ Planned |
```

No edges between siblings. `github.md` and `google.md` are parallel leaves. `validation/` is a subdir — executes after both leaves complete.

---

## 3. Tier 3: Campaign (CLI-UX Normalization)

Deeper nesting with parallel groups at multiple depths. Diamond dependencies resolved by depth placement.

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

- Depth 0 leaves: `test_setup` ∥ `fixtures`
- Depth 1 leaves: `color_system` ∥ `table_formatter`
- Depth 2 leaves: `reporter` ∥ `conversion_bridge` ∥ `list_commands`
- Depth 3 leaves: `mcp_handlers` ∥ `package_handlers` ∥ `env_system_handlers` ∥ `show_commands`
- Depth 4 leaves: `deprecation_flags` ∥ `mcp_list_fix`

### Diamond Resolution

`deprecation_flags` depends on both handler work (depth 3) and output work (depth 2). Placed at depth 4, breadth-first guarantees everything above is complete.

### Root `README.md` Status

```mermaid
graph TD
    test_setup[Test Setup]:::done
    fixtures[Fixtures]:::done
    core_rendering[Core Rendering]:::inprogress

    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
```

### Git Branch History (flat off milestone)

```
milestone/cli-ux-normalization
  ├── task/test-setup              → merged (depth 0)
  ├── task/fixtures                → merged (depth 0)
  ├── task/color-system            → merged (depth 1)
  ├── task/table-formatter         → merged (depth 1)
  ├── task/reporter                → merged (depth 2)
  ├── task/conversion-bridge       → merged (depth 2)
  ├── task/list-commands           → merged (depth 2)
  ├── task/mcp-handlers            → merged (depth 3)
  ├── task/package-handlers        → merged (depth 3)
  ├── task/env-system-handlers     → merged (depth 3)
  ├── task/show-commands           → merged (depth 3)
  ├── task/deprecation-flags       → merged (depth 4)
  └── task/mcp-list-fix            → merged (depth 4)
```

All task branches flat off the milestone. Execution order governed by the roadmap tree, not by branch hierarchy.
