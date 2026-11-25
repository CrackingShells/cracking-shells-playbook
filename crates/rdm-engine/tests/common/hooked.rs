//! A file system the real one cannot be made to be on cue: the real `RealFs` with two hooks. One
//! runs before a stat or a read (a concurrent actor acting between the listing and the access),
//! one rewrites what a stat reports (a file system whose signature hides a change).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rdm_engine::fs::{Durability, FileStat, Fs, RealFs};
use rdm_engine::store::Options;

#[derive(Clone, Copy, PartialEq)]
pub enum At {
    Stat,
    Read,
    /// Before a hard link is made, with the path of the new name.
    Link,
}

type Before = Box<dyn Fn(At, &Path) + Send + Sync>;
type Report = Box<dyn Fn(&Path, FileStat) -> FileStat + Send + Sync>;

pub struct Hooked {
    before: Before,
    report: Report,
    /// Whether `hard_link` fails with `Unsupported`, as on a volume without hard links.
    refuse_links: bool,
}

impl Hooked {
    fn new(before: Before, report: Report) -> Arc<Self> {
        Arc::new(Self {
            before,
            report,
            refuse_links: false,
        })
    }
}

/// A file system that writes `bytes` at `target` once, just before the next hard link to it
/// after the returned flag is set: an external writer taking the name between the store's check
/// and its publish. With `refuse_links` the volume has no hard links, so the link then fails.
pub fn race_at_link(
    refuse_links: bool,
    target: &Path,
    bytes: &[u8],
) -> (Arc<Hooked>, Arc<AtomicBool>) {
    let armed = Arc::new(AtomicBool::new(false));
    let (flag, target, bytes) = (Arc::clone(&armed), target.to_path_buf(), bytes.to_vec());
    let fs = Arc::new(Hooked {
        before: Box::new(move |when, p| {
            if when == At::Link && p == target && flag.swap(false, Ordering::SeqCst) {
                std::fs::write(p, &bytes).unwrap();
            }
        }),
        report: Box::new(|_, s| s),
        refuse_links,
    });
    (fs, armed)
}

/// A file system on a volume that refuses hard links.
pub fn no_hard_links() -> Arc<Hooked> {
    Arc::new(Hooked {
        before: Box::new(|_, _| {}),
        report: Box::new(|_, s| s),
        refuse_links: true,
    })
}

/// Store options over `fs`.
pub fn options_with(fs: Arc<Hooked>) -> Options {
    Options {
        fs,
        ..Options::default()
    }
}

/// A file system whose stat of a path always reports what it first saw: the signature of a
/// coarse-timestamp file system after a same-size in-place edit, which hides the edit from every
/// read that trusts the signature.
pub fn frozen_stat() -> Arc<Hooked> {
    let first_seen = Mutex::new(BTreeMap::<PathBuf, FileStat>::new());
    Hooked::new(
        Box::new(|_, _| {}),
        Box::new(move |p, s| {
            *first_seen
                .lock()
                .unwrap()
                .entry(p.to_path_buf())
                .or_insert(s)
        }),
    )
}

/// A file system that removes `path` once, at the next `at` of it after the returned flag is set:
/// a `git checkout` or an editor's rename-save acting after the listing named the file.
pub fn vanish_at(at: At, path: &Path) -> (Arc<Hooked>, Arc<AtomicBool>) {
    let armed = Arc::new(AtomicBool::new(false));
    let (flag, target) = (Arc::clone(&armed), path.to_path_buf());
    let fs = Hooked::new(
        Box::new(move |when, p| {
            if when == at && p == target && flag.swap(false, Ordering::SeqCst) {
                std::fs::remove_file(p).unwrap();
            }
        }),
        Box::new(|_, s| s),
    );
    (fs, armed)
}

impl Fs for Hooked {
    fn create_dir_all(&self, p: &Path) -> std::io::Result<()> {
        RealFs.create_dir_all(p)
    }
    fn write_new(&self, p: &Path, b: &[u8]) -> std::io::Result<()> {
        RealFs.write_new(p, b)
    }
    fn sync_file(&self, p: &Path, t: Durability) -> std::io::Result<()> {
        RealFs.sync_file(p, t)
    }
    fn sync_dir(&self, p: &Path, t: Durability) -> std::io::Result<()> {
        RealFs.sync_dir(p, t)
    }
    fn rename(&self, a: &Path, b: &Path) -> std::io::Result<()> {
        RealFs.rename(a, b)
    }
    fn hard_link(&self, a: &Path, b: &Path) -> std::io::Result<()> {
        (self.before)(At::Link, b);
        if self.refuse_links {
            return Err(std::io::ErrorKind::Unsupported.into());
        }
        RealFs.hard_link(a, b)
    }
    fn remove_file(&self, p: &Path) -> std::io::Result<()> {
        RealFs.remove_file(p)
    }
    fn read(&self, p: &Path) -> std::io::Result<Vec<u8>> {
        (self.before)(At::Read, p);
        RealFs.read(p)
    }
    fn list_dir(&self, p: &Path) -> std::io::Result<Vec<String>> {
        RealFs.list_dir(p)
    }
    fn stat(&self, p: &Path) -> std::io::Result<FileStat> {
        (self.before)(At::Stat, p);
        RealFs.stat(p).map(|s| (self.report)(p, s))
    }
}
