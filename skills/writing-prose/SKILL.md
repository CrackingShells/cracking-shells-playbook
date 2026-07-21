---
name: writing-prose
description: Drafts, revises, trims, or reviews prose the user will sign in their own name, in their voice, with register control against LLM-default rhetoric. Use whenever the task touches stakeholder-voiced text (applications, proposals, papers, cover letters, reports, statements, announcements, captions), even for small requests like tightening a paragraph, fixing a caption, or replacing a boring sentence, and when reviewing or verifying such prose. Also use to set up or maintain a project's prose config under __canons__/prose_writing/. If the project has a more specific prose skill for the document set at hand, prefer that one; this skill is the general method those derive from.
---

# Writing Prose

Prose someone signs is judged twice: by the signer, who must be able to say they wrote every sentence, and by the audience it was written to move. This skill carries the method for passing both judgments. It exists because unguided LLM prose has a recognizable default register that signers reject, and because the corrections a signer makes are expensive; the method's job is to make each correction paid for once, ever.

## Rule scopes

Every rule this skill works with is either universal or scoped by a path glob. The scope tells you where the rule lives, how long it is valid, and which documents it binds.

1. **Universal**: countermeasures to LLM-default rhetoric. Constant across authors and projects because the priors they correct are constant. They live in this skill and bind every invocation: read [references/register.md](references/register.md) before drafting a first sentence.
2. **Scoped**: everything the harvesting loop records, on a continuum of path globs. `~/**` is the author's standing doctrine (punctuation rules, vocabulary boundaries, verification preferences), portable across all their assignments; a project glob is that assignment's config (protected wording, result scoping, authority precedence, terminology exceptions), living at `__canons__/prose_writing/`; a narrower glob binds one document family within a project. Wider is better: when in doubt, a rule belongs to the widest scope under which it holds.

Run `writing-prose rules <document>` (uninstalled: `python scripts/writing_prose/cli.py rules <document>`) to list the recorded rules binding a given document; by default it filters to conventional authoring formats, so code and config files never match, and the default is maskable per call (`-a`/`--allow` admits extra patterns, `-d`/`--deny` rejects, deny wins). Installing the CLI is optional; see [scripts/INSTALL.md](scripts/INSTALL.md).

## Workflow

1. **Locate the assignment config.** Look for `__canons__/prose_writing/` in the project root. If it exists, read `config.md` first; it names the audience, the register anchor, the gates and their owners, and where the author-level doctrine lives. If it does not exist, ask the user one question: run the setup interview now, or proceed on author-level defaults? Both answers are fine; the interview takes minutes and the config can also accrete later through harvesting. See [references/config-and-harvest.md](references/config-and-harvest.md) for the interview and the file set.
2. **Load the author-level doctrine** from wherever `config.md` points (commonly agent memory or a personal doctrine file). If nothing exists yet, the first drafting rounds double as its harvest.
3. **Draft inside the register, not toward it.** Read [references/register.md](references/register.md) first, then write in the target register from the opening sentence. Polishing a draft into a voice it was not written in costs more than starting there, and the signer can tell.
4. **Run the reviews before presenting.** The three passes and how to dispatch them are in [references/reviews.md](references/reviews.md). Give review agents the complete protection material (see below); a reviewer who knows only half the protected wording will propose deleting the other half.
5. **Present in rounds.** The signer is the only reliable detector of voice drift; short reviewable rounds beat long private polishing. Surface any phrasing you are unsure they would own, and name every trade a constraint forced (words cut, a claim softened, a caption shortened) so each can be vetoed individually.
6. **Harvest every correction.** When the signer corrects anything, classify it to the most general tier that fits and record it the same day, per [references/config-and-harvest.md](references/config-and-harvest.md). The default is general: a new correction is usually about the model or about the author, only occasionally about the assignment.

## Protection material

Before any review or trim, assemble one merged list of what cannot change, from both of its homes: the config's protection list (locked wording, mandated content) and the honesty register (claims with their exact scoping). Hand the merged list to every reviewer and consult it before every cut. Protected content is never traded for fit, for flow, or for a reviewer's better sentence; a proposal that touches it is surfaced to the signer, not adopted.

## Fit and budgets

Length constraints (page counts, word limits, form character caps) are common in signed prose. Principles that hold everywhere:

- The rendered artifact is the ground truth; a recorded number about remaining room goes stale with every edit. Measure at session start, and prefer recording bracketed observations (this length passed, that length failed) over derived formulas.
- Any lever you pull to gain room must state its validity condition. A lever that is mechanically available but silently degrades the artifact (compressing a figure, cutting a legally required clause) is not a lever; if one is documented without its condition, treat it as suspect and verify before pulling.
- Fit trims come from unprotected prose. When only protected content is left to cut, the budget question goes to the signer or to the gate's owner.
- A gate needs a named owner and a waiver path written next to it. A gate stated as absolute will eventually be enforced against the person it serves.

## Non-negotiables

- Voice ownership is the gate: nothing ships that the signer could not plausibly have written. When uncertain, ask; a guessed phrasing that feels foreign wastes the draft.
- Accuracy scoping is never traded: every claim keeps its honest qualifiers regardless of what fit or flow would prefer.
- Verification of prose is either mechanical and falsifiable (a character-class check, a byte diff, computed arithmetic) or done by a reader (an agent or the signer). A keyword search that would pass any text mentioning the keyword verifies nothing.
- The skill's own working documents (reports, comments, commit messages) follow the author's register rules too; vocabulary drift introduced in intermediate documents seeds the next draft's vocabulary and compounds.
