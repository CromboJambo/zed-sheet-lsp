# Nustage Integration Boundary

This document defines the intended boundary between `nustage` and `zed-sheet-lsp`.

## Goal

Use Zed as the editor and witness surface for `nustage` pipelines without reimplementing the pipeline model, sidecar format, or execution engine in this repository.

## Ownership

### `nustage` owns

- canonical pipeline types:
  - `TransformationStep`
  - `StepType`
  - `TransformationPipeline`
- sidecar format:
  - `SidecarFile`
  - `SidecarMetadata`
  - `.nustage.json` path conventions
- schema snapshots and drift reporting
- source loading for CSV, Parquet, and Excel
- transformation execution
- optional export/transpilation helpers like Power Query M generation

### `zed-sheet-lsp` owns

- Zed extension manifest and installation
- language registration for source file types
- LSP server process management
- text-document intelligence
- editor UX:
  - hover
  - completion
  - diagnostics
  - code actions
  - symbols
  - rename
- future preview-panel hooks if Zed exposes them

## Recommended Data Flow

1. User opens `sales.tsv` or `sales.csv` in Zed.
2. `zed-sheet-lsp` loads source text and looks for the matching `nustage` sidecar.
3. `zed-sheet-lsp` uses `nustage` types to parse the sidecar and understand the pipeline.
4. Diagnostics are produced from canonical pipeline validation, not from ad hoc regex parsing in the LSP.
5. Hover and completion are driven by schema snapshots and step metadata from `nustage`.
6. Preview and witness panes, if added later, request rendered state from `nustage`-owned execution APIs.

## Concrete Near-Term Contract

The first useful contract is library-level, not RPC-level.

`zed-sheet-lsp` should depend on a small stable slice of `nustage`:

- `sidecar::SidecarFile`
- `transformations::{TransformationStep, StepType, ColumnSchema}`
- helpers for sidecar-path resolution
- schema diff helpers

This is enough to implement:

- sidecar-aware diagnostics
- pipeline step hover
- column and step-name completion
- workspace symbol indexing over source files plus sidecars

## What This Repo Should Stop Owning

These parts should not become separate local models here:

- custom sidecar schema like `.zedsheets.json`
- local formula syntax tied to `$row.column`
- local dependency graph format separate from canonical pipeline steps
- preview semantics that diverge from `nustage`

Those are the sources of drift.

## Migration Path

### Phase 1

Reposition the repo as a TSV/tabular LSP adapter for `nustage`.

- keep current TSV hover and completion
- de-emphasize spreadsheet claims
- document the thin-adapter architecture

### Phase 2

Replace local sidecar handling with canonical `nustage` sidecar loading.

- read `.nustage.json`
- surface schema and pipeline metadata in hover
- diagnose missing columns and invalid step references from canonical steps

### Phase 3

Add pipeline-aware editor affordances.

- rename column across sidecar steps
- definition from source columns to pipeline steps and back
- symbols for steps, derived columns, and named outputs
- code actions for common step edits

### Phase 4

Add witness-layer integration when Zed supports enough preview surface.

- rendered table preview
- step-by-step preview state
- schema diff or row-count summary

## Design Rule

`zed-sheet-lsp` should be thin enough that if `nustage` evolves its pipeline model, this repo mostly updates adapters and diagnostics rather than inventing a parallel product.
