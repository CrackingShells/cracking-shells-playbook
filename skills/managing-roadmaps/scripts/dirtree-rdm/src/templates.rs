/// Scaffold templates for new README.md and leaf task .md files.

pub fn readme_template(title: &str) -> String {
    format!(
        r#"# {title}

## Context
<1-2 sentences: where this node fits in the parent campaign, what it depends on, what it produces>

## Goal
<One-line objective for this level>

## Pre-conditions
- [ ] <Measurable entry criteria>

## Success Gates
- ⬜ <Measurable completion criteria>

## Status
```mermaid
graph TD
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
"#
    )
}

pub fn leaf_template(title: &str) -> String {
    format!(
        r#"# {title}

**Goal**: <One-line objective>
**Pre-conditions**:
- [ ] <Entry criteria>
**Success Gates**:
- ⬜ <Measurable gate>
**References**: [R01 §<section>](<path>) — <what to find>

## Step 1: <Title>
**Goal**: <Unique change intent>
**Implementation Logic**:
<WHAT and WHY. Describe scope.>
**Deliverables**: <file paths> (~N LOC)
**Consistency Checks**: `<command>` (expected: PASS)
**Commit**: `feat(<scope>): <description>`
"#
    )
}
