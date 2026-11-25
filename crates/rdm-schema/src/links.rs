//! The declared links (`contracts.md`, `Links`): every field of one record that holds the id of
//! another, so an update or a delete reaches every reference and a value derivable from an id is
//! never stored. The reverse index is derived from [`LINKS`] on load, by the engine.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ids::{AmendmentId, CommentId, NodeId};

/// The file kind that holds a link field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Owner {
    Directory,
    Leaf,
    Comment,
    Amendment,
}

/// What a link points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target {
    Node,
    Criterion,
    Step,
    /// A criterion or a step (`ItemId`).
    Item,
}

/// Where the target of a link is looked up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    /// A node anywhere in the campaign.
    Campaign,
    /// A criterion or step of the file that holds the link.
    OwnerFile,
    /// A criterion or step of the node a change names.
    ChangeNode,
}

/// What a delete of the target does to the record that holds the link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OnDelete {
    /// The referrer is inside the deleted subtree and goes with it.
    Cascade,
    /// The referrer stays untouched and the delete reports it.
    Report,
    /// The referrer is history and is never edited.
    Keep,
}

/// One link: a field of one record that holds the id of another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Link {
    pub owner: Owner,
    /// JSON pointer to the field; `*` is any array index and `{key}` is a map key.
    pub pointer: &'static str,
    pub target: Target,
    pub scope: Scope,
    pub on_delete: OnDelete,
}

const fn link(owner: Owner, pointer: &'static str, target: Target, scope: Scope, on_delete: OnDelete) -> Link {
    Link { owner, pointer, target, scope, on_delete }
}

/// Every link of every record. The `id` field of a record is its identity, not a link, and is
/// not listed. Values copied into opaque JSON (a snapshot's values, `old` and `new` of a change)
/// are historical copies, not links.
pub static LINKS: [Link; 11] = [
    link(Owner::Directory, "/parent_id", Target::Node, Scope::Campaign, OnDelete::Cascade),
    link(Owner::Leaf, "/parent_id", Target::Node, Scope::Campaign, OnDelete::Cascade),
    link(Owner::Directory, "/references/*/node_id", Target::Node, Scope::Campaign, OnDelete::Report),
    link(Owner::Leaf, "/references/*/node_id", Target::Node, Scope::Campaign, OnDelete::Report),
    link(Owner::Leaf, "/steps/*/references/*/node_id", Target::Node, Scope::Campaign, OnDelete::Report),
    link(Owner::Directory, "/marks/*/criterion_id", Target::Criterion, Scope::OwnerFile, OnDelete::Keep),
    link(Owner::Leaf, "/marks/*/criterion_id", Target::Criterion, Scope::OwnerFile, OnDelete::Keep),
    link(Owner::Comment, "/node_id", Target::Node, Scope::Campaign, OnDelete::Keep),
    link(Owner::Amendment, "/changes/*/node_id", Target::Node, Scope::Campaign, OnDelete::Keep),
    link(Owner::Amendment, "/changes/*/item_id", Target::Item, Scope::ChangeNode, OnDelete::Keep),
    link(Owner::Amendment, "/snapshot/{key}", Target::Node, Scope::Campaign, OnDelete::Keep),
];

/// A record that holds a link.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecordRef {
    Node(NodeId),
    Comment(CommentId),
    Amendment(AmendmentId),
}

impl fmt::Display for RecordRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(id) => write!(f, "node {id}"),
            Self::Comment(id) => write!(f, "comment {id}"),
            Self::Amendment(id) => write!(f, "amendment {id}"),
        }
    }
}

/// One record that points at a node, as the reverse index and the delete report give it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct Referrer {
    /// The index of the row of [`LINKS`] this referrer follows.
    pub link: usize,
    pub from: RecordRef,
    /// The concrete JSON pointer of the field in `from`.
    pub at: String,
}
