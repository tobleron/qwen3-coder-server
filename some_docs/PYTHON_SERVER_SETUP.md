# Qwen3-Coder llama-cpp-python Server Setup Guide

This guide walks you through setting up and using the new Python-based Qwen3-Coder server with proper tool calling support.

## Why Python Instead of C++ llama-server?

The C++ `llama-server` outputs tool calls in XML format, but OpenCode expects OpenAI-compatible JSON format. The Python version (`llama-cpp-python`) includes:

1. **Automatic XML → JSON Conversion**: Parses Qwen3's XML tool calls and converts them to OpenAI format
2. **Native Tool Calling Support**: Full OpenAI-compatible `/v1/chat/completions` endpoint
3. **Better Integration**: Works seamlessly with OpenCode without format issues

### Example Format Conversion

```
Model Output (XML):
<tool_call>
  <function=bash>
    <parameter=command>ls -la</parameter>
  </function>
</tool_call>

↓ (Automatic Conversion)

API Response (JSON):
{
  "tool_calls": [{
    "type": "function",
    "function": {
      "name": "bash",
      "arguments": "{\"command\": \"ls -la\"}"
    }
  }]
}
```

## Quick Start

### 1. Start the Server

```bash
cd /home/r2/Desktop/rubox
./start_python_server.sh
```

The script will:
- Check for existing servers and kill them
- Create a Python virtual environment (if needed)
- Install dependencies (including CUDA support if GPU detected)
- Load the model
- Wait for initialization
- Start listening on port 8081

### 2. Verify Server is Running

```bash
curl http://localhost:8081/health
```

Response should be:
```json
{
  "status": "ok",
  "model": "cerebras-qwen3",
  "context_window": 32768,
  "tool_calling_enabled": true
}
```

### 3. Configure OpenCode

In your `~/.config/opencode/opencode.json`:

```json
{
  "provider": {
    "llama_local": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Llama Server Local",
      "options": {
        "baseURL": "http://192.168.1.186:8081/v1",
        "apiKey": "sk-no-key-required"
      },
      "models": {
        "cerebras-qwen3": {
          "name": "Cerebras Qwen3 Coder 25B",
          "modalities": {
            "input": ["text"],
            "output": ["text"]
          },
          "limit": {
            "context": 32768,
            "output": 8192
          }
        }
      }
    }
  }
}
```

### 4. Test with OpenCode

Use OpenCode to analyze code or ask it to execute tools. The tool calls should now work correctly.

## Project Structure

```
/home/r2/Desktop/rubox/
├── python-server/
│   ├── venv/                      # Python virtual environment (auto-created)
│   ├── config.py                  # Server configuration
│   ├── qwen_server.py             # Main server implementation
│   ├── tool_parser.py             # XML → JSON conversion logic
│   ├── test_tool_calling.py       # Test script
│   └── requirements.txt           # Python dependencies
├── start_python_server.sh         # Startup script
├── python_server.log              # Server logs
├── models/
│   └── cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf
└── ...
```

## Configuration

### Model Parameters (in `python-server/config.py`)

```python
N_GPU_LAYERS = 26          # GPU layer offload (adjust based on your VRAM)
N_CTX = 32768              # Context window (32k tokens, reduced from 131k for accuracy)
N_BATCH = 256              # Batch size (reduced from 512 for accuracy)
N_UBATCH = 128             # Micro-batch size (reduced from 512 for accuracy)
TEMPERATURE = 0.7          # Default temperature
MAX_TOKENS = 8192          # Max output tokens
```

### For RTX 3060 (12GB)

The configuration is optimized for RTX 3060:
- **Context**: 32k tokens (good balance of memory and capability)
- **GPU Layers**: 26 out of 49 (most layers on GPU, some on CPU)
- **Batch Size**: Conservative for stability
- **Memory Usage**: ~11.5GB peak

If you have issues:
- **Out of Memory**: Reduce `N_GPU_LAYERS` to 20-24
- **Too Slow**: Increase `N_GPU_LAYERS` to 28-30
- **Inaccurate Tool Calls**: Increase `N_CTX` slowly to 64k max

## Testing

### Run Test Suite

```bash
cd /home/r2/Desktop/rubox/python-server
python3 test_tool_calling.py
```

This tests:
1. Server health check
2. Simple chat (no tools)
3. Tool calling (bash command)
4. Multiple tools

### Manual Testing with curl

Test basic chat:
```bash
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "cerebras-qwen3",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 100
  }'
```

