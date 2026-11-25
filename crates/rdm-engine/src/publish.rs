//! The three ways the store publishes a file (`store.md`, §Write order, Effects): `put`
//! replaces, `create` never overwrites, `del` unlinks. Each goes through the [`Fs`] trait, so a
//! recording file system sees every effect, and each leaves its directory synced to the tier.

use std::io;
use std::path::Path;

use crate::fs::{Durability, Fs, atomic_replace, stage};

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
/// is no create-only primitive in [`Fs`]: this is composed from `hard_link`, and from "check the
/// name is free, then rename" on a volume that refuses hard links (see [`link_or_rename`]).
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
    let tmp = stage(fs, tmp_dir, bytes)?;
    let published = fs
        .sync_file(&tmp, tier)
        .and_then(|()| link_or_rename(fs, &tmp, target));
    let _ = fs.remove_file(&tmp);
    published?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    fs.sync_dir(parent, tier)?;
    fs.sync_dir(tmp_dir, tier)
}

/// Publishes `tmp` as `target` without replacing it. A volume that refuses hard links (FAT and
/// exFAT media, some bind and FUSE mounts: `Unsupported` or `PermissionDenied`) gets the
/// fallback of `store.md` (§Windows, Create-only publish): the writer lock is held, so checking
/// that the name is free and then renaming races only an external writer. A name that is taken
/// fails with `AlreadyExists`, as a refused link does on a volume that has them.
fn link_or_rename(fs: &dyn Fs, tmp: &Path, target: &Path) -> io::Result<()> {
    match fs.hard_link(tmp, target) {
        Err(e) if matches!(e.kind(), io::ErrorKind::Unsupported | io::ErrorKind::PermissionDenied) => {
            match fs.stat(target) {
                Ok(_) => Err(io::ErrorKind::AlreadyExists.into()),
                Err(e) if e.kind() == io::ErrorKind::NotFound => fs.rename(tmp, target),
                Err(e) => Err(e),
            }
        }
        other => other,
    }
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
