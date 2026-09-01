# Register Rules

Universal countermeasures to LLM-default rhetoric. These are tier-one rules: they correct generation priors that are constant across authors and projects, so they apply to every stakeholder-voiced draft unless the author's doctrine explicitly overrides one.

A deliberate omission: this file describes failure patterns structurally and never quotes sample sentences. Quoted examples become templates; a model given three sample openers will reproduce their sentence structure instead of generalizing the rule. Where calibration is needed, it comes from the mechanisms in the second half of this file, not from enumerated examples.

## Contents

- The pattern lint: LLM-default rhetoric to catch
- Causal locality
- Discourse level
- Claims and their scoping
- Calibrated vividness: holding the register between two failure poles
- Calibrated clarity: the product of precision, method, and simplification
- Mechanics that hold everywhere

## The pattern lint: LLM-default rhetoric to catch

Word filters miss these because they are shapes, not words. Self-check every draft against each shape:

| Pattern | What it does |
|---|---|
| Marketing and talk-stage lexicon | Words and constructions from pitch decks and stage talks rather than from the signer's working vocabulary |
| Teaser openers | A short sentence that announces something is coming instead of delivering it; the content arrives one sentence late |
| Definitional announcements | Declaring that a term has a precise meaning and then giving it, instead of stating the thing directly |
| Enumerated-failure cadence | Numbering problems rhetorically so the list structure carries drama the content does not |
| Fragments for punch | Sentence fragments used as emphasis devices. The compound failure is the run: several in sequence, standing in for the subordination that would have carried the argument |
| Antithesis as ornament | A contrast constructed for rhythm when the contrast is not the content. Contrastive phrasing is correct exactly when the contrast is the point being made |
| Ascending adjective or noun triples | Three-beat escalations that read as rhetoric unless the three items are independently load-bearing |
| Defensive framing | Telling readers their own expectations or pre-empting objections on the page. Defenses live in reserve and are used when challenged |
| Punctuation as a crutch | Colons and semicolons as frequent joints. They are effects; spend them only where the effect is earned (a true definition, a claim-then-cause reveal, one list), a few per section |

## Causal locality

Every sentence grounds the one that follows. Answers sit directly after their questions, an actor directly after the thing it acts on, and hidden premises get spoken in one plain clause rather than left for the reader to supply. A passage can be logically complete and still read as disconnected when its links are long-range; readers do not reconnect separated items. Two consequences:

- Judge drafts by edge length, not connectivity: how far each pronoun, definite reference, or presupposition reaches back to its supplier. Adjacent is good; a reach across several sentences is a defect even when the graph technically connects.
- Downgrade sufficiency claims to the necessity a skeptic would grant ("holds only when" rather than "is guaranteed by").

**Connection is carried by the joint, not by adjacency.** Two statements placed next to each other assert that some relation holds without saying which one, and the reader supplies whichever is cheapest — often the wrong one, sometimes none. Where the relation is causal, concessive, conditional or temporal, the joint has to name it. The diagnostic is a passage of individually true statements that somehow does not argue anything: its subordination has been dissolved, the clauses separated into sentences, and the connectives discarded with them. The repair is not longer sentences but the restored joint, which is frequently shorter than the two sentences it replaces.

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

## Calibrated clarity: the product of precision, method, and simplification

Clarity is not produced by removing difficulty. It is produced by three factors applied together, and they combine multiplicatively rather than additively, so any factor left at zero zeroes the result:

- **the most precise available word** for the object, the action, and the actor;
- **a deliberate stylistic method**, chosen for this passage;
- **simplification of what is genuinely incidental** to the point being made.

Simplification alone yields vagueness. Precision alone yields jargon. Method alone yields ornament. The failure this rule set exists to prevent is the first, because it is the one a model reaches for by default: of the three factors, simplification is the only one that can be executed by deletion, so an instruction to avoid jargon collapses to it unless the other two are named as obligations. The result reads as evasive rather than plain, and it is recognisable by a uniform blandness that never risks a wrong word because it never risks a specific one.

1. **Name the object, then teach the name in the same breath.** A named object can be looked up, disputed, and remembered; a paraphrased one evaporates on the page. Where a term of art is the precise word, write it and attach its meaning by apposition inside the sentence, never as a definitional announcement and never as a glossary paragraph. A reader outside the specialty does not need less precision; they need the same precision with the teaching attached. Substituting a category word for a name treats the reader as unable to learn a noun, and vagueness offered as courtesy still reads as vagueness.
2. **Simplification is the last of the three to reach for, not the first.** Apply it to complexity that is incidental to the point. Applied to the point itself it removes the content and leaves the shape of an explanation behind.
3. **The stylistic method is chosen, not defaulted.** Name it before drafting the passage: apposition; the concrete instance placed before the abstraction; the telling detail carrying a general condition; sentence length varied so pace tracks meaning; controlled repetition of a phrase that has earned it; withholding and later revealing; analogy reserved for work no literal phrase can do. A passage drafted with no method chosen defaults to enumeration.
4. **Under-precision is unrecoverable in the same way flattening is.** A specific word that overreaches is one veto away from fixed. A word that was never risked leaves nothing to veto, and a reviewer cannot object to what is not on the page. Under uncertainty, write the precise word and flag it.
5. **Both directions get reported.** A review returns findings for impenetrable passages and for vague ones, or states explicitly that a direction is clean. Reporting only jargon drives the next draft into blandness, which is the same class of defect as reporting only inflation driving it into flatness.
6. **The offence test.** A passage that reads as though it were written to avoid giving offence has failed, whatever else is true of it. Draft for a sharp colleague in an adjacent field who will be irritated by condescension and pleased by exactness.

## Mechanics that hold everywhere

- Introduce every abbreviation fully at its first use in the document, even obvious ones, and re-derive first-use positions after any content move. The author's doctrine may list deliberate exceptions; only those are exempt.
- Use one term per concept throughout a document; synonym rotation reads as imprecision in signed prose.
- Keep the register discipline in every layer of the production chain, including design documents, specs, and captions drafted inside them. Drift introduced upstream compounds downstream.
