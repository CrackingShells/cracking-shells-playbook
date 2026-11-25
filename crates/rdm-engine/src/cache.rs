//! The hot cache (`store.md`, §Cache): the decoded files of a campaign kept in memory, one entry
//! per file with the signature it had when it was read, revalidated against the directory on
//! every read and brought up to date by every write.
//!
//! The cache is never the source of truth. Every read starts with a revalidation, so a cached
//! read equals a cold read made at that moment (I10), and every write revalidates once more
//! after it has taken the lock and recovered, so the replay check and the stale check never see
//! a state older than the lock.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use rdm_schema::campaign::{Campaign, Diagnostic};
use rdm_schema::error::{Rule, StoreError};
use rdm_schema::ids::{AmendmentId, CommentId, Name, NodeId, Rev};
use rdm_schema::ops::{
    BatchOp, CommentOp, Commented, CreateOp, DeleteOp, Deleted, OpResult, Receipt, RelocateOp,
    ReplaceOp, Request, Written,
};
use rdm_schema::stored::{Amendment, Comment, Node};

use crate::codec::{Stored, decode};
use crate::fs::{FileStat, Fs};
use crate::links::ReverseIndex;
use crate::rev;
use crate::store::{OpIndex, Options, State, Store, invalid, io_error};
use crate::write::Recovered;

/// A file whose modification time is not older than this when it was read is read again on every
/// validation until a read finds it old enough (git's racily-clean rule). It covers FAT (2 s) and
/// HFS+ (1 s); APFS and NTFS are finer, but the rule does not depend on that.
pub(crate) const RACY_WINDOW: Duration = Duration::from_secs(2);

/// Whether a file with this modification time, read at `read_at`, could still be changed again
/// within the same timestamp tick. A modification time in the future is also racy.
fn is_racy(mtime: SystemTime, read_at: SystemTime) -> bool {
    read_at
        .duration_since(mtime)
        .map_or(true, |age| age < RACY_WINDOW)
}

/// A deterministic fingerprint of a file's bytes, to tell a re-read that found nothing new from
/// one that found a change. Not a security boundary.
pub(crate) fn digest(bytes: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    h.write(bytes);
    h.finish()
}

/// One decoded file and what a revalidation compares it with.
#[derive(Debug, Clone)]
pub(crate) struct Entry<T, A = ()> {
    record: T,
    /// What is derived from the record once and kept with it: a node's `rev`.
    aux: A,
    /// The signature statted before the file was read, so a change made during the read shows up
    /// on the next validation.
    stat: FileStat,
    racy: bool,
    digest: u64,
    /// The warnings the codec made reading this file (`unknown_key`).
    diags: Vec<Diagnostic>,
}

/// The entries of the three directories of a campaign, and the generation they belong to.
#[derive(Debug, Default)]
pub(crate) struct Entries {
    nodes: BTreeMap<String, Entry<Node, Rev>>,
    comments: BTreeMap<String, Entry<Comment>>,
    amendments: BTreeMap<String, Entry<Amendment>>,
    /// Bumped whenever any entry changes.
    generation: u64,
    /// The generation the derived views of the store were built from.
    built: u64,
}

/// What kind of record a path of the campaign holds.
enum Kind {
    Node,
    Comment,
    Amendment,
}

fn kind_of(path: &str) -> Option<(Kind, &str)> {
    let (dir, name) = path.split_once('/')?;
    match dir {
        "nodes" => Some((Kind::Node, name)),
        "_comments" => Some((Kind::Comment, name)),
        "_amendments" => Some((Kind::Amendment, name)),
        _ => None,
    }
}

