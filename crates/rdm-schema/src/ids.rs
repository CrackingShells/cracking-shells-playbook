//! Identifier and key newtypes (`contracts.md`, `Id types`, `Node fields`, `Operations`).
//!
//! Every id is a `String` that deserialises only through its validating `parse`, so a value of
//! one of these types is always well formed and a consumer never re-checks it. The grammar is
//! written once, here, from `store.md` §Ids, and the generated JSON Schema pattern comes from the
//! same constants.

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The alphabet of node, criterion and step ids: Crockford base32 in lower case, no `i`, `l`,
/// `o` or `u` (`store.md`, `Ids`).
pub const ID_ALPHABET: &str = "0123456789abcdefghjkmnpqrstvwxyz";
/// The alphabet of the ULIDs of comments and amendments: Crockford base32 in upper case.
pub const ULID_ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// Length of a ULID.
pub const ULID_LEN: usize = 26;
/// Longest `OpId`, in characters.
pub const OP_ID_MAX_LEN: usize = 128;

/// Start of the message of a deserialisation error caused by a malformed id, so a decoder can
/// tell the rule `malformed_id` from `parse` without parsing prose.
pub const MALFORMED_ID_TAG: &str = "malformed_id: ";
/// Start of the message of a deserialisation error caused by a bad [`Name`]: the rule `name_format`.
pub const NAME_FORMAT_TAG: &str = "name_format: ";

/// Prefix of a node id.
pub const NODE_PREFIX: &str = "n_";
/// Prefix of a criterion id. Differs from the step prefix: `ItemId` reads by prefix.
pub const CRITERION_PREFIX: &str = "c_";
/// Prefix of a step id.
pub const STEP_PREFIX: &str = "s_";
/// Characters after the prefix of a node id.
pub const NODE_BODY_LEN: usize = 8;
/// Characters after the prefix of a criterion or a step id.
pub const ITEM_BODY_LEN: usize = 6;

/// Why a string is not an id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdError {
    /// The string is empty.
    Empty,
    /// The string does not start with the kind prefix of its type.
    WrongPrefix { expected: &'static str },
    /// A character is outside the alphabet (or is a control character, for an `OpId`).
    BadChar,
    /// The body has the wrong length.
    BadLength,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty id"),
            Self::WrongPrefix { expected } => write!(f, "id does not start with `{expected}`"),
            Self::BadChar => f.write_str("id has a character outside its alphabet"),
            Self::BadLength => f.write_str("id has the wrong length"),
        }
    }
}

impl std::error::Error for IdError {}

fn check_prefixed(s: &str, prefix: &'static str, body_len: usize) -> Result<(), IdError> {
    if s.is_empty() {
        return Err(IdError::Empty);
    }
    let Some(body) = s.strip_prefix(prefix) else {
        return Err(IdError::WrongPrefix { expected: prefix });
    };
    if body.chars().count() != body_len {
        return Err(IdError::BadLength);
    }
    if !body.chars().all(|c| ID_ALPHABET.contains(c)) {
        return Err(IdError::BadChar);
    }
    Ok(())
}

fn check_ulid(s: &str) -> Result<(), IdError> {
    if s.is_empty() {
        return Err(IdError::Empty);
    }
    if s.chars().count() != ULID_LEN {
        return Err(IdError::BadLength);
    }
    if !s.chars().all(|c| ULID_ALPHABET.contains(c)) {
        return Err(IdError::BadChar);
    }
    Ok(())
}

fn check_op_id(s: &str) -> Result<(), IdError> {
    if s.is_empty() {
        return Err(IdError::Empty);
    }
    if s.chars().count() > OP_ID_MAX_LEN {
        return Err(IdError::BadLength);
    }
    if s.chars().any(char::is_control) {
        return Err(IdError::BadChar);
    }
    Ok(())
}

/// Defines a string newtype that is built only through a validating `parse`.
macro_rules! checked_string {
    ($(#[$meta:meta])* $name:ident, $check:expr, $pattern:expr) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// The only constructor from a string.
            ///
            /// # Errors
            /// [`IdError`] when `s` is not of the format of this type.
            pub fn parse(s: &str) -> Result<Self, IdError> {
                let check: fn(&str) -> Result<(), IdError> = $check;
                check(s)?;
                Ok(Self(s.to_owned()))
            }

            /// The id as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl FromStr for $name {
            type Err = IdError;
            fn from_str(s: &str) -> Result<Self, IdError> {
                Self::parse(s)
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdError;
            fn try_from(s: String) -> Result<Self, IdError> {
                Self::parse(&s)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let s = String::deserialize(d)?;
                Self::parse(&s).map_err(|e| D::Error::custom(format!("{MALFORMED_ID_TAG}{}: {e}", stringify!($name))))
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> Cow<'static, str> {
                stringify!($name).into()
            }
            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                let pattern: String = $pattern;
                json_schema!({ "type": "string", "pattern": pattern })
            }
        }
    };
}

fn prefixed_pattern(prefix: &str, body_len: usize) -> String {
    format!("^{prefix}[{ID_ALPHABET}]{{{body_len}}}$")
}

checked_string! {
    /// The id of a node, whatever its `kind` (promoting a leaf keeps it, I7).
    NodeId,
    |s| check_prefixed(s, NODE_PREFIX, NODE_BODY_LEN),
    prefixed_pattern(NODE_PREFIX, NODE_BODY_LEN)
}

