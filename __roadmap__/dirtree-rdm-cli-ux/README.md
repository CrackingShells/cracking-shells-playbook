# dirtree-rdm-cli-ux

## Context
Second-pass refinement of `dirtree-rdm` CLI output following the validator-diagnostics upgrade landed in commit `fc28ce0`. Manual usage surfaced two UX frictions: (1) the `# what's wrong:` line on `grammar --rule <name>` output uses error-framing vocabulary in an inspection context where nothing is wrong, and overclaims a specific failure cause when the validator only confidently knows "this line didn't conform to rule X"; (2) the flat `#`-prefix-comment plain-text format is hostile to humans scanning a terminal — no visual hierarchy, no color, no way for the eye to find the actionable expected-form line. This campaign produces an honest, lean diagnostic model plus a TTY-aware rendering layer that inherits the user's terminal palette via ANSI 16 semantic slots. Skill prose is intentionally untouched — the teaching continues to happen at the failure boundary in the validator binary.

## Reference Documents
- [R01 fc28ce0 commit](../../skills/managing-roadmaps/scripts/dirtree-rdm/src/rule.rs) — current `Rule` enum + `Diagnostic` struct shipped in the previous iteration; this campaign refactors it
- [R02 no-color.org](https://no-color.org/) — the de facto standard for `NO_COLOR` env-var opt-out
- [R03 termstandard/colors](https://github.com/termstandard/colors) — `COLORTERM` and TTY color-capability conventions

## Goal
Produce a lean `Diagnostic` model that doesn't overclaim failure cause, plus a TTY-aware render layer that gives humans scannable colored output while keeping piped/non-TTY output parse-stable for agents.

## Pre-conditions
- [x] commit `fc28ce0` (validator-diagnostics upgrade) is merged on this branch
- [x] `cargo test` passes (37 tests) on current `HEAD`
- [x] design decisions captured: drop `Diagnostic.what_phrasing`; add per-violation `hint: Option<&'static str>` fired only by the malformed-Consistency-Checks interception; render plain output minus the redundant `# what's wrong:` line; emit ANSI 16 semantic slots in TTY mode only; honor `NO_COLOR` / `CLICOLOR_FORCE` / `TERM=dumb`; do not read `LS_COLORS` (category error — our semantic categories aren't expressible in its grammar); defer any `DIRTREE_RDM_COLORS` env override until someone actually asks

## Success Gates
- ✅ `Diagnostic` struct exposes only `expected_form` (no `what_phrasing`, no `purpose`); the TDD-rationale prose currently on `step-field-consistency` is removed from `rule.rs`
- ✅ `Violation` carries `hint: Option<&'static str>`; only the malformed-Consistency-Checks interception sets one; generic regex-mismatch paths leave it `None`
- ✅ `grammar --rule <name>` and `grammar --search <pattern>` output no longer contains a `# what's wrong:` line; for self-explanatory rules the form alone is shown
- ✅ TTY-aware render layer respects `std::io::IsTerminal`, `NO_COLOR` (any value), `CLICOLOR_FORCE=1`, `TERM=dumb`, in that precedence order
- ✅ Color palette uses ANSI 16 only (no truecolor escapes), so user terminal palettes are inherited automatically; no new runtime dependency beyond `std`
- ✅ `grammar --search` with >1 match emits compact one-liner-per-match in TTY mode; `--rule` and single-match `--search` keep the full card
- ✅ All existing 37 tests pass; new tests cover env-var precedence, TTY-vs-non-TTY rendering branches, and the hint-only-when-justified discipline
- ✅ Piped stdout (non-TTY) produces clean parse-stable output that agents can rely on (no escape codes leak through)

## Status
```mermaid
graph TD
    refine-output-rendering[Refine validator and grammar CLI output rendering]:::done
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|
| `refine-output-rendering.md` | 📄 Leaf Task | ✅ Done |

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
| `refine-output-rendering.md` | --- | 5 | 4 step commits + 1 clippy fixup; 37 → 51 tests; smoke-tested piped/CLICOLOR_FORCE/originating-failure paths; one acceptable deviation in step 1 (touched main.rs callsites to keep build green when removing Diagnostic.what_phrasing) |
