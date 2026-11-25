//! The redo intent record (`store.md` §Intent and recovery): `.rdm/intent/<ulid>.json`, the
//! ordered whole-file effects of a multi-file operation, published before the first of them.
//!
//! This module writes, completes and lists records. Redoing them is recovery's job, which
//! belongs to the store. What survives a kill is guarded by `crash_test`, not here.

use crate::fs::{Durability, Fs, atomic_replace};
use crate::layout;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

/// Format version of the record itself. A newer one is refused by the reader of the record.
pub const INTENT_FORMAT: u32 = 1;

/// One effect of step 5, in the order it is applied. `path` is relative to the campaign, with `/`
/// separators. `Put` and `Create` carry the full new content as text and `Del` carries none; a
/// record that breaks this does not decode.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase", deny_unknown_fields)]
pub enum IntentEffect {
    /// Replace the file (whole-file put, idempotent).
    Put { path: String, bytes: String },
    /// Create the file; never overwrites.
    Create { path: String, bytes: String },
    /// Unlink the file; a missing one is skipped.
    Del { path: String },
}

impl IntentEffect {
    /// The campaign-relative, `/`-separated path the effect touches.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            IntentEffect::Put { path, .. }
            | IntentEffect::Create { path, .. }
            | IntentEffect::Del { path } => path,
        }
    }
}

/// An intent record: its format version and the effects in the order they are applied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intent {
    pub format: u32,
    pub effects: Vec<IntentEffect>,
}

impl Intent {
    pub fn new(effects: Vec<IntentEffect>) -> Self {
        Intent {
            format: INTENT_FORMAT,
            effects,
        }
    }
}

/// A record found in `.rdm/intent/`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Pending {
    /// A record this binary can redo.
    Record { ulid: String, intent: Intent },
    /// A record whose `format` is higher than [`INTENT_FORMAT`]: refuse it (`NewerFormat`).
    Newer { ulid: String, found: u32 },
    /// A record that does not decode (a power-loss artefact): recovery removes it and reports it.
    Undecodable { ulid: String },
}

impl Pending {
    pub fn ulid(&self) -> &str {
        match self {
            Pending::Record { ulid, .. }
            | Pending::Newer { ulid, .. }
            | Pending::Undecodable { ulid } => ulid,
        }
    }
}

fn is_ulid(s: &str) -> bool {
    rdm_schema::ids::AmendmentId::parse(s).is_ok()
}

/// Publish the intent `<ulid>` under `root`: written under `.rdm/tmp/`, synced, renamed into
/// `.rdm/intent/`. Its rename is the commit point of the operation. Refuses a `ulid` that is not
/// a ULID (`InvalidInput`) and never overwrites a record (`AlreadyExists`); the writer lock is
/// held, so the check and the publish do not race.
pub fn begin(
    fs: &dyn Fs,
    root: &Path,
    ulid: &str,
    intent: &Intent,
    tier: Durability,
) -> io::Result<()> {
    if !is_ulid(ulid) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{ulid:?} is not a ULID"),
        ));
    }
    let bytes = serde_json::to_vec(intent).map_err(io::Error::other)?;
    let record = layout::intent_record(root, ulid);
    match fs.read(&record) {
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("an intent {ulid} is already recorded"),
            ));
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    fs.create_dir_all(&layout::intent_dir(root))?;
    atomic_replace(fs, &record, &layout::tmp_dir(root), &bytes, tier)
}

/// Remove the intent `<ulid>` and sync `.rdm/intent/`. A record already gone is not an error.
pub fn complete(fs: &dyn Fs, root: &Path, ulid: &str, tier: Durability) -> io::Result<()> {
    let dir = layout::intent_dir(root);
    match fs.remove_file(&layout::intent_record(root, ulid)) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    fs.sync_dir(&dir, tier)
}

/// The records in `.rdm/intent/`, oldest ULID first (the ULID is the order). Entries that are
/// not a ULID with `.json` are ignored. A missing directory holds nothing.
pub fn pending(fs: &dyn Fs, root: &Path) -> io::Result<Vec<Pending>> {
    let dir = layout::intent_dir(root);
    let names = match fs.list_dir(&dir) {
        Ok(names) => names,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };
    let mut found = Vec::new();
    for name in names {
        let Some(ulid) = layout::intent_ulid(&name).filter(|u| is_ulid(u)) else {
            continue;
        };
        let bytes = match fs.read(&dir.join(&name)) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err),
        };
        found.push(decode(ulid, &bytes));
    }
    found.sort_by(|a, b| a.ulid().cmp(b.ulid()));
    Ok(found)
}

fn decode(ulid: &str, bytes: &[u8]) -> Pending {
    let ulid = ulid.to_string();
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return Pending::Undecodable { ulid };
    };
    match value.get("format").and_then(serde_json::Value::as_u64) {
        Some(found) if found > u64::from(INTENT_FORMAT) => Pending::Newer {
            ulid,
            found: u32::try_from(found).unwrap_or(u32::MAX),
        },
        Some(_) => match serde_json::from_value(value) {
            Ok(intent) => Pending::Record { ulid, intent },
            Err(_) => Pending::Undecodable { ulid },
        },
        None => Pending::Undecodable { ulid },
    }
}
