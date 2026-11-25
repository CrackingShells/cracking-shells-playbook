# rdm — Contracts

The types and data layouts that M1 freezes, in the crate `rdm-schema`. The Rust types are the
contract: JSON and the JSON Schemas are derived from them, and "schema" here means the data
structures broadly (stored nodes, the campaign model, operation signatures), not only the schema
files. Rulings of Eliott, 2026-11-25, are in the
[A2 gap analysis](../../__reports__/m1-core-foundation/00-gap_analysis_v0.md) and are not reopened
here.

Scope: the stored types, the campaign model, `MOVES` as data, and the store operation request,
result and error types. Tool input and output are M4's. M2 adds semantics (the FSM, impact
analysis) on top of these shapes and never changes them. The disk layout, the id string format,
the order-key format, `rev` hashing, the lock, write order, recovery and the cache are in
[store.md](store.md). This document names them and does not restate them.

The settled meaning of every stored field stays in [stored-schema.md](stored-schema.md); this
document adds the Rust names, what each type derives instead of storing, and who relies on it.

Three consumers read it: the **codec** (`rdm-engine`, serialises the types), the **store**
(`rdm-engine`, persists them), and **M2/M4** (graph semantics, tools). Each type below says which
one relies on a rule.

## Id inventory

**Rule.** A record gets an id when it is pointed at from outside **and** its natural key can
change. The natural key is whatever else would name it: a path segment, a position, a title, a
label. A record that nothing points at, or whose only key never changes, gets none. Every id costs
a minting rule, a uniqueness rule and a link to declare, so an id with no pointer is excess.

| Record | Pointed at from outside | Natural key | Verdict | Reason |
|:--|:--|:--|:--|:--|
| Node (campaign root, directory, leaf) | `parent_id`, `Reference.node_id`, `Comment.node_id`, `Change.node_id`, `Amendment.snapshot` keys | `name` among siblings, and the chain of parents: both change on a rename or a move | **id** (`NodeId`) | I7: a rename or a move changes no id |
| Criterion | `Mark.criterion_id`, `Change.item_id` | position and text: a reorder or an edit changes both | **id** (`CriterionId`) | A mark must survive a reorder |
| Step | `Change.item_id` | position (shown as the step number) and title | **id** (`StepId`) | A recorded step correction names the step |
| Check | none: a step owns its checks, and no record marks, runs or cites one | position | **no id** | An edit of a check is recorded at the array level: a `set` of `/checks` on the step, with the whole old and new array |
| Reference | none: `Reference.node_id` points *out*, nothing points *in*. Prose cites `label` (`R01`), and a text cite is not a declared link | `label`, unique within its array | **no id** | As for a check: a `set` of `/references` on the owner (the node, or the step through its `item_id`) |
| Comment | none stored | none: a file named by its id, ordered by it | **id** (`CommentId`, a ULID) | Identity is the file name and the order. Not by the rule: there is no natural key to protect |
| Amendment | none stored (the display id is derived from it) | none | **id** (`AmendmentId`, a ULID) | As a comment |
| History entry | none: entries are append-only, and `op_id` groups them without naming one | position, never used | **no id** | `op_id` is a shared retry key, not the id of an entry |
| Mark | none: append-only | position, never used | **no id** | As a history entry |
| Change | none | position in `changes` | **no id** | Part of its amendment |
| `Commit`, `Progress`, `Actor` | none | none | **no id** | Single objects reached through their owner |
| `op_id` | the records an operation wrote | the caller | **`OpId`**, caller-supplied | A retry key, not minted by the store, never a record id |

**Decided (Eliott, 2026-11-25), proposal confirmed.** Nodes, criteria and steps keep ids. Checks and references
do not, and are recorded at the array level in a change. This **overturns** the current
[stored-schema.md](stored-schema.md) (M0b), which gave ids to checks and references, and the
current code, whose `Check` and `Reference` carry an `id`. The consequence is that `Change.item_id`
names a criterion or a step only. A reference or a check that later gains a pointer (a check result
record, say) gets an id then, as a compatible addition (see `Version chain`).

### Id types

Three newtypes over `String`, one validation, written once in `rdm-schema`.

| Rust type | Names | Kind prefix |
|:--|:--|:--|
| `NodeId` | a node, whatever its `kind`: promoting a leaf to a directory keeps its id (I7), so the id does not encode the kind | the node prefix |
| `CriterionId` | a criterion, in `preconditions` or `success_gates` | the criterion prefix |
| `StepId` | a step | the step prefix |

```rust
pub struct NodeId(String);          // likewise CriterionId, StepId (one macro)
impl NodeId {
    pub fn parse(s: &str) -> Result<Self, IdError>;   // the only constructor from a string
    pub fn as_str(&self) -> &str;
}
// FromStr, TryFrom<String>, Display; Serialize as a JSON string;
// Deserialize through `parse`; Ord, Hash. JsonSchema: a string with the id pattern.
pub enum IdError { Empty, WrongPrefix { expected: &'static str }, BadChar, BadLength }
pub enum ItemId { Criterion(CriterionId), Step(StepId) }   // JSON form: the bare string
```

- **Validation.** Each type accepts exactly a string in the format of [store.md](store.md) §Ids
  with the kind prefix of this type. The grammar is implemented once, in `rdm-schema`, from that
  section, and the generated JSON Schema pattern comes from the same constant. A deserialised id
  is always valid, so a function that takes a `NodeId` never re-checks.
- **`ItemId`** is the one place the prefix carries meaning: it reads as a `CriterionId` or a
  `StepId` by its prefix, so the prefixes of the two must differ.
- **`CommentId` and `AmendmentId`** are ULIDs, not store.md ids: the read model orders comments and
  amendments by id, oldest first, which only a time-ordered id gives. Both validate as 26
  characters of Crockford base32.
- **`OpId`** is one line of 1 to 128 characters with no control character. The caller chooses it;
  the store never mints one.
- **Never reused.** An id, once minted, is not given to another record, including after a delete
  (`stored-schema.md`, `Stable identity`). Sub-item ids (`CriterionId`, `StepId`) are unique within
  their node file. `NodeId`s are unique within the campaign.

## Node fields

A node is one file, `nodes/<NodeId>.json` (layout: store.md). Three fields say where it sits; the
rest of the file is as in stored-schema.md.

| Field | Rust type | On | Stored | Meaning and consumer |
|:--|:--|:--|:--|:--|
| `id` | `NodeId` | every node | yes | Identity. Immutable (I7) |
| `name` | `Name` | every node | yes | The path segment, `^[a-z][a-z0-9_-]*$`. Unique among the children of one parent. **For the campaign root it is the campaign slug**, which `Campaign::open` is given and checks (`Campaign model`); the root's name is not part of any path |
| `parent_id` | `NodeId` | every node but the campaign root | yes | The directory this node is in. Always a `Directory` node |
| `order` | `OrderKey` | every node but the campaign root | yes | Position among siblings. An opaque newtype: its format and how a key is made between two others are store.md's (§Order key). Compared as given, never parsed. Ties break by `id` |
| `kind` | `DirectoryKind` or `LeafKind` | every node | yes | `campaign` exactly when `parent_id` is absent, so the type of the root is checked against its place |

On `Directory`, `parent_id` and `order` are `Option`: they are absent for the campaign root and for
it only (`root_shape` rule below). `name` is required on every node, the root included. On `Leaf`
all three are required, so a leaf cannot be a root by construction. The root's path is `""`. The
campaign slug is the anchor of validation: the root is the parentless node that carries it.

```rust
pub struct Name(String);                  // parse(&str) -> Result<Name, NameError>
pub struct OrderKey(String);              // opaque; format in store.md §Order key
pub enum Node { Directory(Directory), Leaf(Leaf) }   // discriminated by `kind` in the file
impl Node {
    pub fn id(&self) -> &NodeId;
    pub fn name(&self) -> &Name;
    pub fn parent_id(&self) -> Option<&NodeId>;
    pub fn order(&self) -> Option<&OrderKey>;
    pub fn decode(raw: serde_json::Value) -> Result<Node, StoreError>;  // by `kind`; see Version chain
}
```

