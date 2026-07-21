# Reference — `actionable-guidelines` layer

## Contents

- Purpose
- When to use
- Degrees of freedom
- Supporting references (gated)
- Pre-authoring gate
- Chapter template
- Writing style
- Template
- Worked mini-example
- Self-audit checklist

## Purpose

The `actionable-guidelines` layer turns the atoms in `token-vocabulary` into **enforceable production rules**. It is the prescriptive layer of the canon: closed sets, hard prohibitions, channel-to-data-type discipline, fixed-vs-free splits. It is read by *makers* — the team designers and agents who produce individual assets.

The author produces *N chapters*, one per orthogonal concern (e.g., colour application, container geometry, edge rules, modulation, layout). The number of chapters and their scope is determined by the chapter-splitting test, not by intuition.

## When to use

Open this reference when:

- Atoms exist in `token-vocabulary` and the author needs prescriptive rules for asset production.
- An existing system's rules need to be made explicit and citable.
- A canon's chapters need restructuring (in which case re-run the splitting test first).

## Degrees of freedom

**Low.** This is the prescriptive layer. Closed-set discipline and Bertin channel discipline are non-negotiable. The reference *explains why* each rule exists rather than commanding it in caps — but the rules are the rules.

Authors do not invent channel-assignment rules or anti-patterns. They pick them from the Bertin primer (see Supporting references).

## Supporting references

This layer depends on two supporting reference files. The gates below trigger them at specific moments; do not pre-read them.

| Supporting reference | When triggered | What it provides |
|:---|:---|:---|
| [chapter-splitting.md](chapter-splitting.md) | The **pre-authoring gate** below, before any chapter is written | The two-condition test, anti-tests, tier table |
| [bertin-primer.md](bertin-primer.md) | The **per-step gates** in the chapter template (§3, §4, §5) | Visual variables, data-type→channel matrix, perceptual maxima, anti-patterns |

## Pre-authoring gate

> **Gate.** Before writing the first chapter of this canon, read [chapter-splitting.md](chapter-splitting.md) in full and apply its two-condition test (§B.2) to your planned chapter list. Return with:
>
> (a) the final list of chapter titles you will author, and
> (b) the tier each occupies in the §B.4 structural-progression table.
>
> If you cannot answer (a) and (b), do not begin authoring chapters.

## Chapter template

Each chapter contains the following sections in order. Every step has an explicit purpose; gates name the supporting reference and the artefact to bring back.

### 1. Header rule

One imperative sentence stating the chapter's binding rule. Example: *"Commit to one register per figure; never mix mid-figure."* This is the sentence a consumer should be able to quote from memory.

### 2. Fixed vs free

State explicitly what is **locked** (geometry, structure, proportions) and what is **variable** (colour family, register choice, accent). The split is the chapter's contract with the consumer — the locked items are non-negotiable; the variable items are the consumer's commitments per asset.

Use a table:

| Aspect | Status |
|:---|:---|
| <e.g., corner radius> | Locked at <value> from `tokens.md` |
| <e.g., accent colour> | Variable — consumer picks from <set> |

### 3. Channel assignment (Bertin)

For each property in the chapter that varies, declare:

- **Data type** — categorical / ordinal / quantitative.
- **Visual channel** used (from §A.1).
- **Level of organisation** the channel must support (from §A.2).
- **Closed set of values** used.
- **Perceptual rationale** in one line.

> **Gate.** Before filling in §3 for any property, read [bertin-primer.md](bertin-primer.md), specifically §A.5 (data-type → channel matching matrix) and §A.6 (perceptual maxima per channel).
>
> For each property: quote the §A.5 matrix row that justifies your channel pick, and the §A.6 line that justifies your closed-set size.
>
> **If §A.5 marks your channel "forbidden" for the chapter's data type, pick another channel — do not author the property.**

Use a table:

| Property | Data type | Channel | Level | Closed set | Rationale |
|:---|:---|:---|:---|:---|:---|
| <name> | quantitative / ordinal / categorical | hue / value / length / … | associative / selective / ordered / quantitative | `{a, b, c}` | <one-line citation of §A.5 or §A.6 or §A.3> |

### 4. Hard prohibitions

Explicit "never" statements with one-line rationale each.

> **Gate.** Before writing §4, read [bertin-primer.md](bertin-primer.md) §A.7 (named anti-patterns). Copy in verbatim every anti-pattern from §A.7 that touches the channels declared in §3 of this chapter. Then add project-specific prohibitions on top.

Format each line as: *"Never <action> — <one-line rationale>."*

### 5. Modulation axes

Include this section only if a property in the chapter varies along a scale (alpha, stroke-width, lightness, etc.). State the sanctioned axes and the stop positions (perceptual / log curve, not linear).

> **Gate.** Before authoring §5, read [bertin-primer.md](bertin-primer.md) §A.6 (perceptual maxima) and §A.7 (anti-patterns — especially rainbow-for-quantitative, area-for-ratio).
>
> **If the chapter's modulation crosses hues, do not author this section — return to §3 and re-pick a channel.** Cross-hue modulation is forbidden by §A.7.

