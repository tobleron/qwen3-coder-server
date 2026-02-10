# Installation & Setup Guide

Complete step-by-step guide to set up the Qwen3-Coder Server from scratch.

## Prerequisites Check

### Step 1: Verify Python Installation

```bash
python3 --version
# Should be 3.9 or higher
```

If Python 3 is not installed:
- **Ubuntu/Debian:** `sudo apt-get install python3 python3-pip python3-venv`
- **macOS:** `brew install python3`
- **Windows:** Download from https://www.python.org/

### Step 2: Verify NVIDIA GPU & CUDA

```bash
nvidia-smi
```

Should output GPU info and CUDA version. If not found:
- Install NVIDIA GPU driver from https://www.nvidia.com/Download/driverDetails.aspx
- Install CUDA Toolkit matching your GPU from https://developer.nvidia.com/cuda-downloads

**Tested with:**
- CUDA 12.1
- cuDNN 8.9+
- NVIDIA Driver 535+

### Step 3: Check Disk Space

```bash
# You need ~20GB free
df -h /path/to/where/you/want/repo
```

## Installation Steps

### Step 1: Clone the Repository

```bash
git clone https://github.com/yourusername/qwen3-coder-server.git
cd qwen3-coder-server
```

### Step 2: Download the Model

The model is **not** included in the repository (14.1 GB). Download it:

**Option A: Using `huggingface-cli` (recommended)**

```bash
# Install huggingface-cli if needed
pip install huggingface-hub

# Download model
huggingface-cli download cerebras/Qwen3-Coder-REAP-25B-A3B \
  cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf \
  --local-dir ./models \
  --local-dir-use-symlinks False
```

**Option B: Using wget**

```bash
mkdir -p models
wget https://huggingface.co/cerebras/Qwen3-Coder-REAP-25B-A3B/resolve/main/cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf \
  -O models/cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf

# Verify download (should be ~14.1 GB)
ls -lh models/
```

**Option C: Manual Download**

1. Go to https://huggingface.co/cerebras/Qwen3-Coder-REAP-25B-A3B
2. Click on "cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf"
3. Click "Download"
4. Move file to `./models/` directory

### Step 3: Make Script Executable

```bash
chmod +x start_python_server.sh
```

### Step 4: Start the Server

```bash
./start_python_server.sh
```

This script will:
1. Check for existing Python installations
2. Create virtual environment (`python-server/venv/`)
3. Install dependencies from `requirements.txt`
4. Load the model (30-40 seconds)
5. Start server on port 8081

**Expected output:**
```
════════════════════════════════════════════════════════
   Qwen3-Coder llama-cpp-python Server
════════════════════════════════════════════════════════

✓ No running instances found
✓ Found Python 3.11.x
✓ Virtual environment created
✓ Virtual environment activated
✓ Dependencies installed
✓ Model found (14.1G)
🚀 Starting Qwen3-Coder server...
⏳ Waiting for server to initialize...
✓ Server is ready!

════════════════════════════════════════════════════════
Server Running Successfully!
════════════════════════════════════════════════════════

📝 Log file: python_server.log
🌐 API Endpoints:
   Health: http://localhost:8081/health
   Chat: http://localhost:8081/v1/chat/completions
   Models: http://localhost:8081/v1/models
```

## Verification

### Test 1: Health Check

```bash
curl http://localhost:8081/health
```

Expected response:
```json
{
  "status": "ok",
  "model": "cerebras-qwen3",
  "context_window": 131072,
  "tool_calling_enabled": true
}
```

### Test 2: Run Test Suite

```bash
cd python-server
python3 test_tool_calling.py
```

Expected output (all tests PASS):
```
Health Check
  ✓ Server is healthy
  ℹ Model: cerebras-qwen3
  ℹ Context Window: 131072 tokens
  ℹ Tool Calling: ENABLED

Simple Chat (No Tools)
  ✓ Response: Hello! How can I...

Tool Calling
  ✓ Model called 1 tool(s)

Multiple Tools
  ✓ Model called 1 tool(s)

Results: 3/3 tests passed
```

### Test 3: Simple Chat Request

```bash
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "cerebras-qwen3",
    "messages": [{"role": "user", "content": "Hello! What is 2+2?"}],
    "max_tokens": 100
  }'
```

Expected response:
```json
{
  "id": "chatcmpl-qwen3",
  "object": "chat.completion",
  "created": 1707531234,
  "model": "cerebras-qwen3",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "2+2 equals 4."
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 12,
    "completion_tokens": 5,
    "total_tokens": 17
  }
}
```

## Configuration

### Adjust Model Parameters

Edit `python-server/config.py`:

```python
# Context window (default 131072 = 131k tokens)
N_CTX = 131072

# GPU layers to offload (26 for RTX 3060)
N_GPU_LAYERS = 26

# Temperature (0.0=deterministic, 2.0=chaotic)
TEMPERATURE = 0.7

# Repetition penalty (prevent loops)
REPEAT_PENALTY = 1.1
```

Then restart the server:
```bash
pkill -f "qwen_server.py"
./start_python_server.sh
```

### RTX 3060 Out of Memory?

If you see CUDA OOM errors:

```python
# Option 1: Reduce GPU layers
N_GPU_LAYERS = 20  # Default 26

# Option 2: Reduce context window
N_CTX = 65536  # Default 131072 (131k)

# Option 3: Both
N_GPU_LAYERS = 22
N_CTX = 65536
```

## Stopping the Server

### Option 1: Kill Process

```bash
pkill -f "qwen_server.py"
```

### Option 2: Ctrl+C in Terminal

If running in foreground, press `Ctrl+C`.

## Updating

To update to the latest code:

```bash
cd /path/to/repo
git pull origin main
./start_python_server.sh
```

The script will reinstall dependencies if requirements.txt changed.

## Directory Structure

```
qwen3-coder-server/
├── README.md                          # Overview & quick start
├── SETUP.md                           # This file
├── API.md                             # API documentation
├── TROUBLESHOOTING.md                 # Common issues
├── requirements.txt                   # Python dependencies (pinned)
├── start_python_server.sh             # Server launcher script
│
├── python-server/
│   ├── config.py                      # Configuration & parameters
│   ├── qwen_server.py                 # FastAPI server implementation
│   ├── tool_parser.py                 # XML→JSON tool call converter
│   ├── test_tool_calling.py           # Test suite
│   ├── requirements-dev.txt           # Development dependencies
│   └── venv/                          # Python virtual environment (auto-created)
│
├── config/
│   └── server/
│       └── cerebras-qwen3.json        # Reference configuration
│
├── models/
│   └── cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf  # (you download this)
│
└── .gitignore                         # Git ignore rules
```

## Environment Variables (Optional)

You can override config with environment variables:

```bash
# Set context window
export RUBOX_N_CTX=65536

# Set GPU layers
export RUBOX_N_GPU_LAYERS=20

# Set log level
export RUBOX_LOG_LEVEL=DEBUG

./start_python_server.sh
```

## Next Steps

1. ✅ Verify server is running: `curl http://localhost:8081/health`
2. ✅ Run test suite: `cd python-server && python3 test_tool_calling.py`
3. ✅ Check [API.md](API.md) for endpoint documentation
4. ✅ Configure OpenCode or other client to use `http://localhost:8081/v1`

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for:
- Server won't start
- CUDA out of memory errors
- Tool calls not working
- Slow responses
- And more...

## Support

- Check logs: `tail -f python_server.log`
- Test suite: `cd python-server && python3 test_tool_calling.py`
- API docs: See [API.md](API.md)

---

**Last Updated:** February 2026
