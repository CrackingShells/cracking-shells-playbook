/// Low-level README.md read/write helpers.
///
/// Provides targeted mutations that operate on the in-memory line buffer,
/// then persist atomically via a temp file + rename. Every write is guarded
/// by pre- and post-validation.
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

use crate::validate::{validate_readme_str, Violation};

// ── status helpers ─────────────────────────────────────────────────────────

pub const STATUSES: &[&str] = &["planned", "inprogress", "done", "amendment", "blocked"];

pub fn status_emoji(status: &str) -> &'static str {
    match status {
        "done"       => "✅ Done",
        "inprogress" => "🔄 In Progress",
        "planned"    => "⬜ Planned",
        "amendment"  => "🔵 Amendment",
        "blocked"    => "🚫 Blocked",
        _            => "⬜ Planned",
    }
}

pub fn validate_status(status: &str) -> Result<()> {
    if STATUSES.contains(&status) {
        Ok(())
    } else {
        bail!(
            "invalid status {:?}; must be one of: {}",
            status,
            STATUSES.join(", ")
        )
    }
}

pub fn validate_node_name(name: &str) -> Result<()> {
    let re = regex::Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
    if re.is_match(name) {
        Ok(())
    } else {
        bail!(
            "invalid node name {:?}; must match ^[a-z][a-z0-9_-]*$ (no numeric prefixes, lowercase only)",
            name
        )
    }
}

// ── atomic write with validation ───────────────────────────────────────────

/// Write `lines` to `path` atomically via a temp file.
/// Validates the result before committing; returns violations on post-check failure.
pub fn atomic_write(path: &Path, lines: &[String]) -> Result<Vec<Violation>> {
    let tmp = path.with_extension("md.tmp");
    let content = lines.join("\n") + "\n";
    std::fs::write(&tmp, &content)
        .with_context(|| format!("writing temp file {}", tmp.display()))?;

    let violations = validate_readme_str(&content)?;
    if !violations.is_empty() {
        std::fs::remove_file(&tmp).ok();
        return Ok(violations);
    }

    std::fs::rename(&tmp, path)
        .with_context(|| format!("renaming {} → {}", tmp.display(), path.display()))?;
    Ok(vec![])
}

// ── line buffer mutations ──────────────────────────────────────────────────

/// Find the line index of the first `classDef` inside the Mermaid block.
/// Returns None if not found.
fn find_first_classdef(lines: &[String]) -> Option<usize> {
    let mut in_mermaid = false;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "```mermaid" {
            in_mermaid = true;
            continue;
        }
        if in_mermaid && line.trim().starts_with("classDef") {
            return Some(i);
        }
        if in_mermaid && line.trim() == "```" {
            break;
        }
    }
    None
}

/// Find the line index of a node declaration in the Mermaid block by node_id.
fn find_mermaid_node(lines: &[String], node_id: &str) -> Option<usize> {
    let prefix = format!("    {node_id}[");
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(&prefix) {
            return Some(i);
        }
    }
    None
}

/// Find the line index of a row in the Nodes table by filesystem name (with backticks).
fn find_nodes_table_row(lines: &[String], fs_name: &str) -> Option<usize> {
    let needle = format!("| `{fs_name}` |");
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(&needle) || line.contains(&needle) {
            return Some(i);
        }
    }
    None
}

/// Find where the Nodes table ends (line after last data row).
fn find_nodes_table_end(lines: &[String]) -> Option<usize> {
    let mut in_nodes = false;
    let mut last_row: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "## Nodes" {
            in_nodes = true;
            continue;
        }
        if in_nodes {
            if line.starts_with('|') {
                last_row = Some(i);
            } else if last_row.is_some() && !line.trim().is_empty() {
                break;
            }
        }
    }
    last_row.map(|i| i + 1)
}

// ── public mutations ───────────────────────────────────────────────────────

