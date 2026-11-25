//! The three ways the store publishes a file (`store.md`, §Write order, Effects): `put`
//! replaces, `create` never overwrites, `del` unlinks. Each goes through the [`Fs`] trait, so a
//! recording file system sees every effect, and each leaves its directory synced to the tier.

use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::fs::{Durability, Fs, atomic_replace};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_name() -> String {
    format!(
        "c{}-{}.tmp",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// `put X`: write the bytes under `tmp_dir`, sync them, rename over `target`, sync the
/// directories.
pub(crate) fn put(
    fs: &dyn Fs,
    target: &Path,
    tmp_dir: &Path,
    bytes: &[u8],
    tier: Durability,
) -> io::Result<()> {
    if let Some(parent) = target.parent() {
        fs.create_dir_all(parent)?;
    }
    atomic_replace(fs, target, tmp_dir, bytes, tier)
}

/// `create X`: the same, but published with a hard link and then the temporary name is
/// unlinked, so it fails with `AlreadyExists` if `target` exists instead of replacing it. There
/// is no create-only primitive in [`Fs`]: this is composed from `hard_link`.
pub(crate) fn create(
    fs: &dyn Fs,
    target: &Path,
    tmp_dir: &Path,
    bytes: &[u8],
    tier: Durability,
) -> io::Result<()> {
    if let Some(parent) = target.parent() {
        fs.create_dir_all(parent)?;
    }
    fs.create_dir_all(tmp_dir)?;
    let tmp = loop {
        let candidate = tmp_dir.join(temp_name());
        match fs.write_new(&candidate, bytes) {
            Ok(()) => break candidate,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => {
                let _ = fs.remove_file(&candidate);
                return Err(e);
            }
        }
    };
    let published = fs
        .sync_file(&tmp, tier)
        .and_then(|()| fs.hard_link(&tmp, target));
    let _ = fs.remove_file(&tmp);
    published?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    fs.sync_dir(parent, tier)?;
    fs.sync_dir(tmp_dir, tier)
}

/// `del X`: unlink. A file already gone is skipped.
pub(crate) fn del(fs: &dyn Fs, target: &Path, tier: Durability) -> io::Result<()> {
    match fs.remove_file(target) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    }
    fs.sync_dir(target.parent().unwrap_or_else(|| Path::new(".")), tier)
}