`Node` is not a serde-tagged enum: `Directory.kind` already holds `campaign` or `directory`, so
`Node::decode` reads `kind` and picks `Leaf` (`leaf`) or `Directory` (`campaign`, `directory`). Any
other value is `Invalid` with the rule `kind`.

**Derived from these fields, never stored (I6)**, by the campaign model (below): the children of a
node, its depth (the ordering rule I1), its path, its siblings' order. Nothing in a file holds a
child list or a path.

**Rules the validator checks per node** (each is an `Invalid` rule code, see `Errors`):

| Rule code | Statement |
|:--|:--|
| `root_shape` | `parent_id` and `order` are both absent exactly when `kind` is `campaign` |
| `name_format` | `name` matches the `Name` pattern |
| `malformed_id` | an id field does not parse as its type |

Rules that need the whole campaign (one root, the parent exists and is a directory, no cycle,
sibling names unique) are in `Campaign model`.

**What changes against the current code.** `Directory` and `Leaf` gain the three fields, in the
canonical order `schema_version`, `id`, `kind`, `name`, `parent_id`, `order`, then the rest of the existing
order.
`Check` and `Reference` lose `id`. Every `id`, `node_id`, `criterion_id` and `item_id` field changes
from `String` to its newtype. `Leaf.kind`, `Check.expected` and the other enums are unchanged.
`stored-schema.md` is edited to match.

## Campaign model

The shape M2 consumes instead of designing it: the nodes of one campaign as a table by id, with
everything about the tree derived from `parent_id`, `name` and `order`. A **parent-pointer tree**
(adjacency by parent id), not a nested value: a rename or a move edits one record and rebuilds no
subtree.

```rust
// In rdm-schema, pure: no I/O. The engine's `Store` (held by the public `CachedCampaign` handle) reads and decodes the files and calls `open`.
pub struct Campaign { /* nodes; root: Option<NodeId>; children index; depth memo;
                         unvalidated: set of NodeId; diagnostics: Vec<Diagnostic> */ }
#[must_use = "an opened campaign carries diagnostics; read them"]
pub struct Opened { pub campaign: Campaign, pub diagnostics: Vec<Diagnostic> }
pub struct Diagnostic { pub file: String, pub rule: Rule, pub detail: String }  // always a warning

impl Campaign {
    /// The only constructor. `expected_slug` is the campaign's name as its caller knows it.
    pub fn open(nodes: impl IntoIterator<Item = Node>, expected_slug: &Name) -> Result<Opened, StoreError>;
    pub fn diagnostics(&self) -> &[Diagnostic];                         // the same list, kept on the value
    pub fn is_validated(&self, id: &NodeId) -> bool;
    pub fn root(&self) -> Option<&NodeId>;                              // None: no validated root
    pub fn get(&self, id: &NodeId) -> Option<&Node>;
    pub fn nodes(&self) -> impl Iterator<Item = &Node>;                 // by NodeId, validated or not
    pub fn parent(&self, id: &NodeId) -> Option<&NodeId>;               // None for a root or an orphan
    pub fn children(&self, id: &NodeId) -> &[NodeId];                   // sorted by (order, id); empty for a leaf
    pub fn depth(&self, id: &NodeId) -> Option<u32>;                    // I1; None for an unvalidated node
    pub fn path(&self, id: &NodeId) -> Option<String>;                  // "" for the root; None if unvalidated
    pub fn resolve(&self, path: &str) -> Option<&NodeId>;               // inverse of `path`, validated nodes only
    pub fn subtree(&self, id: &NodeId) -> impl Iterator<Item = &NodeId>; // pre-order, `id` first
    pub fn insert(&mut self, node: Node);                               // incremental: derived indexes and the
    pub fn remove_subtree(&mut self, id: &NodeId) -> Vec<NodeId>;       // validated set equal a rebuild
}
```

| Derived | Definition | Never stored because | Relied on by |
|:--|:--|:--|:--|
| `children(id)` | the nodes whose `parent_id` is `id`, sorted by `(order, id)` | a child list in the parent would be a second copy that a move must edit in two files | M2 (waves, ready set), the read model |
| `depth(id)` | the number of **directories between the campaign root and the node**: a node directly under the root has depth 0, so this is `level` in `read-model.md`. The ordering rule I1 is this number, computed from the `parent_id` chain | it would go stale on every move of an ancestor | M2 (ready set), the read model |
| `path(id)` | the `name`s from the root's child down to the node, joined by `/`; `""` for the root | it would go stale on every rename or move above it, and a stored path is the value a flat layout exists to avoid | M3 (export), the read model, M4 (a tool takes a path or an id) |

### Open: refuse a bad file, report a bad tree

`open` separates two kinds of fault (Eliott, 2026-11-25).

**Per-file faults refuse `open`** with a `StoreError`, because the file itself is not sound. Any
one of these in any file fails the whole open, naming the file:

| Fault | Error |
|:--|:--|
| the text is not JSON, or not an object of the expected type | `Invalid`, rule `parse` |
| `schema_version` missing or malformed | `Invalid`, rule `schema_version` |
| the file is newer than the binary | `NewerFormat` (raised first, `Version chain`) |
| an id field does not parse as its type | `Invalid`, rule `malformed_id` |
| `nodes/<id>.json` holds another `id` | `Invalid`, rule `file_id_mismatch` |
| a node breaks a rule of its own record: `kind`, `root_shape`, `name_format`, the stored-schema rules, `sub_item_id_unique`, `reference_target_derived` | `Invalid`, that rule |

**Cross-file damage is loaded and reported.** A git merge of two branches can leave files that are
each sound and wrong together. They are loaded, and each defect is **one `Diagnostic`** naming the
file or files and the rule. What cannot be done is validating the node against the tree.

| Rule code | Defect | Which nodes become unvalidated |
|:--|:--|:--|
| `parent_missing` | a `parent_id` names no node (an orphan) | the node and its subtree |
| `parent_not_directory` | a `parent_id` names a leaf | the node and its subtree |
| `cycle` | following `parent_id` never reaches a parentless node | every node on the cycle and every node below it |
| `root_count` | no parentless node carries the expected slug, or more than one does (one diagnostic, filed under the smallest of their ids, naming the others) | every node: there is no validated root, and none of the slug-named nodes is chosen as one |
| `root_name_mismatch` | a parentless node carries another name than the slug | that node and its subtree |
| `name_collision` | two children of one parent share a `name` | the colliding siblings and their subtrees (a path through them is ambiguous) |
| `duplicate_id` | two node records hold one `NodeId` (reachable through `open` over nodes; through the store a copied file is already `file_id_mismatch`) | both records |

**The anchor.** `expected_slug` is the campaign's slug as the caller knows it (the campaign
directory's name, as M4 and the CLI resolve it). The **root** is the one parentless node whose
`name` equals the slug. Every node reachable from the root by `parent_id` is **validated**: it has
a depth and a path. Every other node is **loaded but unvalidated**: an orphan, a node on a cycle, a
second parentless node, a node under a root whose name does not match, a node below a name
collision. When no parentless node carries the slug, `open` still succeeds: `root()` is `None`,
**every node is unvalidated**, and the diagnostics say so (`root_count`, and `root_name_mismatch`
for each parentless node of another name). A caller then sees the campaign, damaged, instead of
nothing, and cannot mistake it for a sound one.

