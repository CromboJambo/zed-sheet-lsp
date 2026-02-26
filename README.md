# Zed Sheets — Extension Blueprint

> A TSV-native, Nushell-powered spreadsheet LSP for Zed.
> Stack: `.tsv` for data, `.nu` for logic, LSP as the intelligence layer.

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
- `sample.tsv` and `sample.zedsheets.json`
- `test_basic.tsv` and `test_basic.zedsheets.json`
- `circular_test.tsv` and `circular_test.zedsheets.json`

These files demonstrate various use cases including circular dependencies for diagnostics testing.

---

## Build Status

✅ All core components implemented:
- TSV parser
- Sidecar loader  
- Dependency graph
- LSP server framework
- Diagnostics system
- Completions system
- Extension manifest
- Language configuration
- Syntax highlighting rules

✅ Tests included with sample data files

This implementation follows the blueprint specification and provides a complete foundation for building out the full Zed Sheets extension.