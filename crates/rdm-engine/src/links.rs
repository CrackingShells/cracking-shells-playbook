//! The reverse link index (`contracts.md`, `Reverse index`): who points at a node, derived from
//! the declared [`LINKS`], never stored, kept in step by write-through.

use std::collections::BTreeMap;

use rdm_schema::campaign::Campaign;
use rdm_schema::ids::NodeId;
use rdm_schema::links::{LINKS, Owner, RecordRef, Referrer, Target};
use rdm_schema::stored::{Amendment, Comment, Node};
use serde_json::Value;

/// A record borrowed for the index: a node, a comment or an amendment.
#[derive(Debug, Clone, Copy)]
pub enum Record<'a> {
    Node(&'a Node),
    Comment(&'a Comment),
    Amendment(&'a Amendment),
}

impl Record<'_> {
    fn owner(&self) -> Owner {
        match self {
            Record::Node(Node::Directory(_)) => Owner::Directory,
            Record::Node(Node::Leaf(_)) => Owner::Leaf,
            Record::Comment(_) => Owner::Comment,
            Record::Amendment(_) => Owner::Amendment,
        }
    }

    /// The reference to this record.
    #[must_use]
    pub fn record_ref(&self) -> RecordRef {
        match self {
            Record::Node(n) => RecordRef::Node(n.id().clone()),
            Record::Comment(c) => RecordRef::Comment(c.id.clone()),
            Record::Amendment(a) => RecordRef::Amendment(a.id.clone()),
        }
    }

    fn value(&self) -> Value {
        match self {
            Record::Node(n) => serde_json::to_value(n),
            Record::Comment(c) => serde_json::to_value(c),
            Record::Amendment(a) => serde_json::to_value(a),
        }
        .unwrap_or(Value::Null)
    }
}

fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// Follows the segments of a `LINKS` pointer (`*` any array index, `{key}` any map key) through
/// `value` and collects the concrete pointer and the node id each one reaches.
fn follow(value: &Value, segments: &[&str], at: &str, out: &mut Vec<(String, NodeId)>) {
    let Some((first, rest)) = segments.split_first() else {
        if let Some(id) = value.as_str().and_then(|s| NodeId::parse(s).ok()) {
            out.push((at.to_owned(), id));
        }
        return;
    };
    match (*first, value) {
        ("*", Value::Array(items)) => {
            for (i, item) in items.iter().enumerate() {
                follow(item, rest, &format!("{at}/{i}"), out);
            }
        }
        ("{key}", Value::Object(map)) => {
            // The keys are the links; the values are historical copies.
            for key in map.keys() {
                if let Ok(id) = NodeId::parse(key) {
                    out.push((format!("{at}/{}", escape(key)), id));
                }
            }
        }
        (name, Value::Object(map)) => {
            if let Some(child) = map.get(name) {
                follow(child, rest, &format!("{at}/{name}"), out);
            }
        }
        _ => {}
    }
}

/// Every link of a record that points at a node: the `LINKS` row, the concrete pointer, the
/// target.
fn links_of(record: &Record) -> Vec<(usize, String, NodeId)> {
    let value = record.value();
    let owner = record.owner();
    let mut found = Vec::new();
    for (row, link) in LINKS.iter().enumerate() {
        if link.owner != owner || link.target != Target::Node {
            continue;
        }
        let segments: Vec<&str> = link.pointer.split('/').skip(1).collect();
        let mut hits = Vec::new();
        follow(&value, &segments, "", &mut hits);
        found.extend(hits.into_iter().map(|(at, id)| (row, at, id)));
    }
    found
}

/// Who points at each node, from every class of link.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReverseIndex {
    by_target: BTreeMap<NodeId, Vec<Referrer>>,
    /// The targets each record holds, so a replaced record drops its old entries by itself.
    by_record: BTreeMap<RecordRef, Vec<NodeId>>,
}

impl ReverseIndex {
    /// Walks each record once and follows every `LINKS` row whose owner matches:
    /// O(records x links).
    #[must_use]
    pub fn build(campaign: &Campaign, comments: &[Comment], amendments: &[Amendment]) -> Self {
        let mut index = Self::default();
        for n in campaign.nodes() {
            index.replace_record(RecordRef::Node(n.id().clone()), Some(&Record::Node(n)));
        }
        for c in comments {
            index.replace_record(RecordRef::Comment(c.id.clone()), Some(&Record::Comment(c)));
        }
        for a in amendments {
            index.replace_record(
                RecordRef::Amendment(a.id.clone()),
                Some(&Record::Amendment(a)),
            );
        }
        index
    }

    /// Every record that points at `target`, ordered by `(from, at)`.
    #[must_use]
    pub fn referrers(&self, target: &NodeId) -> &[Referrer] {
        self.by_target.get(target).map_or(&[], Vec::as_slice)
    }

    /// Drops the old entries of one record and adds its new ones; `None`: the record is gone.
    /// The work is in proportion to the record, not to the campaign.
    pub fn replace_record(&mut self, from: RecordRef, record: Option<&Record>) {
        if let Some(old) = self.by_record.remove(&from) {
            for target in old {
                if let Some(list) = self.by_target.get_mut(&target) {
                    list.retain(|r| r.from != from);
                    if list.is_empty() {
                        self.by_target.remove(&target);
                    }
                }
            }
        }
        let Some(record) = record else { return };
        let mut targets = Vec::new();
        for (link, at, target) in links_of(record) {
            let list = self.by_target.entry(target.clone()).or_default();
            list.push(Referrer {
                link,
                from: from.clone(),
                at,
            });
            list.sort_by(|a, b| (&a.from, &a.at).cmp(&(&b.from, &b.at)));
            targets.push(target);
        }
        targets.sort();
        targets.dedup();
        if !targets.is_empty() {
            self.by_record.insert(from, targets);
        }
    }

    /// Every target that has a referrer, with its referrers: the whole index, for comparison.
    pub fn entries(&self) -> impl Iterator<Item = (&NodeId, &[Referrer])> {
        self.by_target.iter().map(|(k, v)| (k, v.as_slice()))
    }
}