**How "loud" is enforced.** The diagnostics are part of the return value: `open` returns
`Opened { campaign, diagnostics }`, a `#[must_use]` struct whose fields must be taken apart to reach
the campaign, so dropping it unread is a compiler warning (the workspace builds with `-D warnings`).
The same list stays on the `Campaign` (`diagnostics()`), so every later read can surface it again:
M4's read tools carry it in their output, and the CLI prints it to standard error on every command.
`depth` and `path` return `Option`, so a consumer cannot compute with a position that was never
validated. The cache reopens with the same slug on revalidation (store.md §Cache), so a repaired
campaign loses its warnings and a damaged one keeps them.

**Writes and unvalidated nodes.**

| Write | On an unvalidated node | Rule code |
|:--|:--|:--|
| create under it, replace its content | **refused**: the write would build on a position no one verified | `unvalidated_node` |
| comment on it | **allowed**: a comment is an append-only record that depends on the node's id, not its position; it is how a reader annotates the damage | |
| relocate it into the validated tree (new `parent_id` validated) | **allowed**: a repair | |
| relocate or rename a colliding sibling | **allowed**: a repair | |
| delete it (its subtree) | **allowed**: a repair. A node on a cycle is deleted with every node below it, the whole cycle included | |
| any other relocate of it (a rename of an orphan, say) | **allowed**: it builds nothing on the unverified position, and the rule below still holds | |

A write never adds a defect: after it the set of diagnostics is a subset of the set before. A
batch is checked as a whole under this rule, so a repair that fixes one defect and a write that
opens another are refused.

**The one exception, the root family.** While the campaign has **no validated root**, `root_count`
and `root_name_mismatch` may change form in a repair: renaming one of two slug-named roots away
turns one `root_count` into a `root_name_mismatch` on the renamed node, and that write is allowed.
While a validated root exists, the two codes follow the rule above like every other code, so a write
that takes the validated root away (a rename of a sound root, a second slug-named root) is refused.
In particular **a sound root cannot be renamed**: renaming the campaign is renaming its directory
first, after which the root is unsound (`root_name_mismatch`) and its rename is the repair.

`open` builds the children index in one pass, then computes depth by walking from the root down
with a memo and a visiting mark (a depth-first search with three colours), so a cycle is found,
not followed, and the whole open is O(n). `insert` and `remove_subtree` update the same indexes in
O(depth) and O(subtree); a rebuild is the oracle for them (`reverse_index_equals_brute_force_scan`
also compares children, depth and path with a rebuild).

Siblings are parallel (I1): `order` only fixes how they are listed, never when they run, and no
stored field expresses a dependency between them. `Campaign` holds nodes only. Comments and
amendments are separate files, read by the cache (store.md §Cache) and joined to a node by id.

### Diagnostics

One type for every warning, from three producers:

| Producer | Rules | When |
|:--|:--|:--|
| a layer's `decode_*` | `unknown_key` | per unknown key kept without interpretation: `file` is the file, `detail` is the key's JSON pointer (`/steps/0/x_note`). The decode returns it with the value (`Decoded<T>`, `Version chain`) |
| `Campaign::open` | the cross-file rules above | per defect |
| `Campaign::open`, from the link table | `dangling_reference`, `references_empty` | per reference whose `node_id` names no node; per leaf with no reference |

`open` aggregates the decode diagnostics of every file with its own and returns them sorted by
`(file, rule, detail)`, so two opens of the same files give the same list. A defect that spans
several files (a cycle, a collision, a second root) names in `file` the file of the smallest id
involved, and lists the other ids in `detail`.

**Settled while building the store (coordinator, 2026-11-25).** The root has depth 0. A change
that adds, removes or moves a sub-item names in `field` the array's pointer (`/steps`). A mark
names its criterion by id, so `Mark.criterion` is a declared link: a write that leaves a mark on a
missing criterion is refused like a dangling reference, and `open` reports one. `id_changed`
applies only when a replace payload carries an id of its own that differs from the target. A diagnostic is a
warning by type; an error is a `StoreError`.

## Moves

`MOVES` is the move table of [statuses.md](statuses.md) as data. Its consumers are M2 (the FSM:
which move is allowed, who may make it, what it needs, what it does to the ready set) and M4 and the
CLI (the refusal text). The document is the oracle: the test `move_table_matches_statuses_doc`
parses it and compares it with `MOVES`, so a change to the table of the document fails the build
until `MOVES` follows.

**The table it encodes** is the first table after the heading `## Transitions` in
`__design__/rdm/statuses.md`, whose header row is exactly

`| From | To | Allowed | Who | Reason | Precondition | Ready set |`

Cells hold no `|`, so the test splits a row on it. Rows are the 25 ordered pairs of the five
statuses, each once. A column added, renamed or reordered in the document fails the test by
design: the header is part of the contract.

```rust
pub struct Move {
    pub from: Status, pub to: Status,
    pub allowed: bool,
    pub who: Option<Who>,
    pub reason: ReasonRule,
    pub precondition: &'static str,
    pub ready_set: &'static [ReadyEffect],
}
pub enum Who { Agent, Human, Both }
pub enum ReasonRule { Required, OnlyForOverride, NotRequired }
pub enum ReadyEffect { Enter, Leave, Release, Hold }
pub static MOVES: [Move; 25];                       // row-major: index = from * 5 + to
pub fn lookup(from: Status, to: Status) -> &'static Move;   // total: all 25 pairs exist
```

| Column | Cell | Field | Parse rule |
|:--|:--|:--|:--|
| From, To | `` `planned` `` … `` `done` `` | `from`, `to` | the backticked word, as the serde name of `Status` |
| Allowed | `allowed` or `refused` | `allowed` | `allowed` is `true` |
| Who | `agent`, `human`, `both`, or empty | `who` | empty is `None`; the document uses `both` on every allowed row (no move is human-only) |
| Reason | `required`, `only for an override`, `none`, or empty | `reason` | `required` is `Required`; `only for an override` is `OnlyForOverride`; `none` and empty are `NotRequired` |
| Precondition | prose | `precondition` | **verbatim**, trimmed, backticks kept. On a refused row it is the refusal message; M2 owns the logic that checks an allowed row's precondition, keyed on `(from, to)` |
| Ready set | `none`, or uppercase tokens then optional prose | `ready_set` | the leading tokens `^[A-Z]+(, [A-Z]+)*` as `Enter`, `Leave`, `Release`, `Hold`; `none` is empty. The prose after them ("if nothing shallower holds it back", the `HOLD:` explanation) restates the ready-set definition and is not encoded |

At the date of this document 16 rows are allowed and 9 refused; the test reads the document, not
these numbers. The test also requires, for every row, `allowed` exactly when `who` is `Some`, and
that `MOVES[i]` is the pair its index says. Verbatim preconditions mean M2 re-reads a move whenever
its prose changes, which is the point. `statuses.md` is not edited.

The test reads the document at `../../__design__/rdm/statuses.md` from the crate directory. The
file is not part of a published crate, so the test is a development-time guard only.

## Links

Eliott, 2026-11-25: every key-value link between records is declared, so an update or a delete
reaches every reference automatically, and a value derivable from an id is never stored. A **link**
is a field of one record that holds the id of another. `LINKS` is the list of all of them, and the
reverse index is derived from it on load.

```rust
pub struct Link {
    pub owner: Owner,                 // the file kind that holds the field
    pub pointer: &'static str,        // JSON pointer; `*` is any array index, `{key}` is a map key
    pub target: Target,
    pub scope: Scope,
    pub on_delete: OnDelete,
}
pub enum Owner { Directory, Leaf, Comment, Amendment }
pub enum Target { Node, Criterion, Step, Item }          // `Item`: a criterion or a step (`ItemId`)
pub enum Scope { Campaign, OwnerFile, ChangeNode }
pub enum OnDelete { Cascade, Report, Keep }
pub static LINKS: [Link; 11];
```