/// Add a node to the Mermaid block and Nodes table.
pub fn add_node(
    readme_path: &Path,
    node_id: &str,
    fs_name: &str,
    is_dir: bool,
    title: &str,
) -> Result<()> {
    // pre-flight
    pre_flight(readme_path)?;

    let content = std::fs::read_to_string(readme_path)?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    // insert Mermaid node declaration before first classDef
    if let Some(idx) = find_first_classdef(&lines) {
        lines.insert(idx, format!("    {node_id}[{title}]:::planned"));
    } else {
        bail!("README.md Mermaid block is missing classDef lines; run validate to diagnose");
    }

    // Directory nodes are stored with a trailing slash in the Nodes table.
    let table_name = if is_dir { format!("{fs_name}/") } else { fs_name.to_string() };
    let type_label = if is_dir { "📁 Directory" } else { "📄 Leaf Task" };
    let new_row = format!("| `{table_name}` | {type_label} | ⬜ Planned |");
    if let Some(end_idx) = find_nodes_table_end(&lines) {
        lines.insert(end_idx, new_row);
    } else {
        bail!("README.md Nodes table not found; run validate to diagnose");
    }

    let violations = atomic_write(readme_path, &lines)?;
    report_violations_or_ok(readme_path, violations)
}

/// Update a node's status in the Mermaid block and Nodes table.
pub fn update_node_status(
    readme_path: &Path,
    node_id: &str,
    fs_name: &str,
    is_dir: bool,
    new_status: &str,
) -> Result<()> {
    validate_status(new_status)?;
    pre_flight(readme_path)?;

    let content = std::fs::read_to_string(readme_path)?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    // update Mermaid node :::status
    if let Some(idx) = find_mermaid_node(&lines, node_id) {
        let line = &lines[idx];
        let re = regex::Regex::new(r":::(done|inprogress|planned|amendment|blocked)$").unwrap();
        lines[idx] = re
            .replace(line, format!(":::{new_status}").as_str())
            .to_string();
    } else {
        bail!("node '{node_id}' not found in Mermaid block of {}", readme_path.display());
    }

    // Directory nodes are stored with a trailing slash in the Nodes table.
    let table_name = if is_dir { format!("{fs_name}/") } else { fs_name.to_string() };
    if let Some(idx) = find_nodes_table_row(&lines, &table_name) {
        let emoji_re = regex::Regex::new(
            r"(✅ Done|🔄 In Progress|⬜ Planned|🔵 Amendment|🚫 Blocked)",
        )
        .unwrap();
        lines[idx] = emoji_re
            .replace(&lines[idx], status_emoji(new_status))
            .to_string();
    } else {
        bail!("node '{table_name}' not found in Nodes table of {}", readme_path.display());
    }

    let violations = atomic_write(readme_path, &lines)?;
    report_violations_or_ok(readme_path, violations)
}

/// Remove a node from the Mermaid block and Nodes table.
/// Returns the node's current status (for use by `move`).
pub fn remove_node(
    readme_path: &Path,
    node_id: &str,
    fs_name: &str,
    is_dir: bool,
) -> Result<String> {
    pre_flight(readme_path)?;

    let content = std::fs::read_to_string(readme_path)?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    // capture current status before removal
    let current_status = if let Some(idx) = find_mermaid_node(&lines, node_id) {
        let re = regex::Regex::new(r":::(done|inprogress|planned|amendment|blocked)$").unwrap();
        re.captures(&lines[idx])
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "planned".to_string())
    } else {
        bail!("node '{node_id}' not found in Mermaid block of {}", readme_path.display());
    };

    // remove Mermaid line
    if let Some(idx) = find_mermaid_node(&lines, node_id) {
        lines.remove(idx);
    }
    // Directory nodes are stored with a trailing slash in the Nodes table.
    let table_name = if is_dir { format!("{fs_name}/") } else { fs_name.to_string() };
    if let Some(idx) = find_nodes_table_row(&lines, &table_name) {
        lines.remove(idx);
    } else {
        bail!("node '{table_name}' not found in Nodes table of {}", readme_path.display());
    }

    let violations = atomic_write(readme_path, &lines)?;
    report_violations_or_ok(readme_path, violations)?;
    Ok(current_status)
}

