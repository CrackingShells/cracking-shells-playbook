//! The campaign model (`contracts.md`, `Campaign model`): the nodes of one campaign as a table
//! by id, with everything about the tree derived from `parent_id`, `name` and `order`, never
//! stored (I6), and the diagnostics of the load that produced it.
//!
//! Pure: no I/O. The engine's `Store` reads and decodes the files and calls [`Campaign::open`].

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Rule, StoreError};
use crate::ids::{Name, NodeId};
use crate::stored::{Node, Reference};
use crate::validate::check_node;

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

/// The file a node lives in, relative to the campaign root: `nodes/<id>.json`.
#[must_use]
pub fn node_file(id: &NodeId) -> String {
    format!("nodes/{id}.json")
}

/// The nodes of one campaign, by id, and what is derived from them.
#[derive(Debug, Clone, PartialEq)]
pub struct Campaign {
    slug: Name,
    nodes: BTreeMap<NodeId, Node>,
    /// Ids that two or more records held; only the first record is kept.
    duplicates: BTreeMap<NodeId, usize>,
    /// Children by parent, sorted by `(order, id)`.
    children: BTreeMap<NodeId, Vec<NodeId>>,
    root: Option<NodeId>,
    /// Depth and path of every validated node.
    placed: BTreeMap<NodeId, (u32, String)>,
    by_path: BTreeMap<String, NodeId>,
    diagnostics: Vec<Diagnostic>,
}

/// An opened campaign with the diagnostics of its load.
#[must_use = "an opened campaign carries diagnostics; read them"]
#[derive(Debug, Clone, PartialEq)]
pub struct Opened {
    pub campaign: Campaign,
    pub diagnostics: Vec<Diagnostic>,
}

fn order_key(n: &Node) -> &str {
    n.order().map_or("", |o| o.as_str())
}

fn diag(id: &NodeId, rule: Rule, detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        file: node_file(id),
        rule,
        detail: detail.into(),
    }
}

impl Campaign {
    /// The only constructor. `expected_slug` is the campaign's name as its caller knows it: the
    /// root is the parentless node that carries it.
    ///
    /// # Errors
    /// `Invalid` naming `nodes/<id>.json` and the rule, for the first node (in id order) that
    /// breaks a rule of its own record (`validate::check_node`). Damage between files is not an
    /// error: it loads, and is reported in [`Opened::diagnostics`].
    pub fn open(
        nodes: impl IntoIterator<Item = Node>,
        expected_slug: &Name,
    ) -> Result<Opened, StoreError> {
        let mut table = BTreeMap::new();
        let mut duplicates: BTreeMap<NodeId, usize> = BTreeMap::new();
        for node in nodes {
            let id = node.id().clone();
            match table.entry(id) {
                std::collections::btree_map::Entry::Occupied(e) => {
                    *duplicates.entry(e.key().clone()).or_insert(1) += 1;
                }
                std::collections::btree_map::Entry::Vacant(e) => {
                    e.insert(node);
                }
            }
        }
        for (id, node) in &table {
            check_node(node).map_err(|f| StoreError::Invalid {
                file: node_file(id),
                rule: f.rule,
                detail: f.detail,
            })?;
        }
        let mut campaign = Self {
            slug: expected_slug.clone(),
            nodes: table,
            duplicates,
            children: BTreeMap::new(),
            root: None,
            placed: BTreeMap::new(),
            by_path: BTreeMap::new(),
            diagnostics: Vec::new(),
        };
        campaign.derive();
        let diagnostics = campaign.diagnostics.clone();
        Ok(Opened {
            campaign,
            diagnostics,
        })
    }

