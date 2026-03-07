#!/usr/bin/env bash
#
# demo-standalone.sh
#
# Standalone demo script that demonstrates the "fake" preview toggle feature
# WITHOUT requiring Zed to be installed. Perfect for recording a demo video
# to show the Zed team what the preview API would unlock.
#
# Usage:
#   ./demo-standalone.sh demo/assets/sample.tsv
#
# This creates a tmux layout that simulates:
#   - Left: tabiew grid preview (like the eyeball preview tab)
#   - Right: Raw TSV with simulated LSP hover info

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
        exit 1
    fi
}

check_dependency "tabiew" "tabiew not found. Install with: curl -L https://raw.githubusercontent.com/rodaine/tabiew/main/install.sh | bash"
check_dependency "nu" "Nushell not found. Install with: curl -fsSL https://nushell.sh/install.ps1 | bash"
check_dependency "tmux" "tmux not found"

# Create or reset tmux session
echo "=== ZED SHEETS DEMO: Simulating Preview Toggle Feature ==="
echo ""
echo "This demo shows what the Zed preview toggle would enable for TSV files."
echo "Left pane: Grid preview (like the eyeball tab)"
echo "Right pane: Raw TSV with column info overlay"
echo ""

# Clean up any existing session
tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true

# Create tmux session
echo "[START] Creating tmux session..."
tmux new-session -d -s "$TMUX_SESSION" -x 200 -y 50

# Create split panes: tabiew on left (70%), raw content on right (30%)
tmux split-window -h -t "$TMUX_SESSION"
tmux select-pane -t "$TMUX_SESSION" -l

# Start tabiew in left pane
tmux send-keys -t "$TMUX_SESSION" "tabiew --port 3000 '$TSV_FILE'" Enter
sleep 1

# In right pane, show raw TSV content
tmux select-pane -t "$TMUX_SESSION" -r
tmux send-keys -t "$TMUX_SESSION" "cat '$TSV_FILE'" Enter
sleep 0.5

# Create LSP hover info overlay file
HOVER_INFO_FILE="/tmp/zed-sheets-hover-info"

cat > "$HOVER_INFO_FILE" << 'HOVER_EOF'
# This file shows the hover info that would appear when hovering in Zed
# Each line represents a hover target with its info

# Column headers (line 0 of TSV)
product_name|Column|String
product_type|Column|String
quantity|Column|Number
unit_price|Column|Number
total_cost|Column|Number
stock_date|Column|String
warehouse_location|Column|String
status|Column|String
HOVER_EOF

# In right pane, show the hover info alongside raw content
echo "" >> "$HOVER_INFO_FILE"
echo "# Hover over cells to see this info:" >> "$HOVER_INFO_FILE"
cat "$TSV_FILE" >> "$HOVER_INFO_FILE"

tmux select-pane -t "$TMUX_SESSION" -r
tmux send-keys -t "$TMUX_SESSION" "cat '$HOVER_INFO_FILE'" Enter
sleep 0.5

# Show demo instructions
echo "[INFO] Demo session created with 2 panes:"
echo "  Left:  tabiew grid preview"
echo "  Right: Raw TSV + column info overlay"
echo ""

# Start asciinema recording if available
RECORDING=""
if command -v asciinema &> /dev/null; then
    echo "[RECORD] Starting asciinema recording..."
    RECORDING="asciinema rec --stdin recording.cast"
    $RECORDING
    echo "Recording started. Commands will be recorded."
else
    echo "[INFO] Recording disabled (asciinema not available)"
fi

echo ""
echo "=== DEMO WALKTHROUGH ==="
echo ""
echo "Left pane (tabiew grid preview):"
echo "  ✓ Shows rendered grid like the 'eyeball' preview tab"
echo "  ✓ Headers visible as first row"
echo "  ✓ Mouse hover shows cell values"
echo "  ✓ Click to select and copy"
echo ""
echo "Right pane (raw TSV + hover info):"
echo "  ✓ Shows raw TSV content"
echo "  ✓ Top section shows column metadata (would appear on hover)"
echo "  ✓ Bottom section shows raw TSV"
echo "  ✓ Edit cells to see LSP diagnostics"
echo ""
echo "=== DEMO STEPS ==="
echo ""
echo "1. Observe the grid preview on the left (tabiew)"
echo ""
echo "2. Observe the raw TSV on the right with column info"
echo "   - Hover would show: Column name + type"
echo ""
echo "3. Run these Nu commands in the tabiew pane:"
echo ""

