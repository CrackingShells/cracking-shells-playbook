//! Integration tests for `dirtree-rdm grammar` CLI modes.
//!
//! Exercises --list, --rule, --search (literal + regex), the token-overlap
//! fallback when literal search returns zero results, and the no-overlap
//! exit-code path. Also verifies backward-compat for the positional
//! `grammar readme` / `grammar leaf` dumps.

use std::process::Command;

/// Path to the binary built by `cargo test`.
fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_dirtree-rdm")
}

/// Run the binary with `NO_COLOR=1` in the env — emulates piped/non-TTY
/// agent-facing output regardless of the harness's actual stdout TTY-ness.
fn run(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(bin())
        .args(args)
        .env_remove("CLICOLOR_FORCE")
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to spawn dirtree-rdm");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// Run the binary with `CLICOLOR_FORCE=1` — emulates an interactive TTY
/// for tests that need to assert on ANSI escape presence.
fn run_force_color(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(bin())
        .args(args)
        .env_remove("NO_COLOR")
        .env("CLICOLOR_FORCE", "1")
        .env("TERM", "xterm-256color")
        .output()
        .expect("failed to spawn dirtree-rdm");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

const ESC: &str = "\x1b[";

#[test]
fn grammar_list_groups_rules_by_source() {
    let (stdout, _stderr, code) = run(&["grammar", "--list"]);
    assert_eq!(code, 0, "--list should exit 0");
    assert!(stdout.contains("# readme.bnf"), "missing readme group header in:\n{stdout}");
    assert!(stdout.contains("# leaf.bnf"), "missing leaf group header in:\n{stdout}");
    assert!(stdout.contains("# Shared"), "missing shared group header in:\n{stdout}");
    // Spot-check that some known rules appear in their expected groups.
    assert!(stdout.contains("step-field-consistency"), "step-field-consistency should appear in --list output");
    assert!(stdout.contains("mermaid-classdef-done"), "mermaid-classdef-done should appear in --list output");
    assert!(stdout.contains("h1-title"), "h1-title (shared) should appear in --list output");
}

#[test]
fn grammar_rule_known_name_prints_excerpt_and_expected_form() {
    let (stdout, _stderr, code) = run(&["grammar", "--rule", "step-field-consistency"]);
    assert_eq!(code, 0, "known --rule should exit 0; stdout was:\n{stdout}");
    assert!(stdout.contains("<step-field-consistency> ::="), "missing BNF excerpt in:\n{stdout}");
    assert!(stdout.contains("source: leaf.bnf"), "missing source annotation in:\n{stdout}");
    assert!(stdout.contains("expected form:"), "missing expected-form line in:\n{stdout}");
    // The decommissioned `# diagnostic` / `# what's wrong:` block must not
    // reappear; the grammar inspector now carries only the BNF excerpt and
    // the positive `expected form:` example.
    assert!(
        !stdout.contains("# diagnostic"),
        "grammar inspector must not emit a `# diagnostic` section; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("what's wrong"),
        "grammar inspector must not emit a `what's wrong` line; got:\n{stdout}"
    );
}

#[test]
fn grammar_rule_unknown_name_errors_with_hint() {
    let (_stdout, stderr, code) = run(&["grammar", "--rule", "does-not-exist"]);
    assert_ne!(code, 0, "unknown rule should exit non-zero");
    assert!(
        stderr.contains("--list"),
        "unknown-rule error must suggest --list; stderr was:\n{stderr}"
    );
}

#[test]
fn grammar_search_literal_substring_finds_consistency() {
    let (stdout, _stderr, code) = run(&["grammar", "--search", "consistency"]);
    assert_eq!(code, 0, "literal substring with match should exit 0");
    assert!(
        stdout.contains("<step-field-consistency>"),
        "literal substring 'consistency' should match step-field-consistency; got:\n{stdout}"
    );
}

#[test]
fn grammar_search_regex_anchored_step_prefix() {
    let (stdout, _stderr, code) = run(&["grammar", "--search", "^step-", "-e"]);
    assert_eq!(code, 0, "regex search with matches should exit 0");
    // Every rule whose name starts with "step-" should appear, and no rules
    // that don't start with "step-".
    assert!(stdout.contains("<step-heading>"), "regex ^step- should include step-heading; got:\n{stdout}");
    assert!(stdout.contains("<step-field-commit>"), "regex ^step- should include step-field-commit; got:\n{stdout}");
    assert!(!stdout.contains("<h1-title>"), "regex ^step- must not include h1-title; got:\n{stdout}");
}

#[test]
fn grammar_search_literal_with_regex_metacharacters_does_not_interpret_them() {
    // `\(expected:` contains a backslash-paren that, as regex, would be
    // literal `(expected:`. In substring mode the search text itself must
    // appear verbatim in the grammar body. The BNF for step-field-consistency
    // has `\(expected:` inside the regex literal, so this confirms substring
    // mode does NOT interpret regex metacharacters.
    let (stdout, _stderr, code) = run(&["grammar", "--search", r"\(expected:"]);
    assert_eq!(code, 0, "literal search with regex metacharacters should still find verbatim matches");
    assert!(
        stdout.contains("<step-field-consistency>"),
        r"literal '\(expected:' must find step-field-consistency by body match; got: {stdout}"
    );
}

#[test]
fn grammar_search_concept_query_falls_back_to_token_overlap() {
    // No rule body contains the literal substring "test pass fail expected",
    // so the literal pass returns zero matches and the fallback kicks in.
    let (stdout, _stderr, code) = run(&["grammar", "--search", "test pass fail expected"]);
    assert_eq!(code, 0, "fallback with matches should exit 0; output was:\n{stdout}");
    assert!(
        stdout.contains("no exact match for"),
        "fallback output must announce that no exact match was found; got:\n{stdout}"
    );
    assert!(
        stdout.contains("closest matches by term overlap"),
        "fallback output must label results as token-overlap matches; got:\n{stdout}"
    );
    assert!(
        stdout.contains("<step-field-consistency>"),
        "fallback for 'test pass fail expected' must surface step-field-consistency; got:\n{stdout}"
    );
}

#[test]
fn grammar_search_no_overlap_exits_non_zero_with_hint() {
    let (_stdout, stderr, code) = run(&["grammar", "--search", "xyzqzz"]);
    assert_ne!(code, 0, "no-overlap search should exit non-zero");
    assert!(
        stderr.contains("--list"),
        "no-match error must suggest --list; stderr was:\n{stderr}"
    );
}

#[test]
fn grammar_search_regex_no_match_exits_non_zero_without_fallback() {
    // In regex mode, the fallback is intentionally skipped — a regex that
    // doesn't match means "really nothing", and a token-overlap fallback
    // would be surprising. The error message should hint at --list.
    let (_stdout, stderr, code) = run(&["grammar", "--search", "^nothing-matches-this$", "-e"]);
    assert_ne!(code, 0, "regex-mode no-match should exit non-zero");
    assert!(
        stderr.contains("regex mode") || stderr.contains("--list"),
        "regex-mode no-match should hint at --list or note regex mode; stderr was:\n{stderr}"
    );
}

#[test]
fn grammar_backward_compat_positional_readme() {
    let (stdout, _stderr, code) = run(&["grammar", "readme"]);
    assert_eq!(code, 0, "positional `grammar readme` should still dump the readme BNF");
    assert!(stdout.contains("BNF Grammar: README.md"), "expected readme.bnf header in:\n{stdout}");
}

#[test]
fn grammar_backward_compat_positional_leaf() {
    let (stdout, _stderr, code) = run(&["grammar", "leaf"]);
    assert_eq!(code, 0, "positional `grammar leaf` should still dump the leaf BNF");
    assert!(stdout.contains("BNF Grammar: Leaf Task"), "expected leaf.bnf header in:\n{stdout}");
}

#[test]
fn grammar_no_mode_errors_with_hint() {
    let (_stdout, stderr, code) = run(&["grammar"]);
    assert_ne!(code, 0, "no mode + no positional should exit non-zero");
    assert!(
        stderr.contains("--list") || stderr.contains("--rule") || stderr.contains("--search"),
        "no-mode error must suggest the new flags; stderr was:\n{stderr}"
    );
}

#[test]
fn list_count_matches_rule_all_size() {
    let (stdout, _stderr, _code) = run(&["grammar", "--list"]);
    // Count non-empty, non-comment lines as rule entries.
    let rule_lines: usize = stdout
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .count();
    // We don't import Rule::ALL here (this is an integration test against the
    // binary), but the rule count is currently 35; assert it's at least 30 to
    // catch regressions where --list silently drops rules without removing
    // them from the enum.
    assert!(
        rule_lines >= 30,
        "expected at least 30 rule entries in --list output, got {rule_lines}; output:\n{stdout}"
    );
}

// ── TTY-aware color rendering ─────────────────────────────────────────────

#[test]
fn grammar_output_emits_no_escapes_when_piped() {
    // NO_COLOR is on by default in run(); every grammar mode must produce
    // ANSI-free output so agents can parse it.
    for args in [
        &["grammar", "--list"][..],
        &["grammar", "--rule", "step-field-consistency"][..],
        &["grammar", "--search", "consistency"][..],
    ] {
        let (stdout, _stderr, code) = run(args);
        assert_eq!(code, 0, "args {args:?} should exit 0");
        assert!(
            !stdout.contains(ESC),
            "piped output for {args:?} must not contain ANSI escapes; got:\n{stdout}"
        );
    }
}

#[test]
fn grammar_output_emits_escapes_under_force_color() {
    // With CLICOLOR_FORCE=1, every grammar mode must emit at least one
    // ANSI escape — the precedence rule forces color on even without a TTY.
    for args in [
        &["grammar", "--list"][..],
        &["grammar", "--rule", "step-field-consistency"][..],
        &["grammar", "--search", "consistency"][..],
    ] {
        let (stdout, _stderr, code) = run_force_color(args);
        assert_eq!(code, 0, "args {args:?} should exit 0");
        assert!(
            stdout.contains(ESC),
            "forced-color output for {args:?} must contain ANSI escapes; got:\n{stdout}"
        );
    }
}

#[test]
fn grammar_search_compact_layout_under_force_color() {
    // `^step-` matches many rules. Under forced color, output should switch
    // to the compact one-liner-per-rule layout: no `# rule:` heading line,
    // each rule occupies a single output line.
    let (stdout, _stderr, code) = run_force_color(&["grammar", "--search", "^step-", "-e"]);
    assert_eq!(code, 0, "regex search should exit 0; got:\n{stdout}");
    // Compact layout strips the "# rule:" / "# expected form:" comment
    // headings entirely.
    assert!(
        !stdout.contains("# rule:"),
        "compact layout must not emit `# rule:` headings; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("# expected form:"),
        "compact layout must not emit `# expected form:` headings; got:\n{stdout}"
    );
    // Every step-* rule occupies exactly one non-blank line.
    let step_lines: usize = stdout
        .lines()
        .filter(|l| l.contains("step-"))
        .count();
    assert!(
        step_lines >= 6,
        "expected one line per step-* rule (at least 6); got {step_lines} in:\n{stdout}"
    );
}

#[test]
fn grammar_search_single_match_keeps_full_card_under_force_color() {
    // A single match keeps the full-card layout even with color on, so the
    // user gets the BNF excerpt without an extra `--rule` round-trip.
    let (stdout, _stderr, code) = run_force_color(&["grammar", "--search", "step-field-consistency"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("# rule:"),
        "single-match search must keep the full card; got:\n{stdout}"
    );
    assert!(
        stdout.contains("# expected form:"),
        "single-match search must include the expected-form line; got:\n{stdout}"
    );
}

#[test]
fn grammar_search_piped_multi_match_keeps_full_card() {
    // Piped output (NO_COLOR=1) always uses the full card layout regardless
    // of match count — preserves agent-parsable structure.
    let (stdout, _stderr, code) = run(&["grammar", "--search", "^step-", "-e"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("# rule:"),
        "piped multi-match search must keep the full card; got:\n{stdout}"
    );
}
