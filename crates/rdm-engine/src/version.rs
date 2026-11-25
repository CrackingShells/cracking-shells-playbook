//! The format version chain (`contracts.md`, `Version chain`).
//!
//! Each format line is a layer that overrides only what changed; a behaviour a layer does not
//! define falls through to its `base`, and the root of the chain, [`Nothing`], defines none. A
//! layer reads the files of its own version into the current types, so the rest of the code knows
//! one model. There is one encoder, in [`crate::codec`], and no layer writes.
//!
//! Reading order, for every file: the stamp `schema_version` is read from the parsed value
//! before anything else of it is looked at ([`read_version`]); a file newer than the binary,
//! whichever part of the triple is higher, is refused ([`layer_of`]); an older one goes to the
//! newest layer whose line it has reached, or is `unknown_format`.

use rdm_schema::campaign::Decoded;
use rdm_schema::error::{Behaviour, NotInVersion, Rule, StoreError};
use rdm_schema::stored::{Amendment, Comment, Node};
use rdm_schema::{FORMAT_VERSION, Version, is_newer, parse_version, version_string};
use serde_json::Value;

use crate::codec::{current_amendment, current_comment, current_node};

/// The root of the chain: every behaviour is `NotInVersion`.
pub struct Nothing;

/// One format line. A layer implements only the behaviours that changed; the others are its
/// base's. A layer that removes a behaviour overrides it back to `Err(NotInVersion)`, which stops
/// the lookup before it reaches an older layer that still has it.
///
/// A decode returns `Result<Decoded<_>, StoreError>` inside the [`Behaviour`]: the outer error
/// says the layer has no such behaviour, the inner one that the file is faulty.
pub trait FormatLayer: Sync {
    /// The format line this layer reads: (major, minor). A patch bump (Y) has no layer.
    fn line(&self) -> (u16, u16);

    /// The layer this one overrides. [`Nothing`] under the first layer.
    fn base(&self) -> &dyn FormatLayer;

    /// Reads a node file.
    fn decode_node(&self, file: &str, raw: Value) -> Behaviour<Result<Decoded<Node>, StoreError>> {
        self.base().decode_node(file, raw)
    }

    /// Reads a comment file.
    fn decode_comment(
        &self,
        file: &str,
        raw: Value,
    ) -> Behaviour<Result<Decoded<Comment>, StoreError>> {
        self.base().decode_comment(file, raw)
    }

    /// Reads an amendment file.
    fn decode_amendment(
        &self,
        file: &str,
        raw: Value,
    ) -> Behaviour<Result<Decoded<Amendment>, StoreError>> {
        self.base().decode_amendment(file, raw)
    }
}

fn not_in_version<T>(behaviour: &'static str) -> Behaviour<T> {
    Err(NotInVersion { behaviour })
}

impl FormatLayer for Nothing {
    fn line(&self) -> (u16, u16) {
        (0, 0)
    }

    fn base(&self) -> &dyn FormatLayer {
        self
    }

    fn decode_node(&self, _: &str, _: Value) -> Behaviour<Result<Decoded<Node>, StoreError>> {
        not_in_version("decode_node")
    }

    fn decode_comment(&self, _: &str, _: Value) -> Behaviour<Result<Decoded<Comment>, StoreError>> {
        not_in_version("decode_comment")
    }

    fn decode_amendment(
        &self,
        _: &str,
        _: Value,
    ) -> Behaviour<Result<Decoded<Amendment>, StoreError>> {
        not_in_version("decode_amendment")
    }
}

/// The layer of the format `0.1`, the current one: it reads with the serde of the current types.
pub struct V0_1;

impl FormatLayer for V0_1 {
    fn line(&self) -> (u16, u16) {
        (0, 1)
    }

    fn base(&self) -> &dyn FormatLayer {
        &Nothing
    }

    fn decode_node(&self, file: &str, raw: Value) -> Behaviour<Result<Decoded<Node>, StoreError>> {
        Ok(current_node(file, raw))
    }

    fn decode_comment(
        &self,
        file: &str,
        raw: Value,
    ) -> Behaviour<Result<Decoded<Comment>, StoreError>> {
        Ok(current_comment(file, raw))
    }

    fn decode_amendment(
        &self,
        file: &str,
        raw: Value,
    ) -> Behaviour<Result<Decoded<Amendment>, StoreError>> {
        Ok(current_amendment(file, raw))
    }
}

/// The layers, oldest first.
pub static CHAIN: &[&dyn FormatLayer] = &[&V0_1];

/// The newest layer whose line is not past `found`'s, or `None` when `found` is older than the
/// first layer.
#[must_use]
pub fn layer_for(found: Version) -> Option<&'static dyn FormatLayer> {
    CHAIN
        .iter()
        .rev()
        .copied()
        .find(|layer| layer.line() <= (found.0, found.1))
}

/// Reads the stamp of the parsed file: no id is validated and no shape rule runs.
///
/// # Errors
/// `Invalid` with the rule `schema_version` when the stamp is missing, not a string, or not
/// `N.N.N`.
pub fn read_version(file: &str, raw: &Value) -> Result<Version, StoreError> {
    let invalid = |detail: &str| StoreError::invalid(file, Rule::SchemaVersion, detail);
    match raw.get("schema_version") {
        None => Err(invalid("`schema_version` is missing")),
        Some(Value::String(s)) => parse_version(s)
            .ok_or_else(|| invalid(&format!("`schema_version` {s:?} is not of the form N.N.N"))),
        Some(_) => Err(invalid("`schema_version` is not a string")),
    }
}

/// The version a parsed file carries and the layer that reads it, chosen from its stamp alone.
///
/// # Errors
/// `Invalid` with the rule `schema_version` (see [`read_version`]); `NewerFormat` when the file
/// is newer than [`FORMAT_VERSION`], a higher patch included, with nothing else of it looked at;
/// `Invalid` with the rule `unknown_format` when it is older than the first layer.
pub fn layer_of(
    file: &str,
    raw: &Value,
) -> Result<(Version, &'static dyn FormatLayer), StoreError> {
    let found = read_version(file, raw)?;
    if is_newer(found, FORMAT_VERSION) {
        return Err(StoreError::NewerFormat {
            found,
            supported: FORMAT_VERSION,
            file: file.to_owned(),
        });
    }
    layer_for(found).map(|layer| (found, layer)).ok_or_else(|| {
        StoreError::invalid(
            file,
            Rule::UnknownFormat,
            format!("no layer reads format {}", version_string(found)),
        )
    })
}
