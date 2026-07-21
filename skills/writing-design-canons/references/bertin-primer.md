# Reference — Bertin primer

## Contents

- §A.1 — The visual variables
- §A.2 — Levels of organisation
- §A.3 — Channel effectiveness ranking for quantitative data
- §A.4 — Channel categories for non-quantitative data
- §A.5 — Data-type → channel matching matrix
- §A.6 — Perceptual maxima per channel
- §A.7 — Named anti-patterns with rationale
- §A.8 — Pointer for further reading

## Purpose

This primer is the empirical backbone of the `actionable-guidelines` reference. Authors do not invent channel-assignment rules — they pick them from this file. The gates inside `actionable-guidelines.md` will direct you to specific sub-sections (§A.5, §A.6, §A.7) at specific moments. When a gate fires, read the gated sub-section in full and bring back what the gate asks for.

The primer condenses Bertin's *Sémiologie graphique* (1967) and its modern extensions (Cleveland-McGill 1984, Mackinlay 1986, Munzner *Visualization Analysis and Design*).

---

## §A.1 — The visual variables

The atoms by which any visual system encodes information. Each variable supports specific levels of organisation (see §A.2); use this list as a vocabulary of channels available for assignment.

| Variable | Definition | Levels supported (see §A.2) |
|:---|:---|:---|
| **Position (planar)** | Where a mark sits in the 2D plane | associative, selective, ordered, quantitative |
| **Size (area)** | The 2D extent of a mark | selective, ordered, quantitative (with caveats) |
| **Length** | A 1D extent of a mark | selective, ordered, quantitative |
| **Value** (lightness) | How dark or light a mark is | associative, selective, ordered |
| **Saturation** | How chromatic vs grey a mark is | selective, ordered |
| **Hue** | The colour family of a mark | associative, selective (categorical only) |
| **Orientation** | The angle a mark is rotated to | associative, selective |
| **Shape** | The form of a mark | associative, selective (categorical only) |
| **Texture** | Pattern fill of a mark | associative, selective (categorical only) |
| **Transparency** (modern) | The opacity of a mark | ordered (with care) |
| **Crispness** (modern) | Edge sharpness vs blur | ordered |
| **Motion / duration** (modern) | How and how long a mark animates | selective, ordered |
| **Rate of change** (modern) | Tempo of animation | ordered |

## §A.2 — Levels of organisation

Bertin's four perceptual guarantees. A channel supports a level only if viewers can *reliably* perform the perceptual task using that channel alone.

| Level | Perceptual guarantee |
|:---|:---|
| **Associative** | Viewers can group related marks together (perceive same-ness). |
| **Selective** | Viewers can isolate a single category from all others (find the red ones). |
| **Ordered** | Viewers perceive magnitude progression (rank low to high). |
| **Quantitative** | Viewers can compare exact ratios (this is twice that). |

When assigning a channel, the chapter must commit to which level the channel must support, and verify the channel is capable of it (see §A.5).

## §A.3 — Channel effectiveness ranking for quantitative data

Cleveland-McGill (1984), empirically validated. Rank order, most accurate first:

1. **Position on a common scale** (e.g., scatter plot) — most accurate.
2. **Position on non-aligned scales** (small multiples).
3. **Length** (bar charts).
4. **Angle / slope** (pie slices, scatter slope).
5. **Area** (bubbles, treemaps) — perceived sublinearly per Stevens' power law.
6. **Volume, saturation, density** — least reliable.

> Use the highest-ranked channel that fits the chapter's geometry constraints. Falling to area or below requires explicit rationale.

## §A.4 — Channel categories for non-quantitative data

Mackinlay's framework distinguishes two channel categories:

- **Identity channels** — convey category membership without implying order. *Shape, hue, texture.* Use for **categorical** data.
- **Magnitude channels** — perceptually convey quantity or progression. *Position, length, size, saturation, value.* Use for **ordinal** data (preserves the ranking).

A common error is to use an identity channel (hue) for ordinal data — viewers see categories, not order.

## §A.5 — Data-type → channel matching matrix

