# Observation Reference

Produces a contextual snapshot of a peripheral observation, sufficient for a completely fresh agent thread to orient and begin work without being cold. Forensic content (root-cause theory, suggested directions) is included when the writing agent already has it — never demanded.

---

## Required Front Matter

Every observation report must open with this block:

```
type: observation
topic: <__reports__ subfolder>
spotted-during: <one-line description of the active task when this was noticed>
date: YYYY-MM-DD
domain: <lab / code / writing / data / other>
confidence: hunch | plausible | confirmed
urgency: low | medium | high
deferred-because: <one line: why this was not addressed in-thread>
```

`confidence` reflects the writing agent's certainty about the observation itself: `hunch` = something seemed off, unverified; `plausible` = consistent with available evidence; `confirmed` = directly verified.

---

## Sections

| Section | Contents | Required? |
|:---|:---|:---|
| **What Was Noticed** | The observation itself. One short paragraph or ≤5 bullets. What caught attention, where, in what context. Do not diagnose — describe. | Required |
| **Context** | What was being worked on when this was spotted. What phase the primary task was in. Why this was deferred (expands on the front matter `deferred-because` if more than one line is needed). | Required |
| **Location Map** | Whatever a cold agent needs to find the relevant material: files, line ranges, dataset identifiers, experiment records, notebook references, artifact paths. One entry per line. No prose — pointers only. | Required |
| **Evidence** | What the writing agent already has in hand: log lines, failed assertions, data anomalies, source quotes, instrument readings. Domain-specific — not limited to software artifacts. | Required |
| **Re-observation Steps** | How to observe this again, if re-observation is possible in this domain. Minimal steps only. Omit entirely if the observation is not reproducible on demand (one-time experimental run, transient state, document snapshot, etc.). | Optional |
| **Hand-off Questions** | Bulleted list of what a fresh agent picking this up would need to investigate or decide first. If the writing agent already holds a root-cause theory or suggested direction, state it here as a prefatory note before the questions: *"Working theory, if any: …"* | Required |
| **Scope Boundary** | One sentence stating what this report does not authorize the receiving agent to do. Prevents the fresh agent from touching the primary work thread. | Required |

## Authoring Notes

- The primary purpose is orientation for a cold agent, not a diagnosis for the current agent.
- Keep the report brief enough to produce mid-work without breaking focus on the primary task.
- "What Was Noticed" leads — the cold agent reads it first. Make it self-contained in one reading.
- The "Location Map" is the most valuable navigation aid. Populate it fully even when Evidence is thin.
- "Scope Boundary" is not optional regardless of how obvious it seems. State it explicitly.

---

```markdown
# <Topic> — Observation (vN)

Date: YYYY-MM-DD

---
type: observation
topic:
spotted-during:
date: YYYY-MM-DD
domain: lab | code | writing | data | other
confidence: hunch | plausible | confirmed
urgency: low | medium | high
deferred-because:
---

## What Was Noticed

## Context

## Location Map

## Evidence

## Re-observation Steps
<!-- Omit this section if re-observation is not possible in this domain. -->

## Hand-off Questions
<!-- Working theory, if any: … -->
-

## Scope Boundary
<!-- One sentence: what the receiving agent must not touch. -->
```
