# Software Test Definition Reference

A test definition report specifies what to test, why, and how — before any code is written. It is the planning artifact that keeps test suites small, purposeful, and aligned to real risk.

## Table of Contents

1. [Workflow Checklist](#workflow-checklist)
2. [Core Principles](#core-principles)
3. [What to Test / What Not to Test](#what-to-test--what-not-to-test)
4. [Anti-Explosion Strategy](#anti-explosion-strategy)
5. [Self-Review Feedback Loop](#self-review-feedback-loop)
6. [Test Tiers](#test-tiers)
7. [Report Format](#report-format)
8. [Template](#template)

---

## Workflow Checklist

Copy this checklist and check off items as you work.

```
[ ] 1. List top risks for the change — aim for 3–8 distinct risks.
[ ] 2. Map 1–3 tests per risk — resist mapping more until the list is complete.
[ ] 3. Apply equivalence classes and boundary sets to collapse redundant cases.
[ ] 4. Self-review each proposed test against the feedback loop (see below).
[ ] 5. Draft the report sections: executive summary, scope, test matrix, fixtures.
[ ] 6. Save the finished report to __reports__/<topic>/test-definition.md
```

---

## Core Principles

**Test what we own.** Only write tests for code this project controls. Third-party libraries, language runtimes, and framework internals are out of scope.

**Risk-driven over exhaustive.** A test suite exists to catch regressions and validate behavior against real risks — not to prove that every line executed. Start from the risk list, not the function list.

**Observable behavior over implementation details.** Tests must assert on outputs, side effects, and state changes that a caller can observe. Asserting on private internals makes tests brittle and couples them to implementation choices.

**Consolidate aggressively.** When two scenarios share the same setup and differ only in a parameter value, parameterize them. When two scenarios cover the same risk, keep the more expressive one and delete the other.

---

## What to Test / What Not to Test

### Healthy targets

- Business rules and invariants introduced or changed by this work
- Boundary behavior and failure modes at input edges
- Integration seams: the points where this code hands off to another component
- Critical user workflows that must not regress
- Regression protection for bugs that have occurred before

### Over-testing sources to avoid

- Standard library and framework correctness
- Trivial input permutations that do not probe distinct behavior
- Implementation details (private methods, internal data structures)
- Redundant coverage that duplicates an existing passing test

### Ownership check

Before adding a test, ask: "If I removed the new or changed code, would this test fail?" If the answer is no, the test does not belong to this change.

---

## Anti-Explosion Strategy

Test suites grow until they become a maintenance burden. The following practices keep counts under control.

**Start from risks, not functions.** List 3–8 risks for the change. Each risk gets 1–3 tests. Do not enumerate tests by counting methods or branches.

**Equivalence classes and boundary sets.** Group all inputs that produce the same behavior into one class and pick one representative. For numeric or ordered ranges, add one test at the lower boundary, one at the upper boundary, and one inside the valid range.

**Pairwise over cartesian products.** When two independent dimensions each have N values, the cartesian product is N². Pairwise testing reduces this to O(N) while catching most real-world interactions.

**Parameterize by default.** When two or more tests share identical setup and assertion structure and differ only in their inputs or expected outputs, write one parameterized test instead of N separate ones.

**Size heuristics.**

| Change type | Expected test count |
|-------------|---------------------|
| Bug fix | 2–3 tests |
| New feature | 4–6 tests |
| Refactor | 1–2 tests per changed unit |

If a proposed suite significantly exceeds these bounds, revisit the risk list and apply consolidation before proceeding.

---

## Self-Review Feedback Loop

Apply this checklist to each proposed test before submitting the report. Revise any test that fails a check — do not move on until every test passes every item.

```
[ ] Implementation Focus  — Does the test assert on observable behavior, not internal state?
[ ] Scope Clarity         — Is it obvious from the test name and scenario what behavior is under test?
[ ] Failure Criterion     — If the code is wrong, will this test actually fail?
[ ] Uniqueness            — Does this test cover a risk or case not already covered by another test?
[ ] Consolidation         — Could this test be merged with a sibling into a parameterized case?
[ ] Value Addition        — Would removing this test leave a meaningful gap in coverage?
```

---

## Test Tiers

These definitions replace any external reference to tier definitions. Use the `Tier` column in the test matrix to label each test.

**Development** — Temporary scaffolding written to drive feature work. These tests are useful during development but are deleted or promoted once the feature is complete. They do not ship as permanent suite members.

**Regression** — Permanent coverage for critical behaviors. Once added, a regression test is never deleted without explicit justification. It runs on every CI execution and gates merges.

**Integration** — Tests that cross component or service boundaries. Label each integration test with its scope:

| Scope label | Meaning |
|-------------|---------|
| `component` | Two or more modules within the same service |
| `service` | Two or more services communicating over a network or queue |
| `end_to_end` | Full user journey from entry point to observable outcome |

Integration tests may be slower and depend on external state. Isolate them in a dedicated suite so they can be run selectively.

---

## Report Format

A test definition report has the following sections.

### Executive Summary

State what must be proven by this test suite, enumerate the top risks and how the proposed tests address each one, and give the estimated test volume broken down by tier.

### Scope

Four sub-sections:

- **In scope** — what behaviors and components this report covers
- **Out of scope** — what is explicitly excluded and why
- **Assumptions** — preconditions the tests rely on
- **Trust boundaries** — which external systems are mocked vs. exercised for real

### Test Matrix

One row per scenario. Use this header exactly:

```
| Group | Scenario | Risk it covers | Tier | Setup / Data | Assertion (observable) |
```

The `Group` column clusters related scenarios. The `Assertion` column must describe a result a caller can observe — not an internal state change.

### Fixtures / Test Data Strategy

Describe the data the tests need, where it lives (conventionally `tests/test_data/…`), and the mock-vs-real decision for each external dependency.

### Minimal Must-Run Regression Set

A short named list of the tests that must pass before any merge. This is the subset of regression tests that covers the highest-value scenarios with the least execution time.

---

## Template

---

Adapt sections as needed.

```markdown
# <Topic> — Test Definition (vN)

Date: YYYY-MM-DD

## Executive Summary
- What must be proven:
- Top risks and how tests address them:
- Estimated test volume and tiers (dev/regression/integration):

## Scope

### In scope

### Out of scope

### Assumptions

### Trust boundaries

## Test Matrix

| Group | Scenario | Risk it covers | Tier | Setup / Data | Assertion (observable) |
|------:|----------|----------------|------|--------------|-------------------------|
| Core | ... | ... | regression | ... | ... |

## Fixtures / Test Data Strategy
- Data needed:
- Where it lives (`tests/test_data/...`):
- Mock vs real strategy:

## Observability Requirements
- Capture test execution output as log files to validate behavior at runtime.
- List any logs, metrics, or events that must be present for a test to be considered passing.

## Minimal Must-Run Regression Set
- Test 1:
- Test 2:

## Scope Control Notes
- Consolidations made:
- Justification for any unusually large test count:
```
