mod common;
use common::*;
use rdm_schema::stored::*;
use rdm_schema::*;

#[test]
fn leaf_fixture_round_trips() {
    round_trip::<Leaf>(&fixture("stored_leaf.json"));
}
#[test]
fn directory_fixture_round_trips() {
    round_trip::<Directory>(&fixture("stored_directory.json"));
}
#[test]
fn comment_fixture_round_trips() {
    round_trip::<Comment>(&fixture("stored_comment.json"));
}
#[test]
fn amendment_fixture_round_trips_and_keeps_null_new() {
    let v = fixture("stored_amendment.json");
    round_trip::<Amendment>(&v);
    let a: Amendment = serde_json::from_value(v).unwrap();
    assert_eq!(a.changes[0].new, Some(serde_json::Value::Null));
    assert_eq!(a.changes[1].new, None);
}

#[test]
fn unknown_fields_survive_at_every_level() {
    assert!(unknown_fields_survive::<Leaf>("stored_leaf.json") >= 12);
    assert!(unknown_fields_survive::<Directory>("stored_directory.json") >= 3);
    assert!(unknown_fields_survive::<Comment>("stored_comment.json") >= 2);
    assert!(unknown_fields_survive::<Amendment>("stored_amendment.json") >= 5);
}

#[test]
fn unknown_field_keeps_its_place_after_known_ones() {
    let mut v = fixture("stored_comment.json");
    v["zzz"] = serde_json::json!(1);
    let c: Comment = serde_json::from_value(v).unwrap();
    let s = serde_json::to_string(&c).unwrap();
    assert!(s.find("\"op_id\"").unwrap() < s.find("\"zzz\"").unwrap());
}

#[test]
fn status_is_lowercase_and_has_five_values() {
    let all = [Status::Planned, Status::Ongoing, Status::Paused, Status::Cancelled, Status::Done];
    let names: Vec<_> = all.iter().map(|s| serde_json::to_string(s).unwrap()).collect();
    assert_eq!(names, ["\"planned\"", "\"ongoing\"", "\"paused\"", "\"cancelled\"", "\"done\""]);
    assert!(serde_json::from_str::<Status>("\"blocked\"").is_err());
}

#[test]
fn absent_optionals_stay_absent() {
    let c: Commit = serde_json::from_str(r#"{"type":"docs","breaking":false,"summary":"x"}"#).unwrap();
    assert_eq!(c.scope, None);
    assert_eq!(serde_json::to_string(&c).unwrap(), r#"{"type":"docs","breaking":false,"summary":"x"}"#);
}

#[test]
fn version_constants_and_strings() {
    assert_eq!(FORMAT_VERSION, (0, 1));
    assert_eq!(READ_MODEL_VERSION, (0, 1));
    assert_eq!(version_string(FORMAT_VERSION), "0.1");
    assert_eq!(parse_version("0.1"), Some((0, 1)));
    assert_eq!(parse_version("1"), None);
    assert_eq!(parse_version("a.b"), None);
    assert!(is_readable(parse_version(&version_string(FORMAT_VERSION)).unwrap(), FORMAT_VERSION));
}

#[test]
fn reader_refuses_per_the_read_model_rule() {
    // Higher major: refused.
    assert!(!is_readable((1, 0), (0, 1)));
    assert!(!is_readable((2, 0), (1, 5)));
    // Major 0: a higher minor is refused too.
    assert!(!is_readable((0, 2), (0, 1)));
    // Same or lower: read.
    assert!(is_readable((0, 1), (0, 1)));
    assert!(is_readable((0, 0), (0, 1)));
    // From major 1: a higher minor is read.
    assert!(is_readable((1, 3), (1, 1)));
    assert!(!is_readable((2, 0), (1, 1)));
}

#[test]
fn parse_version_is_strict() {
    for bad in ["01.1", "+1.2", " 1.2", "1.2 ", "1.02", "-1.0", "1", "1.", ".1", "1.2.3", "banana", "", "1.x", "65536.0"] {
        assert_eq!(parse_version(bad), None, "{bad:?}");
    }
    assert_eq!(parse_version("0.0"), Some((0, 0)));
    assert_eq!(parse_version("10.20"), Some((10, 20)));
}

#[test]
fn unknown_keys_keep_their_order_in_the_output_string() {
    // Non-alphabetical unknown keys at two nesting levels, and one known key after them.
    let input = r#"{"schema_version":"0.1","id":"c","node_id":"n","ts":"2026-11-16T12:00:00Z","actor":{"kind":"human","zeta":1,"alpha":2},"text":["a"],"zz":{"y":1,"b":2,"m":3},"aa":true}"#;
    let c: Comment = serde_json::from_str(input).unwrap();
    assert_eq!(serde_json::to_string(&c).unwrap(), input);
}
