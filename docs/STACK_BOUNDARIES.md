# Stack Boundaries

This note defines how `nustage`, `zed-sheet-lsp`, and `git-sheets` should relate to each other.

The goal is simple:

- each project remains useful on its own
- shared workflows compose cleanly
- new features have an obvious home

## The Stack

### `nustage`

Role: canonical transformation and sidecar engine for tabular data.

Owns:

- source loading and schema extraction
- canonical pipeline model
- pipeline validation
- sidecar format and persistence
- execution semantics
- schema history
- export helpers

It should still make sense if Zed does not exist.

### `zed-sheet-lsp`

Role: editor integration layer for Zed.

Owns:

- language registration
- LSP startup and transport
- hover, completion, diagnostics, symbols, rename, code actions
- preview hooks if Zed exposes them
- translating editor events into `nustage`-aware operations

It should still make sense if terminal UI and snapshot tooling do not exist.

### `git-sheets`

Role: snapshot, diff, integrity, and audit tooling for tabular state.

Owns:

- immutable snapshots of table state
- diffing between snapshots
- integrity hashes
- repository layout for history artifacts
- git-adjacent audit workflows

It should still make sense even if there is no pipeline engine.

## Mental Model

Use this sentence to stay oriented:

- `nustage` explains what the data means and how it changes
- `zed-sheet-lsp` explains that model inside the editor
- `git-sheets` records what changed over time

Or more bluntly:

- intent lives in `nustage`
- interaction lives in `zed-sheet-lsp`
- history lives in `git-sheets`

## Feature Placement Rules

When a new feature appears, ask these questions in order.

### 1. Is it the canonical representation of user intent?

Examples:

- pipeline step types
- sidecar schema
- transformation validation
- schema drift detection

Bucket: `nustage`

### 2. Is it only about how the user sees or edits that intent inside Zed?

Examples:

- hover text
- completions
- rename support
- workspace symbols
- code actions
- preview panel wiring

Bucket: `zed-sheet-lsp`

### 3. Is it about recording, diffing, or verifying state over time?

Examples:

- snapshots
- audit history
- before/after table comparison
- integrity hashes
- commit-oriented workflows

Bucket: `git-sheets`

## Anti-Drift Rules

These are the rules that prevent the stack from turning into three partial products.

### Rule 1: one canonical sidecar format

Pipeline intent should live in one sidecar model only.

Current direction:

- `.nustage.json` is the canonical intent sidecar

Do not add another parallel editor-owned sidecar for the same purpose unless there is a strict compatibility boundary and a documented reason.

### Rule 2: editor repos should not invent business semantics

`zed-sheet-lsp` can interpret and present pipeline data, but it should not become the place where transformation semantics are invented.

Bad examples:

- editor-only formula language
- editor-only dependency graph
- editor-only schema rules

Good examples:

- render a pipeline step in hover
- offer completion based on canonical schema
- surface canonical validation errors as diagnostics

### Rule 3: history is not intent

`git-sheets` snapshots are evidence of state, not the definition of a pipeline.

Bad examples:

- using snapshots as the primary transformation model
- inferring canonical intent from diff artifacts

Good examples:

- snapshoting pipeline outputs
- diffing outputs across sidecar revisions
- preserving audit trails for important table changes

### Rule 4: each repo should expose a clean library core

Every repo should be usable without its preferred frontend.

That means:

- `nustage` should have stable library APIs independent of CLI/TUI
- `zed-sheet-lsp` should keep its domain logic separate from Zed-specific startup code where possible
- `git-sheets` should keep snapshot/diff logic separate from CLI commands

### Rule 5: frontends should be thin

Frontends are allowed to orchestrate. They should not become the only place where core logic exists.

Examples:

- LSP diagnostics should come from shared validation rules, not bespoke regex hacks
- CLI commands should call library APIs, not carry the only implementation
- demo scripts should prove workflows, not become product architecture

## Dependency Direction

Recommended direction:

- `zed-sheet-lsp` may depend on a stable library slice of `nustage`
- `zed-sheet-lsp` may optionally call into `git-sheets` for history-oriented features later
- `nustage` should not depend on `zed-sheet-lsp`
- `git-sheets` should not depend on `zed-sheet-lsp`

Strong preference:

- `nustage` and `git-sheets` remain peers
- composition happens at the workflow layer, not through deep mutual dependency

## A Simple Bucket Test

When a feature starts creeping, use this checklist:

1. If removed from the editor, would the feature still matter?
2. If removed from git history, would the feature still matter?
3. Is it expressing intent, interaction, or history?
4. Does implementing it here create a second source of truth?
5. Can the other repos stay functional if this feature lives here?

Use the answers like this:

- mostly intent: `nustage`
- mostly editor interaction: `zed-sheet-lsp`
- mostly history: `git-sheets`
- if it creates duplicate truth, stop and redesign

## Examples

### Column rename across pipeline steps

Bucket: `nustage` plus `zed-sheet-lsp`

Split:

- `nustage` should define how a canonical rename updates step references safely
- `zed-sheet-lsp` should expose rename in the editor

### Show schema changes after applying a step

Bucket: `nustage`, surfaced by `zed-sheet-lsp`

Split:

- `nustage` computes schema history and diff
- `zed-sheet-lsp` renders it in hover, diagnostics, or code actions

### Snapshot the result of a transformed table

Bucket: `git-sheets`, triggered by workflow integration

Split:

- `nustage` produces the current tabular result
- `git-sheets` snapshots and diffs it
- `zed-sheet-lsp` may offer a command later, but should not own the snapshot model

### Grid preview in Zed

Bucket: `zed-sheet-lsp`

With input from:

- `nustage` for the rendered or computed data state

Not `git-sheets`, unless the preview is explicitly a historical diff view.

## Review Heuristics

When reviewing new code, look for these smells.

### Smells that a feature is in the wrong bucket

- another sidecar schema appears
- the LSP defines transformation rules that disagree with `nustage`
- a snapshot format starts carrying canonical pipeline intent
- CLI/TUI code becomes the only implementation of a core behavior
- docs describe one repo as owning everything

### Good signs

- one source of truth for each concept
- library-first domain logic
- thin adapters at frontend boundaries
- explicit contracts between repos
- tests that prove a module works without the other frontends

## Practical Rule

Before adding a feature, write one sentence in the PR description:

"This belongs in `<repo>` because it is primarily about `<intent|interaction|history>`."

If that sentence is hard to write, the design boundary is probably not clear enough yet.
