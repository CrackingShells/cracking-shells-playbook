mod common;
use common::*;
use rdm_schema::read::*;
use serde_json::json;

#[test]
fn outline_example_round_trips() {
    round_trip::<OutlineView>(&fixture("outline.json"));
}
#[test]
fn node_example_round_trips() {
    round_trip::<NodeView>(&fixture("node.json"));
}
#[test]
fn ready_examples_round_trip() {
    round_trip::<ReadyView>(&fixture("ready.json"));
    round_trip::<ReadyView>(&fixture("ready_scoped.json"));
}
#[test]
fn amendments_example_round_trips() {
    round_trip::<AmendmentsView>(&fixture("amendments.json"));
}

#[test]
fn directory_detail_round_trips() {
    let v = json!({
      "read_model_version": "0.1.0", "format_version": "0.1.0", "projection": "node",
      "campaign": {"id": "c", "path": "p", "title": "T"}, "tree_rev": "t",
      "node": { "kind": "directory", "id": "d", "path": "core", "parent": "", "rev": "r", "title": "Core",
        "sealed": false, "comments": [], "amendments": [], "is_campaign": false, "status": "planned",
        "counts": {"planned":1,"ongoing":0,"paused":0,"cancelled":0,"done":0},
        "context": ["c"], "goal": "g", "preconditions": [], "success_gates": [], "references": [],
        "gotchas": [], "notes": [], "marks": [],
        "children": [{"id":"n","path":"core/m1","kind":"leaf","title":"M1","status":"planned"}] },
      "holders": {}
    });
    round_trip::<NodeView>(&v);
    let mut v2 = v;
    assert!(inject_unknown(&mut v2, "$") >= 5);
    round_trip::<NodeView>(&v2);
}

#[test]
fn unknown_fields_survive_in_every_projection() {
    assert!(unknown_fields_survive::<OutlineView>("outline.json") >= 20);
    assert!(unknown_fields_survive::<NodeView>("node.json") >= 10);
    assert!(unknown_fields_survive::<ReadyView>("ready.json") >= 10);
    assert!(unknown_fields_survive::<ReadyView>("ready_scoped.json") >= 4);
    assert!(unknown_fields_survive::<AmendmentsView>("amendments.json") >= 5);
}

#[test]
fn level_source_has_only_depth() {
    assert_eq!(serde_json::to_string(&LevelSource::Depth).unwrap(), "\"depth\"");
    assert!(serde_json::from_str::<LevelSource>("\"other\"").is_err());
}

#[test]
fn wrong_kind_is_refused() {
    let mut v = fixture("outline.json");
    v["root"]["kind"] = json!("leaf");
    assert!(serde_json::from_value::<OutlineView>(v).is_err());
}
