/// Structural validator for README.md and leaf task .md files.
///
/// The grammar in grammar/readme.bnf and grammar/leaf.bnf is the canonical spec.
/// This module implements a section-order parser that enforces every rule
/// in those grammars and reports violations as three-layer diagnostics
/// (what's wrong / expected form / rule pointer) via the `Rule` enum.
use crate::rule::Rule;
use anyhow::{bail, Result};
use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Violation {
    /// 1-indexed line number, or 0 for file-level (header-region) issues.
    pub line: usize,
    pub rule: Rule,
    /// Short context tag (e.g. "Step 2", "got: \"## Steps\""). May be empty;
    /// adds *context* about which file location triggered the rule, not why
    /// the input didn't match.
    pub message: String,
    /// Validator-asserted cause narrowing. Populated only by sites whose own
    /// logic has narrowed beyond "regex didn't match" — currently just the
    /// malformed-Consistency-Checks interception. Every other construction
    /// site explicitly sets `None`. Discipline enforced by reading every
    /// `Violation { ... }` literal, not at runtime.
    pub hint: Option<&'static str>,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diag = self.rule.diagnostic();
        let prefix = if self.line == 0 {
            "header".to_string()
        } else {
            format!("line {}", self.line)
        };
        // Header line: "<prefix>: <message>" — the colon is always present
        // (it's the prefix/body separator); the message may be empty when no
        // site-specific context is available.
        writeln!(f, "{prefix}: {}", self.message)?;
        // Optional hint line.
        if let Some(h) = self.hint {
            writeln!(f, "         hint: {h}")?;
        }
        writeln!(f, "         expected form: {}", diag.expected_form)?;
        write!(
            f,
            "         rule: <{name}>  (run `dirtree-rdm grammar --rule {name}` for grammar excerpt)",
            name = self.rule.name(),
        )
    }
}

// ── helpers ────────────────────────────────────────────────────────────────

fn re(pat: &str) -> Regex {
    Regex::new(pat).expect("bad built-in regex")
}

/// Return the first non-blank line at or after `pos`, advancing `pos`.
fn skip_blank(lines: &[&str], pos: &mut usize) {
    while *pos < lines.len() && lines[*pos].trim().is_empty() {
        *pos += 1;
    }
}

fn expect_heading(
    lines: &[&str],
    pos: &mut usize,
    pattern: &Regex,
    rule: Rule,
) -> std::result::Result<(), Violation> {
    skip_blank(lines, pos);
    if *pos >= lines.len() {
        return Err(Violation {
            line: *pos + 1,
            rule,
            message: "got EOF".to_string(),
            hint: None,
        });
    }
    if !pattern.is_match(lines[*pos]) {
        return Err(Violation {
            line: *pos + 1,
            rule,
            message: format!("got: {:?}", lines[*pos]),
            hint: None,
        });
    }
    *pos += 1;
    Ok(())
}

fn expect_one_or_more(
    lines: &[&str],
    pos: &mut usize,
    pattern: &Regex,
    rule: Rule,
) -> std::result::Result<(), Violation> {
    skip_blank(lines, pos);
    let start = *pos;
    while *pos < lines.len() && !lines[*pos].trim().is_empty() && pattern.is_match(lines[*pos]) {
        *pos += 1;
    }
    if *pos == start {
        let got = if *pos < lines.len() {
            format!("{:?}", lines[*pos])
        } else {
            "EOF".to_string()
        };
        return Err(Violation {
            line: *pos + 1,
            rule,
            message: format!("got: {got}"),
            hint: None,
        });
    }
    Ok(())
}

