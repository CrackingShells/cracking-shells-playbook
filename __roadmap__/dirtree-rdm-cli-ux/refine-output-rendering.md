# Refine validator and grammar CLI output rendering

**Goal**: Make `dirtree-rdm` output honest about cause and scannable for humans without breaking agent-facing plain output.
**Pre-conditions**:
- [x] previous commit `fc28ce0` (validator-diagnostics upgrade) is on HEAD
- [x] `cargo test` is green (37 tests) before starting
**Success Gates**:
- ⬜ `Diagnostic` has exactly one field (`expected_form`); no `what_phrasing`, no `purpose`, no TDD-rationale prose anywhere in `rule.rs` `[static]`
- ⬜ `Violation.hint` is populated only by the malformed-Consistency-Checks interception; every other violation construction site leaves it `None` `[static]`
- ⬜ Piped `grammar --rule step-field-consistency | cat` produces output with no ANSI escape codes and no `# what's wrong:` line `[behavioral]`
- ⬜ Interactive `dirtree-rdm grammar --rule step-field-consistency` in a TTY emits ANSI 16 escape codes (verify by capturing output with `script` or running under a TTY-emulating shell harness) `[behavioral]`
- ⬜ `NO_COLOR=1 dirtree-rdm grammar --rule step-field-consistency` in a TTY produces zero escape codes `[behavioral]`
- ⬜ `dirtree-rdm grammar --search '^step-' -e` in TTY mode renders one line per match (rule name + expected form) rather than full cards; same query piped renders the existing full-card plain format `[behavioral]`
- ⬜ `cargo test` passes; new tests cover (a) hint-only-on-interception, (b) env-var precedence for color decision, (c) compact-vs-card layout switch `[run]`
**References**: [R01 src/rule.rs](../../skills/managing-roadmaps/scripts/dirtree-rdm/src/rule.rs) — current Diagnostic + Rule shipped in fc28ce0

## Step 1: Slim the Diagnostic and add the Violation hint slot
**Goal**: Reshape the data model so the validator stops asserting *cause* and the per-violation `hint` slot is available for justified narrowing.
**Implementation Logic**:
The current `Diagnostic { what_phrasing, expected_form }` couples two different claims: the formal contract (what the rule requires) and a guess at what the user likely got wrong. The validator only ever knows the former with confidence. Drop `what_phrasing` entirely; the BNF excerpt + `expected_form` carry everything the validator can honestly assert about a rule. Remove the TDD-rationale prose currently attached to `step-field-consistency` — it's authoring guidance about a downstream workflow concern, not a property of the BNF rule, and it doesn't belong on the `Rule` enum.

Per-violation cause-narrowing then needs a dedicated slot: add `hint: Option<&'static str>` to `Violation`. The only validate-time path that sets it in this step is the malformed-Consistency-Checks interception (validate.rs around the `consistency_loose` regex branch), where the validator's own logic has narrowed the cause from "regex mismatch" to "trailing content after PASS)/FAIL)". Every other violation construction site explicitly passes `hint: None`. That discipline is enforced by reading every construction site in this commit, not by a runtime check.

This step does not touch any rendering code — the `Display` impl can compile against the new struct shape but its output may temporarily look odd. Step 2 fixes the rendering.
**Deliverables**: `src/rule.rs` — single-field `Diagnostic { expected_form }`, `Rule::diagnostic()` returns the slim form, TDD-rationale string removed from `Rule::StepFieldConsistency` arm; `src/validate.rs` — `Violation { line, rule, message, hint: Option<&'static str> }`, every `Violation { ... }` literal carries an explicit `hint:`, malformed-Consistency-Checks branch sets `hint: Some("trailing content after \`PASS)\`/\`FAIL)\` is not permitted")`, all other branches use `hint: None`; new test `validate::tests::test_hint_only_set_by_consistency_interception` asserting hint absence on generic regex mismatch vs. presence on the interception path
**Consistency Checks**: `cargo test --quiet -- --test-threads=1` (expected: PASS)
**Commit**: `refactor(dirtree-rdm): slim Diagnostic and add Violation hint slot`