The matrix the `actionable-guidelines` chapter template's step-3 gate refers to. For each property the chapter declares, locate the row matching its data type, then pick a channel from "Recommended" or "Acceptable". Channels under "Forbidden" must not be used for that data type.

| Data type | Recommended | Acceptable | Forbidden |
|:---|:---|:---|:---|
| **Quantitative** (ratio comparisons matter) | Position (common scale), length | Position (non-aligned), angle, area (with rationale), saturation, value | Hue, shape, texture, orientation |
| **Ordinal** (ranking matters; ratios do not) | Position, length, value, saturation | Size, transparency, crispness | Hue (non-sequential), shape, texture, arbitrary orientation |
| **Categorical / nominal** (only same/different) | Hue (≤ ~10 categories), shape (≤ ~6), position | Orientation, texture | Value (implies magnitude), saturation (implies magnitude), area (implies magnitude) |

> **Rule of thumb cited by the gate:** if the matrix marks your channel "forbidden" for the chapter's data type, pick another channel — do not author the property.

## §A.6 — Perceptual maxima per channel

Closed-set bounds. When a chapter declares a closed set on a channel, the set size must not exceed these maxima.

| Channel | Maximum distinguishable values | Note |
|:---|:---|:---|
| Hue | ~10 | JNDs smallest in green-yellow, largest in blue-purple |
| Shape | ~6 | Beyond this, viewers conflate shapes under load |
| Saturation | ~6 | Beyond this, steps collapse into "saturated" / "desaturated" |
| Value (lightness) | 8–11 | A gray-scale of more than 11 steps is not reliably resolved |
| Size (categorical use) | ~4 | Use only when paired with another channel |
| Texture | ~4 | Distinguishability degrades fast at small sizes |
| Orientation | ~4 | Beyond cardinal + diagonal, distinctions blur |

**Two perceptual laws to keep in mind:**

- **Stevens' power law for area perception.** Perceived area ∝ actual_area^β where β ≈ 0.6–0.9. A region 4× larger is perceived as ~2.5× larger. **Consequence:** never use area for ratio comparison; use length.
- **OKLab / OKLCH for colour scales.** Perceptually-uniform colour spaces (OKLab, OKLCH) produce sequential scales where equal numeric steps look like equal perceptual steps. RGB and HSL do not. **Consequence:** sequential and diverging scales should be designed in OKLCH, not RGB or HSL.

## §A.7 — Named anti-patterns with rationale

The chapter template's step-4 and step-5 gates direct you to copy these in verbatim when their channels are declared. Each is a one-liner with rationale; do not paraphrase.

- **rainbow-for-quantitative** — Hue has no natural order; rainbow scales create false boundaries and uneven brightness across the range. Use a perceptually-uniform sequential scale (OKLCH).
- **hue-for-ranking** — Hue is associative, not ordered. Viewers cannot rank by hue. Use value, saturation, length, or position for ranking.
- **area-for-ratio** — Stevens' power law: area is perceived ≈ x^0.7. Comparing areas understates ratios. Use length on a common scale instead.
- **texture-for-ordered** — Texture is associative only. Viewers can group by texture but not rank. Use for categorical only.
- **unstated double-encoding** — Encoding the same attribute on two channels without stating why creates ambiguity about which is the primary signal. If double-encoding is intentional (accessibility, robustness), state it.

## §A.8 — Pointer for further reading

- Jacques Bertin, *Sémiologie graphique* (1967; English translation: *Semiology of Graphics*, 1983).
- William Cleveland & Robert McGill, "Graphical Perception: Theory, Experimentation, and Application to the Development of Graphical Methods", *JASA* 79 (1984).
- Jock Mackinlay, "Automating the Design of Graphical Presentations of Relational Information", *ACM TOG* 5 (1986).
- Tamara Munzner, *Visualization Analysis and Design* (CRC Press, 2014) — esp. chapters 5 ("Marks and Channels") and 10 ("Map Color").

No fetch required at authoring time; the matrix in §A.5, the maxima in §A.6, and the anti-patterns in §A.7 are self-contained.
