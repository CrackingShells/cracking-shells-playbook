# rdm — Store

How a campaign lives on disk and how it is read, written, recovered and cached. It replaces
R01's `§Concurrency` (lock, staging, journal) and the "names are paths" layout of `§Data Model`
(`architecture.md`), under the rulings of the A2 gap analysis (2026-11-25).

What this document owns: the on-disk layout, the id string format and minting, the order key, `rev`
and `tree_rev`, the writer lock and its holder record, the write order of each primitive, the
intent and recovery, durability, cache revalidation and Windows. What it does not own, and only
references: the Rust types, the id newtypes and the record inventory, the operation requests and
results, the error type, declared links, `MOVES` and the version chain, all in
[contracts.md](contracts.md). Errors are named by their variant (`Stale { current }`,
`Locked { holder }`, `NewerFormat { found, supported }`, `Invalid`). The stored fields are in
[stored-schema.md](stored-schema.md), the meaning of `rev` and `tree_rev` for consumers in
[read-model.md](read-model.md).

Rulings this document builds on and does not reopen: flat `nodes/<id>.json` holding `name`,
`parent_id` and an order key, depth computed from `parent_id`; readers take no lock; Rust types are
the contract and JSON is derived; a file newer than the binary is refused with `NewerFormat` (any higher part of the `0.X.Y` triple);
declared links; tracked data stays text; the cache is write-through, revalidated on every read,
never the source of truth.

Where a claim is this design's own and not a source's, it says so. Guard names are those of the
[campaign README](../../__roadmap__/m1-core-foundation/README.md).

## Layout

```
__roadmap__/<campaign>/
  nodes/<id>.json            every node: the campaign root, directories and leaves (tracked)
  _amendments/<ulid>.json    one file per amendment, append-only (tracked)
  _comments/<ulid>.json      one file per comment, append-only (tracked)
  .rdm/                      untracked, created by the store
    .gitignore               holds `*`, so the repository's own .gitignore needs no entry
    lock                     the writer lock; empty, never read or written
    holder.json              advisory record of the lock holder
    tmp/                     files being written, before they are published
    intent/<ulid>.json       redo records of operations in flight
    cache/                   optional persisted cache (the hot_cache leaf decides if any)
```

A node is one file, whatever its kind. The tree is not in the file system: a node's place is its
`parent_id`, its place among siblings its order key.

| Stored in the node file | Rule |
|:--|:--|
| `name` | `^[a-z][a-z0-9_-]*$`, unique among the children of one parent. On the root it is the campaign slug, which `open` is given by its caller: the root is the parentless node with that name |
| `parent_id` | The `id` of the parent node. Absent on the root and only there |
| `order` | The sibling order key (Order key). Every node but the root has one; the root has none |

| Derived on every read, never stored | From |
|:--|:--|
| Path (`a/b/c`) | The `name`s along the `parent_id` chain |
| Depth (`level`) | The length of that chain: the one ordering rule, I1 |
| Order of siblings | `(order, id)`, both compared bytewise, ascending |

**What the loader reads.** In `nodes/` only entries named `n_` followed by eight characters of the id
alphabet and `.json`; in `_amendments/` and `_comments/` only 26-character ULIDs and `.json`. Other
names are ignored, so an editor's backup file or a stray temporary cannot be loaded as a node. The
store's own temporary files are not in these directories at all: they live in `.rdm/tmp/`. That keeps
them out of `git status` as well, and `nodes/` changes only when a node is published.

**Why this layout.** Each fact a writer changes is one small file, so git merges two branches that
edit different nodes without a conflict, and a rename, a move or an insert among siblings writes
one file. No directory is ever renamed, so R01's risk "directory renames fail on Windows while a
handle is open" disappears. A path is `nodes/n_xxxxxxxx.json` below the campaign directory
whatever the depth of the tree, so the Windows 260-character limit no longer depends on nesting.
A node's `name` can be any valid slug, including `con` or `nul`, because it is no longer a file name.

