# Gap Analysis: roadmap-mcp-rework — no milestone owns the domain model (v0)

Date: 2026-11-16

Source of amendment A1 of `__roadmap__/roadmap-mcp-rework/`. Evidence: [00-observation_v0.md](00-observation_v0.md) and [00-open_question_v0.md](00-open_question_v0.md). Approved by Eliott on 2026-11-16 in the session that wrote this report.

## Problem Statement

R01 marks its data model provisional and hands it to M1, but M1 is scoped as store, lock and journal. No milestone is scoped to settle status meanings, the stored schema, or the read model every consumer uses. M0 also tests a data model that is not settled and does not probe the MCP Apps surface that the views depend on.

## Evidence

- The legacy grammars (`leaf.bnf`, `readme.bnf`) carry per-criterion state, a step title, the breaking mark of a commit and a reference shape that R01's sketch drops (table in the observation).
- `blocked` is terminal in `graph-model.md` and resumable in R01. `amendment` is the birth state of an added node in the skill and a reopened state in R01. Neither change has a recorded decision.
- The meta-roadmap's Gotchas record that M1 owns the schemas, M2 order and amendments, M4 the tool outputs and M6 the app-only tool, with no owner for the shape they share.
- R01 plans one shared version across three crates, which conflicts with a schema versioned on its own.

## Root Cause

The plan was cut by consumers and layers (store, engine, interchange, consumers, delivery). Domain facts that every layer reads fell between the layers. The data model was sketched in R01 and deferred to the first milestone that writes files, which is the wrong owner for a contract that M2, M3, M4, M5 and M6 all depend on.

## Impact Assessment

- Scope: one new milestone before M1, one new crate, and scope edits to M0, M1, M2, M4, M6, M7 and the meta-roadmap README.
- Dependencies: M1 now needs the new leaf done. The ordering rule (open question) is an input and is not decided here.
- Risk if not addressed: M1 freezes field names while status semantics are open, and the views get built on a read model that nobody specified.

## Proposed Solution

Amend, do not rewrite. Every node is `planned`, the skeleton is sound, and the meta-roadmap is itself evidence for the ordering question.

| Change | Detail |
|:--|:--|
| New leaf | `m0b_domain_model_spec.md`, a root sibling of M0. It runs in parallel with M0 and before `core/`. It owns status meanings, the stored schema inventory, the read model and the `rdm-schema` crate. |
| M0 | Reduced to a feasibility check against a throwaway fixture: render, a full view, one leaf's content, host capabilities and context, time to display. It no longer depends on the schema. |
| M1 | Keeps store, lock, journal, revisions, the kill-injection test, the error type and id minting. Loses schema design and generation. Depends on `rdm-schema`. |
| M2 | Takes status semantics and the ordering rule as given inputs. |
| M4, M6 | Their outputs and the app-only tool use the read model. |
| M7 | Two version lines: `rdm-schema` alone, and core, mcp and cli on one product version. Publish order: schema, core, mcp and cli, binaries, plugin. Publication stays at M7. |
| R01 | Executive Summary (crates), §Data Model, §Deliberately open, §Launch and distribution; I1 only if the ordering decision changes it. |

Decisions taken by Eliott: amend; the crate is `rdm-schema`; publish at M7 with the others; the leaf keeps the label M0b because the roadmap CLI cannot re-index; fixture, message details and run counts belong to M0's own plan.

## Recommendations

1. Land A1 on the meta-roadmap with this report as its source.
2. Run the open question's count over the eight campaigns and the meta-roadmap before M0b's first step.
3. Add a reminder for M0b and add it to the `after:` field of `skills-rdm-M1`. Reminder notes are append-only and change only on Eliott's yes, so this report does not do it.
4. Treat the legacy tool's lack of an amendment writer as a known gap: the Amendment Log row of A1 is written by hand and checked by `dirtree-rdm validate`.
