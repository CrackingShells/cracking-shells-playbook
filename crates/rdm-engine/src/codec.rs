//! The canonical text of a stored record, and the read of any valid text back into the value
//! (`contracts.md`, `Version chain`; `stored-schema.md`, `Files`).
//!
//! One value has one text: the fields in the order of the type, two-space indent, LF, one trailing
//! newline, prose as arrays of lines, the keys the binary does not know after the known ones in
//! the order they came, `null` only where the contract has it. `serde_json` writes that when the
//! types declare the order, so the codec adds the newline and nothing else, and the guards in
//! `tests/codec.rs` hold the rest. Reading takes any valid JSON of the record, whatever its
//! whitespace, line ends, key order or escapes, and keeps what it does not know (I12).

use rdm_schema::campaign::{Decoded, Diagnostic};
use rdm_schema::error::{Behaviour, NotInVersion, Rule, StoreError};
use rdm_schema::ids::{MALFORMED_ID_TAG, NAME_FORMAT_TAG};
use rdm_schema::stored::{
    Actor, Amendment, Change, Check, Comment, Commit, Criterion, Directory, Extra, History, Leaf,
    Mark, Node, Progress, Reference, Step,
};
use rdm_schema::version_string;
use std::fmt;

use serde::Serialize;
use serde::de::{
    DeserializeOwned, DeserializeSeed, Deserializer, Error as _, MapAccess, SeqAccess, Visitor,
};
use serde_json::Value;

use crate::version::{FormatLayer, layer_of};

/// A stored record the codec reads and writes: a node, a comment or an amendment.
pub trait Stored: Serialize + Sized {
    /// Asks `layer` to read the record: the behaviour of that layer for this kind of file.
    fn decode_by(
        layer: &dyn FormatLayer,
        file: &str,
        raw: Value,
    ) -> Behaviour<Result<Decoded<Self>, StoreError>>;
}

impl Stored for Node {
    fn decode_by(
        layer: &dyn FormatLayer,
        file: &str,
        raw: Value,
    ) -> Behaviour<Result<Decoded<Self>, StoreError>> {
        layer.decode_node(file, raw)
    }
}

impl Stored for Comment {
    fn decode_by(
        layer: &dyn FormatLayer,
        file: &str,
        raw: Value,
    ) -> Behaviour<Result<Decoded<Self>, StoreError>> {
        layer.decode_comment(file, raw)
    }
}

impl Stored for Amendment {
    fn decode_by(
        layer: &dyn FormatLayer,
        file: &str,
        raw: Value,
    ) -> Behaviour<Result<Decoded<Self>, StoreError>> {
        layer.decode_amendment(file, raw)
    }
}

// The readers of the current format. Crate-private: only a layer of the chain calls them, so no
// caller can read a file without the version being checked first.

pub(crate) fn current_node(file: &str, raw: Value) -> Result<Decoded<Node>, StoreError> {
    match raw.get("kind").and_then(Value::as_str) {
        Some("leaf") => typed::<Leaf>(file, raw).map(|l| scanned(file, l, scan_leaf, Node::Leaf)),
        Some("campaign" | "directory") => {
            typed::<Directory>(file, raw).map(|d| scanned(file, d, scan_directory, Node::Directory))
        }
        _ => Err(StoreError::invalid(
            file,
            Rule::Kind,
            "`kind` is not `leaf`, `directory` or `campaign`",
        )),
    }
}

pub(crate) fn current_comment(file: &str, raw: Value) -> Result<Decoded<Comment>, StoreError> {
    typed::<Comment>(file, raw).map(|c| scanned(file, c, scan_comment, |c| c))
}

pub(crate) fn current_amendment(file: &str, raw: Value) -> Result<Decoded<Amendment>, StoreError> {
    typed::<Amendment>(file, raw).map(|a| scanned(file, a, scan_amendment, |a| a))
}

/// A stored record as a JSON value. The one policy for turning a record into JSON: it cannot
/// fail (every key of a stored type is a string and every value is JSON), so a failure is a
/// broken invariant and panics, rather than turning into a `null` that every record then shares.
pub(crate) fn json_value<T: Serialize + ?Sized>(record: &T) -> Value {
    serde_json::to_value(record).expect("a stored record serialises to JSON")
}

