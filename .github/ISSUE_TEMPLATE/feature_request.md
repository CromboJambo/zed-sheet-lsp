---
title: "Feature: Preview Toggle API for TSV Grid Views"
labels: ["feature", "enhancement", "preview", "high priority"]
---

## Summary

Add a preview provider API that allows extensions to render alternate views of the same file in new tabs. This would enable a "preview toggle" (eyeball icon) for TSV files that opens a grid view alongside the raw text editor.

## Motivation

TSV files have a natural grid representation that's much easier to read and work with than raw text. Currently, Zed sheets LSP provides:

- Hover tooltips showing column names and values
- Completions for column references
- Diagnostics for invalid data
- git-sheets sidecar files for audit trails

But there's no way to render a grid view in a separate tab. The tmux + tabiew prototype demo shows this is already possible — we just need the Zed API to expose it natively.

## Proposed API

Add a `preview` capability to the language server that returns preview content:

```typescript
// Extension side (Zed)
async function preview(params: PreviewParams): Promise<PreviewContent>

interface PreviewParams {
    textDocument: TextDocumentIdentifier
    position: Position
}

type PreviewContent =
    | SimplePreview
    | WebviewPreview

interface SimplePreview {
    kind: 'plaintext' | 'markdown'
    contents: string | MarkupContent
}

interface WebviewPreview {
    kind: 'webview'
    url: URL
    title: string
}
```

For TSV, we'd use a webview to render the tabiew grid.

## Demo

This repository includes a complete working prototype using tmux + tabiew that simulates the preview toggle feature. It demonstrates:

1. **Hover functionality** - Shows column name + type on hover (already working in Zed LSP)
2. **Grid preview** - tabiew renders TSV as a spreadsheet
3. **Same file, two views** - Raw TSV on right, grid on left
4. **Edit propagation** - Changes in raw TSV update the grid
5. **Completions** - Tab completion for column references
6. **Nu integration** - Filter/transform data in a REPL pane

### Demo Video

<asciinema link will be here after recording>

### Demo Repository

https://github.com/CromboJambo/zed-sheet-lsp/tree/main/demo

## Use Case

A developer working with TSV data:

1. Opens `inventory.tsv` in the editor
2. Hovers over headers to see column names
3. Hovers over cells to see data with type info
4. Uses tab completions to autocomplete references
5. **Clicks eyeball icon** → Opens grid preview in new tab
6. **Edits in raw tab** → Grid updates automatically
7. **Filters/sorts in preview** → Sees transformed view
8. **git-sheets** provides audit trail

This is the natural UX for spreadsheet-like data, and Zed Sheets LSP already provides all the data and hooks.

## Technical Details

### Implementation Approach

1. **Content provider** - Extension registers a new content provider for preview
2. **Preview tab** - Opens in a new tab like the Markdown preview
3. **Webview** - Renders tabiew in a webview with WebSocket sync
4. **Real-time updates** - LSP publishes diagnostics/changes to preview
5. **Shared memory** - Both views reference the same buffer

### Similar Patterns

- **Markdown** - Core editor provides markdown preview in new tab
- **Diff views** - Show changes in separate view
- **Git blame** - Shows blame annotations
- **Format on save** - Shows formatted output

TSV grid preview should follow the same pattern.

## Questions for the Team

1. Is there a roadmap for preview providers in extensions?
2. What's the preferred API surface?
3. Should we use webviews or a different approach?
4. Are there restrictions on what content can be previewed?

## Alternative: Custom Webview Hook

If a general preview API isn't available, a simpler approach:

```typescript
// Extension registers a webview hook
interface WebviewHook {
    register(url: URL, title: string): void
}

// Extension requests preview tab
async function requestPreview(params: PreviewParams): Promise<WebviewHook | null>
```

This would let extensions request a preview tab, and Zed could open it as a webview.

## Demo Evidence

The demo in this repo proves:

- ✅ Hover on TSV cells already works
- ✅ Completions work
- ✅ Grid view is natural and obvious
- ✅ Same file, two views works
- ✅ Edit propagation is simple
- ✅ Nu integration provides real value

All that's missing is the API to expose it.

## Next Steps

1. **Add preview API** - Expose preview capability
2. **Update Zed Sheets LSP** - Register preview provider
3. **Tab integration** - Add eyeball toggle in toolbar
4. **Test** - Verify hover/edit sync works

## Related Work

- [Tabiew](https://github.com/rodaine/tabiew) - TSV grid viewer
- [git-sheets](https://github.com/zed-industries/git-sheets) - Already uses Zed extension API
- [Zed Markdown Preview](https://zed.dev/docs/extension-api#preview-provider) - Core preview system

---

**Demo Link:** [asciinema recording](TBD)  
**GitHub Repo:** https://github.com/CromboJambo/zed-sheet-lsp