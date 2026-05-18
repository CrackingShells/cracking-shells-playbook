//! Integration tests exercising representative validator failure modes
//! end-to-end through the CLI. Each fixture is invalid in exactly one way,
//! letting us assert on the rule-specific violation shape
//! (site-description line, optional hint, expected form, rule pointer).
//!
//! Per-rule coverage of *every* BNF rule is enforced at compile time by the
//! exhaustive `match` in `Rule::diagnostic`/`grammar_excerpt`, and at unit-test
//! level by `rule::tests::diagnostics_and_excerpts_are_non_empty`. The
//! fixtures here cover the distinct *categories* of failure path through the
//! validator (loose-match malformed line, header-prefix render, mermaid
//! sub-validator, expect_one_or_more, step-ordering, step-count constraint),
//! which is where regressions in the Display logic would actually surface.

use std::io::Write;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_dirtree-rdm")
}

/// Run `dirtree-rdm validate <tempfile>` with the given content and return
/// the (combined) output plus exit code. `name` controls the filename so
/// README.md vs leaf-task code paths can both be exercised. The default
/// run forces `NO_COLOR=1` so existing assertions stay ANSI-free.
fn validate(content: &str, name: &str) -> (String, i32) {
    validate_with_env(content, name, &[("NO_COLOR", "1")], &["CLICOLOR_FORCE"])
}

