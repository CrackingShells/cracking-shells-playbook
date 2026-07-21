# writing-design-canons

A Claude Code skill that generates **design canons** — the layered reference documents a team uses to make assets in a coherent style and that stakeholders use to review them.

This skill standardises the artifacts through which creativity is expressed. It does **not** itself generate creative designs.

## What it produces

A canon is the artifact set produced by this skill. Each canon is composed of up to four **layer docs**:

| Layer | What it codifies |
|:---|:---|
| `identity` | Personality, material grounding, motion-as-voice, high-level visual invariants |
| `token-vocabulary` | Atoms: colour families, typography, spacing, motion timings (single source of truth) |
| `actionable-guidelines` | Prescriptive production rules grounded in Bertin's graphic semiology — closed sets, channel-to-data-type discipline, hard prohibitions |
| `consumption-guide` | Operational doc telling consumers how to use the canon to produce individual assets |

The four layers are optional and not sequential. A user with a clear visual sense may enter at `token-vocabulary` and skip `identity`.

## Installation

The skill is authored here and activated by symlinking into `~/.claude/skills/`:

```bash
ln -s "$(pwd)/writing-design-canons" ~/.claude/skills/writing-design-canons
```

After symlinking, invoke as `/writing-design-canons` in Claude Code, or describe the task in natural language ("help me design a colour system for …", "write the production guidelines for our figures", etc.) and the skill will trigger.

## Anatomy

```
writing-design-canons/
├── SKILL.md                              # Entry point: purpose, navigation, contract, workflow
├── README.md                             # This file
└── references/
    ├── identity.md                       # The identity layer reference
    ├── token-vocabulary.md               # The tokens layer reference
    ├── actionable-guidelines.md          # The guidelines layer reference (slim; gates the two below)
    ├── bertin-primer.md                  # Visual variables, channel matrices, anti-patterns
    ├── chapter-splitting.md              # Two-condition test + tier table for splitting guidelines into chapters
    └── consumption-guide.md              # The consumption-guide layer reference
```

## Output layout

Produced canons are saved under `__canons__/<canon-name>/` (auto-created) with files named:

```
NN-<layer>_v#.md
```

- `NN` = batch / round number (stable across all docs in one invocation; increments per invocation).
- `<layer>` = `identity` | `tokens` | `guidelines` | `guidelines/<chapter>` | `consumption`.
- `_v#` = revision counter for the same named doc.

See `SKILL.md` ("Output location and naming") for the full convention.

## Influences

The skill condenses patterns from three families of source material:

- **Brand-identity style** — `ro/sift-visual-identity-v0/`, `ro/glean-visual-identity-v0/` (narrative-first, personality archetypes, material grounding, motion-as-voice).
- **Actionable-guidelines style** — `ro/riken-figure-styling/references/*.jsx` (atoms file as single source of truth, closed-set constraints, registers, hard prohibitions, perceptual grounding).
- **Theoretical backbone** — Jacques Bertin, *Sémiologie graphique* (1967); Cleveland & McGill (1984); Munzner, *Visualization Analysis and Design* (2014).

## Development

This skill is authored as a portable artifact. To revise:

1. Edit files in this directory directly. Changes take effect on the next skill invocation (the symlink is live).
2. When iterating on the contract, treat `SKILL.md` and each reference file as the unit. The "Self-audit checklist" at the bottom of every reference is the contract surface — keep it in sync with the rest of the file.
3. Verification is by dogfood: produce a small canon end-to-end on a deliberately different project (financial dashboard, culinary archive) and check that the output respects each reference's required sections, optional-section gating, and gates.
