//! The writer lock and its holder record (`store.md` §Lock).
//!
//! One advisory whole-file lock per campaign on `.rdm/lock`, taken with `fs4` and released by
//! the kernel when the process dies. The lock file is empty and never read or written. The
//! holder record lives in its own file, `.rdm/holder.json`, because on Windows `LockFileEx`
//! denies other processes read access to the locked file. The record is bytes the caller
//! chose; it is informational and the lock state never depends on it.

use crate::fs::{Durability, Fs, RealFs, atomic_replace};
use crate::layout;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const FIRST_BACKOFF: Duration = Duration::from_millis(5);
const MAX_BACKOFF: Duration = Duration::from_millis(50);

/// Why a lock was not acquired.
#[derive(Debug)]
#[non_exhaustive]
pub enum LockError {
    /// Another writer held the lock for the whole timeout. `holder` is the bytes of
    /// `.rdm/holder.json` as read (`None` when absent or unreadable). They may be garbage, or
    /// the previous holder's record: the caller parses them and treats a failure as unknown.
    Busy { holder: Option<Vec<u8>> },
    /// The lock file could not be opened or locked.
    Io(io::Error),
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockError::Busy {
                holder: Some(bytes),
            } => write!(
                f,
                "the campaign is locked by another writer ({} byte holder record)",
                bytes.len()
            ),
            LockError::Busy { holder: None } => {
                write!(f, "the campaign is locked by an unknown writer")
            }
            LockError::Io(err) => write!(f, "cannot take the writer lock: {err}"),
        }
    }
}

impl std::error::Error for LockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LockError::Io(err) => Some(err),
            LockError::Busy { .. } => None,
        }
    }
}

/// A held writer lock. Dropping it removes the holder record and releases the lock.
#[derive(Debug)]
pub struct WriterLock {
    file: Option<File>,
    holder_path: PathBuf,
}

impl WriterLock {
    /// Take the writer lock of the campaign at `root`, waiting up to `timeout`, then record
    /// `holder_record` in `.rdm/holder.json`.
    ///
    /// Tries at once, then backs off from 5 ms doubling to 50 ms. At the timeout nothing has
    /// changed and `LockError::Busy` carries the current holder record. Writing the record is
    /// best effort: the lock is held even if the advisory file cannot be written.
    pub fn acquire(
        root: &Path,
        timeout: Duration,
        holder_record: &[u8],
    ) -> Result<WriterLock, LockError> {
        // `create_dir`, not `create_dir_all`: a missing root is an error and creates nothing.
        match std::fs::create_dir(layout::rdm_dir(root)) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(LockError::Io(e)),
        }
        // Opened for writing only so that it can be created; its bytes are never touched.
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(layout::lock_path(root))
            .map_err(LockError::Io)?;

        let deadline = Instant::now() + timeout;
        let mut backoff = FIRST_BACKOFF;
        loop {
            // The trait method by its full name: std's inherent `File::try_lock` (Rust 1.89)
            // would otherwise shadow it on newer toolchains.
            match fs4::FileExt::try_lock(&file) {
                Ok(()) => break,
                Err(fs4::TryLockError::Error(err)) => return Err(LockError::Io(err)),
                Err(fs4::TryLockError::WouldBlock) => {
                    let left = deadline.saturating_duration_since(Instant::now());
                    if left.is_zero() {
                        return Err(LockError::Busy {
                            holder: RealFs.read(&layout::holder_path(root)).ok(),
                        });
                    }
                    std::thread::sleep(backoff.min(left));
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                }
            }
        }

        let holder_path = layout::holder_path(root);
        let _ = atomic_replace(
            &RealFs,
            &holder_path,
            &layout::tmp_dir(root),
            holder_record,
            Durability::Process,
        );
        Ok(WriterLock {
            file: Some(file),
            holder_path,
        })
    }

    /// Remove the holder record, then release the lock.
    pub fn release(mut self) {
        self.release_inner();
    }

    fn release_inner(&mut self) {
        if let Some(file) = self.file.take() {
            let _ = RealFs.remove_file(&self.holder_path);
            let _ = fs4::FileExt::unlock(&file);
        }
    }
}

impl Drop for WriterLock {
    fn drop(&mut self) {
        self.release_inner();
    }
}