/// A stored record as pretty JSON text, without the trailing newline (see [`json_value`]).
pub(crate) fn json_text<T: Serialize + ?Sized>(record: &T) -> String {
    serde_json::to_string_pretty(record).expect("a stored record serialises to JSON")
}

/// A stored record as compact JSON bytes (see [`json_value`]).
pub(crate) fn json_bytes<T: Serialize + ?Sized>(record: &T) -> Vec<u8> {
    serde_json::to_vec(record).expect("a stored record serialises to JSON")
}

/// The canonical text of a record: ends with exactly one newline.
#[must_use]
pub fn encode<T: Stored>(value: &T) -> String {
    let mut text = json_text(value);
    text.push('\n');
    text
}

/// Reads a record from the bytes of the file `file` (its path relative to the campaign root, which
/// every error and diagnostic names).
///
/// The text is parsed once, into a value; the stamp is read from the value and picks the layer
/// of the chain (`crate::version`) that reads it, so nothing of a newer file is interpreted.
///
/// # Errors
/// `Invalid` with the rule `parse` when the bytes are not JSON or not an object of the record,
/// `schema_version` or `unknown_format` for the stamp, `kind`, `malformed_id` or `name_format`
/// when that field is the fault, `not_in_version` when the layer lacks the behaviour;
/// `NewerFormat` when the file is newer than the binary.
pub fn decode<T: Stored>(file: &str, bytes: &[u8]) -> Result<Decoded<T>, StoreError> {
    // Editors that write a UTF-8 byte order mark (PowerShell 5, older Notepad): one is skipped.
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let raw = parse(bytes).map_err(|e| StoreError::invalid(file, Rule::Parse, e.to_string()))?;
    if !raw.is_object() {
        return Err(StoreError::invalid(
            file,
            Rule::Parse,
            "the file is not a JSON object",
        ));
    }
    let (found, layer) = layer_of(file, &raw)?;
    match T::decode_by(layer, file, raw) {
        Ok(decoded) => decoded,
        Err(NotInVersion { behaviour }) => Err(StoreError::invalid(
            file,
            Rule::NotInVersion,
            format!("`{behaviour}` is not in format {}", version_string(found)),
        )),
    }
}