/// Match: section heading (## Foo), then pipe header row, then separator row.
fn expect_table_section(
    lines: &[&str],
    pos: &mut usize,
    section_heading: &str,
    col_pattern: &Regex,
    header_rule: Rule,
) -> std::result::Result<(), Violation> {
    // 1. section heading
    let h2_re = Regex::new(&format!("^{}$", regex::escape(section_heading))).unwrap();
    expect_heading(lines, pos, &h2_re, header_rule)?;
    // 2. pipe header row
    skip_blank(lines, pos);
    if *pos >= lines.len() || !col_pattern.is_match(lines[*pos]) {
        return Err(Violation {
            line: *pos + 1,
            rule: header_rule,
            message: format!(
                "got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
            hint: None,
        });
    }
    *pos += 1;
    // 3. separator row
    let sep = re(r"^\|[-: |]+\|$");
    if *pos >= lines.len() || !sep.is_match(lines[*pos]) {
        return Err(Violation {
            line: *pos + 1,
            rule: Rule::TableSeparator,
            message: format!(
                "got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
            hint: None,
        });
    }
    *pos += 1;
    Ok(())
}

// ── README.md validator ────────────────────────────────────────────────────

pub fn validate_readme(path: &Path) -> Result<Vec<Violation>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;
    validate_readme_str(&content)
}

pub fn validate_readme_str(content: &str) -> Result<Vec<Violation>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut pos = 0usize;
    let mut violations: Vec<Violation> = Vec::new();

    macro_rules! check {
        ($expr:expr) => {
            if let Err(v) = $expr {
                violations.push(v);
                // skip to next heading to continue validation
                while pos < lines.len() && !lines[pos].starts_with('#') {
                    pos += 1;
                }
            }
        };
    }

    // <h1-title>
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^# .+"),
        Rule::H1Title
    ));

    // ## Context
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^## Context$"),
        Rule::H2Context
    ));
    check!(expect_one_or_more(
        &lines,
        &mut pos,
        &re(r"^.+"),
        Rule::ContextBody
    ));

    // ## Reference Documents (optional — skip if next heading differs)
    {
        let saved = pos;
        skip_blank(&lines, &mut pos);
        if pos < lines.len() && lines[pos] == "## Reference Documents" {
            pos += 1;
            let ref_item = re(r"^\- \[R\d{2} .+\]\(.+\) \u{2014} .+");
            check!(expect_one_or_more(
                &lines,
                &mut pos,
                &ref_item,
                Rule::ReferenceItem
            ));
        } else {
            pos = saved;
        }
    }

    // ## Goal
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^## Goal$"),
        Rule::H2Goal
    ));
    skip_blank(&lines, &mut pos);
    if pos >= lines.len() || lines[pos].trim().is_empty() {
        violations.push(Violation {
            line: pos + 1,
            rule: Rule::GoalBody,
            message: String::new(),
            hint: None,
        });
    } else {
        pos += 1;
    }

    // ## Pre-conditions
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^## Pre-conditions$"),
        Rule::H2Preconditions
    ));
    check!(expect_one_or_more(
        &lines,
        &mut pos,
        &re(r"^\- \[[ x]\] .+"),
        Rule::CheckboxItem
    ));

    // ## Success Gates
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^## Success Gates$"),
        Rule::H2SuccessGates
    ));
    check!(expect_one_or_more(
        &lines,
        &mut pos,
        &re(r"^\- [✅⬜] .+"),
        Rule::GateItem
    ));

    // ## Gotchas (optional)
    {
        let saved = pos;
        skip_blank(&lines, &mut pos);
        if pos < lines.len() && lines[pos] == "## Gotchas" {
            pos += 1;
            check!(expect_one_or_more(
                &lines,
                &mut pos,
                &re(r"^.+"),
                Rule::GotchasBody
            ));
        } else {
            pos = saved;
        }
    }

    // ## Status (Mermaid block)
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^## Status$"),
        Rule::H2Status
    ));
    check!(validate_mermaid_block(&lines, &mut pos));

    // ## Nodes table
    check!(expect_table_section(
        &lines,
        &mut pos,
        "## Nodes",
        &re(r"^\| Node\s*\| Type\s*\| Status\s*\|$"),
        Rule::TableHeaderNodes
    ));
    // Leaf rows end in .md; directory rows end in /  (enforced by BNF).
    let nodes_leaf_row = re(r"^\| `[a-z][a-z0-9_-]*\.md` \| 📄 Leaf Task \| (✅ Done|🔄 In Progress|⬜ Planned|🔵 Amendment|🚫 Blocked) \|$");
    let nodes_dir_row  = re(r"^\| `[a-z][a-z0-9_-]*/` \| 📁 Directory \| (✅ Done|🔄 In Progress|⬜ Planned|🔵 Amendment|🚫 Blocked) \|$");
    while pos < lines.len() && (nodes_leaf_row.is_match(lines[pos]) || nodes_dir_row.is_match(lines[pos])) {
        pos += 1;
    }
    // 0 rows is valid (empty directory at creation time)

    // ## Amendment Log
    check!(expect_table_section(
        &lines,
        &mut pos,
        "## Amendment Log",
        &re(r"^\| ID\s*\| Date\s*\| Source\s*\| Nodes Added\s*\| Rationale\s*\|$"),
        Rule::TableHeaderAmendment
    ));
    let amend_row = re(r"^\| A\d+ \| \d{4}-\d{2}-\d{2} \| .+ \| .+ \| .+ \|$");
    while pos < lines.len() && amend_row.is_match(lines[pos]) {
        pos += 1;
    }

    // ## Progress
    check!(expect_table_section(
        &lines,
        &mut pos,
        "## Progress",
        &re(r"^\| Node\s*\| Branch\s*\| Commits\s*\| Notes\s*\|$"),
        Rule::TableHeaderProgress
    ));
    let prog_row = re(r"^\| `[a-z][a-z0-9_./-]*` \| (task/[a-z0-9_-]+|--?-?) \| (\d+|--?-?) \| .* \|$");
    while pos < lines.len() && prog_row.is_match(lines[pos]) {
        pos += 1;
    }

    Ok(violations)
}

