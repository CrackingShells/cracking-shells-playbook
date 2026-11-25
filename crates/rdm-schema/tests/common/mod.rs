#![allow(dead_code)]
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

pub fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&text).unwrap()
}

/// Deserialise to `T`, serialise back, and require equal JSON values.
pub fn round_trip<T: Serialize + DeserializeOwned>(v: &Value) {
    let t: T = serde_json::from_value(v.clone()).unwrap_or_else(|e| panic!("deserialise: {e}"));
    assert_eq!(&serde_json::to_value(&t).unwrap(), v);
}

/// Adds an unknown key to every object that is a typed object. Maps and opaque JSON values
/// (`holders`, `snapshot`, `old`, `new`) get none themselves; `holders` entries do.
pub fn inject_unknown(v: &mut Value, path: &str) -> usize {
    let mut n = 0;
    match v {
        Value::Array(a) => {
            for (i, x) in a.iter_mut().enumerate() {
                n += inject_unknown(x, &format!("{path}[{i}]"));
            }
        }
        Value::Object(m) => {
            for (k, x) in m.iter_mut() {
                match k.as_str() {
                    "snapshot" | "old" | "new" => {}
                    "holders" => {
                        if let Value::Object(h) = x {
                            for (_, e) in h.iter_mut() {
                                n += inject_unknown(e, &format!("{path}.holders.*"));
                            }
                        }
                    }
                    _ => n += inject_unknown(x, &format!("{path}.{k}")),
                }
            }
            m.insert(
                "x_unknown".into(),
                json!({ "at": path, "keep": [1, "two", null] }),
            );
            n += 1;
        }
        _ => {}
    }
    n
}

/// Fixture + unknown field at every object level must survive a round trip.
pub fn unknown_fields_survive<T: Serialize + DeserializeOwned>(name: &str) -> usize {
    let mut v = fixture(name);
    let n = inject_unknown(&mut v, "$");
    round_trip::<T>(&v);
    n
}
