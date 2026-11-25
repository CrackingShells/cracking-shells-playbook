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