## Step 2: Clean plain-output rendering
**Goal**: Make the plain (piped/non-TTY) output honest and lean — no `# what's wrong:` line anywhere; `hint:` line rendered only when present.
**Implementation Logic**:
Two rendering surfaces consume the data model: `Violation::Display` in `validate.rs` (validator errors) and `print_rule_summary` / `cmd_grammar_rule` / `cmd_grammar_search` in `main.rs` (grammar inspection). Both currently emit a `# what's wrong:` line built from `Diagnostic.what_phrasing`. With that field gone, both call sites must be updated.

For `Violation::Display`: render `<prefix>: <message>` (where `<prefix>` is `header:` or `line N:` and `<message>` is the existing site context); on a new line, render `   hint: <text>` only if `self.hint.is_some()`; then `   expected form: <expected_form>`; then `   rule: <name>  (run \`dirtree-rdm grammar --rule <name>\` for grammar excerpt)`. The hint line is conditionally present.

For `print_rule_summary` (used by `cmd_grammar_search`): emit `# rule: <name>`, BNF excerpt, `# expected form: <expected_form>`, blank line. No `# what's wrong:` line. No `hint:` (hints are validate-time only).

For `cmd_grammar_rule`: emit `# rule: <name>`, `# source: <file>`, blank line, BNF excerpt, blank line, `# expected form: <expected_form>`. Drop the entire `# diagnostic` section.

Update all integration and unit tests that previously asserted on `what's wrong:` phrasing — they should now assert on `hint:` presence/absence (for validate-time) or on `expected form:` alone (for grammar-time).
**Deliverables**: `src/validate.rs` — rewritten `impl Display for Violation` matching the new contract, `validate::tests::test_consistency_check_trailing_comment_is_flagged` updated to assert on the `hint:` line; `src/main.rs` — `print_rule_summary` and `cmd_grammar_rule` emit the cleaned plain format with no `# what's wrong:` and no `# diagnostic` section; `tests/integration_grammar.rs` — rename `grammar_rule_known_name_prints_excerpt_and_diagnostic` → `grammar_rule_known_name_prints_excerpt_and_expected_form`, remove `# diagnostic`/`what's wrong:` assertions, add negative assertion that output does NOT contain literal `what's wrong`; `tests/integration_validate.rs` — rename helper `assert_three_layer_shape` → `assert_violation_shape` (checks `expected form:` + `rule:`, `hint:` optional), `validator_intercepts_trailing_content_on_consistency_line` asserts the `hint:` line specifically
**Consistency Checks**: `cargo test --quiet` (expected: PASS)
**Commit**: `refactor(dirtree-rdm): drop what's-wrong line from plain output`

## Step 3: Add TTY-aware rendering infrastructure
**Goal**: Add a small color/render module that decides whether to emit ANSI escapes based on env vars + TTY detection, with semantic-slot helpers for the rest of the code to call.
**Implementation Logic**:
Introduce a new module `src/render.rs` owning the color decision and the semantic-slot styling. Three concerns lives there: (a) the env-precedence rule, (b) wrapping a string with an ANSI escape for a semantic role, (c) a public boolean `should_color(stream)` callable per output stream (since `validate` writes to stderr and `grammar` writes to stdout, and TTY-ness differs per stream).

Env-precedence (highest priority first):
1. `NO_COLOR` set to any value → no color, always
2. `CLICOLOR_FORCE=1` → color, always (overrides TTY check)
3. `TERM` unset or `TERM=dumb` → no color
4. `stream.is_terminal()` returns `true` → color
5. Otherwise → no color

Semantic slots emit raw ANSI 16 escape codes when color is active, no escape otherwise. Keep the palette small (5 slots: `rule_name`, `code`, `label`, `error`, `dim`) and pick stock ANSI colors that inherit terminal palette: bold cyan, green, bold yellow, bold red, dim. Avoid 24-bit truecolor entirely so terminal palettes are honored.

Public API:
```
pub fn should_color(stream: ColorStream) -> bool   // ColorStream = Stdout | Stderr
pub fn rule_name(s: &str, on: bool) -> String       // bold cyan
pub fn code(s: &str, on: bool) -> String            // green
pub fn label(s: &str, on: bool) -> String           // bold yellow
pub fn error(s: &str, on: bool) -> String           // bold red
pub fn dim(s: &str, on: bool) -> String             // dim
```

