//! The campaign model (`contracts.md`, `Campaign model`): the nodes of one campaign as a table
//! by id, with the diagnostics of the load that produced it.
//!
//! The model is data. Children, depth and path are derived from `parent_id`, `name` and `order`
//! by the engine, never stored (I6), and `open` that builds the table is the engine's.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::Rule;
use crate::ids::NodeId;
use crate::stored::Node;

/// One warning: a defect the load reports and goes on from. An error is a `StoreError`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct Diagnostic {
    /// The file the defect is in, relative to the campaign root.
    pub file: String,
    pub rule: Rule,
    /// What is wrong; for `unknown_key`, the key's JSON pointer.
    pub detail: String,
}

/// A record decoded from a file, with the warnings of the decode: one `unknown_key` per key kept
/// without interpretation (`contracts.md`, `Version chain`).
#[derive(Debug, Clone, PartialEq)]
pub struct Decoded<T> {
    pub value: T,
    pub diagnostics: Vec<Diagnostic>,
}

/// The nodes of one campaign, by id.
#[derive(Debug, Clone, PartialEq)]
pub struct Campaign {
    nodes: BTreeMap<NodeId, Node>,
    root: Option<NodeId>,
    unvalidated: BTreeSet<NodeId>,
    diagnostics: Vec<Diagnostic>,
}

/// An opened campaign with the diagnostics of its load.
#[must_use = "an opened campaign carries diagnostics; read them"]
#[derive(Debug, Clone, PartialEq)]
pub struct Opened {
    pub campaign: Campaign,
    pub diagnostics: Vec<Diagnostic>,
}

impl Campaign {
    /// Builds the model from what a load found: every node loaded, the validated root if one
    /// carries the expected slug, the set of loaded nodes that do not trace back to it, and the
    /// diagnostics.
    #[must_use]
    pub fn from_parts(
        nodes: BTreeMap<NodeId, Node>,
        root: Option<NodeId>,
        unvalidated: BTreeSet<NodeId>,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        Self { nodes, root, unvalidated, diagnostics }
    }

    /// The diagnostics of the load, kept on the value so every later read can surface them again.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Whether the node is loaded and traces back to the root. A node that is loaded but
    /// unvalidated, or not loaded, is not.
    #[must_use]
    pub fn is_validated(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id) && !self.unvalidated.contains(id)
    }

    /// The validated root; `None` when no parentless node carries the expected slug.
    #[must_use]
    pub fn root(&self) -> Option<&NodeId> {
        self.root.as_ref()
    }

    /// A loaded node, validated or not.
    #[must_use]
    pub fn get(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Every loaded node by `NodeId`, validated or not.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }
}
