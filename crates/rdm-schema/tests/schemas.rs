use rdm_schema::ops::{
    BatchOp, CommentOp, Commented, CreateOp, DeleteOp, Deleted, OpResult, Receipt, RelocateOp, ReplaceOp, Request, Written,
};
use rdm_schema::read::{AmendmentsView, NodeView, OutlineView, ReadyView};
use rdm_schema::stored::{Amendment, Comment, Directory, Leaf};
use schemars::{JsonSchema, schema_for};

/// One pretty JSON Schema with a trailing newline.
fn generated<T: JsonSchema>() -> String {
    let mut s = serde_json::to_string_pretty(&schema_for!(T)).expect("schema serialises");
    s.push('\n');
    s
}

const NAMES: [&str; 18] = [
    "directory", "leaf", "comment", "amendment", "outline_view", "node_view", "ready_view", "amendments_view",
    "create_request", "replace_request", "relocate_request", "delete_request", "comment_request", "batch_request",
    "written_receipt", "deleted_receipt", "commented_receipt", "batch_receipt",
];

/// A file in `schemas/` that no type generates is drift too. Bless removes it.
fn orphans(failures: &mut Vec<String>) {
    let dir = format!("{}/schemas", env!("CARGO_MANIFEST_DIR"));
    let bless = std::env::var("RDM_SCHEMA_BLESS").as_deref() == Ok("1");
    for entry in std::fs::read_dir(&dir).expect("schemas dir").flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        let known = file.strip_suffix(".schema.json").is_some_and(|n| NAMES.contains(&n));
        if known {
            continue;
        }
        if bless {
            std::fs::remove_file(entry.path()).expect("remove orphan");
        } else {
            failures.push(format!("{dir}/{file}: no type generates this file"));
        }
    }
}

/// Compares `want` with the committed file, or writes it when `RDM_SCHEMA_BLESS=1`.
fn check_or_bless(name: &str, want: &str, failures: &mut Vec<String>) {
    let path = format!("{}/schemas/{name}.schema.json", env!("CARGO_MANIFEST_DIR"));
    if std::env::var("RDM_SCHEMA_BLESS").as_deref() == Ok("1") {
        std::fs::write(&path, want).unwrap_or_else(|e| panic!("{path}: {e}"));
        return;
    }
    match std::fs::read_to_string(&path) {
        Err(e) => failures.push(format!("{path}: cannot read ({e}); run RDM_SCHEMA_BLESS=1 cargo test -p rdm-schema")),
        Ok(have) if have == want => {}
        Ok(have) => {
            let line = want
                .lines()
                .zip(have.lines())
                .position(|(a, b)| a != b)
                .unwrap_or_else(|| want.lines().count().min(have.lines().count()));
            failures.push(format!(
                "{path}: differs from the generated schema, first at line {}\n  generated: {:?}\n  committed: {:?}",
                line + 1,
                want.lines().nth(line),
                have.lines().nth(line)
            ));
        }
    }
}

#[test]
fn committed_schemas_match_the_types() {
    let mut failures = Vec::new();
    check_or_bless("directory", &generated::<Directory>(), &mut failures);
    check_or_bless("leaf", &generated::<Leaf>(), &mut failures);
    check_or_bless("comment", &generated::<Comment>(), &mut failures);
    check_or_bless("amendment", &generated::<Amendment>(), &mut failures);
    check_or_bless("outline_view", &generated::<OutlineView>(), &mut failures);
    check_or_bless("node_view", &generated::<NodeView>(), &mut failures);
    check_or_bless("ready_view", &generated::<ReadyView>(), &mut failures);
    check_or_bless("amendments_view", &generated::<AmendmentsView>(), &mut failures);
    check_or_bless("create_request", &generated::<Request<CreateOp>>(), &mut failures);
    check_or_bless("replace_request", &generated::<Request<ReplaceOp>>(), &mut failures);
    check_or_bless("relocate_request", &generated::<Request<RelocateOp>>(), &mut failures);
    check_or_bless("delete_request", &generated::<Request<DeleteOp>>(), &mut failures);
    check_or_bless("comment_request", &generated::<Request<CommentOp>>(), &mut failures);
    check_or_bless("batch_request", &generated::<Request<BatchOp>>(), &mut failures);
    check_or_bless("written_receipt", &generated::<Receipt<Written>>(), &mut failures);
    check_or_bless("deleted_receipt", &generated::<Receipt<Deleted>>(), &mut failures);
    check_or_bless("commented_receipt", &generated::<Receipt<Commented>>(), &mut failures);
    check_or_bless("batch_receipt", &generated::<Receipt<Vec<OpResult>>>(), &mut failures);
    orphans(&mut failures);
    assert!(failures.is_empty(), "schema drift:\n{}", failures.join("\n"));
}

#[test]
fn version_fields_carry_the_pattern() {
    for (file, pointer) in [
        ("leaf", "/properties/schema_version"),
        ("directory", "/properties/schema_version"),
        ("comment", "/properties/schema_version"),
        ("amendment", "/properties/schema_version"),
        ("outline_view", "/properties/format_version"),
        ("outline_view", "/properties/read_model_version"),
    ] {
        let path = format!("{}/schemas/{file}.schema.json", env!("CARGO_MANIFEST_DIR"));
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let pat = v.pointer(&format!("{pointer}/pattern")).and_then(|p| p.as_str());
        assert_eq!(pat, Some(r"^[0-9]+\.[0-9]+\.[0-9]+$"), "{file} {pointer}");
    }
}
