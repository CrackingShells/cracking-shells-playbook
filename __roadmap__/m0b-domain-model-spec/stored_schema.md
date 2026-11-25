# Stored schema

**Goal**: Every stored fact has a named field and shape, and every production of the legacy grammars maps to a stored field, a derived field or a recorded drop, in a committed traceability table with a check script.
**Pre-conditions**:
- [ ] `status_semantics.md` Step 1 is committed on the same task branch (the status field and comments follow it)
**Success Gates**:
- ⬜ [static] `__design__/rdm/stored-schema.md` gives the field inventory, with name, type, required or optional, and meaning, for `_dir.json`, a leaf, a step, a criterion, a reference, a commit, a history entry, a comment and an amendment
- ⬜ [run] `uv run --no-project tools/check_rdm_traceability.py` passes. It extracts every production name from `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh grammar leaf` and `grammar readme`, and fails unless each appears in `__design__/rdm/traceability.md` with a disposition of `stored`, `derived` or `dropped` and a target or reason
- ⬜ [static] Legacy data that the grammar accepts but real campaigns stretch has a stated home or drop: the 16 checked pre-conditions, Progress rows on directories, em-dash Progress cells the validator skips, commit counts above the step count, and the `Requires` step field
**References**: [R01 §Data Model](../../__design__/rdm/architecture.md) — the sketch and the five omissions it admits; [observation §Evidence](../../__reports__/rdm_domain_model_review/00-observation_v0.md) — the grammar-versus-sketch table; [legacy grammars](../../skills/managing-roadmaps/scripts/dirtree-rdm/grammar/) — `leaf.bnf` (22 productions) and `readme.bnf` (58)

## Step 1: Write the stored schema
**Goal**: The stored types are specified field by field.
**Implementation Logic**:
1. Start from R01's sketch: `_dir.json`, a leaf, a step, a history entry and an amendment. Settle these defaults, and argue against any you find defective:
   - **Criteria:** pre-conditions and gates become `{text, met: bool}`, at both leaf and directory level.
   - **Steps:** a step carries `number`, `title`, `goal`, `implementation_logic` (lines), `deliverables` (lines), `references[]`, `checks[{command, expected: PASS|FAIL}]`, `commit{type, scope?, breaking, summary}`, `requires[]` (optional), `done`, `commit_sha?` and `done_at?`.
   - **References:** `{label?, title, target, note?}`. `label` keeps the `R01` labels that other text cites, and directory "Reference Documents" and leaf references share the shape.
   - **Status fields:** `status` uses the M0b vocabulary, `status_reason?` replaces `blocked_reason`, and `history[]` is `{ts, from, to, actor, reason?, override?}`.
   - **Comments:** `comments[] {ts, actor, text}`, append-only, which is the only change a sealed node accepts.
   - **Amendments:** `{schema_version, id (ULID), legacy_id?, ts, actor, description, source?, op_id?, changes[], snapshot?}`. The display id is derived from the ULID, not sequential, because a sequential id collides when two branches mint one. Legacy `A1` is kept in `legacy_id`.
   - **Directory notes:** `_dir.json` gains `notes` for the Progress notes of a directory.
   - **Prose:** gotchas are stored as prose lines, as all multi-line prose is stored.
   - **Unknown fields:** each type keeps them, for I12.
   - **`schema_version`:** it is the format version as `"MAJOR.MINOR"`.
2. For each field, give its type, whether it is required, and one-line meaning. Mark what is new against R01.
**Deliverables**: `__design__/rdm/stored-schema.md`: sections `Files`, `Directory`, `Leaf`, `Step`, `Criterion`, `Reference`, `Commit`, `History`, `Comment`, `Amendment`, `Format version`, `Changes against R01`
**Consistency Checks**: `for t in Directory Leaf Step Criterion Reference Commit History Comment Amendment; do grep -q "^## $t" __design__/rdm/stored-schema.md || exit 1; done` (expected: PASS)
**Commit**: `docs(design): specify the rdm stored schema`

## Step 2: Trace every legacy production
**Goal**: No legacy fact is dropped silently.
**Implementation Logic**:
1. **Table:** list every production of both grammars, as printed by `dirtree-rdm.sh grammar leaf|readme`, one row each. Give its disposition: `stored` (with the field path), `derived` (with the derivation) or `dropped` (with the reason). Presentation drops include the classDef colours, the emoji labels, the backticks around the commit, and the `(expected: …)` trailer, which becomes `checks[].expected`.
2. **Stretched legacy data:** add a section for data the grammar accepts but real campaigns stretch, as listed in the third Success Gate, with evidence from `__roadmap__/*`.
3. **Check script:** `tools/check_rdm_traceability.py` is stdlib-only. It shells out to the grammar printer, parses each `<name> ::=` line, and checks that every name appears in a table row with a valid disposition.
**Deliverables**: `__design__/rdm/traceability.md`: sections `Leaf grammar`, `README grammar`, `Stretched legacy data`; `tools/check_rdm_traceability.py`: functions `production_names`, `table_rows`, `main`
**Consistency Checks**: `uv run --no-project tools/check_rdm_traceability.py` (expected: PASS)
**Commit**: `docs(design): trace the legacy grammar to the rdm stored schema`
