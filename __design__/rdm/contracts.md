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

**Verdict on the proposal: confirmed.** Nodes, criteria and steps keep ids. Checks and references
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
| `name` | `Name` | every node but the campaign root | yes | The path segment, `^[a-z][a-z0-9_-]*$`. Unique among the children of one parent. Shown in paths and by M3's export |
| `parent_id` | `NodeId` | every node but the campaign root | yes | The directory this node is in. Always a `Directory` node |
| `order` | `OrderKey` | every node but the campaign root | yes | Position among siblings. An opaque newtype: its format and how a key is made between two others are store.md's (§Order key). Compared as given, never parsed. Ties break by `id` |
| `kind` | `DirectoryKind` or `LeafKind` | every node | yes | `campaign` exactly when `parent_id` is absent, so the type of the root is checked against its place |

On `Directory`, `name`, `parent_id` and `order` are `Option`: they are absent for the campaign root
and for it only (`root_shape` rule below). On `Leaf` they are required, so a leaf cannot be a root
by construction. The campaign root has no name of its own: its path is `""` and the campaign's
location is the argument of every call (`read-model.md`).

```rust
pub struct Name(String);                  // parse(&str) -> Result<Name, NameError>
pub struct OrderKey(String);              // opaque; format in store.md §Order key
pub enum Node { Directory(Directory), Leaf(Leaf) }   // discriminated by `kind` in the file
impl Node {
    pub fn id(&self) -> &NodeId;
    pub fn name(&self) -> Option<&Name>;
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
| `root_shape` | `parent_id`, `name` and `order` are all absent exactly when `kind` is `campaign` |
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
pub struct Campaign { /* nodes: BTreeMap<NodeId, Node>; root: NodeId; children index; depth memo */ }
impl Campaign {
    pub fn load(nodes: impl IntoIterator<Item = Node>) -> Result<Campaign, StoreError>;  // Invalid on a shape rule
    pub fn root(&self) -> &NodeId;
    pub fn get(&self, id: &NodeId) -> Option<&Node>;
    pub fn nodes(&self) -> impl Iterator<Item = &Node>;                 // by NodeId
    pub fn parent(&self, id: &NodeId) -> Option<&NodeId>;               // None for the root
    pub fn children(&self, id: &NodeId) -> &[NodeId];                   // sorted by (order, id); empty for a leaf
    pub fn depth(&self, id: &NodeId) -> u32;                            // I1; see below
    pub fn path(&self, id: &NodeId) -> String;                          // "" for the root
    pub fn resolve(&self, path: &str) -> Option<&NodeId>;               // inverse of `path`
    pub fn subtree(&self, id: &NodeId) -> impl Iterator<Item = &NodeId>; // pre-order, `id` first
    pub fn insert(&mut self, node: Node);                               // incremental: keeps the
    pub fn remove_subtree(&mut self, id: &NodeId) -> Vec<NodeId>;       // derived indexes equal to a rebuild
}
```

| Derived | Definition | Never stored because | Relied on by |
|:--|:--|:--|:--|
| `children(id)` | the nodes whose `parent_id` is `id`, sorted by `(order, id)` | a child list in the parent would be a second copy that a move must edit in two files | M2 (waves, ready set), the read model |
| `depth(id)` | the number of **directories between the campaign root and the node**: a node directly under the root has depth 0, so this is `level` in `read-model.md`. The ordering rule I1 is this number, computed from the `parent_id` chain | it would go stale on every move of an ancestor | M2 (ready set), the read model |
| `path(id)` | the `name`s from the root's child down to the node, joined by `/`; `""` for the root | it would go stale on every rename or move above it, and a stored path is the value a flat layout exists to avoid | M3 (export), the read model, M4 (a tool takes a path or an id) |

**Shape rules**, checked by `Campaign::load` and by every write before it is acknowledged (I3). Each
is an `Invalid` rule code naming the offending file:

| Rule code | Statement |
|:--|:--|
| `file_id_mismatch` | `nodes/<id>.json` holds that `id` (a copy or a hand-rename of a file is refused, not guessed) |
| `duplicate_id` | no two files hold the same `NodeId` (a bad merge) |
| `root_count` | exactly one node has `kind: campaign` |
| `parent_missing` | every `parent_id` names a node of the campaign |
| `parent_not_directory` | every `parent_id` names a `Directory` |
| `cycle` | following `parent_id` from any node reaches the root |
| `name_collision` | two children of one parent never share a `name` |

`load` builds the children index in one pass, then computes depth by walking each parent chain with
a memo and a visiting mark (a depth-first search with three colours), so a cycle is found, not
followed, and the whole load is O(n). `insert` and `remove_subtree` update the same indexes in
O(depth) and O(subtree); a rebuild is the oracle for them (`derived_views_equal_a_rebuild`, below).

Siblings are parallel (I1): `order` only fixes how they are listed, never when they run, and no
stored field expresses a dependency between them. `Campaign` holds nodes only. Comments and
amendments are separate files, read by the cache (store.md §Cache) and joined to a node by id.

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
