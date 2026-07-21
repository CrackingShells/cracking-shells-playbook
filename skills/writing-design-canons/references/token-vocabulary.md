# Reference — `token-vocabulary` layer

## Contents

- Purpose
- When to use
- Degrees of freedom
- Required sections
- Naming-spirit declaration (with examples)
- Writing style
- Template
- Worked mini-example
- Self-audit checklist

## Purpose

The `token-vocabulary` layer defines the **atoms** of the visual system: the closed sets of colours, type, spacing, and motion timings that every downstream artifact consumes. This file is **the single source of truth**. `actionable-guidelines` and `consumption-guide` reference tokens; they never redefine them.

If a guideline chapter needs a token that does not exist here, add it here first and re-author the chapter.

## When to use

Open this reference when:

- Identity is settled (or pre-existing) and the author needs to commit to atoms.
- The author has a clear visual sense and wants to skip `identity` entirely.
- An existing system's atoms need to be extracted and named into a citable form.

## Degrees of freedom

**Medium-low.** Closed sets are mandatory — every dimension enumerates its allowed values and says "no others". Naming spirit is the author's commitment (see the declaration section below); once declared, it must be applied consistently across all tokens in the file.

## Required sections

The produced `tokens.md` contains the following sections in order. Each is a closed set; the file must make clear that no additions are sanctioned without revising this doc.

