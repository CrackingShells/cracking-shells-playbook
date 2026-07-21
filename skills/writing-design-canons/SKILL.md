---
name: writing-design-canons
description: >
  Produces design canons — layered reference documents a team uses to make
  assets in a coherent style and stakeholders use to review them; does not
  itself generate creative designs. Triggers when starting a visual identity,
  defining design tokens, writing production guidelines for
  figures/slides/illustrations/web assets, or extracting an existing visual
  system's operating contract. Covers four doc types: identity (personality,
  material grounding, motion-as-voice), token-vocabulary (shared atoms),
  actionable-guidelines (prescriptive rules grounded in Bertin's graphic
  semiology, closed sets, hard prohibitions), and consumption-guide (how
  consumers apply the canon to make assets). Use when the user says "design a
  brand", "define a colour system", "write a style guide", "set up figure
  guidelines", "establish a visual vocabulary", "make a graphic chart", "write a
  design canon", "standardise our visual system" — including when they don't
  name the doc type.
---

# Writing Design Canons

## Purpose

This skill produces **design canons** — the standardised reference documents a team uses to make assets in a coherent style and that stakeholders use to review them. **It does not generate creative designs.** It codifies the design decisions of a project into stable, citable artifacts so the system survives turnover, agent context resets, and time.

The controlling heuristic for every authoring decision: *does this make the produced canon more reliably consumable by a designer or stakeholder who wasn't in the room?*

## Terminology

These terms are used consistently throughout this skill and in the artifacts it produces. Use them; do not introduce synonyms.

| Term | Meaning |
|:---|:---|
| **canon** | The whole artifact set produced — the authoritative reference body for a visual system. |
| **layer doc** (or just *doc*) | A single file within a canon (e.g., `identity.md`, `tokens.md`). |
| **the four layers** | `identity`, `token-vocabulary`, `actionable-guidelines`, `consumption-guide`. |
| **consumer** | A team designer or agent who reads the canon to produce individual assets under it. |
| **author** | The user invoking this skill to write the canon. |

## Navigation

Read this table first. Identify where the user is in their design process, then open the matching reference.

| Situation | Layer doc | Reference |
|:---|:---|:---|
| No clear visual image yet — the user needs to commit to tone, atmosphere, personality, material grounding, and high-level visual invariants first | `identity` | [references/identity.md](references/identity.md) |
| Identity is settled (or pre-existing) — the user needs to define the shared atoms (colours, type, spacing, motion) downstream docs will consume | `token-vocabulary` | [references/token-vocabulary.md](references/token-vocabulary.md) |
| Atoms exist — the user needs to author the prescriptive production rules: closed sets, channel-to-data-type discipline, hard prohibitions | `actionable-guidelines` | [references/actionable-guidelines.md](references/actionable-guidelines.md) |
| Canon is complete — the user needs the meta-doc that tells consumers how to use the canon's artifacts to produce individual assets | `consumption-guide` | [references/consumption-guide.md](references/consumption-guide.md) |

> **Gate.** Before opening a reference, confirm with the user *which layer* this round produces. Users may invoke ambiguously ("help me with my brand"); ask one clarifying question if scope is unclear. The four layers are optional and not sequential — a user with a clear visual sense may enter at `token-vocabulary` and skip `identity`.

### Supporting references

The `actionable-guidelines` reference invokes two supporting references via imperative gates. Do not open them on their own — `actionable-guidelines.md` will direct you to them with explicit triggers naming the sections to read.

| File | Role |
|:---|:---|
| [references/bertin-primer.md](references/bertin-primer.md) | Visual variables, levels of organisation, channel-effectiveness rankings, data-type→channel matching matrix, perceptual maxima, named anti-patterns. The empirical and theoretical backbone of `actionable-guidelines`. |
| [references/chapter-splitting.md](references/chapter-splitting.md) | The two-condition test for *when* to split a guidelines doc into multiple chapters, plus the structural-progression tier table. |

## Core Contract

### Layered, not pipelined

The four layers serve different consumers and live at different altitudes:

- `identity` answers *why this system exists and what it feels like* — read by stakeholders and new team members.
- `token-vocabulary` answers *what atoms are sanctioned* — the single source of truth.
- `actionable-guidelines` answers *what rules govern asset production* — read by makers.
- `consumption-guide` answers *how to use the canon when producing an asset* — read by consumers on day one.

A user can enter at any layer. Do not force a top-down pipeline.

### Closed sets for prescriptive layers

