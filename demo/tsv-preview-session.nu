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

# Create the session script
def create-session-script --in-raw --files:[$env.ARGS] {
    |$file as string

    """
    #!/bin/bash
    set -e

    SESSION_NAME="$env.TMUX_SESSION_NAME"
    TSV_FILE="$file"

    echo "Starting Zed Sheets demo session..."
    echo "File: $TSV_FILE"

    # Create tmux session if it doesn't exist
    if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        tmux new-session -d -s "$SESSION_NAME" -x 160 -y 100

        # Set up panes: tabiew on left (50%), Zed on right (50%)
        tmux split-window -v -t "$SESSION_NAME" -c "$TSV_FILE"

        # Pane 1: tabiew (grid preview) - left side
        (tmux select-pane -t "$SESSION_NAME" -l)
        tmux send-keys -t "$SESSION_NAME" "tabiew $TSV_FILE --port $env.TABIEW_PORT" Enter

        # Pane 2: Zed (raw TSV with LSP) - right side
        (tmux select-pane -t "$SESSION_NAME" -r)
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
        tmux attach -t "$SESSION_NAME"
    fi
    """
}

# Generate and run the session script
if $#args > 0 {
    $file = $args[0]
    if ($file | path-exists) {
        $script_content = (create-session-script $file)
        print $script_content
        echo ""
        print $"Run the following in your terminal:"
        print $"  $ (echo $script_content | lines | str replace "''" "") | str replace "''' " "''" ) | into string
        print $"  eval '(echo $script_content | lines | str replace \"'\"'\" \"\"\" '') | str replace \"'\"'\"' ' \"'\"'\"'\"'\"' ' \"'\"')\""
    } else {
        print $"[ERROR] File not found: $file"
        exit 1
    }
} else {
    print "Usage: nu demo/tsv-preview-session.nu <tsv-file>"
    print ""
    print "Or use the bash wrapper:"
    print "  ./demo/tsv-preview-session.sh <tsv-file>"
}
