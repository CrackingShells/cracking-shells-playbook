# Reference — `identity` layer

## Contents

- Purpose
- When to use
- Degrees of freedom
- Required sections (four pillars)
- Optional sections (with inclusion rules)
- Writing style
- Template
- Worked mini-example
- Self-audit checklist

## Purpose

The `identity` layer codifies *why this visual system exists and what it feels like*. It is read by stakeholders, by new team members on day one, and by the authors of the downstream layers when they need to remember what the system is *for*. It is **not** the place for hex codes, font sizes, or production rules — those live in `token-vocabulary` and `actionable-guidelines`.

A weak `identity` produces canons whose downstream rules feel arbitrary. A strong `identity` is the reason a token name like `--wet` or a constraint like "orthogonal grids, sharp corners" feels inevitable rather than imposed.

## When to use

Open this reference when:

- The user has no clear visual image yet and needs to commit to tone, atmosphere, personality before tokens or rules can be defined.
- The user has a vague visual hunch and wants to anchor it in named decisions before downstream layers.
- An existing visual system needs its implicit identity extracted and named for the first time (formalisation pass).

Skip this reference if the user already has a clear visual sense and wants to go directly to `token-vocabulary` or `actionable-guidelines`. Identity is optional.

## Degrees of freedom

**High.** The author has wide latitude on metaphor, register, and structure within each pillar. This reference enforces *which questions get answered*, not *how poetically*. A scientific-instrument canon's identity is terse and precise; a poetry-app canon's identity is allowed metaphor.

## Required sections — four pillars

The produced `identity.md` is organised into four pillars. Group related decisions; do not flatten into a linear list. This grouping is what makes the produced doc scannable for a stakeholder.

### Pillar A — Character (the *who*)

- **A1. Personality archetype** — 1–3 sentences. Concrete and anchored, not abstract. ("Patient lab technician." "Studious librarian." Not: "professional and trustworthy.")
- **A2. Negations** — what the system is *not*. Often clarifies more than positives. ("Not flashy. Not corporate-modern. Not nostalgic.")
- **A3. Adjacent moods** — 2–4 metaphorical anchors that triangulate the system. Each is a brief phrase, not a paragraph.

### Pillar B — Sensory anchor (the *what it feels like*)

- **B1. Material / sensory grounding** — the tangible metaphor the system anchors in. Examples from prior canons: "wet, dry, and a band of water"; "parchment, water, and a single gleam". Required because it is the most-skipped section in weak canons and the most reliable section for downstream coherence. **Authors who skip this produce canons whose downstream rules feel arbitrary.**
- **B2. Tone & atmosphere statement** — one paragraph. The vibe the visual system must transmit. Not a list of adjectives; a single coherent sentence or two.

### Pillar C — Visual invariants that carry voice (the *what it looks like at the highest level*)

This pillar lives in `identity` (not `actionable-guidelines`) because these invariants *carry voice*. They are not yet the prescriptive rules of step-by-step production — they are the high-level decisions about how the system *behaves*.

- **C1. Geometry & compositional constraints** — high-level rules. ("Four by four, asymmetric, never centred." "Orthogonal grids, sharp corners." "Always one focal element on the page.") Prescriptive numerical details belong in `actionable-guidelines`.
- **C2. Motion philosophy** — whether motion is a one-shot reveal, a continuous organism, or absent. If motion is present, name the tier structure (idle / reveal / processing) and the master cycle duration if applicable.
- **C3. Cross-surface scalability** — how the system adapts from large-format display down to icon size. What survives, what is dropped at each step.
- **C4. Simplified / cropped fallback** — what the system reduces to when space is constrained. The minimum recognisable form.

### Pillar D — Verbal voice

- **D1. Voice & tone of language** — sentence rhythm, register. ("Short, declarative, slightly mechanical." "Considered sentences. No hype.") Not the full copywriting guide; the voice principle.

## Optional sections — with inclusion rules

Each optional section is gated by an explicit *include when* rule. Do not include an optional section if its condition does not fire.

| Optional section | Include when … |
|:---|:---|
| **Phonetic / semantic resonance of the name** | The system has a chosen name whose sound or meaning meaningfully informs the visual hunch. Skip if the name is functional or generic. |
| **Tagline candidates** | The system will have user-facing taglines or surface text (web product, brand, publication). Skip for internal-only systems. Present multiple candidates without ranking. |
| **Token seeds** | The author already has token-name hunches and wants to record them before the next layer. Otherwise let them emerge naturally in `token-vocabulary`. |
| **Tensions & resolutions** (Risk/Mitigation pattern) | The canon is in active design exploration and naming tensions helps the author commit. Must be in a clearly-labelled **Internal Notes (not for consumers)** section. Remove or move to a side note before sharing the canon with downstream consumers. |
| **Breakage modes** ("what breaks the world") | An existing system is being formalised and known failure modes are worth surfacing. The resolved form of these ships in `actionable-guidelines` as hard prohibitions; do not duplicate. |
| **Designer risks / guardrails** | Handoff is to a less-experienced practitioner or to an agent, and friction points are worth naming. |

