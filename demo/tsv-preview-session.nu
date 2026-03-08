#!/usr/bin/env nu
# zed-sheet-lsp/demo/tsv-preview-session.nu
#
# Demo script that "fakes" the Zed preview toggle feature
# Uses tmux split panes: tabiew (grid view) on left, Zed (raw TSV) on right
#
# Usage:
#   nu demo/tsv-preview-session.nu demo/assets/sample.tsv
#   OR
#   ./demo/tsv-preview-session.sh demo/assets/sample.tsv
#
# This creates a tmux session with:
#   - Left pane: tabiew showing grid preview (like the eyeball tab in Zed)
#   - Right pane: Zed showing the raw TSV with LSP hover/completions working

# Configuration
$env.TMUX_SESSION_NAME = "zed-sheets-demo"
$env.TABIEW_PORT = 3000

# Ensure dependencies are installed
if (which tabiew | is-empty) {
    print $"[ERROR] tabiew not found. Install with: curl -L https://raw.githubusercontent.com/rodaine/tabiew/main/install.sh | bash"
    exit 1
}

if (which nu | is-empty) {
    print $"[ERROR] Nushell not found. Install with: curl -fsSL https://nushell.sh/install.ps1 | nix run"
    exit 1
}

def create-session-script [$file: string] {
    let template = '
#!/bin/bash
set -e

SESSION_NAME="__SESSION_NAME__"
TSV_FILE="__TSV_FILE__"

echo "Starting Zed Sheets demo session..."
echo "File: $TSV_FILE"

# Create tmux session if it does not exist
if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    tmux new-session -d -s "$SESSION_NAME" -x 160 -y 100

    # Set up panes: tabiew on left (50%), Zed on right (50%)
    tmux split-window -h -t "$SESSION_NAME" -c "$(dirname "$TSV_FILE")"

    # Pane 1: tabiew (grid preview)
    tmux select-pane -t "$SESSION_NAME":0.0
    tmux send-keys -t "$SESSION_NAME":0.0 "tabiew \"$TSV_FILE\" --port __TABIEW_PORT__" Enter

    # Pane 2: Zed (raw TSV with LSP)
    tmux select-pane -t "$SESSION_NAME":0.1
    tmux send-keys -t "$SESSION_NAME":0.1 "zed open \"$TSV_FILE\"" Enter

    # Enable mouse support for better UX
    tmux set-option -g mouse on

    echo "Session '"'"'$SESSION_NAME'"'"' created with 2 panes"
    echo "Left: tabiew grid preview"
    echo "Right: Zed raw TSV with LSP hover"
    echo ""
    echo "Controls:"
    echo "  Press Ctrl+b then % to split pane"
    echo "  Press Ctrl+b then x to close pane"
else
    echo "Session '"'"'$SESSION_NAME'"'"' already exists"
fi

tmux attach -t "$SESSION_NAME"
'

    $template
    | str replace -a "__SESSION_NAME__" $env.TMUX_SESSION_NAME
    | str replace -a "__TSV_FILE__" $file
    | str replace -a "__TABIEW_PORT__" ($env.TABIEW_PORT | into string)
}

def main [file?: string] {
    if ($file | is-not-empty) {
        if ($file | path exists) {
            print $"Creating session script for: $file"
            print ""
            print (create-session-script $file)
            print ""
            print "To run this session, copy and paste the script above into your terminal."
        } else {
            print $"[ERROR] File not found: $file"
            exit 1
        }
    } else {
        print "Usage: nu demo/tsv-preview-session.nu <tsv-file>"
        print ""
        print "This will generate a tmux session script with:"
        print "  - Left pane: tabiew showing grid preview (like Zed's eyeball tab)"
        print "  - Right pane: Zed showing raw TSV with LSP hover/completions"
    }
}
