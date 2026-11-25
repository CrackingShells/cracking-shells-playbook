# R01 sync

**Goal**: R01 describes the domain model M0b settled, in the same merge.
**Pre-conditions**:
- [ ] `../../read_model.md` is done and reviewed by Eliott
**Success Gates**:
- ⬜ [static] R01's FSM diagram, §Data Model sketch, I5, I8 and I9 and §Deliberately open agree with `statuses.md`, `stored-schema.md` and `read-model.md`, and the M0b items are no longer marked provisional
- ⬜ [static] The launch section states two version lines (`rdm-schema` alone; `rdm-engine`, `rdm-mcp` and `rdm-cli` together) and the format-version rule from `read-model.md`
- ⬜ [static] I1 and §Proposed State are unchanged by this leaf: depth stays the ordering, and M0 owns §Proposed State
**References**: [R01](../../../../__design__/rdm/architecture.md) — the sections to change; [the three spec documents](../../../../__design__/rdm/) — the source of truth

## Step 1: Bring R01 in line
**Goal**: R01 points to the spec documents and states their decisions.
**Implementation Logic**:
1. Replace the provisional sketch in §Data Model with a summary and a link to `stored-schema.md`.
2. Redraw the FSM in §Graph operations with the five statuses, and link `statuses.md`.
3. Restate I5, I8 and I9 under the sealing rule.
4. In §Deliberately open, remove the M0b items.
5. In the launch section, state the version lines.
6. Note the change and its date in the header paragraph.
7. Leave the skill text alone: M8 owns it.
**Deliverables**: `__design__/rdm/architecture.md`: §Data Model, §Graph operations, §Contracts & Invariants, §Deliberately open, §Launch and distribution
**Consistency Checks**: `! grep -n 'amendment: content edit\|blocked --> inprogress' __design__/rdm/architecture.md` (expected: PASS)
**Commit**: `docs(design): bring R01 in line with the m0b domain model`