1. **Naming-spirit declaration** — the first section. States which spirit the names follow and gives the rule the author commits to. (See the next section of this reference for guidance.)
2. **Colour families** — for each family: name, role, solid + tint variants (or whatever variant scheme the system uses), hex / rgba / OKLCH values, accessibility note (contrast ratio against canonical backgrounds).
3. **Typography stack** — primary, mono, optional display. Closed size scale.
4. **Spacing / sizing scale** — closed numeric set.
5. **Motion timings** (if motion is part of the system per the identity layer's C2) — closed set of named durations + easings.

## Naming-spirit declaration

The first section of the produced tokens doc states the spirit. Token names must convey what they are *used for* in this project, not what they *look like* in the abstract.

The reference offers illustrative examples on a spectrum. **These are not an exhaustive set**; many in-between styles exist in real design contexts. The author commits to *one* spirit and applies it consistently — but the spirit itself is the author's choice, matched to the project.

| Spirit | Example tokens | Best when … |
|:---|:---|:---|
| Poetic / metaphor-anchored | `--wet`, `--gleam`, `--dry-grain` | The project is itself poetic, narrative-driven, or where identity is anchored in tangible metaphor (sift/glean style). |
| Engineering / systematic | `--blue-600`, `--stroke-1.25`, `--space-8` | The project is scientific, technical, or where consumers expect deterministic, predictable naming (riken style). |
| Purpose / state-anchored | `--severity-critical`, `--draft-state`, `--call-to-action` | The project is domain-driven (medical, financial, regulatory) where token meaning maps directly to functional state. |
| In-between hybrid | `--ink`, `--paper`, `--accent-warm` | The project sits between spirits — use a coherent in-between vocabulary rather than mixing two styles. |

> **Default-with-escape-hatch.** Pick the spirit closest to the project's centre of gravity from the identity layer (B1 material grounding, A1 personality). For projects that genuinely sit between two spirits, use a coherent hybrid — do not mix two pure spirits in the same file.

The declaration in the produced doc looks like:

```markdown
## Naming spirit

This canon uses **<spirit name>** token naming. Rationale: <one sentence tying back to the identity layer or the project's nature>. All tokens below follow this spirit; deviations are not sanctioned.
```

## Writing style

- **Terse and table-heavy.** Every value enumerated. The reader should scan, not read.
- **No prose about *why this colour*** — that belongs in `identity`. This file states *what* the atoms are.
- **Every closed set is explicit.** End each section with a one-line statement: "No additional <colours / sizes / durations> are sanctioned. To add one, revise this document."
- **Accessibility notes are mandatory** for colour families: state the contrast ratio against the canonical background colours.
- **Renderer caveats are explicit.** If a scale (spacing, type size, motion duration) is intended to be log-spaced or modular but the rendering substrate accepts only fixed values, ship the fixed numeric set and note the source progression.

## Template

```markdown
# <Canon name> — Token vocabulary

## Naming spirit
This canon uses **<spirit>** token naming. Rationale: <one sentence>. All tokens follow this spirit; deviations are not sanctioned.

## Colour families

> Closed set. Each family is named, scoped to a role, and ships in fixed variants. No additional families or variants are sanctioned without revising this doc.

| Token | Role | Solid | Tint | OKLCH | Contrast vs `--paper` |
|:---|:---|:---|:---|:---|:---|
| `--<token-name>` | <one-line role> | `#…` | `#…` | `oklch(…)` | <ratio> |
| … | … | … | … | … | … |

## Typography stack

> Closed scale. Sizes follow a <ratio> modular scale; no other sizes are sanctioned.

| Token | Family | Size (px) | Line-height | Use |
|:---|:---|:---|:---|:---|
| `--type-display` | <font> | … | … | … |
| `--type-body` | <font> | … | … | … |
| `--type-mono` | <font> | … | … | … |

## Spacing / sizing scale

> Closed set. Values follow <a log progression / modular scale / fixed numeric set because the renderer requires it>.

| Token | Value (px) | Use |
|:---|:---|:---|
| `--space-1` | 4 | … |
| `--space-2` | 8 | … |
| … | … | … |

## Motion timings

> Include only if the identity layer's motion philosophy (C2) is non-absent.

| Token | Duration | Easing | Use |
|:---|:---|:---|:---|
| `--motion-reveal` | 700ms | cubic-bezier(…) | … |
| … | … | … | … |
```

## Worked mini-example — Plumb-line tokens (engineering spirit)

```markdown
# Plumb-line — Token vocabulary

## Naming spirit
This canon uses **engineering / systematic** token naming. Rationale: Plumb-line consumers are typesetters and editors who expect deterministic, predictable atom names. All tokens follow this spirit; deviations are not sanctioned.

## Colour families

> Closed set. Three families, each with a single solid value and a tinted variant for backgrounds. No additional families are sanctioned without revising this doc.

| Token | Role | Solid | Tint | OKLCH | Contrast vs `--paper` |
|:---|:---|:---|:---|:---|:---|
| `--ink` | Body text, rules | `#1a1a1a` | `#4a4a4a` | `oklch(0.18 0 0)` | 13.5:1 |
| `--paper` | Background | `#fafaf7` | n/a | `oklch(0.98 0.005 90)` | — |
| `--mark-red` | Marginalia, corrections | `#a02020` | `#e8c8c8` | `oklch(0.45 0.18 25)` | 6.2:1 |

## Typography stack

> Closed scale. Sizes follow a 1.25 modular scale; no other sizes are sanctioned.

| Token | Family | Size (px) | Line-height | Use |
|:---|:---|:---|:---|:---|
| `--type-display` | "Source Serif Pro" | 32 | 1.2 | Article titles |
| `--type-body` | "Source Serif Pro" | 16 | 1.5 | Running text |
| `--type-marginal` | "Source Serif Pro" | 13 | 1.4 | Marginalia |
| `--type-mono` | "JetBrains Mono" | 14 | 1.5 | Code, citations |

## Spacing / sizing scale

> Closed set. Values follow a 1.5× progression (modular scale).

| Token | Value (px) | Use |
|:---|:---|:---|
| `--space-1` | 4 | Hairline gaps |
| `--space-2` | 6 | Inline spacing |
| `--space-3` | 9 | Small margins |
| `--space-4` | 14 | Default gap |
| `--space-5` | 21 | Section gap |
| `--space-6` | 32 | Page margin |

## Motion timings

Not applicable — Plumb-line's motion philosophy is absent (per identity §C2).
```

## Self-audit checklist

- [ ] Naming-spirit declaration is the first section and states the spirit chosen with a one-sentence rationale.
- [ ] Every required section (colour families, typography stack, spacing/sizing scale) is present.
- [ ] Motion timings included if and only if the identity layer's C2 is non-absent.
- [ ] Each section ends with an explicit "closed set" statement.
- [ ] Colour families include contrast ratios against canonical backgrounds.
- [ ] Spacing / sizing scale notes whether values follow a modular / log progression or are fixed (with reason).
- [ ] No prose explaining *why a colour was chosen* — that lives in identity.
- [ ] Token names follow the declared spirit consistently. (Spot-check: no `--blue-600` in a poetic-spirit file; no `--wet` in an engineering-spirit file.)
