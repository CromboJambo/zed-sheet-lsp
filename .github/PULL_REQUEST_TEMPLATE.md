---
title: "Feature: Preview Toggle for TSV Files"
labels: ["feature", "enhancement", "preview"]

---

## Description

<!--- Describe your changes in detail. Explain what problem this solves and how it works. -->

## Related Issue

<!--- If there is an open issue related to this PR, link it below. -->

- [ ] Related to issue #___

## Type of Change

- [ ] New feature (non-breaking change which adds functionality)
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] This change requires a documentation update

## Screenshots / Demo

<!--- Add screenshots or asciinema recordings to demonstrate the feature. -->

## How to Test

<!--- Describe the tests that you ran to verify your changes. -->

- Open a .tsv file in Zed
- Hover over a header cell -> shows column name
- Hover over a data cell -> shows column name + value
- Press the eyeball toggle -> opens grid preview in new tab

## Checklist

<!--- Go over all the following points, and put an `x` in all the boxes that apply. -->

- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works

## Demo Link

<!--- If this is a demo for the preview API feature request, paste the asciinema URL here. -->

- asciinema demo: ___
- GitHub repo: https://github.com/CromboJambo/zed-sheet-lsp

## What This Demonstrates

This PR demonstrates the "fake" preview toggle feature for TSV files using tmux and tabiew as a prototype. It shows:

1. **LSP hover on TSV cells** - Shows column name + type on hover
2. **Grid preview** - tabiew renders the TSV as a spreadsheet view
3. **Same file, two views** - Raw TSV on right, grid preview on left
4. **Edit in raw, see in preview** - Changes propagate between views
5. **Completions** - Tab completion for column references
6. **Diagnostics** - Warnings/errors shown in the editor

All the plumbing exists in the LSP - what's missing is the Zed preview API to expose this functionality.

## Technical Notes

The demo uses a tmux-based layout:

```
+--------------------------------+--------------------------------+
|  tabiew (grid preview)         |  Zed (raw TSV)                 |
|                                |  + hover info overlay         |
++--------------------------------+--------------------------------+
|  Nu REPL (optional)            |                                |
+--------------------------------+--------------------------------+
```

This simulates what the preview toggle would look like with native Zed support.

## Next Steps After API Availability

Once Zed exposes the preview API, this functionality can be integrated directly:

1. Replace tmux panes with Zed preview tabs
2. Use Zed's native preview provider API
3. Integrate sidecar file viewer
4. Add real-time sync between raw and preview

---

**Note:** This PR serves as a proof-of-concept to demonstrate the value of a preview API for TSV files. See the README in the demo folder for more details.