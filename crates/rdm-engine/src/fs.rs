//! The `Fs` trait over the effects the store needs, its real and recording implementations,
//! and `atomic_replace` (`store.md` §Reads, §Durability, §Windows).

use std::io;
use std::path::{Path, PathBuf};
#[cfg(any(test, feature = "test-support"))]
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Durability tier of a publish (`store.md` §Durability). Chosen by the store when it is opened.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Durability {
    /// No sync. Ordering still comes from the rename.
    Process,
    /// Sync each file before it is published and each touched directory after. On macOS this is
    /// plain `fsync(2)`, not `F_FULLFSYNC`.
    #[default]
    Fsync,
    /// As `Fsync`, with `F_FULLFSYNC` on macOS. Equal to `Fsync` elsewhere.
    Full,
}

/// What a revalidation compares of a file (`store.md`, §Cache): its identity, modification time
/// and size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileStat {
    /// The inode on Unix, the creation time on Windows: a replace by rename gives the new file a
    /// new identity.
    pub identity: u64,
    /// The modification time.
    pub mtime: std::time::SystemTime,
    /// The length in bytes.
    pub size: u64,
    /// The inode change time on Unix, as seconds and nanoseconds; `None` on Windows. No
    /// user-level call sets it, so an in-place edit that restores the modification time still
    /// moves it.
    pub ctime: Option<(i64, i64)>,
}

/// The effects the store performs on the file system, plus the reads it needs.
///
/// Paths are taken as given. The mutating methods are the ones a `RecordingFs` logs.
pub trait Fs {
    /// Create `path` and its missing ancestors. An existing directory is not an error.
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    /// Create `path` with `bytes`. Fails with `AlreadyExists` if it exists. No sync.
    fn write_new(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;
    /// Sync a file's data to the tier. A no-op for `Durability::Process`.
    fn sync_file(&self, path: &Path, tier: Durability) -> io::Result<()>;
    /// Sync a directory's entries to the tier. A no-op for `Process`, and on Windows.
    fn sync_dir(&self, path: &Path, tier: Durability) -> io::Result<()>;
    /// Rename `from` over `to`, replacing it (with the Windows retry on `RealFs`).
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    /// Create `link` as a hard link to `original`. Fails with `AlreadyExists` if `link` exists.
    fn hard_link(&self, original: &Path, link: &Path) -> io::Result<()>;
    /// Unlink a file (with the Windows retry on `RealFs`).
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    /// Read a whole file.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    /// File names in a directory, sorted bytewise. A missing directory is `NotFound`.
    fn list_dir(&self, path: &Path) -> io::Result<Vec<String>>;
    /// The signature of a file ([`FileStat`]). A missing file is `NotFound`. No default: every
    /// implementation says what it reports.
    fn stat(&self, path: &Path) -> io::Result<FileStat>;
}

/// One logged effect. Paths are relative to the root the `RecordingFs` was built with, so a
/// prefix of the log applies to a copy of that directory (see [`Effect::apply`]). Test support:
/// built for tests and under the `test-support` feature only.
#[cfg(any(test, feature = "test-support"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    CreateDir {
        path: PathBuf,
    },
    /// Create a file holding all of `bytes`. Written under `.rdm/tmp/` by the store, so a
    /// half-written one is never visible under a published name and needs no effect of its own.
    Write {
        path: PathBuf,
        bytes: Vec<u8>,
    },
    SyncFile {
        path: PathBuf,
        tier: Durability,
    },
    SyncDir {
        path: PathBuf,
        tier: Durability,
    },
    Rename {
        from: PathBuf,
        to: PathBuf,
    },
    Link {
        original: PathBuf,
        link: PathBuf,
    },
    Remove {
        path: PathBuf,
    },
}

#[cfg(any(test, feature = "test-support"))]
impl Effect {
    /// Apply this effect to a copy of the recorded directory rooted at `dir`. Syncs do nothing:
    /// the copy models the file system as a process kill leaves it, where every completed
    /// effect is visible.
    pub fn apply(&self, dir: &Path) -> io::Result<()> {
        match self {
            Effect::CreateDir { path } => std::fs::create_dir_all(dir.join(path)),
            Effect::Write { path, bytes } => RealFs.write_new(&dir.join(path), bytes),
            Effect::SyncFile { .. } | Effect::SyncDir { .. } => Ok(()),
            Effect::Rename { from, to } => std::fs::rename(dir.join(from), dir.join(to)),
            Effect::Link { original, link } => {
                std::fs::hard_link(dir.join(original), dir.join(link))
            }
            Effect::Remove { path } => std::fs::remove_file(dir.join(path)),
        }
    }
}

