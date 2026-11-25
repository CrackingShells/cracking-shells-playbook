mod common;
use common::*;
use rdm_schema::ids::*;
use rdm_schema::stored::*;
use rdm_schema::*;
use serde_json::json;

#[test]
fn status_is_lowercase_and_has_five_values() {
    let all = [
        Status::Planned,
        Status::Ongoing,
        Status::Paused,
        Status::Cancelled,
        Status::Done,
    ];
    let names: Vec<_> = all
        .iter()
        .map(|s| serde_json::to_string(s).unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "\"planned\"",
            "\"ongoing\"",
            "\"paused\"",
            "\"cancelled\"",
            "\"done\""
        ]
    );
    assert!(serde_json::from_str::<Status>("\"blocked\"").is_err());
}

#[test]
fn version_constants_and_strings() {
    assert_eq!(FORMAT_VERSION, (0, 1, 0));
    assert_eq!(READ_MODEL_VERSION, (0, 1, 0));
    assert_eq!(version_string(FORMAT_VERSION), "0.1.0");
    assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
    assert_eq!(parse_version("0.1"), None);
    assert_eq!(parse_version("1"), None);
    assert_eq!(parse_version("a.b"), None);
}

#[test]
fn only_newer_versions_are_refused() {
    // Newer, whichever field is higher: a higher major, a higher X, a higher Y.
    assert!(is_newer((1, 0, 0), (0, 1, 0)));
    assert!(is_newer((2, 0, 0), (1, 5, 0)));
    assert!(is_newer((0, 2, 0), (0, 1, 0)));
    assert!(is_newer((0, 2, 0), (0, 1, 7)));
    assert!(is_newer((0, 1, 1), (0, 1, 0)));
    // The same and older versions are accepted, a lower major included.
    assert!(!is_newer((0, 1, 0), (0, 1, 0)));
    assert!(!is_newer((0, 1, 0), (1, 0, 0)));
    assert!(!is_newer((0, 0, 9), (0, 1, 0)));
    assert!(!is_newer((0, 1, 0), (0, 1, 3)));
}

#[test]
fn parse_version_is_strict() {
    for bad in [
        "01.1.0",
        "+1.2.3",
        " 1.2.3",
        "1.2.3 ",
        "1.02.3",
        "-1.0.0",
        "1",
        "1.2",
        "1.",
        ".1.2",
        "1.2.",
        "1.2.3.4",
        "banana",
        "",
        "1.x.0",
        "65536.0.0",
        "0.1.00",
    ] {
        assert_eq!(parse_version(bad), None, "{bad:?}");
    }
    assert_eq!(parse_version("0.0.0"), Some((0, 0, 0)));
    assert_eq!(parse_version("10.20.30"), Some((10, 20, 30)));
}

#[test]
fn marks_point_by_criterion_id_and_corrections_are_flagged() {
    let v: Leaf = serde_json::from_value(fixture("stored_leaf.json")).unwrap();
    let gate = &v.success_gates[0].id;
    assert!(v.marks.iter().all(|m| &m.criterion_id == gate));
    assert_eq!(v.marks[0].correction, None);
    assert_eq!(v.marks[1].correction, Some(true));
    assert!(
        v.marks[1]
            .reason
            .as_deref()
            .is_some_and(|r| !r.trim().is_empty())
    );
    let rev = v.history.last().unwrap();
    assert_eq!(
        (rev.from, rev.to, rev.correction),
        (Status::Done, Status::Ongoing, Some(true))
    );
}

#[test]
fn a_mark_with_a_position_is_refused() {
    let mut v = fixture("stored_leaf.json");
    v["marks"][0]
        .as_object_mut()
        .unwrap()
        .remove("criterion_id");
    v["marks"][0]["index"] = serde_json::json!(0);
    assert!(serde_json::from_value::<Leaf>(v).is_err());
}

#[test]
fn reordering_criteria_leaves_marks_valid() {
    let mut v: Leaf = serde_json::from_value(fixture("stored_leaf.json")).unwrap();
    let extra = Criterion {
        id: CriterionId::parse("c_4d7k1h").unwrap(),
        text: "another".into(),
        met: false,
        extra: Default::default(),
    };
    v.success_gates.insert(0, extra);
    v.preconditions.reverse();
    for m in &v.marks {
        assert!(
            v.success_gates
                .iter()
                .chain(&v.preconditions)
                .any(|c| c.id == m.criterion_id)
        );
    }
}

#[test]
fn ids_are_unique_and_changes_name_items_by_id() {
    // Criteria and steps are the only sub-items with ids: a check and a reference have none.
    let v: Leaf = serde_json::from_value(fixture("stored_leaf.json")).unwrap();
    let mut ids: Vec<&str> = v
        .preconditions
        .iter()
        .chain(&v.success_gates)
        .map(|c| c.id.as_str())
        .collect();
    ids.extend(v.steps.iter().map(|s| s.id.as_str()));
    let n = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), n);
    assert_eq!(n, 5);
    let a: Amendment = serde_json::from_value(fixture("stored_amendment.json")).unwrap();
    assert_eq!(a.correction, Some(true));
    assert_eq!(
        a.changes[3].item_id,
        Some(ItemId::Step(v.steps[0].id.clone()))
    );
    assert_eq!(a.changes[3].field.as_deref(), Some("/commit_sha"));
}