fn validate_mermaid_block(
    lines: &[&str],
    pos: &mut usize,
) -> std::result::Result<(), Violation> {
    skip_blank(lines, pos);
    if *pos >= lines.len() || lines[*pos] != "```mermaid" {
        return Err(Violation {
            line: *pos + 1,
            rule: Rule::MermaidBlock,
            message: format!(
                "got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
            hint: None,
        });
    }
    *pos += 1;

    // graph TD
    if *pos >= lines.len() || lines[*pos] != "graph TD" {
        return Err(Violation {
            line: *pos + 1,
            rule: Rule::MermaidGraphDecl,
            message: format!(
                "got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
            hint: None,
        });
    }
    *pos += 1;

    let node_decl = re(r"^    [a-z][a-z0-9_-]*\[.+\]:::(done|inprogress|planned|amendment|blocked)$");
    let classdef = re(r"^    classDef ");
    let edge = re(r"-->");

    // node declarations (zero or more) — no --> edges allowed
    while *pos < lines.len() && !classdef.is_match(lines[*pos]) && lines[*pos] != "```" {
        if edge.is_match(lines[*pos]) {
            return Err(Violation {
                line: *pos + 1,
                rule: Rule::MermaidNodeDecl,
                message: format!(
                    "sibling edges (-->) are forbidden; got: {:?}",
                    lines[*pos]
                ),
                hint: None,
            });
        }
        if !node_decl.is_match(lines[*pos]) && !lines[*pos].trim().is_empty() {
            return Err(Violation {
                line: *pos + 1,
                rule: Rule::MermaidNodeDecl,
                message: format!("got: {:?}", lines[*pos]),
                hint: None,
            });
        }
        *pos += 1;
    }

    // 5 required classDef lines (order enforced)
    let classdefs: [(Rule, &str); 5] = [
        (Rule::MermaidClassdefDone, r"^    classDef done +fill:#166534,color:#bbf7d0$"),
        (Rule::MermaidClassdefInprogress, r"^    classDef inprogress +fill:#854d0e,color:#fef08a$"),
        (Rule::MermaidClassdefPlanned, r"^    classDef planned +fill:#374151,color:#e5e7eb$"),
        (Rule::MermaidClassdefAmendment, r"^    classDef amendment +fill:#1e3a5f,color:#bfdbfe$"),
        (Rule::MermaidClassdefBlocked, r"^    classDef blocked +fill:#7f1d1d,color:#fecaca$"),
    ];
    for (rule, pat) in classdefs {
        let r = re(pat);
        if *pos >= lines.len() || !r.is_match(lines[*pos]) {
            return Err(Violation {
                line: *pos + 1,
                rule,
                message: format!(
                    "got: {:?}",
                    lines.get(*pos).unwrap_or(&"EOF")
                ),
                hint: None,
            });
        }
        *pos += 1;
    }

    // closing fence
    if *pos >= lines.len() || lines[*pos] != "```" {
        return Err(Violation {
            line: *pos + 1,
            rule: Rule::MermaidBlock,
            message: format!(
                "expected closing ``` fence, got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
            hint: None,
        });
    }
    *pos += 1;
    Ok(())
}

// ── Leaf task validator ────────────────────────────────────────────────────

pub fn validate_leaf(path: &Path) -> Result<Vec<Violation>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;
    validate_leaf_str(&content)
}

pub fn validate_leaf_str(content: &str) -> Result<Vec<Violation>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut pos = 0usize;
    let mut violations: Vec<Violation> = Vec::new();

    macro_rules! check {
        ($expr:expr) => {
            if let Err(v) = $expr {
                violations.push(v);
                while pos < lines.len() && !lines[pos].starts_with('#') && !lines[pos].starts_with("**") {
                    pos += 1;
                }
            }
        };
    }

    // <h1-title>
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^# .+"),
        Rule::H1Title
    ));

    // Task header fields (order-insensitive, all 4 required)
    let mut has_goal = false;
    let mut has_precond = false;
    let mut has_gates = false;
    let mut has_refs = false;

    let field_goal = re(r"^\*\*Goal\*\*: .+");
    let field_precond = re(r"^\*\*Pre-conditions\*\*:");
    let field_gates = re(r"^\*\*Success Gates\*\*:");
    let field_refs = re(r"^\*\*References\*\*: .+");
    let checkbox = re(r"^\- \[[ x]\] .+");
    let gate_item = re(r"^\- [✅⬜] .+");
    let step_heading = re(r"^## Step [1-5]: .+");

    skip_blank(&lines, &mut pos);

    while pos < lines.len() && !step_heading.is_match(lines[pos]) {
        if lines[pos].trim().is_empty() {
            pos += 1;
            continue;
        }
        if field_goal.is_match(lines[pos]) {
            has_goal = true;
            pos += 1;
        } else if field_precond.is_match(lines[pos]) {
            has_precond = true;
            pos += 1;
            while pos < lines.len() && checkbox.is_match(lines[pos]) {
                pos += 1;
            }
        } else if field_gates.is_match(lines[pos]) {
            has_gates = true;
            pos += 1;
            while pos < lines.len() && gate_item.is_match(lines[pos]) {
                pos += 1;
            }
        } else if field_refs.is_match(lines[pos]) {
            has_refs = true;
            pos += 1;
        } else {
            pos += 1; // unknown line in header area; tolerate
        }
    }

    if !has_goal {
        violations.push(Violation { line: 0, rule: Rule::FieldGoal, message: String::new(), hint: None });
    }
    if !has_precond {
        violations.push(Violation { line: 0, rule: Rule::FieldPreconditions, message: String::new(), hint: None });
    }
    if !has_gates {
        violations.push(Violation { line: 0, rule: Rule::FieldSuccessGates, message: String::new(), hint: None });
    }
    if !has_refs {
        violations.push(Violation { line: 0, rule: Rule::FieldReferences, message: String::new(), hint: None });
    }

    // Steps: 1-5, sequential numbering
    let commit_re = re(r"^\*\*Commit\*\*: `(feat|fix|test|docs|chore|refactor|style|perf|ci|build|revert)\([a-z][a-z0-9_-]*\): .+`$");
    let consistency_re = re(r"^\*\*Consistency Checks\*\*: .+\(expected: (PASS|FAIL)\)$");
    // Loose match that catches lines starting with the field but malformed
    // (most commonly: trailing content after `PASS)`/`FAIL)`, or wrong outcome word).
    // Used to distinguish "malformed line present" from "field entirely absent".
    let consistency_loose = re(r"^\*\*Consistency Checks\*\*:");
    let step_goal = re(r"^\*\*Goal\*\*: .+");
    let impl_logic = re(r"^\*\*Implementation Logic\*\*:");
    let deliverables = re(r"^\*\*Deliverables\*\*: .+");

    let mut step_count = 0usize;
    let mut expected_step = 1usize;

    while pos < lines.len() {
        skip_blank(&lines, &mut pos);
        if pos >= lines.len() { break; }
        if !step_heading.is_match(lines[pos]) { break; }

        // extract step number
        let step_num_re = re(r"^## Step (\d+): .+");
        let step_num = step_num_re
            .captures(lines[pos])
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);

        if step_num != expected_step {
            violations.push(Violation {
                line: pos + 1,
                rule: Rule::StepHeading,
                message: format!("expected Step {expected_step}, got Step {step_num}"),
                hint: None,
            });
        }
        // Line of this step's heading — used as the anchor for any
        // missing-field violations within the step.
        let step_heading_line = pos + 1;
        pos += 1;
        expected_step += 1;
        step_count += 1;

        // check required step fields
        let mut s_goal = false;
        let mut s_impl = false;
        let mut s_deliv = false;
        let mut s_consist = false;
        let mut s_commit = false;
        // Track if we saw a malformed **Consistency Checks**: line — emit
        // the more-specific rule violation in that case and treat the field
        // as "present but malformed" rather than "missing entirely".
        let mut consist_malformed = false;

        while pos < lines.len() && !step_heading.is_match(lines[pos]) {
            if lines[pos].trim().is_empty() { pos += 1; continue; }
            if step_goal.is_match(lines[pos]) { s_goal = true; pos += 1; }
            else if impl_logic.is_match(lines[pos]) {
                s_impl = true; pos += 1;
                // consume body lines
                while pos < lines.len() && !lines[pos].starts_with("**") && !step_heading.is_match(lines[pos]) {
                    pos += 1;
                }
            }
            else if deliverables.is_match(lines[pos]) { s_deliv = true; pos += 1; }
            else if consistency_re.is_match(lines[pos]) { s_consist = true; pos += 1; }
            else if consistency_loose.is_match(lines[pos]) {
                // Line starts with `**Consistency Checks**:` but full regex
                // doesn't match. The validator's own logic has narrowed the
                // cause from "regex mismatch" to "trailing content after
                // PASS)/FAIL)" — this is one of the (few) sites where a
                // site-specific hint is justified.
                violations.push(Violation {
                    line: pos + 1,
                    rule: Rule::StepFieldConsistency,
                    message: format!("Step {step_count}, got: {:?}", lines[pos]),
                    hint: Some("trailing content after `PASS)`/`FAIL)` is not permitted"),
                });
                s_consist = true;
                consist_malformed = true;
                pos += 1;
            }
            else if commit_re.is_match(lines[pos]) { s_commit = true; pos += 1; }
            else { pos += 1; }
        }

        let sn = step_count;
        // Use the step's heading line for missing-field violations rather
        // than `pos` (which now points past the step) — that line is the
        // anchor a reader can find in the file.
        if !s_goal    { violations.push(Violation { line: step_heading_line, rule: Rule::StepFieldGoal,         message: format!("Step {sn}"), hint: None }); }
        if !s_impl    { violations.push(Violation { line: step_heading_line, rule: Rule::StepFieldImplLogic,    message: format!("Step {sn}"), hint: None }); }
        if !s_deliv   { violations.push(Violation { line: step_heading_line, rule: Rule::StepFieldDeliverables, message: format!("Step {sn}"), hint: None }); }
        if !s_consist { violations.push(Violation { line: step_heading_line, rule: Rule::StepFieldConsistency,  message: format!("Step {sn}"), hint: None }); }
        if !s_commit  { violations.push(Violation { line: step_heading_line, rule: Rule::StepFieldCommit,       message: format!("Step {sn}"), hint: None }); }
        let _ = consist_malformed;
    }

    if step_count == 0 {
        violations.push(Violation {
            line: pos + 1,
            rule: Rule::Step,
            message: "leaf task has no steps (minimum 1 required)".to_string(),
            hint: None,
        });
    }
    if step_count > 5 {
        violations.push(Violation {
            line: pos + 1,
            rule: Rule::Step,
            message: format!("leaf task has {step_count} steps (maximum 5 allowed)"),
            hint: None,
        });
    }

    Ok(violations)
}