Tests cover the env-var precedence matrix using a helper that sets/unsets vars within a `std::env::set_var` scope. No actual TTY is required for the env-precedence tests; they set `CLICOLOR_FORCE=1` to force-enable or `NO_COLOR=1` to force-disable, asserting the boolean result. A separate test confirms that when color is `false`, the slot helpers pass strings through unchanged.

This step touches no rendering call sites yet — that's step 4.
**Deliverables**: `src/render.rs` (new) — `enum ColorStream { Stdout, Stderr }`, `fn should_color(stream: ColorStream) -> bool` consulting env + `IsTerminal` in documented precedence, styling helpers `rule_name`/`code`/`label`/`error`/`dim` returning escape-wrapped or passthrough strings; `src/main.rs` — `mod render;` declaration so the module is loaded; new tests in `render::tests` — `no_color_env_disables_color`, `clicolor_force_enables_color`, `term_dumb_disables_color`, `helpers_passthrough_when_off`, `helpers_emit_escape_when_on`
**Consistency Checks**: `cargo test --quiet render::` (expected: PASS)
**Commit**: `feat(dirtree-rdm): add TTY-aware color rendering module`

## Step 4: Apply rendering to validator + grammar outputs and add compact search layout
**Goal**: Wire the render module into all output paths; add compact one-liner-per-match layout for `--search` results in TTY mode.
**Implementation Logic**:
Two paths consume `render`:

**Validator output** (`validate_and_report` in `validate.rs`): compute `let on = render::should_color(ColorStream::Stderr)` once per file. Wrap the `FAIL <path>` header with `render::error`. In `Violation::Display`, wrap the line/header prefix with `render::error`, the `<rule-name>` placeholder in the rule pointer with `render::rule_name`, each label (`hint:`, `expected form:`, `rule:`) with `render::label`, the BNF code excerpts with `render::code`. The `Display` impl gains a `color_on: bool` parameter or is rewritten to take a render context — pick whichever keeps existing callers cleanest (probably: keep `Display` as-is for piping, add a separate `format_violation(v, on) -> String` used by `validate_and_report`).

**Grammar output** (`cmd_grammar_rule`, `cmd_grammar_search`, `cmd_grammar_list` in `main.rs`): compute `let on = render::should_color(ColorStream::Stdout)`. Apply the same role-to-slot mapping. `--list` headers (`# Shared`, `# readme.bnf`, `# leaf.bnf`) become `render::dim`.

**Compact search layout**: change `cmd_grammar_search` so that when `on == true` and the match count is >1, each match is rendered as a single line: `<rule-name>  <expected_form>` (rule name in `rule_name`, expected form in `code`, separator dim). Full card rendering remains for: `cmd_grammar_rule` (single rule, always), `cmd_grammar_search` with exactly 1 match (any mode), and `cmd_grammar_search` when `on == false` (piped — preserves agent-parsable output).

Tests must cover both modes. For non-TTY assertions, set `NO_COLOR=1` in the test process env before running the binary via `Command::new`. For TTY-mode assertions, set `CLICOLOR_FORCE=1` and assert the presence of an ANSI escape prefix (`\x1b[`) plus the absence of `# what's wrong:` in the output. The compact-layout switch is tested by a multi-match query (`--search "^step-" -e`) with `CLICOLOR_FORCE=1` asserting the match line is a single line per rule.
**Deliverables**: `src/validate.rs` — `format_violation(v: &Violation, on: bool) -> String` (or equivalent context-passing API), `validate_and_report` consults `render::should_color(ColorStream::Stderr)` once and reuses for all violations of the file; `src/main.rs` — `cmd_grammar_rule`/`cmd_grammar_search`/`cmd_grammar_list`/`print_rule_summary` consume `render::should_color(ColorStream::Stdout)`, `cmd_grammar_search` branches on `on && matches.len() > 1` to emit compact layout and falls back to full card otherwise; `tests/integration_grammar.rs` — new tests `grammar_search_compact_layout_under_force_color`, `grammar_output_emits_no_escapes_when_piped`, `grammar_output_emits_escapes_under_force_color`; `tests/integration_validate.rs` — new tests `validator_emits_no_escapes_when_piped` and `validator_emits_escapes_under_force_color`
**Consistency Checks**: `cargo test --quiet` (expected: PASS)
**Commit**: `feat(dirtree-rdm): render TTY output with ANSI semantic slots and compact search`