/// Apply the first `count` effects of `log` (clamped) to a copy rooted at `dir`.
#[cfg(any(test, feature = "test-support"))]
pub fn replay_prefix(log: &[Effect], count: usize, dir: &Path) -> io::Result<()> {
    for effect in log.iter().take(count) {
        effect.apply(dir)?;
    }
    Ok(())
}

/// Attempts and waits of the held-open-target retry (`store.md` §Windows): 10 attempts,
/// waiting 5, 10, 20, 40, 80 ms and then 100 ms between them.
const RETRY_ATTEMPTS: usize = 10;

fn retry_wait(after_attempt: usize) -> Duration {
    const MS: [u64; 5] = [5, 10, 20, 40, 80];
    Duration::from_millis(MS.get(after_attempt).copied().unwrap_or(100))
}

/// `ERROR_ACCESS_DENIED` (5) and `ERROR_SHARING_VIOLATION` (32): how Windows reports a target
/// another program holds open. Always false elsewhere.
fn is_sharing_violation(err: &io::Error) -> bool {
    cfg!(windows) && is_sharing_code(err)
}

/// The two codes, on any platform (so a unit test can hold them off Windows).
fn is_sharing_code(err: &io::Error) -> bool {
    matches!(err.raw_os_error(), Some(5 | 32))
}

/// Run `op`, retrying while `retryable` says its error is a held-open target.
fn with_retry<T>(
    what: &str,
    path: &Path,
    retryable: impl Fn(&io::Error) -> bool,
    sleep: impl Fn(Duration),
    mut op: impl FnMut() -> io::Result<T>,
) -> io::Result<T> {
    let mut attempt = 0;
    loop {
        match op() {
            Ok(value) => return Ok(value),
            Err(err) if retryable(&err) => {
                attempt += 1;
                if attempt >= RETRY_ATTEMPTS {
                    let code = err
                        .raw_os_error()
                        .map_or(String::new(), |c| format!(" (os error {c})"));
                    return Err(io::Error::new(
                        err.kind(),
                        format!(
                            "{what} {} still failing after {attempt} attempts: {err}{code}",
                            path.display()
                        ),
                    ));
                }
                sleep(retry_wait(attempt - 1));
            }
            Err(err) => return Err(err),
        }
    }
}

/// The real file system.
#[derive(Clone, Copy, Debug, Default)]
pub struct RealFs;

fn sync_handle(file: &std::fs::File, tier: Durability) -> io::Result<()> {
    // std's `sync_all` issues F_FULLFSYNC on macOS, which is the `Full` tier; the `Fsync`
    // tier asks for plain fsync(2) (`store.md` §Durability).
    #[cfg(target_os = "macos")]
    if tier == Durability::Fsync {
        return rustix::fs::fsync(file).map_err(io::Error::from);
    }
    let _ = tier;
    file.sync_all()
}

impl Fs for RealFs {
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn write_new(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(bytes)
    }

    fn sync_file(&self, path: &Path, tier: Durability) -> io::Result<()> {
        if tier == Durability::Process {
            return Ok(());
        }
        let file = std::fs::OpenOptions::new().write(true).open(path)?;
        sync_handle(&file, tier)
    }

    fn sync_dir(&self, path: &Path, tier: Durability) -> io::Result<()> {
        if tier == Durability::Process || cfg!(windows) {
            return Ok(());
        }
        sync_handle(&std::fs::File::open(path)?, tier)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        with_retry(
            "rename to",
            to,
            is_sharing_violation,
            std::thread::sleep,
            || std::fs::rename(from, to),
        )
    }

    fn hard_link(&self, original: &Path, link: &Path) -> io::Result<()> {
        std::fs::hard_link(original, link)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        with_retry(
            "remove",
            path,
            is_sharing_violation,
            std::thread::sleep,
            || std::fs::remove_file(path),
        )
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn list_dir(&self, path: &Path) -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(path)? {
            if let Ok(name) = entry?.file_name().into_string() {
                names.push(name);
            }
        }
        names.sort();
        Ok(names)
    }