/// The file names of a directory that a loader reads: `<id>.json` and no other.
fn listed(
    fs: &dyn Fs,
    root: &Path,
    dir: &str,
    keep: impl Fn(&str) -> bool,
) -> Result<Vec<String>, StoreError> {
    match fs.list_dir(&root.join(dir)) {
        Ok(names) => Ok(names
            .into_iter()
            .filter(|n| n.strip_suffix(".json").is_some_and(&keep))
            .collect()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(io_error(dir, &e)),
    }
}

/// Revalidates one directory's entries. Returns whether anything changed.
fn sync_kind<T: Stored, A>(
    fs: &dyn Fs,
    root: &Path,
    dir: &str,
    keep: impl Fn(&str) -> bool,
    id_of: impl Fn(&T) -> String,
    aux_of: impl Fn(&T) -> A,
    map: &mut BTreeMap<String, Entry<T, A>>,
) -> Result<bool, StoreError> {
    let mut changed = false;
    let names = listed(fs, root, dir, keep)?;
    // Entries whose file is no longer listed are dropped.
    let before = map.len();
    map.retain(|name, _| names.binary_search(name).is_ok());
    changed |= map.len() != before;
    for name in names {
        let path = root.join(dir).join(&name);
        // The instant is taken before the stat, so an entry is racy if it could have been
        // modified again after the read within the same tick.
        let read_at = SystemTime::now();
        let stat = match fs.stat(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                changed |= map.remove(&name).is_some();
                continue;
            }
            Err(e) => return Err(io_error(&format!("{dir}/{name}"), &e)),
        };
        if let Some(e) = map.get(&name)
            && e.stat == stat
            && !e.racy
        {
            continue;
        }
        let file = format!("{dir}/{name}");
        let bytes = match fs.read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                changed |= map.remove(&name).is_some();
                continue;
            }
            Err(e) => {
                map.remove(&name);
                return Err(io_error(&file, &e));
            }
        };
        let sum = digest(&bytes);
        let racy = is_racy(stat.mtime, read_at);
        if let Some(e) = map.get_mut(&name)
            && e.digest == sum
        {
            // Read again and found the same bytes: only the signature moves.
            e.stat = stat;
            e.racy = racy;
            continue;
        }
        // A file that does not decode is reported for itself and is not cached: the entry it
        // replaces is dropped so the next validation reads it again.
        let decoded = match decode::<T>(&file, &bytes) {
            Ok(d) => d,
            Err(e) => {
                map.remove(&name);
                return Err(e);
            }
        };
        let stem = name.strip_suffix(".json").unwrap_or(&name);
        let held = id_of(&decoded.value);
        if held != stem {
            map.remove(&name);
            return Err(invalid(
                &file,
                Rule::FileIdMismatch,
                format!("the file is named {stem} and holds the id {held}"),
            ));
        }
        let aux = aux_of(&decoded.value);
        map.insert(
            name,
            Entry {
                record: decoded.value,
                aux,
                stat,
                racy,
                digest: sum,
                diags: decoded.diagnostics,
            },
        );
        changed = true;
    }
    Ok(changed)
}

impl Entries {
    /// Revalidates every entry against the directories: list, stat, read what differs.
    ///
    /// On an error the entries that were read stay, the one that failed is dropped, and the
    /// generation is bumped if anything changed, so the next call starts from the files again.
    pub(crate) fn revalidate(&mut self, fs: &dyn Fs, root: &Path) -> Result<(), StoreError> {
        let mut changed = false;
        let result = (|| {
            changed |= sync_kind(
                fs,
                root,
                "nodes",
                |s| NodeId::parse(s).is_ok(),
                |n: &Node| n.id().to_string(),
                rev::rev,
                &mut self.nodes,
            )?;
            changed |= sync_kind(
                fs,
                root,
                "_comments",
                |s| CommentId::parse(s).is_ok(),
                |c: &Comment| c.id.to_string(),
                |_| (),
                &mut self.comments,
            )?;
            changed |= sync_kind(
                fs,
                root,
                "_amendments",
                |s| AmendmentId::parse(s).is_ok(),
                |a: &Amendment| a.id.to_string(),
                |_| (),
                &mut self.amendments,
            )?;
            Ok(())
        })();
        // A failure part-way may have changed entries before it: count them.
        if changed || result.is_err() {
            self.generation += 1;
        }
        result
    }

    /// Whether the derived views lag the entries.
    pub(crate) fn is_stale(&self) -> bool {
        self.generation != self.built
    }

    /// Drops the entry of a path, so the next validation reads it.
    pub(crate) fn forget(&mut self, path: &str) {
        let gone = match kind_of(path) {
            Some((Kind::Node, n)) => self.nodes.remove(n).is_some(),
            Some((Kind::Comment, n)) => self.comments.remove(n).is_some(),
            Some((Kind::Amendment, n)) => self.amendments.remove(n).is_some(),
            None => false,
        };
        if gone {
            self.generation += 1;
        }
    }

