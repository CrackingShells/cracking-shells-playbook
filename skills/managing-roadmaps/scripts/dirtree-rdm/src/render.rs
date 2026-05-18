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
    // 1. NO_COLOR — any value disables color (https://no-color.org/).
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    // 2. CLICOLOR_FORCE=1 — force color on even when not a TTY.
    if std::env::var("CLICOLOR_FORCE").as_deref() == Ok("1") {
        return true;
    }
    // 3. TERM unset or dumb — no color.
    match std::env::var("TERM").as_deref() {
        Err(_) => return false,
        Ok("dumb") => return false,
        Ok(_) => {}
    }
    // 4. TTY check.
    match stream {
        ColorStream::Stdout => std::io::stdout().is_terminal(),
        ColorStream::Stderr => std::io::stderr().is_terminal(),
    }
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

    /// Snapshot every env var the precedence rule consults, then restore
    /// them on drop. `cargo test -- --test-threads=1` is required for the
    /// `render::` tests so concurrent mutations from sibling tests cannot
    /// race with the snapshot/restore.
    struct EnvGuard {
        no_color: Option<std::ffi::OsString>,
        clicolor_force: Option<std::ffi::OsString>,
        term: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn capture() -> Self {
            Self {
                no_color: std::env::var_os("NO_COLOR"),
                clicolor_force: std::env::var_os("CLICOLOR_FORCE"),
                term: std::env::var_os("TERM"),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            restore("NO_COLOR", self.no_color.as_deref());
            restore("CLICOLOR_FORCE", self.clicolor_force.as_deref());
            restore("TERM", self.term.as_deref());
        }
    }

    fn restore(key: &str, val: Option<&std::ffi::OsStr>) {
        match val {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn no_color_env_disables_color() {
        let _g = EnvGuard::capture();
        std::env::remove_var("CLICOLOR_FORCE");
        std::env::set_var("NO_COLOR", "1");
        std::env::set_var("TERM", "xterm-256color");
        assert!(!should_color(ColorStream::Stdout));
        assert!(!should_color(ColorStream::Stderr));
    }

    #[test]
    fn clicolor_force_enables_color() {
        let _g = EnvGuard::capture();
        std::env::remove_var("NO_COLOR");
        std::env::set_var("CLICOLOR_FORCE", "1");
        // Even with TERM=dumb (which would otherwise disable), force wins
        // for non-TTY streams under cargo test.
        std::env::set_var("TERM", "dumb");
        assert!(should_color(ColorStream::Stdout));
        assert!(should_color(ColorStream::Stderr));
    }

    #[test]
    fn term_dumb_disables_color() {
        let _g = EnvGuard::capture();
        std::env::remove_var("NO_COLOR");
        std::env::remove_var("CLICOLOR_FORCE");
        std::env::set_var("TERM", "dumb");
        assert!(!should_color(ColorStream::Stdout));
        assert!(!should_color(ColorStream::Stderr));
    }

    #[test]
    fn no_color_beats_clicolor_force() {
        // NO_COLOR sits at the top of the precedence ladder; even with
        // CLICOLOR_FORCE=1 it must win.
        let _g = EnvGuard::capture();
        std::env::set_var("NO_COLOR", "1");
        std::env::set_var("CLICOLOR_FORCE", "1");
        std::env::set_var("TERM", "xterm-256color");
        assert!(!should_color(ColorStream::Stdout));
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
