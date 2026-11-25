//! The hot cache (`store.md`, §Cache): the decoded files of a campaign kept in memory, one entry
//! per file with the signature it had when it was read, revalidated against the directory on
//! every read and brought up to date by every write.
//!
//! The cache is never the source of truth. Every read starts with a revalidation, so a cached
//! read equals a cold read made at that moment (I10), and every write revalidates once more
//! after it has taken the lock and recovered, so the replay check and the stale check never see
//! a state older than the lock.

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
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
use crate::layout::RecordKind;
use crate::links::ReverseIndex;
use crate::rev;
use crate::store::{OpIndex, Options, State, Store, io_error};
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

/// The entries of one directory, sorted by file name, as the listing is: a revalidation walks the
/// listing and the table together in one pass.
#[derive(Debug)]
struct Table<T, A = ()> {
    rows: Vec<(String, Entry<T, A>)>,
}

impl<T, A> Default for Table<T, A> {
    fn default() -> Self {
        Self { rows: Vec::new() }
    }
}

impl<T, A> Table<T, A> {
    fn find(&self, name: &str) -> Result<usize, usize> {
        self.rows.binary_search_by(|(n, _)| n.as_str().cmp(name))
    }

    fn get(&self, name: &str) -> Option<&Entry<T, A>> {
        self.find(name).ok().map(|i| &self.rows[i].1)
    }

    fn put(&mut self, name: String, entry: Entry<T, A>) {
        match self.find(&name) {
            Ok(i) => self.rows[i].1 = entry,
            Err(i) => self.rows.insert(i, (name, entry)),
        }
    }

    fn take(&mut self, name: &str) -> bool {
        self.find(name).map(|i| self.rows.remove(i)).is_ok()
    }

    fn values(&self) -> impl Iterator<Item = &Entry<T, A>> {
        self.rows.iter().map(|(_, e)| e)
    }
}

/// The entries of the three directories of a campaign, and the generation they belong to.
#[derive(Debug, Default)]
pub(crate) struct Entries {
    nodes: Table<Node, Rev>,
    comments: Table<Comment>,
    amendments: Table<Amendment>,
    /// Bumped whenever any entry changes.
    generation: u64,
    /// The generation the derived views of the store were built from.
    built: u64,
}

/// The file names of a directory that a loader reads: `<id>.json` and no other, sorted.
fn listed(
    fs: &dyn Fs,
    root: &Path,
    record: RecordKind,
    keep: fn(&str) -> bool,
) -> Result<Vec<String>, StoreError> {
    let dir = record.dir();
    match fs.list_dir(&record.dir_path(root)) {
        Ok(mut names) => {
            names.retain(|n| RecordKind::id_of_file(n).is_some_and(keep));
            Ok(names)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(io_error(dir, &e)),
    }
}

/// What a directory of the campaign holds: its kind, which file names it reads, and how a record
/// of it gives its id and what is kept with it.
struct Dir<T, A> {
    record: RecordKind,
    keep: fn(&str) -> bool,
    id_of: fn(&T) -> String,
    aux_of: fn(&T) -> A,
}

const NODES: Dir<Node, Rev> = Dir {
    record: RecordKind::Node,
    keep: NodeId::is_valid,
    id_of: node_id_of,
    aux_of: rev::rev,
};
const COMMENTS: Dir<Comment, ()> = Dir {
    record: RecordKind::Comment,
    keep: CommentId::is_valid,
    id_of: |c| c.id.to_string(),
    aux_of: |_| (),
};
const AMENDMENTS: Dir<Amendment, ()> = Dir {
    record: RecordKind::Amendment,
    keep: AmendmentId::is_valid,
    id_of: |a| a.id.to_string(),
    aux_of: |_| (),
};

/// What looking at one file found.
enum Loaded<T, A> {
    /// The file is gone: listed and then removed or renamed before it was statted or read.
    Gone,
    /// The signature is unchanged and the entry is clean: nothing was read.
    Kept,
    /// The file was read again and holds the bytes the entry holds: only its signature moves.
    Same(FileStat, bool),
    /// A new or changed record.
    New(Entry<T, A>),
}

/// Looks at one file: stat, and read it again unless its entry is clean and its signature
/// unchanged (`force` reads regardless, for the files a write touches). A file that is gone is
/// absent and never an `Io` error (`store.md`, §Cache, revalidation step 4).
fn load_file<T: Stored, A>(
    fs: &dyn Fs,
    path: &Path,
    kind: &Dir<T, A>,
    name: &str,
    prev: Option<&Entry<T, A>>,
    read_at: SystemTime,
    force: bool,
) -> Result<Loaded<T, A>, StoreError> {
    let file = kind.record.logical_of_file(name);
    let stat = match fs.stat(path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Loaded::Gone),
        Err(e) => return Err(io_error(&file, &e)),
    };
    if !force
        && let Some(e) = prev
        && e.stat == stat
        && !e.racy
    {
        return Ok(Loaded::Kept);
    }
    let bytes = match fs.read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Loaded::Gone),
        Err(e) => return Err(io_error(&file, &e)),
    };
    let sum = digest(&bytes);
    let racy = is_racy(stat.mtime, read_at);
    if prev.is_some_and(|e| e.digest == sum) {
        return Ok(Loaded::Same(stat, racy));
    }
    let decoded = decode::<T>(&file, &bytes)?;
    let stem = RecordKind::id_of_file(name).unwrap_or(name);
    let held = (kind.id_of)(&decoded.value);
    if held != stem {
        return Err(StoreError::invalid(
            &file,
            Rule::FileIdMismatch,
            format!("the file is named {stem} and holds the id {held}"),
        ));
    }
    let aux = (kind.aux_of)(&decoded.value);
    Ok(Loaded::New(Entry {
        record: decoded.value,
        aux,
        stat,
        racy,
        digest: sum,
        diags: decoded.diagnostics,
    }))
}

