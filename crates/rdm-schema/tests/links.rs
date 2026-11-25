use std::collections::BTreeSet;

use rdm_schema::links::{LINKS, Owner, Target};
use rdm_schema::stored::{Amendment, Comment, Directory, Leaf};
use schemars::{JsonSchema, schema_for};
use serde_json::Value;

type Found = BTreeSet<(Owner, String, Target)>;

fn target_of(def: &str) -> Option<Target> {
    match def {
        "NodeId" => Some(Target::Node),
        "CriterionId" => Some(Target::Criterion),
        "StepId" => Some(Target::Step),
        "ItemId" => Some(Target::Item),
        _ => None,
    }
}

/// Walks a JSON Schema and records, at its JSON pointer, every place that references an id
/// newtype, apart from the `id` property (a record's identity, not a link). `*` stands for an
/// array index and `{key}` for a map key.
struct Walk<'a> {
    root: &'a Value,
    owner: Owner,
    found: &'a mut Found,
}

impl Walk<'_> {
    fn schema(&mut self, s: &Value, at: &str) {
        if let Some(r) = s.get("$ref").and_then(Value::as_str) {
            let name = r.strip_prefix("#/$defs/").unwrap_or_else(|| panic!("unexpected $ref {r}"));
            match target_of(name) {
                Some(t) => {
                    self.found.insert((self.owner, at.to_owned(), t));
                }
                None => {
                    let def = self.root["$defs"][name].clone();
                    self.schema(&def, at);
                }
            }
        }
        for key in ["anyOf", "oneOf", "allOf"] {
            for v in s.get(key).and_then(Value::as_array).into_iter().flatten() {
                self.schema(v, at);
            }
        }
        for (name, sub) in s.get("properties").and_then(Value::as_object).into_iter().flatten() {
            if name != "id" {
                self.schema(sub, &format!("{at}/{name}"));
            }
        }
        if let Some(items) = s.get("items") {
            self.schema(items, &format!("{at}/*"));
        }
        if let Some(keys) = s.get("propertyNames") {
            self.schema(keys, &format!("{at}/{{key}}"));
        }
    }
}

fn walk<T: JsonSchema>(owner: Owner, found: &mut Found) {
    let root = serde_json::to_value(schema_for!(T)).unwrap();
    Walk { root: &root, owner, found }.schema(&root, "");
}

#[test]
fn links_cover_every_id_field() {
    let mut in_schema = Found::new();
    walk::<Directory>(Owner::Directory, &mut in_schema);
    walk::<Leaf>(Owner::Leaf, &mut in_schema);
    walk::<Comment>(Owner::Comment, &mut in_schema);
    walk::<Amendment>(Owner::Amendment, &mut in_schema);
    let declared: Found = LINKS.iter().map(|l| (l.owner, l.pointer.to_owned(), l.target)).collect();
    assert_eq!(declared.len(), LINKS.len(), "a link is declared twice");
    let undeclared: Vec<_> = in_schema.difference(&declared).collect();
    let dangling: Vec<_> = declared.difference(&in_schema).collect();
    assert!(undeclared.is_empty(), "id fields of the schemas that LINKS does not declare: {undeclared:?}");
    assert!(dangling.is_empty(), "links of LINKS that no id field of the schemas has: {dangling:?}");
}
