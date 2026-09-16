# dirtree-rdm CLI Reference

## Contents

- [Invocation](#invocation)
- [init — Bootstrap a new campaign](#init--bootstrap-a-new-campaign)
- [add — Create a new node](#add--create-a-new-node)
- [status — Read current status (read-only)](#status--read-a-nodes-current-status-read-only)
- [update — Change a node's status](#update--change-a-nodes-status)
- [move — Move a node to a different parent](#move--move-a-node-to-a-different-parent)
- [insert — Create an intermediate directory](#insert--create-an-intermediate-directory-wrapping-one-node)
- [validate — Check against BNF grammar (read-only)](#validate--check-a-readmemd-against-the-bnf-grammar-read-only)
- [ls — List nodes (read-only)](#ls--list-nodes-in-a-directory-read-only)
- [grammar — Print the BNF grammar source (read-only)](#grammar--print-the-bnf-grammar-source-read-only)
- [Error Handling](#error-handling)
- [BNF Grammar](#bnf-grammar)

---

`dirtree-rdm` is a compiled Rust CLI that provides safe CRUD operations for dirtree roadmap nodes. Every mutating command:

1. **Pre-flight validates** the affected README.md against the BNF grammar — aborts if invalid
2. **Writes to a temp file**, then post-flight validates the result
3. **Atomically commits** the temp file only if validation passes

Use it instead of manually editing README.md files to avoid corrupting Mermaid graphs or Nodes tables.

---

## Invocation

```bash
bash skills/managing-roadmaps/scripts/dirtree-rdm.sh <command> [args]
```

The wrapper auto-detects the platform and dispatches to the correct pre-compiled binary.

---

## Commands

### `init` — Bootstrap a new campaign

```bash
dirtree-rdm.sh init <campaign-path>
```

Creates `<campaign-path>/` and a BNF-valid template `README.md` inside it. Unlike `add`, this command does **not** require a parent `README.md` — it is the correct entry point for starting a new campaign.

- `<campaign-path>`: path to the new campaign directory (e.g. `__roadmap__/my-campaign`)
- Campaign name must match `^[a-z][a-z0-9_-]*$`
- Parent directory must exist (typically `__roadmap__/`), but must NOT have a `README.md`
- After `init`, fill in the prose sections of the generated `README.md` (Context, Goal, Pre-conditions, Success Gates), then use `add` to create child nodes

**Example:**
```bash
dirtree-rdm.sh init __roadmap__/oauth-support
dirtree-rdm.sh init __roadmap__/fix-threading-deadlock
```

---

### `add` — Create a new node

```bash
dirtree-rdm.sh add <node-path> [--type leaf|dir] [--title "Human Title"]
```

Creates a new leaf task `.md` or directory node and updates the parent `README.md`.

- `--type leaf` (default): creates `<node-path>.md` with leaf task template
- `--type dir`: creates `<node-path>/` with `README.md` scaffold
- Parent directory must already exist and contain a `README.md`
- Node name must match `^[a-z][a-z0-9_-]*$` (no numeric prefixes)
- New node is added with status `:::planned`

**Example:**
```bash
dirtree-rdm.sh add __roadmap__/oauth/setup/database_migration.md --title "Database Migration"
dirtree-rdm.sh add __roadmap__/oauth/providers --type dir --title "OAuth Providers"
```

---

### `status` — Read a node's current status (read-only)

```bash
dirtree-rdm.sh status <node-path>
```

Prints the node's current status from its parent `README.md`. Does not modify any files.

**Example:**
```bash
dirtree-rdm.sh status __roadmap__/oauth/setup/database_migration.md
# → database_migration  planned
```

---

### `update` — Change a node's status

```bash
dirtree-rdm.sh update <node-path> <status>
```

Updates the node's `:::status` in the Mermaid graph and the status emoji in the Nodes table of the parent `README.md`.

Valid status values: `planned` `inprogress` `done` `amendment` `blocked`

**Example:**
```bash
dirtree-rdm.sh update __roadmap__/oauth/setup/database_migration.md inprogress
dirtree-rdm.sh update __roadmap__/oauth/setup/database_migration.md done
```

**Note:** Marking a node `blocked` is the correct way to "delete" it — never physically remove roadmap nodes.

---

### `move` — Move a node to a different parent

```bash
dirtree-rdm.sh move <source-path> <dest-parent-path>
```

Moves a node (leaf or directory) to a new parent directory, updating both the source and destination `README.md` files. **Status is preserved.**

Both source parent and destination parent must have a valid `README.md`.

**Example:**
```bash
# Move github.md from setup/ into providers/
dirtree-rdm.sh move __roadmap__/oauth/setup/github.md __roadmap__/oauth/providers/

# Move siblings one-by-one when restructuring
dirtree-rdm.sh move __roadmap__/oauth/billing.md __roadmap__/oauth/payments/
dirtree-rdm.sh move __roadmap__/oauth/invoices.md __roadmap__/oauth/payments/
```

---

### `insert` — Create an intermediate directory wrapping one node

```bash
dirtree-rdm.sh insert <new-dir-path> --wraps <child-path> [--title "Human Title"]
```

Creates a new intermediate directory and moves exactly one existing sibling into it as its first child. Updates both the grandparent and new directory `README.md` files.

To wrap additional siblings into the new directory, use `move` after `insert`.

**Example:**
```bash
# Before: oauth/github.md  oauth/google.md  (siblings)
# Goal:   oauth/providers/github.md  oauth/providers/google.md

dirtree-rdm.sh insert __roadmap__/oauth/providers --wraps __roadmap__/oauth/github.md --title "OAuth Providers"
dirtree-rdm.sh move __roadmap__/oauth/google.md __roadmap__/oauth/providers/
```

---

### `validate` — Check a README.md against the BNF grammar (read-only)

```bash
dirtree-rdm.sh validate <dir-path-or-file>
```

Validates a `README.md` (pass a directory to validate its `README.md`) or leaf task `.md` against the BNF grammar. Reports violations with line numbers and production names. Exits 0 if clean, 1 if violations found.

Run this first when a README.md may have been hand-edited or corrupted.

**Example:**
```bash
dirtree-rdm.sh validate __roadmap__/oauth/
dirtree-rdm.sh validate __roadmap__/oauth/setup/database_migration.md
```

---

### `ls` — List nodes in a directory (read-only)

```bash
dirtree-rdm.sh ls [dir-path]
```

Prints the Nodes table from a directory's `README.md` in a compact format.

**Example:**
```bash
dirtree-rdm.sh ls __roadmap__/oauth/
# Node               Type         Status
# -----------------------------------------
# database_migration  📄 Leaf Task  ✅ Done
# providers           📁 Directory  🔄 In Progress
```

---

### `grammar` — Print the BNF grammar source (read-only)

```bash
dirtree-rdm.sh grammar [readme|leaf]
```

Prints the full BNF grammar for either document type to stdout. Defaults to `readme` when no argument is given.

Use this when a `validate` error names a production you don't recognise — pipe it through `grep` to find the exact production rule:

```bash
dirtree-rdm.sh grammar readme | grep "mermaid-classdef"
dirtree-rdm.sh grammar leaf   | grep "step-field"
```

**Example:**
```bash
dirtree-rdm.sh grammar readme   # full README.md grammar
dirtree-rdm.sh grammar leaf     # full leaf task grammar
```

---

## Error Handling

When a mutating command encounters an invalid README.md (pre-flight failure), it aborts with a structured error:

```
error: pre-flight validation failed for __roadmap__/oauth/README.md:
  line 18: [mermaid-classdef-done] expected `classDef done` classDef line, got: "    classDef done fill:#000,color:#fff"
Fix the README.md manually or run `dirtree-rdm validate <dir>` for details.
```

Fix the reported violation, then retry the command.

---

## BNF Grammar

See [dirtree-bnf.md](dirtree-bnf.md) for the complete grammar specification covering all valid README.md and leaf task structures.