#[test]
fn malformed_ids_are_refused() {
    // One row per class for each prefixed id type: wrong prefix, a letter outside the alphabet
    // (`i`, `l`, `o`, `u`, or upper case), wrong length. `parse` and the deserialiser agree.
    fn check<T: serde::de::DeserializeOwned>(
        parse: fn(&str) -> Result<(), IdError>,
        good: &str,
        bad: &[(&str, IdError)],
    ) {
        assert_eq!(parse(good), Ok(()), "{good}");
        assert!(serde_json::from_value::<T>(json!(good)).is_ok(), "{good}");
        for (s, why) in bad {
            assert_eq!(parse(s), Err(*why), "{s:?}");
            assert!(serde_json::from_value::<T>(json!(s)).is_err(), "{s:?}");
        }
    }
    let node = IdError::WrongPrefix { expected: "n_" };
    check::<NodeId>(
        |s| NodeId::parse(s).map(drop),
        "n_7k3q9x2m",
        &[
            ("", IdError::Empty),
            ("x_7k3q9x2m", node),
            ("7k3q9x2m", node),
            ("c_8m2xqv", node),
            ("n_7k3q9x2i", IdError::BadChar),
            ("n_7k3q9x2l", IdError::BadChar),
            ("n_7k3q9x2o", IdError::BadChar),
            ("n_7k3q9x2u", IdError::BadChar),
            ("n_7K3Q9X2M", IdError::BadChar),
            ("n_7k3q9x2", IdError::BadLength),
            ("n_7k3q9x2mm", IdError::BadLength),
            ("n_", IdError::BadLength),
        ],
    );
    let crit = IdError::WrongPrefix { expected: "c_" };
    check::<CriterionId>(
        |s| CriterionId::parse(s).map(drop),
        "c_8m2xqv",
        &[
            ("", IdError::Empty),
            ("s_8m2xqv", crit),
            ("n_7k3q9x2m", crit),
            ("c_8m2xqu", IdError::BadChar),
            ("c_8M2XQV", IdError::BadChar),
            ("c_8m2xq", IdError::BadLength),
            ("c_8m2xqvv", IdError::BadLength),
        ],
    );
    let step = IdError::WrongPrefix { expected: "s_" };
    check::<StepId>(
        |s| StepId::parse(s).map(drop),
        "s_0f4tzn",
        &[
            ("", IdError::Empty),
            ("c_0f4tzn", step),
            ("s_0f4tzo", IdError::BadChar),
            ("s_0f4tz", IdError::BadLength),
            ("s_0f4tznn", IdError::BadLength),
        ],
    );
    // A ULID is 26 characters of upper-case Crockford base32.
    for bad in [
        "01JD000000000000000000001",
        "01JD00000000000000000000001",
        "01jd0000000000000000000001",
        "01JD000000000000000000000I",
    ] {
        assert!(
            CommentId::parse(bad).is_err() && AmendmentId::parse(bad).is_err(),
            "{bad}"
        );
    }
    assert!(CommentId::parse("01JD0000000000000000000001").is_ok());
    // An op id is one line of 1 to 128 characters.
    assert!(OpId::parse("op1").is_ok() && OpId::parse(&"x".repeat(128)).is_ok());
    for bad in ["", "two\nlines", "tab\there", &"x".repeat(129)] {
        assert!(OpId::parse(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn an_item_id_reads_as_a_criterion_or_a_step_by_its_prefix() {
    assert_eq!(
        ItemId::parse("c_8m2xqv"),
        Ok(ItemId::Criterion(CriterionId::parse("c_8m2xqv").unwrap()))
    );
    assert_eq!(
        ItemId::parse("s_0f4tzn"),
        Ok(ItemId::Step(StepId::parse("s_0f4tzn").unwrap()))
    );
    assert!(ItemId::parse("n_7k3q9x2m").is_err());
    assert!(ItemId::parse("c_8m2xqi").is_err());
    let id: ItemId = serde_json::from_value(json!("s_0f4tzn")).unwrap();
    assert_eq!(serde_json::to_value(&id).unwrap(), json!("s_0f4tzn"));
}

#[test]
fn names_follow_the_path_segment_pattern() {
    for good in ["core", "m1_core_foundation", "a", "x-1_y"] {
        assert_eq!(
            Name::parse(good).map(|n| n.to_string()),
            Ok(good.to_owned())
        );
    }
    assert_eq!(Name::parse(""), Err(NameError::Empty));
    for bad in ["1core", "_core", "-core", "Core"] {
        assert_eq!(Name::parse(bad), Err(NameError::BadStart), "{bad}");
    }
    for bad in ["co re", "co/re", "coRe", "cor.e", "corë"] {
        assert_eq!(Name::parse(bad), Err(NameError::BadChar), "{bad}");
    }
}

#[test]
fn a_snapshot_key_that_is_not_a_node_id_is_refused() {
    let mut v = fixture("stored_amendment.json");
    let snapshot = v["snapshot"].as_object_mut().unwrap();
    let node = snapshot.remove("n_9n3s7y5a").unwrap();
    snapshot.insert("core/m2".into(), node);
    assert!(serde_json::from_value::<Amendment>(v).is_err());
}