/// Revalidates one directory's entries in one ordered walk over the sorted listing and the sorted
/// table. Returns whether anything changed. A file that does not decode is reported for itself and
/// is not cached: the entry it replaces is dropped so the next validation reads it again.
fn sync_kind<T: Stored, A>(
    fs: &dyn Fs,
    root: &Path,
    kind: &Dir<T, A>,
    read_at: SystemTime,
    table: &mut Table<T, A>,
) -> Result<bool, StoreError> {
    let names = listed(fs, root, kind.record, kind.keep)?;
    let mut path = kind.record.dir_path(root);
    let mut changed = false;
    let mut i = 0;
    for name in &names {
        // Entries sorted before this name are no longer listed.
        let gone = table.rows[i..].partition_point(|(n, _)| n.as_str() < name.as_str());
        if gone > 0 {
            table.rows.drain(i..i + gone);
            changed = true;
        }
        let have = table.rows.get(i).is_some_and(|(n, _)| n == name);
        path.push(name);
        let loaded = load_file(
            fs,
            &path,
            kind,
            name,
            have.then(|| &table.rows[i].1),
            read_at,
            false,
        );
        path.pop();
        match loaded {
            Err(e) => {
                if have {
                    table.rows.remove(i);
                }
                return Err(e);
            }
            Ok(Loaded::Gone) => {
                if have {
                    table.rows.remove(i);
                    changed = true;
                }
            }
            Ok(Loaded::Kept) => i += 1,
            Ok(Loaded::Same(stat, racy)) => {
                let e = &mut table.rows[i].1;
                e.stat = stat;
                e.racy = racy;
                i += 1;
            }
            Ok(Loaded::New(entry)) => {
                if have {
                    table.rows[i].1 = entry;
                } else {
                    table.rows.insert(i, (name.clone(), entry));
                }
                i += 1;
                changed = true;
            }
        }
    }
    if i < table.rows.len() {
        table.rows.truncate(i);
        changed = true;
    }
    Ok(changed)
}

fn node_id_of(n: &Node) -> String {
    n.id().to_string()
}

impl Entries {
    /// Revalidates every entry against the directories: list, stat, read what differs.
    ///
    /// On an error the entries that were read stay, the one that failed is dropped, and the
    /// generation is bumped, so the next call starts from the files again.
    pub(crate) fn revalidate(&mut self, fs: &dyn Fs, root: &Path) -> Result<(), StoreError> {
        // One instant for the whole pass, taken before the first stat.
        let read_at = SystemTime::now();
        let mut changed = false;
        let result = (|| {
            changed |= sync_kind(fs, root, &NODES, read_at, &mut self.nodes)?;
            changed |= sync_kind(fs, root, &COMMENTS, read_at, &mut self.comments)?;
            changed |= sync_kind(fs, root, &AMENDMENTS, read_at, &mut self.amendments)?;
            Ok(())
        })();
        // A failure part-way may have changed entries before it: count them.
        if changed || result.is_err() {
            self.generation += 1;
        }
        result
    }

