//! Revisions (`store.md`, §Revisions): SHA-256 over canonical text, the first 16 hexadecimal
//! characters, opaque to every consumer.

use std::collections::BTreeMap;

use rdm_schema::ids::{NodeId, Rev, SubtreeRev, TreeRev};
use rdm_schema::stored::Node;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn hex16(hash: impl AsRef<[u8]>) -> String {
    hash.as_ref()
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn lines<'a>(domain: &str, entries: impl Iterator<Item = (&'a NodeId, &'a Rev)>) -> String {
    let mut text = format!("{domain}\n");
    for (id, rev) in entries {
        text.push_str(&format!("{id} {rev}\n"));
    }
    hex16(Sha256::digest(text.as_bytes()))
}

/// The `rev` of a node: the hash of the canonical encoding of the node as decoded, with
/// `schema_version` left out. Formatting of the file, and the stamp, do not count.
#[must_use]
pub fn rev(node: &Node) -> Rev {
    let mut value = serde_json::to_value(node).unwrap_or(Value::Null);
    if let Value::Object(map) = &mut value {
        // `remove` would swap the last key into the first slot under `preserve_order`.
        map.shift_remove("schema_version");
    }
    // Cannot fail: a `Value` has string keys only.
    let mut text = serde_json::to_string_pretty(&value).unwrap_or_default();
    text.push('\n');
    Rev::new(hex16(Sha256::digest(text.as_bytes())))
}

/// The `tree_rev`: the hash of `rdm-tree\n` and one line `<id> <rev>\n` per node in ascending
/// bytewise order of id.
#[must_use]
pub fn tree_rev(revs: &BTreeMap<NodeId, Rev>) -> TreeRev {
    TreeRev::new(lines("rdm-tree", revs.iter()))
}

/// The `SubtreeRev` of the nodes `ids` (a node and its descendants): the same form with the
/// domain `rdm-subtree`, over the lines of those nodes sorted by id.
#[must_use]
pub fn subtree_rev<'a>(
    ids: impl Iterator<Item = &'a NodeId>,
    revs: &BTreeMap<NodeId, Rev>,
) -> SubtreeRev {
    let mut picked: Vec<(&NodeId, &Rev)> = ids.filter_map(|id| revs.get_key_value(id)).collect();
    picked.sort();
    SubtreeRev::new(lines("rdm-subtree", picked.into_iter()))
}