**Faults and merge damage: report, do not refuse.** Git merges files, not the invariants between
them, so a merge can leave a campaign that no single writer would have produced. The rule of `open`
(R3 of Eliott's review, 2026-11-25) separates two classes.

- **Per-file faults fail `open`**, naming the file: an unparsable file, a malformed id, `file_id_mismatch`, a file newer than the binary (`NewerFormat`). One file cannot be loaded, so nothing can be said of the campaign.
- **Cross-file merge damage is loaded and reported loudly**, as one diagnostic per defect naming the file or files and the rule. The rule codes are contracts.md's.

| Defect | Rule | Produced by |
|:--|:--|:--|
| A `parent_id` naming no node (orphan) | `parent_missing` | One branch deletes a node while another adds or moves a child under it |
| A `parent_id` naming a leaf | `parent_not_directory` | One branch replaces a directory by a leaf of the same id while another adds a child under it |
| A cycle in the `parent_id` chain | `cycle` | Two branches each move one of two nodes under the other |
| No parentless node carries the slug `open` was given, or more than one does | `root_count` | A merge of unrelated histories |
| A parentless node whose `name` is not the slug | `root_name_mismatch` | A campaign directory renamed by hand; the repair is a rename of the root through the store |
| Two siblings with one `name` | `name_collision` | Two branches each add a node of that name under one parent. The two files differ, so git sees no conflict |
| Two files with one id, across files | `duplicate_id` | Minting collided across branches: the same path, so git reports an add/add conflict, but a hand-copied file passes unseen |

**The anchor.** The root is the node without `parent_id` whose `name` equals the slug the caller gave
`open`. Nodes reachable from it by `parent_id` are **validated**; everything else (an orphan, a cycle, a
second parentless node, a root whose name does not match) is loaded but **unvalidated**. A campaign
with damage is therefore readable, which is what a person repairing it needs.

**What a write may build on.** A write that builds on an unvalidated node (it names one, or places
something under it) is refused with `Invalid`, rule `unvalidated_node`. A repair write is
allowed: relocating a node into the validated tree, renaming a colliding sibling or the root, or deleting.
A comment is allowed too: it is append-only and points at the id, not the position (contracts.md §Campaign model). The
check sits in step 3 of the write algorithm, under the lock, on the files as they are then.

These are consequences of the flat-layout ruling, not objections to it. Guarded by
`open_names_invalid_campaigns` (S20) and `requests_breaking_a_rule_are_refused` (S22).

## Order key

**Technique:** fractional indexing (Figma; Greenspan's midpoint algorithm as in
`rocicorp/fractional-indexing`), in the form below. What the sources support: Sources.

- **Alphabet.** The 62 digits `0-9A-Za-z`, in ascending ASCII order, value = position. A key is a non-empty string of digits that does not end in `0`, read as a base-62 fraction between 0 and 1. Keys compare with plain bytewise comparison, never locale order. `validate` rejects any other string.
- **A first key** is `V`, the middle digit.
- **A key between `a` and `b`** (`a < b`; `none` stands for 0 as a lower bound and for 1 as an upper bound):
  1. Strip the common prefix (`a` is read as padded with `0`), apply the rules below to what is left, and put the prefix back.
  2. If the first digits differ by more than one, take the middle digit (rounded down).
  3. If they are adjacent and `b` has more than one digit, take `b`'s first digit alone: it lies between because `b` has a non-zero tail.
  4. Otherwise take `a`'s first digit followed by a key between the rest of `a` and `none`.

  | between `a` and `b` | key |
  |:--|:--|
  | `none`, `none` | `V` |
  | `V`, `none` | `k` |
  | `none`, `V` | `F` |
  | `V`, `o` | `e` |
  | `V`, `W` | `VV` |
  | `V`, `VV` | `VF` |
- **An insert** between neighbours `a` and `b` writes one file, the inserted node, and nothing else. Two branches that insert at the same spot produce equal keys; the order is then `(order, id)`, so every reader agrees without jitter, and jitter would only lengthen keys. The root has no key.
- **Length stays short.** Each insert into one gap halves it, so a run of inserts at one spot, or appends at the end, adds about one character per six inserts. Measured on a model of this algorithm: after 10, 50, 100 and 200 appends the key is 2, 9, 17 and 34 characters; after 200 inserts at one spot, 41. Bulk creation (an import, the 160-node benchmark) uses the recursive form that places `n` keys at once: 160 keys are at most 2 characters, and so are 1 000.
- **Rebalance.** A key longer than 24 characters (about 130 inserts at one spot, far beyond the size of a directory in a real campaign) makes the insert rebalance its siblings: the same operation rewrites all `n` order keys evenly with the bulk form, as one batch, and its amendment records each as a `set` of `/order`. This rule is this design's own; neither source names one. Only the nodes whose key changes get a new `rev`.
- Guarded by `order_keys_sort_strictly_between_their_bounds` (a property test over random bounds, in `campaign_store`).

Sibling order is therefore stored. This supersedes the line of `read-model.md` that says siblings have
no stored order and are listed by `level`, leaves before directories, then `path` (reported as a finding).

## Ids

| Kind | Prefix | Body | Example | Unique within |
|:--|:--|:--|:--|:--|
| Node (root, directory, leaf) | `n_` | 8 characters | `n_7k3q9x2m` | the campaign (its file name) |
| Criterion | `c_` | 6 characters | `c_8m2xqv` | its node file |
| Step | `s_` | 6 characters | `s_0f4tzn` | its node file |
| Amendment, comment | none | a ULID, 26 characters | `01J9ZQ3M8R5K7V2N4X6Y8A0B1C` | the campaign; the ULID is the order |

Checks and references carry no id under the proposed inventory (`contracts.md` fixes the inventory;
the table gains one row per kind that gets an id).

- **Alphabet.** Crockford base32 in lower case, `0123456789abcdefghjkmnpqrstvwxyz`: no `i`, `l`, `o` or `u`, so no id is misread as a number. Lower case because the default macOS and Windows file systems are case-insensitive and a node's id is its file name; mixed-case ids could collide there and not on Linux. The ULIDs of amendments and comments stay in the canonical upper case that the stored schema already fixes; their order is time order.
- **Why random.** A counter collides across worktrees (two branches mint `n_0042`). A hash of content changes with the content, and I7 says an id never changes. A UUID is 36 characters for no gain. A random body needs no coordination.
- **Why this length.** Eight characters are 40 bits. With 2 000 nodes present, a new mint collides with probability 2 000 / 2^40 = 1.8e-9. Two branches that each mint 500 ids share one with probability about 500^2 / 2^40 = 2.3e-7, and that case is loud, not silent: the same id is the same path, which git reports as an add/add conflict. Six characters (30 bits) suffice for sub-items, which only have to be unique inside one file of a few dozen of them: 50 / 2^30 = 4.7e-8 per mint.
- **Minting.** The caller mints ids, outside the lock, with the engine's minter (contracts.md); the store never mints a node, criterion or step id, which is what lets a batch name a node it creates and a retried request carry the same ids. The minter draws the body from an RNG injected at construction (seeded in tests, the operating system's generator otherwise). Each character takes five bits of a random byte, which is exactly uniform because 256 is a multiple of 32. It checks the candidate against the ids its caller can see: the campaign as last read, and the ids the same batch has already minted. On a collision it draws again; after 32 collisions in a row it fails (a real generator never gets there, and a test generator that always collides must still end). Guarded by `minting_retries_on_collision`.
- **The store refuses a collision it finds.** Between the caller's read and the write, another process can have created the same id. Under the lock the store checks every id the request introduces against the disk and refuses a collision with `Invalid`, rule `duplicate_id`; the caller mints again and retries. Guarded by `open_names_invalid_campaigns` for the across-files case; the store's own refusal under the lock has no separate guard (S8).
- **Never reused.** The check sees live ids only. A deleted node's id could be minted again with probability under 2e-9 per mint, and a reference to the deleted id would then resolve to the new node instead of the amendment's snapshot. This residual risk is accepted and not tested: scanning every amendment on each mint would cost more than the risk is worth.

## Revisions

| | Definition |
|:--|:--|
| `rev` of a node | The first 16 hexadecimal characters (64 bits) of SHA-256 over the canonical encoding of the node as decoded, with `schema_version` left out |
| `tree_rev` | The first 16 hexadecimal characters of SHA-256 over the text `rdm-tree\n` followed by one line `<id> <rev>\n` per node, in ascending bytewise order of `id` |
| `SubtreeRev` | The same form with its own domain prefix `rdm-subtree\n`, over the `<id> <rev>\n` lines of a node and all its descendants, sorted by `id`. It is `DeleteOp.expected_subtree_rev` |