    /// Reads the node files of `ids` from the disk whatever their entries say: the files a write
    /// touches are judged against the disk and never against a signature (`store.md`, write
    /// algorithm step 3.2). A file that is gone is dropped.
    pub(crate) fn reread_nodes(
        &mut self,
        fs: &dyn Fs,
        root: &Path,
        ids: &BTreeSet<NodeId>,
    ) -> Result<(), StoreError> {
        let read_at = SystemTime::now();
        let mut path = RecordKind::Node.dir_path(root);
        let mut changed = false;
        let mut failed = None;
        for id in ids {
            let name = RecordKind::Node.file_name(id);
            path.push(&name);
            let loaded = load_file(
                fs,
                &path,
                &NODES,
                &name,
                self.nodes.get(&name),
                read_at,
                true,
            );
            path.pop();
            match loaded {
                Err(e) => {
                    self.nodes.take(&name);
                    changed = true;
                    failed = Some(e);
                    break;
                }
                Ok(Loaded::Gone) => changed |= self.nodes.take(&name),
                Ok(Loaded::Kept) => {}
                Ok(Loaded::Same(stat, racy)) => {
                    if let Ok(i) = self.nodes.find(&name) {
                        self.nodes.rows[i].1.stat = stat;
                        self.nodes.rows[i].1.racy = racy;
                    }
                }
                Ok(Loaded::New(entry)) => {
                    self.nodes.put(name, entry);
                    changed = true;
                }
            }
        }
        if changed {
            self.generation += 1;
        }
        failed.map_or(Ok(()), Err)
    }

    /// Whether the derived views lag the entries.
    pub(crate) fn is_stale(&self) -> bool {
        self.generation != self.built
    }

