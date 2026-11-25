//! Guards of the codec (`contracts.md`, `Version chain` and the canonical form): one value has one
//! text, any valid text reads losslessly, unknown keys survive in place and are reported once each.
//!
//! The canonical fixtures are written by hand from `contracts.md` and `stored-schema.md` (field
//! order, two-space indent, LF, one trailing newline, prose as line arrays, `null` only as a
//! `Change` value) and reviewed line by line. Nothing here asks the encoder what canonical is.

use std::collections::BTreeSet;
use std::fmt::Debug;
use std::path::PathBuf;

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use rdm_engine::codec::{Stored, decode, encode};
use rdm_schema::Version;
use rdm_schema::campaign::Diagnostic;
use rdm_schema::error::{Rule, StoreError};
use rdm_schema::ids::*;
use rdm_schema::stored::*;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
enum Kind {
    Node,
    Comment,
    Amendment,
}

fn dir(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(sub)
}

fn read(sub: &str, name: &str) -> String {
    let path = dir(sub).join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Fails with the first byte at which the two texts differ, shown with a little context.
#[track_caller]
fn same_bytes(what: &str, got: &str, want: &str) {
    if got == want {
        return;
    }
    let at = got
        .bytes()
        .zip(want.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| got.len().min(want.len()));
    let around = |t: &str| {
        let from = (0..=at.saturating_sub(30))
            .rev()
            .find(|i| t.is_char_boundary(*i))
            .unwrap_or(0);
        let to = (at + 30..=t.len())
            .find(|i| t.is_char_boundary(*i))
            .unwrap_or(t.len());
        t[from..to.min(t.len())].to_owned()
    };
    panic!(
        "{what}: first difference at byte {at} (got {} bytes, want {})\n  got:  {:?}\n  want: {:?}",
        got.len(),
        want.len(),
        around(got),
        around(want)
    );
}

/// Decodes `bytes` as a record of `kind` and encodes the value back.
fn reencode(kind: Kind, file: &str, bytes: &[u8]) -> Result<(String, Vec<Diagnostic>), StoreError> {
    match kind {
        Kind::Node => decode::<Node>(file, bytes).map(|d| (encode(&d.value), d.diagnostics)),
        Kind::Comment => decode::<Comment>(file, bytes).map(|d| (encode(&d.value), d.diagnostics)),
        Kind::Amendment => {
            decode::<Amendment>(file, bytes).map(|d| (encode(&d.value), d.diagnostics))
        }
    }
}

/// Every hand-written canonical fixture and the record it holds.
const CANONICAL: &[(&str, Kind)] = &[
    ("leaf.json", Kind::Node),
    ("leaf_paused.json", Kind::Node),
    ("directory.json", Kind::Node),
    ("campaign.json", Kind::Node),
    ("comment.json", Kind::Comment),
    ("comment_escapes.json", Kind::Comment),
    ("amendment.json", Kind::Amendment),
];

#[test]
fn canonical_files_round_trip_byte_for_byte() {
    let mut listed: Vec<String> = std::fs::read_dir(dir("canonical"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    listed.sort();
    let mut named: Vec<String> = CANONICAL.iter().map(|(n, _)| (*n).to_owned()).collect();
    named.sort();
    assert_eq!(
        listed, named,
        "a canonical fixture is on disk and not in the table, or the reverse"
    );

    for (name, kind) in CANONICAL {
        let text = read("canonical", name);
        let (again, diagnostics) =
            reencode(*kind, name, text.as_bytes()).unwrap_or_else(|e| panic!("{name}: {e}"));
        same_bytes(
            &format!("{name} does not re-encode to its own bytes"),
            &again,
            &text,
        );
        assert_eq!(
            diagnostics,
            vec![],
            "{name} is canonical and holds no unknown key"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Valid input that is not canonical

/// Each row is a file a person, an editor or git writes, made by the command in
/// `fixtures/noncanonical/produce.sh`, and the canonical text it must re-encode to.
struct Row {
    what: &'static str,
    file: &'static str,
    expected_dir: &'static str,
    expected_file: &'static str,
    kind: Kind,
    /// The JSON pointers of the unknown keys the file holds, each reported once.
    unknown: &'static [&'static str],
}

const fn row(
    what: &'static str,
    file: &'static str,
    expected_dir: &'static str,
    expected_file: &'static str,
    kind: Kind,
) -> Row {
    Row {
        what,
        file,
        expected_dir,
        expected_file,
        kind,
        unknown: &[],
    }
}

const NONCANONICAL: &[Row] = &[
    row(
        "reordered keys (`jq -S`)",
        "reordered_keys_leaf.json",
        "canonical",
        "leaf.json",
        Kind::Node,
    ),
    row(
        "four-space indent (`jq --indent 4`)",
        "four_space_indent_directory.json",
        "canonical",
        "directory.json",
        Kind::Node,
    ),
    row(
        "CRLF (`unix2dos`)",
        "crlf_comment.json",
        "canonical",
        "comment.json",
        Kind::Comment,
    ),
    row(
        "no trailing newline",
        "no_trailing_newline_leaf_paused.json",
        "canonical",
        "leaf_paused.json",
        Kind::Node,
    ),
    row(
        "`\\u` escapes (`jq -a`)",
        "unicode_escapes_comment.json",
        "canonical",
        "comment_escapes.json",
        Kind::Comment,
    ),
    row(
        "explicit nulls on absent optionals",
        "explicit_nulls_leaf.json",
        "canonical",
        "leaf.json",
        Kind::Node,
    ),
    row(
        "`git merge-file` on two edits",
        "merge_leaf.json",
        "noncanonical",
        "merge_leaf.expected.json",
        Kind::Node,
    ),
    row(
        "UTF-8 byte order mark (PowerShell 5, old Notepad)",
        "bom_leaf.json",
        "canonical",
        "leaf.json",
        Kind::Node,
    ),
    Row {
        what: "`jq -S` over a leaf with unknown keys at two levels",
        file: "sorted_unknown_keys_leaf.json",
        expected_dir: "noncanonical",
        expected_file: "sorted_unknown_keys_leaf.expected.json",
        kind: Kind::Node,
        unknown: &["/x_alpha", "/x_zeta", "/steps/0/x_note"],
    },
];

#[test]
fn noncanonical_valid_input_reads_and_reencodes_canonically() {
    for r in NONCANONICAL {
        let (what, file) = (r.what, r.file);
        let input = read("noncanonical", file);
        let want = read(r.expected_dir, r.expected_file);
        assert_ne!(
            input, want,
            "{what}: the fixture is canonical already, so it proves nothing"
        );
        let (got, diagnostics) =
            reencode(r.kind, file, input.as_bytes()).unwrap_or_else(|e| panic!("{what}: {e}"));
        same_bytes(
            &format!("{what}: {file} does not re-encode to {}", r.expected_file),
            &got,
            &want,
        );
        let mut got: Vec<_> = diagnostics
            .iter()
            .map(|d| (d.file.as_str(), d.rule, d.detail.as_str()))
            .collect();
        got.sort_unstable();
        let mut want: Vec<_> = r
            .unknown
            .iter()
            .map(|p| (file, Rule::UnknownKey, *p))
            .collect();
        want.sort_unstable();
        assert_eq!(
            got, want,
            "{what}: the unknown keys the file holds, once each"
        );
        // An expectation outside `canonical/` is itself canonical.
        if r.expected_dir != "canonical" {
            let text = read(r.expected_dir, r.expected_file);
            let (again, _) = reencode(r.kind, r.expected_file, text.as_bytes()).unwrap();
            same_bytes(&format!("{} is canonical", r.expected_file), &again, &text);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Unknown keys (I12)

/// Adds three unknown keys to every typed object of `v`, after its known ones, and records the
/// JSON pointer each will be reported at. `snapshot`, `old` and `new` are opaque and get none.
fn inject_unknown(v: &mut Value, pointer: &str, expected: &mut Vec<String>) {
    match v {
        Value::Array(items) => {
            for (i, item) in items.iter_mut().enumerate() {
                inject_unknown(item, &format!("{pointer}/{i}"), expected);
            }
        }
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if !matches!(key.as_str(), "snapshot" | "old" | "new") {
                    inject_unknown(child, &format!("{pointer}/{key}"), expected);
                }
            }
            // Not alphabetical, one name that needs the escapes of RFC 6901, one `null` value.
            map.insert("x_zeta".into(), json!({ "b": 1, "a": [null, true, "s"] }));
            map.insert("x/odd~é".into(), json!([3, 2, 1]));
            map.insert("x_null".into(), Value::Null);
            expected.extend([
                format!("{pointer}/x_zeta"),
                format!("{pointer}/x~1odd~0é"),
                format!("{pointer}/x_null"),
            ]);
        }
        _ => {}
    }
}

/// `/steps/3/checks/0` is the shape `/steps/*/checks/*`.
fn shape(pointer: &str) -> String {
    pointer
        .rsplit_once('/')
        .map_or(String::new(), |(parent, _)| {
            parent
                .split('/')
                .map(|s| {
                    if s.bytes().all(|b| b.is_ascii_digit()) && !s.is_empty() {
                        "*"
                    } else {
                        s
                    }
                })
                .collect::<Vec<_>>()
                .join("/")
        })
}

#[test]
fn unknown_keys_survive_in_place_at_every_level() {
    let mut shapes = BTreeSet::new();
    for (name, kind) in CANONICAL {
        let mut value: Value = serde_json::from_str(&read("canonical", name)).unwrap();
        let mut expected = Vec::new();
        inject_unknown(&mut value, "", &mut expected);
        // The file with the keys written after the known ones, by the JSON library and not the codec.
        let mut text = serde_json::to_string_pretty(&value).unwrap();
        text.push('\n');

        let (again, diagnostics) =
            reencode(*kind, name, text.as_bytes()).unwrap_or_else(|e| panic!("{name}: {e}"));
        same_bytes(
            &format!("{name}: a key was dropped, moved or reordered"),
            &again,
            &text,
        );

        let mut got: Vec<(String, Rule, String)> = diagnostics
            .into_iter()
            .map(|d| (d.file, d.rule, d.detail))
            .collect();
        got.sort();
        let mut want: Vec<(String, Rule, String)> = expected
            .iter()
            .map(|p| ((*name).to_owned(), Rule::UnknownKey, p.clone()))
            .collect();
        want.sort();
        let missing: Vec<_> = want.iter().filter(|w| !got.contains(w)).collect();
        let unexpected: Vec<_> = got.iter().filter(|g| !want.contains(g)).collect();
        assert!(
            missing.is_empty() && unexpected.is_empty() && got.len() == want.len(),
            "{name}: exactly one `unknown_key` per kept key, naming the file and the path\n  missing: {missing:?}\n  unexpected: {unexpected:?}\n  reported {} for {} keys",
            got.len(),
            want.len()
        );
        shapes.extend(expected.iter().map(|p| shape(p)));
    }
    // Every typed object of the stored schema is exercised by some fixture.
    let required = [
        "",
        "/preconditions/*",
        "/success_gates/*",
        "/references/*",
        "/steps/*",
        "/steps/*/references/*",
        "/steps/*/checks/*",
        "/steps/*/commit",
        "/progress",
        "/history/*",
        "/history/*/actor",
        "/marks/*",
        "/marks/*/actor",
        "/actor",
        "/changes/*",
    ];
    for r in required {
        assert!(
            shapes.contains(r),
            "no fixture reaches the level `{r}`; got {shapes:?}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Identity over arbitrary stored values

fn text() -> impl Strategy<Value = String> {
    // Any scalar value, control characters included: the escapes are part of the contract.
    proptest::string::string_regex("(?s).{0,12}").unwrap()
}

fn json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::from),
        any::<u64>().prop_map(Value::from),
        (-1.0e15f64..1.0e15).prop_map(Value::from),
        text().prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            proptest::collection::vec((text(), inner), 0..4)
                .prop_map(|kv| Value::Object(kv.into_iter().collect())),
        ]
    })
}

/// Unknown keys: a prefix no known key has, so they never collide with one.
fn extra() -> impl Strategy<Value = Extra> {
    prop_oneof![
        3 => Just(Extra::new()),
        1 => proptest::collection::vec(("x_[a-z/~é]{1,4}", json_value()), 1..3).prop_map(|kv| kv.into_iter().collect()),
    ]
}

fn opt<T: Debug + Clone + 'static>(
    s: impl Strategy<Value = T>,
) -> impl Strategy<Value = Option<T>> {
    proptest::option::of(s)
}

fn lines() -> impl Strategy<Value = Lines> {
    proptest::collection::vec(text(), 0..3)
}

fn node_id() -> impl Strategy<Value = NodeId> {
    "n_[0-9a-hjkmnp-tv-z]{8}".prop_map(|s| NodeId::parse(&s).unwrap())
}
fn criterion_id() -> impl Strategy<Value = CriterionId> {
    "c_[0-9a-hjkmnp-tv-z]{6}".prop_map(|s| CriterionId::parse(&s).unwrap())
}
fn step_id() -> impl Strategy<Value = StepId> {
    "s_[0-9a-hjkmnp-tv-z]{6}".prop_map(|s| StepId::parse(&s).unwrap())
}
fn ulid() -> impl Strategy<Value = String> {
    "[0-9A-HJKMNP-TV-Z]{26}"
}
fn op_id() -> impl Strategy<Value = OpId> {
    "[ -~]{1,20}".prop_map(|s| OpId::parse(&s).unwrap())
}
fn name() -> impl Strategy<Value = Name> {
    "[a-z][a-z0-9_-]{0,10}".prop_map(|s| Name::parse(&s).unwrap())
}
fn order() -> impl Strategy<Value = OrderKey> {
    text().prop_map(OrderKey::new)
}
fn version() -> impl Strategy<Value = String> {
    Just("0.1.0".to_owned())
}
fn status() -> impl Strategy<Value = Status> {
    prop_oneof![
        Just(Status::Planned),
        Just(Status::Ongoing),
        Just(Status::Paused),
        Just(Status::Cancelled),
        Just(Status::Done)
    ]
}

fn actor() -> impl Strategy<Value = Actor> {
    (
        prop_oneof![
            Just(ActorKind::Agent),
            Just(ActorKind::Human),
            Just(ActorKind::Import)
        ],
        opt(text()),
        extra(),
    )
        .prop_map(|(kind, name, extra)| Actor { kind, name, extra })
}

fn criterion() -> impl Strategy<Value = Criterion> {
    (criterion_id(), text(), any::<bool>(), extra()).prop_map(|(id, text, met, extra)| Criterion {
        id,
        text,
        met,
        extra,
    })
}

fn reference() -> impl Strategy<Value = Reference> {
    (
        opt(text()),
        text(),
        opt(text()),
        opt(text()),
        opt(node_id()),
        extra(),
    )
        .prop_map(|(label, title, target, note, node_id, extra)| Reference {
            label,
            title,
            target,
            note,
            node_id,
            extra,
        })
}

fn mark() -> impl Strategy<Value = Mark> {
    (
        text(),
        actor(),
        criterion_id(),
        any::<bool>(),
        opt(text()),
        opt(text()),
        opt(any::<bool>()),
        extra(),
    )
        .prop_map(
            |(ts, actor, criterion_id, met, evidence, reason, correction, extra)| Mark {
                ts,
                actor,
                criterion_id,
                met,
                evidence,
                reason,
                correction,
                extra,
            },
        )
}

fn check() -> impl Strategy<Value = Check> {
    (
        text(),
        prop_oneof![Just(Expected::Pass), Just(Expected::Fail)],
        opt(text()),
        extra(),
    )
        .prop_map(|(text, expected, command, extra)| Check {
            text,
            expected,
            command,
            extra,
        })
}

fn commit() -> impl Strategy<Value = Commit> {
    (text(), opt(text()), any::<bool>(), text(), extra()).prop_map(
        |(r#type, scope, breaking, summary, extra)| Commit {
            r#type,
            scope,
            breaking,
            summary,
            extra,
        },
    )
}

fn step() -> impl Strategy<Value = Step> {
    (
        (
            step_id(),
            text(),
            text(),
            lines(),
            lines(),
            proptest::collection::vec(reference(), 0..2),
        ),
        (
            proptest::collection::vec(check(), 0..2),
            commit(),
            opt(lines()),
            any::<bool>(),
            opt(text()),
            opt(text()),
            extra(),
        ),
    )
        .prop_map(
            |(
                (id, title, goal, implementation_logic, deliverables, references),
                (checks, commit, requires, done, commit_sha, done_at, extra),
            )| Step {
                id,
                title,
                goal,
                implementation_logic,
                deliverables,
                references,
                checks,
                commit,
                requires,
                done,
                commit_sha,
                done_at,
                extra,
            },
        )
}

fn history() -> impl Strategy<Value = History> {
    (
        (text(), status(), status(), actor(), opt(text())),
        (
            opt(any::<bool>()),
            opt(op_id()),
            opt(any::<bool>()),
            extra(),
        ),
    )
        .prop_map(
            |((ts, from, to, actor, reason), (r#override, op_id, correction, extra))| History {
                ts,
                from,
                to,
                actor,
                reason,
                r#override,
                op_id,
                correction,
                extra,
            },
        )
}

fn progress() -> impl Strategy<Value = Progress> {
    (opt(text()), opt(any::<u64>()), lines(), extra()).prop_map(
        |(branch, commits, notes, extra)| Progress {
            branch,
            commits,
            notes,
            extra,
        },
    )
}

fn leaf() -> impl Strategy<Value = Leaf> {
    (
        (
            version(),
            node_id(),
            name(),
            node_id(),
            order(),
            text(),
            status(),
            opt(text()),
            text(),
        ),
        (
            proptest::collection::vec(criterion(), 0..3),
            proptest::collection::vec(criterion(), 0..3),
            proptest::collection::vec(reference(), 0..2),
            proptest::collection::vec(step(), 0..3),
            progress(),
            proptest::collection::vec(history(), 0..3),
            proptest::collection::vec(mark(), 0..3),
            extra(),
        ),
    )
        .prop_map(
            |(
                (schema_version, id, name, parent_id, order, title, status, status_reason, goal),
                (preconditions, success_gates, references, steps, progress, history, marks, extra),
            )| Leaf {
                schema_version,
                id,
                kind: LeafKind::Leaf,
                name,
                parent_id,
                order,
                title,
                status,
                status_reason,
                goal,
                preconditions,
                success_gates,
                references,
                steps,
                progress,
                history,
                marks,
                extra,
            },
        )
}

fn directory() -> impl Strategy<Value = Directory> {
    (
        (
            version(),
            node_id(),
            prop_oneof![
                Just(DirectoryKind::Campaign),
                Just(DirectoryKind::Directory)
            ],
            name(),
            opt(node_id()),
            opt(order()),
            text(),
            lines(),
        ),
        (
            proptest::collection::vec(reference(), 0..2),
            text(),
            proptest::collection::vec(criterion(), 0..3),
            proptest::collection::vec(criterion(), 0..3),
            lines(),
            lines(),
            proptest::collection::vec(mark(), 0..3),
            extra(),
        ),
    )
        .prop_map(
            |(
                (schema_version, id, kind, name, parent_id, order, title, context),
                (references, goal, preconditions, success_gates, gotchas, notes, marks, extra),
            )| Directory {
                schema_version,
                id,
                kind,
                name,
                parent_id,
                order,
                title,
                context,
                references,
                goal,
                preconditions,
                success_gates,
                gotchas,
                notes,
                marks,
                extra,
            },
        )
}

fn node() -> impl Strategy<Value = Node> {
    prop_oneof![
        leaf().prop_map(Node::Leaf),
        directory().prop_map(Node::Directory)
    ]
}

fn comment() -> impl Strategy<Value = Comment> {
    (
        version(),
        ulid(),
        node_id(),
        text(),
        actor(),
        lines(),
        opt(op_id()),
        extra(),
    )
        .prop_map(
            |(schema_version, id, node_id, ts, actor, text, op_id, extra)| Comment {
                schema_version,
                id: CommentId::parse(&id).unwrap(),
                node_id,
                ts,
                actor,
                text,
                op_id,
                extra,
            },
        )
}

fn change() -> impl Strategy<Value = Change> {
    (
        node_id(),
        prop_oneof![
            Just(ChangeOp::Add),
            Just(ChangeOp::Remove),
            Just(ChangeOp::Set),
            Just(ChangeOp::Rename),
            Just(ChangeOp::Move)
        ],
        opt(prop_oneof![
            criterion_id().prop_map(ItemId::Criterion),
            step_id().prop_map(ItemId::Step)
        ]),
        opt(text()),
        opt(json_value()),
        opt(json_value()),
        extra(),
    )
        .prop_map(|(node_id, op, item_id, field, old, new, extra)| Change {
            node_id,
            op,
            item_id,
            field,
            old,
            new,
            extra,
        })
}

fn amendment() -> impl Strategy<Value = Amendment> {
    (
        (
            version(),
            ulid(),
            opt(text()),
            text(),
            actor(),
            text(),
            opt(text()),
        ),
        (
            opt(op_id()),
            opt(any::<bool>()),
            proptest::collection::vec(change(), 0..3),
            opt(proptest::collection::vec((node_id(), json_value()), 0..3)),
            extra(),
        ),
    )
        .prop_map(
            |(
                (schema_version, id, legacy_id, ts, actor, description, source),
                (op_id, correction, changes, snapshot, extra),
            )| Amendment {
                schema_version,
                id: AmendmentId::parse(&id).unwrap(),
                legacy_id,
                ts,
                actor,
                description,
                source,
                op_id,
                correction,
                changes,
                snapshot: snapshot
                    .map(|kv| kv.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
                extra,
            },
        )
}

/// The value reads back equal from its text, and the text of the read value is the same text:
/// the second half also holds the order of keys, which `==` on a JSON map ignores.
fn identity<T: Stored + PartialEq + Debug>(value: &T) -> Result<(), TestCaseError> {
    let text = encode(value);
    prop_assert!(
        text.ends_with("}\n") && !text.ends_with("\n\n"),
        "one trailing newline, ends {:?}",
        &text[text.len().saturating_sub(8)..]
    );
    prop_assert!(!text.contains('\r'), "no CR in a canonical text");
    let back = decode::<T>("nodes/x.json", text.as_bytes())
        .map_err(|e| TestCaseError::fail(e.to_string()))?;
    prop_assert_eq!(&back.value, value);
    prop_assert_eq!(encode(&back.value), text);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("proptest-regressions/codec.txt"))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn decode_encode_is_identity(n in node(), c in comment(), a in amendment()) {
        identity(&n)?;
        identity(&c)?;
        identity(&a)?;
    }
}

// ---------------------------------------------------------------------------------------------
// The format version is read first

/// Each row is a hand-written canonical file of the shape a newer layer would write, with a body
/// that does not parse as the current type, and the version it carries.
type NewerRow = (&'static str, &'static str, Kind, Version);
const NEWER: &[NewerRow] = &[
    (
        "a higher X, known fields change type",
        "leaf_0_2_0.json",
        Kind::Node,
        (0, 2, 0),
    ),
    (
        "a higher Y, a known field changes type",
        "leaf_0_1_1.json",
        Kind::Node,
        (0, 1, 1),
    ),
    (
        "a higher Y of a comment",
        "comment_0_1_1.json",
        Kind::Comment,
        (0, 1, 1),
    ),
    (
        "a higher X of an amendment",
        "amendment_0_2_0.json",
        Kind::Amendment,
        (0, 2, 0),
    ),
];

#[test]
fn newer_file_refused_before_parsing() {
    for (row, name, kind, found) in NEWER {
        let text = read("newer", name);
        let file = format!("from-{name}");
        let broken = match kind {
            Kind::Node => serde_json::from_str::<Leaf>(&text).is_err(),
            Kind::Comment => serde_json::from_str::<Comment>(&text).is_err(),
            Kind::Amendment => serde_json::from_str::<Amendment>(&text).is_err(),
        };
        assert!(
            broken,
            "{row}: the fixture parses as the current type, so it proves nothing"
        );
        match reencode(*kind, &file, text.as_bytes()) {
            Err(StoreError::NewerFormat {
                found: got,
                supported,
                file: named,
            }) => {
                assert_eq!(
                    (got, supported, named),
                    (*found, rdm_schema::FORMAT_VERSION, file),
                    "{row}"
                );
            }
            other => panic!("{row}: want NewerFormat {{ found: {found:?} }}, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// A refused file names the rule it breaks and itself

/// Sets (or, with `None`, removes) the value at `pointer` of a canonical fixture, as a hand edit
/// that breaks it would.
fn edited(fixture: &str, pointer: &str, to: Option<Value>) -> String {
    let mut value: Value = serde_json::from_str(&read("canonical", fixture)).unwrap();
    match to {
        Some(v) => {
            *value
                .pointer_mut(pointer)
                .unwrap_or_else(|| panic!("{pointer} is not in {fixture}")) = v
        }
        None => {
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            value
                .pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key)
                .unwrap();
        }
    }
    serde_json::to_string_pretty(&value).unwrap()
}

#[test]
fn refused_files_name_their_rule_and_file() {
    let bad = |kind, fixture, pointer: &str, to, rule| {
        (
            kind,
            fixture,
            edited(fixture, pointer, to),
            rule,
            pointer.to_owned(),
        )
    };
    let rows: Vec<(Kind, &str, String, Rule, String)> = vec![
        // Not JSON, or not an object of the type.
        (
            Kind::Node,
            "leaf.json",
            "{\"schema_version\": ".to_owned(),
            Rule::Parse,
            "truncated".to_owned(),
        ),
        (
            Kind::Node,
            "leaf.json",
            "[1, 2]".to_owned(),
            Rule::Parse,
            "an array".to_owned(),
        ),
        (
            Kind::Comment,
            "comment.json",
            String::new(),
            Rule::Parse,
            "empty".to_owned(),
        ),
        bad(
            Kind::Node,
            "leaf.json",
            "/goal",
            Some(json!(5)),
            Rule::Parse,
        ),
        bad(Kind::Node, "leaf.json", "/title", None, Rule::Parse),
        // `kind` is not one of the three values, or is missing.
        bad(
            Kind::Node,
            "leaf.json",
            "/kind",
            Some(json!("task")),
            Rule::Kind,
        ),
        bad(Kind::Node, "directory.json", "/kind", None, Rule::Kind),
        // An id field that does not parse as its type: a letter outside the alphabet, a wrong prefix,
        // a wrong length, a lower-case ULID, an empty op id, a snapshot key that is a path.
        bad(
            Kind::Node,
            "leaf.json",
            "/id",
            Some(json!("n_7k3q9x2i")),
            Rule::MalformedId,
        ),
        bad(
            Kind::Node,
            "leaf.json",
            "/parent_id",
            Some(json!("c_3d9f6h")),
            Rule::MalformedId,
        ),
        bad(
            Kind::Node,
            "leaf.json",
            "/steps/0/id",
            Some(json!("s_zz")),
            Rule::MalformedId,
        ),
        bad(
            Kind::Node,
            "leaf.json",
            "/marks/0/criterion_id",
            Some(json!("c_8m2xqu")),
            Rule::MalformedId,
        ),
        bad(
            Kind::Node,
            "leaf.json",
            "/references/1/node_id",
            Some(json!("n_3d9f6h1")),
            Rule::MalformedId,
        ),
        bad(
            Kind::Node,
            "campaign.json",
            "/preconditions/0/id",
            Some(json!("x_4d7k1h")),
            Rule::MalformedId,
        ),
        bad(
            Kind::Comment,
            "comment.json",
            "/id",
            Some(json!("01jd0000000000000000000001")),
            Rule::MalformedId,
        ),
        bad(
            Kind::Comment,
            "comment.json",
            "/op_id",
            Some(json!("")),
            Rule::MalformedId,
        ),
        bad(
            Kind::Amendment,
            "amendment.json",
            "/changes/0/node_id",
            Some(json!("n_7k3q9x2")),
            Rule::MalformedId,
        ),
        bad(
            Kind::Amendment,
            "amendment.json",
            "/changes/3/item_id",
            Some(json!("n_7k3q9x2m")),
            Rule::MalformedId,
        ),
        {
            let mut v: Value = serde_json::from_str(&read("canonical", "amendment.json")).unwrap();
            let snapshot = v["snapshot"].as_object_mut().unwrap();
            let node = snapshot.remove("n_9n3s7y5a").unwrap();
            snapshot.insert("core/m2".into(), node);
            (
                Kind::Amendment,
                "amendment.json",
                serde_json::to_string_pretty(&v).unwrap(),
                Rule::MalformedId,
                "/snapshot key".to_owned(),
            )
        },
        // The stamp is read first: missing, not a string, not `N.N.N`, or older than the first layer.
        bad(
            Kind::Node,
            "leaf.json",
            "/schema_version",
            None,
            Rule::SchemaVersion,
        ),
        bad(
            Kind::Node,
            "leaf.json",
            "/schema_version",
            Some(json!(1)),
            Rule::SchemaVersion,
        ),
        bad(
            Kind::Comment,
            "comment.json",
            "/schema_version",
            Some(json!("0.1")),
            Rule::SchemaVersion,
        ),
        bad(
            Kind::Amendment,
            "amendment.json",
            "/schema_version",
            Some(json!("0.01.0")),
            Rule::SchemaVersion,
        ),
        bad(
            Kind::Node,
            "leaf.json",
            "/schema_version",
            Some(json!("0.0.9")),
            Rule::UnknownFormat,
        ),
        // Nothing else of the file is looked at before the stamp: a bad stamp over a bad id.
        {
            let mut v: Value =
                serde_json::from_str(&edited("leaf.json", "/id", Some(json!("n_7k3q9x2i"))))
                    .unwrap();
            v["schema_version"] = json!("x");
            (
                Kind::Node,
                "leaf.json",
                serde_json::to_string_pretty(&v).unwrap(),
                Rule::SchemaVersion,
                "stamp before id".to_owned(),
            )
        },
        // A key written twice, at any level: the later one must not win silently.
        {
            let t = read("canonical", "leaf.json").replace(
                "  \"title\": \"M1 Core foundation\",\n",
                "  \"title\": \"M1 Core foundation\",\n  \"title\": \"again\",\n",
            );
            (
                Kind::Node,
                "leaf.json",
                t,
                Rule::Parse,
                "duplicate key at /title".to_owned(),
            )
        },
        {
            let t = read("canonical", "comment.json").replace(
                "  \"op_id\": \"op2\"\n",
                "  \"op_id\": \"op2\",\n  \"x_a\": 1,\n  \"x_a\": 2\n",
            );
            (
                Kind::Comment,
                "comment.json",
                t,
                Rule::Parse,
                "duplicate key at /x_a".to_owned(),
            )
        },
        {
            let t = read("canonical", "leaf.json").replace(
                "        \"breaking\": false,\n",
                "        \"breaking\": false,\n        \"breaking\": true,\n",
            );
            (
                Kind::Node,
                "leaf.json",
                t,
                Rule::Parse,
                "duplicate key at /steps/0/commit/breaking".to_owned(),
            )
        },
        // A name that is not a path segment.
        bad(
            Kind::Node,
            "leaf.json",
            "/name",
            Some(json!("Core")),
            Rule::NameFormat,
        ),
        bad(
            Kind::Node,
            "directory.json",
            "/name",
            Some(json!("co re")),
            Rule::NameFormat,
        ),
    ];
    for (kind, fixture, text, rule, what) in rows {
        let file = format!("nodes/from-{fixture}");
        match reencode(kind, &file, text.as_bytes()) {
            Err(StoreError::Invalid {
                file: named,
                rule: got,
                detail,
            }) => {
                assert!(
                    !detail.starts_with(MALFORMED_ID_TAG) && !detail.starts_with(NAME_FORMAT_TAG),
                    "{fixture} at {what}: the detail carries a tag: {detail}"
                );
                if what.starts_with("duplicate key at ") {
                    assert!(
                        detail.contains(&what),
                        "{fixture}: the detail does not name {what}: {detail}"
                    );
                }
                assert_eq!(
                    (named.as_str(), got),
                    (file.as_str(), rule),
                    "{fixture} at {what}"
                );
            }
            other => panic!("{fixture} at {what}: want Invalid {{ {rule:?} }}, got {other:?}"),
        }
    }
}
