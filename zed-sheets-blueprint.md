# Zed Sheets — Extension Blueprint

> A TSV-native, Nushell-powered spreadsheet LSP for Zed.
> Stack: `.tsv` for data, `.nu` for logic, LSP as the intelligence layer.

---

## Repository Structure

```
zed-sheets/
├── extension.toml          # Zed extension manifest
├── Cargo.toml              # Rust workspace
├── src/
│   └── lib.rs              # Zed extension entry point (compiles to WASM)
├── zed-sheets-lsp/         # The actual LSP binary (runs outside WASM)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── document.rs     # TSV parsing & in-memory grid model
│       ├── sidecar.rs      # .zedsheets.json sidecar (column types, nu bindings)
│       ├── dag.rs          # Dependency graph across TSV+nu
│       ├── diagnostics.rs  # Validation & error reporting
│       └── completions.rs  # Autocomplete for nu pipelines & column refs
├── languages/
│   └── tsv/
│       ├── config.toml
│       └── highlights.scm  # Basic column/header syntax highlighting
└── tests/
    └── fixtures/           # Sample .tsv + .zedsheets.json pairs
```

---

## extension.toml

```toml
id = "zed-sheets"
name = "Zed Sheets"
description = "TSV-native spreadsheet LSP with Nushell scripting"
version = "0.1.0"
schema_version = 1
authors = ["you"]
repository = "https://github.com/you/zed-sheets"

[grammars.tsv]
repository = "https://github.com/nickel-lang/tree-sitter-tsv"  # or write a minimal one
rev = "COMMIT_SHA"

[language_servers.zed-sheets-lsp]
name = "Zed Sheets LSP"
languages = ["TSV"]
```

---

## languages/tsv/config.toml

```toml
name = "TSV"
grammar = "tsv"
path_suffixes = ["tsv"]
tab_size = 4
hard_tabs = true
line_comments = ["#"]
```

---

## src/lib.rs (Zed Extension — compiles to WASM)

```rust
use zed_extension_api::{self as zed, LanguageServerId, Worktree, Result, Command};

struct ZedSheetsExtension;

impl zed::Extension for ZedSheetsExtension {
    fn new() -> Self {
        ZedSheetsExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> Result<Command> {
        // Download or locate the zed-sheets-lsp binary
        let path = self.get_or_download_lsp_binary()?;
        Ok(Command {
            command: path,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(ZedSheetsExtension);
```

---

## Sidecar Format: `.zedsheets.json`

Lives alongside `data.tsv` as `data.zedsheets.json`. Keeps the TSV pure.

```json
{
  "version": 1,
  "columns": {
    "revenue": { "type": "number", "unit": "USD" },
    "region":  { "type": "string" },
    "margin":  {
      "type": "derived",
      "nu_expr": "$row.revenue - $row.cost"
    },
    "total":   {
      "type": "derived",
      "nu_expr": "$data | get revenue | math sum"
    }
  },
  "named_ranges": {
    "q1_data": { "rows": [1, 90] }
  }
}
```

---

## LSP Core: document.rs

```rust
pub struct Grid {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Grid {
    pub fn parse_tsv(content: &str) -> Self {
        let mut lines = content.lines();
        let headers = lines
            .next()
            .unwrap_or("")
            .split('\t')
            .map(String::from)
            .collect();
        let rows = lines
            .map(|l| l.split('\t').map(String::from).collect())
            .collect();
        Grid { headers, rows }
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == name)
    }
}
```

---

## LSP Core: dag.rs

Tracks which derived columns depend on which source columns. Used for:
- Circular dependency detection → LSP error diagnostic
- Propagating hover info ("this column is used by: margin, total")
- Rename refactoring (update all `$row.old_name` → `$row.new_name`)

```rust
use std::collections::{HashMap, HashSet};

pub struct DependencyGraph {
    /// derived_col → set of source cols it references
    pub edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn has_cycle(&self) -> bool {
        // DFS cycle detection
        // ...
    }

    pub fn dependents_of(&self, col: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter_map(|(k, deps)| if deps.contains(col) { Some(k.clone()) } else { None })
            .collect()
    }
}
```

---

## LSP Capabilities (What Zed Gets)

| LSP Feature | What It Does in Zed Sheets |
|---|---|
| `textDocument/hover` | Column name → type, unit, nu expression, dependents |
| `textDocument/completion` | Inside `.nu` sidecar expressions: column names, nu builtins |
| `textDocument/diagnostic` | Circular deps, type mismatches, missing columns |
| `textDocument/rename` | Rename a header → updates all `$row.name` references in sidecar |
| `textDocument/definition` | Jump from derived column → its nu expression in sidecar |
| `workspace/symbol` | List all columns, named ranges across the workspace |

---

## Nushell as the Formula Layer

Instead of inventing formula syntax, `derived` columns are just nu pipelines. Examples:

```nu
# Single-row derived value
$row.revenue * $row.quantity

# Aggregate (whole column)
$data | get price | math sum

# Conditional
if $row.status == "active" { $row.value } else { 0 }

# String ops
$row.first_name + " " + $row.last_name
```

The LSP evaluates these via `nu --stdin` for diagnostics, or shells out to validate syntax without running. For hover, it can show the inferred output type.

---

## MVP Build Order

**Week 1 — Parse & Serve**
- TSV parser → `Grid` struct
- Sidecar loader → column metadata
- Minimal LSP over stdio (tower-lsp crate)
- Register with Zed extension, open a TSV, see it start

**Week 2 — Hover + Diagnostics**
- Hover on column header → type, nu_expr if derived
- Validate nu_expr syntax via `nu --parse-only`
- Cycle detection in DAG → error diagnostic

**Week 3 — Completions + Rename**
- Column name completions inside nu expressions
- Rename header → updates sidecar references
- Named range support in sidecar

**Week 4 — Polish**
- Download/install LSP binary via extension (GitHub releases)
- `textDocument/definition` (TSV header ↔ sidecar entry)
- Basic test suite with fixture files

---

## Key Crates

```toml
[dependencies]
tower-lsp = "0.20"       # LSP server framework
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# For DAG
petgraph = "0.6"
```

---

## What You're NOT Doing (yet)

- No custom file format — TSV stays TSV
- No GUI / canvas — that's a later layer
- No pivot tables — nu handles reshaping natively
- No Excel compat — not the goal

The sidecar + nu approach means the whole thing is scriptable, composable, and stays in the developer's mental model. You can `nu script.nu` your sheet from the terminal, pipe it through other tools, diff the TSV in git — it's just files.