    /// Drops the entry of a path, so the next validation reads it.
    pub(crate) fn forget(&mut self, path: &str) {
        let gone = match RecordKind::from_logical(path) {
            Some((RecordKind::Node, n)) => self.nodes.take(n),
            Some((RecordKind::Comment, n)) => self.comments.take(n),
            Some((RecordKind::Amendment, n)) => self.amendments.take(n),
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
        let name = RecordKind::Node.file_name(&node.id());
        let Some(stat) = self.stat_written(at, RecordKind::Node, &name) else {
            self.nodes.take(&name);
            return false;
        };
        self.nodes.put(
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
        let name = RecordKind::Comment.file_name(&comment.id);
        let Some(stat) = self.stat_written(at, RecordKind::Comment, &name) else {
            self.comments.take(&name);
            return false;
        };
        self.comments.put(
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
        let name = RecordKind::Amendment.file_name(&amendment.id);
        let Some(stat) = self.stat_written(at, RecordKind::Amendment, &name) else {
            self.amendments.take(&name);
            return false;
        };
        self.amendments.put(
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

    fn stat_written(&self, at: &Published<'_>, record: RecordKind, name: &str) -> Option<FileStat> {
        at.fs.stat(&record.dir_path(at.root).join(name)).ok()
    }

    /// Write-through of a deleted node file.
    pub(crate) fn deleted_node(&mut self, id: &NodeId) {
        self.nodes.take(&RecordKind::Node.file_name(id));
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
/// `read` returns what a cold open would return at that moment: the same campaign,
/// revisions, reverse index, diagnostics and op index. A write returns only after its files are
/// published, so a cold reader that starts after the return sees them.
pub struct CachedCampaign {
    store: Store,
}

impl CachedCampaign {
    /// Opens the campaign at `root` and reads every file once.
    ///
    /// # Errors
    /// A per-file fault refuses the open, naming the file: `Invalid` (an unparsable file, a
    /// malformed id, `file_id_mismatch`, a rule of the node itself), `NewerFormat`, or `Io`.
    /// Damage between files loads, and is in [`Store::diagnostics`]. A `root` that is not a
    /// directory is `Io` (file `.`), and nothing is created.
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
    /// cold open would hold now. It takes `&mut self`, so readers of one handle are serialized:
    /// a server shares the handle behind a mutex (a read costs one revalidation, milliseconds). A file that does not decode fails the read, naming that file,
    /// and is not cached: the next read tries it again.
    ///
    /// # Errors
    /// As [`CachedCampaign::open`].
    pub fn read(&mut self) -> Result<&Store, StoreError> {
        self.store.refresh()?;
        Ok(&self.store)
    }

    /// What the last recovery did.
    #[must_use]
    pub fn last_recovered(&self) -> Option<&Recovered> {
        self.store.last_recovered()
    }

    /// Recovery on request: takes the writer lock, removes the stray temporary files, redoes every
    /// intent record left by an interrupted write, and reads the campaign again. Every write does
    /// this on its own on taking the lock; a reader that wants the settled state calls it.
    ///
    /// # Errors
    /// `Locked`, `NewerFormat` for an intent record newer than this binary, `Invalid` when a
    /// create finds other bytes where it must create a file, `Io`.
    pub fn recover(&mut self) -> Result<Recovered, StoreError> {
        self.store.recover()
    }

    /// Replaces one node file if its current `rev` is `expected_rev` (I4). It cannot change
    /// `id`, `name`, `parent_id` or `order`; a content edit needs a note and writes an amendment.
    ///
    /// # Errors
    /// `Stale`, `Locked`, `NotFound`, `Invalid` with the rule that is broken, `Io`.
    pub fn replace(&mut self, req: Request<ReplaceOp>) -> Result<Receipt<Written>, StoreError> {
        self.store.replace(req)
    }

    /// Writes a new node file. The caller minted the ids in `node`; `node.order` is not read, the
    /// store sets it from `position` (default last) and rebalances the siblings when a key grows
    /// past the limit.
    ///
    /// # Errors
    /// `Locked`, `Invalid` with the rule that is broken (`duplicate_id`, `parent_missing`,
    /// `parent_not_directory`, `name_collision`, `root_count`, `root_name_mismatch`,
    /// `unvalidated_node`, `position_not_sibling`, `note_required`, a shape rule), `Io`.
    pub fn create(&mut self, req: Request<CreateOp>) -> Result<Receipt<Written>, StoreError> {
        self.store.create(req)
    }

    /// Renames or moves one node: one node file and one amendment, no id changes.
    ///
    /// # Errors
    /// `Stale`, `Locked`, `NotFound`, `Invalid` (`nothing_to_do`, `parent_missing`,
    /// `parent_not_directory`, `cycle`, `name_collision`, `position_not_sibling`,
    /// `root_delete_or_move`, `unvalidated_node`, `note_required`), `Io`.
    pub fn move_node(&mut self, req: Request<RelocateOp>) -> Result<Receipt<Written>, StoreError> {
        self.store.move_node(req)
    }

    /// Removes a node and every descendant if the `SubtreeRev` matches. The amendment, with the
    /// snapshot of the removed nodes, is written before the first node file is removed.
    ///
    /// # Errors
    /// `Stale`, `Locked`, `NotFound`, `Invalid` (`root_delete_or_move`, `note_required`,
    /// `expected_rev_required`), `Io`.
    pub fn delete_subtree(
        &mut self,
        req: Request<DeleteOp>,
    ) -> Result<Receipt<Deleted>, StoreError> {
        self.store.delete_subtree(req)
    }

    /// Writes one comment file. It has no revision and never conflicts.
    ///
    /// # Errors
    /// `Locked`, `NotFound` for a missing node, `Invalid` (`note_forbidden`), `Io`.
    pub fn comment(&mut self, req: Request<CommentOp>) -> Result<Receipt<Commented>, StoreError> {
        self.store.comment(req)
    }

    /// Applies the ops in order as one transaction: each is checked against the state the
    /// earlier ones produced, the whole result is checked once, and either everything is
    /// written or nothing. `dry_run` runs every check and writes nothing.
    ///
    /// # Errors
    /// The first failing op as `InBatch { index, source }`; `Stale` for a stale
    /// `expected_tree_rev`; `Locked`; `Io`.
    pub fn batch(&mut self, req: Request<BatchOp>) -> Result<Receipt<Vec<OpResult>>, StoreError> {
        self.store.batch(req)
    }
}