- **What `rev` counts:** every field of the node file in canonical order, including `name`, `parent_id` and `order`, and the unknown fields the file carries (I12), in their order. Not counted: the file's bytes (whitespace, CRLF, `\u` escapes), its mtime, the `schema_version` stamp, amendments and comments. The hash is taken over what the serialiser would write for the decoded value, so a whitespace-only edit keeps the `rev` and a change of any value changes it. A number keeps its JSON form where no type fixes it: a typed integer field refuses `1e5`, while in an unknown key `1e5` re-encodes as `100000.0` and so has another `rev` than `100000` (I12 keeps unknown values as written, so they are not normalised). Guarded by `rev_tracks_value_not_formatting`.
- **`SubtreeRev`** changes when any node in the subtree is created, removed, moved into or out of it, or edited. A move of the subtree's root changes that node's `rev` and so its `SubtreeRev`; the subtree's descendants keep their `rev`s. The distinct prefix keeps a subtree whose lines happen to equal the whole campaign's from sharing its `tree_rev`.
- **Consequences.** A move changes the `rev` of the moved node only: its children keep theirs because they name their parent by id. A comment or an amendment changes no `rev` and not the `tree_rev`, because `tree_rev` covers `nodes/` only. Creating, deleting, moving or editing any node, a status move included, changes `tree_rev`. It is deliberately coarse: a batch that presents `expected_tree_rev` is refused when any node changed anywhere, which is what a structural batch wants and a field edit does not use. Guarded by `tree_rev_changes_on_node_ops_not_comments`.
- **Why SHA-256.** It is a published function with an independent implementation in every language, so another tool (the importer, a CI script) can recompute a `rev` from a file. It needs no C build (`blake3` compiles C and assembly by default and declares an MSRV of exactly 1.88). A node is a few kilobytes, so speed does not matter, and an unchanged node is not re-hashed (see Cache).
- **Why 64 bits.** `rev` is compared only with the value the same node had when the caller read it. A different value with the same `rev` would let one stale write through, with probability 2^-64. Sixteen characters keep the opaque string short in every tool output. It is not a security boundary.
- **A function of the value and of the current types.** The decoder upgrades an older file through the version chain before the encoder runs, so one old file read by two binaries with different layers can have two `rev`s. A consumer compares `rev`s it got from one server, and the `rev` of an unmodified file is stable for a given binary.
- **Opaque to consumers** (`read-model.md`): they send it back as `expected_rev` and never parse it.

## Reads

Readers take no lock and write nothing (I11).

