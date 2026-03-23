mod grammar;
mod readme;
mod templates;
mod validate;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

// ── CLI surface ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "dirtree-rdm",
    about = "CRUD operations for dirtree roadmap nodes with BNF-validated README.md mutations",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new leaf task (.md) or directory node and update the parent README.md
    Add {
        /// Path to the new node (leaf: path/to/name.md, dir: path/to/name/ or path/to/name)
        node_path: PathBuf,
        /// Node type: leaf (default) or dir
        #[arg(long, default_value = "leaf")]
        r#type: String,
        /// Human-readable title for the node (used in Mermaid label and file scaffold)
        #[arg(long, default_value = "")]
        title: String,
    },
    /// (Read-only) Print the current status of a node from its parent README.md
    Status {
        /// Path to the node (leaf .md or directory)
        node_path: PathBuf,
    },
    /// Update a node's status in the parent README.md (Mermaid + Nodes table)
    Update {
        /// Path to the node (leaf .md or directory)
        node_path: PathBuf,
        /// New status: planned | inprogress | done | amendment | blocked
        status: String,
    },
    /// Move a node to a new parent directory (preserves status)
    Move {
        /// Path to the source node
        source_path: PathBuf,
        /// Path to the destination parent directory
        dest_parent: PathBuf,
    },
    /// Create a new intermediate directory that wraps exactly one existing node
    Insert {
        /// Path for the new intermediate directory
        new_dir_path: PathBuf,
        /// Path to the existing node that becomes the child
        #[arg(long)]
        wraps: PathBuf,
        /// Human-readable title for the new directory
        #[arg(long, default_value = "")]
        title: String,
    },
    /// (Read-only) Validate README.md (or leaf .md) against BNF grammar
    Validate {
        /// Path to a directory (validates its README.md) or a .md file
        path: PathBuf,
    },
    /// (Read-only) Print the Nodes table from a directory's README.md
    Ls {
        /// Directory to list (defaults to current directory)
        #[arg(default_value = ".")]
        dir_path: PathBuf,
    },
}

// ── main ───────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Add { node_path, r#type, title } => cmd_add(&node_path, &r#type, &title),
        Command::Status { node_path }              => cmd_status(&node_path),
        Command::Update { node_path, status }      => cmd_update(&node_path, &status),
        Command::Move { source_path, dest_parent } => cmd_move(&source_path, &dest_parent),
        Command::Insert { new_dir_path, wraps, title } => cmd_insert(&new_dir_path, &wraps, &title),
        Command::Validate { path }                 => cmd_validate(&path),
        Command::Ls { dir_path }                   => cmd_ls(&dir_path),
    }
}

// ── add ────────────────────────────────────────────────────────────────────

fn cmd_add(node_path: &Path, node_type: &str, title: &str) -> Result<()> {
    let is_dir = node_type == "dir"
        || node_path.to_str().map(|s| s.ends_with('/')).unwrap_or(false);
    let node_path_clean = PathBuf::from(
        node_path.to_str().unwrap_or("").trim_end_matches('/'),
    );

    let (parent, readme, node_id, fs_name, _) =
        readme::parse_node_path(&node_path_clean)?;

    if !parent.exists() {
        bail!("parent directory does not exist: {}", parent.display());
    }
    if !readme.exists() {
        bail!(
            "parent README.md not found at {}; parent must be an existing roadmap directory",
            readme.display()
        );
    }
    if node_path_clean.exists() {
        bail!("node already exists: {}", node_path_clean.display());
    }

    let effective_title = if title.is_empty() { &node_id } else { title };

    if is_dir {
        std::fs::create_dir_all(&node_path_clean)
            .with_context(|| format!("creating directory {}", node_path_clean.display()))?;
        let child_readme = node_path_clean.join("README.md");
        std::fs::write(&child_readme, templates::readme_template(effective_title))
            .with_context(|| format!("writing {}", child_readme.display()))?;
        println!("created  {}/", node_path_clean.display());
    } else {
        let leaf_path = if node_path_clean.extension().is_none() {
            node_path_clean.with_extension("md")
        } else {
            node_path_clean.clone()
        };
        std::fs::write(&leaf_path, templates::leaf_template(effective_title))
            .with_context(|| format!("writing {}", leaf_path.display()))?;
        println!("created  {}", leaf_path.display());
    }

    readme::add_node(&readme, &node_id, &fs_name, is_dir, effective_title)?;
    println!("updated  {}", readme.display());
    Ok(())
}

