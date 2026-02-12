# Qwen3-Coder Server

A production-ready OpenAI-compatible API server for the **Cerebras-Qwen3-Coder-REAP-25B** model running on consumer GPUs (RTX 3060+) via `llama-cpp-python`.

## Features

✅ **OpenAI-Compatible API** — Drop-in replacement for OpenAI chat completions endpoint
✅ **Tool/Function Calling** — Full support for tool definitions with automatic XML→JSON conversion
✅ **Streaming Responses** — Real-time token streaming with proper SSE format
✅ **Optimized for Q4_K_M Quantization** — 14GB GGUF loads in ~12GB VRAM on RTX 3060
✅ **131k Token Context** — Extended context window for large files/repositories
✅ **Production Logging** — Comprehensive error tracking and debugging

## What's Included

- **FastAPI Server** (`python-server/qwen_server.py`) — OpenAI-compatible `/v1/chat/completions` endpoint
- **Tool Parser** (`python-server/tool_parser.py`) — Converts Qwen3's XML tool calls to OpenAI JSON format
- **Configuration** (`python-server/config.py`) — Centralized settings (context, GPU layers, penalties)
- **Launcher Script** (`start_python_server.sh`) — Automated setup, venv management, health checks
- **Test Suite** (`python-server/test_tool_calling.py`) — Validates server and tool call parsing

## What's NOT Included

❌ The **14GB model file** (you download separately)
❌ `llama-cpp-python` source (installed via pip)
❌ Build artifacts or logs

## Quick Start

### 1. Prerequisites

- **Python 3.9+**
- **NVIDIA GPU** with 12GB+ VRAM (RTX 3060+)
- **CUDA Toolkit** compatible with your GPU
- **~15GB disk space** for the model file

### 2. Download Model

Download the quantized model from HuggingFace:

```bash
# Create models directory
mkdir -p /path/to/this/repo/models

# Download model (14.1 GB)
wget https://huggingface.co/cerebras/Qwen3-Coder-REAP-25B-A3B/resolve/main/cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf \
  -O /path/to/this/repo/models/cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf
```

Or use `huggingface-cli`:

```bash
huggingface-cli download cerebras/Qwen3-Coder-REAP-25B-A3B \
  cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf \
  --local-dir ./models \
  --local-dir-use-symlinks False
```

### 3. Install & Start Server

```bash
cd /path/to/this/repo

# Make script executable
chmod +x start_python_server.sh

# Start server (handles venv, dependencies, model loading)
./start_python_server.sh
```

The server will:
- Create a Python virtual environment
- Install dependencies from `requirements.txt`
- Load the model (takes 30-40 seconds on first run)
- Start listening on `http://0.0.0.0:8081/v1`

### 4. Test Server

```bash
# In another terminal, test health endpoint
curl http://localhost:8081/health

# Expected response:
# {"status":"ok","model":"cerebras-qwen3","context_window":131072,"tool_calling_enabled":true}
```

### 5. Run Test Suite

```bash
cd python-server
python3 test_tool_calling.py
```

Should show:
```
Health Check
  ✓ Server is healthy

Simple Chat (No Tools)
  ✓ Response: ...

Tool Calling
  ✓ Model called 1 tool(s)

Multiple Tools
  ✓ Model called 1 tool(s)
```

## Configuration

Edit `python-server/config.py` to adjust:

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `N_GPU_LAYERS` | 26 | GPU layer offload (increase for speed, decrease for RAM) |
| `N_CTX` | 131072 | Context window (131k tokens = ~500k characters) |
| `TEMPERATURE` | 0.7 | Response randomness (0.0=deterministic, 2.0=chaotic) |
| `REPEAT_PENALTY` | 1.1 | Prevents runaway repetition |
| `TOP_P` | 0.8 | Nucleus sampling diversity |
| `TOP_K` | 20 | Top-K sampling |
| `MAX_TOKENS` | 16384 | Maximum output length per request |

### RTX 3060 (12GB) Tuning

**Default settings** use ~11.5GB peak VRAM:
- Model weights: 7.6 GB
- KV cache (131k context, Q4_0): 3.4 GB
- Other: 0.5 GB

