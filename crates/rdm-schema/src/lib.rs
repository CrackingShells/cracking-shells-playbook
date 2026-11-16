//! Stored types, read-model types and version constants of the rdm roadmap format.
//!
//! This crate holds types only. It has no I/O and no store. Canonical serialisation
//! and the store belong to M1.

pub mod read;
pub mod stored;

/// A `(major, minor, patch)` triple, `0.X.Y` while the major is 0, comparable field by field.
pub type Version = (u16, u16, u16);

/// Format version of the stored files (`schema_version`), `"0.1.0"`.
pub const FORMAT_VERSION: Version = (0, 1, 0);

/// Version of the read model (`read_model_version`), `"0.1.0"`. Separate from the format.
pub const READ_MODEL_VERSION: Version = (0, 1, 0);

/// Renders a version as the `"N.N.N"` string stored in files and payloads.
#[must_use]
pub fn version_string(v: Version) -> String {
    format!("{}.{}.{}", v.0, v.1, v.2)
}

/// Parses `"N.N.N"`. Strict: exactly three parts, ASCII digits only, no sign, no whitespace, and
/// no leading zero except `0` itself. Returns `None` for any other form.
#[must_use]
pub fn parse_version(s: &str) -> Option<Version> {
    fn part(p: &str) -> Option<u16> {
        let ok = !p.is_empty()
            && p.bytes().all(|b| b.is_ascii_digit())
            && (p == "0" || !p.starts_with('0'));
        if ok { p.parse().ok() } else { None }
    }
    let (major, rest) = s.split_once('.')?;
    let (minor, patch) = rest.split_once('.')?;
    Some((part(major)?, part(minor)?, part(patch)?))
}

/// Whether a reader that supports `supported` may read data stamped `found`.
///
/// A different major is refused. While the supported major is 0, a higher X (the minor) is
/// refused too. A higher Y (the patch) is read, and so is anything lower.
#[must_use]
pub fn is_readable(found: Version, supported: Version) -> bool {
    if found.0 != supported.0 {
        return false;
    }
    if supported.0 == 0 && found.1 > supported.1 {
        return false;
    }
    true
}
