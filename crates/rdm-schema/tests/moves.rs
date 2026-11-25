use rdm_schema::moves::{MOVES, ReadyEffect, ReasonRule, Who, lookup};
use rdm_schema::stored::Status;

const HEADER: &str = "| From | To | Allowed | Who | Reason | Precondition | Ready set |";

/// The cells of the move table of `statuses.md`: the first table after `## Transitions`, whose
/// header row is exactly `HEADER`. The document is the oracle, nothing of it is copied here.
fn doc_rows() -> Vec<Vec<String>> {
    let path = format!(
        "{}/../../__design__/rdm/statuses.md",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut lines = text.lines().skip_while(|l| *l != "## Transitions");
    assert!(
        lines.next().is_some(),
        "no `## Transitions` heading in {path}"
    );
    let mut table = lines.skip_while(|l| !l.starts_with('|'));
    assert_eq!(
        table.next(),
        Some(HEADER),
        "the header of the first table after `## Transitions` changed"
    );
    assert!(
        table.next().is_some_and(|l| l.starts_with("|:--")),
        "no separator row"
    );
    table
        .take_while(|l| l.starts_with('|'))
        .map(|l| {
            l.trim()
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(|c| c.trim().to_owned())
                .collect()
        })
        .collect()
}

fn word<T: serde::de::DeserializeOwned>(w: &str) -> T {
    serde_json::from_value(serde_json::Value::String(w.to_owned()))
        .unwrap_or_else(|e| panic!("{w:?}: {e}"))
}

fn status(cell: &str) -> Status {
    word(
        cell.strip_prefix('`')
            .and_then(|c| c.strip_suffix('`'))
            .unwrap_or_else(|| panic!("{cell:?} is not backticked")),
    )
}

/// The leading uppercase tokens of a ready-set cell, `^[A-Z]+(, [A-Z]+)*`.
fn effects(cell: &str) -> Vec<ReadyEffect> {
    if cell == "none" {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut rest = cell;
    loop {
        let n = rest.bytes().take_while(u8::is_ascii_uppercase).count();
        assert!(n > 0, "{cell:?}: no token");
        out.push(word::<ReadyEffect>(&rest[..n].to_lowercase()));
        match rest[n..].strip_prefix(", ") {
            Some(next) if next.starts_with(|c: char| c.is_ascii_uppercase()) => rest = next,
            _ => return out,
        }
    }
}

#[test]
fn move_table_matches_statuses_doc() {
    let rows = doc_rows();
    assert_eq!(
        rows.len(),
        25,
        "the table holds the 25 ordered pairs of the five statuses"
    );
    let mut seen = [false; 25];
    for cells in &rows {
        assert_eq!(cells.len(), 7, "{cells:?}");
        let (from, to) = (status(&cells[0]), status(&cells[1]));
        let at = from as usize * 5 + to as usize;
        assert!(
            !std::mem::replace(&mut seen[at], true),
            "{from:?} -> {to:?} appears twice in the document"
        );
        let m = &MOVES[at];
        let pair = format!("{from:?} -> {to:?}");
        assert_eq!(
            (m.from, m.to),
            (from, to),
            "MOVES[{at}] is not the pair its index says"
        );
        assert_eq!(m.allowed, cells[2] == "allowed", "{pair}: Allowed");
        let who = (!cells[3].is_empty()).then(|| word::<Who>(&cells[3]));
        assert_eq!(m.who, who, "{pair}: Who");
        let reason = match cells[4].as_str() {
            "required" => ReasonRule::Required,
            "only for an override" => ReasonRule::OnlyForOverride,
            "none" | "" => ReasonRule::NotRequired,
            other => panic!("{pair}: unknown reason cell {other:?}"),
        };
        assert_eq!(m.reason, reason, "{pair}: Reason");
        assert_eq!(m.precondition, cells[5], "{pair}: Precondition");
        assert_eq!(
            m.ready_set,
            effects(&cells[6]).as_slice(),
            "{pair}: Ready set"
        );
        // `allowed` exactly when someone may make the move, and `lookup` finds the same row.
        assert_eq!(m.allowed, m.who.is_some(), "{pair}");
        assert!(std::ptr::eq(lookup(from, to), m), "{pair}: lookup");
    }
    assert!(
        seen.iter().all(|s| *s),
        "a pair of MOVES has no row in the document"
    );
}