| # | Owner | Pointer | Target | Scope | On delete of the target |
|:--|:--|:--|:--|:--|:--|
| 1 | `Directory` | `/parent_id` | node | campaign | `Cascade` |
| 2 | `Leaf` | `/parent_id` | node | campaign | `Cascade` |
| 3 | `Directory` | `/references/*/node_id` | node | campaign | `Report` |
| 4 | `Leaf` | `/references/*/node_id` | node | campaign | `Report` |
| 5 | `Leaf` | `/steps/*/references/*/node_id` | node | campaign | `Report` |
| 6 | `Directory` | `/marks/*/criterion_id` | criterion | owner file | `Keep` |
| 7 | `Leaf` | `/marks/*/criterion_id` | criterion | owner file | `Keep` |
| 8 | `Comment` | `/node_id` | node | campaign | `Keep` |
| 9 | `Amendment` | `/changes/*/node_id` | node | campaign | `Keep` |
| 10 | `Amendment` | `/changes/*/item_id` | criterion or step | the node of that change | `Keep` |
| 11 | `Amendment` | `/snapshot/{key}` | node | campaign | `Keep` |

The `id` field of each record is its identity, not a link, and is not listed. Targets in the
**campaign** scope are nodes anywhere. Criterion and step ids are unique within a node file only
(`Id types`), so a link to one resolves inside a file (`OwnerFile`) or inside the node a change
names (`ChangeNode`), and is not a key of the reverse index.

**What a delete does to a referenced node.** A subtree delete never rewrites a record outside the
subtree. Per class:

| `OnDelete` | Meaning | Rows | Reason |
|:--|:--|:--|:--|
| `Cascade` | the referrer is inside the subtree and goes with it | `parent_id` | a child of a deleted node is by definition in the subtree, so a `parent_id` to a deleted node cannot survive outside it |
| `Report` | the referrer stays untouched, and the delete **reports** it to the caller (`Deleted.referrers`, `Operations`); M2's impact analysis decides what to do with it (I9) | `Reference.node_id` | a delete is **not refused** by a reference: a document pointer is not corruption, and refusing would make a documented node undeletable. It is **not rewritten** either: clearing the pointer is a content edit of another node, so it needs its own amendment (I5) and changes that node's `rev` under another writer, and a delete would then write an unbounded set of unrelated files. The reference keeps its `title`, and a reader shows it as dangling (warning `dangling_reference`, never an error, so I3 holds) |
| `Keep` | the referrer is history and is never edited | comments, amendments, marks | append-only records (I5): a comment or a change that names a deleted node resolves through the `snapshot` of the amendment that removed it. A mark that names a removed criterion resolves through the `old` of the `remove` change that removed it |

**Writes check what they introduce.** A write that sets a `Campaign`-scope link, or changes it,
must point at an existing node, else `Invalid` (rule `dangling_id`) or, for a comment, `NotFound`.
A link may dangle only by a later delete of its target.

**Historical copies are not links.** The values below hold ids inside opaque JSON and are copies of
the past, not references: the values under `/snapshot/*`, and `old` and `new` of a change (a node
`move` holds `{"parent_id", "order"}` there). They are excluded from `LINKS`, from the reverse
index, and from the scan that tests it. Nothing reads them as live; a restore reads them through
the version chain of the time they were written.

### Reverse index

Derived from `LINKS` on load, never stored (I6), and kept in step by write-through (store.md
§Cache). It answers "who points at this node", for the delete report and for M2's impact analysis.

```rust
pub enum RecordRef { Node(NodeId), Comment(CommentId), Amendment(AmendmentId) }
pub struct Referrer { pub link: usize, pub from: RecordRef, pub at: String }  // `at`: the concrete JSON pointer
pub struct ReverseIndex { /* BTreeMap<NodeId, Vec<Referrer>> */ }
impl ReverseIndex {
    pub fn build(c: &Campaign, comments: &[Comment], amendments: &[Amendment]) -> ReverseIndex;
    pub fn referrers(&self, target: &NodeId) -> &[Referrer];   // every class, ordered by (from, at)
    pub fn replace_record(&mut self, from: RecordRef, record: Option<&Record>);  // None: the record is gone
}
```

`build` walks each record once and follows every `LINKS` row whose owner matches, so it is
O(records × links). `replace_record` drops the old entries of one record and adds its new ones, so
a write touches the index in proportion to the record, not the campaign. The `children` index of
the campaign model is the `parent_id` slice of the same data, built in the same pass.

The oracle is independent of `LINKS`: for each record, a **brute-force scan** serialises it and
finds every string equal to a campaign node id outside the identity field and the historical
copies above. Both must give the same referrers. A new id field that is not in `LINKS` shows as a
difference, and `links_cover_every_id_field` catches it earlier: it reads the generated JSON Schema
of every record, collects each property whose schema is an id newtype (`NodeId`, `CriterionId`,
`StepId`, `ItemId`) other than `id`, and requires the set to equal the pointers of `LINKS`. The
`{key}` row 11 is a map key, which a property walk does not see, and is checked by its own
assertion in the same guard: a `snapshot` key that is not a `NodeId` is refused.

**Reference.target is derived, not stored, when `node_id` is set** (`stored-schema.md`,
`Reference`). A record holding both is `Invalid` (rule `reference_target_derived`). The derived
value is the path of the node (`Campaign model`), relative to the campaign root.

## Operations

