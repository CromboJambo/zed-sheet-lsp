# Zed Sheets — Extension Blueprint

> A TSV-native, Nushell-powered spreadsheet LSP for Zed.  
> Stack: `.tsv` for data, `.nu` for logic, LSP as the intelligence layer.

---

## 🎥 Demo: Preview Toggle Feature

This demo "fakes" the preview toggle (eyeball icon) feature using tmux + tabiew to demonstrate what Zed Sheets LSP already provides. It's a proof-of-concept showing that the preview API all that's missing.

### Quick Demo

```bash
# Run the demo (requires: tabiew, nu, asciinema)
./demo/demo-record.sh demo/assets/sample.tsv
```

This creates a tmux layout with:
- **Left pane**: tabiew grid preview (like the eyeball preview tab)
- **Right pane**: Raw TSV with LSP hover/completions working
- **Bottom pane**: Nu REPL for filtering and transforming data

### What the Demo Shows

1. **Hover on TSV cells** - Shows column name + type (already works in Zed LSP)
2. **Grid preview** - tabiew renders TSV as a spreadsheet
3. **Same file, two views** - Raw TSV on right, grid on left
4. **Edit propagation** - Changes in raw TSV update the grid
5. **Completions** - Tab completion for column references
6. **Nu integration** - Filter/transform data in a REPL pane

### Demo Walkthrough

See the [`demo/`](demo/) folder for:
- [`demo-record.sh`](demo/demo-record.sh) - Complete demo recording script
- [`demo-standalone.sh`](demo/demo-standalone.sh) - Demo without Zed installed
- [`tsv-preview-session.sh`](demo/tsv-preview-session.sh) - Simple tmux session launcher
- [`tsv-preview-session.nu`](demo/tsv-preview-session.nu) - Nushell version

### Why This Demo Matters

The demo proves that Zed Sheets LSP already provides:
- ✅ Hover on TSV cells (column name + type)
- ✅ Completions for column references
- ✅ Diagnostics for invalid data
- ✅ Grid view via tabiew
- ✅ Nu integration for filtering/transforming

All that's missing is the **preview API** to expose this functionality in Zed. The demo video (asciinema recording) shows exactly what the preview toggle would unlock.

---

## Overview

This project implements a Zed extension that provides spreadsheet functionality using:
- TSV files for data storage
- Nushell scripts for column formulas
- LSP for intelligence and diagnostics

The extension is built with a layered architecture:
1. **TSV parsing** - handles data structure
2. **Sidecar metadata** - stores column types and formulas  
3. **Dependency graph** - tracks formula dependencies
4. **LSP server** - provides IDE features like hover, diagnostics, completions

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
├── demo/                   # Demo infrastructure for preview toggle
│   ├── assets/
│   │   └── sample.tsv      # Sample TSV data file
│   ├── demo.sh             # Full demo with Zed + tabiew
│   ├── demo-standalone.sh  # Demo without Zed (for recording)
│   ├── demo-record.sh      # Demo recording script
│   ├── tsv-preview-session.sh  # Simple tmux session launcher
│   ├── tsv-preview-session.nu    # Nushell version
│   └── README.md           # Demo documentation
└── tests/
    └── fixtures/           # Sample .tsv + .zedsheets.json pairs
```

---

## Features Implemented

### LSP Capabilities (What Zed Gets)

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

## Testing

Sample test fixtures included:
- `tests/fixtures/sample.tsv` and `tests/fixtures/sample.tsv.zedsheets.json`
- `tests/fixtures/test_basic.tsv` and `tests/fixtures/test_basic.tsv.zedsheets.json`
- `tests/fixtures/circular_test.tsv` and `tests/fixtures/circular_test.tsv.zedsheets.json`

These files demonstrate various use cases including circular dependencies for diagnostics testing.

---

## Build

```bash
# Build the workspace
cargo build --workspace

# Or use the build script
./build.sh
```

---

## Development

### Run the LSP Server

```bash
cargo run --package zed-sheets-lsp
```

### Run Tests

```bash
cargo test
```

---

## License

This project is licensed under the MIT License. See the main repo for details.

---

**Demo Link:** [asciinema recording](TBD)  
**GitHub Repo:** https://github.com/CromboJambo/zed-sheet-lsp