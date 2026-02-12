#!/bin/bash

# Path to llama-server
LLAMA_SERVER="./tools/llama.cpp/build/bin/llama-server"
CONFIG_DIR="./config/server"

# Check if llama-server exists
if [ ! -f "$LLAMA_SERVER" ]; then
    echo "Error: llama-server not found at $LLAMA_SERVER"
    exit 1
fi

# Load available configs
configs=("$CONFIG_DIR"/*.json)
if [ ! -e "${configs[0]}" ]; then
    echo "Error: No configuration files found in $CONFIG_DIR"
    exit 1
fi

echo "Select a model to run:"
for i in "${!configs[@]}"; do
    name=$(jq -r '.name' "${configs[$i]}")
    echo "$((i+1))) $name"
done
echo "q) Quit"

read -p "Choice [1-${#configs[@]}]: " choice

if [[ "$choice" == "q" || "$choice" == "Q" ]]; then
    echo "Exiting."
    exit 0
fi

# Validate choice
if ! [[ "$choice" =~ ^[0-9]+$ ]] || [ "$choice" -lt 1 ] || [ "$choice" -gt "${#configs[@]}" ]; then
    echo "Invalid choice."
    exit 1
fi

selected_config="${configs[$((choice-1))]}"
MODEL_PATH=$(jq -r '.model_path' "$selected_config")
# Read args into an array safely
mapfile -t ARGS < <(jq -r '.args[]' "$selected_config")

# Check if model exists
if [ ! -f "$MODEL_PATH" ]; then
    echo "Error: Model not found at $MODEL_PATH"
    exit 1
fi

echo "Starting llama-server for $(jq -r '.name' "$selected_config")..."
"$LLAMA_SERVER" -m "$MODEL_PATH" "${ARGS[@]}"