Use a table:

| Axis | Curve | Stops | Purpose |
|:---|:---|:---|:---|
| <e.g., stroke-width> | log / linear / perceptual | `{0.5, 0.75, 1, 1.25, 1.75}` | <what it encodes> |

### 6. Side-by-side specimens

Comparison-first visuals. The strongest teaching format is *this vs that* — engineering vs message-passing, linear vs perceptual, allowed vs forbidden.

For each specimen pair, give:

- A short caption naming what is being compared.
- A description of the *correct* example.
- A description of the *forbidden* counter-example (when the chapter has hard prohibitions, illustrate at least one).

Specimens may be prose descriptions for downstream rendering, or embedded snippets where the medium supports it (SVG, JSX).

## Writing style

- **Prescriptive, imperative, evidence-cited.** Every rule has a one-line rationale.
- **Hard prohibitions are encouraged.** Negative space teaches faster than positive rules alone.
- **Strict-template pattern.** "ALWAYS use this exact template" — the chapter structure is non-negotiable. The author fills in the table cells; the author does not restructure the chapter.
- **Cite tokens, do not redefine them.** When referencing a colour, type, or spacing value, cite it as `--token-name` from `tokens.md`. Never inline a hex code in a chapter.
- **Imperative cross-references only.** Every pointer to a supporting reference is a gate (see the gates above). No passive "see also" pointers.

## Template

```markdown
# <Canon name> — Guidelines, Chapter NN: <chapter title>

## Header rule
<one imperative sentence>

## Fixed vs free
| Aspect | Status |
|:---|:---|
| <…> | Locked at <…> from `tokens.md` |
| <…> | Variable — consumer picks from <…> |

## Channel assignment (Bertin)
| Property | Data type | Channel | Level | Closed set | Rationale |
|:---|:---|:---|:---|:---|:---|
| <…> | <…> | <…> | <…> | <…> | <citation of §A.5 / §A.6 / §A.3> |

## Hard prohibitions
- Never <…> — <rationale, citing §A.7 if applicable>.
- Never <…> — <rationale>.

## Modulation axes
<include only if a property varies along a scale>
| Axis | Curve | Stops | Purpose |
|:---|:---|:---|:---|
| <…> | <…> | <…> | <…> |

## Side-by-side specimens
**Correct vs forbidden — <what is compared>:**
- Correct: <description>
- Forbidden: <description>
```

## Worked mini-example — Plumb-line, Chapter 01: Marginalia rules

```markdown
# Plumb-line — Guidelines, Chapter 01: Marginalia rules

## Header rule
Marginalia sit at the left edge of the column at fixed offset; never inline, never to the right of the main text.

## Fixed vs free
| Aspect | Status |
|:---|:---|
| Left offset from column | Locked at `--space-5` (21 px) from `tokens.md` |
| Type token | Locked at `--type-marginal` |
| Marginalia colour | Variable — consumer picks `--ink` (default) or `--mark-red` (correction) |

## Channel assignment (Bertin)
| Property | Data type | Channel | Level | Closed set | Rationale |
|:---|:---|:---|:---|:---|:---|
| Marginalia colour | categorical (2: comment, correction) | hue | selective | `{--ink, --mark-red}` | §A.5: hue is recommended for categorical ≤10. §A.6: 2 hues is well within the ~10 maximum. |

## Hard prohibitions
- Never use a third marginalia colour — the set is closed at 2 (cited from `tokens.md`).
- Never use `--mark-red` for non-correction marginalia — hue carries semantic load (correction vs comment); §A.7 unstated-double-encoding warns against ambiguating it.
- Never inline marginalia mid-paragraph — violates the header rule and breaks Z-reading.

## Side-by-side specimens
**Correct vs forbidden — marginalia placement:**
- Correct: marginalia left of the column, aligned to the line it annotates, `--type-marginal` size, `--ink` colour.
- Forbidden: marginalia inline between sentences, `--type-body` size — looks like a parenthetical, not an annotation.
```

## Self-audit checklist

- [ ] Pre-authoring gate satisfied: planned chapter list mapped onto the [chapter-splitting.md](chapter-splitting.md) tier table; the two-condition test passes for each planned chapter.
- [ ] Every chapter has a one-sentence header rule.
- [ ] Fixed-vs-free table is present and every variable item has its allowed-set named.
- [ ] Channel-assignment table has at least one row per varying property, with §A.5 or §A.6 or §A.3 citations in the rationale column.
- [ ] No property in the channel-assignment table uses a §A.5-"forbidden" channel.
- [ ] Hard-prohibitions section includes every §A.7 anti-pattern that touches the chapter's declared channels.
- [ ] If a modulation section is present, the curve is perceptual / log (not linear) and no axis crosses hues.
- [ ] Every token referenced is cited as `--token-name`, never inlined as a hex code.
- [ ] At least one side-by-side specimen pair (correct vs forbidden) for chapters with hard prohibitions.
