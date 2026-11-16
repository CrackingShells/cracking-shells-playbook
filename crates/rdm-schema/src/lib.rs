//! Stored types, read-model types and version constants of the rdm roadmap format.
//!
//! This crate holds types only. It has no I/O and no store. Canonical serialisation
//! and the store belong to M1.

pub mod read;
pub mod stored;

/// A `(major, minor)` pair, comparable field by field.
pub type Version = (u16, u16);

/// Format version of the stored files (`schema_version`), `"0.1"`.
pub const FORMAT_VERSION: Version = (0, 1);

/// Version of the read model (`read_model_version`), `"0.1"`. Separate from the format.
pub const READ_MODEL_VERSION: Version = (0, 1);

/// Renders a version as the `"MAJOR.MINOR"` string stored in files and payloads.
#[must_use]
pub fn version_string(v: Version) -> String {
    format!("{}.{}", v.0, v.1)
}

/// Parses `"MAJOR.MINOR"`. Returns `None` for any other form.
#[must_use]
pub fn parse_version(s: &str) -> Option<Version> {
    let (major, minor) = s.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// Whether a reader that supports `supported` may read data stamped `found`.
///
/// A higher major is refused. While the supported major is 0, a higher minor is refused
/// too. From major 1, a higher minor is read. Lower versions are read.
#[must_use]
pub fn is_readable(found: Version, supported: Version) -> bool {
    if found.0 > supported.0 {
        return false;
    }
    if found.0 == supported.0 && supported.0 == 0 && found.1 > supported.1 {
        return false;
    }
    true
}
