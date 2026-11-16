# Status semantics

**Goal**: The meaning of every status is written as a transition table, with who sets each move, its preconditions, its effect on the ready set and the sealing rule. It follows the vocabulary Eliott chose on 2026-11-16.
**Pre-conditions**:
- [x] Vocabulary decided by Eliott (2026-11-16): `planned / ongoing / paused / cancelled / done`
**Success Gates**:
- ⬜ [static] `__design__/rdm/statuses.md` has a transition table covering every ordered pair of the five statuses: allowed or refused, who sets it, required reason, precondition and effect on the ready set
- ⬜ [static] The document states the sealing rule per status, whether a step recorded done inside an `ongoing` leaf is sealed, how `paused` and `cancelled` apply to a directory (applied to its leaves) and how a directory's status is derived
- ⬜ [static] The legacy mapping is stated: `inprogress`→`ongoing`, `blocked`→`cancelled` (with the reason that the legacy skill used it to abandon work), `amendment`→`planned`, with provenance from the amendment log, and `done`→`done`
- ⬜ [behavioral] Eliott has answered the one open question the document raises (the scope of `paused` on deeper levels) and the answer is written in
**References**: [R01 §Graph operations, §Contracts & Invariants](../../__design__/rdm/architecture.md) — the provisional FSM, I5, I8 and I9 that this replaces; [graph-model.md §Status State Machine](../../skills/managing-roadmaps/references/graph-model.md) — the legacy meanings; [M0b README Gotchas](README.md) — the decisions already taken

## Step 1: Write the status semantics
**Goal**: One document answers what each status means and how it may change.
**Implementation Logic**:
1. **Statuses and transitions:**
   - Define each of `planned`, `ongoing`, `paused`, `cancelled` and `done` in one sentence.
   - Write the 5x5 table of moves: for each, whether it is allowed, who may set it (agent, human or both), whether a reason is required, its precondition, and its effect on the ready set.
   - Defaults to state, and to argue against if you find a defect:
     - `done` and `cancelled` are terminal.
     - `paused → ongoing` and `paused → planned` are allowed with a reason.
     - `ongoing → done` requires every step done with its commit (I8).
     - Entering `paused` or `cancelled` requires a reason.
2. **Ready set:**
   - A leaf is ready when it is `planned` and every shallower leaf is `done` or `cancelled`. So `cancelled` holds nothing back, as Eliott asked.
   - Whether `paused` holds back every deeper leaf (R01's global rule) or only leaves under its own parent (the legacy parent-scoped rule) is Eliott's choice. Write both readings with one example each from the meta-roadmap, recommend one, and leave it marked OPEN for him.
3. **Subtrees:**
   - Pausing or cancelling a directory sets every non-terminal leaf under it, as one operation, recorded once.
   - A directory's status is derived from its leaves. State the derivation as a rule table.
4. **Sealing:**
   - A `done` or `cancelled` node's content cannot change; only an append-only comment can be added to it.
   - Content edits to `planned`, `ongoing` or `paused` nodes need an amendment note.
   - Status moves and progress never produce an amendment (I5).
   - State whether a step recorded done inside an `ongoing` leaf is sealed. The recommended answer is yes, because its commit is recorded.
5. **Legacy mapping:** map each legacy status to a new one, as in the Success Gates.
6. **R01 deltas:** list what changes in R01's FSM diagram and in I5, I8 and I9 (acknowledgement for reopening becomes sealing). Do not edit R01: `r01_sync` does.
**Deliverables**: `__design__/rdm/statuses.md`: sections `Statuses`, `Transitions`, `Ready set`, `Subtrees and directories`, `Sealing`, `Legacy mapping`, `R01 deltas`, `Open for Eliott`
**Consistency Checks**: `for s in planned ongoing paused cancelled done; do grep -q "\`$s\`" __design__/rdm/statuses.md || exit 1; done` (expected: PASS)
**Commit**: `docs(design): specify the rdm status semantics`
