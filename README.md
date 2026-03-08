# Zed Sheets

Zed-facing language tooling for `nustage`.

This repository is not the core spreadsheet or Power Query replacement. It is the thin integration layer that lets Zed understand tabular source files and, eventually, `nustage` sidecars and pipeline state.

## Positioning

`nustage` owns the product model:

- source loading for grid-shaped data
- canonical step pipeline
- sidecar persistence
- schema history
- export/transpilation helpers

`zed-sheet-lsp` owns the editor integration:

- language registration for `.tsv`
- LSP lifecycle inside Zed
- hover, completion, and diagnostics
- future pipeline-aware code actions, rename, symbols, and preview hooks

If `nustage` is the stage, this repo is the adapter that lets Zed witness and manipulate it.

## What Works Today

This repo is currently a narrow TSV-focused spike.

- Zed extension manifest and language wiring in [extension.toml](./extension.toml) and [languages/tsv/config.toml](./languages/tsv/config.toml)
- Extension bootstrap that launches `zed-sheets-lsp` from `PATH` or downloads a published release asset in [src/lib.rs](./src/lib.rs)
- LSP server with:
  - full document sync
  - cell/header hover
  - column completion
  - basic diagnostics for missing referenced columns and circular references
  in [zed-sheets-lsp/src/document.rs](./zed-sheets-lsp/src/document.rs)

## What Does Not Exist Yet

The repo should be evaluated as an adapter spike, not a finished product.

- No in-Zed grid preview
- No live `nustage` sidecar integration
- No pipeline execution or schema inference from `nustage`
- No real `rename`, `definition`, or `workspace/symbol` support in the active server
- No meaningful automated test coverage yet

The files under `demo/` are still useful for pitching the witness-layer idea, but they are not an actual Zed integration surface.

## Architecture Direction

Target split:

1. `nustage`
   - canonical sidecar: `.nustage.json`
   - transformation step model
   - schema snapshots and drift detection
   - execution and preview semantics
   - optional export targets like Power Query M

2. `zed-sheet-lsp`
   - file association and startup in Zed
   - diagnostics against source data and sidecar definitions
   - editing affordances for pipeline authoring
   - preview or witness-pane hooks when Zed exposes enough API surface

That boundary is described in [docs/NUSTAGE_INTEGRATION.md](./docs/NUSTAGE_INTEGRATION.md).
Cross-repo placement rules are in [docs/STACK_BOUNDARIES.md](./docs/STACK_BOUNDARIES.md).

## Current Assessment

This repository makes sense if you read it as:

"Can Zed act as a good editor and witness surface for `nustage` pipelines over TSV and similar tabular data?"

It makes much less sense if you read it as:

"Is this already a spreadsheet product inside Zed?"

The first is plausible. The second is still mostly pitch material.

## Development

Build the workspace:

```bash
cargo build --workspace
```

Run the LSP directly:

```bash
cargo run --package zed-sheets-lsp
```

Run tests:

```bash
cargo test --workspace
```

Note: the workspace currently compiles, but `cargo test --workspace` exercises effectively zero real tests. See [docs/NUSTAGE_REUSE_AUDIT.md](./docs/NUSTAGE_REUSE_AUDIT.md) for the current gaps and risks.
