# rdm domain model review — Open Question (v0)

Date: 2026-11-16

---
type: open-question
question: Should roadmap ordering stay implicit in directory depth, move to explicit dependency edges, or combine both with one as default?
domain: code
date-opened: 2026-11-16
status: open
spawned-from: __reports__/rdm_domain_model_review/00-observation_v0.md
closure-type: both
---

## The Question

Should roadmap ordering stay implicit in directory depth, move to explicit dependency edges, or combine both with one as default?

## Handoff Contract

- Receiving agent is asked to: gather evidence from the eight legacy campaigns and the rework meta-roadmap on how often depth forces an ordering the work does not need, and fill the comparison table below with it.
- Receiving agent must NOT: choose an option, edit R01 or the legacy skill text, or implement any edge field or validation.

## Context

How order works today, and what is known:

| Aspect | Known |
|:--|:--|
| R01 rule | A leaf is ready when it is not done and every leaf of smaller depth is done. The rule is global, across unrelated branches. |
| Legacy skill rule | Order is scoped to siblings within one parent; edges between siblings are forbidden by the BNF. |
| What depth gives | Order visible in the tree, no extra field, a simple validator, parallel work per level. |
| Cost seen in this repository | The rework meta-roadmap notes that M6 needs only M4 but waits on M5; the exact edges are kept in reminders outside the model. |
| Stated position | Depth-only order is weak but works rather well. Applying things per depth is a property of topological layering, so the question is a real discussion. |
| Layering under edges | If order came from explicit edges, levels remain computable as topological generations (longest path from the start), so "per level" operations would still exist. The level would be derived instead of read from the path. |
| Consumers | A read model that emits a `level` per leaf hides where that level comes from. |

## Option Landscape

### Option A — Depth only (status quo)
- Description: order is the depth of a node's path; no edge field exists.
- Benefit: order is visible in the directory tree and needs no extra field.
- Cost: work is over-sequenced where one node only needs one other, and exact dependencies live outside the model.
- Unknown: how many nodes in the eight legacy campaigns are nested only to wait.

### Option B — Explicit edges, depth derived
- Description: each node lists the ids it waits for; the level is the topological generation; nesting only groups.
- Benefit: exact dependencies, with parallel work per level still available.
- Cost: changes the stored model, the operations and the legacy import, and adds cycle validation.
- Unknown: how legacy depth translates into edges, and whether authors can maintain them.

### Option C — Depth by default, optional extra edges
- Description: depth orders nodes unless a node lists extra ids it waits for.
- Benefit: legacy roadmaps keep their meaning and exact edges appear only where needed.
- Cost: two sources of truth, and a rule is needed when an edge contradicts depth.
- Unknown: whether the conflict rule can stay simple, for example edges may only add waits.

## Comparison

| Option | Complexity | Reversibility | Fit with legacy data | Effect on consumers | Cost to learn more |
|--------|-----------|--------------|----------------------|---------------------|--------------------|
| A | low | high: can grow into C later | exact | none | medium: count forced nestings |
| B | high | low: changes stored model and import | needs a translation rule | stored schema and operations change; a read model hides levels | medium: prototype the translation |
| C | medium | medium | exact | stored schema gains an optional field; operations gain edge operations | medium: define the conflict rule |

*What this table does not settle:* how often over-sequencing occurs in real campaigns, how much weight the visible-in-the-tree property deserves, and whether agents or humans will maintain edges reliably.

## What Would Close This Question

[Both]: evidence of how often depth forces needless ordering across the eight legacy campaigns and the meta-roadmap, plus Eliott's choice of option.

## Next Step to Reduce Uncertainty

Count, in the eight legacy campaigns and the meta-roadmap, the nodes whose depth is greater than their stated prerequisites require, and bring the counts back.
