---
applyTo: '**/*'
description: 'Test Definition Reports: format + scope control (risk-driven, trust boundaries, consolidation)'
---

# Reporting: Test Definition (Stage 1)

This document defines how to write stakeholder-reviewable **Test Definition Reports** (typically during Stage 1 — Analysis — per [code change phases](./code-change-phases.instructions.md)).

**Primary goal**: clearly prove that the proposed implementation can be validated with a *risk-driven*, *non-excessive* test plan.

**Non-goal**: a massive enumeration of every possible input combination.

## 1. Relationship to the Testing Standard
The canonical technical testing standard is [testing.instructions.md](./testing.instructions.md) (framework, structure, decorators, execution conventions).

This document focuses on:
- The **report format**
- The **method** for defining tests
- Preventing **over-testing** and **stakeholder review overload**

## 2. Core Principles (must follow)

### 2.1 Test what we own
Test our implementation, not standard library / framework behavior.

Use the trust boundary rules and examples from [testing.instructions.md](./testing.instructions.md).

### 2.2 Risk-driven > exhaustive
Write tests to cover:
- Critical paths
- High-risk failure modes
- Important regressions

Avoid combinatorial explosion unless the system’s logic genuinely depends on it.

### 2.3 Observable behavior > implementation details
Prefer tests that assert public outputs/side effects and externally observable behavior.

### 2.4 Consolidate aggressively
If multiple cases can be covered by a single parameterized/structured test without losing clarity, consolidate.

## 3. What to Test vs What Not to Test (and why)

### 3.1 What to test (healthy targets)
Test cases should map to one of these categories; if a test doesn’t fit, it usually doesn’t belong in the test definition plan.

- **Business rules / invariants you own**
	- Examples: validation rules, merging logic, precedence rules, normalization, idempotency.

- **Boundary behavior and failure modes**
	- Examples: missing/invalid input, partial config, permission errors, timeouts, retries, corrupted data.

- **Integration seams** (your code + dependency interaction)
	- Examples: correct use of `requests` responses, correct invocation of CLI tools, correct parsing/formatting at boundaries.
	- Focus on *your handling* of dependency behavior, not the dependency itself.

- **Critical user workflows (end-to-end)**
	- One test can often cover many units by asserting the observable outcomes of the whole workflow.

- **Regression protection for changed behavior**
	- If you’re modifying existing behavior, add targeted regression tests around the changed surface.

### 3.2 What not to test (common over-testing sources)
These patterns reliably produce “cancerous” test growth without increasing confidence.

- **Stdlib / framework correctness** (trust boundary)
	- Don’t test that a parser, validator, serializer, or test framework behaves as documented.
	- Do test that *you call it correctly* and handle its success/failure outputs.

- **Trivial input permutations**
	- Avoid creating many tests that differ only by unimportant values.
	- Prefer equivalence classes + a small set of representative values.

- **Implementation details and private helpers**
	- Don’t lock in internal refactoring decisions.
	- Assert stable external behavior (public API, file outputs, logs/events if required).

- **Redundant coverage**
	- If a higher-level integration test already proves a behavior end-to-end, don’t also list multiple unit tests for every internal step unless the risk justifies it.

### 3.3 Quick “ownership” test
For each candidate test, answer:
- “If we removed the new/changed code, would this fail?” If **no**, it’s likely testing something you don’t own.

## 4. Anti-Explosion Strategy (required method)
When defining tests, use the strategy below. It is designed to keep the test plan reviewable and bounded.

### 4.1 Start from risks, not functions
1. List the top risks/failure modes (typically 3–8).
2. For each risk, define 1–3 tests that would detect it via observable behavior.
3. Stop when every major risk has coverage.

**Rule**: the test matrix must include a “Risk it covers” column, and every test must map to a risk.

### 4.2 Use equivalence classes + boundary sets
For each input dimension, define:
- **Equivalence classes**: groups of inputs expected to behave the same
- **Boundary values**: min/max/empty/near-empty/near-max

Then choose 1 representative per class + boundary values as needed.

**Example** (string input):
- Classes: empty, normal, oversized
- Boundaries: length 0, length 1, length max, length max+1

### 4.3 Prefer pairwise sampling over full cartesian products
If you have multiple independent input dimensions, avoid enumerating all combinations.

Default approach:
- Use a **pairwise** (2-way) representative set, then add a small number of targeted cases for known risky interactions.