One request type and one result type per store primitive. These are the signatures M2 composes
and M4 wraps as tool inputs (tool envelopes, names and output budgets are M4's). The types live in
`rdm-schema` and derive `Serialize`, `Deserialize` and `JsonSchema`, so M4 builds its input schemas
from them. The trait that implements them lives in `rdm-engine`, because `rdm-schema` holds no I/O.

**The split of work.** The store is persistence, structural validity, the revision check (I4),
the amendment record (I5), atomicity and replay. It does **not** decide whether a status move is
legal, whether a finished node needs a reason, or what an operation means for the ready set: M2
computes the new node values and calls the store. A caller that states a rule the store does not
check (the FSM) cannot get it from the store; a rule the store checks (a shape rule, a stale
revision) cannot be skipped by a caller.

```rust
pub struct Rev(String);        // one node: hash of its canonical encoding. Format: store.md §Revisions
pub struct SubtreeRev(String); // a node and every descendant; defined in store.md §Revisions, as tree_rev over the subtree
pub struct TreeRev(String);    // the campaign. Consumers compare for equality, never parse

pub struct Request<T> { pub header: Header, pub op: T }
pub struct Header {
    pub op_id: Option<OpId>,         // the retry key; see `op_id replay`
    pub actor: Actor,                // stamped on the amendment or comment the operation writes
    pub note: Option<EditNote>,      // the amendment's words; see `The amendment rule`
}
pub struct EditNote { pub description: String, pub source: Option<String>, pub correction: bool }

pub enum Position { First, Last, Before(NodeId), After(NodeId) }   // a sibling, resolved to an OrderKey by the store

pub struct CreateOp   { pub node: Node, pub position: Option<Position> }
pub struct ReplaceOp  { pub expected_rev: Option<Rev>, pub node: Node }
pub struct RelocateOp { pub id: NodeId, pub expected_rev: Option<Rev>,
                        pub parent_id: Option<NodeId>, pub name: Option<Name>, pub position: Option<Position> }
pub struct DeleteOp   { pub id: NodeId, pub expected_subtree_rev: Option<SubtreeRev> }
pub struct CommentOp  { pub node_id: NodeId, pub text: Lines }
pub struct BatchOp    { pub expected_tree_rev: Option<TreeRev>, pub ops: Vec<Op>, pub dry_run: bool }
pub enum Op { Create(CreateOp), Replace(ReplaceOp), Relocate(RelocateOp), Delete(DeleteOp), Comment(CommentOp) }

pub struct Written   { pub id: NodeId, pub rev: Rev }
pub struct Deleted   { pub removed: Vec<NodeId>, pub referrers: Vec<Referrer> }   // pre-order; `Report` links only
pub struct Commented { pub id: CommentId }
pub enum OpResult { Written(Written), Deleted(Deleted), Commented(Commented) }
pub struct Receipt<R> {
    pub result: R,
    pub amendment: Option<AmendmentId>,   // the one amendment the operation wrote, if any
    pub tree_rev: TreeRev,                // after the write
    pub replayed: bool,                   // true: nothing was written, the first outcome is returned
}
```

**`expected_rev` is `Option` only for a batch.** Outside a batch it is required (a `None` is
`Invalid`, rule `expected_rev_required`). Inside a batch, `None` is allowed solely for a node the
same batch created earlier, which has no revision yet. The same holds for `expected_subtree_rev`.

| Primitive | Request, result | What it does | Refused (`Invalid` rule) |
|:--|:--|:--|:--|
| create | `Request<CreateOp>` → `Receipt<Written>` | writes a new node file. The ids in `node` (its own, its criteria and its steps) are minted **by the caller, outside the lock**, from the engine's minter (store.md §Ids), so a batch can name a node it creates, an importer can build a campaign, and a replay returns the same ids. The store checks uniqueness **under the lock** and refuses a collision (`duplicate_id`); the caller re-mints. `node.order` is **not read**: the store sets it from `position`. `parent_id` is `None` only for the campaign root, which is created into an empty campaign (`position` is then `None`) | `duplicate_id`, `parent_missing`, `parent_not_directory`, `name_collision`, `root_count`, `root_name_mismatch` (the root's `name` is not the campaign slug), `unvalidated_node` (the parent is unvalidated), any shape or stored-schema rule, `position_not_sibling`, `note_required` |
| replace | `Request<ReplaceOp>` → `Receipt<Written>` | replaces one node file with `node` if its current `rev` equals `expected_rev` (I4). It cannot change `id`, `name`, `parent_id` or `order`. `kind` may change (promoting a leaf to a directory keeps the id, I7) | `id_changed`, `use_relocate`, `note_required`, `empty_edit`, `leaf_has_children`, `unvalidated_node`, any shape or stored-schema rule |
| relocate | `Request<RelocateOp>` → `Receipt<Written>` | **rename or move**: one primitive that changes `name`, `parent_id` and `order` of one node. A field left `None` is kept; a new `parent_id` with no `position` means `Last`. At least one of the three is given. It writes **one node file** and one amendment, and renames no directory (I7). Children follow because they point at the id. On the root only `name` may change: that is the repair for `root_name_mismatch` after a campaign is renamed, and the caller then opens with the new slug. A new `parent_id` or `order` on the root is `root_delete_or_move` | `nothing_to_do`, `parent_missing`, `parent_not_directory`, `cycle` (the new parent is in the node's own subtree), `name_collision`, `position_not_sibling`, `root_delete_or_move`, `note_required` |
| delete | `Request<DeleteOp>` → `Receipt<Deleted>` | removes the node and every descendant if the `SubtreeRev` matches. Writes one amendment whose `snapshot` holds the removed nodes. Reports the `Report`-class referrers outside the subtree (`Links`); rewrites nothing outside it | `root_delete_or_move`, `note_required` |
| comment | `Request<CommentOp>` → `Receipt<Commented>` | writes one comment file. It has no `expected_rev`: it is append-only and never conflicts, and a comment never changes a `rev` (`stored-schema.md`) | `NotFound` for a missing node; `note_forbidden` when `header.note` is set |
| batch | `Request<BatchOp>` → `Receipt<Vec<OpResult>>` | applies `ops` in order as **one transaction**. Each op is checked against the state the earlier ops produced, then the whole result is checked once (shape rules, I3). All or nothing: a failed op leaves nothing written. `dry_run` runs every check and writes nothing, for M2's impact report. The batch writes **one amendment** carrying every change of its ops | the first failing op, as `InBatch { index, source }`; `expected_tree_rev` stale is `Stale` |

A single-op request behaves as a batch of one; the write algorithm for each primitive is the row of
the same name in store.md §Write order. A batch has no nested batch.

### The amendment rule (I5)

The store enforces the half of I5 that is mechanical: no content change goes unrecorded. M2 decides
when a state edit also carries a note.

Let the **diff** of a replace be the changes between the old and the new node, computed by one pure
function in `rdm-schema` (`diff(&Node, &Node) -> Vec<Change>`), not by the caller. It reports
`criteria` and `steps` by id (`add`, `remove`, `move`, `set` with `item_id`), every other array as
one `set` of the whole array, and every other field as a `set` at its pointer. It leaves out
`/schema_version`, `/history`, `/marks`, `/status` and `/status_reason`: those are state moves,
recorded by the history and mark entries themselves. A diff entry is **state** when its pointer is
under `/progress`, or is a criterion's `/met`, or a step's `/done`, `/commit_sha` or `/done_at`
(`statuses.md`, `Finished nodes`); every other entry is **content**.

| `header.note` | Diff | Result |
|:--|:--|:--|
| absent | no content entry | applied, **no amendment** (a status move, a mark, a step recorded done) |
| absent | a content entry | `Invalid`, rule `note_required` |
| present | at least one entry | applied, **one amendment**: `description`, `source`, `correction` from the note, `actor` and `op_id` from the header, `changes` = the whole diff (state entries included: a correction of a finished node's progress is recorded) |
| present | none | `Invalid`, rule `empty_edit` (no amendment with nothing in it) |

Create, relocate and delete are structure edits: the note is required, and their `changes` are
`add`, `move` or `rename`, and `remove` as `stored-schema.md` (`Change`) defines. A comment forbids a
note. A note's `description` is non-blank and, when `correction` is true, is the reason.

### `op_id` replay

**The store owns it, not M2.** A retried write must be checked and applied inside one critical
section: the check "has this `op_id` been applied" and the write are one step under the writer lock
(store.md §Lock), and only the store holds the lock and the records. M2 sits above the lock and
could only check and then write, which two processes retrying the same `op_id` race through.

How it works, with no state beyond the records:

- Every write that is not rev-guarded leaves a record that carries the `op_id`: an amendment
  (create, relocate, delete, a noted replace) or a comment. A status move carries it in the
  `History.op_id` that M2 sets; a directory pause or cancel writes one entry per leaf with the
  same `op_id`, which is how the store recognises the whole batch (it was atomic).
- Under the lock, before applying, the store looks the `op_id` up in an **op index** (built on load
  with the reverse index, kept in step by write-through: store.md §Cache). If any record carries it,
  the store writes nothing and returns `Receipt { replayed: true, .. }` with the result rebuilt from
  those records (created ids from the amendment's `add` changes, `removed` from its `remove`
  changes, the comment id) and the *current* `tree_rev`.
- A write that carries no `op_id` is never replayed.

**Limits, stated.** (1) An edit that writes no `op_id`-bearing record (a progress edit, a mark, a
step recorded done) is not replayed, but it is rev-guarded: a retry after success meets a changed
`rev` and returns `Stale`, and the caller re-reads. (2) The store holds no fingerprint of the
request, so it cannot detect every reuse of an `op_id` for a *different* request. An `op_id` is unique
per logical operation, by the caller's contract. Where the records cannot rebuild the result the
request asks for (the node a create or relocate names is not among the amendment's changes, a
delete's subtree root is not among its `remove`s, a batch names more comments than carry the
`op_id`, the records are of another kind than the request), the store never fabricates one: it
refuses with `Invalid`, rule `op_id_reused`, and writes nothing. (3) Replay lasts as long as the records do.
Choosing the records as the ledger over a separate store of operations keeps one source of truth
and adds no file to any write order; the price is limit (1), which `expected_rev` already covers.

M2 keeps what only it can: choosing the `op_id` of a composite, setting it on every history entry
the composite writes, and finding the leaves of a directory `uncancel` through it.

## Errors

One error type for every store operation and load. M4 maps its variants to tool errors, the CLI to
exit codes; nothing maps a message string.

```rust
#[non_exhaustive]
pub enum StoreError {
    Stale { current: Current },
    Locked { holder: Option<LockHolder> },
    NewerFormat { found: Version, supported: Version, file: String },
    Invalid { file: String, rule: Rule, detail: String },
    NotFound { target: RecordRef },
    Io { file: String, message: String },
    InBatch { index: usize, source: Box<StoreError> },
}
pub enum Current {                                   // what the presented revision should have been
    Node    { id: NodeId, rev: Rev },                // a node that no longer exists is NotFound, not Stale
    Subtree { id: NodeId, rev: SubtreeRev },
    Tree    { tree_rev: TreeRev },
}
pub struct LockHolder { pub pid: u32, pub host: String, pub since: Ts }   // the record of store.md §Lock; advisory
impl StoreError {
    pub fn root(&self) -> &StoreError;               // unwraps `InBatch`
    pub fn code(&self) -> &'static str;              // of the root: STALE, LOCKED, NEWER_FORMAT, INVALID, NOT_FOUND, IO
}
```

| Variant | Raised when | Writes happened | Consumer's move |
|:--|:--|:--|:--|
| `Stale { current }` | a presented `expected_rev`, `expected_subtree_rev` or `expected_tree_rev` is not the current one (I4). `current` holds the one that should have been presented | none | re-read, reapply the intent |
| `Locked { holder }` | the writer lock was not taken within the wait of store.md §Lock. `holder` is the advisory record, `None` if it is absent or unreadable | none | retry later |
| `NewerFormat { found, supported, file }` | a file's `schema_version` is not readable by this binary (`Version chain`). Raised before anything else of the file is parsed, by a load or a write that must read it | none | upgrade; M7 may do it for the user |
| `Invalid { file, rule, detail }` | a file or a request breaks a rule. `file` is the path relative to the campaign root (`nodes/<id>.json`), or `<request>` when the fault is in the request. `rule` is the code below | none | fix the input, or repair the file by hand |
| `NotFound { target }` | a node a request names does not exist (an unknown `node_id` of a comment, an unknown `id` to replace, relocate or delete). Checked before any presented revision, so a node another writer removed is `NotFound`, never `Stale` | none | re-read |
| `Io { file, message }` | the operating system refused a read or a write | maybe a recoverable prefix (store.md §Intent and recovery) | retry or report |
| `InBatch { index, source }` | one op of a batch failed. `source` is any of the above | none: a batch is atomic | as `source` |

`Rule` is `#[non_exhaustive]`, each variant with a stable snake-case code.
M2 may add codes without changing existing ones. A warning (`unknown_key`, `dangling_reference`,
`references_empty`, and the cross-file rules of `Campaign model`) is a `Diagnostic`, not a `StoreError`.

| Rule code | Statement | Defined in |
|:--|:--|:--|
| `parse` | the file is not JSON, or not an object of the expected type | codec |
| `schema_version` | `schema_version` is missing, not a string, or not `N.N.N` | `Version chain` |
| `unknown_format` | no layer reads this version though it is not newer (older than the first layer) | `Version chain` |
| `not_in_version` | a layer reports `NotInVersion` for a behaviour the file needs | `Version chain` |
| `kind` | `kind` is not `leaf`, `directory` or `campaign` | `Node fields` |
| `malformed_id` | an id field does not parse as its type | `Id types` |
| `root_shape`, `name_format` | see `Node fields` | `Node fields` |
| `unknown_key` | a key the decoder keeps without interpreting (a warning) | `Campaign model`, `Diagnostics` |
| `unvalidated_node` | a write builds on a node that does not trace back to the root | `Campaign model` |
| `file_id_mismatch`, `sub_item_id_unique` | per-file: refuse `open` | `Campaign model` |
| `duplicate_id`, `root_count`, `root_name_mismatch`, `parent_missing`, `parent_not_directory`, `cycle`, `name_collision` | cross-file: reported by `open`, and refused as an `Invalid` when a write would introduce one | `Campaign model` |
| `dangling_id`, `reference_target_derived` | see `Links` | `Links` |
| `goal_line`, `criteria_required`, `steps_range`, `status_steps`, `status_reason`, `step_state`, `mark_target`, `correction_reason` | the rules of `stored-schema.md` (`Directory`, `Leaf`, `Step`, `Mark`, `History`), one code each, in the order they are listed there | `stored-schema.md` |
| `expected_rev_required`, `note_required`, `note_forbidden`, `empty_edit`, `id_changed`, `use_relocate`, `leaf_has_children`, `nothing_to_do`, `position_not_sibling`, `root_delete_or_move`, `op_id_reused` | request rules | `Operations` |

**Stale, replay and the lock.** `Locked` is the only error a caller retries unchanged. `Stale` is
never retried unchanged: the intent is reapplied on the fresh state. A replayed write returns
`Ok`, never `Stale`.

## Version chain

Eliott, 2026-11-25: each format version is a layer that overrides only what changed; behaviour a
version lacks defaults to "not in this version"; a field removed later is overridden back to that
default; an older file falls through to the layer that reads it; a file newer than the binary is
refused. The pattern is a **chain of responsibility** whose handlers are upcasters (as in event
sourcing): a layer reads the files of its own version into the *current* in-memory types, so the
rest of the code knows one model.

```rust
pub struct NotInVersion { pub behaviour: &'static str }   // the caller names the file and its version
pub type Behaviour<T> = Result<T, NotInVersion>;
pub struct Decoded<T> { pub value: T, pub diagnostics: Vec<Diagnostic> }   // one `unknown_key` per key kept

pub trait FormatLayer: Sync {
    /// The format line this layer reads: (major, minor). A patch bump (Y) is compatible and has no layer.
    fn line(&self) -> (u16, u16);
    /// The layer this one overrides. `&Nothing` under the first layer.
    fn base(&self) -> &dyn FormatLayer;
    // Each behaviour defaults to the base layer's. A layer overrides only the ones that changed.
    // The outer Result says whether the version has the behaviour; the inner one whether this file decodes.
    fn decode_node(&self, file: &str, raw: serde_json::Value) -> Behaviour<Result<Decoded<Node>, StoreError>>           { self.base().decode_node(file, raw) }
    fn decode_comment(&self, file: &str, raw: serde_json::Value) -> Behaviour<Result<Decoded<Comment>, StoreError>>     { self.base().decode_comment(file, raw) }
    fn decode_amendment(&self, file: &str, raw: serde_json::Value) -> Behaviour<Result<Decoded<Amendment>, StoreError>> { self.base().decode_amendment(file, raw) }
}
pub struct Nothing;   // the root of the chain: every behaviour is Err(NotInVersion)
pub static CHAIN: &[&dyn FormatLayer];                          // oldest first; one entry today
pub fn is_newer(found: Version, supported: Version) -> bool;    // the whole triple, compared field by field
pub fn layer_for(found: Version) -> Option<&'static dyn FormatLayer>;  // the newest layer whose line <= found's
```

- **`NotInVersion`.** `Nothing` implements every behaviour as `Err(NotInVersion)`, so a behaviour no
  layer defines is absent by default. A layer **removes** a behaviour by overriding it back to
  `Err(NotInVersion)`: that stops the lookup before it reaches an older layer that still has it.
  A caller turns `NotInVersion` into `Invalid` with the rule `not_in_version`, naming the file
  and its version in the detail (`Nothing` has no version of its own to report). No behaviour is
  absent today, so no code path reaches it yet.
- **A layer is a struct holding `base`.** It overrides a behaviour by implementing the method, and
  inherits every other by not mentioning it. The layer of the current format (`0.1`) decodes with
  the serde of the types of this document, so its decoders are one line each. The decoded value is
  always the current type, with every unknown key kept in `extra` (I12) and reported as one
  `unknown_key` diagnostic in the returned `Decoded` (the decode produces it, `open` aggregates
  it). Old data is *upgraded on read*, never converted in place.
- **There is one encoder, not one per layer.** Writing always serialises the current types. A
  layer cannot write, so no version can be written that is not the current one.

### Reading order

The version is read first. Every file read, a node, a comment or an amendment, goes through the
same steps, in this order:

1. Parse the bytes **once** into a `serde_json::Value` and read **only** `schema_version` from it.
   No typed value is built, no id is validated and no shape rule runs; the same value is handed
   to the layer in step 5, so the file is never parsed twice. The text must be valid JSON and its
   top level an object; otherwise `Invalid` (rule `parse`). A missing or non-string stamp is
   `Invalid` (rule `schema_version`).
2. `parse_version` the stamp (strict, as today). A malformed one is `Invalid` (rule `schema_version`).
3. If `is_newer(found, FORMAT_VERSION)`, return `NewerFormat { found, supported: FORMAT_VERSION, file }`
   **and stop**. Nothing of the file is parsed past the stamp. A load that meets such a file fails
   as a whole (a per-file fault, `Campaign model`); the error names the file.
4. Otherwise pick `layer_for(found)`. If there is none (older than the first layer), `Invalid`
   (rule `unknown_format`).
5. Call the layer's `decode_*` on the value. An older version falls through to its own layer
   here, not through the current one, and the layer's inherited behaviours come from its `base`.

**Every newer file is refused, a higher Y included** (Eliott, 2026-11-25). He is not convinced a
patch change will only ever add optional fields, and an older binary that misread one would break
the rule that old code never misreads a field. So `is_newer` compares the whole triple, and `0.1.1`
is refused by a `0.1.0` binary. This **overrules** the earlier reading of the `Format version`
table, which read a higher Y; stored-schema.md is edited to match. Unknown keys still survive in
place (I12), but only in a file the binary can read at all; each one is reported
(`unknown_key`).

`is_newer` replaces `is_readable`, which is removed. **One rule serves stored files and the read
model** (Eliott, 2026-11-25): a consumer refuses a projection whose `read_model_version` is newer than
its own, a higher Y included, and reads an older one (`read-model.md`, `Versions`, decision 10). An
older stored file goes to its layer, or `unknown_format` (step 4). Making the refusal transparent is M7's (A2, ruling 5, and the answer recorded
for M7).

### Write-version rule

**An edit rewrites the files it touches in the current version, and no others.** A write encodes
the whole file with the current types and stamps `FORMAT_VERSION`: the store sets
`schema_version` on the value before it calls `encode`, which is a pure serialiser and stamps
nothing. A file the operation does not
touch keeps its version; **a read never writes** (readers need no coordination). So one campaign may
hold files of different versions, and of different patch levels, and that is legal.

Consequences, accepted: after a new binary edits a node, an older binary refuses the campaign on
that file with `NewerFormat`, which is the refusal doing its job, and now whether the bump was X or
Y. Rewriting on touch (not keeping the old stamp) is the only choice that needs no per-layer
encoder, and it stops an old shape from outliving the code that reads it. A stamp never
understates its content, since a binary cannot read, hence cannot rewrite, a file newer than it.

### `FORMAT_VERSION`

**It stays `0.1.0`, redefined, and is frozen when M1 merges to `main`** (decided, Eliott, 2026-11-25). Nothing is stored yet: the importer is M3, the store is built after this
document, and the only `0.1.0` files that exist are the crate's own hand-written fixtures, in a
shape this document replaces (the flat node fields, no ids on checks and references, no `Change`
paths). There is no old file to read, so no layer for the old shape, and the campaign rule
"fixtures come only from real producers" forbids writing one. `0.1.0` is therefore **redefined**
by this document, and the chain starts with one layer, `0.1`. The fixtures are rewritten, not read.

**From now on**, with the freeze at the merge of the M1 campaign into `main` (until then a change
edits layer `0.1` in place and amends this document):

| Change to a stored type | Bump | Chain |
|:--|:--|:--|
| a field removed, renamed, given another meaning, or made required; a record kind removed | X (`0.X.y`) | a **new layer**, with `base` the previous one; `CHAIN` gains an entry; the older layer stays |
| a new optional field, a new enum value, a new record kind in a new directory: additive in meaning | Y (`0.x.Y`) | none: the same layer reads it. **Older binaries refuse it too** (`NewerFormat`); the bump is a promise about meaning, not about compatibility |
| a field added with a default for old files | Y if optional, else X | a layer only if the default needs code |

A bump changes, in one commit: `FORMAT_VERSION`, the types, the committed JSON Schemas (re-blessed
and reviewed), `stored-schema.md`, and, for X, the layer. The commit type and scope decide the
crate version (`stored-schema.md`, `Release`; wiring is M7's). Nothing checks "bumped when the
schema changed" automatically: the schema drift guard shows the change in the diff, and a reviewer
decides X or Y.

## Invariants and guards

Every invariant of this document, with its final guard. The names are those of the campaign
README, of [store.md](store.md) for what it owns, or **new** (marked †, to be added to the README's
guard list by the coordinator). Several rows share one guard when a distinct failure each row
turns red is a parameter row of it (the guards below say which). "Not tested" is followed by the
reason.

| # | Invariant | Guarded by |
|:--|:--|:--|
| C1 | An id field holds a string of the format of store.md §Ids with its type's kind prefix; a malformed one is refused at deserialisation | `malformed_ids_are_refused` |
| C2 | `NodeId`s are unique in the campaign; `CriterionId`s and `StepId`s are unique within their node file | `open_names_invalid_campaigns` (rows: a sub-item duplicate is refused; a cross-file `duplicate_id` is reported) |
| C3 | Checks and references have no id; `Change.item_id` names a criterion or a step | `committed_schemas_match_the_types` (the generated schema has no such field) |
| C4 | A node holds `name`, `parent_id` and `order`; the root shape rule holds | `open_names_invalid_campaigns` (row: per-file faults are refused) |
| C5 | Per-file faults refuse `open`; each cross-file defect is reported, naming file and rule, and the sound files load; nodes off the root are unvalidated | `open_names_invalid_campaigns` (rows: per-file faults are refused; each cross-file defect is reported naming file and rule, and the sound files load; no parentless node with the slug leaves every node unvalidated) |
| C6 | Children, depth and path are derived from the `parent_id` chain and never stored; the incremental indexes equal a rebuild | `reverse_index_equals_brute_force_scan` (the proptest also compares children, depth and path with a rebuild) |
| C7 | A move or rename writes one node file and one amendment, and changes no id (I7) | `move_writes_one_file_and_keeps_ids` |
| C8 | `MOVES` equals the move table of `statuses.md` | `move_table_matches_statuses_doc` |
| C9 | `LINKS` lists every id field of every record | `links_cover_every_id_field` |
| C10 | The reverse index equals a brute-force scan, after any operation sequence | `reverse_index_equals_brute_force_scan` |
| C11 | A delete rewrites nothing outside the subtree, and reports every `Report` referrer | `delete_reports_referrers_and_touches_only_the_subtree` † |
| C12 | No stored field holds a value derivable from an id: a `Reference` with both `node_id` and `target` is refused | `open_names_invalid_campaigns` (row: an editor adds `target` beside `node_id`) |
| C13 | The committed JSON Schemas equal those generated from the types, request and result types included | `committed_schemas_match_the_types` |
| C14 | Every object keeps the keys it does not know, in place, at every level (I12), and each one yields exactly one `unknown_key` diagnostic with its JSON path | `unknown_keys_survive_in_place_at_every_level` (extended: also asserts one diagnostic per unknown key, with its path) |
| C15 | A canonical file round-trips byte for byte; valid non-canonical input reads and re-encodes canonically | `canonical_files_round_trip_byte_for_byte` (the codec guard of the campaign) |
| C16 | A file newer than the binary, whether the higher part is X or Y, is refused with `NewerFormat { found, supported }` before anything else of it is parsed | `newer_file_refused_before_parsing` (rows: a higher X; a higher Y; each a hand-written canonical file of the shape a newer layer would write, with a body that would not parse as the current type) |
| C17 | An older version reads through its own layer; a removed behaviour is `NotInVersion` | Not tested: one layer exists, and the README defers chain fall-through "until a second real layer exists" |
| C18 | An edit rewrites only the files it touches, in the current version; a read writes nothing | Not tested for the version stamp: with one layer every write is the current version, so the test would restate the encoder (as C17). "A read writes nothing" is `cached_read_equals_cold_read` of store.md (the files do not change) |
| C19 | A stale `expected_rev`, `expected_subtree_rev` or `expected_tree_rev` changes nothing (I4) | `stale_replace_changes_nothing` |
| C20 | No content change goes unrecorded; an amendment is written only with a note, and never empty (I5) | `requests_breaking_a_rule_are_refused` † (rows: `note_required`, `empty_edit`); the success path is exercised by every store write with a note |
| C21 | A replace cannot change `id`, `name`, `parent_id` or `order` | `requests_breaking_a_rule_are_refused` † (row: a replace that changes `parent_id` or `name` is refused with `use_relocate`) |
| C22 | A batch is atomic: a failed op leaves nothing written; a dry run writes nothing | `every_write_prefix_loads_and_recovers_to_pre_or_post` (store.md) for the crash side; `failed_batch_writes_nothing` † for the logical side |
| C23 | A retried `op_id` returns the first outcome and writes nothing | `replayed_op_id_writes_nothing` † |
| C24 | A held lock yields `Locked { holder }` and is released when the holder dies | `lock_survives_holder_kill` (store.md) |
| C25 | Every `StoreError` variant has a stable `code()`, and `root()` unwraps `InBatch` | Not tested: a total `match` the compiler checks; M4 tests its own mapping |
| C26 | `FORMAT_VERSION` is `0.1.0` and its string round-trips | `version_constants_and_strings` (existing) |
| C27 | A bump of X or Y accompanies a change to a stored type | Not tested: needs the git history of the schemas; a reviewer reads the drift diff |
| C28 | A write builds on no unvalidated node, except to repair; a write never adds a defect | `requests_breaking_a_rule_are_refused` † (rows: create under, and replace of, an unvalidated node are refused with `unvalidated_node`; relocate into the tree, rename of a colliding sibling or of the root, and delete are accepted) |
| C29 | The diagnostics of `open` cannot be dropped silently: `Opened` is `#[must_use]`, `depth` and `path` are `Option` | Not tested: a compiler check (`-D warnings` holds `unused_must_use`), and the types are the guard |

### Guards by leaf

What each implementing leaf owes, with the bug that turns each red (the leaf report records it
seen red).

| Leaf | Guard | Red when |
|:--|:--|:--|
| `schema_contracts` | `malformed_ids_are_refused` | an id newtype accepts a wrong prefix, a wrong length or a char outside the alphabet |
| | `committed_schemas_match_the_types` | a field is added to a type and the schema is not re-blessed |
| | `move_table_matches_statuses_doc` | one cell of `statuses.md` changes and `MOVES` does not |
| | `links_cover_every_id_field` | an id field is added to a record and not to `LINKS` |
| | `version_constants_and_strings` | `FORMAT_VERSION` or its string changes unnoticed |
| `codec_and_versions` | `newer_file_refused_before_parsing` | the newer-file check runs after the typed decode (a newer body then fails as `parse`), or it ignores the patch |
| | `unknown_keys_survive_in_place_at_every_level` | a level drops a key, moves it, or reports none or two diagnostics |
| | `canonical_files_round_trip_byte_for_byte` | the encoder reorders keys or changes the indent |
| `campaign_store` | `stale_replace_changes_nothing` | the revision is compared before the lock is taken |
| | `move_writes_one_file_and_keeps_ids` | a move rewrites a child or a parent, or re-mints an id |
| | `requests_breaking_a_rule_are_refused` | a content change passes without a note, an empty note writes an amendment, a replace moves a node, or a write builds on an unvalidated node |
| | `replayed_op_id_writes_nothing` | the `op_id` lookup runs before the lock, or after the write |
| | `failed_batch_writes_nothing` | a batch applies its ops one by one with no whole-batch check |
| | `delete_reports_referrers_and_touches_only_the_subtree` | a delete rewrites a `Reference.node_id` outside the subtree, or omits a referrer from the report |
| | `open_names_invalid_campaigns` | an orphan, a cycle, a name collision or a second root is silently loaded as sound; or a per-file fault is only reported; or a node off the root gets a path |
| | `reverse_index_equals_brute_force_scan` | `replace_record` forgets to drop a record's old entries, or `remove_subtree` leaves a stale child |

The coordinator assigned the leaves (2026-11-25): `open` and the reverse index are guarded through
the store, where a real merge produces the damage. store.md §Invariants and guards lists the store's
own guards (revisions, minting, order keys, crash prefixes, lock, cache).

### Existing tests that change on purpose

`schema_contracts` changes these first, so each failure is a decision and not a surprise:

| Test (`crates/rdm-schema/tests/`) | Why its behaviour changes |
|:--|:--|
| `stored.rs`: `ids_are_unique_and_changes_name_items_by_id` | checks and references no longer have ids, so the collected id set shrinks to criteria and steps; the ids are typed; the fixture change `item_id` names a step |
| `stored.rs`: `reordering_criteria_leaves_marks_valid` | it builds a `Criterion` with the literal id `cg_new`, which is now malformed; it mints a valid `CriterionId` |
| `stored.rs`: `unknown_keys_keep_their_order_in_the_output_string`, `unknown_field_keeps_its_place_after_known_ones` | their inline JSON uses ids such as `"c"` and `"n"`, now malformed |
| `stored.rs`: `amendment_fixture_round_trips_and_keeps_null_new` | the fixture's `remove` and `move` changes carry `path_before` and `path_after`, which `Change` no longer has |
| `stored.rs`: `unknown_fields_survive_at_every_level` | gains the assertion of exactly one `unknown_key` diagnostic per injected key, with its path |
| `stored.rs`: `leaf_fixture_round_trips`, `directory_fixture_round_trips`, `comment_fixture_round_trips` | every fixture is rewritten with valid ids, the node fields, no check or reference ids, and no `Reference.target` beside a `node_id`; the injection counts are recomputed |
| `stored.rs`: `marks_point_by_criterion_id_and_corrections_are_flagged` | reads fixture ids that become typed |
| `schemas.rs`: `committed_schemas_match_the_types` | the generated schemas change on purpose and are re-blessed under review; request and result schemas join the list |
| `read.rs` and `fixtures/{outline,node,ready,ready_scoped,amendments}.json` | read-model id fields (`id`, `leaf_ids`, `held_by`, the holder keys) become the newtypes (`parent` is a path in `read-model.md` and stays a string), and `rev` and `tree_rev` become `Rev` and `TreeRev`; their fixtures hold valid ids. The read model stays a projection, outside `LINKS` |
| `Reference.target` fixtures (`stored_leaf.json`, `node.json`) | `target` is relative to the campaign root, not the node file (`../../../__design__/…` becomes `../../__design__/…` from `__roadmap__/<campaign>/`); a reference with `node_id` has none |

`parse_version_is_strict` does not change. `reader_refuses_per_the_read_model_rule` changes on
purpose, before the code: `is_readable` is removed, and the guard becomes
`only_newer_versions_are_refused`, over `is_newer`, with rows a higher major, a higher X and a higher
Y refused, and the same and lower versions accepted (bug caught: the patch ignored, or an older
version refused). The stored-file path is guarded besides by `newer_file_refused_before_parsing`.
