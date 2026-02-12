#!/bin/bash

# Lightweight selector for the Python-based servers
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

function show_menu() {
    echo ""
    echo "Choose which server to launch:"
    echo "  1) Qwen3-Coder (python-server/qwen_server.py)"
    echo "  2) GLM-4.7-Flash (python-server/glm_server.py)"
    echo "  3) Exit"
    echo ""
}

while true; do
    show_menu
    read -rp "Select option [1-3]: " choice
    case "$choice" in
        1)
            bash "$SCRIPT_DIR/start_python_server.sh"
            break
            ;;
        2)
            bash "$SCRIPT_DIR/start_glm_server.sh"
            break
            ;;
        3)
            echo "Abort requested."
            exit 0
            ;;
        *)
            echo "Invalid selection; please choose 1, 2, or 3."
            ;;
    esac
done
