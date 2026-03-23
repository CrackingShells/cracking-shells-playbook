# Dirtree BNF Grammar Reference

This document is the human/agent-readable mirror of the canonical grammar files:
- `scripts/dirtree-rdm/grammar/readme.bnf` — README.md productions
- `scripts/dirtree-rdm/grammar/leaf.bnf` — Leaf task .md productions

The Rust validator (`dirtree-rdm`) embeds these files at compile time and validates every README.md and leaf task against them before and after any write.

---

## README.md Grammar

```
<readme> ::= <h1-title>
             <section-context>
             <section-reference-documents>?
             <section-goal>
             <section-preconditions>
             <section-success-gates>
             <section-gotchas>?
             <section-status>
             <section-nodes>
             <section-amendment-log>
             <section-progress>
```

### Section Order (enforced)

| # | Section | Heading | Required? |
|:--|:--------|:--------|:----------|
| 1 | Title | `# <text>` | yes |
| 2 | Context | `## Context` | yes |
| 3 | Reference Documents | `## Reference Documents` | no |
| 4 | Goal | `## Goal` | yes |
| 5 | Pre-conditions | `## Pre-conditions` | yes |
| 6 | Success Gates | `## Success Gates` | yes |
| 7 | Gotchas | `## Gotchas` | no |
| 8 | Status | `## Status` | yes |
| 9 | Nodes | `## Nodes` | yes |
| 10 | Amendment Log | `## Amendment Log` | yes |
| 11 | Progress | `## Progress` | yes |

### Mermaid Block (`## Status`)

```
<mermaid-block> ::=
  ```mermaid
  graph TD
      <id>[<title>]:::<status>   (0 or more)
      classDef done       fill:#166534,color:#bbf7d0
      classDef inprogress fill:#854d0e,color:#fef08a
      classDef planned    fill:#374151,color:#e5e7eb
      classDef amendment  fill:#1e3a5f,color:#bfdbfe
      classDef blocked    fill:#7f1d1d,color:#fecaca
  ```
```

**Constraints:**
- Node IDs: `^[a-z][a-z0-9_-]*$`
- Status values: `done | inprogress | planned | amendment | blocked`
- All 5 `classDef` lines required in exact order with exact colors
- **No `-->` edges** — siblings are always parallel; ordering comes from tree depth only

### Nodes Table (`## Nodes`)

```
| Node | Type | Status |
|:-----|:-----|:-------|
| `<fs-name>` | 📄 Leaf Task | ⬜ Planned |
| `<dir-name>` | 📁 Directory | 🔄 In Progress |
```

**Constraints:**
- `<fs-name>` matches filesystem entry exactly (with `.md` for leaves, without for dirs)
- Type: `📄 Leaf Task` or `📁 Directory`
- Status emoji must match the node's `:::status` in Mermaid
- 0 rows valid (new empty directory); rows must be 1:1 with filesystem entries

### Amendment Log Table (`## Amendment Log`)

```
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|
| A1 | YYYY-MM-DD | path/to/report.md | ["node.md"] | one-line rationale |
```

### Progress Table (`## Progress`)

```
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
| `<fs-name>` | task/<name> | <n> | <free text> |
```

---

## Leaf Task Grammar

```
<leaf-task> ::= <h1-title>
                <task-header>    (all 4 fields required, order-insensitive)
                <step>+          (1–5 steps, sequentially numbered)
```

### Task Header Fields (all required)

```
**Goal**: <one-line objective>
**Pre-conditions**:
- [ ] <entry criterion>
**Success Gates**:
- ⬜ <completion criterion>
**References**: [R<nn> §<section>](<path>) — <what to find>
```

### Step Structure

```
## Step N: <Title>
**Goal**: <unique change intent>
**Implementation Logic**:
<WHAT and WHY — no code dumps>
**Deliverables**: <file paths> (~N LOC)
**Consistency Checks**: `<command>` (expected: PASS|FAIL)
**Commit**: `<type>(<scope>): <description>`
```

**Required step fields:** Goal, Implementation Logic, Deliverables, Consistency Checks, Commit

**Commit type** must be one of: `feat fix test docs chore refactor style perf ci build revert`

**Step count:** minimum 1, maximum 5

---

## Terminal Patterns

| Terminal | Pattern |
|:---------|:--------|
| node-id | `^[a-z][a-z0-9_-]*$` |
| node-name-file | `^[a-z][a-z0-9_-]*\.md$` |
| node-name-dir | `^[a-z][a-z0-9_-]*$` |
| status-value | `done \| inprogress \| planned \| amendment \| blocked` |
| commit-type | `feat \| fix \| test \| docs \| chore \| refactor \| style \| perf \| ci \| build \| revert` |
| date | `\d{4}-\d{2}-\d{2}` |

---

## Status Emoji Map

| Status value | Emoji |
|:-------------|:------|
| `planned` | ⬜ Planned |
| `inprogress` | 🔄 In Progress |
| `done` | ✅ Done |
| `amendment` | 🔵 Amendment |
| `blocked` | 🚫 Blocked |
