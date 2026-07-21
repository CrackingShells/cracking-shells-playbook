# Review Passes

Three passes, run before presenting any round to the signer. They are complementary: the first checks structure, the second checks composition, the third checks what a machine can falsify. None of the three substitutes for the signer's read; their job is to make that read cheap.

## Contents

- What every reviewer receives
- Pass 1: causal-locality graph
- Pass 2: recomposition
- Pass 3: mechanical checks
- Handling reviewer output

## What every reviewer receives

Dispatch passes 1 and 2 as independent agents, and give each one:

- The draft, with its reading order made explicit (if a figure or caption physically precedes the prose, say so; reviewers otherwise assume source order is reading order).
- The register rules ([register.md](register.md)) and the author-level doctrine.
- The complete merged protection material: the assignment's protection list (locked wording, mandated content) AND its honesty register. This completeness matters: a reviewer given only one of the two sources will, with the best intentions, propose rewording something the other source protects. It happens reliably, not occasionally.
- The instruction to review both register poles (inflation and flatness) and to return findings or an explicit all-clear for each.

## Pass 1: causal-locality graph

An agent builds a per-sentence graph of the piece and judges edge lengths, not connectivity.

Method to give the agent:

1. Number every sentence in reading order, including captions and any text the reader meets before the body.
2. For each sentence, list its assertions and its presuppositions (what the reader must already hold for it to land).
3. Draw a labeled edge from each presupposition, pronoun, or definite reference to the sentence that supplies it.
4. Emit the edges as JSON and run `writing-prose edges edges.json` (uninstalled: `python <skill>/scripts/writing_prose/cli.py edges edges.json`) for the arithmetic: it judges each reference by its nearest earlier supplier and returns a worst-first verdict table (dangling, forward, long, ok). The semantic work of finding the edges stays with the agent; the script only removes counting slips and keeps verdicts consistent.
5. Beyond what the script grades, report as defects: sufficiency claims a skeptic grants only as necessity, and claims that exist only in a caption or sidebar and never in prose.
6. For each defect, give the direction of repair (reorder to adjacency, or speak the premise in one plain clause), not a rewrite.

Ask for the verdict table plus the defect list. The table is what makes disagreements with pass 2 adjudicable. Never reuse a previous round's edge list: any edit renumbers sentences, so an edge list describes exactly one draft state. Keeping the round's JSON as a dated record is fine; feeding it back in as input is not.

## Pass 2: recomposition

A strong reviewer agent reads the doctrine and the draft and returns numbered proposals, each in the form:

- ORIGINAL: the exact current text
- PROPOSED: the replacement
- WHY: one or two clauses naming the rule involved

Instruct it to propose only the few changes that matter, to say explicitly when something is strong as it stands, and to include a verdict on both register poles. Adopt selectively; the proposals are inputs, not patches. Two standing filters when adopting:

- Anything touching protection material is surfaced to the signer instead of adopted, even when the proposal is better prose.
- When passes 1 and 2 disagree about a sentence, the disagreement itself is the finding; present both positions and let the signer call it.

## Pass 3: mechanical checks

Falsifiable checks only. State each check by what it detects, and use whatever tool the environment provides for it; do not hard-code a specific command line into project documents, because prescribed invocations collide with environment policies and go stale.

Typical checks, drawn from the author doctrine and config:

- Forbidden character classes (specific punctuation or quote characters the author bans).
- Banned vocabulary, with the expected near-misses named so a hit can be triaged instead of pattern-matched (a legitimate technical term sharing a stem with a banned word should be listed as an expected hit).
- Budget gates: the rendered artifact compiled or counted against its limit.

Mechanical checks are necessary, not sufficient. They check characters and words; the failures that matter are shapes and register, which only passes 1 and 2 and the signer catch. Never present a mechanical pass as evidence of prose quality.

## Handling reviewer output

Consolidate the three passes into one round for the signer: what was adopted, what was declined and why, and the short list of genuine calls only they can make. Keep declined proposals visible; a reviewer suggestion the orchestrator silently discarded is indistinguishable from one never made, and the signer may weigh it differently.
