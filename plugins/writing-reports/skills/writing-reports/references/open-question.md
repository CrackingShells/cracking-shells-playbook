# Open-Question Reference

Produces a structured framing of an open question for hand-off to a fresh agent thread. Does not make a decision. Does not propose a direction. Terminal output is a next step for reducing uncertainty, not a recommendation.

---

## Required Front Matter

Every open-question report must open with this block:

```
type: open-question
question: <one-line restatement of the question>
domain: <lab / code / writing / data / other>
date-opened: YYYY-MM-DD
status: open | in-progress | closed | abandoned
spawned-from: <topic path or free-text description of the originating session>
closure-type: decision | evidence | both
```

`closure-type` states what kind of resolution would close this question: `decision` = someone chooses a direction; `evidence` = a result or measurement settles it; `both` = needs both.

`spawned-from` allows the receiving agent to trace back to the originating context. Use a `__reports__` path when available; otherwise a short free-text description.

---

## Sections

| Section | Contents | Required? |
|:---|:---|:---|
| **The Question** | One sentence. Must be interrogative — end with `?`. Hard cap: ≤25 words. The discipline of the cap transforms a vague topic into a precise, delegatable question. | Required |
| **Handoff Contract** | Two lines. Line 1: what the receiving agent is asked to produce or do. Line 2: what it is explicitly NOT asked to do. The second line is the scope boundary — it prevents the fresh agent from drifting into the originating task or over-building. | Required |
| **Context** | Covers both situational context (what was happening when this question arose) and epistemic context (what is already known). At least one structured artifact is required — a table, timeline, diagram, annotated list — in whatever form suits the domain. No Mermaid prescription; no prose-only paragraphs. | Required |
| **Option Landscape** | One subsection per identified option (`### Option A — <name>`): one-line description + one benefit + one cost + one unknown. Symmetric depth required across all options — same number of elements per option to prevent implicit advocacy through differential detail. **If the option space is not yet known, replace this section entirely with a `Known Unknowns` section** listing what must be discovered before options can be enumerated. | Optional / Conditional |
| **Comparison** | Required only if Option Landscape is present with ≥2 options. Options as rows × evaluation axes as columns. Axes are domain-specific — choose axes that fit the question. Suggested starting axes (adapt or replace): complexity, reversibility, confidence, fit with existing constraints, cost to learn more, evidence quality. Below the table, add a required note: *"What this table does not settle: …"* | Conditional |
| **What Would Close This Question** | States the exit condition explicitly. Label the closure type inline: `[Decision]`, `[Evidence]`, or `[Both]`. What information, result, or choice would make this question no longer open? | Required |
| **Next Step to Reduce Uncertainty** | One line. Not "adopt X." What to do next to move from open to closed — run experiment Y, consult source Z, commission a findings report on metric M, produce an architecture analysis of option B. This section does not recommend an option. | Required |

## Authoring Notes

**This report does not conclude.** It does not recommend an option. Writing the Comparison table is an analytical act, not a decision act — the table must explicitly note what remains unsettled after comparison. The Option Landscape must use symmetric depth; writing one option with more enthusiasm or detail than others is implicit advocacy.

The "Next Step to Reduce Uncertainty" section names an action that narrows the question, not one that implements a direction. If the agent finds itself writing "implement option B," it has crossed into architecture report territory — stop and use the `architecture` report type instead.

When the option space is genuinely unknown, the `Known Unknowns` section is the honest alternative to a fabricated Option Landscape. Hallucinated options are worse than no options.

---

```markdown
# <Topic> — Open Question (vN)

Date: YYYY-MM-DD

---
type: open-question
question:
domain: lab | code | writing | data | other
date-opened: YYYY-MM-DD
status: open
spawned-from:
closure-type: decision | evidence | both
---

## The Question
<!-- One sentence, interrogative, ≤25 words, ends with ? -->

## Handoff Contract
- Receiving agent is asked to:
- Receiving agent must NOT:

## Context
<!-- At least one structured artifact (table, timeline, diagram, annotated list) in domain-appropriate form. -->
<!-- Cover both: what was happening when this arose, and what is already known. -->

## Option Landscape
<!-- Use this section if options are already identified. One subsection per option, symmetric depth. -->
<!-- If options are not yet known, replace this entire section with a Known Unknowns section. -->

### Option A — <name>
- Description:
- Benefit:
- Cost:
- Unknown:

### Option B — <name>
- Description:
- Benefit:
- Cost:
- Unknown:

## Comparison
<!-- Required only if Option Landscape has ≥2 options. -->
<!-- Choose axes appropriate to the domain — do not use the suggested axes if they don't fit. -->

| Option | Complexity | Reversibility | Confidence | Fit with existing constraints | Cost to learn more |
|--------|-----------|--------------|------------|-------------------------------|-------------------|
| A | | | | | |
| B | | | | | |

*What this table does not settle:*

## What Would Close This Question
<!-- State the exit condition. Label closure type: [Decision], [Evidence], or [Both]. -->
[Decision / Evidence / Both]:

## Next Step to Reduce Uncertainty
<!-- One line. Not "adopt X." What action narrows this question toward closure? -->
```