// ── status (read-only) ─────────────────────────────────────────────────────

fn cmd_status(node_path: &Path) -> Result<()> {
    let (_, readme, node_id, _, _) = readme::parse_node_path(node_path)?;
    if !readme.exists() {
        bail!("README.md not found at {}", readme.display());
    }
    let status = readme::read_node_status(&readme, &node_id)?;
    println!("{node_id}  {status}");
    Ok(())
}

// ── update ─────────────────────────────────────────────────────────────────

fn cmd_update(node_path: &Path, status: &str) -> Result<()> {
    readme::validate_status(status)?;
    let (_, readme, node_id, fs_name, _) = readme::parse_node_path(node_path)?;
    if !readme.exists() {
        bail!("README.md not found at {}", readme.display());
    }
    readme::update_node_status(&readme, &node_id, &fs_name, status)?;
    println!("updated  {} → {status}", node_path.display());
    Ok(())
}

// ── move ───────────────────────────────────────────────────────────────────

fn cmd_move(source_path: &Path, dest_parent: &Path) -> Result<()> {
    if !dest_parent.is_dir() {
        bail!("destination parent is not a directory: {}", dest_parent.display());
    }
    let dest_readme = dest_parent.join("README.md");
    if !dest_readme.exists() {
        bail!(
            "destination parent has no README.md: {}",
            dest_readme.display()
        );
    }

    let source_clean = PathBuf::from(
        source_path.to_str().unwrap_or("").trim_end_matches('/'),
    );
    let (_, src_readme, node_id, fs_name, is_dir) =
        readme::parse_node_path(&source_clean)?;

    if !src_readme.exists() {
        bail!("source README.md not found at {}", src_readme.display());
    }

    // Check dest doesn't already have this node
    let dest_node = dest_parent.join(&fs_name);
    if dest_node.exists() {
        bail!("destination already contains a node named {:?}", fs_name);
    }

    // Capture title from Mermaid before removal
    let src_content = std::fs::read_to_string(&src_readme)?;
    let title = extract_mermaid_title(&src_content, &node_id)
        .unwrap_or_else(|| node_id.clone());

    // Remove from source README (captures status)
    let current_status = readme::remove_node(&src_readme, &node_id, &fs_name)?;
    println!("removed  {node_id} from {}", src_readme.display());

    // Move filesystem entry
    std::fs::rename(&source_clean, &dest_node)
        .with_context(|| format!("moving {} → {}", source_clean.display(), dest_node.display()))?;
    println!("moved    {} → {}", source_clean.display(), dest_node.display());

    // Add to destination README with preserved status
    readme::add_node(&dest_readme, &node_id, &fs_name, is_dir, &title)?;
    // Set preserved status (add_node sets :::planned by default)
    if current_status != "planned" {
        readme::update_node_status(&dest_readme, &node_id, &fs_name, &current_status)?;
    }
    println!("updated  {} (status: {current_status})", dest_readme.display());
    Ok(())
}

fn extract_mermaid_title(content: &str, node_id: &str) -> Option<String> {
    let prefix = format!("    {node_id}[");
    for line in content.lines() {
        if line.starts_with(&prefix) {
            let inner = line.trim_start_matches(&*prefix);
            if let Some(end) = inner.find(']') {
                return Some(inner[..end].to_string());
            }
        }
    }
    None
}

// ── insert ─────────────────────────────────────────────────────────────────