/// Parses the text once into a value, refusing a key written twice in one object, at any level:
/// the later value would win silently, even for a known field, and a clean git merge can leave
/// two. The error names the key's JSON pointer.
fn parse(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let value = Unique {
        pointer: String::new(),
    }
    .deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

/// Builds a [`Value`] like `Value::deserialize` does, and fails on a duplicate key.
struct Unique {
    pointer: String,
}

impl<'de> DeserializeSeed<'de> for Unique {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        d.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Unique {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("any valid JSON value")
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
        Ok(Value::from(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
        Ok(Value::from(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
        Ok(Value::from(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element_seed(Unique {
            pointer: format!("{}/{}", self.pointer, items.len()),
        })? {
            items.push(item);
        }
        Ok(Value::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut object = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            // RFC 6901: `~` is `~0` and `/` is `~1`, in that order.
            let pointer = format!(
                "{}/{}",
                self.pointer,
                key.replace('~', "~0").replace('/', "~1")
            );
            if object.contains_key(&key) {
                return Err(A::Error::custom(format!("duplicate key at {pointer}")));
            }
            let value = map.next_value_seed(Unique { pointer })?;
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }
}

/// Reads `raw` as a `T`, telling the rules apart by the tag the id and name types put on their
/// messages (`rdm_schema::ids`): everything else is `parse`.
fn typed<T: DeserializeOwned>(file: &str, raw: Value) -> Result<T, StoreError> {
    serde_json::from_value(raw).map_err(|e| {
        let message = e.to_string();
        let (rule, detail) = if let Some(rest) = message.strip_prefix(MALFORMED_ID_TAG) {
            (Rule::MalformedId, rest)
        } else if let Some(rest) = message.strip_prefix(NAME_FORMAT_TAG) {
            (Rule::NameFormat, rest)
        } else {
            (Rule::Parse, message.as_str())
        };
        StoreError::invalid(file, rule, detail)
    })
}

/// Scans a decoded `value` for the keys it kept, and wraps it as the record the caller asked for.
fn scanned<T, R>(file: &str, value: T, scan: fn(&mut Scan, &T), wrap: fn(T) -> R) -> Decoded<R> {
    let mut s = Scan {
        file,
        path: Vec::new(),
        found: Vec::new(),
    };
    scan(&mut s, &value);
    Decoded {
        value: wrap(value),
        diagnostics: s.found,
    }
}

// ---------------------------------------------------------------------------------------------
// The unknown keys kept in `extra`, one diagnostic each, at the JSON pointer of the key.

struct Scan<'a> {
    file: &'a str,
    path: Vec<String>,
    found: Vec<Diagnostic>,
}

impl Scan<'_> {
    fn extra(&mut self, extra: &Extra) {
        for key in extra.keys() {
            let mut pointer = String::new();
            for segment in self.path.iter().map(String::as_str).chain([key.as_str()]) {
                pointer.push('/');
                // RFC 6901: `~` is `~0` and `/` is `~1`, in that order.
                pointer.push_str(&segment.replace('~', "~0").replace('/', "~1"));
            }
            self.found.push(Diagnostic {
                file: self.file.to_owned(),
                rule: Rule::UnknownKey,
                detail: pointer,
            });
        }
    }

    fn field<R>(&mut self, name: &str, inside: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(name.to_owned());
        let r = inside(self);
        self.path.pop();
        r
    }

    fn each<T>(&mut self, name: &str, items: &[T], one: fn(&mut Self, &T)) {
        self.field(name, |s| {
            for (i, item) in items.iter().enumerate() {
                s.field(&i.to_string(), |s| one(s, item));
            }
        });
    }
}

fn scan_actor(s: &mut Scan, a: &Actor) {
    s.extra(&a.extra);
}

fn scan_criterion(s: &mut Scan, c: &Criterion) {
    s.extra(&c.extra);
}

fn scan_reference(s: &mut Scan, r: &Reference) {
    s.extra(&r.extra);
}

fn scan_mark(s: &mut Scan, m: &Mark) {
    s.extra(&m.extra);
    s.field("actor", |s| scan_actor(s, &m.actor));
}

fn scan_check(s: &mut Scan, c: &Check) {
    s.extra(&c.extra);
}

fn scan_commit(s: &mut Scan, c: &Commit) {
    s.extra(&c.extra);
}

fn scan_step(s: &mut Scan, step: &Step) {
    s.extra(&step.extra);
    s.each("references", &step.references, scan_reference);
    s.each("checks", &step.checks, scan_check);
    s.field("commit", |s| scan_commit(s, &step.commit));
}

fn scan_history(s: &mut Scan, h: &History) {
    s.extra(&h.extra);
    s.field("actor", |s| scan_actor(s, &h.actor));
}

fn scan_progress(s: &mut Scan, p: &Progress) {
    s.extra(&p.extra);
}

fn scan_leaf(s: &mut Scan, l: &Leaf) {
    s.extra(&l.extra);
    s.each("preconditions", &l.preconditions, scan_criterion);
    s.each("success_gates", &l.success_gates, scan_criterion);
    s.each("references", &l.references, scan_reference);
    s.each("steps", &l.steps, scan_step);
    s.field("progress", |s| scan_progress(s, &l.progress));
    s.each("history", &l.history, scan_history);
    s.each("marks", &l.marks, scan_mark);
}

fn scan_directory(s: &mut Scan, d: &Directory) {
    s.extra(&d.extra);
    s.each("references", &d.references, scan_reference);
    s.each("preconditions", &d.preconditions, scan_criterion);
    s.each("success_gates", &d.success_gates, scan_criterion);
    s.each("marks", &d.marks, scan_mark);
}

fn scan_comment(s: &mut Scan, c: &Comment) {
    s.extra(&c.extra);
    s.field("actor", |s| scan_actor(s, &c.actor));
}

fn scan_change(s: &mut Scan, c: &Change) {
    s.extra(&c.extra);
}

fn scan_amendment(s: &mut Scan, a: &Amendment) {
    s.extra(&a.extra);
    s.field("actor", |s| scan_actor(s, &a.actor));
    s.each("changes", &a.changes, scan_change);
}
