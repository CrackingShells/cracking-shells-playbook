# Reference — `consumption-guide` layer

## Contents

- Purpose
- When to use
- Degrees of freedom
- Required sections
- Writing style
- Template
- Worked mini-example
- Self-audit checklist

## Purpose

The `consumption-guide` is a **meta-document that lives inside the produced canon**. It tells a *consumer* — a team designer or agent who has the canon in front of them — how to consume the canon's artifacts to produce an individual downstream asset.

It is **not** documentation for this skill. The skill's own documentation is in `SKILL.md`. The consumption-guide is what a fresh team designer reads on day one to learn how to apply *this specific canon* to *this specific asset*.

A canon without a consumption-guide is a stack of references no one knows how to use together. The consumption-guide is the operational bridge.

## When to use

Open this reference when:

- A canon is otherwise complete (`identity`, `tokens`, `guidelines` exist) and the author needs the operational doc that ties them together for consumers.
- An existing canon has unwritten conventions that consumers learn by osmosis — the consumption-guide makes them explicit and citable.

## Degrees of freedom

**Medium.** Checklist-driven but project-specific. The required sections are fixed; the content within each section is necessarily tied to the canon's particulars.

## Required sections

### 1. Artifact map

A guide to which layer doc to read in which order, for each downstream task. The consumer arrives with a task ("make a slide", "make a figure", "make a logo lockup"); the artifact map tells them where to start.

Use a table:

| Downstream task | Read first | Read second | Read third |
|:---|:---|:---|:---|
| <task> | <layer doc> | <layer doc> | <layer doc> |

For each task, explain in one sentence *why* this reading order — e.g., "Identity sets the register; guidelines turn the register into rules; tokens supply the values."

### 2. Commitment checklist

Before producing an asset, the consumer must make binding decisions. These are the *variable* items from the guidelines chapters' fixed-vs-free tables, gathered in one place. The checklist gives the consumer a single page to scan.

Format as a checklist the consumer copies into their working notes:

```
Asset commitments:
- [ ] Register: ___________________________
- [ ] Hue family (pick 2–3 from <set>): _____________
- [ ] Motion tier (if applicable): __________________
- [ ] Dash vocabulary (pick 1): _____________________
- [ ] <other binding decisions specific to this canon>
```

The consumer fills these in *before* drawing anything. If a binding decision cannot be made, the asset is not ready to produce.

### 3. Token reuse rules

How the consumer references tokens in their working medium. Atoms are read from `tokens.md`; they are never redefined locally. Spell out the syntax in the canon's typical media:

| Medium | Reference syntax |
|:---|:---|
| CSS | `var(--token-name)` |
| JSX / React | `A_TOKENS.<token-name>` or imported constant |
| Design tool (Figma, Sketch) | Linked style named after the token |
| Manuscript / caption | Token name in monospace, no hex value |

Add a one-line rule: *"If you find yourself inlining a value not present in `tokens.md`, stop. Add it to `tokens.md` first and reload."*

### 4. Verification checklist

After producing an asset and before shipping, the consumer self-audits. The checklist makes the audit mechanical, not judgemental.

```
Asset verification:
- [ ] Every value used appears in tokens.md (no inlined hex codes, sizes, durations).
- [ ] The asset commits to one register / mode / variant — no mid-asset mixing.
- [ ] Every closed set declared in the relevant guidelines chapter is respected.
- [ ] Every hard prohibition in the relevant guidelines chapter is honoured.
- [ ] Channel-to-data-type discipline holds: no §A.5-forbidden channels used.
- [ ] If anything in the asset cannot be cited back to identity, tokens, or guidelines — see §6 (Escalation path).
```

### 5. Common drifts and how to catch them

Failure modes specific to *this canon*. The consumer learns to watch for them. Each is a short item: *what the drift looks like*, *why it happens*, *how to catch it*.

This section is the *resolved* form of the identity layer's optional "breakage modes". The identity layer's breakage-modes section is brainstorming; this section is operational.

Examples (the actual list is canon-specific):

- *Adding a seventh hue.* Happens when the consumer needs a category not in the closed set. **Catch:** the verification checklist's "closed sets" line will fail. **Resolve:** either re-use an existing hue with a different role, or follow §6 to add a hue to `tokens.md`.
- *Mid-figure register mix.* Happens when the consumer changes the asset's mode partway through. **Catch:** the verification checklist's "one register" line will fail. **Resolve:** pick the dominant register and recast the off-register elements.
- *Using hue for ranking within a kind.* Happens when the consumer wants to highlight a primary item among siblings. **Catch:** the channel-to-data-type discipline line will fail. **Resolve:** use value or weight, not hue.

### 6. Escalation path

When the asset cannot be produced within the canon's rules, the consumer does not silently bend the rule. Spell out the escalation:

