//! Embeds the built view when it exists, a placeholder page otherwise, and
//! generates the table of fixture leaves that `leaf_get` serves.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs};

const PLACEHOLDER: &str = "<!doctype html><html><head><meta charset=\"utf-8\">\
<title>rdm graph</title></head><body><p>The view is not built: \
run the build in crates/rdm-mcp/view first.</p></body></html>";

fn collect(dir: &Path, root: &Path, out: &mut Vec<(String, PathBuf)>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("fixtures/leaves must exist")
        .map(|e| e.expect("readable entry").path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            collect(&p, root, out);
        } else {
            let rel = p.strip_prefix(root).expect("under root");
            out.push((rel.to_string_lossy().replace('\\', "/"), p));
        }
    }
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets it"));
    let out = PathBuf::from(env::var("OUT_DIR").expect("cargo sets it"));

    let view = manifest.join("view/dist/index.html");
    println!("cargo:rerun-if-changed={}", view.display());
    let html = fs::read_to_string(&view).unwrap_or_else(|_| PLACEHOLDER.to_owned());
    fs::write(out.join("view.html"), html).expect("write view.html");

    let leaves = manifest.join("fixtures/leaves");
    println!("cargo:rerun-if-changed={}", leaves.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest.join("fixtures/roadmap.json").display()
    );
    let mut files = Vec::new();
    collect(&leaves, &leaves, &mut files);
    let mut table = String::from("pub static LEAVES: &[(&str, &str)] = &[\n");
    for (rel, abs) in &files {
        writeln!(
            table,
            "    ({rel:?}, include_str!({:?})),",
            abs.to_string_lossy()
        )
        .expect("write to String");
    }
    table.push_str("];\n");
    fs::write(out.join("leaves.rs"), table).expect("write leaves.rs");
}
