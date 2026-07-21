# Register Rules

Universal countermeasures to LLM-default rhetoric. These are tier-one rules: they correct generation priors that are constant across authors and projects, so they apply to every stakeholder-voiced draft unless the author's doctrine explicitly overrides one.

A deliberate omission: this file describes failure patterns structurally and never quotes sample sentences. Quoted examples become templates; a model given three sample openers will reproduce their sentence structure instead of generalizing the rule. Where calibration is needed, it comes from the mechanisms in the second half of this file, not from enumerated examples.

## Contents

- The pattern lint: LLM-default rhetoric to catch
- Causal locality
- Discourse level
- Claims and their scoping
- Calibrated vividness: holding the register between two failure poles
- Mechanics that hold everywhere

## The pattern lint: LLM-default rhetoric to catch

Word filters miss these because they are shapes, not words. Self-check every draft against each shape:

| Pattern | What it does |
|---|---|
| Marketing and talk-stage lexicon | Words and constructions from pitch decks and stage talks rather than from the signer's working vocabulary |
| Teaser openers | A short sentence that announces something is coming instead of delivering it; the content arrives one sentence late |
| Definitional announcements | Declaring that a term has a precise meaning and then giving it, instead of stating the thing directly |
| Enumerated-failure cadence | Numbering problems rhetorically so the list structure carries drama the content does not |
| Fragments for punch | Sentence fragments used as emphasis devices |
| Antithesis as ornament | A contrast constructed for rhythm when the contrast is not the content. Contrastive phrasing is correct exactly when the contrast is the point being made |
| Ascending adjective or noun triples | Three-beat escalations that read as rhetoric unless the three items are independently load-bearing |
| Defensive framing | Telling readers their own expectations or pre-empting objections on the page. Defenses live in reserve and are used when challenged |
| Punctuation as a crutch | Colons and semicolons as frequent joints. They are effects; spend them only where the effect is earned (a true definition, a claim-then-cause reveal, one list), a few per section |

## Causal locality

Every sentence grounds the one that follows. Answers sit directly after their questions, an actor directly after the thing it acts on, and hidden premises get spoken in one plain clause rather than left for the reader to supply. A passage can be logically complete and still read as disconnected when its links are long-range; readers do not reconnect separated items. Two consequences:

- Judge drafts by edge length, not connectivity: how far each pronoun, definite reference, or presupposition reaches back to its supplier. Adjacent is good; a reach across several sentences is a defect even when the graph technically connects.
- Downgrade sufficiency claims to the necessity a skeptic would grant ("holds only when" rather than "is guaranteed by").

Reordering content changes locality. After moving any block (a figure, a section, a quote), recompute what now comes first: first uses of abbreviations, first mentions that later references depend on, and any text like a caption that now reads before its context exists.

## Discourse level

Keep the discourse at the level the reader can hold at that point in the document. A mechanism the reader cannot yet relate to its parts contributes nothing where it stands; either build the ladder first or let a higher-level statement carry the point. Corollary: explanations and analogies given to you as teaching material during the session are not prose; find the target-register expression of the same logic.

## Claims and their scoping

- State status with plain temporal markers (already, currently, at present, explicit dates) rather than rhetorical emphasis. The reader should learn when something is true, not be pushed to feel it.
- Every claim keeps its honest qualifiers. Scoping is recorded in the assignment's honesty register and is not negotiable during trims or reviews.
- When a drafted word connects to the document's central idea, spend a sentence cashing the connection rather than leaving it implicit; unspent thematic connections are the cheapest value on the page.

## Calibrated vividness: holding the register between two failure poles

Polarized writing fails at either pole: inflated rhetoric on one side, flat beige prose on the other. LLMs presented with a one-sided rule set buy safety at the unnamed pole, so this rule set names both and gives the calibration mechanisms explicitly. The goal is finesse across the lexical field of the piece: nuanced word choice, vivid moves spent deliberately where they maximize effect.

1. **Tie-break under uncertainty: keep the vivid phrasing and flag it.** When torn between a plainer and a more vivid rendering, do not silently pick the plain one. The two errors are asymmetrically recoverable: a flagged overreach is one veto away from fixed, while a flattened sentence hides the option and nobody can veto it back into existence.
2. **The vividness budget is spendable, and unspent budget is a defect.** Each section carries a small budget of vivid moves (a few, placed where they maximize effect; a vivid word repeated loses its charge). Report an over-spent section and an under-spent section the same way: both are register failures.
3. **Both poles get reviewed.** Every review pass returns findings for both failure directions or states explicitly that a direction is clean. A review that only lints one pole drives drafting to the other.
4. **Name rules and files after the target, never after a failure pole.** Instruction vocabulary leaks into generation; a rule named for restraint produces restraint, a rule named for the desired middle produces the middle. This applies to rule names, config keys, and token or variable names in any project asset.

## Mechanics that hold everywhere

- Introduce every abbreviation fully at its first use in the document, even obvious ones, and re-derive first-use positions after any content move. The author's doctrine may list deliberate exceptions; only those are exempt.
- Use one term per concept throughout a document; synonym rotation reads as imprecision in signed prose.
- Keep the register discipline in every layer of the production chain, including design documents, specs, and captions drafted inside them. Drift introduced upstream compounds downstream.