- **Who decides** whether to bend a rule (e.g., the canon owner; the design lead).
- **What gets recorded** when a rule is bent (a note in the asset's metadata; an issue against the canon).
- **When the canon itself gets revised** vs when the bend is a one-off (e.g., "if the same bend happens three times, the canon needs a new chapter or token").

The escalation path is the canon's pressure-release valve. Without it, consumers either silently violate the rules or refuse to ship.

## Writing style

- **Operational, checklist-heavy.** The consumer should be able to scan and act, not read paragraphs.
- **Written for a cold-start consumer.** Assume the consumer has the canon in front of them but no prior context.
- **No prose about the canon's *why*** — that lives in identity. The consumption-guide is about *how*.
- **Cite layer docs by their canon-internal names** (e.g., "see `01-identity_v0.md`", or "see Chapter 03 of the guidelines"). The consumption-guide assumes the consumer is reading the canon as a whole, not visiting through this skill.

## Template

```markdown
# <Canon name> — Consumption guide

## 1. Artifact map
| Downstream task | Read first | Read second | Read third |
|:---|:---|:---|:---|
| <…> | <…> | <…> | <…> |

## 2. Commitment checklist
Copy this into your working notes before producing an asset:

```
Asset commitments:
- [ ] <binding decision>: __________
- [ ] <…>: __________
```

## 3. Token reuse rules
| Medium | Reference syntax |
|:---|:---|
| <…> | <…> |

Rule: If you find yourself inlining a value not present in `tokens.md`, stop. Add it to `tokens.md` first and reload.

## 4. Verification checklist
```
Asset verification:
- [ ] <verification item>
- [ ] <…>
```

## 5. Common drifts
- **<drift name>.** What it looks like / why it happens / how to catch and resolve.
- **<…>.** <…>

## 6. Escalation path
- **Who decides:** <…>
- **What gets recorded:** <…>
- **When the canon revises:** <…>
```

## Worked mini-example — Plumb-line consumption guide

```markdown
# Plumb-line — Consumption guide

## 1. Artifact map
| Downstream task | Read first | Read second | Read third |
|:---|:---|:---|:---|
| Set a journal article page | `00-identity_v0.md` (geometry C1) | Guidelines Ch. 01 (Marginalia) | `00-tokens_v0.md` |
| Design a cover | `00-identity_v0.md` (sensory anchor B1) | `00-tokens_v0.md` | — |
| Add a corrigendum margin note | Guidelines Ch. 01 | `00-tokens_v0.md` | — |

## 2. Commitment checklist
```
Asset commitments:
- [ ] Marginalia colour: ___ (--ink default; --mark-red for correction)
- [ ] Page layout: ___ (single column or facing pages)
- [ ] Display vs body typeface roles confirmed
```

## 3. Token reuse rules
| Medium | Reference syntax |
|:---|:---|
| CSS (web reading view) | `var(--token-name)` |
| InDesign | Paragraph / character style named after the token |
| Manuscript notes | `--token-name` in monospace |

Rule: if you find yourself inlining a value not present in `00-tokens_v0.md`, stop. Add it to `tokens.md` first and reload.

## 4. Verification checklist
```
Asset verification:
- [ ] No inlined hex codes or sizes (everything via tokens).
- [ ] Marginalia respect the closed set {--ink, --mark-red}.
- [ ] Marginalia sit at the left edge — never inline (per Ch. 01).
- [ ] No motion or animated elements (identity §C2 = absent).
- [ ] Type tokens used at their sanctioned roles only.
```

## 5. Common drifts
- **Inline marginalia.** Happens when the editor pastes a parenthetical aside into the text. **Catch:** verification line 3. **Resolve:** pull the aside out into the margin column.
- **Third marginalia colour.** Happens when an editor wants to mark "query" separately from "correction". **Catch:** verification line 2. **Resolve:** use `--mark-red` for both, distinguished by symbol (?) prefix; or follow §6 to add a third token.

## 6. Escalation path
- **Who decides:** the typesetting lead.
- **What gets recorded:** a one-line note in the article's metadata file under `canon-deviations`.
- **When the canon revises:** if the same deviation appears in three issues, the canon adds a chapter or token to absorb it.
```

## Self-audit checklist

- [ ] All six required sections present (artifact map, commitments, token reuse, verification, common drifts, escalation).
- [ ] The artifact map covers every downstream task type the canon serves.
- [ ] The commitment checklist matches every "variable" item in the guidelines chapters' fixed-vs-free tables.
- [ ] Token reuse rules cover every medium the canon ships into.
- [ ] The verification checklist is mechanical — a consumer can run it without judgement calls.
- [ ] Common drifts are canon-specific, not generic design advice.
- [ ] Escalation path names *who decides*, *what gets recorded*, and *when the canon revises*.
- [ ] No prose explaining the canon's *why* — that lives in identity.
