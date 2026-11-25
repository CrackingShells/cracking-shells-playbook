//! Where the store keeps its own files (`store.md`, §Layout): the `.rdm/` scratch directory of a
//! campaign and the names under it.
//!
//! Two kinds of spelling live here and nowhere else. A **path** is a place on the file system,
//! built from the campaign root. A **logical name** is the `/`-joined text that appears in a
//! `StoreError` (its `file`) and in the effects of an intent record, the same on every platform.
//! A logical record name becomes a path in one function, [`record_path`].

use std::path::{Path, PathBuf};

const RDM_DIR: &str = ".rdm";
const TMP_DIR: &str = "tmp";
const INTENT_DIR: &str = "intent";
const LOCK_FILE: &str = "lock";
const HOLDER_FILE: &str = "holder.json";
const GITIGNORE_FILE: &str = ".gitignore";
const INTENT_EXT: &str = ".json";

/// The logical names of the scratch files and directories, for the `file` of an error.
pub(crate) const LOCK_NAME: &str = ".rdm/lock";
pub(crate) const GITIGNORE_NAME: &str = ".rdm/.gitignore";
pub(crate) const TMP_NAME: &str = ".rdm/tmp";
pub(crate) const INTENT_NAME: &str = ".rdm/intent";

/// `.rdm/`, the directory of everything the store writes that is not a campaign record.
pub(crate) fn rdm_dir(root: &Path) -> PathBuf {
    root.join(RDM_DIR)
}

/// `.rdm/tmp/`: where a file is staged before it is published. Outside the directories the loader
/// reads, and on the file system of the records.
pub(crate) fn tmp_dir(root: &Path) -> PathBuf {
    rdm_dir(root).join(TMP_DIR)
}

/// `.rdm/intent/`: the redo records.
pub(crate) fn intent_dir(root: &Path) -> PathBuf {
    rdm_dir(root).join(INTENT_DIR)
}

/// `.rdm/lock`: the empty file the writer lock is taken on.
pub(crate) fn lock_path(root: &Path) -> PathBuf {
    rdm_dir(root).join(LOCK_FILE)
}

/// `.rdm/holder.json`: the informational record of who holds the lock.
pub(crate) fn holder_path(root: &Path) -> PathBuf {
    rdm_dir(root).join(HOLDER_FILE)
}

/// `.rdm/.gitignore`: holds `*`, so the repository needs no entry of its own.
pub(crate) fn gitignore_path(root: &Path) -> PathBuf {
    rdm_dir(root).join(GITIGNORE_FILE)
}

/// The file name of the intent record `<ulid>`: `<ulid>.json`.
pub(crate) fn intent_file_name(ulid: &str) -> String {
    format!("{ulid}{INTENT_EXT}")
}

/// The ULID an entry of `.rdm/intent/` is named after, if it is named `<something>.json`.
pub(crate) fn intent_ulid(file_name: &str) -> Option<&str> {
    file_name.strip_suffix(INTENT_EXT)
}

/// The path of the intent record `<ulid>`.
pub(crate) fn intent_record(root: &Path, ulid: &str) -> PathBuf {
    intent_dir(root).join(intent_file_name(ulid))
}

/// The logical name of the intent record `<ulid>`: `.rdm/intent/<ulid>.json`.
pub(crate) fn intent_record_name(ulid: &str) -> String {
    format!("{INTENT_NAME}/{}", intent_file_name(ulid))
}

/// The logical name of a file in `.rdm/tmp/`.
pub(crate) fn tmp_file_name(name: &str) -> String {
    format!("{TMP_NAME}/{name}")
}

/// The path of a campaign record given its logical name: `/`-separated, relative to the campaign
/// root. The one place a logical name becomes a path.
pub(crate) fn record_path(root: &Path, logical: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    path.extend(logical.split('/'));
    path
}
