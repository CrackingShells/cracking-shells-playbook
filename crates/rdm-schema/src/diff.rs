//! The amendment rule's pure function (`contracts.md`, `The amendment rule`): the changes
//! between the old and the new value of one node.
//!
//! Criteria and steps are reported by id (`add`, `remove`, `move`, and `set` with `item_id`),
//! every other array as one `set` of the whole array, every other field as a `set` at its
//! pointer. `/schema_version`, `/history`, `/marks`, `/status` and `/status_reason` are left out:
//! they are state moves, recorded by the history and mark entries themselves.
//!
//! For `add`, `remove` and `move` of a sub-item, `field` is the pointer of the array that holds
//! it (`/preconditions`, `/success_gates`, `/steps`), because a criterion id is unique in the
//! file and says nothing of which of the two arrays it is in. `add` holds the item in `new`,
//! `remove` in `old`, and `move` holds the array positions before and after.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::ids::{ItemId, NodeId};
use crate::stored::{Change, ChangeOp, Node};

const LEFT_OUT: [&str; 5] = [
    "schema_version",
    "history",
    "marks",
    "status",
    "status_reason",
];

fn object(node: &Node) -> Map<String, Value> {
    match serde_json::to_value(node) {
        Ok(Value::Object(m)) => m,
        _ => Map::new(),
    }
}

fn set(
    node_id: &NodeId,
    item_id: Option<&ItemId>,
    field: &str,
    old: Option<&Value>,
    new: Option<&Value>,
) -> Change {
    Change {
        node_id: node_id.clone(),
        op: ChangeOp::Set,
        item_id: item_id.cloned(),
        field: Some(format!("/{field}")),
        old: old.cloned(),
        new: new.cloned(),
        extra: Map::new(),
    }
}

/// The keys of both objects, the first one's order first.
fn keys<'a>(a: &'a Map<String, Value>, b: &'a Map<String, Value>) -> Vec<&'a String> {
    let mut keys: Vec<&String> = a.keys().collect();
    keys.extend(b.keys().filter(|k| !a.contains_key(*k)));
    keys
}

fn item_id(v: &Value) -> Option<ItemId> {
    v.get("id")
        .and_then(Value::as_str)
        .and_then(|s| ItemId::parse(s).ok())
}

fn items(out: &mut Vec<Change>, node_id: &NodeId, array: &str, old: &[Value], new: &[Value]) {
    let pointer = format!("/{array}");
    let position = |list: &[Value]| -> BTreeMap<String, usize> {
        list.iter()
            .enumerate()
            .filter_map(|(i, v)| item_id(v).map(|id| (id.to_string(), i)))
            .collect()
    };
    let (old_at, new_at) = (position(old), position(new));
    let sub = |op, id: ItemId, old: Option<&Value>, new: Option<&Value>| Change {
        node_id: node_id.clone(),
        op,
        item_id: Some(id),
        field: Some(pointer.clone()),
        old: old.cloned(),
        new: new.cloned(),
        extra: Map::new(),
    };
    for v in old {
        if let Some(id) = item_id(v).filter(|id| !new_at.contains_key(id.as_str())) {
            out.push(sub(ChangeOp::Remove, id, Some(v), None));
        }
    }
    for v in new {
        if let Some(id) = item_id(v).filter(|id| !old_at.contains_key(id.as_str())) {
            out.push(sub(ChangeOp::Add, id, None, Some(v)));
        }
    }
    // A reorder: take the surviving ids in the old order and bring them, one by one, to the new.
    let mut order: Vec<&String> = old_at.keys().filter(|k| new_at.contains_key(*k)).collect();
    order.sort_by_key(|k| old_at[*k]);
    let target: Vec<&String> = {
        let mut t: Vec<&String> = new_at.keys().filter(|k| old_at.contains_key(*k)).collect();
        t.sort_by_key(|k| new_at[*k]);
        t
    };
    for (i, want) in target.iter().enumerate() {
        if order[i] != *want {
            let from = order.iter().position(|o| o == want).unwrap_or(i);
            let moved = order.remove(from);
            order.insert(i, moved);
            if let Ok(id) = ItemId::parse(want) {
                let (a, b) = (Value::from(old_at[*want]), Value::from(new_at[*want]));
                out.push(sub(ChangeOp::Move, id, Some(&a), Some(&b)));
            }
        }
    }
    // The fields of the items that stayed.
    for v in new {
        let Some(id) = item_id(v) else { continue };
        let Some(&at) = old_at.get(id.as_str()) else {
            continue;
        };
        let (Some(o), Some(n)) = (old[at].as_object(), v.as_object()) else {
            continue;
        };
        for k in keys(o, n) {
            if k != "id" && o.get(k) != n.get(k) {
                out.push(set(node_id, Some(&id), k, o.get(k), n.get(k)));
            }
        }
    }
}

/// The changes between `old` and `new`, computed by this one function and not by the caller.
#[must_use]
pub fn diff(old: &Node, new: &Node) -> Vec<Change> {
    let (o, n) = (object(old), object(new));
    let node_id = new.id();
    let mut out = Vec::new();
    for k in keys(&o, &n) {
        if LEFT_OUT.contains(&k.as_str()) {
            continue;
        }
        let (ov, nv) = (o.get(k), n.get(k));
        if ov == nv {
            continue;
        }
        match (k.as_str(), ov, nv) {
            (
                "preconditions" | "success_gates" | "steps",
                Some(Value::Array(a)),
                Some(Value::Array(b)),
            ) => {
                items(&mut out, node_id, k, a, b);
            }
            _ => out.push(set(node_id, None, k, ov, nv)),
        }
    }
    out
}

/// Whether a change is **state** and not content: a pointer under `/progress`, a criterion's
/// `/met`, or a step's `/done`, `/commit_sha` or `/done_at` (`statuses.md`, `Finished nodes`).
#[must_use]
pub fn is_state(change: &Change) -> bool {
    if change.op != ChangeOp::Set {
        return false;
    }
    let field = change.field.as_deref().unwrap_or("");
    match &change.item_id {
        None => field == "/progress" || field.starts_with("/progress/"),
        Some(ItemId::Criterion(_)) => field == "/met",
        Some(ItemId::Step(_)) => matches!(field, "/done" | "/commit_sha" | "/done_at"),
    }
}