fn cmd_insert(new_dir_path: &Path, wraps: &Path, title: &str) -> Result<()> {
    let new_dir_clean = PathBuf::from(
        new_dir_path.to_str().unwrap_or("").trim_end_matches('/'),
    );
    let wraps_clean = PathBuf::from(
        wraps.to_str().unwrap_or("").trim_end_matches('/'),
    );

    if new_dir_clean.exists() {
        bail!("new directory already exists: {}", new_dir_clean.display());
    }
    if !wraps_clean.exists() {
        bail!("node to wrap does not exist: {}", wraps_clean.display());
    }

    // new_dir and wraps must share the same parent (grandparent README)
    let new_parent = new_dir_clean
        .parent()
        .ok_or_else(|| anyhow::anyhow!("new dir path has no parent"))?;
    let wrap_parent = wraps_clean
        .parent()
        .ok_or_else(|| anyhow::anyhow!("wraps path has no parent"))?;

    let canonical_new_parent = new_parent.canonicalize().unwrap_or_else(|_| new_parent.to_path_buf());
    let canonical_wrap_parent = wrap_parent.canonicalize().unwrap_or_else(|_| wrap_parent.to_path_buf());

    if canonical_new_parent != canonical_wrap_parent {
        bail!(
            "new directory and wrapped node must share the same parent.\n  new_dir parent: {}\n  wraps parent:   {}",
            new_parent.display(),
            wrap_parent.display()
        );
    }

    let grandparent_readme = new_parent.join("README.md");
    if !grandparent_readme.exists() {
        bail!(
            "grandparent README.md not found at {}",
            grandparent_readme.display()
        );
    }

    let (_, _, wrap_node_id, wrap_fs_name, wrap_is_dir) =
        readme::parse_node_path(&wraps_clean)?;
    let (_, _, new_node_id, new_fs_name, _) =
        readme::parse_node_path(&new_dir_clean)?;

    let effective_title = if title.is_empty() { &new_node_id } else { title };

    // Get wrapped node's current status and title
    let grandparent_content = std::fs::read_to_string(&grandparent_readme)?;
    let wrap_title = extract_mermaid_title(&grandparent_content, &wrap_node_id)
        .unwrap_or_else(|| wrap_node_id.clone());
    let wrap_status = readme::read_node_status(&grandparent_readme, &wrap_node_id)?;

    // Remove wrapped node from grandparent README
    readme::remove_node(&grandparent_readme, &wrap_node_id, &wrap_fs_name)?;
    println!("removed  {wrap_node_id} from {}", grandparent_readme.display());

    // Create new intermediate directory
    std::fs::create_dir_all(&new_dir_clean)
        .with_context(|| format!("creating {}", new_dir_clean.display()))?;
    println!("created  {}/", new_dir_clean.display());

    // Move wrapped node into new directory
    let new_wrap_dest = new_dir_clean.join(&wrap_fs_name);
    std::fs::rename(&wraps_clean, &new_wrap_dest)
        .with_context(|| format!("moving {} → {}", wraps_clean.display(), new_wrap_dest.display()))?;
    println!("moved    {} → {}", wraps_clean.display(), new_wrap_dest.display());

    // Write new directory's README.md (child is the wrapped node)
    let new_readme = new_dir_clean.join("README.md");
    std::fs::write(&new_readme, templates::readme_template(effective_title))?;

    // Add wrapped node to new README
    readme::add_node(&new_readme, &wrap_node_id, &wrap_fs_name, wrap_is_dir, &wrap_title)?;
    if wrap_status != "planned" {
        readme::update_node_status(&new_readme, &wrap_node_id, &wrap_fs_name, &wrap_status)?;
    }
    println!("updated  {} (child: {wrap_node_id})", new_readme.display());

    // Add new dir to grandparent README
    readme::add_node(&grandparent_readme, &new_node_id, &new_fs_name, true, effective_title)?;
    println!("updated  {} (new node: {new_node_id})", grandparent_readme.display());

    Ok(())
}

// ── validate ───────────────────────────────────────────────────────────────

fn cmd_validate(path: &Path) -> Result<()> {
    let target = if path.is_dir() {
        path.join("README.md")
    } else {
        path.to_path_buf()
    };

    if !target.exists() {
        bail!("file not found: {}", target.display());
    }

    let clean = validate::validate_and_report(&target)?;
    if !clean {
        std::process::exit(1);
    }
    Ok(())
}

// ── ls ─────────────────────────────────────────────────────────────────────

fn cmd_ls(dir_path: &Path) -> Result<()> {
    let readme = if dir_path.is_dir() {
        dir_path.join("README.md")
    } else {
        dir_path.to_path_buf()
    };

    if !readme.exists() {
        bail!("README.md not found at {}", readme.display());
    }

    let rows = readme::read_nodes_table(&readme)?;
    if rows.is_empty() {
        println!("(no nodes)");
        return Ok(());
    }

    let name_w = rows.iter().map(|r| r.0.len()).max().unwrap_or(4).max(4);
    let type_w = rows.iter().map(|r| r.1.chars().count()).max().unwrap_or(4).max(4);

    println!(
        "{:<name_w$}  {:<type_w$}  Status",
        "Node", "Type",
        name_w = name_w,
        type_w = type_w,
    );
    println!("{}", "-".repeat(name_w + type_w + 20));
    for (name, typ, status) in &rows {
        println!(
            "{:<name_w$}  {:<type_w$}  {status}",
            name, typ,
            name_w = name_w,
            type_w = type_w,
        );
    }
    Ok(())
}
