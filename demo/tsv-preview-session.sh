#!/usr/bin/env bash
#
# tsv-preview-session.sh
#
# Demo script that "fakes" the Zed preview toggle feature
# Uses tmux split panes: tabiew (grid view) on left, Zed (raw TSV) on right
#
# Usage:
#   ./tsv-preview-session.sh demo/assets/sample.tsv
#
# This creates a tmux session with:
#   - Left pane: tabiew showing grid preview (like the eyeball tab in Zed)
#   - Right pane: Zed showing the raw TSV with LSP hover/completions working
#

set -euo pipefail

SESSION_NAME="zed-sheets-demo"
TABIEW_PORT=3000

# Check dependencies
if [[ -z "$(which tabiew 2>/dev/null)" ]]; then
    echo "[ERROR] tabiew not found. Install with:"
    echo "  curl -L https://raw.githubusercontent.com/rodaine/tabiew/main/install.sh | bash"
    exit 1
fi

if [[ -z "$(which nu 2>/dev/null)" ]]; then
    echo "[ERROR] Nushell not found. Install with:"
    echo "  curl -fsSL https://nushell.sh/install.ps1 | bash"
    exit 1
fi

if [[ -z "$(which zed 2>/dev/null)" ]]; then
    echo "[ERROR] Zed not found. Make sure Zed is installed."
    exit 1
fi

if [[ -z "$(which asciinema 2>/dev/null)" ]]; then
    echo "[WARNING] asciinema not found. Recording will be skipped."
    RECORDING=""
else
    RECORDING="asciinema rec --stdin recording.cast"
fi

# Parse arguments
if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <tsv-file>"
    echo ""
    echo "Examples:"
    echo "  $0 demo/assets/sample.tsv"
    echo ""
    echo "This creates a tmux session with:"
    echo "  - Left pane: tabiew grid preview"
    echo "  - Right pane: Zed raw TSV with LSP hover"
    exit 1
fi

TSV_FILE="$1"

if [[ ! -f "$TSV_FILE" ]]; then
    echo "[ERROR] File not found: $TSV_FILE"
    exit 1
fi

# Create the session script
echo "Starting Zed Sheets demo session..."
echo "File: $TSV_FILE"
echo ""

# Create tmux session if it doesn't exist
if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    tmux new-session -d -s "$SESSION_NAME" -x 160 -y 100

    # Set up panes: tabiew on left (50%), Zed on right (50%)
    tmux split-window -v -t "$SESSION_NAME" -c "$TSV_FILE"

    # Pane 1: tabiew (grid preview) - left side
    tmux select-pane -t "$SESSION_NAME" -l
    tmux send-keys -t "$SESSION_NAME" "tabiew $TSV_FILE --port $TABIEW_PORT" Enter

    # Pane 2: Zed (raw TSV with LSP) - right side
    tmux select-pane -t "$SESSION_NAME" -r
    tmux send-keys -t "$SESSION_NAME" "zed open '$TSV_FILE'" Enter

    # Enable mouse support for better UX
    tmux set-option -g mouse on

    echo "Session '$SESSION_NAME' created with 2 panes"
    echo "Left: tabiew grid preview"
    echo "Right: Zed raw TSV with LSP hover"
    echo ""
    echo "Controls:"
    echo "  Press 'b' to see all available tmux binding keys"
    echo "  Press Ctrl+b then '%' to split pane"
    echo "  Press Ctrl+b then 'x' to close pane"
else
    echo "Session '$SESSION_NAME' already exists"
fi

# Start asciinema recording if asciinema is available
if [[ -n "$RECORDING" ]]; then
    echo ""
    echo "Starting asciinema recording..."
    $RECORDING
    echo "Recording started. Press Ctrl+D to stop."
else
    echo ""
    echo "Recording disabled (asciinema not found)"
fi
