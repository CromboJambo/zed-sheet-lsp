# MVP Sheet Model

This note defines the MVP object model for `zed-sheet-lsp`.

The short version:

- Markdown tables are the preferred witness surface
- TSV remains a compatibility input
- the editor should operate on cells and layers, not raw delimiters
- `nustage` remains the canonical owner of pipeline intent and execution

## Product Sentence

`zed-sheet-lsp` is not a spreadsheet clone. It is a text-native tabular editor and witness layer for Markdown tables, linked documents, and `nustage` pipelines.

## Why This Model Exists

Plain Markdown and TSV are acceptable storage formats but poor direct editing primitives.

The user usually means:

- edit this cell
- inspect this formula
- follow this linked document
- freeze this preview
- lock this authored layer

The file, however, stores:

- row-major text
- delimiters
- formatting noise
- mixed authored and derived content

The MVP model lifts source text into cell-addressable objects while preserving plain-text persistence.

## Layers

The first MVP layer set is:

- `Values`
- `Formulas`
- `Links`
- `Preview`

These are not all persisted in Markdown.

### Markdown should store

- visible authored values
- simple formulas such as `=price * qty`
- inline links like `[User](./user.md)`

### Sidecar should eventually store

- lock state
- freeze state
- hidden state
- schema/type metadata
- pipeline bindings
- cached preview state
- richer cell-level metadata that would make Markdown noisy

## Current Rust Types

The frontend-agnostic entrypoint now lives in [zed-sheets-lsp/src/core.rs](../zed-sheets-lsp/src/core.rs).

That module exposes `CoreSheetDocument`, which bundles:

- parsed source
- normalized `SheetModel`
- resolved sidecar state

The goal is simple: Zed should not be the only consumer of the model. A future `zellij` frontend should be able to load the same core document and render or edit it differently.

The current starter types live in [zed-sheets-lsp/src/model.rs](../zed-sheets-lsp/src/model.rs).

Core objects:

- `SheetModel`
- `Cell`
- `CellValue`
- `CellKind`
- `CellMeta`
- `Layer`
- `LayerKind`
- `CellAddress`
- `LinkTarget`
- `FormulaValue`
- `CellRef`

Source parsing still begins in [zed-sheets-lsp/src/document.rs](../zed-sheets-lsp/src/document.rs):

- `SourceDocument`
- `SourceFormat`
- `TableBlock`
- `TableCell`

`SourceDocument::to_sheet_model()` is the current bridge from parsed source into richer editor semantics, and `CoreSheetDocument` is the reusable bundle that frontends should depend on.

## MVP Interpretation Rules

For now, cells are interpreted conservatively:

- `[label](./path.md)` becomes a link cell
- `=expr` becomes a derived/formula cell
- `@ref` becomes a reference cell
- numbers become numeric literals
- `true` and `false` become booleans
- everything else stays text

This is enough for the LSP to start reasoning about:

- linked documents
- focused formula editing
- layer-aware affordances
- future preview composition

## Boundaries

This model is an editor-facing witness model, not a replacement for canonical `nustage` semantics.

`zed-sheet-lsp` may:

- parse source tables
- classify cells
- expose cell/layer UX in the editor
- carry temporary editor metadata

`zed-sheet-lsp` should not become the long-term owner of:

- transformation semantics
- preview execution
- canonical sidecar schema
- pipeline validation rules

Those should move to or be shared from `nustage`.

## Near-Term Next Steps

1. Replace local sidecar ownership with canonical `nustage` sidecar parsing. The current migration path should load `.nustage.json` first and only fall back to `.zedsheets.json` temporarily.
2. Move hover/completion/diagnostics to consume `SheetModel` instead of reaching directly into `Grid`.
3. Add link-aware editor affordances:
   - hover target preview
   - go-to-definition for local Markdown links
4. Add formula-focused editing affordances against `CellKind::Derived`.
5. Add lock/freeze/hidden state as sidecar-backed editor metadata instead of source syntax.
