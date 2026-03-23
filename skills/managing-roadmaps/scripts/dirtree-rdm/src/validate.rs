/// Structural validator for README.md and leaf task .md files.
///
/// The grammar in grammar/readme.bnf and grammar/leaf.bnf is the canonical spec.
/// This module implements a section-order parser that enforces every production
/// in those grammars, reporting violations with file path + line number +
/// expected production name.
use anyhow::{bail, Result};
use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Violation {
    pub line: usize,
    pub production: &'static str,
    pub message: String,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line {}: [{}] {}",
            self.line, self.production, self.message
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
    production: &'static str,
) -> std::result::Result<(), Violation> {
    skip_blank(lines, pos);
    if *pos >= lines.len() {
        return Err(Violation {
            line: *pos + 1,
            production,
            message: format!("expected heading matching <{production}>, got EOF"),
        });
    }
    if !pattern.is_match(lines[*pos]) {
        return Err(Violation {
            line: *pos + 1,
            production,
            message: format!(
                "expected heading matching <{production}>, got: {:?}",
                lines[*pos]
            ),
        });
    }
    *pos += 1;
    Ok(())
}

fn expect_one_or_more(
    lines: &[&str],
    pos: &mut usize,
    pattern: &Regex,
    production: &'static str,
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
            production,
            message: format!("expected one or more <{production}>, got: {got}"),
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
    production: &'static str,
) -> std::result::Result<(), Violation> {
    // 1. section heading
    let h2_re = Regex::new(&format!("^{}$", regex::escape(section_heading))).unwrap();
    expect_heading(lines, pos, &h2_re, production)?;
    // 2. pipe header row
    skip_blank(lines, pos);
    if *pos >= lines.len() || !col_pattern.is_match(lines[*pos]) {
        return Err(Violation {
            line: *pos + 1,
            production,
            message: format!(
                "expected table column header row, got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
        });
    }
    *pos += 1;
    // 3. separator row
    let sep = re(r"^\|[-: |]+\|$");
    if *pos >= lines.len() || !sep.is_match(lines[*pos]) {
        return Err(Violation {
            line: *pos + 1,
            production: "table-separator",
            message: format!(
                "expected table separator row after column header, got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
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
        "h1-title"
    ));

    // ## Context
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^## Context$"),
        "h2-context"
    ));
    check!(expect_one_or_more(
        &lines,
        &mut pos,
        &re(r"^.+"),
        "context-body"
    ));

    // ## Reference Documents (optional — skip if next heading differs)
    {
        let saved = pos;
        skip_blank(&lines, &mut pos);
        if pos < lines.len() && lines[pos] == "## Reference Documents" {
            pos += 1;
            let ref_item = re(r"^\- \[R\d{2} .+\]\(.+\) --- .+");
            check!(expect_one_or_more(
                &lines,
                &mut pos,
                &ref_item,
                "reference-item"
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
        "h2-goal"
    ));
    skip_blank(&lines, &mut pos);
    if pos >= lines.len() || lines[pos].trim().is_empty() {
        violations.push(Violation {
            line: pos + 1,
            production: "goal-body",
            message: "expected one-line goal body".to_string(),
        });
    } else {
        pos += 1;
    }

    // ## Pre-conditions
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^## Pre-conditions$"),
        "h2-preconditions"
    ));
    check!(expect_one_or_more(
        &lines,
        &mut pos,
        &re(r"^\- \[[ x]\] .+"),
        "checkbox-item"
    ));

    // ## Success Gates
    check!(expect_heading(
        &lines,
        &mut pos,
        &re(r"^## Success Gates$"),
        "h2-success-gates"
    ));
    check!(expect_one_or_more(
        &lines,
        &mut pos,
        &re(r"^\- [✅⬜] .+"),
        "gate-item"
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
                "gotchas-body"
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
        "h2-status"
    ));
    check!(validate_mermaid_block(&lines, &mut pos));

    // ## Nodes table
    check!(expect_table_section(
        &lines,
        &mut pos,
        "## Nodes",
        &re(r"^\| Node\s*\| Type\s*\| Status\s*\|$"),
        "table-header-nodes"
    ));
    let nodes_row = re(r"^\| `[a-z][a-z0-9_./-]*` \| (📄 Leaf Task|📁 Directory) \| (✅ Done|🔄 In Progress|⬜ Planned|🔵 Amendment|🚫 Blocked) \|$");
    while pos < lines.len() && nodes_row.is_match(lines[pos]) {
        pos += 1;
    }
    // 0 rows is valid (empty directory at creation time)

    // ## Amendment Log
    check!(expect_table_section(
        &lines,
        &mut pos,
        "## Amendment Log",
        &re(r"^\| ID\s*\| Date\s*\| Source\s*\| Nodes Added\s*\| Rationale\s*\|$"),
        "table-header-amendment"
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
        "table-header-progress"
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
            production: "mermaid-block",
            message: format!(
                "expected ```mermaid fence, got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
        });
    }
    *pos += 1;

    // graph TD
    if *pos >= lines.len() || lines[*pos] != "graph TD" {
        return Err(Violation {
            line: *pos + 1,
            production: "mermaid-graph-decl",
            message: format!(
                "expected 'graph TD', got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
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
                production: "mermaid-node-decl",
                message: format!(
                    "sibling edges (-->) are forbidden; ordering comes from tree depth, not edges. Line: {:?}",
                    lines[*pos]
                ),
            });
        }
        if !node_decl.is_match(lines[*pos]) && !lines[*pos].trim().is_empty() {
            return Err(Violation {
                line: *pos + 1,
                production: "mermaid-node-decl",
                message: format!(
                    "invalid node declaration (expected `    id[Title]:::status`): {:?}",
                    lines[*pos]
                ),
            });
        }
        *pos += 1;
    }

    // 5 required classDef lines (order enforced)
    let classdefs = [
        ("classDef done", "mermaid-classdef-done", r"^    classDef done +fill:#166534,color:#bbf7d0$"),
        ("classDef inprogress", "mermaid-classdef-inprogress", r"^    classDef inprogress +fill:#854d0e,color:#fef08a$"),
        ("classDef planned", "mermaid-classdef-planned", r"^    classDef planned +fill:#374151,color:#e5e7eb$"),
        ("classDef amendment", "mermaid-classdef-amendment", r"^    classDef amendment +fill:#1e3a5f,color:#bfdbfe$"),
        ("classDef blocked", "mermaid-classdef-blocked", r"^    classDef blocked +fill:#7f1d1d,color:#fecaca$"),
    ];
    for (name, production, pat) in classdefs {
        let r = re(pat);
        if *pos >= lines.len() || !r.is_match(lines[*pos]) {
            return Err(Violation {
                line: *pos + 1,
                production,
                message: format!(
                    "expected `{name}` classDef line, got: {:?}",
                    lines.get(*pos).unwrap_or(&"EOF")
                ),
            });
        }
        *pos += 1;
    }

    // closing fence
    if *pos >= lines.len() || lines[*pos] != "```" {
        return Err(Violation {
            line: *pos + 1,
            production: "mermaid-block",
            message: format!(
                "expected closing ``` fence, got: {:?}",
                lines.get(*pos).unwrap_or(&"EOF")
            ),
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
        "h1-title"
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
        violations.push(Violation { line: 0, production: "field-goal", message: "missing **Goal**: field in task header".to_string() });
    }
    if !has_precond {
        violations.push(Violation { line: 0, production: "field-preconditions", message: "missing **Pre-conditions**: field in task header".to_string() });
    }
    if !has_gates {
        violations.push(Violation { line: 0, production: "field-success-gates", message: "missing **Success Gates**: field in task header".to_string() });
    }
    if !has_refs {
        violations.push(Violation { line: 0, production: "field-references", message: "missing **References**: field in task header".to_string() });
    }

    // Steps: 1-5, sequential numbering
    let commit_re = re(r"^\*\*Commit\*\*: `(feat|fix|test|docs|chore|refactor|style|perf|ci|build|revert)\([a-z][a-z0-9_-]*\): .+`$");
    let consistency_re = re(r"^\*\*Consistency Checks\*\*: .+\(expected: (PASS|FAIL)\)$");
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
                production: "step-heading",
                message: format!("expected Step {expected_step}, got Step {step_num}"),
            });
        }
        pos += 1;
        expected_step += 1;
        step_count += 1;

        // check required step fields
        let mut s_goal = false;
        let mut s_impl = false;
        let mut s_deliv = false;
        let mut s_consist = false;
        let mut s_commit = false;

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
            else if commit_re.is_match(lines[pos]) { s_commit = true; pos += 1; }
            else { pos += 1; }
        }

        let sn = step_count;
        if !s_goal    { violations.push(Violation { line: pos+1, production: "step-field-goal",        message: format!("Step {sn} missing **Goal**") }); }
        if !s_impl    { violations.push(Violation { line: pos+1, production: "step-field-impl-logic",  message: format!("Step {sn} missing **Implementation Logic**") }); }
        if !s_deliv   { violations.push(Violation { line: pos+1, production: "step-field-deliverables",message: format!("Step {sn} missing **Deliverables**") }); }
        if !s_consist { violations.push(Violation { line: pos+1, production: "step-field-consistency", message: format!("Step {sn} missing **Consistency Checks**") }); }
        if !s_commit  { violations.push(Violation { line: pos+1, production: "step-field-commit",      message: format!("Step {sn} missing **Commit**") }); }
    }

    if step_count == 0 {
        violations.push(Violation {
            line: pos + 1,
            production: "step",
            message: "leaf task has no steps (minimum 1 required)".to_string(),
        });
    }
    if step_count > 5 {
        violations.push(Violation {
            line: pos + 1,
            production: "step",
            message: format!("leaf task has {step_count} steps (maximum 5 allowed)"),
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
pub fn validate_and_report(path: &Path) -> Result<bool> {
    let violations = validate_file(path)?;
    if violations.is_empty() {
        println!("OK  {}", path.display());
        Ok(true)
    } else {
        eprintln!("FAIL  {}", path.display());
        for v in &violations {
            eprintln!("  {v}");
        }
        Ok(false)
    }
}