fn validate_with_env(
    content: &str,
    name: &str,
    set: &[(&str, &str)],
    unset: &[&str],
) -> (String, i32) {
    let dir = tempdir();
    let path = dir.join(name);
    {
        let mut f = std::fs::File::create(&path).expect("create fixture");
        f.write_all(content.as_bytes()).expect("write fixture");
    }
    let mut cmd = Command::new(bin());
    cmd.arg("validate").arg(&path);
    for key in unset {
        cmd.env_remove(key);
    }
    for (k, v) in set {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("spawn dirtree-rdm validate");
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    (combined, out.status.code().unwrap_or(-1))
}

const ESC: &str = "\x1b[";

fn tempdir() -> std::path::PathBuf {
    // Use a per-test subdirectory under the OS temp dir. Cleanup is best-effort
    // (validator runs are fast, leftover files are harmless).
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    p.push(format!("dirtree-rdm-test-{nanos}-{:?}", std::thread::current().id()));
    std::fs::create_dir_all(&p).expect("mkdir tempdir");
    p
}

/// Common assertions: every violation in `output` must include the
/// site-description line, the positive `expected form:` example, and the
/// `rule:` pointer. The `hint:` line is optional — only the
/// malformed-Consistency-Checks interception sets one.
fn assert_violation_shape(output: &str) {
    assert!(output.contains("FAIL"), "expected FAIL header in:\n{output}");
    assert!(output.contains("expected form:"), "missing expected-form line in:\n{output}");
    assert!(
        output.contains("dirtree-rdm grammar --rule"),
        "missing CLI rule pointer in:\n{output}"
    );
    // The decommissioned `what's wrong:` slot must not reappear.
    assert!(
        !output.contains("what's wrong"),
        "rendered output must no longer carry a `what's wrong` line; got:\n{output}"
    );
}

// ── 1. Trailing-content failure mode (the originating case) ────────────────

#[test]
fn validator_intercepts_trailing_content_on_consistency_line() {
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
    let (output, code) = validate(leaf, "trailing.md");
    assert_ne!(code, 0, "should exit non-zero");
    assert_violation_shape(&output);
    // The trailing-content phrasing now travels on a dedicated `hint:` line
    // (the only validator path permitted to set one).
    assert!(
        output.contains("hint: trailing content after `PASS)`/`FAIL)`"),
        "diagnostic must surface trailing-content phrasing on a `hint:` line; got:\n{output}"
    );
    assert!(
        output.contains("dirtree-rdm grammar --rule step-field-consistency"),
        "rule pointer must reference step-field-consistency; got:\n{output}"
    );
}

// ── 2. File-level missing field renders as `header:`, not `line 0:` ────────

#[test]
fn validator_uses_header_prefix_for_file_level_violations() {
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
    let (output, code) = validate(leaf, "missing-goal.md");
    assert_ne!(code, 0);
    assert_violation_shape(&output);
    assert!(
        output.contains("header: "),
        "file-level violation must use `header:` prefix; got:\n{output}"
    );
    assert!(
        !output.contains("line 0:"),
        "file-level violation must NOT use misleading `line 0:`; got:\n{output}"
    );
    assert!(
        output.contains("field-goal"),
        "missing-goal violation must reference field-goal rule; got:\n{output}"
    );
}

// ── 3. Expect_one_or_more path — reference-item missing R\d{2} prefix ──────

#[test]
fn validator_flags_reference_item_without_id_prefix() {
    let readme = "\
# Test Roadmap

## Context
Some context.

## Reference Documents
- [some-file](path/file.md) — missing the R-id prefix

## Goal
One-line goal.

## Pre-conditions
- [ ] precond

## Success Gates
- ⬜ gate

## Status
```mermaid
graph TD
    classDef done       fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
";
    let (output, code) = validate(readme, "README.md");
    assert_ne!(code, 0);
    assert_violation_shape(&output);
    assert!(
        output.contains("reference-item"),
        "violation must reference the reference-item rule; got:\n{output}"
    );
}

// ── 4. Mermaid sub-validator — classDef hex mismatch ───────────────────────

#[test]
fn validator_flags_mermaid_classdef_with_wrong_hex() {
    // Wrong fill hex on `classDef done` — should trip mermaid-classdef-done.
    let readme = "\
# Test Roadmap

## Context
Some context.

## Goal
One-line goal.

## Pre-conditions
- [ ] precond

## Success Gates
- ⬜ gate

## Status
```mermaid
graph TD
    classDef done       fill:#000000,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned    fill:#374151,color:#e5e7eb
    classDef amendment  fill:#1e3a5f,color:#bfdbfe
    classDef blocked    fill:#7f1d1d,color:#fecaca
```

## Nodes
| Node | Type | Status |
|:-----|:-----|:-------|

## Amendment Log
| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress
| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
";
    let (output, code) = validate(readme, "README.md");
    assert_ne!(code, 0);
    assert_violation_shape(&output);
    assert!(
        output.contains("mermaid-classdef-done"),
        "violation must reference mermaid-classdef-done; got:\n{output}"
    );
}

// ── 5. Step ordering — Step 3 where Step 1 expected ────────────────────────

#[test]
fn validator_flags_out_of_order_step() {
    let leaf = "\
# Test Leaf

**Goal**: do the thing
**Pre-conditions**:
- [ ] precond
**Success Gates**:
- ⬜ gate
**References**: R01

## Step 3: skipped ahead
**Goal**: implement
**Implementation Logic**:
add code
**Deliverables**: file.rs
**Consistency Checks**: `pytest` (expected: PASS)
**Commit**: `feat(core): add stub`
";
    let (output, code) = validate(leaf, "out-of-order.md");
    assert_ne!(code, 0);
    assert_violation_shape(&output);
    assert!(
        output.contains("step-heading"),
        "violation must reference step-heading rule; got:\n{output}"
    );
    assert!(
        output.contains("expected Step 1, got Step 3"),
        "site message must name expected/got step numbers; got:\n{output}"
    );
}

// ── 6. Step-count constraint — zero steps ──────────────────────────────────

#[test]
fn validator_flags_zero_steps() {
    let leaf = "\
# Test Leaf

**Goal**: do the thing
**Pre-conditions**:
- [ ] precond
**Success Gates**:
- ⬜ gate
**References**: R01
";
    let (output, code) = validate(leaf, "no-steps.md");
    assert_ne!(code, 0);
    assert_violation_shape(&output);
    assert!(
        output.contains("<step>"),
        "violation must reference the step rule; got:\n{output}"
    );
    assert!(
        output.contains("minimum 1 required"),
        "violation must explain minimum-1 constraint; got:\n{output}"
    );
}

// ── 7. Negative control — well-formed leaf must pass clean ─────────────────

#[test]
fn validator_accepts_well_formed_leaf_task() {
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
**Consistency Checks**: `pytest tests/test_thing.py` (expected: PASS)
**Commit**: `feat(core): add stub`
";
    let (output, code) = validate(leaf, "good.md");
    assert_eq!(code, 0, "well-formed leaf must validate clean; got:\n{output}");
    assert!(output.starts_with("OK"), "expected OK header; got:\n{output}");
}

// ── 8. TTY-aware color rendering on the validator path ────────────────────

const TRAILING_LEAF: &str = "\
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

#[test]
fn validator_emits_no_escapes_when_piped() {
    // `validate()` already forces NO_COLOR=1 — confirm zero ANSI escapes
    // reach the agent on the validator failure path.
    let (output, code) = validate(TRAILING_LEAF, "trailing.md");
    assert_ne!(code, 0);
    assert!(
        !output.contains(ESC),
        "piped validator output must not contain ANSI escapes; got:\n{output}"
    );
}

#[test]
fn validator_emits_escapes_under_force_color() {
    // CLICOLOR_FORCE=1 must produce at least one escape in the validator's
    // stderr output (the FAIL header, the rule pointer, etc. are wrapped).
    let (output, code) = validate_with_env(
        TRAILING_LEAF,
        "trailing.md",
        &[("CLICOLOR_FORCE", "1"), ("TERM", "xterm-256color")],
        &["NO_COLOR"],
    );
    assert_ne!(code, 0);
    assert!(
        output.contains(ESC),
        "forced-color validator output must contain ANSI escapes; got:\n{output}"
    );
    // Hint phrasing must still be present (escapes wrap, they don't replace).
    assert!(
        output.contains("trailing content after `PASS)`/`FAIL)`"),
        "forced-color output must still surface the trailing-content hint; got:\n{output}"
    );
}