checked_string! {
    /// The id of a criterion, in `preconditions` or `success_gates`.
    CriterionId,
    |s| check_prefixed(s, CRITERION_PREFIX, ITEM_BODY_LEN),
    prefixed_pattern(CRITERION_PREFIX, ITEM_BODY_LEN)
}

checked_string! {
    /// The id of a step.
    StepId,
    |s| check_prefixed(s, STEP_PREFIX, ITEM_BODY_LEN),
    prefixed_pattern(STEP_PREFIX, ITEM_BODY_LEN)
}

checked_string! {
    /// The id of a comment: a ULID, so comments order by id, oldest first.
    CommentId,
    check_ulid,
    format!("^[{ULID_ALPHABET}]{{{ULID_LEN}}}$")
}

checked_string! {
    /// The id of an amendment: a ULID, so amendments order by id, oldest first.
    AmendmentId,
    check_ulid,
    format!("^[{ULID_ALPHABET}]{{{ULID_LEN}}}$")
}

checked_string! {
    /// The retry key of one write: one line of 1 to 128 characters with no control character.
    /// The caller chooses it; the store never mints one.
    OpId,
    check_op_id,
    format!("^[^\\u0000-\\u001f\\u007f-\\u009f]{{1,{OP_ID_MAX_LEN}}}$")
}

/// The id of a criterion or of a step, as `Change.item_id` names one. Its JSON form is the bare
/// string, read as a [`CriterionId`] or a [`StepId`] by its prefix.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ItemId {
    Criterion(CriterionId),
    Step(StepId),
}

impl ItemId {
    /// Reads a criterion id or a step id by its prefix.
    ///
    /// # Errors
    /// [`IdError`]: `WrongPrefix` when the prefix is neither, else the error of the matching type.
    pub fn parse(s: &str) -> Result<Self, IdError> {
        if s.is_empty() {
            Err(IdError::Empty)
        } else if s.starts_with(CRITERION_PREFIX) {
            CriterionId::parse(s).map(Self::Criterion)
        } else if s.starts_with(STEP_PREFIX) {
            StepId::parse(s).map(Self::Step)
        } else {
            Err(IdError::WrongPrefix { expected: CRITERION_PREFIX })
        }
    }

    /// The id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Criterion(id) => id.as_str(),
            Self::Step(id) => id.as_str(),
        }
    }
}

impl FromStr for ItemId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, IdError> {
        Self::parse(s)
    }
}

impl TryFrom<String> for ItemId {
    type Error = IdError;
    fn try_from(s: String) -> Result<Self, IdError> {
        Self::parse(&s)
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ItemId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ItemId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::parse(&s).map_err(|e| D::Error::custom(format!("{MALFORMED_ID_TAG}ItemId: {e}")))
    }
}

impl JsonSchema for ItemId {
    fn schema_name() -> Cow<'static, str> {
        "ItemId".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let pattern = format!(
            "^({CRITERION_PREFIX}[{ID_ALPHABET}]{{{ITEM_BODY_LEN}}}|{STEP_PREFIX}[{ID_ALPHABET}]{{{ITEM_BODY_LEN}}})$"
        );
        json_schema!({ "type": "string", "pattern": pattern })
    }
}

/// Why a string is not a [`Name`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    /// The string is empty.
    Empty,
    /// The first character is not `a` to `z`.
    BadStart,
    /// A later character is not `a` to `z`, `0` to `9`, `_` or `-`.
    BadChar,
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty name"),
            Self::BadStart => f.write_str("a name starts with a letter a-z"),
            Self::BadChar => f.write_str("a name holds only a-z, 0-9, `_` and `-`"),
        }
    }
}

impl std::error::Error for NameError {}

/// A node's path segment, `^[a-z][a-z0-9_-]*$`. For the campaign root, the campaign slug.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(String);

impl Name {
    /// The only constructor from a string.
    ///
    /// # Errors
    /// [`NameError`] when `s` is not of the form `^[a-z][a-z0-9_-]*$`.
    pub fn parse(s: &str) -> Result<Self, NameError> {
        let mut chars = s.chars();
        match chars.next() {
            None => return Err(NameError::Empty),
            Some(c) if !c.is_ascii_lowercase() => return Err(NameError::BadStart),
            Some(_) => {}
        }
        if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') {
            return Err(NameError::BadChar);
        }
        Ok(Self(s.to_owned()))
    }

    /// The name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Name {
    type Err = NameError;
    fn from_str(s: &str) -> Result<Self, NameError> {
        Self::parse(s)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Name {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::parse(&s).map_err(|e| D::Error::custom(format!("{NAME_FORMAT_TAG}Name: {e}")))
    }
}

impl JsonSchema for Name {
    fn schema_name() -> Cow<'static, str> {
        "Name".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({ "type": "string", "pattern": "^[a-z][a-z0-9_-]*$" })
    }
}

/// Defines an opaque string newtype: no validation here, its format is the engine's.
macro_rules! opaque_string {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wraps a value produced by the engine (or read back from a file).
            #[must_use]
            pub fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            /// The value as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

opaque_string! {
    /// The position of a node among its siblings. Opaque: its format and how a key is made
    /// between two others are `store.md`'s (§Order key). Compared as given, never parsed.
    OrderKey
}

opaque_string! {
    /// The revision of one node: a hash of its canonical encoding (`store.md`, §Revisions).
    /// Consumers compare for equality and never parse.
    Rev
}

opaque_string! {
    /// The revision of a node and every descendant (`store.md`, §Revisions).
    SubtreeRev
}

opaque_string! {
    /// The revision of the campaign (`store.md`, §Revisions). Consumers compare for equality and
    /// never parse.
    TreeRev
}