**If OOM errors:**
```python
N_GPU_LAYERS = 20  # Instead of 26
N_CTX = 65536      # Instead of 131072
```

**For faster responses (if VRAM allows):**
```python
N_GPU_LAYERS = 28
```

## OpenCode Integration

Configure OpenCode CLI to use this server:

**File:** `~/.config/opencode/opencode.json`

```json
{
  "provider": {
    "llama_local": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Qwen3-Coder Local",
      "options": {
        "baseURL": "http://192.168.1.186:8081/v1",
        "apiKey": "sk-no-key-required"
      },
      "models": {
        "cerebras-qwen3": {
          "name": "Cerebras Qwen3 Coder 25B",
          "tool_call": true,
          "reasoning": true,
          "limit": {
            "context": 131072,
            "output": 16384
          }
        }
      }
    }
  }
}
```

**Note:** Replace `192.168.1.186` with your server's IP address.

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Server health check |
| `/v1/models` | GET | List available models |
| `/v1/chat/completions` | POST | Chat completions (main endpoint) |
| `/` | GET | Server info |

See [API.md](API.md) for detailed examples.

## Tool Calling

The server automatically converts Qwen3's native XML tool calls to OpenAI-compatible JSON format.

**Model Output (XML):**
```xml
<tool_call>
<function=bash>
<parameter=command>ls -la</parameter>
</function>
</tool_call>
```

**API Response (JSON):**
```json
{
  "finish_reason": "tool_calls",
  "message": {
    "content": null,
    "tool_calls": [{
      "id": "call_abc123",
      "type": "function",
      "function": {
        "name": "bash",
        "arguments": "{\"command\": \"ls -la\"}"
      }
    }]
  }
}
```

## System Requirements

### Hardware
- **GPU:** NVIDIA RTX 3060 (12GB) or better
- **CPU:** Modern processor (Ryzen 5+ / i7+)
- **RAM:** 16GB+ system RAM
- **Storage:** ~20GB (model + OS + dependencies)

### Software
- **Python:** 3.9, 3.10, 3.11, 3.12 (tested with 3.11)
- **CUDA:** 11.8+ or 12.x
- **cuDNN:** Compatible with your CUDA version

## Performance

On RTX 3060 with settings above:

| Metric | Value |
|--------|-------|
| Model load time | 30-40 seconds (first run only) |
| Prompt processing | 75-125 tokens/sec |
| Generation speed | 2-3 tokens/sec |
| Typical response | 5-15 seconds |
| Peak VRAM | 11.5 GB |

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues and solutions.

## Documentation

- **[SETUP.md](SETUP.md)** — Detailed installation and configuration
- **[API.md](API.md)** — API endpoints and examples
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** — Common issues and fixes

## Model Information

**Model:** Cerebras-Qwen3-Coder-REAP-25B-A3B
**Base Model:** Qwen3-Coder-30B-A3B-Instruct
**Quantization:** Q4_K_M (4-bit, Medium)
**File Size:** 14.1 GB
**Context:** 262,144 tokens (131,072 used by default)
**Architecture:** Mixture of Experts (103 experts, 8 active)
**Parameters:** 24.87 Billion

**Source:** https://huggingface.co/cerebras/Qwen3-Coder-REAP-25B-A3B

## Tool Versions

| Tool | Version | Purpose |
|------|---------|---------|
| llama-cpp-python | 0.3.0 | Inference engine & OpenAI-compatible server |
| llama.cpp | 9ac2693a3 | Core inference backend (commit hash) |
| FastAPI | 0.104.1 | Web framework |
| Uvicorn | 0.24.0 | ASGI server |
| Pydantic | 2.5.0 | Data validation |

See [requirements.txt](requirements.txt) for complete dependency list with pinned versions.

## License

This project is licensed under Apache 2.0 (matching the Qwen3-Coder model license).

## Support

- **Issues:** Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- **Model Info:** https://huggingface.co/cerebras/Qwen3-Coder-REAP-25B-A3B
- **llama.cpp:** https://github.com/ggerganov/llama.cpp

## Contributing

Feel free to submit issues and improvements!

---

**Last Updated:** February 2026
**Status:** Production Ready ✅