Only escalate beyond pairwise if:
- a specific 3-way+ interaction is a documented risk, or
- the logic is inherently combinatorial.

### 4.4 Make decision tables when logic branches on conditions
If behavior is “if/else” driven by a handful of booleans/enums, represent it as a decision table:
- Rows = representative combinations that exercise each distinct outcome
- Tests = one per row (often parameterized)

This prevents dozens of near-duplicate tests.

### 4.5 Parameterize by default
Prefer one parameterized test with clearly named cases over many near-identical tests.

**Constraint**: parameterization must remain readable. If the parameter table becomes huge, you’re back to explosion—revisit equivalence classes.

### 4.6 Choose the highest-value test tier
To control growth:
- Use **integration** tests to validate workflows and integration seams.
- Use **regression** tests to protect stable public behavior.
- Use **development** tests sparingly as scaffolding, then migrate/remove.

Avoid creating unit-level tests for every helper when an integration/regression test already gives strong confidence.

## 5. Test Definition Self-Review Checklist (required)
Before submitting a test definition report, apply this checklist to each test:
- **Implementation Focus**: Can I name the code change this test validates?
- **Scope Clarity**: Is this testing our code (not stdlib/framework)?
- **Failure Criterion**: Would the test fail if the feature code were removed?
- **Uniqueness**: Is this scenario already covered by another test?
- **Consolidation**: Can it be merged without losing coverage?
- **Value Addition**: Does it add unique value?

## 6. Expected Test Types (what to include)
Use the org’s three-tier model (definitions and decorators live in [testing.instructions.md](./testing.instructions.md)):
- **Development tests**: temporary scaffolding to drive feature work
- **Regression tests**: permanent coverage for existing/critical behaviors
- **Integration tests**: cross-component and end-to-end workflows

In the report, explicitly label each proposed test with:
- Tier: development | regression | integration
- Scope (for integration): component | service | end_to_end
- Required conditions (if any): api keys, external services, skip_ci, slow

## 7. Recommended report format

### 7.1 Executive Summary (required)
- What must be proven for stakeholder approval
- Top 3 risks and how tests address them
- Estimated test volume and where tests will live (dev/regression/integration)

### 7.2 Scope (required)
- In scope / out of scope
- Assumptions
- Dependencies and trust boundaries

### 7.3 Test Matrix (required)
Provide a table. Keep it reviewable.

Template:

| Group | Scenario | Risk it covers | Tier | Setup / Data | Assertion (observable) |
|------:|----------|----------------|------|--------------|-------------------------|
| Core | ... | ... | regression | ... | ... |

**Guidelines**:
- Use 3–4 groups maximum (functional grouping)
- Keep “Setup/Data” and “Assertion” short and precise

### 7.4 Fixtures / Test Data Strategy (required)
- What test data is needed
- Where it will live (e.g., `tests/test_data/...`)
- What is mocked vs real

### 7.5 Observability Requirements (recommended)
If the feature requires logging/metrics/events to validate behavior, list them here.

### 7.6 Minimal “Must-Run” Regression Set (required)
List the smallest set of tests that must run in CI to protect the change.

## 8. Scope control: size heuristics (review guardrails)

### 8.1 Test-to-change ratio sanity check
Use these as guidance (not strict rules):
- Bug fixes: ~2:1 to 3:1
- New features: ~4:1 to 6:1
- Refactors: ~1:1 to 2:1

If the plan exceeds these ranges, include a short justification (risk, criticality, integration complexity).

### 8.2 Red flags
- Many tests that validate framework or stdlib behavior
- Many tests that differ only by trivial input variations
- Tests asserting internal implementation details

## 9. Examples: healthy vs explosive test definitions

### 9.1 Explosive (avoid)
- “Test every flag combination for the CLI” (cartesian product explosion)
- “Test that JSON serialization works” (stdlib behavior)
- “Test every private helper” (implementation detail coupling)

### 9.2 Healthy (prefer)
- Define 3–5 equivalence classes for inputs + boundary values, then pick representatives.
- Add 1–2 end-to-end integration tests that cover the critical workflow.
- Add 2–6 regression tests for key behaviors and failure modes.
- Use parameterization to cover representative cases without multiplying test count.

## 10. Output and execution conventions (during execution)
When tests are eventually executed, capture results via Wobble log files as defined in [testing.instructions.md](./testing.instructions.md).

In the test definition report, it’s sufficient to state which Wobble commands you expect to run (high level), without pasting huge outputs.
