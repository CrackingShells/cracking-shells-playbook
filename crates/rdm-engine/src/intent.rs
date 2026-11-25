//! The redo intent record (`store.md` §Intent and recovery): `.rdm/intent/<ulid>.json`, the
//! ordered whole-file effects of a multi-file operation, published before the first of them.
//!
//! This module writes, completes and lists records. Redoing them is recovery's job, which
//! belongs to the store. What survives a kill is guarded by `crash_test`, not here.

use crate::fs::{Durability, Fs, atomic_replace};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

/// Format version of the record itself. A newer one is refused by the reader of the record.
pub const INTENT_FORMAT: u32 = 1;

/// What to do to one path on redo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// Replace the file (whole-file put, idempotent).
    Put,
    /// Create the file; never overwrites.
    Create,
    /// Unlink the file; a missing one is skipped.
    Del,
}

/// One effect of step 5. `path` is relative to the campaign, with `/` separators. `bytes` is the
/// full new content as text for `Put` and `Create`, and absent for `Del`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentEffect {
    pub action: Action,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<String>,
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

fn intent_dir(root: &Path) -> std::path::PathBuf {
    root.join(".rdm").join("intent")
}

/// Publish the intent `<ulid>` under `root`: written under `.rdm/tmp/`, synced, renamed into
/// `.rdm/intent/`. Its rename is the commit point of the operation.
pub fn begin(
    fs: &dyn Fs,
    root: &Path,
    ulid: &str,
    intent: &Intent,
    tier: Durability,
) -> io::Result<()> {
    let bytes = serde_json::to_vec(intent).map_err(io::Error::other)?;
    let dir = intent_dir(root);
    fs.create_dir_all(&dir)?;
    atomic_replace(
        fs,
        &dir.join(format!("{ulid}.json")),
        &root.join(".rdm").join("tmp"),
        &bytes,
        tier,
    )
}

/// Remove the intent `<ulid>` and sync `.rdm/intent/`. A record already gone is not an error.
pub fn complete(fs: &dyn Fs, root: &Path, ulid: &str, tier: Durability) -> io::Result<()> {
    let dir = intent_dir(root);
    match fs.remove_file(&dir.join(format!("{ulid}.json"))) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    fs.sync_dir(&dir, tier)
}

/// The records in `.rdm/intent/`, oldest ULID first (the ULID is the order). Entries that are
/// not a 26-character ULID with `.json` are ignored. A missing directory holds nothing.
pub fn pending(fs: &dyn Fs, root: &Path) -> io::Result<Vec<Pending>> {
    let dir = intent_dir(root);
    let names = match fs.list_dir(&dir) {
        Ok(names) => names,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };
    let mut found = Vec::new();
    for name in names {
        let Some(ulid) = name.strip_suffix(".json").filter(|u| u.len() == 26) else {
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