    /// Rebuilds everything derived from the table: children, the root, the validated set with
    /// depth and path, and the cross-file diagnostics.
    fn derive(&mut self) {
        let mut children: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
        for (id, node) in &self.nodes {
            if let Some(p) = node.parent_id() {
                children.entry(p.clone()).or_default().push(id.clone());
            }
        }
        for list in children.values_mut() {
            list.sort_by(|a, b| {
                (order_key(&self.nodes[a]), a).cmp(&(order_key(&self.nodes[b]), b))
            });
        }
        let mut diags = Vec::new();

        for (id, n) in &self.duplicates {
            diags.push(diag(
                id,
                Rule::DuplicateId,
                format!("{n} records hold the id {id}"),
            ));
        }

        // The root: the parentless node that carries the slug.
        let parentless: Vec<&NodeId> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.parent_id().is_none())
            .map(|(id, _)| id)
            .collect();
        let (named, others): (Vec<&NodeId>, Vec<&NodeId>) = parentless
            .into_iter()
            .partition(|id| self.nodes[*id].name() == &self.slug);
        for id in &others {
            diags.push(diag(
                id,
                Rule::RootNameMismatch,
                format!(
                    "the parentless node is named `{}`, not `{}`",
                    self.nodes[*id].name(),
                    self.slug
                ),
            ));
        }
        // Two roots after a merge of two inits: no arbitrary tree is validated.
        let root = match named.as_slice() {
            [only] if !self.duplicates.contains_key(*only) => Some((*only).clone()),
            _ => None,
        };
        match named.len() {
            0 => diags.push(Diagnostic {
                file: "nodes/".to_owned(),
                rule: Rule::RootCount,
                detail: format!("no parentless node is named `{}`", self.slug),
            }),
            1 => {}
            _ => {
                let all: Vec<String> = named.iter().map(|id| id.to_string()).collect();
                diags.push(diag(
                    named[0],
                    Rule::RootCount,
                    format!(
                        "{} parentless nodes are named `{}`: {}",
                        named.len(),
                        self.slug,
                        all.join(", ")
                    ),
                ));
            }
        }

        // A parent that is missing or is a leaf.
        for (id, node) in &self.nodes {
            let Some(p) = node.parent_id() else { continue };
            match self.nodes.get(p) {
                None => diags.push(diag(
                    id,
                    Rule::ParentMissing,
                    format!("`parent_id` names no node: {p}"),
                )),
                Some(Node::Leaf(_)) => {
                    diags.push(diag(
                        id,
                        Rule::ParentNotDirectory,
                        format!("`parent_id` names a leaf: {p}"),
                    ));
                }
                Some(Node::Directory(_)) => {}
            }
        }

        // Cycles in the `parent_id` chain: one diagnostic per cycle.
        let mut settled: BTreeSet<&NodeId> = BTreeSet::new();
        for start in self.nodes.keys() {
            let mut path: Vec<&NodeId> = Vec::new();
            let mut at = Some(start);
            while let Some(id) = at {
                if settled.contains(id) {
                    break;
                }
                if let Some(pos) = path.iter().position(|p| *p == id) {
                    let mut members: Vec<&NodeId> = path[pos..].to_vec();
                    members.sort();
                    let list: Vec<String> = members.iter().map(|m| m.to_string()).collect();
                    diags.push(diag(
                        members[0],
                        Rule::Cycle,
                        format!("following `parent_id` cycles through {}", list.join(", ")),
                    ));
                    break;
                }
                path.push(id);
                at = self
                    .nodes
                    .get(id)
                    .and_then(Node::parent_id)
                    .filter(|p| self.nodes.contains_key(*p));
            }
            settled.extend(path);
        }

        // Two siblings with one name.
        let mut colliding: BTreeSet<NodeId> = BTreeSet::new();
        for (parent, list) in &children {
            if !self.nodes.contains_key(parent) {
                continue;
            }
            let mut by_name: BTreeMap<&str, Vec<&NodeId>> = BTreeMap::new();
            for id in list {
                by_name
                    .entry(self.nodes[id].name().as_str())
                    .or_default()
                    .push(id);
            }
            for (name, mut group) in by_name {
                if group.len() < 2 {
                    continue;
                }
                group.sort();
                let others: Vec<String> = group.iter().skip(1).map(|g| node_file(g)).collect();
                diags.push(diag(
                    group[0],
                    Rule::NameCollision,
                    format!("the name `{name}` is also used by {}", others.join(", ")),
                ));
                colliding.extend(group.into_iter().cloned());
            }
        }

        // Validated: reachable from the root through directories, skipping colliding siblings.
        let mut placed = BTreeMap::new();
        let mut by_path = BTreeMap::new();
        if let Some(r) = &root {
            placed.insert(r.clone(), (0, String::new()));
            by_path.insert(String::new(), r.clone());
            let mut queue = vec![r.clone()];
            while let Some(d) = queue.pop() {
                if !matches!(self.nodes[&d], Node::Directory(_)) {
                    continue;
                }
                let (depth, path) = placed[&d].clone();
                for c in children.get(&d).map_or(&[][..], Vec::as_slice) {
                    if colliding.contains(c)
                        || self.duplicates.contains_key(c)
                        || placed.contains_key(c)
                    {
                        continue;
                    }
                    let name = self.nodes[c].name();
                    let (cd, cp) = if d == *r {
                        (0, name.to_string())
                    } else {
                        (depth + 1, format!("{path}/{name}"))
                    };
                    placed.insert(c.clone(), (cd, cp.clone()));
                    by_path.insert(cp, c.clone());
                    queue.push(c.clone());
                }
            }
        }