/// Read the current status of a node from the Mermaid block.
pub fn read_node_status(readme_path: &Path, node_id: &str) -> Result<String> {
    let content = std::fs::read_to_string(readme_path)?;
    let lines: Vec<String> = content.lines().map(String::from).collect();
    if let Some(idx) = find_mermaid_node(&lines, node_id) {
        let re = regex::Regex::new(r":::(done|inprogress|planned|amendment|blocked)$").unwrap();
        if let Some(caps) = re.captures(&lines[idx]) {
            return Ok(caps.get(1).unwrap().as_str().to_string());
        }
    }
    bail!("node '{node_id}' not found in {}", readme_path.display())
}

/// Read all rows from the Nodes table.
pub fn read_nodes_table(readme_path: &Path) -> Result<Vec<(String, String, String)>> {
    let content = std::fs::read_to_string(readme_path)?;
    // Leaf rows end in .md; directory rows end in /  (per BNF).
    let row_re = regex::Regex::new(
        r"^\| `([a-z][a-z0-9_-]*(?:\.md|/))` \| (📄 Leaf Task|📁 Directory) \| (✅ Done|🔄 In Progress|⬜ Planned|🔵 Amendment|🚫 Blocked) \|$"
    ).unwrap();
    let mut rows = Vec::new();
    for line in content.lines() {
        if let Some(caps) = row_re.captures(line) {
            rows.push((
                caps[1].to_string(),
                caps[2].to_string(),
                caps[3].to_string(),
            ));
        }
    }
    Ok(rows)
}

// ── helpers ────────────────────────────────────────────────────────────────

fn pre_flight(readme_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(readme_path)
        .with_context(|| format!("reading {}", readme_path.display()))?;
    let violations = validate_readme_str(&content)?;
    if violations.is_empty() {
        return Ok(());
    }
    let mut msg = format!(
        "pre-flight validation failed for {}:\n",
        readme_path.display()
    );
    for v in &violations {
        msg.push_str(&format!("  {v}\n"));
    }
    msg.push_str("Run `dirtree-rdm validate <dir>` for details. Do not edit the Mermaid block, Nodes table, or Amendment Log by hand.");
    bail!("{}", msg)
}