## Writing style

- Short, declarative, *matched to the project's voice*. Identity prose for a scientific-instrument canon is terse and precise; identity prose for a poetry-app canon is allowed metaphor.
- No hex codes, no font sizes, no specific pixel dimensions. Those belong to `token-vocabulary` and `actionable-guidelines`.
- Avoid generic adjectives ("clean", "modern", "professional"). Replace with anchored archetypes or material metaphors.
- Each pillar is scannable on its own. Use headings, not walls of prose.

## Template

Use this skeleton, filling in each pillar's sub-sections. Headings are required; sub-sections that are part of the required-sections list are mandatory.

```markdown
# <Canon name> — Identity

## A. Character

### A1. Personality archetype
<1–3 concrete sentences>

### A2. Negations
- Not <…>
- Not <…>
- Not <…>

### A3. Adjacent moods
- <brief phrase>
- <brief phrase>
- <brief phrase>

## B. Sensory anchor

### B1. Material / sensory grounding
<the tangible metaphor — one short paragraph>

### B2. Tone & atmosphere
<one paragraph: the vibe the visual system transmits>

## C. Visual invariants that carry voice

### C1. Geometry & compositional constraints
<high-level rules; not prescriptive numerics>

### C2. Motion philosophy
<one-shot / continuous / absent; tier structure if applicable>

### C3. Cross-surface scalability
<what survives from large-format down to icon size>

### C4. Simplified / cropped fallback
<the minimum recognisable form>

## D. Verbal voice

### D1. Voice & tone of language
<sentence rhythm, register>

<!-- Optional sections below — include only if the inclusion rule fires. -->

## (Optional) Phonetic / semantic resonance of the name
<…>

## (Optional) Tagline candidates
- <…>
- <…>

## (Optional) Token seeds
- `--<name>` — <intended use>

## Internal Notes (not for consumers)
<!-- Includes tensions/resolutions, breakage modes, designer risks if their inclusion rules fire. Remove before sharing the canon externally. -->
```

## Worked mini-example — "Plumb-line", an academic publishing co-op

```markdown
# Plumb-line — Identity

## A. Character

### A1. Personality archetype
A careful proofreader at a small university press. Reads twice before marking. Knows the difference between en dash and em dash and cares about it.

### A2. Negations
- Not editorial-glossy.
- Not vintage-letterpress nostalgia.
- Not startup-clean.

### A3. Adjacent moods
- The hush of a reading room at 4pm.
- A pencil margin note in a journal article.
- The thin red line that marks the edge of a printer's bleed.

## B. Sensory anchor

### B1. Material / sensory grounding
Ruled paper, a sharp pencil, and the faint impression of a previous page bleeding through.

### B2. Tone & atmosphere
Precise but unfussy. The system should feel like reading a well-set page — present but never the subject of attention.

## C. Visual invariants that carry voice

### C1. Geometry & compositional constraints
Single columns of justified text are the default. Marginalia sit at the left edge; figures sit centred with a baseline rule above. No bleed-edge layouts.

### C2. Motion philosophy
Absent. Pages reveal by paint, not animation.

### C3. Cross-surface scalability
At display sizes (poster, banner), the system reduces to a single ruled line and the wordmark. At icon size, only the line survives.

### C4. Simplified / cropped fallback
The plumb-line itself — a single thin vertical rule — is the system's minimum.

## D. Verbal voice

### D1. Voice & tone of language
Sentences are short and citational. Avoid superlatives. Footnotes are welcome.
```

## Self-audit checklist

Before saving, verify:

- [ ] All four pillars (A–D) are present with all required sub-sections.
- [ ] No hex codes, font sizes, or specific pixel dimensions appear (those belong elsewhere).
- [ ] Personality archetype is concrete and anchored, not a list of generic adjectives.
- [ ] Material/sensory grounding (B1) is a tangible metaphor, not an abstract mood.
- [ ] Geometry/motion sections sit at the *voice* altitude, not at the *prescriptive numerics* altitude.
- [ ] Every optional section included has its inclusion-rule trigger satisfied.
- [ ] If "Tensions & resolutions" or "Breakage modes" appear, they are inside an `Internal Notes (not for consumers)` section.
- [ ] Voice of the prose matches the project's spirit (terse for scientific, allowed-metaphor for poetic).
