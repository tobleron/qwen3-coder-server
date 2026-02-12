#!/bin/bash

# Qwen Model Server Launcher with Safety Kill
# This script safely starts the Cerebras-Qwen3 llama-server
# It kills any existing llama-server instances before starting a new one

set -e

echo "🔍 Checking for existing llama-server instances..."

# Kill any running llama-server processes
if pgrep -f "llama-server" > /dev/null; then
    echo "⚠️  Found running llama-server instance(s). Killing them..."
    pkill -f "llama-server" || true
    sleep 2
    echo "✓ Existing instances terminated"
else
    echo "✓ No running instances found"
fi

echo ""
echo "🚀 Starting Cerebras-Qwen3 (25B) llama-server (Optimized)..."
echo "   Model: models/cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf"
echo "   Port: 8081"
echo "   Context: 131,072 tokens"
echo ""

# Start the server in background with nohup
nohup ./tools/llama.cpp/build/bin/llama-server \
  -m models/cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf \
  -ngl 26 \
  -c 131072 \
  -fa on \
  -ctk q4_0 \
  -ctv q4_0 \
  --jinja \
  -t 8 \
  -b 512 \
  -ub 512 \
  --no-mmap \
  --mlock \
  --host 0.0.0.0 \
  --port 8081 \
  --temp 0.2 \
  --top-p 0.8 \
  --top-k 20 \
  --repeat-penalty 1.1 \
  --presence-penalty 0.0 \
  --frequency-penalty 0.0 \
  --slot-save-state \
  --verbose-prompt -v > server.log 2>&1 &

# Capture PID
SERVER_PID=$!
echo "✓ Server started with PID: $SERVER_PID"
echo ""
echo "📝 Log file: server.log"
echo "   View logs with: tail -f server.log"
echo ""
echo "To connect from OpenCode on Mac:"
echo "   Base URL: http://<YOUR_DESKTOP_IP>:8081/v1"
echo "   Model: cerebras-qwen3"
echo ""
