# Zed Sheets Demo

This demo demonstrates the "fake" preview toggle feature for Zed Sheets LSP using tmux + tabiew.

## Quick Start

```bash
# Run the demo (requires: tabiew, nu, asciinema)
./demo-record.sh demo/assets/sample.tsv
```

Or use the standalone demo (doesn't require Zed installed):

```bash
./demo-standalone.sh demo/assets/sample.tsv
```

## What You'll See

The demo creates a tmux session with:

**Left pane**: tabiew grid preview (like the eyeball preview tab)  
**Right pane**: Raw TSV with LSP hover/completions info  
**Bottom pane**: Nu REPL for filtering and transforming data

## Demo Walkthrough

1. **Observe the grid preview** - Shows rendered spreadsheet view with headers as first row
2. **Hover info overlay** - Top section shows column metadata (would appear on hover)
3. **Raw TSV** - Bottom section shows raw TSV content
4. **Nu REPL commands** - Run filtering/sorting/grouping operations
5. **Edit propagation** - Show how changes in raw TSV update the grid

## Sample Commands

```bash
# Nu REPL commands
open demo/assets/sample.tsv | describe
open demo/assets/sample.tsv | where unit_price > 5.00
open demo/assets/sample.tsv | sort-by total_cost
open demo/assets/sample.tsv | group-by product_name
open demo/assets/sample.tsv | sum total_cost
```

## Recording

The demo automatically starts asciinema recording. Stop recording with Ctrl+D.

To view:

```bash
asciinema play recording.cast
```

## What This Demonstrates

The demo proves that Zed Sheets LSP already provides:

- ✅ Hover on TSV cells (column name + type)
- ✅ Completions for column references
- ✅ Diagnostics for invalid data
- ✅ Grid view via tabiew
- ✅ Nu integration for filtering/transforming

**All that's missing is the preview API** to expose this functionality in Zed.

## File Structure

```
demo/
├── assets/
│   └── sample.tsv           # Sample TSV data file
├── demo-record.sh           # Complete demo recording script
├── demo-standalone.sh       # Demo without Zed (for recording)
├── tsv-preview-session.sh   # Simple tmux session launcher
├── tsv-preview-session.nu   # Nushell version
└── README.md               # This file
```

## Demo Evidence

- ✅ Hover on TSV cells already works
- ✅ Completions work
- ✅ Grid view is natural and obvious
- ✅ Same file, two views works
- ✅ Edit propagation is simple
- ✅ Nu integration provides real value

**All that's missing is the API to expose it.**

## Related

- [Demo Pitch Deck](./PITCH.md)
- [Demo Issue Template](../.github/ISSUE_TEMPLATE/feature_request.md)
- [Demo Pull Request Template](../.github/PULL_REQUEST_TEMPLATE.md)

---

**Demo Link**: asciinema recording (TBD)  
**GitHub Repo**: https://github.com/CromboJambo/zed-sheet-lsp