    /// Builds the derived views from the entries: the campaign, its revisions, the reverse index,
    /// the op index and the diagnostics of the files.
    pub(crate) fn build(&mut self, slug: &Name) -> Result<State, StoreError> {
        let opened = Campaign::open(
            self.nodes
                .values()
                .map(|e| e.record.clone())
                .collect::<Vec<_>>(),
            slug,
        )?;
        let campaign = opened.campaign;
        let revs = self
            .nodes
            .values()
            .map(|e| (e.record.id().clone(), e.aux.clone()))
            .collect();
        let comments: BTreeMap<CommentId, Comment> = self
            .comments
            .values()
            .map(|e| (e.record.id.clone(), e.record.clone()))
            .collect();
        let amendments: BTreeMap<AmendmentId, Amendment> = self
            .amendments
            .values()
            .map(|e| (e.record.id.clone(), e.record.clone()))
            .collect();
        let mut file_diags: BTreeMap<String, Vec<Diagnostic>> = BTreeMap::new();
        let all = self
            .nodes
            .values()
            .flat_map(|e| &e.diags)
            .chain(self.comments.values().flat_map(|e| &e.diags))
            .chain(self.amendments.values().flat_map(|e| &e.diags));
        for d in all {
            file_diags
                .entry(d.file.clone())
                .or_default()
                .push(d.clone());
        }
        let reverse = ReverseIndex::build(
            &campaign,
            &comments.values().cloned().collect::<Vec<_>>(),
            &amendments.values().cloned().collect::<Vec<_>>(),
        );
        let mut ops = OpIndex::default();
        amendments.values().for_each(|a| ops.add_amendment(a));
        comments.values().for_each(|c| ops.add_comment(c));
        campaign
            .nodes()
            .for_each(|n| ops.replace_node(n.id(), Some(n)));
        self.built = self.generation;
        Ok(State {
            campaign,
            comments,
            amendments,
            revs,
            file_diags,
            reverse,
            ops,
        })
    }

    /// Write-through of a node file the writer has published. `read_at` is the instant taken
    /// before the write began, so the entry is racy and is read once more within the window.
    /// Returns whether the file could be statted.
    pub(crate) fn wrote_node(
        &mut self,
        at: &Published<'_>,
        node: &Node,
        text: &str,
        rev: Rev,
        diags: Vec<Diagnostic>,
    ) -> bool {
        let name = format!("{}.json", node.id());
        let Some(stat) = self.stat_written(at, "nodes", &name) else {
            self.nodes.remove(&name);
            return false;
        };
        self.nodes.insert(
            name,
            Entry {
                record: node.clone(),
                aux: rev,
                racy: is_racy(stat.mtime, at.began),
                stat,
                digest: digest(text.as_bytes()),
                diags,
            },
        );
        true
    }

    /// Write-through of a comment file.
    pub(crate) fn wrote_comment(
        &mut self,
        at: &Published<'_>,
        comment: &Comment,
        text: &str,
    ) -> bool {
        let name = format!("{}.json", comment.id);
        let Some(stat) = self.stat_written(at, "_comments", &name) else {
            self.comments.remove(&name);
            return false;
        };
        self.comments.insert(
            name,
            Entry {
                record: comment.clone(),
                aux: (),
                racy: is_racy(stat.mtime, at.began),
                stat,
                digest: digest(text.as_bytes()),
                diags: Vec::new(),
            },
        );
        true
    }

    /// Write-through of an amendment file.
    pub(crate) fn wrote_amendment(
        &mut self,
        at: &Published<'_>,
        amendment: &Amendment,
        text: &str,
    ) -> bool {
        let name = format!("{}.json", amendment.id);
        let Some(stat) = self.stat_written(at, "_amendments", &name) else {
            self.amendments.remove(&name);
            return false;
        };
        self.amendments.insert(
            name,
            Entry {
                record: amendment.clone(),
                aux: (),
                racy: is_racy(stat.mtime, at.began),
                stat,
                digest: digest(text.as_bytes()),
                diags: Vec::new(),
            },
        );
        true
    }

