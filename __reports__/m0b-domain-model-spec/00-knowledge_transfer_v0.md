# M0b domain model spec: Knowledge Transfer (v0)

Date: 2026-11-16

## Executive Summary

| Item | Outcome |
|:--|:--|
| Shipped | Four spec docs in `__design__/rdm/` (statuses, stored schema, read model, traceability), and the `rdm-schema` crate |
| Reviewed | One adversarial verifier, then Eliott's review. Both found real defects |
| Rulings from review | Criteria are enforced (reversing an agent decision). Versions become three-part `0.X.Y` |
| Status | Both rulings are decided and being applied by the amendment node `read/crate/review/`, running in parallel with this report |

## Wins

| Win | Detail |
|:--|:--|
| Four spec docs | Statuses, stored schema, read model, traceability, with their Decisions sections |
| `rdm-schema` crate | 22 tests, committed JSON Schemas, and a drift test that also catches orphan schema files |
| Traceability checker | Covers all 79 grammar rules and checks stored targets against `stored.rs` |
| Legacy finding | The legacy validator silently read 1 of 14 Progress rows |

## What the adversarial verifier caught (all fixed)

| Defect | Fix |
|:--|:--|
| Order of unknown keys lost | `preserve_order` enabled |
| Hollow traceability checker | Targets now checked against `stored.rs` |
| Bulleted References in legacy leaves | Grammar coverage extended |
| Loose version patterns, parsing not strict | Patterns pinned, strict parsing |
| Null rule undocumented | Documented in the stored schema |
| Orphan schema files | Drift test fails on them |

## Pain Points

| Pain point | Fix for next time |
|:--|:--|
| **Main lesson: Eliott's review reversed one agent decision.** The specs read "30 of 48 done leaves carry an open gate" as licence to make `met` informational | Data showing a rule is broken in practice is evidence of poor upkeep, not a reason to drop the rule. Escalate such inferences instead of encoding them |
| The verifier missed that semantic error | It attacked consistency, not intent. Brief verifiers to challenge any rule an agent relaxed |

### Eliott's rulings

| Topic | Ruling |
|:--|:--|
| Criteria | Pre-conditions are enforced before `ongoing`, gates before `done`, and every mark is traced (who, when, evidence). Legacy done leaves with open gates become `validate` errors to reconcile, never auto-ticked |
| Versions | Three-part `0.X.Y`. Bumps come from commit types and scopes through the repo's existing multi-semantic-release setup. M7 wires the crate in |

## Root Causes

- An agent inferred a rule change from observed data, with no step forcing escalation.
- Verification checked internal consistency, so intent errors passed.

## Next-Cycle Changes

| Area | Change |
|:--|:--|
| Review process | The Eliott-review gate caught what the verifier missed. Brief verifiers to challenge any rule an agent relaxed |
| Instructions | Agents escalate any inference that relaxes a rule, instead of encoding it |

## Artifacts to Preserve

- `__design__/rdm/statuses.md`, `stored-schema.md`, `read-model.md`, `traceability.md`
- `crates/rdm-schema` and its committed JSON Schemas
- `tools/check_rdm_traceability.py`

## Open Questions

- Confirm the amendment node `read/crate/review/` lands both rulings in the specs and the crate.
