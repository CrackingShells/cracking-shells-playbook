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

fn run(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to spawn dirtree-rdm");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

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
fn grammar_rule_known_name_prints_excerpt_and_diagnostic() {
    let (stdout, _stderr, code) = run(&["grammar", "--rule", "step-field-consistency"]);
    assert_eq!(code, 0, "known --rule should exit 0; stdout was:\n{stdout}");
    assert!(stdout.contains("<step-field-consistency> ::="), "missing BNF excerpt in:\n{stdout}");
    assert!(stdout.contains("source: leaf.bnf"), "missing source annotation in:\n{stdout}");
    assert!(stdout.contains("expected form:"), "missing expected-form line in:\n{stdout}");
    assert!(
        stdout.contains("trailing content after `PASS)`/`FAIL)`"),
        "step-field-consistency diagnostic must surface the trailing-content phrasing; got:\n{stdout}"
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
