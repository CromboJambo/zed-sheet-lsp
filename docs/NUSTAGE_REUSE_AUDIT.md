# Nustage Reuse Audit

Audit target: `/home/crombo/nustage`

Purpose: identify which parts of `nustage` are currently reusable by `zed-sheet-lsp`, which parts are promising but unstable, and which parts should be avoided for now.

## Reuse Now

### `src/sidecar/mod.rs`

Status: good candidate for direct reuse.

Useful pieces:

- `SidecarFile`
- `SidecarMetadata`
- `SidecarFile::load`
- `SidecarFile::sidecar_path_for_source`
- schema history and metadata fields
- pipeline text and schema diff helpers

Why it is useful:

- gives `zed-sheet-lsp` a canonical sidecar format immediately
- replaces the local `.zedsheets.json` experiment
- captures source path, pipeline steps, and schema snapshots in one model

Risk:

- currently tied to filesystem loading and saving, so the LSP will likely want a parse-from-string path as well

### `src/transformations/mod.rs`

Status: reusable as the canonical model layer, but not yet as the sole validation engine.

Useful pieces:

- `TransformationStep`
- `StepType`
- `TransformationPipeline`
- `ColumnSchema`
- the step taxonomy itself

Why it is useful:

- gives the LSP a stable vocabulary for hover, completion, rename, and diagnostics
- aligns Zed features with the actual product model instead of an invented cell-formula model

Risk:

- validation is uneven and only covers some step kinds
- the file is doing both model definition and execution logic
- the step model is more stable than the execution code

Recommendation:

Treat the types as canonical now. Treat the execution and validation behavior as provisional until covered by tests.

### `src/mcode/mod.rs`

Status: reusable as a secondary capability, not a core dependency.

Useful pieces:

- `generate_m_code`
- field-reference translation from `@Field` to Power Query M syntax

Why it is useful:

- gives a strong export story for future editor commands
- can support hover or code actions like "show equivalent M"

Risk:

- useful but not load-bearing for Zed integration
- should not block the LSP architecture

## Reuse Later

### `src/data/mod.rs`

Status: promising for source loading and schema extraction, but not yet clean enough to rely on broadly from the LSP.

Useful pieces:

- `load_data`
- `get_schema`

Problems:

- placeholder implementations remain in `get_unique_values` and `get_column_stats`
- Excel loading is still rough around inference and fidelity
- pulling this module directly into the LSP would also pull in heavy execution dependencies

Recommendation:

Use this through a narrow `nustage` library facade later, or isolate a lighter schema-loading layer before integrating it into the editor path.

### `src/ironcalc/mod.rs`

Status: not ready for dependency from `zed-sheet-lsp`.

Problems:

- placeholder implementations remain
- unrelated to the immediate Zed adapter boundary

Recommendation:

Keep it out of the LSP until its role in the broader product is clearer.

## Avoid For Now

### CLI and TUI surfaces

Files:

- `src/main.rs`
- `src/cli/mod.rs`
- `src/tui.rs`
- `src/tui_grid.rs`

Status: useful for demos and manual workflows, but not reusable as editor integration primitives.

Reason:

- they are product frontends, not shared model APIs
- Zed needs library contracts and stable metadata, not terminal UI code

## Gaps That Matter To Zed

### Parse and validate from in-memory content

The LSP works on open buffers, not only files on disk. `nustage` will be easier to consume if it exposes:

- `SidecarFile::from_str`
- validation helpers that accept source schema and pipeline without filesystem assumptions

### Stable diagnostics API

The LSP should not reconstruct diagnostics from regexes or duplicate business rules. `nustage` should eventually expose:

- invalid step references
- missing columns
- schema drift
- unsupported transformations

in a structured form.

### Source-type flexibility

This repo currently only wires `.tsv` in Zed. The broader product wants CSV, Parquet, and Excel-adjacent workflows. The editor integration should grow from "TSV support" to "tabular source support," but only after the sidecar and schema contracts are stable.

## Bottom Line

Best immediate reuse:

1. `sidecar`
2. transformation types
3. selected schema helpers

Do not build the next phase of `zed-sheet-lsp` around:

1. custom `.zedsheets.json` metadata
2. local `$row.column` formula semantics
3. TUI or CLI code
4. placeholder-heavy modules

## Recommended Next Code Changes

1. Replace local sidecar loading in `zed-sheets-lsp/src/document.rs` with a canonical `nustage` sidecar parser.
2. Rename the local project language in docs from spreadsheet/formula framing to pipeline/sidecar framing.
3. Add real tests around sidecar-backed diagnostics and hover using fixture pairs.