// ── public entry points ────────────────────────────────────────────────────

/// Validate a file, auto-detecting type by name.
pub fn validate_file(path: &Path) -> Result<Vec<Violation>> {
    match path.file_name().and_then(|n| n.to_str()) {
        Some("README.md") => validate_readme(path),
        Some(name) if name.ends_with(".md") => validate_leaf(path),
        _ => bail!("unsupported file type: {}", path.display()),
    }
}

/// Validate and print results; return true if clean.
///
/// Each violation may render across multiple lines (three-layer diagnostic);
/// every line is indented with two spaces so the output remains visually
/// grouped under the per-file `FAIL` header.
pub fn validate_and_report(path: &Path) -> Result<bool> {
    let violations = validate_file(path)?;
    if violations.is_empty() {
        println!("OK  {}", path.display());
        Ok(true)
    } else {
        eprintln!("FAIL  {}", path.display());
        for v in &violations {
            for line in format!("{v}").lines() {
                eprintln!("  {line}");
            }
        }
        Ok(false)
    }
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid README with an optional Reference Documents section.
    fn readme_with_refs(refs: &str) -> String {
        format!(
            "# Test Roadmap\n\n\
             ## Context\nSome context text here.\n\n\
             ## Reference Documents\n{refs}\n\n\
             ## Goal\nOne-line goal.\n\n\
             ## Pre-conditions\n- [ ] criteria met\n\n\
             ## Success Gates\n- \u{2705} gate\n\n\
             ## Status\n\
             ```mermaid\n\
             graph TD\n\
             \x20   classDef done       fill:#166534,color:#bbf7d0\n\
             \x20   classDef inprogress fill:#854d0e,color:#fef08a\n\
             \x20   classDef planned    fill:#374151,color:#e5e7eb\n\
             \x20   classDef amendment  fill:#1e3a5f,color:#bfdbfe\n\
             \x20   classDef blocked    fill:#7f1d1d,color:#fecaca\n\
             ```\n\n\
             ## Nodes\n\
             | Node | Type | Status |\n\
             |:-----|:-----|:-------|\n\n\
             ## Amendment Log\n\
             | ID | Date | Source | Nodes Added | Rationale |\n\
             |:---|:-----|:-------|:------------|:----------|\n\n\
             ## Progress\n\
             | Node | Branch | Commits | Notes |\n\
             |:-----|:-------|:--------|:------|\n"
        )
    }

    // ── Bug 1: reference-item separator must be em dash (—), not --- ──────

    #[test]
    fn test_reference_item_emdash_accepted() {
        let readme = readme_with_refs(
            "- [R01 Foo](../../some/path.md) \u{2014} description here",
        );
        let violations = validate_readme_str(&readme).unwrap();
        assert!(
            violations.is_empty(),
            "em-dash reference item with ../../ path should be valid; got: {violations:?}"
        );
    }

    #[test]
    fn test_reference_item_absolute_path_accepted() {
        let readme = readme_with_refs(
            "- [R01 sensor.md](/absolute/path/sensor.md) \u{2014} description",
        );
        let violations = validate_readme_str(&readme).unwrap();
        assert!(
            violations.is_empty(),
            "em-dash reference item with absolute path should be valid; got: {violations:?}"
        );
    }

    #[test]
    fn test_reference_item_triple_dash_rejected() {
        // Three ASCII hyphens must NOT be accepted — only em dash is valid per BNF.
        let readme = readme_with_refs(
            "- [R01 Foo](../../some/path.md) --- description here",
        );
        let violations = validate_readme_str(&readme).unwrap();
        assert!(
            !violations.is_empty(),
            "triple-dash separator should be rejected (BNF requires em dash)"
        );
    }

    #[test]
    fn test_reference_item_multiple_entries() {
        let readme = readme_with_refs(
            "- [R01 Foo](../../foo.md) \u{2014} first\n\
             - [R02 Bar](../bar/baz.md) \u{2014} second",
        );
        let violations = validate_readme_str(&readme).unwrap();
        assert!(
            violations.is_empty(),
            "multiple em-dash reference items should be valid; got: {violations:?}"
        );
    }

    // ── Originating failure: trailing content after PASS)/FAIL) ───────────

    #[test]
    fn test_consistency_check_trailing_comment_is_flagged() {
        let leaf = "\
# Test Leaf

**Goal**: do the thing
**Pre-conditions**:
- [ ] precond
**Success Gates**:
- ⬜ gate
**References**: R01

## Step 1: do it
**Goal**: implement
**Implementation Logic**:
add code
**Deliverables**: file.rs
**Consistency Checks**: pytest (expected: FAIL) because not implemented yet
**Commit**: `feat(core): add stub`
";
        let violations = validate_leaf_str(leaf).unwrap();
        // Must produce a step-field-consistency violation with the malformed
        // line context.
        let trailing = violations
            .iter()
            .find(|v| v.rule == Rule::StepFieldConsistency && !v.message.is_empty())
            .expect("expected a step-field-consistency violation for trailing content");
        let rendered = format!("{trailing}");
        assert!(
            rendered.contains("expected form: **Consistency Checks**"),
            "rendered violation must include the expected-form line; got:\n{rendered}"
        );
        assert!(
            rendered.contains("trailing content after `PASS)`/`FAIL)`"),
            "rendered violation must mention trailing-content phrasing; got:\n{rendered}"
        );
        assert!(
            rendered.contains("dirtree-rdm grammar --rule step-field-consistency"),
            "rendered violation must include the grammar CLI pointer; got:\n{rendered}"
        );
    }

    #[test]
    fn test_consistency_check_well_formed_pass() {
        let leaf = "\
# Test Leaf

**Goal**: do the thing
**Pre-conditions**:
- [ ] precond
**Success Gates**:
- ⬜ gate
**References**: R01

## Step 1: do it
**Goal**: implement
**Implementation Logic**:
add code
**Deliverables**: file.rs
**Consistency Checks**: `pytest` (expected: PASS)
**Commit**: `feat(core): add stub`
";
        let violations = validate_leaf_str(leaf).unwrap();
        assert!(
            !violations.iter().any(|v| v.rule == Rule::StepFieldConsistency),
            "well-formed consistency line should not produce a violation; got: {violations:?}"
        );
    }

    #[test]
    fn test_hint_only_set_by_consistency_interception() {
        // The malformed-Consistency-Checks branch is the ONE site permitted
        // to set `hint`. Every other violation must leave it `None`. This
        // test pins both halves: a malformed consistency line produces a
        // violation with `hint = Some(_)`, while every other violation in
        // the same fixture (and a separate generic-mismatch fixture) leaves
        // it `None`.

        // Fixture A: trailing content on Consistency Checks triggers the
        // interception path; every other rule passes; hint must be Some.
        let leaf_trailing = "\
# Test Leaf

**Goal**: do the thing
**Pre-conditions**:
- [ ] precond
**Success Gates**:
- ⬜ gate
**References**: R01

## Step 1: do it
**Goal**: implement
**Implementation Logic**:
add code
**Deliverables**: file.rs
**Consistency Checks**: pytest (expected: FAIL) because not implemented yet
**Commit**: `feat(core): add stub`
";
        let vs = validate_leaf_str(leaf_trailing).unwrap();
        let intercept = vs
            .iter()
            .find(|v| v.rule == Rule::StepFieldConsistency)
            .expect("expected an interception violation");
        assert!(
            intercept.hint.is_some(),
            "interception path must set hint; got hint=None"
        );
        assert!(
            intercept.hint.unwrap().contains("trailing content after `PASS)`/`FAIL)`"),
            "interception hint must name the trailing-content phrasing; got: {:?}",
            intercept.hint
        );

        // Fixture B: missing-fields leaf produces several generic-mismatch
        // violations; none of them may carry a hint.
        let leaf_missing = "\
# Test Leaf

**Pre-conditions**:
- [ ] precond
**Success Gates**:
- ⬜ gate
**References**: R01
";
        let vs = validate_leaf_str(leaf_missing).unwrap();
        assert!(
            !vs.is_empty(),
            "expected violations for missing-goal+no-steps fixture"
        );
        for v in &vs {
            assert!(
                v.hint.is_none(),
                "generic-mismatch violation must not carry a hint; got rule={:?} hint={:?}",
                v.rule,
                v.hint
            );
        }
    }

    #[test]
    fn test_header_violation_renders_as_header_prefix() {
        // Missing Goal field at file level — uses line 0 sentinel.
        let leaf = "\
# Test Leaf

**Pre-conditions**:
- [ ] precond
**Success Gates**:
- ⬜ gate
**References**: R01

## Step 1: do it
**Goal**: implement
**Implementation Logic**:
add code
**Deliverables**: file.rs
**Consistency Checks**: `pytest` (expected: PASS)
**Commit**: `feat(core): add stub`
";
        let violations = validate_leaf_str(leaf).unwrap();
        let missing_goal = violations
            .iter()
            .find(|v| v.rule == Rule::FieldGoal)
            .expect("expected a field-goal violation");
        let rendered = format!("{missing_goal}");
        assert!(
            rendered.starts_with("header:"),
            "file-level violation must render with `header:` prefix, not `line 0:`; got:\n{rendered}"
        );
    }
}
