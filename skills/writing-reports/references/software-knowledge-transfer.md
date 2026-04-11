# Software Knowledge Transfer Reference

## Purpose

A knowledge transfer (KT) report captures what happened during a development cycle and what to do
differently next time. Keep it short and practical — aim for roughly 1–2 pages. Its value is in
surfacing actionable changes for the next cycle, not in chronicling every decision made.

## Recommended Sections

1. **Executive Summary** — what shipped or changed, and the primary outcomes of the cycle
2. **Wins** — things that worked well and should be carried forward
3. **Pain Points** — review friction, failure modes, tooling gaps, and anything that slowed progress
4. **Root Causes** — why problems occurred, not just what the symptoms were
5. **Next-Cycle Changes** — concrete updates to instructions, workflows, checklists, or review
   processes that address the root causes
6. **Artifacts to Preserve** — diagrams, test matrices, decision tables, scripts, or other outputs
   worth keeping for future reference; link rather than embed
7. **Open Questions** — unknowns that should be validated or resolved in the next cycle

## Focus Areas

When writing a KT report, direct attention to these four areas:

- **Prompting and instructions** — which instructions were unclear, missing, or routinely ignored;
  what prompt patterns worked or failed
- **Review process** — where reviews stalled, what feedback was repetitive, and whether the review
  criteria matched the actual quality bar
- **Testing** — where the test plan expanded beyond its original scope, why that happened, and
  whether coverage gaps or unexpected edge cases were the driver
- **Documentation** — what was unclear, missing, or out of date and caused confusion or rework

## What Not to Do

- Do not paste large code blocks into the KT report. Link to the relevant commit, file, or
  analysis-stage artifact instead.
- Do not rewrite the full architecture or reproduce design decisions already documented elsewhere.
  Reference existing artifacts rather than duplicating their content.

---

Adapt sections as needed. Aim for ~1–2 pages.

---

```markdown
# <Topic> — Knowledge Transfer (vN)

Date: YYYY-MM-DD

## Executive Summary
- What shipped / changed:
- Primary outcomes:

## Wins
-

## Pain Points
-

## Root Causes
-

## Next-cycle Changes
- Instruction changes:
- Workflow changes:
- Review process changes:

## Artifacts to Preserve
-

## Open Questions
-
```
