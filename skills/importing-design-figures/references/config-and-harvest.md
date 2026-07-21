# Project Config and Correction Harvesting

The per-project layer is a config, not a method: it records the facts of one Claude Design
project and one consuming document set, and nothing the universal method already carries. It
lives at `__canons__/design_figures/`, beside any visual-identity canon the project keeps,
because both are the same kind of object: production constraints codified so assets come out
right.

## The file set

Flat names, no round prefixes or version suffixes (those belong to `__reports__/`); these are
living state, edited in place with dated entries. Create a file only when there is real
content for it — empty scaffolds invite invented content.

| File | Carries |
|---|---|
| `config.md` | Project id and where the component files are listed; the `--field` token if non-default; the target artboard width and the consuming document's text-page height (the two numbers the anti-squish math needs); the **placement model** (how the document embeds an image — fixed box? aspect preserved? any height→layout threshold); fit gates with their owners and waiver paths; filename couplings; a pointer to where cross-project render prefs live |
| `figure_ledger.md` | One row per figure: figure id → current source component filename (tracking `… v2` renames) → the plate **selector** that works → last measured **aspect** and **height%**, dated. The durable form of "which selector, what height" |
| `render_log.md` | Dated bracketed observations: this height fit / that height failed, page counts under a configuration. Never derived formulas or a single "remaining room" number |

## The setup interview

Offered once, when the skill finds no config in a project; the user may decline and proceed
on defaults, and the config then accretes through harvesting. Keep it to what only the user
can answer:

1. Which Claude Design project (id or name), and where are its component files listed?
2. What is the target artboard width, and the text-page height of the consuming document?
   (These drive the height math; a wrong text-page height squishes everything.)
3. How does the document embed an image — fixed box? Does it preserve aspect
   (`keepaspectratio` / width-only) or stretch? Any height→layout threshold?
4. Where do rendered figures live in the repo, and is any filename coupled to logic (e.g. a
   masthead that matches a filename substring)?
5. What fit gate exists (page or size limit), who owns it, and what happens when content and
   gate conflict?
6. Any non-default `--field` / background token colour?

Answers seed `config.md`. Anything the user does not know yet is left absent, not defaulted;
absence is informative and a default would masquerade as a decision.

## The harvesting loop

Record every correction the same day it happens, because unrecorded ones get re-made and
each re-making costs a round.

1. Capture the fact where it belongs: a new/changed selector or a fresh measured height goes
   to `figure_ledger.md`; a fit result goes to `render_log.md`, dated.
2. Classify its scope (next section).
3. Backport it to that scope's home: a universal insight (a new format fact, a selector
   pattern that holds for any project, a harness fix) is a proposal against this skill's
   references or scripts; cross-project render preferences go to the author root; project
   facts to the config files here.
4. If the correction supersedes something recorded elsewhere (a stale ledger height after a
   reshape, a superseded component version, a note in a report), update or mark the stale
   record where it stands. Stale records get mined as truth by later sessions.

## Scope classification

Ask for the widest scope under which the fact holds, and default wide when unsure:

- **Universal** — a property of the `.dc.html` format, the render harness, or fixed-box
  placement physics. Holds for any project and any document. It is a proposal against this
  skill, the rarest and most valuable find (e.g. "auto-detect should read `--field` from
  `:root`").
- **Cross-project (author) render prefs** — a standing preference the author carries across
  projects (default scale, the zero-dep Chrome choice). Lives in the author root, rare.
- **Project** (`__canons__/design_figures/`) — this project's ids, tokens, selectors,
  measured heights, placement model. The default home for most facts.
- **Narrower** — a rule binding one document family within the project; record it here with
  an explicit note of the narrower scope.

A correction can split: the principle goes wide (a selector *pattern*) while the instance
stays narrow (this figure's exact selector in the ledger).

## What never goes in the config

- **Measured heights treated as permanent.** They change when a component is reshaped, so
  they are dated observations in the ledger, re-measured on each render — never a fixed
  constant, and never in the skill's references.
- **Rules already carried by the universal method.** Duplicating them creates two copies
  that disagree after the next change.
- **Content invented to fill a template.** Every entry needs a source: a stakeholder
  decision, a document that mandates it, or a recorded render.
