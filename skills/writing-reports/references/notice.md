# Notice Reference

Produces a lightweight, fast-to-write capture of an improvement idea or enhancement noticed outside the current task's scope. Domain-agnostic. Designed to be triaged in under a minute and cheap to decline.

---

## Required Front Matter

Every notice report must open with this block:

```
type: notice
title: <≤10 word noun-phrase or imperative>
tag: improvement | gap | experiment | open-hypothesis
effort: S | M | L
reversibility: <one word or short phrase>
evidence: anecdotal | observed | verified
spotted-during: <one-line description of the active task when this was noticed>
date: YYYY-MM-DD
status: open | accepted | declined | deferred
```

The front matter is the triage surface. An agent reading only the front matter can make an accept/decline/defer decision without opening the body.

`evidence`: `anecdotal` = noticed without direct verification; `observed` = directly seen during this session; `verified` = checked against a test, source, measurement, or prior record.

`tag`: `improvement` = making something better; `gap` = something missing; `experiment` = an idea worth testing; `open-hypothesis` = a hunch that needs investigation before it can even be a proposal.

---

## Sections

| Section | Contents | Required? |
|:---|:---|:---|
| **TL;DR** | Exactly 3 bullets, hard cap. Bullet 1: the idea (≤15 words). Bullet 2: why surface it now (≤15 words). Bullet 3: the primary cost or risk if accepted (≤15 words). No prose. | Required |
| **Context** | ≤4 lines. The situational reasoning behind *why this matters now* — not where it was spotted (that is in `spotted-during` front matter), but what makes the timing significant. Omit if bullet 2 of TL;DR already captures the timing fully. | Optional |
| **Scope Delta** | Bulleted pointers — what would shift if accepted, one line each. No full designs, implementations, or methodology. Pointers only. | Required |
| **Accept / Decline** | Required two-column table. `Accept`: benefit gained + cost incurred. `Decline`: what stays stable + cost avoided. The Decline column must name what the current approach does well that this notice might disturb — not generic filler. Symmetric specificity in both columns. | Required |
| **If Accepted — Next Step** | One line: the concrete next action, for a human or an agent. Examples: "open a dedicated thread and produce an open-question report," "run a scoped experiment," "commission an architecture analysis." | Required |
| **If Declined — Next Step** | One line: what happens on decline. Examples: "archive and move on," "revisit after X milestone," "blocked until Y is resolved." Closes the decline path explicitly — this is what makes declining feel low-cost. | Required |

## Authoring Notes

- The Verdict Block (tag, effort, reversibility, evidence) lives in front matter, not in the report body. Do not reproduce it as a section.
- "Where this came from" lives in the `spotted-during` front matter field, not in the body. The body starts with TL;DR.
- The report is approximately one page. If it is growing beyond that, it has become an architecture or open-question report — escalate via the "If Accepted — Next Step" field.
- The Accept / Decline table achieves its purpose only when both columns are equally specific. Vague Decline cells ("no immediate benefit lost") defeat the symmetric framing and make declining feel costly.

---

```markdown
# <Topic> — Notice (vN)

Date: YYYY-MM-DD

---
type: notice
title:
tag: improvement | gap | experiment | open-hypothesis
effort: S | M | L
reversibility:
evidence: anecdotal | observed | verified
spotted-during:
date: YYYY-MM-DD
status: open
---

## TL;DR
- <!-- Idea: ≤15 words -->
- <!-- Why now: ≤15 words -->
- <!-- Primary cost or risk if accepted: ≤15 words -->

## Context
<!-- ≤4 lines. Why the timing is significant. Omit if TL;DR bullet 2 is sufficient. -->

## Scope Delta
<!-- Bulleted pointers — what shifts if accepted. One line each. No implementations. -->
-

## Accept / Decline

| | Accept | Decline |
|---|---|---|
| **Benefit** | | |
| **Cost** | | |

## If Accepted — Next Step

## If Declined — Next Step
```