fn report_violations_or_ok(path: &Path, violations: Vec<Violation>) -> Result<()> {
    if violations.is_empty() {
        Ok(())
    } else {
        let mut msg = format!(
            "post-write validation failed for {} (change was NOT committed):\n",
            path.display()
        );
        for v in &violations {
            msg.push_str(&format!("  {v}\n"));
        }
        bail!("{}", msg)
    }
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Bug 2: directory nodes must be stored/found with trailing slash ────

    #[test]
    fn test_find_nodes_table_row_dir_with_slash() {
        let lines: Vec<String> = vec![
            "| `optimize/` | 📁 Directory | ⬜ Planned |".to_string(),
        ];
        assert!(
            find_nodes_table_row(&lines, "optimize/").is_some(),
            "should find directory row by 'name/'"
        );
    }

    #[test]
    fn test_find_nodes_table_row_leaf_no_slash() {
        let lines: Vec<String> = vec![
            "| `work.md` | 📄 Leaf Task | ⬜ Planned |".to_string(),
        ];
        assert!(
            find_nodes_table_row(&lines, "work.md").is_some(),
            "should find leaf row by 'name.md'"
        );
    }

    #[test]
    fn test_add_node_writes_dir_with_slash() {
        // Build a minimal valid README in a temp file, call add_node for a dir,
        // and confirm the Nodes table row ends with `/`.
        let tmp = std::env::temp_dir().join("dirtree_rdm_test_add_dir.md");
        let content = "# Test\n\n\
            ## Context\nsome context\n\n\
            ## Goal\nsome goal\n\n\
            ## Pre-conditions\n- [ ] criteria\n\n\
            ## Success Gates\n- \u{2705} gate\n\n\
            ## Status\n\
            ```mermaid\n\
            graph TD\n\
            \x20   classDef done       fill:#166534,color:#bbf7d0\n\
            \x20   classDef inprogress fill:#854d0e,color:#fef08a\n\
            \x20   classDef planned    fill:#374151,color:#e5e7eb\n\
            \x20   classDef amendment  fill:#1e3a5f,color:#bfdbfe\n\
            \x20   classDef blocked    fill:#7f1d1d,color:#fecaca\n\
            ```\n\n\
            ## Nodes\n\
            | Node | Type | Status |\n\
            |:-----|:-----|:-------|\n\n\
            ## Amendment Log\n\
            | ID | Date | Source | Nodes Added | Rationale |\n\
            |:---|:-----|:-------|:------------|:----------|\n\n\
            ## Progress\n\
            | Node | Branch | Commits | Notes |\n\
            |:-----|:-------|:--------|:------|\n";
        std::fs::write(&tmp, content).unwrap();

        add_node(&tmp, "mydir", "mydir", true, "My Dir").unwrap();

        let written = std::fs::read_to_string(&tmp).unwrap();
        std::fs::remove_file(&tmp).ok();

        assert!(
            written.contains("| `mydir/` |"),
            "add_node should write directory with trailing slash; got:\n{written}"
        );
    }

    #[test]
    fn test_update_node_status_dir_with_slash() {
        // README has a directory row with trailing slash; update_node_status must find it.
        let tmp = std::env::temp_dir().join("dirtree_rdm_test_update_dir.md");
        let content = "# Test\n\n\
            ## Context\nsome context\n\n\
            ## Goal\nsome goal\n\n\
            ## Pre-conditions\n- [ ] criteria\n\n\
            ## Success Gates\n- \u{2705} gate\n\n\
            ## Status\n\
            ```mermaid\n\
            graph TD\n\
            \x20   optimize[Optimize]:::planned\n\
            \x20   classDef done       fill:#166534,color:#bbf7d0\n\
            \x20   classDef inprogress fill:#854d0e,color:#fef08a\n\
            \x20   classDef planned    fill:#374151,color:#e5e7eb\n\
            \x20   classDef amendment  fill:#1e3a5f,color:#bfdbfe\n\
            \x20   classDef blocked    fill:#7f1d1d,color:#fecaca\n\
            ```\n\n\
            ## Nodes\n\
            | Node | Type | Status |\n\
            |:-----|:-----|:-------|\n\
            | `optimize/` | 📁 Directory | ⬜ Planned |\n\n\
            ## Amendment Log\n\
            | ID | Date | Source | Nodes Added | Rationale |\n\
            |:---|:-----|:-------|:------------|:----------|\n\n\
            ## Progress\n\
            | Node | Branch | Commits | Notes |\n\
            |:-----|:-------|:--------|:------|\n";
        std::fs::write(&tmp, content).unwrap();

        let result = update_node_status(&tmp, "optimize", "optimize", true, "inprogress");
        let written = std::fs::read_to_string(&tmp).unwrap();
        std::fs::remove_file(&tmp).ok();

        assert!(result.is_ok(), "update_node_status should succeed for dir with slash; err: {result:?}");
        assert!(
            written.contains("| `optimize/` | 📁 Directory | 🔄 In Progress |"),
            "Nodes table row should be updated; got:\n{written}"
        );
    }
}

// ── path utilities ─────────────────────────────────────────────────────────

/// Given a node path, return (parent_dir, readme_path, node_id, fs_name, is_dir).
pub fn parse_node_path(node_path: &Path) -> Result<(PathBuf, PathBuf, String, String, bool)> {
    let is_dir = node_path.is_dir()
        || node_path
            .to_str()
            .map(|s| s.ends_with('/'))
            .unwrap_or(false);
    let node_path = if is_dir {
        // strip trailing slash for basename extraction
        PathBuf::from(
            node_path
                .to_str()
                .unwrap_or("")
                .trim_end_matches('/'),
        )
    } else {
        node_path.to_path_buf()
    };

    let parent = node_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("node path has no parent: {}", node_path.display()))?
        .to_path_buf();
    let readme = parent.join("README.md");

    let basename = node_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("cannot determine basename of {}", node_path.display()))?;

    let (node_id, fs_name) = if is_dir {
        (basename.to_string(), basename.to_string())
    } else {
        let stem = node_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("cannot determine stem of {}", node_path.display()))?;
        (stem.to_string(), basename.to_string())
    };

    validate_node_name(&node_id)?;
    Ok((parent, readme, node_id, fs_name, is_dir))
}
