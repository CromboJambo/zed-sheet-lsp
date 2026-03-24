# Team Architecture

This document defines the current team-facing architecture for `zed-sheet-lsp`.

The goal is to let multiple contributors work in parallel without inventing competing product shapes.

## Product Boundary

`zed-sheet-lsp` is not a spreadsheet clone.

It is a text-native tabular editor and witness layer for:

- Markdown tables
- linked Markdown documents
- `nustage` sidecars and pipeline metadata

## Golden Path

The current MVP workflow is:

1. open a `.sheet.md` file
2. parse the Markdown table into semantic cells
3. resolve the matching sidecar, preferring `.nustage.json`
4. classify cells as literal, link, formula, or reference
5. expose hover, completion, and diagnostics from the semantic sheet model

That workflow is captured in package fixtures and integration tests under:

- [zed-sheets-lsp/tests/fixtures/golden/demo.sheet.md](../zed-sheets-lsp/tests/fixtures/golden/demo.sheet.md)
- [zed-sheets-lsp/tests/fixtures/golden/demo.nustage.json](../zed-sheets-lsp/tests/fixtures/golden/demo.nustage.json)
- [zed-sheets-lsp/tests/fixtures/golden/docs/user.md](../zed-sheets-lsp/tests/fixtures/golden/docs/user.md)
- [zed-sheets-lsp/tests/golden_path.rs](../zed-sheets-lsp/tests/golden_path.rs)

## Module Ownership

### `src/core.rs`

Owns the frontend-agnostic document bundle:

- parsed source
- normalized `SheetModel`
- resolved sidecar state
- reusable helpers like link-target resolution

Every frontend should prefer this module as its entrypoint.

### `src/document.rs`

Owns the Zed LSP adapter:

- open/change/close lifecycle
- hover
- completion
- diagnostics publication

This module should stay thin. It should orchestrate, not invent core semantics.

### `src/model.rs`

Owns the rich sheet model:

- cells
- layers
- links
- formulas
- references
- editor metadata

This is the editor-facing semantic layer.

### `src/sidecar.rs`

Owns sidecar resolution and local compatibility parsing:

- resolve `.nustage.json` first
- fall back to `.zedsheets.json` temporarily
- keep the migration seam isolated

This module should converge toward canonical `nustage` types over time.

## Parallel Work Buckets

These are the current safe workstreams for team development:

- core source parsing and source normalization
- rich cell and layer semantics
- sidecar migration toward canonical `nustage`
- Zed LSP affordances
- demo fixtures and workflow documentation
- future terminal or `zellij` frontend work against `core`

## Rules

- Do not add a new parallel sidecar format.
- Do not put product semantics only in the Zed adapter.
- Prefer changes in `core`, `model`, or `sidecar` before adding frontend-specific behavior.
- Keep `.sheet.md` readable without the sidecar.
- Treat `.zedsheets.json` as migration compatibility, not as the long-term target.

## Definition Of Ready For Team Work

A task is ready to hand to the team when:

- the target module is obvious
- the fixture or demo path affected by the change is identified
- tests can express success without manual interpretation
- the task does not require inventing a new ownership boundary

## Near-Term Team Tasks

1. replace local sidecar parsing with canonical `nustage` types
2. add link-aware navigation and richer hover
3. add formula-focused editing semantics
4. add sidecar-backed lock, freeze, and hidden state
5. prototype a terminal or `zellij` witness frontend against `core`
