use rdm_schema::read::{AmendmentsView, NodeView, OutlineView, ReadyView};
use rdm_schema::stored::{Amendment, Comment, Directory, Leaf};
use schemars::{JsonSchema, schema_for};

/// One pretty JSON Schema with a trailing newline.
fn generated<T: JsonSchema>() -> String {
    let mut s = serde_json::to_string_pretty(&schema_for!(T)).expect("schema serialises");
    s.push('\n');
    s
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
    assert!(failures.is_empty(), "schema drift:\n{}", failures.join("\n"));
}