    fn stat_written(&self, at: &Published<'_>, dir: &str, name: &str) -> Option<FileStat> {
        at.fs.stat(&at.root.join(dir).join(name)).ok()
    }

    /// Write-through of a deleted node file.
    pub(crate) fn deleted_node(&mut self, id: &NodeId) {
        self.nodes.remove(&format!("{id}.json"));
    }

    /// The derived views were updated in step with the entries: they belong to the generation.
    /// With `false` (a file could not be statted) they are left stale and rebuilt next time.
    pub(crate) fn settle(&mut self, in_step: bool) {
        self.generation += 1;
        if in_step {
            self.built = self.generation;
        }
    }
}

/// Where a write-through stats the files it published, and the instant the entries it makes
/// count as read at: taken before the write began, so each entry is racy.
pub(crate) struct Published<'a> {
    pub(crate) fs: &'a dyn Fs,
    pub(crate) root: &'a Path,
    pub(crate) began: SystemTime,
}

/// A campaign held in memory by a long-lived process: reads are served from decoded entries
/// revalidated against the files, writes go through to the files and into the entries.
///
/// `read` returns what a fresh [`Store::open`] would return at that moment: the same campaign,
/// revisions, reverse index, diagnostics and op index. A write returns only after its files are
/// published, so a cold reader that starts after the return sees them.
pub struct CachedCampaign {
    store: Store,
}

impl CachedCampaign {
    /// Opens the campaign at `root` and reads every file once.
    ///
    /// # Errors
    /// As [`Store::open`].
    pub fn open(
        root: impl Into<PathBuf>,
        slug: &Name,
        options: Options,
    ) -> Result<Self, StoreError> {
        Ok(Self {
            store: Store::open(root, slug, options)?,
        })
    }

    /// Revalidates every entry against the files and returns the store, which then holds what a
    /// cold open would hold now. A file that does not decode fails the read, naming that file,
    /// and is not cached: the next read tries it again.
    ///
    /// # Errors
    /// As [`Store::open`].
    pub fn read(&mut self) -> Result<&Store, StoreError> {
        self.store.refresh()?;
        Ok(&self.store)
    }

    /// What the last recovery did.
    #[must_use]
    pub fn last_recovered(&self) -> Option<&Recovered> {
        self.store.last_recovered()
    }

    /// As [`Store::recover`].
    ///
    /// # Errors
    /// As [`Store::recover`].
    pub fn recover(&mut self) -> Result<Recovered, StoreError> {
        self.store.recover()
    }

    /// As [`Store::replace`].
    ///
    /// # Errors
    /// As [`Store::replace`].
    pub fn replace(&mut self, req: Request<ReplaceOp>) -> Result<Receipt<Written>, StoreError> {
        self.store.replace(req)
    }

    /// As [`Store::create`].
    ///
    /// # Errors
    /// As [`Store::create`].
    pub fn create(&mut self, req: Request<CreateOp>) -> Result<Receipt<Written>, StoreError> {
        self.store.create(req)
    }

    /// As [`Store::move_node`].
    ///
    /// # Errors
    /// As [`Store::move_node`].
    pub fn move_node(&mut self, req: Request<RelocateOp>) -> Result<Receipt<Written>, StoreError> {
        self.store.move_node(req)
    }

    /// As [`Store::delete_subtree`].
    ///
    /// # Errors
    /// As [`Store::delete_subtree`].
    pub fn delete_subtree(
        &mut self,
        req: Request<DeleteOp>,
    ) -> Result<Receipt<Deleted>, StoreError> {
        self.store.delete_subtree(req)
    }

    /// As [`Store::comment`].
    ///
    /// # Errors
    /// As [`Store::comment`].
    pub fn comment(&mut self, req: Request<CommentOp>) -> Result<Receipt<Commented>, StoreError> {
        self.store.comment(req)
    }

    /// As [`Store::batch`].
    ///
    /// # Errors
    /// As [`Store::batch`].
    pub fn batch(&mut self, req: Request<BatchOp>) -> Result<Receipt<Vec<OpResult>>, StoreError> {
        self.store.batch(req)
    }
}