`token-vocabulary` and `actionable-guidelines` produce homogeneous assets from rules. This requires *closed sets*: every dimension that varies enumerates its allowed values and explicitly states "no others". Open guidance defeats the standardisation purpose — it returns the consumer to creative judgement, which is what the canon exists to suppress.

`identity` and `consumption-guide` may use open guidance where judgement is appropriate (tone, atmosphere, escalation cases).

### Atoms as single source of truth

`token-vocabulary` is canonical. `actionable-guidelines` and `consumption-guide` reference tokens; they never redefine them. If a guideline chapter needs a new token, add it to `token-vocabulary` first and re-author the chapter — do not inline.

### Topic-dependent voice

Naming and tone match the project's spirit, not a default aesthetic. The reference files describe spectra (poetic / engineering / purpose-anchored / many in-between) and ask the author to commit to one spirit *consistently*. The skill enforces commitment, not a particular flavour.

### Degrees of freedom per layer

| Layer | Freedom | Why |
|:---|:---|:---|
| `identity` | High | Judgement-driven; the author has wide latitude on metaphor and register within each pillar. |
| `token-vocabulary` | Medium-low | Closed sets are mandatory; naming spirit is the author's commitment. |
| `actionable-guidelines` | Low | Strict prescriptive layer; closed-set discipline and Bertin channel discipline are non-negotiable. |
| `consumption-guide` | Medium | Checklist-driven but project-specific. |

## Authoring workflow

When invoked, copy this checklist into your response and track progress:

```
Canon Progress:
- [ ] 1. Confirm scope: which of the four layers does this round produce?
- [ ] 2. Open the matching reference file(s) from the Navigation table.
- [ ] 3. Draft the layer doc following the reference's required-sections list.
- [ ] 4. Self-audit: every required section present? Optional sections gated by their inclusion rules? Closed sets enumerated where required? Imperative gates in references obeyed (read each gated sub-section before authoring the step it gates)?
- [ ] 5. Save under __canons__/<canon-name>/NN-<layer>_v#.md.
- [ ] 6. Update __canons__/<canon-name>/README.md (round/version index).
```

## Output location and naming

Produced canons go in `__canons__/<canon-name>/` at the repository root (auto-create if missing). File-naming pattern mirrors `writing-reports`:

`NN-<layer>_v#.md`

- `NN` — two-digit **batch / round** number. Stable across *all* docs written in a single skill invocation. The next invocation uses `max(existing NN) + 1`; an empty canon starts at `00`. Do not increment per file.
- `<layer>` — one of `identity`, `tokens`, `guidelines` (or `guidelines/<chapter>` for split chapters), `consumption`.
- `_v#` — revision counter for that named layer. Increments only when the *same named doc* is re-generated in a later batch.

**Example timeline:**

```
__canons__/<canon-name>/
  00-identity_v0.md      ┐ batch 0 (one invocation, two docs written together)
  00-tokens_v0.md        ┘
  01-guidelines_v0.md      batch 1 (next invocation)
  02-identity_v1.md        batch 2 (revision of identity; original 00-identity_v0.md stays)
```

Maintain a `README.md` in `__canons__/<canon-name>/` listing all docs grouped by `NN` in chronological order, marking the latest version of each layer.

## Writing-style contract

These rules apply both to *this skill's own prose* and to the documents the skill produces.

1. **Imperative for instructions**; explain the *why* in one short sentence per rule. Today's models follow rationale better than rules-without-reasons.
2. **No heavy-handed MUSTs** — reframe as rationale. If a rule is non-negotiable, say *why* it is non-negotiable rather than shouting it.
3. **Examples in `Input → Output` form** where templates apply.
4. **Default-with-escape-hatch** when offering choices. Not: "you could use A or B or C". Yes: "Use A; for the rare case of X, use B."
5. **Consistent terminology** — the table above is binding. Do not introduce synonyms for *canon*, *layer doc*, *consumer*, *author*.
6. **Imperative-pointer discipline.** Every cross-reference — whether to another file, another section in the same file, or a table the author must consult — is phrased as an imperative *gate* with a trigger. **Not**: "the chapter template draws from the Bertin primer". **Yes**: "**Before completing step 3, read the Bertin primer §A.5 and quote the matching row.**" Passive pointers (*see X*, *X is also available*) are forbidden because agents skip them. Every gate names:
   - (a) when to read (the trigger),
   - (b) what to read (the exact section),
   - (c) what to bring back (the artefact or row to quote).

## When in doubt

Return to the controlling heuristic: *does this make the produced canon more reliably consumable by a designer or stakeholder who wasn't in the room?* If the answer is no, the choice fails the contract regardless of how good the prose is.
