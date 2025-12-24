# Roadmap Generation Guidelines

## Overview

This document defines a **scalable, composable approach** to roadmap generation. You must select the "Mode" that best fits the complexity of your work. Every plan must be trackable through a strictly numbered hierarchy and ensure that implementation is bound by measurable verification.

### The 4-Level Hierarchy
1.  **Phase**: Strategic objective.
2.  **Milestone**: A delivery target (version or state).
3.  **Task**: A verifiable unit of work.
4.  **Step**: The atomic change. **Every Step is a bijection to exactly one Git commit.**

### The 3 Composition Modes

| Mode | Structure | Versioning Impact | Git Workflow |
| :--- | :--- | :--- | :--- |
| **Mode 1: Task** | **Task** → **Steps** | Patch or Pre-release (e.g., v0.1.x) | Single Branch (`fix/...`) |
| **Mode 2: Milestone** | **Milestone** → **Tasks** → **Steps** | Minor or Major (based on scope) | Feature Branch (`feat/...`) + Task Branches |
| **Mode 3: Campaign** | **Phases** → **Milestones** → **Tasks** → **Steps** | Strategic Progression (Major/Minor) | Full Hierarchy (`dev` → `milestone` → `task`) |

---

# MODE 1: THE TASK PLAN (Template)
*Use for: Bug fixes, chores, small refactors.*

# Task [Task-ID]: [Title]
**Goal**: [High-level objective]
**Pre-conditions**:
- [ ] [Measurable check, e.g., "Issue #123 reproduced in test suite"]
**Success Gates**:
- ✅ [Measurable gate, e.g., "All tests in `tests/core/` pass"]

## Implementation Steps
*Strict Bijection: 1 Step = 1 Commit*

### Step [Task-ID].[Step-ID]: [Title]
- **Goal**: [Unique change intent]
- **Pre-conditions**: [Measurable check, e.g., "Branch `fix/bug-x` created from `main`"]
- **Deliverables**: [Specific files to be modified]
- **Consistency Checks**: [Verification command, e.g., `npm run lint && pytest path/to/test.py`]
- **Commit**: `[type]([scope]): [description]` (Conventional Commit)

---

# MODE 2: THE MILESTONE PLAN (Template)
*Use for: New features or distinct release targets.*

# Milestone [Milestone-ID]: [Title]
**Target Version**: [e.g., v0.2.0, v1.0.0-beta]
**Objective**: [Strategic Goal]

## Task [Milestone-ID].[Task-ID]: [Title]
**Goal**: [Objective]
**Pre-conditions**: [Measurable entry criteria]
**Success Gates**:
- ✅ [Measurable gate]

### Step [Milestone-ID].[Task-ID].[Step-ID]: [Title]
- **Goal**: [Intent]
- **Pre-conditions**: [Measurable check]
- **Consistency Checks**: [Verification command]
- **Commit**: `[type]([scope]): [description]`

---

# MODE 3: THE CAMPAIGN ROADMAP (Template)
*Use for: Major projects or organizational initiatives.*

# [Project Name] – Roadmap [Version]

## Phase [Phase-ID]: [Title]
**Objective**: [Objective]

### Milestone [Phase-ID].[Milestone-ID]: [Title]
**Target Version**: [e.g., v0.5.0, v1.0.0]

#### Task [Phase-ID].[Milestone-ID].[Task-ID]: [Title]
**Goal**: [Objective]
**Pre-conditions**: [Measurable entry criteria]
**Success Gates**:
- ✅ [Measurable gate]

##### Step [Phase-ID].[Milestone-ID].[Task-ID].[Step-ID]: [Title]
- **Goal**: [Intent]
- **Pre-conditions**: [Measurable check]
- **Consistency Checks**: [Verification command]
- **Commit**: `[type]([scope]): [description]`
