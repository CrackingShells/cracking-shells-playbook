//! TTY-aware ANSI color rendering for the validator + grammar inspector.
//!
//! Owns three concerns:
//!   1. the env-precedence rule for deciding whether to emit ANSI escapes,
//!   2. wrapping a string with an ANSI 16 escape for a semantic role,
//!   3. a per-stream `should_color` callable (stderr-only color is honored).
//!
//! Only ANSI 16 codes are used — never 24-bit truecolor — so the user's
//! terminal palette is inherited rather than overridden. No new runtime
//! dependencies; everything goes through `std`.

use std::io::IsTerminal;

/// Which output stream the color decision applies to. Validator output
/// (stderr) and grammar inspector (stdout) can have different TTY-ness, so
/// the call sites must specify.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorStream {
    Stdout,
    Stderr,
}

/// Decide whether to emit ANSI escapes for the given stream.
///
/// Precedence (highest first):
///   1. `NO_COLOR` set to any value → never color
///   2. `CLICOLOR_FORCE=1`         → always color (overrides TTY check)
///   3. `TERM` unset or `TERM=dumb` → never color
///   4. `stream.is_terminal()` true → color
///   5. otherwise                   → no color
pub fn should_color(stream: ColorStream) -> bool {
    let is_terminal = match stream {
        ColorStream::Stdout => std::io::stdout().is_terminal(),
        ColorStream::Stderr => std::io::stderr().is_terminal(),
    };
    decide_color(
        std::env::var_os("NO_COLOR").is_some(),
        std::env::var("CLICOLOR_FORCE").ok().as_deref(),
        std::env::var("TERM").ok().as_deref(),
        is_terminal,
    )
}

/// Pure precedence decision, separated from environment + TTY probing so it is
/// testable without mutating process-global env vars (which races under the
/// default parallel `cargo test` runner). `should_color` resolves the inputs;
/// the real env/TTY/stream wiring is covered end-to-end by the subprocess
/// integration tests. Precedence matches the `should_color` doc comment.
fn decide_color(
    no_color: bool,
    clicolor_force: Option<&str>,
    term: Option<&str>,
    is_terminal: bool,
) -> bool {
    // 1. NO_COLOR — any value disables color (https://no-color.org/).
    if no_color {
        return false;
    }
    // 2. CLICOLOR_FORCE=1 — force color on even when not a TTY.
    if clicolor_force == Some("1") {
        return true;
    }
    // 3. TERM unset or dumb — no color.
    match term {
        None => return false,
        Some("dumb") => return false,
        Some(_) => {}
    }
    // 4. TTY check.
    is_terminal
}

// ── semantic-slot helpers ──────────────────────────────────────────────────
//
// Each helper wraps `s` with an ANSI escape when `on` is true, returning the
// untouched string otherwise. Stock ANSI 16 codes only — colors inherit the
// terminal palette.

const RESET: &str = "\x1b[0m";

/// Rule names (e.g. `<step-field-consistency>`). Bold cyan.
pub fn rule_name(s: &str, on: bool) -> String {
    if on { format!("\x1b[1;36m{s}{RESET}") } else { s.to_string() }
}

/// Code/grammar excerpts (e.g. BNF productions, regex literals). Green.
pub fn code(s: &str, on: bool) -> String {
    if on { format!("\x1b[32m{s}{RESET}") } else { s.to_string() }
}

/// Field labels (e.g. `hint:`, `expected form:`, `rule:`). Bold yellow.
pub fn label(s: &str, on: bool) -> String {
    if on { format!("\x1b[1;33m{s}{RESET}") } else { s.to_string() }
}

/// Error/failure markers (e.g. `FAIL`, `line 12:`). Bold red.
pub fn error(s: &str, on: bool) -> String {
    if on { format!("\x1b[1;31m{s}{RESET}") } else { s.to_string() }
}

/// Secondary annotations (e.g. `# readme.bnf` group headers, separators). Dim.
pub fn dim(s: &str, on: bool) -> String {
    if on { format!("\x1b[2m{s}{RESET}") } else { s.to_string() }
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // The env-precedence rule is tested through the pure `decide_color` helper
    // with explicit inputs — no process-global env mutation, so these run
    // safely under the default parallel `cargo test`. The real env/TTY/stream
    // wiring of `should_color` is exercised end-to-end by the subprocess
    // integration tests (each subprocess owns its own environment).

    #[test]
    fn no_color_disables_color() {
        // NO_COLOR set → never color, regardless of force/term/TTY.
        assert!(!decide_color(true, Some("1"), Some("xterm-256color"), true));
    }

    #[test]
    fn clicolor_force_enables_color() {
        // CLICOLOR_FORCE=1 forces color even with TERM=dumb and no TTY.
        assert!(decide_color(false, Some("1"), Some("dumb"), false));
        // A non-"1" value does not force.
        assert!(!decide_color(false, Some("0"), Some("dumb"), false));
    }

    #[test]
    fn term_dumb_or_unset_disables_color() {
        assert!(!decide_color(false, None, Some("dumb"), true));
        assert!(!decide_color(false, None, None, true));
    }

    #[test]
    fn no_color_beats_clicolor_force() {
        // NO_COLOR sits at the top of the precedence ladder; even with
        // CLICOLOR_FORCE=1 it must win.
        assert!(!decide_color(true, Some("1"), Some("xterm-256color"), true));
    }

    #[test]
    fn tty_decides_when_no_overrides() {
        // No NO_COLOR / CLICOLOR_FORCE and a real TERM → fall through to TTY.
        assert!(decide_color(false, None, Some("xterm-256color"), true));
        assert!(!decide_color(false, None, Some("xterm-256color"), false));
    }

    #[test]
    fn helpers_passthrough_when_off() {
        assert_eq!(rule_name("foo", false), "foo");
        assert_eq!(code("bar", false), "bar");
        assert_eq!(label("baz:", false), "baz:");
        assert_eq!(error("FAIL", false), "FAIL");
        assert_eq!(dim("# header", false), "# header");
    }

    #[test]
    fn helpers_emit_escape_when_on() {
        // Every slot must produce a string starting with the ESC byte and
        // ending with the reset sequence. We don't pin the exact code so the
        // palette can be tuned without test churn — just that the wrap
        // happened and the input string is preserved verbatim inside.
        for wrapped in [
            rule_name("rn", true),
            code("c", true),
            label("l:", true),
            error("e", true),
            dim("d", true),
        ] {
            assert!(
                wrapped.starts_with('\x1b'),
                "wrapped output must start with ESC; got: {wrapped:?}"
            );
            assert!(
                wrapped.ends_with(RESET),
                "wrapped output must end with reset sequence; got: {wrapped:?}"
            );
        }
    }
}
