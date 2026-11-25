mod common;
use common::*;
use rdm_schema::campaign::Campaign;
use rdm_schema::error::{Rule, StoreError};
use rdm_schema::ids::{Name, NodeId};
use rdm_schema::stored::Node;
use schemars::schema_for;

fn node(name: &str) -> Node {
    serde_json::from_value(fixture(name)).unwrap()
}

#[test]
fn unvalidated_nodes_stay_loaded_but_are_not_validated() {
    // A directory and a leaf under it, but no root: both load, none is validated, and the
    // missing root and the orphan are reported.
    let (dir, leaf) = (node("stored_directory.json"), node("stored_leaf.json"));
    let (dir_id, leaf_id) = (dir.id().clone(), leaf.id().clone());
    let slug = Name::parse("m1_core_foundation").unwrap();
    let opened = Campaign::open([dir, leaf], &slug).unwrap();
    let c = &opened.campaign;
    assert!(c.get(&leaf_id).is_some() && c.get(&dir_id).is_some());
    assert_eq!(c.nodes().count(), 2);
    assert!(!c.is_validated(&dir_id) && !c.is_validated(&leaf_id));
    assert!(!c.is_validated(&NodeId::parse("n_0m5r8c3h").unwrap()), "a node that is not loaded is not validated");
    assert_eq!(c.root(), None);
    let rules: Vec<Rule> = opened.diagnostics.iter().map(|d| d.rule).collect();
    assert!(rules.contains(&Rule::RootCount) && rules.contains(&Rule::ParentMissing), "{rules:?}");
}

#[test]
fn a_node_reads_as_the_type_its_kind_names() {
    assert!(matches!(node("stored_leaf.json"), Node::Leaf(_)));
    assert!(matches!(node("stored_directory.json"), Node::Directory(_)));
    let mut root = fixture("stored_directory.json");
    root["kind"] = "campaign".into();
    assert!(matches!(serde_json::from_value::<Node>(root).unwrap(), Node::Directory(_)));
    let mut other = fixture("stored_leaf.json");
    other["kind"] = "milestone".into();
    assert!(matches!(Node::decode(other.clone()), Err(StoreError::Invalid { rule: Rule::Kind, .. })));
    assert!(serde_json::from_value::<Node>(other).is_err());
    // A leaf body under a directory kind is not read as a leaf.
    let mut mixed = fixture("stored_leaf.json");
    mixed["kind"] = "directory".into();
    assert!(matches!(Node::decode(mixed.clone()), Err(StoreError::Invalid { rule: Rule::Parse, .. })));
    assert!(serde_json::from_value::<Node>(mixed).is_err());
}

#[test]
fn rule_codes_are_their_serde_names() {
    // The generated schema lists the serde name of every variant: the oracle for `code`.
    let schema = serde_json::to_value(schema_for!(Rule)).unwrap();
    let names: Vec<&str> = schema["oneOf"].as_array().expect("a oneOf schema").iter().map(|v| v["const"].as_str().unwrap()).collect();
    assert!(names.len() > 30);
    for name in names {
        let rule: Rule = serde_json::from_value(name.into()).unwrap();
        assert_eq!(rule.code(), name);
    }
}
