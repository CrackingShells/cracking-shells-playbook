# Amendment Workflow

This document provides complete details on how to properly amend roadmaps after initial creation.

## Contents

- [Amendment Overview](#amendment-overview)
- [The Amendment Cycle](#the-amendment-cycle)
- [Amendment Log Format](#amendment-log-format)
- [Reopening Closed Work](#reopening-closed-work)
- [Common Amendment Patterns](#common-amendment-patterns)
- [Escalation vs. Amendment](#escalation-vs-amendment)
- [Amendment Review Checklist](#amendment-review-checklist)
- [Tracking Amendment Progress](#tracking-amendment-progress)

---

## Amendment Overview

Amendments evolve a roadmap after initial creation when:
- Gaps are discovered during execution
- Enhancement requests are made
- New requirements emerge
- Scope changes are needed

**Important Principles:**
- Amendments require formal review and approval
- Every amendment must have a source document in the reports store (e.g., `__reports__/` in directory-tree backend)
- New nodes preserve existing nodes (never delete/rename)
- Amendment ID format: `A1`, `A2`, etc.

---

## The Amendment Cycle

### 1. Discovery and Analysis

**Agent Identifies Gap:**

When execution reveals a gap or enhancement request, the agent:
1. Documents the gap with measurable criteria
2. Determines if formal amendment is needed
3. Produces analysis in the reports store

**Gap Classification:**

| Type | Example | Action |
|:-----|:---------|:--------|
| Missing functionality | Need OAuth2 for Google | Raise amendment |
| Broken assumption | Database schema changed | Raise amendment |
| Implementation error | Path typo in previous task | Fix (bug), don't amend |
| UX improvement | Better error messages | Not a gap (future campaign) |

**When to NOT Amend:**
- UX/API usability improvements
- "Nice to have" features out of scope
- Speculative problems without evidence
- Typos or minor code issues within current scope

### 2. Gap Analysis Report

**Report Location:** reports store (directory-tree default: `__reports__/<amendment-id>/<topic>/`)

**Report Structure:**

```markdown
# Gap Analysis: <Campaign Name> - <Issue>

## Problem Statement
<One paragraph describing the gap>

## Evidence
<Minimal reproduction steps that demonstrate the gap>

## Root Cause
<Why existing roadmap doesn't address this>

## Impact Assessment
- Scope: <What's included/excluded>
- Dependencies: <What requires this to be complete>
- Risk: <What could fail if not addressed>

## Proposed Solution
<High-level approach to address the gap>

## Recommendations
1. <Immediate action required>
2. <Architectural consideration>
3. <Future improvements>
```

**Key Requirements:**
- **Evidence-based**: Must have minimal reproduction or concrete demonstration
- **Measurable**: Must define success criteria clearly
- **Root cause**: Must identify why it wasn't in original plan
- **Impact**: Must assess dependencies and risks

### 3. Architecture Report (When Appropriate)

**When to Create:**
- Amended scope is significant (affects multiple tasks)
- Requires architectural decisions
- Changes fundamental design approach

**Report Location:** reports store (directory-tree default: `__reports__/<campaign-name>-arch-factors/`)

**Contents:**
- Design decisions for the amendment
- Trade-offs considered
- Constraints and dependencies
- Migration strategy (if applicable)

**Simpler cases** (bug fixes, minor additions):
- Gap analysis report alone is sufficient
- No separate architecture report needed

### 4. Agent Review

**Review Process:**

The reviewing agent (ANY entity per generic agent model):
1. Reads gap analysis report and validation evidence
2. Reviews architecture report (if created)
3. Confirms the gap is valid and needs roadmap update
4. Approves or rejects/amends the amendment proposal

**Agent Genericness**: The roadmap uses "agent" as a generic role holder. Review can be performed by human, LLM, automated system, or organizational role—whatever entity is most appropriate for your team's structure.

**Review Decision Matrix:**

| Decision | Rationale |
|:---------|:----------|
| Approve | Gap is valid, clearly documented, evidence-based |
| Reject | Gap is not a roadmap issue, can be addressed later |
| Amend | Gap document needs clarification or adjustment |

### 5. Roadmap Update (On Approval)

**Update Scope:**

When approved, update the roadmap to reflect the new work:

1. **Create new task files** at appropriate depth
2. **Update README.md** in affected level:
   - Add nodes to status graph
   - Update Nodes table
   - Log amendment in Amendment Log

3. **Add dependency information**:
   - If new work depends on existing nodes, place deeper in tree
   - Ensure breadth-first execution will complete dependencies first

4. **Preserve existing structure**:
   - Never rename or renumber existing nodes
   - Mark abandoned work as `:::blocked` (don't delete)

**Depth Placement Rules:**

**New leaf at same depth:**
- Add to existing leaves/subdirectories
- Maintain parallelism discipline

**New leaf deeper than existing:**
- Depends on existing work → Place deeper
- Breadth-first guarantees dependencies complete first

**New leaf between depths:**
- Depends on multiple parents at different locations
- Place as deep as all parents
- Example: `deprecation_flags` depends on handler work (depth 3) and output work (depth 2) → placed at depth 4

### 6. Continue Execution

**Execution After Amendment:**

1. **Resume execution** at the next depth level
2. **Check if new node appears earlier**: May need to process immediately
3. **Parallel execution**: New nodes follow same execution rules
4. **Status tracking**: Start with `:::amendment`, transition to `:::done` when complete

---

## Amendment Log Format

Every amended README.md must include an Amendment Log table:

```markdown
## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|
```

**Table Columns:**

1. **ID**: Sequential amendment ID (A1, A2, A3...)
2. **Date**: Date approved (YYYY-MM-DD)
3. **Source**: Path or reference to assessment report in reports store
4. **Nodes Added**: Array of node IDs added
   - Format: `[ "node_a", "node_b" ]`
5. **Rationale**: One-line explanation of why amendment was needed

**Example Entry:**

```markdown
| A1 | 2026-02-20 | oauth2-support-gap-analysis.md | ["google.md", "db-update.md"] | Google auth + schema migration added during implementation |
| A2 | 2026-03-15 | cli-ux-security-audit.md | ["sec-headers.md"] | Security audit revealed missing headers, added to depth 3 |
```

---

## Reopening Closed Work

**Never remove nodes** that were previously completed.

**Deadline/Blocked Work:**

When work is abandoned:
1. Change node status to `:::blocked`
2. Add note in Progress table or nodes table
3. Log in amendment log if a new amendment closes it

**Example:**

```markdown
## Amendments (open)
| A3 | 2026-04-01 | security-verification-gap.md | ["security-header.md"] | Security audit for any other endpoints |
## Amendment Log (closed)
| A2 | 2026-03-15 | cli-ux-security-audit.md | ["sec-headers.md"] | Security audit revealed missing headers, added to depth 3 |
| closed-v1 | 2026-02-28 | migration-completed.md | ["v1-legacy-deprecator.md"] | V1 migration completed, legacy deprecator no longer needed |
```

---

## Common Amendment Patterns

### Pattern 1: Missing Dependency

**Scenario:** Implementation reveals missing prerequisite work

**Solution:**
1. Identify missing component
2. Create report: "Revealed missing validation"
3. Create new leaf at appropriate depth
4. Update README.md

**Example:**
```markdown
# OAuth2 Support (Level: email-config)

## Amendment Log
| A1 | 2026-02-21 | oauth2-github-google-gap.md | ["google.md", "db-fields.md"] |
```

### Pattern 2: Performance Issue

**Scenario:** Optimization revealed during testing

**Solution:**
1. Create performance analysis report
2. Add optimization task at deeper depth
3. Update arrays in status graph.js
4. Log amendment

**Example:**
```markdown
# CLI-UX Normalization (Level: advanced-components)

## Amendment Log
| A1 | 2026-03-01 | cli-ux-performance-audit.md | ["cache-output.md"] | Performance audit showed caching needed for expensive output |
```

### Pattern 3: Security Requirement

**Scenario:** Security audit adds new security layer

**Solution:**
1. Security analysis report in `__reports__/security/`
2. Add security task at appropriate depth
3. Update references
4. Log amendment

**Example:**
```markdown
# API Gateway Redesign (Level: security-layer)

## Amendment Log
| A1 | 2026-04-01 | gateway-fips-req.md | ["tls-mtls.md"] | FIPS compliance requires mutual TLS |
```

### Pattern 4: Technical Debt

**Scenario:** Refactoring needs organizational priority

**Solution:**
1. Technical debt analysis report
2. Create refactoring tasks in new campaign
3. Maintain original roadmap unchanged
4. Link refactoring campaign to original work

---

## Escalation vs. Amendment

**When to Check Escalation Ladder First:**

If unexpected failure occurs:
1. Try Levels 1-4 (see [graph-model.md](graph-model.md) §Failure Escalation Ladder)
2. Only raise amendment after exhausting all levels
3. Use evidence from Level 4 in amendment report

**Example:**

```
Level 1: Check Yourself → TypeError in code (fixable)
Level 2: Check Downstream → noop, no later step
Level 3: Check Upstream → typo in previous task (fix with patch commit)
Level 4: Prove Problem → confirms persistent issue not fixable
   → Prove with minimal reproductiontest
Level 5: Amendment → Gap analysis + review
```

---

## Amendment Review Checklist

Before submitting amendment for review:

- [ ] Gap is valid (matches original scope requirements)
- [ ] Evidence documented with measurable criteria
- [ ] Root cause identified and explained
- [ ] Impact assessed (dependencies, risks)
- [ ] Report location in reports store is correct
- [ ] Depth placement appropriate (deeper if depends on existing work)
- [ ] Node names are descriptive (no numeric prefixes)
- [ ] Proposed plan includes next steps for actual implementation

---

## Tracking Amendment Progress

**Status Progression:**

```
:::amendment (initially added)
    ↓
:::inprogress (when work begins)
    ↓
:::done (when executed and verified)
```

**Status Graph Notes:**

When adding amendment nodes:
1. Use `:::amendment` classDef initially
2. Transition to `:::inprogress` when work starts
3. Transition to `:::done` when complete
4. Never change status to `:::blocked` for an amendment (deprecated nodes should remain as originally planned)

**Nodes Table:**

Amended nodes show status "🔵 Amendment" initially, then "✅ Done" when complete.

---

## Summary

The amendment process ensures:
- **Evidence-based**: All changes validated with reports
- **Proper review**: Formal approval before roadmap changes
- **Consistent structure**: New nodes follow existing patterns
- **Transparent tracking**: Amendment log records all changes
- **Dependency management**: Placement preserves breadth-first execution
- **No destruction**: Existing work preserved, never deleted
