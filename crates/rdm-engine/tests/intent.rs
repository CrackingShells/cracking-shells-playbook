//! Guard of the intent record as a later binary or a crash leaves it on disk.

use rdm_engine::fs::{Durability, RealFs};
use rdm_engine::intent::{Intent, IntentEffect, Pending, begin, complete, pending};
use std::path::Path;

fn put(dir: &Path, name: &str, bytes: &[u8]) {
    let d = dir.join(".rdm").join("intent");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join(name), bytes).unwrap();
}

const A: &str = "01J9ZQ3M8R5K7V2N4X6Y8A0B1A";
const B: &str = "01J9ZQ3M8R5K7V2N4X6Y8A0B1B";
const C: &str = "01J9ZQ3M8R5K7V2N4X6Y8A0B1C";
const D: &str = "01J9ZQ3M8R5K7V2N4X6Y8A0B1D";

#[test]
fn pending_reports_newer_and_undecodable_records_oldest_first() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // A record this binary wrote, through `begin`.
    let mine = Intent::new(vec![
        IntentEffect::Put {
            path: "nodes/n_aaaaaaaa.json".into(),
            bytes: "{}\r\n".into(),
        },
        IntentEffect::Create {
            path: "_amendments/X.json".into(),
            bytes: "é".into(),
        },
        IntentEffect::Del {
            path: "nodes/n_bbbbbbbb.json".into(),
        },
    ]);
    begin(&RealFs, root, D, &mine, Durability::Fsync).unwrap();
    // What a newer binary would leave after a crash: a higher format with an effect this one
    // does not know. It must be refused as newer, not misread as damage or redone.
    put(
        root,
        &format!("{A}.json"),
        br#"{"format":2,"effects":[{"action":"patch","path":"p","diff":"d"}]}"#,
    );
    // Truncated JSON, a put without bytes, a del with bytes: none decodes.
    put(
        root,
        &format!("{B}.json"),
        br#"{"format":1,"effects":[{"action":"put","pa"#,
    );
    put(
        root,
        &format!("{C}.json"),
        br#"{"format":1,"effects":[{"action":"put","path":"p"}]}"#,
    );
    put(
        root,
        "01J9ZQ3M8R5K7V2N4X6Y8A0B1E.json",
        br#"{"format":1,"effects":[{"action":"del","path":"p","bytes":"x"}]}"#,
    );
    // Names that are not a ULID plus .json are ignored.
    put(root, "notes.txt", b"x");
    put(root, "01J9ZQ3M8R5K7V2N4X6Y8A0B1F.json.bak", b"x");
    put(root, "short.json", b"{}");

    let got = pending(&RealFs, root).unwrap();
    let ulids: Vec<&str> = got.iter().map(Pending::ulid).collect();
    assert_eq!(
        ulids,
        [A, B, C, D, "01J9ZQ3M8R5K7V2N4X6Y8A0B1E"],
        "oldest ULID first, odd names ignored"
    );
    assert_eq!(
        got[0],
        Pending::Newer {
            ulid: A.into(),
            found: 2
        }
    );
    assert_eq!(got[1], Pending::Undecodable { ulid: B.into() });
    assert_eq!(got[2], Pending::Undecodable { ulid: C.into() });
    assert_eq!(
        got[3],
        Pending::Record {
            ulid: D.into(),
            intent: mine
        }
    );
    assert_eq!(
        got[4],
        Pending::Undecodable {
            ulid: "01J9ZQ3M8R5K7V2N4X6Y8A0B1E".into()
        }
    );

    complete(&RealFs, root, D, Durability::Fsync).unwrap();
    assert_eq!(pending(&RealFs, root).unwrap().len(), 4);
    assert_eq!(std::fs::read_dir(root.join(".rdm/tmp")).unwrap().count(), 0);
}

#[test]
fn begin_refuses_a_name_that_is_not_a_ulid_and_never_overwrites_a_record() {
    let dir = tempfile::tempdir().unwrap();
    let intent = Intent::new(vec![IntentEffect::Del { path: "p".into() }]);
    for bad in [
        "",
        "short",
        "../../escape",
        "01j9zq3m8r5k7v2n4x6y8a0b1a",
        "01J9ZQ3M8R5K7V2N4X6Y8A0B1AA",
    ] {
        let err = begin(&RealFs, dir.path(), bad, &intent, Durability::Process).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput, "{bad:?}");
    }
    assert!(!dir.path().join(".rdm").join("intent").exists());
    begin(&RealFs, dir.path(), A, &intent, Durability::Process).unwrap();
    let other = Intent::new(vec![IntentEffect::Del { path: "q".into() }]);
    let err = begin(&RealFs, dir.path(), A, &other, Durability::Process).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(
        pending(&RealFs, dir.path()).unwrap(),
        [Pending::Record {
            ulid: A.into(),
            intent
        }]
    );
}