    fn stat(&self, path: &Path) -> io::Result<FileStat> {
        let meta = std::fs::metadata(path)?;
        #[cfg(unix)]
        let identity = std::os::unix::fs::MetadataExt::ino(&meta);
        #[cfg(unix)]
        let ctime = Some((
            std::os::unix::fs::MetadataExt::ctime(&meta),
            std::os::unix::fs::MetadataExt::ctime_nsec(&meta),
        ));
        #[cfg(not(unix))]
        let ctime = None;
        #[cfg(not(unix))]
        let identity = meta
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX));
        Ok(FileStat {
            identity,
            mtime: meta.modified()?,
            size: meta.len(),
            ctime,
        })
    }
}

/// A file system that performs every effect on an inner one and logs it, in order, once it
/// has succeeded. Paths must lie under `root` and are logged relative to it. Test support: built
/// for tests and under the `test-support` feature only.
#[cfg(any(test, feature = "test-support"))]
#[derive(Debug)]
pub struct RecordingFs<F = RealFs> {
    inner: F,
    root: PathBuf,
    log: Mutex<Vec<Effect>>,
}

#[cfg(any(test, feature = "test-support"))]
impl RecordingFs<RealFs> {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::over(RealFs, root)
    }
}

#[cfg(any(test, feature = "test-support"))]
impl<F: Fs> RecordingFs<F> {
    pub fn over(inner: F, root: impl Into<PathBuf>) -> Self {
        RecordingFs {
            inner,
            root: root.into(),
            log: Mutex::new(Vec::new()),
        }
    }

    /// A copy of the log so far.
    pub fn effects(&self) -> Vec<Effect> {
        self.log.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn rel(&self, path: &Path) -> io::Result<PathBuf> {
        path.strip_prefix(&self.root)
            .map(Path::to_path_buf)
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "{} is outside the recorded root {}",
                        path.display(),
                        self.root.display()
                    ),
                )
            })
    }

    fn push(&self, effect: Effect) {
        self.log
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(effect);
    }
}

