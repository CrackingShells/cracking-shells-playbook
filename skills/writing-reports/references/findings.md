# Findings Reference

Produces a data-first briefing — target 5-minute read — containing actual measurements, tables, statistical results, and professional-quality visualizations. Used to give the PI a clear view of what the numbers show and what decisions they open or close. The visualizations are the primary claims, not decorations for the prose.

---

## Required Front Matter

Every findings report must open with this block:

```
type: findings
topic: <__reports__ subfolder>
date: YYYY-MM-DD
version: vN
prior-version: <path to prior findings report, or "none">
key-metric: <metric name>: <value> <unit> (prior: <value>, delta: ±N)
decision-required: none | confirm | intervene
```

`decision-required` is read before the body. `none` = informational only; `confirm` = PI should acknowledge a direction before work continues; `intervene` = PI action required immediately.

`key-metric` uses the same field names every version so the PI's eye trains on fixed slots. Use `prior: N/A` on the first run.

---

## Sections

| Section | Contents | Required? |
|:---|:---|:---|
| **Headline Result** | Fixed key-value block — not prose. Fields: `metric`, `value`, `unit`, `prior` (or "N/A if first run"), `direction` (up / down / stable / new). Same shape every version. | Required |
| **Results Tables** | The data, directly. Each table gets a short heading. No narrative wrapping — interpretation belongs in Observations. Hard cap: max 20 rows per table; add a summary row if the data exceeds this. | Required |
| **Observations** | Four-column table: `Signal \| Baseline / Expected \| Observed \| Interpretation`. The Baseline column is what earns this section its keep — without it, the section duplicates Results Tables. Cite the source inline in the Observed column: `[source: file / log / instrument]`. | Required |
| **Charts & Visualizations** | When data carries the claim, the visualization IS the claim — make it carry its own weight. Choose the medium that communicates each claim most faithfully: a **rendered graphic** (chart, plot, infographic) produced and saved alongside this report, then embedded via a markdown image link; a **Mermaid diagram** where it cleanly expresses a relationship, flow, state, or schema; or a **structured text / ASCII** form where that conveys the point most directly. All three are first-class — match the medium to what the data needs to say rather than defaulting to whichever is cheapest to type. Whatever the medium, each visualization carries: labeled axes/series with units, statistical annotations where applicable (confidence intervals, p-values, effect sizes, error bars), and a one-line caption. Rendered image files live in the same `__reports__/<topic>/` folder as the report so they travel with it. Omit only if the findings are non-visual by nature (formal proof, pure text analysis). | Required when quantitative data present |
| **Contradictions & Surprises** | ≤5 bullets. Include only when the results contain something unexpected, contradictory, or worth flagging explicitly. Not a summary of the tables — this is the exception list. Drop entirely if nothing surprises. | Conditional |
| **Steering Questions** | ≤5 bullets, ranked. Each item tagged with a temporal tier: `[now]` / `[next run]` / `[later]`. What decisions or directions do these results open up or close off? | Required |
| **Pointers** | Links only — to raw data, prior findings versions, related artifacts, instrument records. No prose. | Required |

## Authoring Notes

- All bullet sections: ≤5 bullets. Results Tables: ≤20 rows per table.
- If `decision-required: none`, the PI may stop reading after Headline Result.
- The distinction between Results Tables (raw data) and Observations (interpreted signals against a baseline) is structural — both are required precisely because they serve different purposes. Do not merge them.
- "Contradictions & Surprises" is a conditional section, not a default summary. If there are no surprises, drop it entirely rather than writing "No surprises this cycle."
- Rendered visualization assets live in the same `__reports__/<topic>/` folder as the report, named after the report with a short descriptive slug (e.g. `01-findings_v0-throughput.png`), and embedded with a relative markdown image link so the report and its figures move together.

---

```markdown
# <Topic> — Findings (vN)

Date: YYYY-MM-DD

---
type: findings
topic:
date: YYYY-MM-DD
version: vN
prior-version: none
key-metric: <metric>: <value> <unit> (prior: N/A, delta: N/A)
decision-required: none | confirm | intervene
---

## Headline Result

metric:
value:
unit:
prior:
direction: up | down | stable | new

## Results Tables

### <Table heading>

| Column | Column | Column |
|--------|--------|--------|
| ... | ... | ... |

## Observations

| Signal | Baseline / Expected | Observed [source] | Interpretation |
|--------|--------------------|--------------------|----------------|
| ... | ... | ... | ... |

## Charts & Visualizations
<!-- Pick the medium that carries each claim best, don't default to one: a rendered graphic saved alongside this report and embedded as ![caption](file.png); a Mermaid diagram; or a structured text / ASCII form. -->
<!-- Every visualization: labeled axes/series + units, statistical annotations where applicable, one-line caption. Save rendered image files in this same folder. -->
<!-- Omit section only if findings are non-visual by nature. -->

## Contradictions & Surprises
<!-- Include only if results contain something unexpected or contradictory. Drop entirely if none. -->
-

## Steering Questions
- [now]
- [next run]
- [later]

## Pointers
-
```