1. List `nodes/` and read every matching file whole (the cache turns this into a revalidation, see Cache).
2. Decode each file through the version chain. A file whose `schema_version` is newer than the binary's, whether the higher part is X or Y of `0.X.Y` (the whole triple is compared), fails with `NewerFormat { found, supported, .. }` before anything else in it is parsed.
3. A key the decoder does not know is kept in place (I12) and produces a warning, a `Diagnostic` naming the file and the key path with the rule `unknown_key` (the type is contracts.md's). A read reports the diagnostics of the files it read.
4. Faults and cross-file damage follow the rule of Layout: per-file faults fail `open`; merge damage is loaded and reported with its diagnostics.

**Technique:** publish by rename (Maildir delivery, Sources). Every file a writer publishes is written
completely under another name first and then renamed into place, so a reader opening a path gets the
old file or the new one, never a torn one. Guarded by `reader_sees_old_or_new_never_partial`.

What a reader is **not** given is a snapshot across files. During a multi-file operation it can read
some files before and some after. The write order below makes every such mixture a valid campaign
(no node names a missing parent, no later step is needed to make the files parse), and that is all it
promises: the guard is "every prefix loads" (`every_write_prefix_loads_and_recovers_to_pre_or_post`),
not isolation. A reader never recovers a crashed operation; recovery belongs to the next writer.
`rev` and `tree_rev` come from the files actually read, so a mixture yields a `tree_rev` that no
committed state has. A write that presents it is judged under the lock against the files as they are
then (Write algorithm), so such a read cannot cause a lost update.

A file an external program is rewriting in place (an editor that truncates, then writes) can be read
half-written. That is a decode failure of that one file, reported for it and not cached, and the next
read tries again. The store's own writes never do this.

## Lock

**Technique:** one advisory whole-file lock per campaign on `.rdm/lock`, taken with the `fs4` crate
(Probe answers), held for the write window only: from before recovery to after the cache update.
The kernel releases it when the process dies, so no stale lock exists. Readers never touch it.

- **Acquire.** `try_lock`, repeated with a back-off that starts at 5 ms and doubles to 50 ms, until a timeout of 5 seconds. A write takes milliseconds, so five seconds is three orders of magnitude of contention. At the timeout the operation fails with `Locked { holder }` and has changed nothing. The timeout and back-off are store options; the numbers here are the defaults.
- **In-process.** A second open of the lock file in the same process is also refused (probed on macOS), so two threads or two server tasks of one process are serialised by the same mechanism.
- **Holder record.** After acquiring the lock the writer replaces `.rdm/holder.json` with `{pid, host, since}` (process id, host name, UTC instant of the acquisition: contracts.md's `LockHolder`). It is removed before the lock is released. It is advisory only: it fills `Locked { holder }` so the caller can say who is writing, and nothing ever decides by it. It can lag by one writer (a waiter may read the previous holder's record in the instant before the new one is written), name a dead process after a kill (the next holder overwrites it), and a process id means nothing across hosts.
- **Why a separate file.** On Windows an exclusive `LockFileEx` region denies other processes read as well as write access to it, and `fs4` locks the whole file there (its source passes the maximum length). A holder record inside the lock file would be unreadable by exactly the process that needs it. Hence the lock file stays empty and is never read or written. Verified from the Microsoft documentation of `LockFileEx`; `second_writer_gets_locked_with_holder` runs on Windows in CI.
- **A missing, truncated or garbage holder record** never blocks or fails the lock: it is only read to fill the error, and a record that does not parse is reported as an unknown holder. Guarded by `garbage_holder_record_does_not_break_locking`.
- **A killed holder.** The lock is free as soon as the kernel has released it, which `lock_survives_holder_kill` shows on all three systems. On Windows the documentation says the release after a process ends "depends upon available system resources", so a waiter must expect a short delay, which the five-second timeout covers.
- **Not covered.** A campaign on a network file system (NFS, SMB), where advisory locks are unreliable, is out of scope. Worktrees are separate copies of the files, so their locks are independent (R01 said the same); git reconciles them.

Guarded by `second_writer_gets_locked_with_holder` and `lock_survives_holder_kill`.

## Write order

### The write algorithm

Every primitive runs these steps; the table below gives step 5 per primitive.

1. Take the lock (Lock).
2. Recover: if `.rdm/intent/` or `.rdm/tmp/` holds anything, run Intent and recovery first.
3. Under the lock, in this order:
   1. **Replay.** If the request carries an `op_id` and the op index finds a record that carries it (Cache), write nothing and return the first outcome with `replayed: true`. The check and the write are one critical section, which is why the store, not M2, owns it. Guarded by `replayed_op_id_writes_nothing`.
   2. Read the files the operation touches **from disk**, not from the cache. Check `expected_rev`, `expected_subtree_rev` and `expected_tree_rev` where given. On a mismatch release the lock and return `Stale { current }`; nothing has been written. This makes a stale write harmless whatever any cache or earlier read held. Guarded by `stale_replace_changes_nothing`.
   3. Check the structure: sibling `name` unique, no cycle, the parent exists and is a directory, and every id the request introduces is absent from the disk (`duplicate_id`).
   4. **Unvalidated nodes.** If the request builds on a node that is not reachable from the root, refuse it with `Invalid` unless it is a repair (relocate it into the validated tree, rename a colliding sibling, delete). Guarded by `open_names_invalid_campaigns`.
4. Compute every after-image, mint the amendment's ULID (the only id the store mints), build the amendment.
5. Apply (table).
6. Update the cache from the written values (Cache), and remove `holder.json`.
7. Release the lock, then return. A write is acknowledged only by returning `Ok`, after step 5 is complete under the durability tier in force.

The `op_id` replay rule, what it returns and its limits are in `contracts.md`; the check sits in step 3.1 and reads the op index of the Cache section.

### The three rules

**Technique:** Soft Updates ordering rules (McKusick and Ganger, Sources), applied to files instead
of disk blocks. In this store the only pointers are `parent_id` (child to parent), a `node_id`, `item_id`
or `Reference.node_id` held by a comment, an amendment or another node, and an id inside a `Mark`.

| Rule | Applied as |
|:--|:--|
| 1. Never point to a structure before it is initialised | Created nodes are written before anything that names them: the parents of a batch before their children, and every node before the amendment that lists it and before a node that was edited to point at it |
| 2. Never re-use a resource before nullifying every earlier pointer to it | Ids are never reused (Ids), so no pointer can come to name a different thing. Before a node is removed, the nodes that pointed at it as their parent are removed first (deepest first) or re-pointed first (edits come before deletes), so no intermediate state has an orphan |
| 3. Never reset the last pointer to a live resource before the new pointer is set | Replacing a file is one rename, so the name never points at nothing. A removed subtree is first copied into the amendment's `snapshot`, which is written and durable before the first unlink: the snapshot is the new pointer to the data |

The apply order that follows from the rules, for any operation: **(a) creates, parents first; (b)
replaces; (c) the amendment; (d) deletes, deepest first.** The amendment sits after the replaces so
that an amendment never claims an edit that has not happened; the only exception the rules force is
a delete, whose before-image must precede the unlink.

### Effects

Notation. `I`: publish the intent record `.rdm/intent/<ulid>.json` (written under `.rdm/tmp/`,
synced, renamed into place). `put X`: write the new bytes under `.rdm/tmp/`, sync them, rename over
`X`. `create X`: the same, but publish with `link()` and then unlink the temporary name, so it fails
if `X` exists instead of replacing it. `del X`: unlink. `N`, `A`, `C`: a node, an amendment, a comment.
After the last effect of an operation with an intent: sync every touched directory, remove `I`, sync
`.rdm/intent/`. `pre` and `post` are the states before and after the operation.

| Primitive | Ordered file effects | A reader sees after each prefix | Recovery from each prefix |
|:--|:--|:--|:--|
| **Replace**, no amendment (a status move, progress, a criterion mark; one node) | `put N` | Before the rename: `pre`. After it: `post` | Nothing to recover: stray temp file removed |
| **Replace**, content edit | `I`, `put N`, `create A` | After `I`: `pre`. After `put N`: new `N`, no amendment yet. After `create A`: `post` | Any prefix after `I`: redo what is missing, giving `post`. Before `I` completes: `pre` |
| **Create** a node | `I`, `create N`, `create A` | After `I`: `pre`. After `create N`: the node exists without its amendment. After `create A`: `post` | As above |
| **Move or rename** | `I`, `put N` (new `parent_id`, `name`, `order`), `create A` | After `put N`: the node at its new place, no amendment yet. No other node file changes | As above |
| **Delete a subtree** of `k` nodes | `I`, `create A` (with the `snapshot`), `del N_k` to `del N_1` (deepest first, the subtree's root last) | After `create A`: the amendment, every node still present. After `j` deletes: a smaller subtree, a valid campaign (no orphan) | As above: redo the remaining deletes. A `del` of a missing file is skipped |
| **Comment** | `create C` | Before the link: `pre`. After it: `post` | Nothing to recover |
| **Batch** (any mix of the above) | `I`, creates (parents first), puts, `create A` (one amendment, all its changes), deletes (deepest first) | After each effect, the prefix of that order | As above |
| **Batch with no amendment** (a directory pause or cancel: several leaves, one `op_id`) | `I`, puts | After `j` puts: `j` of the leaves moved | As above: redo the rest |

Notes on the table:

- A comment never changes a node file, so it changes no `rev`; creating its file needs no intent.
- A move writes one node file and one amendment. `move_writes_one_file_and_keeps_ids` counts files under `nodes/` only (the amendment is written besides, in `_amendments/`): exactly one node file is written and no id changes.
- Every operation that writes two or more files publishes the intent first. A prefix without an intent would be valid for readers, but nothing would complete it: an edit with no amendment, or an amendment for an edit never made. The intent is what turns "every prefix loads" into "every prefix recovers to `pre` or `post`".
- Before `I` is renamed into place the operation has done nothing a reader can see (only temporary files in `.rdm/tmp/`). The rename of `I` is the commit point of a multi-file operation; the rename of the node file is that of a single-file one.
- Guarded by `every_write_prefix_loads_and_recovers_to_pre_or_post`, which cuts each operation after every one of these effects, including the effects inside `I`, and by `killed_workers_lose_no_acknowledged_write`.

### Alternatives considered

| Alternative | Why not |
|:--|:--|
| R01's staged journal: stage files, write a journal of steps, mark it committed, apply | The same idea with two phases. The intent here is one record and one rename, with whole-file after-images, so the commit point is one atomic step and recovery has no "uncommitted" case to discard |
| Copy the campaign, edit the copy, swap the directory | Costs a write of the whole campaign per edit, renames a directory (the Windows hazard the flat layout removed), and breaks every reader that holds a path |
| A database (SQLite in WAL mode) | Tracked data stays text (ruling): a binary file neither merges per node nor shows in a pull request |
| An append-only operation log as the source of truth | Same ruling, and it makes every reader replay |

## Intent and recovery

**Technique:** a redo-only record written before the data (write-ahead logging, ARIES, Sources), in
the form of whole-file after-images, so redo is idempotent by construction. ARIES has no such
"intent" record; this is this design's own application of the log-before-data and repeat-history
ideas, simplified to one record per operation and no undo, because a stale or failed operation has
written nothing before its intent.

**The intent record** (untracked, `.rdm/intent/<ulid>.json`): the format version of the record itself
(an unknown newer one is refused with `NewerFormat`), and the ordered list of effects of step 5:
for each, its action (`put`, `create` or `del`), its path relative to the campaign, and for `put` and
`create` the full new bytes as text. The record is complete before it is published, so a reader of
`.rdm/intent/` never sees a partial one. It holds the amendment's bytes too: **the amendment cannot
serve as the intent**, which the leaf proposed, because a stored `Change` carries no after-image of an
added node and only a field-level `old` and `new` for an edit, and because an old amendment
and a crashed one look alike (a delete's amendment, replayed after a later restore, would delete
the restored node again). See the findings in the leaf report.

**Recovery** runs under the lock on every acquisition, before the operation's own work. It costs
two directory listings when there is nothing to do.

1. Remove everything in `.rdm/tmp/`: stray temporary files of killed writers.
2. For each record in `.rdm/intent/`, oldest ULID first, redo its effects in order:
   - `put`: if the destination already holds exactly these bytes, skip it; otherwise write it as a put.
   - `create`: if the destination holds exactly these bytes, skip it. If it holds other bytes the file was made by something else (a checkout, a merge) after the crash: stop, leave the record, and fail with `Invalid` naming the file. A create never overwrites.
   - `del`: unlink; a file already gone is skipped.
3. Sync the touched directories, remove the record, sync `.rdm/intent/`.
4. A record that does not decode (a power-loss artefact under the weak durability tier) is removed and reported; the ordering rules above mean the files it was meant to complete form a valid prefix anyway.

Recovery is itself restartable: every step is idempotent, so a kill during recovery is repaired by
the next recovery. Not tested by a named guard (see Invariants and guards).

**Why whole-file puts rather than patches.** A `put` of a given file is idempotent whatever the file
holds, so redo needs no log sequence number and no comparison of versions (ARIES needs the page LSN
for that). The price is size: an intent holds each file it writes once more, in untracked, deleted-at-commit storage. A
160-node import is about 0.5 to 1 MB for a moment.


- **A newer intent record.** The intent format is one integer (`INTENT_FORMAT`). A record whose
  format is higher refuses the write with `NewerFormat`, reported as `found = (0, n, 0)` and
  `supported = (0, INTENT_FORMAT, 0)`, so one error type covers both kinds of newer file.

## Durability

Three tiers, chosen when the store is opened. The default is `Fsync`.

| Tier | What each publish does | A process kill (`SIGKILL`, a crash, a panic) | An operating-system crash or power loss |
|:--|:--|:--|:--|
| `Process` | No sync. The rename and the intent still give the ordering | Every acknowledged write survives: the data is in the kernel's cache, which a kill does not discard | The last seconds can be lost, and the disk may have written files out of order, so even the prefixes are not promised |
| `Fsync` (default) | Sync every temporary file before it is published; after the renames, sync each touched directory; the intent, its directory, and the removal of the intent likewise | Same | Writes whose sync returned survive on Linux and Windows file systems that honour it. On macOS this tier is plain `fsync(2)`, whose manual describes `F_FULLFSYNC` as the call that asks the drive to flush its cache, so the last writes can still be lost in a power cut, but the intent plus the ordering rules (each prefix valid, the intent redone) still bound the damage to "pre or post" as long as the drive keeps its own writes in order |
| `Full` | As `Fsync`, with `F_FULLFSYNC` on macOS (the drive flushes its cache). Equal to `Fsync` elsewhere | Same | Survives on macOS as it does elsewhere. About 65 times the cost of a plain publish (probe: 5 ms to 10 ms per file and directory against 0.16 ms) |

- **The standard library's `sync_all` is the `Full` tier on macOS**: measured 4.9 ms median, the same as `F_FULLFSYNC`, because std issues `F_FULLFSYNC` there. The `Fsync` tier therefore calls `fsync(2)` itself (through `rustix`, already in the tree through `fs4`) on macOS, and `sync_all` everywhere else.
- **Directory sync** is not possible on Windows through the standard library and is skipped there: the rename is then left to the file system's own metadata journal (an assumption of this design, not a sourced claim). This is the weakest point of the Windows tier and is stated, not tested.
- **Why `Fsync` as the default for a git-tracked store.** What a power cut can cost is the last acknowledged writes, not the history: committed work is in git, and uncommitted work in a worktree has the same exposure as every source file beside it. A plain `fsync` costs a measured 0.16 ms per file and directory, so an edit (an intent, a node, an amendment and three directories) costs about a millisecond; `F_FULLFSYNC` would cost about 30 ms per edit and 1.6 seconds for a 160-node import, which an agent doing many small edits would feel. `Full` stays available for a user who wants it.
- **What a process kill guarantees is the contract the tests hold** (`killed_workers_lose_no_acknowledged_write`). Power loss is not tested: it needs a file system that injects reordering, which this campaign does not build (README, "Deliberately not tested"). The argument above is the evidence, and it rests on the Soft Updates ordering and the ALICE observation that rename ordering and directory durability vary between file systems, which is why `Fsync` syncs both.

## Cache

The long-lived server keeps the decoded campaign in memory. It is a cache: other processes (another
worktree's server, the CLI, an editor, `git checkout`) change the files under it, so it is
revalidated against the files on every read, writes go straight through to the files, and it is
never the source of truth. A writer never reads it: step 3 of the write algorithm reads the disk, so
a stale cache cannot cause a lost update, only a stale read, and only in the one case named below.

**An entry** holds, per file: the decoded record, its `rev`, a **signature** `(identity, mtime, size)`
where identity is the inode on Unix and the creation time on Windows (a replace by rename gives the
new file a new identity), the wall-clock instant `read_at` taken before the file was statted, and a
`racy` flag. The same entry shape serves `nodes/`, `_amendments/` and `_comments/`: append-only files
are checked like the rest, so every kind of external change is covered by one rule. Derived structures
(paths, depth, the reverse link index, `tree_rev`, the op index and the diagnostics) belong to a generation counter that is bumped when
any entry changes; they are rebuilt from the entries and never outlive them. Guarded by `reverse_index_equals_brute_force_scan`, whose op sequences also compare every derived view to a rebuild.

**Warnings and diagnostics are kept with the decoded nodes.** The `unknown_key` warnings of a file are stored in its entry, and the cross-file diagnostics (orphan, cycle, root, name collision, duplicate id) are derived with the other views. A warm read therefore reports exactly what a cold open would. This is part of `cached_read_equals_cold_read`.

**The op index** maps an `op_id` to the records that carry it (amendments, comments, `History.op_id` entries). It is built on open with the reverse index, updated by write-through, and read in step 3.1 of the write algorithm, under the lock. Its limits: it knows only what the records carry, so an edit that writes no `op_id`-bearing record is not replayed (it is rev-guarded instead); it keeps no fingerprint of the request, so the same `op_id` reused for a different request is not detected; and, being derived from the entries, it is only as fresh as the revalidation, which is why the replay check runs after taking the lock and a recovery, on a validation made then. The full statement of the rule is `contracts.md`'s (`op_id` replay).

**Revalidation, on every read:**

1. List the directory and take each entry's metadata (one `lstat` per file on Unix; free with the listing on Windows).
2. A cached entry whose signature is unchanged and whose `racy` flag is clear is kept.
3. Every other file is read, decoded and hashed again. The signature stored is the one statted before the read, so a change made during the read shows up on the next validation.
4. Entries whose file is no longer listed are dropped; a new name is read.
5. If anything changed, bump the generation.

**Why not the directory `stat`.** The probe shows a directory's mtime changes for a create, a rename and
an unlink but not for an in-place write (a text editor saving, `git merge-file`). A directory-stat fast
path would serve stale data after exactly the edits a person makes by hand, which breaks
`cached_read_equals_cold_read`. The directory is therefore not trusted to say "nothing changed"; a
per-file check on every read is. Its measured cost, 0.25 ms for 160 files and 2.7 ms for 2 000 (from
Python, so an upper bound), is small next to a tool call.

**The racily-clean rule** (git, racy-git, Sources, adapted). A file changed twice within one
timestamp tick has the same mtime and, if the size matches, the same signature. So an entry whose
mtime is not older than its `read_at` by at least `RACY_WINDOW` is marked `racy` and re-read on every
validation until a read finds it old enough. `RACY_WINDOW` is 2 seconds, which covers FAT (2 s) and
HFS+ (1 s); APFS and NTFS are finer but the rule does not depend on it. A future mtime is also racy.
Git zeroes the cached size to force this; this store keeps an explicit flag, because it has no index
file whose own timestamp to compare with.

**What stays undetected.** An external program that rewrites a file in place keeping its size and
setting its mtime back to the old value (`touch -r`, some synchronisers) leaves the signature
unchanged. It is not detected until the entry is evicted or the server restarts; the same limit holds
for git's index. Nothing is written from such a stale entry (writes read the disk). Not tested.

**`notify` is not adopted in M1.** File-event services (inotify, FSEvents, `ReadDirectoryChangesW`)
coalesce, overflow and miss events, so they can only be an accelerator beside the scan; the scan costs
under a millisecond at the expected size, so there is nothing to accelerate.

**Write-through.** Step 6 of the write algorithm updates the entries from what was written, after the
files have been published and before the lock is released:

- The value is the decoded after-image the writer holds; the `rev` is that of the after-image's bytes.
- The signature is statted from the published path afterwards, with `read_at` taken before the write began, so the entry is `racy`: the next reads within two seconds re-read the file once, and if an external program replaced it in between, the re-read finds it.
- Created files are added and deleted files dropped. The cache is never updated before its files are written, so it is never ahead of the disk.
- If the operation fails after writing anything, the entries of every path it touched are dropped, not guessed.

A write returns after step 5 under the durability tier in force, so a cold reader (another process,
a fresh open) that starts after the return sees the written files. Guarded by
`write_through_visible_to_cold_reader_on_return`; and by `cached_read_equals_cold_read` for external
change: an editor writing in place, an atomic replacement by another process, `git checkout`, a new
file, a deleted file, a same-size edit within one tick.

**Persisted form.** Whether a binary cache is built at all is decided by the `hot_cache` leaf from its
cold-versus-warm measurement. If one is, it lives in `.rdm/cache/` (untracked), carries the crate
version and format version in a header, is validated by the same signatures, and is discarded when
any check fails. It is never read as a source of truth and never tracked.

## Windows

Windows has no `rename(2)`: the semantics below are the ones the design relies on, with the evidence
that is and is not available (Probe answers, Sources).

| Concern | Decision |
|:--|:--|
| Replace a file | `std::fs::rename`, which on the MSRV (1.88) is `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING` (from Rust 1.86). Microsoft documents neither atomicity nor the error for a held-open target, so atomic replace on NTFS is an observed property that `killed_workers_lose_no_acknowledged_write` and `every_write_prefix_loads_and_recovers_to_pre_or_post` test on the Windows runner. `ReplaceFile` is not used: its documented failure states can leave the replaced file renamed or absent, and its write-through flag is documented as unsupported |
| Create-only publish | `std::fs::hard_link` (`CreateHardLinkW`, NTFS), as on Unix. If the volume refuses hard links, the store falls back to "check the name is free, then rename" under the lock; the only remaining race is an external writer creating that exact name in the same instant |
| Held-open target | A program that has the target open without `FILE_SHARE_DELETE` (an editor, a scanner, an indexer) makes the rename or an unlink fail. Retry on `ERROR_ACCESS_DENIED` (5) and `ERROR_SHARING_VIOLATION` (32): up to 10 attempts, so 9 waits of 5, 10, 20, 40, 80 ms and then 100 ms for the rest, about 555 ms in all, then fail with the I/O error of the last attempt, its path and its code. That is inside the lock timeout (5 s), and the lock is held throughout, so no other writer interleaves. These two codes, and that holders release within that window, are this design's expectations, not the documentation's; `replace_while_target_open` measures them and the numbers are set by what it shows |
| This store's own readers | They open with `std`'s default share mode, `FILE_SHARE_READ`, `WRITE` and `DELETE`, so they never block a replace or an unlink. Only foreign programs do |
| Lock | `fs4` locks the whole file with `LockFileEx`; the holder record is a separate file for that reason (Lock). A killed process's lock is released by the system after a short, resource-dependent delay, which the lock timeout covers |
| Directory sync | Not available through `std`; skipped (Durability) |
| Long paths | A path below a campaign is at most 40 characters (`_amendments/<26>.json` is 39), whatever the depth of the tree, so only the campaign's own location counts against 260 characters. `std` adds the `\\?\` prefix to long absolute paths, but the store does not rely on it |
| Names | Ids are lower case and carry a prefix, so they cannot collide on a case-insensitive file system or be a reserved device name (`con`, `nul`); a node's `name` is not a file name |
| Line endings | A checkout with `core.autocrlf` can turn LF into CRLF. The decoder accepts CRLF and `rev` is taken over the decoded value, so neither changes a `rev`. `ci_matrix` confirms what a runner does (`git ls-files --eol`) |

## Invariants and guards

Every row names the guard from the campaign README that holds it, or says "not tested" with the
reason. Invariant numbers are those of R01 `§Contracts & Invariants` (I10 is reworded by the A2
rulings from "the server keeps no state between calls" to the cache rule of this document).

| # | Invariant | Rule above | Guarded by |
|:--|:--|:--|:--|
| S1 | Readers take no lock, and see each file as the old or the new version, never a torn one | Reads | `reader_sees_old_or_new_never_partial` |
| S2 | Every prefix of every operation's file effects loads, and recovery makes it exactly the state before or after the operation (I3) | Write order, Intent and recovery | `every_write_prefix_loads_and_recovers_to_pre_or_post` |
| S3 | A process kill loses no acknowledged write (I3) | Durability | `killed_workers_lose_no_acknowledged_write` |
| S4 | A write based on a stale `rev`, `SubtreeRev` or `tree_rev` is refused with `Stale { current }` and changes nothing (I4) | Write algorithm, step 3 | `stale_replace_changes_nothing` |
| S5 | A move writes exactly one file under `nodes/` (the amendment is written besides), and no id changes (I7) | Write order | `move_writes_one_file_and_keeps_ids` |
| S6 | `rev` follows the decoded value, not the bytes | Revisions | `rev_tracks_value_not_formatting` |
| S7 | `tree_rev` changes on any node operation and not on a comment or an amendment | Revisions | `tree_rev_changes_on_node_ops_not_comments` |
| S8 | A minted id that collides is drawn again, and minting ends; an id the store finds already on disk under the lock is refused with `duplicate_id` | Ids | `minting_retries_on_collision` for the minter. The store's own refusal is not tested separately by a named guard: it shares the cross-file `duplicate_id` row of `open_names_invalid_campaigns` |
| S9 | A second writer, after the timeout, gets `Locked { holder }` and the file is unchanged | Lock | `second_writer_gets_locked_with_holder` |
| S10 | A killed holder does not leave the lock held | Lock | `lock_survives_holder_kill` |
| S11 | A missing, truncated or garbage holder record never breaks locking | Lock | `garbage_holder_record_does_not_break_locking` |
| S12 | A replace or unlink against a target another program holds open succeeds once it is released inside the retry window, and fails cleanly otherwise | Windows | `replace_while_target_open` |
| S13 | A cached read equals a cold read after any kind of external change (I10, reworded) | Cache | `cached_read_equals_cold_read` |
| S14 | A write returns only after its files are written, and a cold reader then sees them | Cache, write-through | `write_through_visible_to_cold_reader_on_return` |
| S15 | `between` returns a key strictly between its bounds, of the allowed characters, not ending in `0`, and keys stay short | Order key | `order_keys_sort_strictly_between_their_bounds` |
| S16 | An id of a deleted record is never minted again | Ids | Not tested: a probability under 2e-9 per mint, accepted |
| S17 | Power loss leaves a valid campaign | Durability | Not tested: needs a fault-injecting file system (README, "Deliberately not tested"); argued under Durability |
| S18 | A kill at any point of recovery's own effects is repaired by the next recovery | Intent and recovery | `every_write_prefix_loads_and_recovers_to_pre_or_post` (a row that cuts recovery's own effects too, `crash_test`) |
| S19 | Recovery never overwrites a create-only file that holds other bytes | Intent and recovery | Not tested: it needs an external writer in the crash window |
| S20 | `open` fails on a per-file fault (unparsable file, malformed id, `file_id_mismatch`, `NewerFormat`), and loads and reports cross-file damage (`parent_missing`, `cycle`, `root_count`, `name_collision`, cross-file `duplicate_id`) as one diagnostic per defect | Layout, Reads | `open_names_invalid_campaigns` |
| S21 | An in-place edit that preserves size and restores mtime is detected | Cache | Not tested: documented limit, undetectable by stat |
| S22 | A write that builds on an unvalidated node is refused; a repair write is allowed | Write algorithm, step 3.4 | `requests_breaking_a_rule_are_refused` |
| S23 | A retried `op_id` returns the first outcome and writes nothing | Write algorithm, step 3.1 | `replayed_op_id_writes_nothing` |
| S24 | An unknown key survives in place and produces an `unknown_key` diagnostic, and a warm read reports the same diagnostics as a cold open | Reads, Cache | `unknown_keys_survive_in_place_at_every_level` (survival) and `cached_read_equals_cold_read` (equal diagnostics) |
| S25 | A file newer than the binary, at any part of `0.X.Y`, is refused with `NewerFormat` | Reads | Not tested here: `codec_and_versions` owns the version refusal and its guard; the store only carries it |

## Probe answers

Four probes were assigned to this leaf. Probe code lived in a scratch directory and is deleted;
nothing of it is committed. Machine: macOS 26 (Darwin 25.6), APFS, local `cargo` stable 1.99 and
`cargo +1.88`.

| Question | Answer | Decision |
|:--|:--|:--|
| Lock crate: do `fs4` and `fd-lock` build on 1.88, does a second process see the lock, is it released by `kill -9` of the holder? | `fs4` 1.1.0 (declares MSRV 1.75) and `fd-lock` 4.0.4 both build with `cargo +1.88`. With both, a second process got `BUSY` while the holder lived and acquired the lock right after `kill -9` of the holder (the lock file stays on disk, unlocked). A second `open` of the lock file inside the holder's own process also got `BUSY` (per-open-file-description `flock`, not per-process) | `fs4`. Same semantics as `fd-lock`, a plain `lock`/`try_lock` on the `File` (the names std adopted in 1.89, so dropping the crate later is mechanical), and no guard that borrows the lock owner. std's own `File::lock` is not usable at the 1.88 MSRV |
| Does a rename into `nodes/` change the directory mtime; does an in-place write of an existing file? | Create, rename to a new name, rename over an existing name, and unlink each changed the directory mtime. An in-place write (`r+`, or truncate and rewrite) did not (delta 0), though it changed the file's own mtime. A rename over an existing file also changes its inode, and its mtime even at equal size | The directory mtime cannot certify "nothing changed": an editor writing in place is missed. It is not a fast path. Revalidation is one `readdir` plus a `stat` per entry (see Cache) |
| Granularity of the mtimes? | Nanoseconds: 300 file writes gave 300 distinct `mtime_ns`, none a multiple of 1 000; consecutive renames into one directory gave distinct directory mtimes (156 µs and 277 µs apart) | APFS needs no coarse-timestamp care, but the cache does not rely on it: a conservative racy window (see Cache) covers FAT (2 s), HFS+ (1 s) and the coarse kernel clock of ext4 |
| Cost of `fsync` on a file, on its directory, and `F_FULLFSYNC`? (300 iterations, 1.5 KB file, write + rename) | No fsync: median 0.148 ms, p95 0.255. `fsync` file: 0.157 / 0.222. `fsync` file and directory: 0.157 / 0.289. `F_FULLFSYNC` file: 4.99 / 6.04. `F_FULLFSYNC` file and directory: 9.97 / 10.98. On macOS plain `fsync` does not flush the drive's cache, which is why it is as cheap as no fsync | Default tier `Fsync` (file and directory); `Full` is an opt-in (see Durability). `F_FULLFSYNC` costs about 65 times a plain write and would turn a 160-node import into 1.6 s of waiting |
| What does a revalidation scan cost? (extra probe, feeds the cache decision) | `scandir` plus `stat` of every entry: median 0.25 ms (p95 0.35) for 160 files, 2.7 ms (p95 3.4) for 2 000 files, measured from Python, so an upper bound for Rust. Stat of the directory alone: 1 µs | A full scan on every read is affordable at the expected size. No fast path is built, and `notify` is not adopted in M1 |
| Which hash for `rev`: do `sha2` and `blake3` build on 1.88? | `sha2` 0.11.1 and `blake3` 1.8.7 both build. `blake3` declares MSRV 1.88, exactly the floor, and its default build compiles C and assembly (`cc`) | SHA-256 (see Revisions) |
| Win32 `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING`, and `ReplaceFile`, against a target another program holds open without `FILE_SHARE_DELETE`: which error, when? | **The Microsoft pages do not say.** Neither page names `ERROR_ACCESS_DENIED` (5) or `ERROR_SHARING_VIOLATION` (32) for this case, neither promises atomicity, and `MoveFileEx` says nothing about `FILE_SHARE_DELETE`. What the pages do say: `MoveFileEx` fails if the new name is an existing directory; `ReplaceFile` opens the replaced file with read, delete and synchronize access and share mode read, write and delete, and the replacement with no sharing; `ReplaceFile`'s `REPLACEFILE_WRITE_THROUGH` is documented as not supported; a failed `ReplaceFile` can leave the replaced file renamed or absent (error 1175, 1176, 1177). Rust's `OpenOptions` opens with `FILE_SHARE_READ`, `WRITE` and `DELETE` by default, so this program's own readers never block a replace. Rust's `fs::rename` calls `MoveFileExW(REPLACE_EXISTING)` from Rust 1.86 and falls back to a POSIX-semantics `SetFileInformationByHandle` only on `ERROR_ACCESS_DENIED` (1.85 used the POSIX path first) | The replace is `std::fs::rename`, not `ReplaceFile`: its failure states are not documented as atomic and it leaves the replaced file absent in some of them. Retry on 5 and 32 (the two codes a held-open target is reported with in practice, which is this design's own claim) with a bounded back-off, then fail. Numbers under Windows. `replace_while_target_open` settles the codes on a real Windows runner |

### What the probes did not settle

- Linux and Windows lock release on kill: only macOS was run. `lock_survives_holder_kill` runs on all three operating systems in CI.
- The Windows error code of a held-open target and whether `std::fs::rename` is atomic there: from documentation only, as the table says. `replace_while_target_open` and `killed_workers_lose_no_acknowledged_write` on the Windows runner are the evidence.
- Power loss: no probe can show it on this machine. See Durability.

## Sources

Each source was read, by a research subagent for the PDFs and pages, and by this author for the
Rust `OpenOptionsExt` page. The third column is what the source supports; a claim it does not
support is dropped or marked as this design's own wherever it would have appeared.

| Technique | Source read | What it supports | What it does not, and what this design does with that |
|:--|:--|:--|:--|
| Ordering of dependent writes | [McKusick and Ganger, "Soft Updates", USENIX ATC 1999](https://www.usenix.org/legacy/publications/library/proceedings/usenix99/full_papers/mckusick/mckusick.pdf), §2 | Three rules: never point to a structure before it is initialised; never re-use a resource before nullifying every earlier pointer to it; never reset the old pointer to a live resource before the new one is set (its example is a rename: do not drop the old name before the new one is written) | Soft Updates is about a file system's own metadata and a per-block dependency tracker. This design borrows the rules for the order of files, not the tracker |
| Publish by write-then-link | [Bernstein, Maildir](https://cr.yp.to/proto/maildir.html) and [maildir(5)](https://manpages.debian.org/testing/qmail/maildir.5.en.html) | Write the message under `tmp/`, then `link()` it into `new/`; a reader never sees a partial message; unique names built from time, process id, host and a counter need no lock; the man page requires `write`, `fsync`, `close` each checked before the `link()`; `tmp/`, `new/`, `cur/` on one file system | Bernstein's page gives the rationale and the name scheme but not the delivery steps; those and the fsync are in maildir(5). Maildir publishes with `link()`, not `rename()`. This design uses `link()` for create-only files (it fails when the name exists, which a rename does not) and `rename()` for replacement, which Maildir never does |
| Redo from a durable log | [Mohan et al., ARIES, ACM TODS 17(1), 1992](https://web.stanford.edu/class/cs345d-01/rl/aries.pdf) | Write-ahead logging; the redo pass repeats history; compensation log records are redo-only | Redo idempotence by page LSN: only partly confirmed from the OCR copy read. A "redo-only intent record that recovery rolls forward" is not a construct of ARIES. This design's intent is its own application of log-before-data, with whole-file after-images, so redo is idempotent by construction (putting the same bytes twice) and needs no LSN |
| What a crash can expose | [Pillai et al., "All File Systems Are Not Created Equal", OSDI 2014](https://www.usenix.org/system/files/conference/osdi14/osdi14-paper-pillai.pdf) | Applications assume rename is atomic; most of the vulnerabilities it found sit in missing `fsync` of a parent directory or of data before a rename; whether an append persists before a later rename differs between file systems and even between configurations of one; a single-sector write is not atomic on every file system | The claim "an unsynced temp file renamed into place can leave a zero-length file" was not found in those words and is not cited. This design fsyncs the temp file before the rename and the directory after it because the paper shows the behaviour varies, not because it measured that outcome |
| Racily clean entries | [git, racy-git](https://git-scm.com/docs/racy-git) | An entry whose mtime is not older than the index's own timestamp may have changed within one timestamp tick and must be compared by content; git ignores the nanosecond fields by default; coarse file systems stay problematic | Git also zeroes the cached size so a later stat check fails. This design has no index file to timestamp, so it keeps a per-entry flag and re-reads flagged entries (see Cache): an adaptation, not git's mechanism |
| Atomic replace on Windows | [`MoveFileExW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw), [`ReplaceFileW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew), [`std::fs::rename`](https://doc.rust-lang.org/std/fs/fn.rename.html), [`OpenOptionsExt::share_mode`](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html) | See the probe answer above | Neither Microsoft page promises atomicity or names the error for a held-open target. Atomic replace on NTFS is therefore an observed property this design relies on and tests (`killed_workers_lose_no_acknowledged_write`, `replace_while_target_open`), not a documented guarantee |
| Fractional indexing | [Figma, "Realtime editing of ordered sequences"](https://www.figma.com/blog/realtime-editing-of-ordered-sequences/), [rocicorp/fractional-indexing](https://github.com/rocicorp/fractional-indexing) (Greenspan's design; the article was unreachable, the README states it is based on it) | A key between two keys is a string midpoint; keys compare with plain lexicographic order over an ascending digit alphabet (base 62 by default in rocicorp, base 95 in Figma's post); Figma lists growth of the key length as a known drawback and not a practical concern; `generateNKeysBetween` is advised for bulk inserts | Neither source names a rebalancing step, and jitter appears only as advice to reduce concurrent collisions. The rebalance threshold and the choice of tie-break instead of jitter are this design's own |
| Lock range on Windows | [`LockFileEx`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfileex) | An exclusive lock denies other processes read and write access to the region; the operating system releases the locks of a process that ends, but how long that takes depends on available resources | Not a statement about `fs4`: that the crate locks the whole file on Windows was read in its source (`LockFileEx` with the maximum length) |
| Advisory locks | [`fs4` 1.1.0](https://docs.rs/fs4/1.1.0/), [`fd-lock` 4.0.4](https://docs.rs/fd-lock/4.0.4/) | Crate behaviour as probed above | Not a literature source. The kernel's release of a `flock` on process death is shown by the probe on macOS only |