# Prepare Nu commands for the demo
echo "   # First, open the file and check structure:"
echo "   open '$TSV_FILE' | describe"
echo ""
echo "   # Filter by price:"
echo "   open '$TSV_FILE' | where unit_price > 5.00"
echo ""
echo "   # Sort by total cost:"
echo "   open '$TSV_FILE' | sort-by total_cost"
echo ""
echo "   # Group by product:"
echo "   open '$TSV_FILE' | group-by product_name"
echo ""
echo "   # Calculate total inventory value:"
echo "   open '$TSV_FILE' | sum total_cost"
echo ""

# Add a Nu pane with the REPL
echo "4. We'll open a Nu REPL pane to show filtering:"
tmux split-window -v -t "$TMUX_SESSION"
tmux select-pane -t "$TMUX_SESSION" -t:0
tmux send-keys -t "$TMUX_SESSION" "nu" Enter

sleep 2

# Show Nu REPL output
echo ""
echo "[Nu REPL] Run these commands in the bottom pane:"
echo ""
echo "  open '$TSV_FILE'"
echo "  | describe"
echo ""
echo "  open '$TSV_FILE'"
echo "  | where unit_price > 5.00"
echo ""
echo "  open '$TSV_FILE'"
echo "  | sort-by total_cost"
echo ""
echo "  open '$TSV_FILE'"
echo "  | group-by product_name | into table { |it| ['product' $it.product_name 'count' ($it | length)] }"
echo ""

# Show the git-sheets audit trail concept
echo ""
echo "5. git-sheets audit trail (conceptual):"
echo "   The .zedsheets.json sidecar file would contain:"
echo ""
echo "   {\"version\":1,\"columns\":{\"product_name\":{\"type\":\"string\"}..."
echo "   }"
echo ""

# Show demo conclusion
echo ""
echo "=== DEMO COMPLETE ==="
echo ""
echo "=== FEATURE SUMMARY ==="
echo ""
echo "This demo shows what the Zed preview toggle API would enable:"
echo ""
echo "  • Preview toggle (eyeball icon): Opens grid view in new tab"
echo "  • Two-pane editing: Edit raw TSV, see grid update"
echo "  • Hover info: Shows column name + type on hover"
echo "  • Completions: Tab after '$' for column references"
echo "  • Diagnostics: Warnings/errors in raw editor"
echo "  • Sidecar files: git-sheets.json for audit trail"
echo "  • Nu integration: Run commands, filter, transform"
echo ""
echo "=== WHAT THIS DEMONSTRATES TO ZED TEAM ==="
echo ""
echo "  1. LSP hover on TSV cells already works (column name + type)"
echo "  2. Grid preview is natural and obvious"
echo "  3. Raw + preview together is the natural UX"
echo "  4. Edit in raw, transform in preview - same data"
echo "  5. Sidecar files provide audit trail"
echo ""
echo "  ALL THAT'S NEEDED IS THE PREVIEW API:"
echo "  • A hook to open preview content in new tab"
echo "  • WebSocket/stdio for real-time updates"
echo "  • Preview content negotiation (like markdown)"
echo ""

# Stop asciinema recording if running
if [[ -n "$RECORDING" ]]; then
    echo ""
    echo "[STOP] Press Ctrl+D to stop asciinema recording"
    echo ""
    echo "Recording will save to: recording.cast"
fi

# Show tmux info
echo ""
echo "[INFO] Session '$TMUX_SESSION' is running in background"
echo ""
echo "Controls:"
echo "  Ctrl+B then 'b' - Show tmux binding keys"
echo "  Ctrl+B then 'd' - Detach from session"
echo "  Ctrl+B then 'c' - Switch to pane"
echo "  Ctrl+B then '%' - Split pane"
echo "  Ctrl+B then 'x' - Close pane"
echo ""
echo "=== DEMO SCRIPT COMPLETE ==="
