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

All patterns follow the same cycle. Trigger determines when to raise the amendment.

| Pattern | Trigger | Report focus |
|:--------|:--------|:-------------|
| Missing Dependency | A required task was omitted from the original plan | Dependency gap analysis |
| Performance Issue | A step reveals unexpected performance constraints | Benchmark evidence + mitigation plan |
| Security Requirement | A security concern surfaces mid-execution | Threat assessment + remediation nodes |
| Technical Debt | Existing code blocks progress beyond planned scope | Debt scope + cleanup tasks |

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
:::amendment (birth state — newly-added nodes only)
    ↓
:::inprogress (when work begins)
    ↓
:::done (when executed and verified)
```

`:::amendment` is the **initial status assigned to newly-added nodes** created during an amendment cycle. It exists solely to visually distinguish recently approved additions from pre-existing nodes. **Existing nodes are never transitioned to `:::amendment`**; they remain in whatever state they are currently in.

**Status Graph Notes:**

When adding amendment nodes:
1. Use `:::amendment` classDef on creation — this is the starting state for new nodes only
2. Transition to `:::inprogress` when work starts
3. Transition to `:::done` when complete
4. Do **not** retroactively mark a node `:::blocked` solely because a later amendment supersedes it. Only mark a node `:::blocked` when its work is genuinely impossible to complete. Abandoned amendment-added nodes follow the same rule as any other node: if the work is truly abandoned, mark it `:::blocked`.

**Nodes Table:**

Amended nodes show status "🔵 Amendment" initially, then "✅ Done" when complete.

