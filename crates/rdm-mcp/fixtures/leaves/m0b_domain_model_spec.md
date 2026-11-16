# M0b Domain model spec

**Goal**: The meaning of every stored fact and the shape every consumer receives are specified and held in the `rdm-schema` crate, before any milestone builds on them: status meanings, the stored schema inventory and the read model.
**Pre-conditions**:
- [ ] R01 read (`__design__/rdm/architecture.md`) and the gap analysis `__reports__/rdm_domain_model_review/01-gap_analysis_v0.md`
- [ ] Reminder `skills-rdm-M0b` read
**Success Gates**:
- ⬜ `crates/rdm-schema` holds the stored types, the read-model types, the format version (`MAJOR.MINOR`) and the read-model version as constants, with no store and no I/O; JSON Schemas are generated from the types and committed, and CI fails when they drift
- ⬜ The meaning of `blocked` and `amendment` is written as a transition table (who sets each move, its preconditions, its effect on the ready set), and R01 is updated in the same merge
- ⬜ Every production of `leaf.bnf` and `readme.bnf` maps to a stored field, a derived field or a recorded drop, in a committed traceability table
- ⬜ The read model has one worked example per projection, and reports `level` together with where it comes from
- ⬜ Depth stays the only ordering rule for now (Eliott, 2026-11-16): the stored schema has no edge field, the read model derives `level` from depth and reports it, and the open question stays open for a later revisit
**References**: [R01 §Data Model, §Graph operations, §Contracts & Invariants](../../__design__/rdm/architecture.md) — the provisional schema sketch and the status machine this milestone settles; [gap analysis](../../__reports__/rdm_domain_model_review/01-gap_analysis_v0.md) — why the milestone exists and what it owns

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m0b-domain-model-spec/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding the details this leaf does not: the transition table and the meaning of `blocked` (terminal in the legacy skill, resumable in R01; whether a blocked leaf satisfies its level) and of `amendment` (a birth state in the skill, a reopened state in R01; whether it stays a status); the stored shape of per-criterion state, step title, step timestamp, the breaking mark of a commit, structured references and amendment display ids (a sequential id collides when two branches mint one); the projections of the read model and what each carries; how the format version relates to the crate version (the files carry `MAJOR.MINOR`, a format major forces a crate major, a format minor a crate minor); the ordering rule, which stays depth for now by Eliott's decision of 2026-11-16 (the schema gains no edge field; `__reports__/rdm_domain_model_review/00-open_question_v0.md` keeps the options, and adding an optional edge field later is a minor format bump); where the work splits into leaves.
The crate is `rdm-schema`, published at M7 with the other crates and kept at `0.x` until then. Its root manifest uses `members = ["crates/*"]`, agreed with M0, so the two parallel branches never edit the same line.
No adversarial verification by default: the work is specification and Eliott reviews it. A fresh-session read of the traceability table is optional, with a brief that names what to attack (fields dropped silently).
**Deliverables**: `__roadmap__/m0b-domain-model-spec/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m0b-domain-model-spec/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m0b-domain-model-spec milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m0b-domain-model-spec`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. `__design__/rdm/architecture.md` changes in the same merge: §Data Model, §Deliberately open, the crates list and the launch section; the invariant I1 only if the ordering choice changes it.
**Deliverables**: `crates/rdm-schema` with the stored types, the read-model types, the version constants and the committed JSON Schemas; the traceability table; the updated R01
**Consistency Checks**: `cargo test -p rdm-schema` (expected: PASS)
**Commit**: `feat(rdm-schema): land the domain model spec`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m0b-domain-model-spec/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M0b` complete; `skills-rdm-M1` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m0b-domain-model-spec/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/` (expected: PASS)
**Commit**: `docs(reports): close the m0b-domain-model-spec milestone`