        // The warnings the link table gives.
        for (id, node) in &self.nodes {
            let mut check = |pointer: String, r: &Reference| {
                if let Some(target) = r.node_id.as_ref().filter(|t| !self.nodes.contains_key(*t)) {
                    diags.push(diag(
                        id,
                        Rule::DanglingReference,
                        format!("{pointer}/node_id names no node: {target}"),
                    ));
                }
            };
            let (own, steps): (&[Reference], &[crate::stored::Step]) = match node {
                Node::Directory(d) => (&d.references, &[]),
                Node::Leaf(l) => (&l.references, &l.steps),
            };
            for (i, r) in own.iter().enumerate() {
                check(format!("/references/{i}"), r);
            }
            for (si, step) in steps.iter().enumerate() {
                for (i, r) in step.references.iter().enumerate() {
                    check(format!("/steps/{si}/references/{i}"), r);
                }
            }
            if matches!(node, Node::Leaf(_)) && own.is_empty() {
                diags.push(diag(id, Rule::ReferencesEmpty, "the leaf has no reference"));
            }
        }

        diags.sort();
        self.children = children;
        self.root = root;
        self.placed = placed;
        self.by_path = by_path;
        self.diagnostics = diags;
    }

    /// The diagnostics of the load, kept on the value so every later read can surface them again.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// The campaign slug `open` was given.
    #[must_use]
    pub fn slug(&self) -> &Name {
        &self.slug
    }

    /// Whether the node is loaded and traces back to the root. A node that is loaded but
    /// unvalidated, or not loaded, is not.
    #[must_use]
    pub fn is_validated(&self, id: &NodeId) -> bool {
        self.placed.contains_key(id)
    }

    /// The validated root; `None` when no parentless node carries the slug.
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

    /// The parent of a node; `None` for a root or an orphan.
    #[must_use]
    pub fn parent(&self, id: &NodeId) -> Option<&NodeId> {
        let p = self.nodes.get(id)?.parent_id()?;
        self.nodes.get(p).map(|n| n.id())
    }

    /// The children of a node, sorted by `(order, id)`; empty for a leaf.
    #[must_use]
    pub fn children(&self, id: &NodeId) -> &[NodeId] {
        match self.nodes.get(id) {
            Some(Node::Directory(_)) => self.children.get(id).map_or(&[], Vec::as_slice),
            _ => &[],
        }
    }

    /// The number of directories between the root and the node (I1): a node directly under the
    /// root has depth 0, and so does the root. `None` for an unvalidated node.
    #[must_use]
    pub fn depth(&self, id: &NodeId) -> Option<u32> {
        self.placed.get(id).map(|p| p.0)
    }

    /// The names from the root's child down to the node, joined by `/`; `""` for the root.
    /// `None` for an unvalidated node.
    #[must_use]
    pub fn path(&self, id: &NodeId) -> Option<String> {
        self.placed.get(id).map(|p| p.1.clone())
    }

    /// The inverse of [`Campaign::path`], validated nodes only.
    #[must_use]
    pub fn resolve(&self, path: &str) -> Option<&NodeId> {
        self.by_path.get(path)
    }

    /// The node and every descendant, in pre-order, the node first. A node that is not loaded
    /// has none. Never visits a node twice, so a cycle ends.
    pub fn subtree(&self, id: &NodeId) -> impl Iterator<Item = &NodeId> {
        let mut out: Vec<&NodeId> = Vec::new();
        if let Some((key, _)) = self.nodes.get_key_value(id) {
            let mut seen = BTreeSet::new();
            let mut stack = vec![key];
            while let Some(n) = stack.pop() {
                if !seen.insert(n) {
                    continue;
                }
                out.push(n);
                stack.extend(self.children(n).iter().rev());
            }
        }
        out.into_iter()
    }

    /// Adds a node, or replaces the one with its id, and brings everything derived up to date.
    /// The result equals a fresh [`Campaign::open`] of the same nodes.
    pub fn insert(&mut self, node: Node) {
        self.duplicates.remove(node.id());
        self.nodes.insert(node.id().clone(), node);
        self.derive();
    }

    /// Removes a node and every descendant and brings everything derived up to date. Returns the
    /// removed ids, in pre-order.
    pub fn remove_subtree(&mut self, id: &NodeId) -> Vec<NodeId> {
        let removed: Vec<NodeId> = self.subtree(id).cloned().collect();
        for r in &removed {
            self.nodes.remove(r);
            self.duplicates.remove(r);
        }
        self.derive();
        removed
    }
}
