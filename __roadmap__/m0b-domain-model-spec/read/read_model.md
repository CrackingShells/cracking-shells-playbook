# Read model

**Goal**: The shape every consumer receives is specified as projections over the stored schema, each with a worked example. `level` is reported together with its source, and both version policies are written down.
**Pre-conditions**:
- [ ] `../status_semantics.md` and `../stored_schema.md` are done and reviewed by Eliott
**Success Gates**:
- ⬜ [static] `__design__/rdm/read-model.md` specifies each projection: the campaign outline with waves, node detail, the ready set with what stalls it, and the amendment log. Each has a field list and a worked JSON example computed by hand from `__roadmap__/roadmap-mcp-rework/` as it stands at the start of this leaf
- ⬜ [static] Every leaf in a projection carries `{level, level_source: "depth"}` and its derived flags (for example `added_by` from the amendment log). No projection field lacks a source in the stored schema or a stated derivation
- ⬜ [static] The version policy is written:
  - The format version is `MAJOR.MINOR` in each file. A format major forces a crate major, and a format minor forces a crate minor.
  - The read-model version has its own constant and its own bump rule.
  - A reader refuses a newer format major.
**References**: [R01 §Graph operations, §Tool surface (suggestion), §Graph view](../../../__design__/rdm/architecture.md) — what the consumers need; [ordering open question](../../../__reports__/rdm_domain_model_review/00-open_question_v0.md) — why `level` carries its source

## Step 1: Write the read model
**Goal**: Consumers have one documented shape to rely on.
**Implementation Logic**:
1. **Projections and consumers:**
   - Define the four projections and name which consumer uses each: M4 tools, M5 `--json` and the M6 view.
   - The M6 app-only tool returns the campaign outline.
2. **Derivations:** derive waves and the ready set by the rule in `statuses.md`. Where the scope of `paused` is still OPEN, use Eliott's answer if it exists, otherwise state the projection under the recommended reading and flag it.
3. **Worked examples:**
   - Compute one example per projection by hand from the real meta-roadmap. Map its legacy statuses through the legacy mapping first.
   - Keep each example short: cut long prose fields to their first line and mark the cut.
4. **Revisions:** state which fields carry revisions (`rev`, `tree_rev`) as opaque strings. M1 defines how they are computed.
5. **Versions:** write both version policies.
**Deliverables**: `__design__/rdm/read-model.md`: sections `Projections`, `Campaign outline`, `Node detail`, `Ready set`, `Amendment log`, `Level and its source`, `Versions`
**Consistency Checks**: `for t in "Campaign outline" "Node detail" "Ready set" "Amendment log" "Versions"; do grep -q "^## $t" __design__/rdm/read-model.md || exit 1; done` (expected: PASS)
**Commit**: `docs(design): specify the rdm read model`
