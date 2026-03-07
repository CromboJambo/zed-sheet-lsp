#!/usr/bin/env bash
#
# demo.sh
#
# Demo walkthrough script for Zed Sheets LSP
# Shows the "fake" preview toggle feature using tmux + tabiew
#
# Usage:
#   ./demo.sh demo/assets/sample.tsv
#
# This orchestrates the full demo:
#   - Starts tmux session with tabiew grid preview
#   - Opens Zed with the TSV file and LSP enabled
#   - Records with asciinema to demonstrate the feature
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_ASSETS="${SCRIPT_DIR}/assets"
TMUX_SESSION="zed-sheets-demo"

# Parse arguments
if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <tsv-file>"
    echo ""
    echo "Examples:"
    echo "  $0 demo/assets/sample.tsv"
    echo ""
    echo "This demonstrates:"
    echo "  1. LSP hover on TSV cells (column name + type)"
    echo "  2. Completions for column references"
    echo "  3. Grid preview in tabiew alongside raw file"
    echo "  4. Nushell filtering via Nu REPL"
    echo "  5. git-sheets audit trail"
    exit 1
fi

TSV_FILE="$1"

if [[ ! -f "$TSV_FILE" ]]; then
    echo "[ERROR] File not found: $TSV_FILE"
    exit 1
fi

# Check dependencies
check_dependency() {
    local cmd="$1"
    local msg="$2"
    if [[ -z "$(which "$cmd" 2>/dev/null)" ]]; then
        echo "[ERROR] $msg"
        echo "  Install with: $cmd --version"
        exit 1
    fi
}

check_dependency "tabiew" "tabiew not found. Install with: curl -L https://raw.githubusercontent.com/rodaine/tabiew/main/install.sh | bash"
check_dependency "nu" "Nushell not found. Install with: curl -fsSL https://nushell.sh/install.ps1 | bash"
check_dependency "asciinema" "asciinema not found (optional, for recording)"

echo "=== ZED SHEETS LSP DEMO ==="
echo ""
echo "Demonstrating preview toggle feature for TSV files"
echo "File: $TSV_FILE"
echo ""

# Clean up any existing tmux session
echo "[CLEANUP] Removing existing tmux session..."
tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true

# Start tmux session
echo "[START] Creating tmux session..."
tmux new-session -d -s "$TMUX_SESSION" -x 160 -y 100

# Create pane with tabiew (grid preview)
tmux split-window -h -t "$TMUX_SESSION" -c "$(dirname "$TSV_FILE")"
tmux select-pane -t "$TMUX_SESSION" -l
tmux send-keys -t "$TMUX_SESSION" "tabiew --port 3000 --no-remote '$TSV_FILE'" Enter

# Create pane with Zed (raw TSV with LSP)
tmux select-pane -t "$TMUX_SESSION" -r
tmux send-keys -t "$TMUX_SESSION" "zed open '$TSV_FILE'" Enter

# Enable mouse support
tmux set-option -g mouse on

echo "[INFO] Session created with 2 panes:"
echo "  Left:  tabiew grid preview (fake 'eyeball' tab)"
echo "  Right: Zed raw TSV with LSP hover/completions"
echo ""

# Start asciinema recording
RECORDING=""
if command -v asciinema &> /dev/null; then
    echo "[RECORD] Starting asciinema recording..."
    RECORDING="asciinema rec --stdin recording.cast"
    $RECORDING
    echo "Recording started. Type commands in tmux, then Ctrl+D to stop."
else
    echo "[INFO] Recording disabled (asciinema not available)"
fi

echo ""
echo "[INFO] Waiting for tabiew and Zed to initialize..."
sleep 3

# Show how to interact with the demo
echo ""
echo "=== INTERACTION GUIDE ==="
echo ""
echo "In the tmux session (use Ctrl+B for tmux bindings):"
echo ""
echo "  1. LEFT PANE (tabiew grid preview):"
echo "     - Shows rendered grid like the 'eyeball' preview tab"
echo "     - Mouse hover over cells"
echo "     - Ctrl+B then 'q' to quit tabiew"
echo ""
echo "  2. RIGHT PANE (Zed raw TSV):"
echo "     - Hover over header (line 0) -> shows column name"
echo "     - Hover over cell -> shows column + value"
echo "     - Type '$' + Tab -> autocompletion for columns"
echo "     - Edit any cell -> LSP updates"
echo ""
echo "  3. OPEN NU REPL IN A NEW PANE (optional):"
echo "     Ctrl+B then ':' then 'split-window -h'"
echo "     Then run: nu --commands 'open '$TSV_FILE"
echo ""
echo "  4. SAMPLE NU COMMANDS:"
echo "     - open '$TSV_FILE' | select product_name, quantity, unit_price"
echo "     - open '$TSV_FILE' | where unit_price > 5.00"
echo "     - open '$TSV_FILE' | group-by product_name | sum quantity"
echo "     - open '$TSV_FILE' | each { |it| ($it.quantity * $it.unit_price) }"
echo ""
echo "  5. git-sheets audit trail:"
echo "     - open '$TSV_FILE'.zedsheets.json (if sidecar exists)"
echo "     - Shows column metadata and dependencies"
echo ""

# Example Nu REPL commands for the demo
echo "[INFO] Sample Nu REPL commands you can run:"
echo ""
echo "  # Show column names and types"
echo "  open '$TSV_FILE' | describe"
echo ""
echo "  # Filter rows by price"
echo "  open '$TSV_FILE' | where unit_price > 5.00"
echo ""
echo "  # Group by product"
echo "  open '$TSV_FILE' | group-by product_name | into table { |it| [$it.product_name ($it | group-by quantity | length) as quantity] }"
echo ""

# Stop asciinema recording
if [[ -n "$RECORDING" ]]; then
    echo ""
    echo "[STOP] Press Ctrl+D to stop asciinema recording"
    echo ""
    # Keep recording running until user exits
    echo ""
    echo "Recording tip: Edit cells in Zed right pane and see grid update in left pane"
fi

echo ""
echo "=== DEMO COMPLETE ==="
echo ""
echo "To view the recording:"
echo "  less recording.cast.cast"
echo "  OR"
echo "  ffplay recording.cast.cast"
echo ""

# Keep tmux session alive
echo "[INFO] Press Ctrl+B then 'd' to detach from tmux session"
echo "[INFO] Session '$TMUX_SESSION' will continue running in background"
echo ""
echo "=== DEMO OVERVIEW ==="
echo ""
echo "This demo shows what the Zed preview toggle would enable:"
echo ""
echo "  BEFORE (current Zed):"
echo "    - Raw TSV in editor"
echo "    - No grid view available"
echo "    - Hover shows basic info (no column names)"
echo ""
echo "  WITH PREVIEW TOGGLE:"
echo "    - Raw TSV in editor (right pane)"
echo "    - Grid preview in new tab (left pane)"
echo "    - Hover shows column name + type"
echo "    - Edit TSV -> grid updates automatically"
echo "    - Completions for column references"
echo "    - git-sheets sidecar files for audit trail"
echo ""
echo "  KEY BENEFITS:"
echo "    - Natural spreadsheet-like UX for TSV files"
echo "    - See structure immediately (headers as row 0)"
echo "    - Filter/sort in preview pane"
echo "    - Edit in raw format, see rendered result"
echo "    - Perfect for data transformation workflows"
echo ""
