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

function detect_running_server() {
    if pgrep -f "python.*glm_server.py" > /dev/null 2>&1; then
        echo "glm"
    elif pgrep -f "python.*qwen_server.py" > /dev/null 2>&1; then
        echo "qwen"
    else
        echo ""
    fi
}

function server_label() {
    case "$1" in
        glm)
            echo "GLM-4.7-Flash"
            ;;
        qwen)
            echo "Qwen3-Coder"
            ;;
        *)
            echo "Unknown"
            ;;
    esac
}

function stop_server() {
    local target="$1"
    local pattern
    local label

    case "$target" in
        glm)
            pattern="python.*glm_server.py"
            label="GLM-4.7-Flash"
            ;;
        qwen)
            pattern="python.*qwen_server.py"
            label="Qwen3-Coder"
            ;;
        *)
            return
            ;;
    esac

    if pgrep -f "$pattern" > /dev/null 2>&1; then
        echo "Stopping running ${label} server..."
        pkill -f "$pattern" || true
        local countdown=0
        while pgrep -f "$pattern" > /dev/null 2>&1; do
            sleep 0.5
            countdown=$((countdown + 1))
            if (( countdown % 6 == 0 )); then
                echo "Waiting for ${label} server to exit..."
            fi
        done
        echo "${label} server stopped."
    fi
}

function prompt_yes_no() {
    local prompt="$1"
    local response
    read -rp "${prompt} [y/N]: " response
    case "${response,,}" in
        y|yes)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

function handle_selection() {
    local target="$1"
    local running

    running="$(detect_running_server)"
    local target_label
    target_label="$(server_label "$target")"

    if [[ -n "$running" ]]; then
        local running_label
        running_label="$(server_label "$running")"
        if [[ "$running" == "$target" ]]; then
            if prompt_yes_no "The ${target_label} server is already running. Restart it?"; then
                stop_server "$target"
            else
                echo "Keeping running server. No changes made."
                exit 0
            fi
        else
            if prompt_yes_no "The ${running_label} server is running. Stop it and launch ${target_label}?"; then
                stop_server "$running"
            else
                echo "Aborting switch. ${running_label} server remains running."
                exit 0
            fi
        fi
    fi

    if [[ "$target" == "qwen" ]]; then
        bash "$SCRIPT_DIR/start_python_server.sh"
    else
        bash "$SCRIPT_DIR/start_glm_server.sh"
    fi
}

while true; do
    show_menu
    read -rp "Select option [1-3]: " choice
    case "$choice" in
        1)
            handle_selection "qwen"
            break
            ;;
        2)
            handle_selection "glm"
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
