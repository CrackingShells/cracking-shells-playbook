# Assignment Config and Correction Harvesting

The per-assignment layer is a config, not a doctrine: it records choices and facts specific to one document set, and nothing that a higher tier already carries. It lives in the project at `__canons__/prose_writing/`, beside any visual-identity canon the project keeps, because both are the same kind of object: production constraints discovered through stakeholder reaction and then codified.

## Contents

- The file set
- The setup interview
- The harvesting loop
- Tier classification
- What never goes in the config

## The file set

Flat names, no round prefixes or version suffixes; those conventions belong to incremental report authoring under `__reports__/`, while these files are living state, edited in place with dated entries.

| File | Carries |
|---|---|
| `config.md` | The interview output: audience and their reading conditions, register anchor, length gates with their owners and waiver paths, authority precedence (which project documents win when they disagree), and a pointer to where the author-level doctrine lives |
| `protection_list.md` | Locked wording kept verbatim, and mandated content per document or section, each entry with the source that mandates it |
| `honesty_register.md` | Every claim the documents make, with its exact scoping; the wording that keeps each claim true |
| `terminology.md` | The abbreviation ledger, first-use decisions, and named exceptions to general mechanics |
| `corrections_log.md` | The harvest destination: dated corrections, each with its tier classification and backport status |

Create the directory and files only when there is content to put in them; empty scaffolds invite invented content.

## The setup interview

Offered once, when the skill finds no config in a project; the user can decline and proceed on author-level defaults, and the config then accretes through harvesting instead. Keep it to what only the user can answer:

1. Who reads this, and under what conditions (a panel skimming six pages, a committee reading one, a stranger deciding in ninety seconds)?
2. What register anchor fits the piece? Ask for it in the user's words rather than offering a menu.
3. What hard gates exist (page, word, character limits), who owns each, and what happens when content and gate conflict?
4. Which existing documents are authorities on content, and in what order of precedence?
5. Where does your standing writing doctrine live, if anywhere yet?

Answers seed `config.md`. Anything the user does not know yet is left absent, not defaulted; absence is informative and a default would masquerade as a decision.

## The harvesting loop

Every correction the signer makes is recorded the same day it happens, because unrecorded corrections get re-made and each re-making costs a review round. The loop:

1. Capture the correction with the text before and after, in `corrections_log.md`, dated.
2. Classify its scope (next section).
3. Backport it to its scope's home: any-author insights are proposals against this skill's references, `~/**` rules go to the author root, project and narrower rules to the config file they belong in, with an explicit `scope:` line when narrower than the project. Mark the log entry backported once done.
4. If the correction supersedes something already recorded anywhere (a config entry, a report, a comment), update or mark the stale record where it stands. Stale records get mined as truth by later sessions and resurface rejected decisions.

## Scope classification

Every recorded rule carries a path-glob scope in its frontmatter (`scope: <glob>`), and a rule binds a document when the document's path matches the glob. The scopes form a continuum:

- `~/**`: the author's standing doctrine, valid for everything they sign anywhere. Lives in the author root (`~/.config/writing-prose/`), not in any project.
- `<project>/**`: this assignment's config, the files in this directory. Their location default is the project scope, so most need no explicit frontmatter.
- Narrower globs (for example `<project>/application_documents/doc1-*.md`): rules binding one document family, recorded here with an explicit `scope:` line.

To classify a correction, ask for the widest glob under which it holds, and default wide when unsure: experience across projects shows most of what a signer flags is standing preference (`~/**`), not assignment fact. Classifying wide is what keeps project configs small and corrections paid for once. Two special cases:

- Would the correction hold for a different author entirely? Then it corrects a model prior and no glob is wide enough: it is a proposal against this skill's own references, the rarest and most valuable find.
- A correction can split: the principle goes to a wider scope while the specific instance stays at a narrow one.

Run `writing-prose rules <document>` (uninstalled: `python <skill>/scripts/writing_prose/cli.py rules <document>`) to invert the mapping: given a document, it lists every recorded rule file whose scope matches, widest first, filtered by default to conventional authoring formats (markdown, tex, docx, and kin) so code and configuration files never pick up prose rules; mask the default per call with `-a`/`--allow` and `-d`/`--deny` globs, deny winning.

## What never goes in the config

- Measurements of the rendered artifact (remaining room, current page counts, slack). They go stale with every edit; record them next to the artifact they measure, dated, and re-measure at session start.
- Rules already carried by a higher tier. Duplicating them here creates two copies that will disagree after the next correction.
- Content invented to fill a template. Every entry needs a source: a stakeholder decision, a document that mandates it, or a recorded correction.