Test with tool calling:
```bash
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "cerebras-qwen3",
    "messages": [{"role": "user", "content": "List the current directory"}],
    "tools": [{
      "type": "function",
      "function": {
        "name": "bash",
        "description": "Execute bash commands",
        "parameters": {
          "type": "object",
          "properties": {
            "command": {"type": "string", "description": "The bash command"}
          },
          "required": ["command"]
        }
      }
    }],
    "max_tokens": 500
  }'
```

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check |
| `/v1/chat/completions` | POST | Chat completions (main endpoint) |
| `/v1/models` | GET | List available models |
| `/` | GET | Server info |

## Troubleshooting

### Server Won't Start

1. Check if port 8081 is already in use:
   ```bash
   lsof -i :8081
   ```

2. Kill any existing process:
   ```bash
   pkill -f "python.*qwen_server.py"
   ```

3. Check logs:
   ```bash
   tail -f python_server.log
   ```

### Out of Memory Errors

The model requires significant VRAM. On RTX 3060 (12GB):

1. Reduce GPU layers:
   ```python
   N_GPU_LAYERS = 20  # Instead of 26
   ```

2. Reduce context window:
   ```python
   N_CTX = 16384  # Instead of 32k
   ```

3. Reduce batch size:
   ```python
   N_BATCH = 128  # Instead of 256
   ```

### Tool Calls Not Working

1. Verify tools are in request (use curl test above)
2. Check server logs: `tail -f python_server.log`
3. Ensure tool definitions have required fields (name, description, parameters)
4. Model may not always call tools - try explicit requests like "Use the bash tool to..."

### Slow Responses

1. Increase GPU layers (if VRAM allows):
   ```python
   N_GPU_LAYERS = 30
   ```

2. Monitor GPU usage:
   ```bash
   nvidia-smi -l 1
   ```

3. Check if system is swapping:
   ```bash
   free -h
   ```

## Migration from C++ llama-server

### Stopping Old Server

```bash
# Kill the old C++ server
pkill -f "llama-server"

# Stop any processes on port 8081
lsof -ti :8081 | xargs kill -9
```

### Starting New Server

```bash
./start_python_server.sh
```

### Updating OpenCode

No changes needed to `opencode.json` - the API is fully compatible. Just update the base URL to point to your new server (if IP changed).

## Performance Characteristics

On RTX 3060 with Cerebras-Qwen3-Coder-25B:

- **Prompt Processing**: ~75-125 tokens/sec
- **Generation**: ~2-3 tokens/sec
- **Model Load Time**: ~30-40 seconds
- **Typical Response Time**: 5-15 seconds for normal queries
- **With Tool Calls**: Same, but finish_reason changes to "tool_calls"

## Advanced Configuration

### CUDA Compilation Flags

If you need custom CUDA compilation:

```bash
LLAMA_CUDA_F16=1 LLAMA_CUDA=1 pip install llama-cpp-python[server]
```

### CPU-Only Mode

To run on CPU (slow, but works):

```bash
pip install llama-cpp-python[server]
# Then in config.py: N_GPU_LAYERS = 0
```

### Custom Model

To use a different model:

```python
# In config.py
MODEL_PATH = ROOT_DIR / "models" / "your-model.gguf"
MODEL_ID = "your-model"
```

## Logs and Debugging

### View Recent Logs

```bash
tail -20 python_server.log
```

### Follow Logs in Real-time

```bash
tail -f python_server.log
```

### Verbose Mode

In `python-server/config.py`, set:

```python
VERBOSE = True
LOG_LEVEL = "DEBUG"
```

Then restart the server for detailed logs.

## Support and Issues

If you encounter issues:

1. Check logs: `tail -f python_server.log`
2. Test with curl: See testing section above
3. Verify model file exists: `ls -lh models/cerebras_Qwen3-Coder-*`
4. Check VRAM: `nvidia-smi`
5. Test with Python script: `python3 test_tool_calling.py`

## Related Files

- **Original C++ Server**: `start_qwen_server.sh` (deprecated)
- **Rust App**: `apps/rubox/` (still uses original server config)
- **Config**: `config/server/cerebras-qwen3.json` (C++ server config)

## Next Steps

1. ✅ Run `./start_python_server.sh`
2. ✅ Verify with `curl http://localhost:8081/health`
3. ✅ Test tool calling: `python3 python-server/test_tool_calling.py`
4. ✅ Use in OpenCode - tool calls should now work!

---

**Version**: 1.0.0
**Last Updated**: February 2026
**Model**: Cerebras-Qwen3-Coder-REAP-25B-A3B
