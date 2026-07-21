# Reference — Chapter-splitting decision rule

## Contents

- §B.1 — When this file fires
- §B.2 — The two-condition test
- §B.3 — Anti-tests (when *not* to merge)
- §B.4 — Structural-progression tier table
- §B.5 — Diagnostic use of the tier table
- §B.6 — Worked example: riken nine-file split

## Purpose

This file holds the test that the `actionable-guidelines` reference's pre-authoring gate refers to. It exists so that the author commits to a chapter list *before* writing any chapter — preventing the common drift of authoring one giant chapter that quietly mixes concerns.

## §B.1 — When this file fires

The `actionable-guidelines` reference contains a gate at the top: *"Before writing the first chapter of this canon, read §B (Chapter-splitting decision rule) and apply its two-condition test to your planned chapter list."* When that gate fires, read this file in full and return with:

- (a) the final list of chapter titles you will author, and
- (b) the tier each occupies in the §B.4 structural-progression table.

If you cannot answer (a) and (b), do not begin authoring chapters.

## §B.2 — The two-condition test

A chapter is warranted when **both** conditions hold:

1. **The concern has its own closed sets, prohibitions, and channel assignments** that do not naturally collapse into a sibling chapter. (If two concerns would share their entire closed-set declaration and prohibition list, they are not two chapters.)
2. **The chapter is self-readable as a unit.** A consumer can read it and apply it to a single asset without flipping to other chapters (other than `tokens.md` for atoms).

If a planned chapter fails either condition, merge it with its closest sibling and re-test.

## §B.3 — Anti-tests (when *not* to merge)

Even if two concerns appear thematically similar, split them into separate chapters when any of the following is true:

- The concerns use **different visual channels** (e.g., hue vs dash-pattern; size vs position).
- The concerns operate at **different levels of organisation** in the §B.4 tier table (atoms vs elements vs relations vs compositions).
- The concerns have their **own register split** (e.g., engineering vs message-passing in the riken canon — same concept, but each register produces a different chapter shape).
- One concern's prohibitions would **dilute the focus** of the sibling — if merging forces you to write "this rule applies to X but not Y", split.

## §B.4 — Structural-progression tier table

The abstract pattern behind well-organised guidelines docs. A canon's chapters should map cleanly onto these tiers; missing tiers and clustering on a single tier are both diagnostic signals.

| Tier | What it governs | Typical chapter title |
|:---|:---|:---|
| **Atoms** | Closed sets of primitives — colours, dash patterns, shapes | "Colour palette", "Dash vocabulary" |
| **System-level invariants** | Modes the whole asset commits to before drawing anything | "Registers", "Orientation modes" |
| **Element-level rules** | Individual marks / objects in isolation | "Containers", "Nodes", "Keystones" |
| **Relation-level rules** | How elements connect to each other | "Edges", "Joiners", "Adjacency" |
| **Cross-cutting modulation** | Rules for any property that varies along a scale | "Perceptual modulation", "Density gradient" |
| **Compositional rules** | Layout / arrangement of the whole asset | "Tables", "Page composition", "Grid" |

## §B.5 — Diagnostic use of the tier table

Use the tier table to test for two failure modes:

1. **Missing chapter.** If a tier is empty *and* the canon's assets need rules at that tier, you are missing a chapter. Common omissions: system-level invariants (asset has no committed mode); cross-cutting modulation (varying properties have no perceptual rationale).
2. **Over-clustering on one tier.** If five planned chapters all sit at *element-level*, you almost certainly need to consolidate — re-apply §B.2 condition 1 to see which pairs share their closed sets and prohibitions.

A balanced canon has at least one chapter per tier it needs, and no tier has more than three.

> **Not every canon needs all six tiers.** A simple icon system may need only Atoms + Element-level + Compositional. A complex diagrammatic system may need all six. The rule is *use the tier table to test the plan*, not to force it.

## §B.6 — Worked example: riken nine-file split

The riken figure-styling canon is held up as the worked example. Its nine files map onto the tier table as follows:

| Riken file | Tier |
|:---|:---|
| `01-color-palette.jsx` | Atoms |
| `07-dashed-vocabulary.jsx` | Atoms |
| `02-registers.jsx` | System-level invariants |
| `03-containers.jsx` | Element-level rules |
| `04-node-stem-pill.jsx` | Element-level rules |
| `05-keystone.jsx` | Element-level rules |
| `06-edges.jsx` | Relation-level rules |
| `08-perceptual-modulation.jsx` | Cross-cutting modulation |
| `09-tables.jsx` | Compositional rules |

Notice three things:

1. **Every tier is present.** Riken's assets need rules at every level.
2. **The element tier has three chapters** — at the top of the comfortable range — because containers, nodes, and keystones each have distinct closed sets and prohibitions (passes §B.2 condition 1).
3. **Atoms have two chapters** — colour and dash-pattern — because they use different visual channels (passes §B.3's first anti-test).
