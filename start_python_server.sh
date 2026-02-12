#!/bin/bash

# Qwen3-Coder llama-cpp-python Server Launcher
# Replaces start_qwen_server.sh with Python-based server
# Provides proper tool calling support with XML-to-JSON conversion

set -e

# Colors for output
ORANGE='\033[38;5;208m'
GREEN='\033[38;5;48m'
RED='\033[38;5;196m'
RESET='\033[0m'

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_SERVER_DIR="$SCRIPT_DIR/python-server"
VENV_DIR="$PYTHON_SERVER_DIR/venv"
LOG_FILE="$SCRIPT_DIR/python_server.log"

echo ""
echo -e "${ORANGE}════════════════════════════════════════════════════════${RESET}"
echo -e "${ORANGE}   Qwen3-Coder llama-cpp-python Server${RESET}"
echo -e "${ORANGE}════════════════════════════════════════════════════════${RESET}"
echo ""

# Check for existing servers
echo -e "${ORANGE}🔍 Checking for existing server instances...${RESET}"
if pgrep -f "python.*qwen_server.py" > /dev/null 2>&1; then
    echo -e "${RED}⚠️  Found running server instance(s). Killing them...${RESET}"
    pkill -f "python.*qwen_server.py" || true
    sleep 2
    echo -e "${GREEN}✓ Existing instances terminated${RESET}"
else
    echo -e "${GREEN}✓ No running instances found${RESET}"
fi

echo ""

# Check Python version
echo -e "${ORANGE}🐍 Checking Python installation...${RESET}"
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}✗ Python 3 not found. Please install Python 3.9+${RESET}"
    exit 1
fi

PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
echo -e "${GREEN}✓ Found Python $PYTHON_VERSION${RESET}"

echo ""

# Create virtual environment if it doesn't exist
if [ ! -d "$VENV_DIR" ]; then
    echo -e "${ORANGE}📦 Creating Python virtual environment...${RESET}"
    python3 -m venv "$VENV_DIR"
    echo -e "${GREEN}✓ Virtual environment created${RESET}"
else
    echo -e "${GREEN}✓ Virtual environment exists${RESET}"
fi

echo ""

# Activate virtual environment
echo -e "${ORANGE}🔌 Activating virtual environment...${RESET}"
source "$VENV_DIR/bin/activate"
echo -e "${GREEN}✓ Virtual environment activated${RESET}"

echo ""

# Install/upgrade dependencies
echo -e "${ORANGE}📚 Installing dependencies...${RESET}"
pip install -q --upgrade pip setuptools wheel 2>/dev/null || true

# Use pinned versions for deterministic behavior across restarts.
if command -v nvidia-smi &> /dev/null; then
    echo -e "${GREEN}✓ NVIDIA GPU detected${RESET}"
else
    echo -e "${ORANGE}ℹ️  No NVIDIA GPU detected - CPU fallback expected${RESET}"
fi
pip install -q -r "$PYTHON_SERVER_DIR/requirements.txt" 2>/dev/null
echo -e "${GREEN}✓ Dependencies installed${RESET}"

echo ""

# Check model file
echo -e "${ORANGE}🔎 Checking model file...${RESET}"
MODEL_PATH="$SCRIPT_DIR/models/cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf"
if [ ! -f "$MODEL_PATH" ]; then
    echo -e "${RED}✗ Model not found at: $MODEL_PATH${RESET}"
    echo -e "${RED}Please ensure the model file exists before starting the server.${RESET}"
    exit 1
fi
MODEL_SIZE=$(du -h "$MODEL_PATH" | cut -f1)
echo -e "${GREEN}✓ Model found ($MODEL_SIZE)${RESET}"

echo ""

# Start the server
echo -e "${ORANGE}🚀 Starting Qwen3-Coder server...${RESET}"
echo -e "${ORANGE}   Model: Cerebras-Qwen3-Coder-25B-A3B-Q4${RESET}"
echo -e "${ORANGE}   Port: 8081${RESET}"
echo -e "${ORANGE}   Context: 131,072 tokens${RESET}"
echo -e "${ORANGE}   Tool Calling: ENABLED (XML→JSON conversion)${RESET}"
echo ""

# Start server in background with nohup
nohup python3 -u "$PYTHON_SERVER_DIR/qwen_server.py" > "$LOG_FILE" 2>&1 &
SERVER_PID=$!

echo -e "${GREEN}✓ Server started with PID: $SERVER_PID${RESET}"
echo ""

# Wait for server to be ready
echo -e "${ORANGE}⏳ Waiting for server to initialize (this may take 1-2 minutes)...${RESET}"

MAX_RETRIES=120
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s http://localhost:8081/health > /dev/null 2>&1; then
        echo ""
        echo -e "${GREEN}✓ Server is ready!${RESET}"
        break
    fi

    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $((RETRY_COUNT % 10)) -eq 0 ]; then
        echo -n "."
    fi
    sleep 0.5
done

if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
    echo ""
    echo -e "${RED}✗ Server failed to start within timeout${RESET}"
    echo -e "${RED}Check logs: tail -f $LOG_FILE${RESET}"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo ""
echo -e "${GREEN}════════════════════════════════════════════════════════${RESET}"
echo -e "${GREEN}Server Running Successfully!${RESET}"
echo -e "${GREEN}════════════════════════════════════════════════════════${RESET}"
echo ""
echo -e "📝 Log file: $LOG_FILE"
echo -e "   View logs with: ${ORANGE}tail -f $LOG_FILE${RESET}"
echo ""
echo -e "🌐 API Endpoints:"
echo -e "   Health: ${ORANGE}http://localhost:8081/health${RESET}"
echo -e "   Chat: ${ORANGE}http://localhost:8081/v1/chat/completions${RESET}"
echo -e "   Models: ${ORANGE}http://localhost:8081/v1/models${RESET}"
echo ""
echo -e "🖥️  OpenCode Configuration (Mac):"
echo -e "   Base URL: ${ORANGE}http://$(hostname -I | awk '{print $1}'):8081/v1${RESET}"
echo -e "   Model: ${ORANGE}cerebras-qwen3${RESET}"
echo ""
echo -e "🧪 Test with curl:"
echo -e "   ${ORANGE}curl http://localhost:8081/health${RESET}"
echo ""
echo -e "✋ To stop server:"
echo -e "   ${ORANGE}pkill -f 'python.*qwen_server.py'${RESET}"
echo ""