#[cfg(any(test, feature = "test-support"))]
impl<F: Fs> Fs for RecordingFs<F> {
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        let path_rel = self.rel(path)?;
        self.inner.create_dir_all(path)?;
        self.push(Effect::CreateDir { path: path_rel });
        Ok(())
    }

    fn write_new(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let path_rel = self.rel(path)?;
        if let Err(err) = self.inner.write_new(path, bytes) {
            // A failure after the file was created leaves it behind: log what is there, so a
            // later `Remove` of it replays. `AlreadyExists` created nothing.
            if err.kind() != io::ErrorKind::AlreadyExists
                && let Ok(left) = self.inner.read(path)
            {
                self.push(Effect::Write {
                    path: path_rel,
                    bytes: left,
                });
            }
            return Err(err);
        }
        self.push(Effect::Write {
            path: path_rel,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }

    fn sync_file(&self, path: &Path, tier: Durability) -> io::Result<()> {
        let path_rel = self.rel(path)?;
        self.inner.sync_file(path, tier)?;
        self.push(Effect::SyncFile {
            path: path_rel,
            tier,
        });
        Ok(())
    }

    fn sync_dir(&self, path: &Path, tier: Durability) -> io::Result<()> {
        let path_rel = self.rel(path)?;
        self.inner.sync_dir(path, tier)?;
        self.push(Effect::SyncDir {
            path: path_rel,
            tier,
        });
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let (from_rel, to_rel) = (self.rel(from)?, self.rel(to)?);
        self.inner.rename(from, to)?;
        self.push(Effect::Rename {
            from: from_rel,
            to: to_rel,
        });
        Ok(())
    }

    fn hard_link(&self, original: &Path, link: &Path) -> io::Result<()> {
        let (original_rel, link_rel) = (self.rel(original)?, self.rel(link)?);
        self.inner.hard_link(original, link)?;
        self.push(Effect::Link {
            original: original_rel,
            link: link_rel,
        });
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        let path_rel = self.rel(path)?;
        self.inner.remove_file(path)?;
        self.push(Effect::Remove { path: path_rel });
        Ok(())
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner.read(path)
    }

    fn list_dir(&self, path: &Path) -> io::Result<Vec<String>> {
        self.inner.list_dir(path)
    }

    fn stat(&self, path: &Path) -> io::Result<FileStat> {
        self.inner.stat(path)
    }
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A temporary name unique across processes (pid) and within one (counter): the one shape of
/// every file under `.rdm/tmp/`.
fn temp_name() -> String {
    format!(
        "{}-{}.tmp",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// Writes `bytes` to a new file with a fresh [`temp_name`] under `tmp_dir`, creating `tmp_dir` if
/// missing, and returns its path. A name that is taken (`AlreadyExists`) is drawn again; any other
/// failure removes what the write left and is returned. The file is neither synced nor published.
pub(crate) fn stage(fs: &dyn Fs, tmp_dir: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
    fs.create_dir_all(tmp_dir)?;
    loop {
        let candidate = tmp_dir.join(temp_name());
        match fs.write_new(&candidate, bytes) {
            Ok(()) => return Ok(candidate),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => {
                let _ = fs.remove_file(&candidate);
                return Err(err);
            }
        }
    }
}

/// Replace `target` with `bytes`, whole or not at all.
///
/// Effects, in order: create `tmp_dir` if missing, write the temporary file, sync it, rename it
/// over `target`, sync `target`'s directory (and `tmp_dir`, which lost an entry). With
/// `Durability::Process` the syncs are skipped. `tmp_dir` must be on the same file system as
/// `target`; the store passes `.rdm/tmp/`, outside the directories the loader reads. A failure
/// removes the temporary file and leaves `target` as it was.
pub fn atomic_replace(
    fs: &dyn Fs,
    target: &Path,
    tmp_dir: &Path,
    bytes: &[u8],
    tier: Durability,
) -> io::Result<()> {
    let tmp = stage(fs, tmp_dir, bytes)?;
    let published = (|| {
        fs.sync_file(&tmp, tier)?;
        fs.rename(&tmp, target)
    })();
    if let Err(err) = published {
        let _ = fs.remove_file(&tmp);
        return Err(err);
    }
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    fs.sync_dir(parent, tier)?;
    if tmp_dir != parent {
        fs.sync_dir(tmp_dir, tier)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[test]
    fn the_retry_makes_ten_attempts_with_the_documented_waits_and_names_path_and_code() {
        let waits = RefCell::new(Vec::new());
        let calls = Cell::new(0);
        let err = with_retry(
            "rename to",
            Path::new("nodes/n.json"),
            |_| true,
            |d| waits.borrow_mut().push(d.as_millis()),
            || {
                calls.set(calls.get() + 1);
                Err::<(), _>(io::Error::from_raw_os_error(32))
            },
        )
        .unwrap_err();
        assert_eq!(calls.get(), 10);
        assert_eq!(*waits.borrow(), vec![5, 10, 20, 40, 80, 100, 100, 100, 100]);
        assert_eq!(waits.borrow().iter().sum::<u128>(), 555);
        let text = err.to_string();
        assert!(
            text.contains("nodes/n.json") && text.contains("os error 32"),
            "{text}"
        );
    }

    #[test]
    fn a_non_retryable_error_is_returned_after_one_call_and_a_retryable_one_can_recover() {
        let calls = Cell::new(0);
        let err = with_retry(
            "r",
            Path::new("x"),
            |_| false,
            |_| panic!("no wait"),
            || {
                calls.set(calls.get() + 1);
                Err::<(), _>(io::Error::from_raw_os_error(2))
            },
        )
        .unwrap_err();
        assert_eq!((calls.get(), err.raw_os_error()), (1, Some(2)));

        let calls = Cell::new(0);
        let ok = with_retry(
            "r",
            Path::new("x"),
            |e| e.raw_os_error() == Some(5),
            |_| {},
            || {
                calls.set(calls.get() + 1);
                if calls.get() < 3 {
                    Err(io::Error::from_raw_os_error(5))
                } else {
                    Ok(7)
                }
            },
        );
        assert_eq!((ok.unwrap(), calls.get()), (7, 3));
    }

    #[test]
    fn only_access_denied_and_sharing_violation_count_as_a_held_open_target() {
        for (code, retried) in [(5, true), (32, true), (2, false), (33, false), (31, false)] {
            assert_eq!(
                is_sharing_code(&io::Error::from_raw_os_error(code)),
                retried,
                "code {code}"
            );
        }
        assert!(!is_sharing_violation(&io::Error::from_raw_os_error(2)));
        assert_eq!(
            is_sharing_violation(&io::Error::from_raw_os_error(32)),
            cfg!(windows)
        );
    }
}
