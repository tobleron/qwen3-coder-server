# Troubleshooting Guide

Common issues and solutions.

## Server Issues

### Server Won't Start

**Error:** `No module named 'llama_cpp'` or similar

**Solution:**
```bash
# Kill any existing processes
pkill -f "qwen_server.py"

# Make sure script is executable
chmod +x start_python_server.sh

# Run again
./start_python_server.sh
```

If that doesn't work:
```bash
# Check logs
tail -50 python_server.log

# Try running manually
cd python-server
source venv/bin/activate
python3 qwen_server.py
```

---

### CUDA Out of Memory (OOM)

**Error:** `CUDA out of memory` or `RuntimeError: out of memory`

**Cause:** Model or KV cache too large for GPU

**Solution 1: Reduce GPU Layers (fastest)**

Edit `python-server/config.py`:
```python
N_GPU_LAYERS = 20  # Was 26
```

Restart: `./start_python_server.sh`

**Solution 2: Reduce Context Window**

```python
N_CTX = 65536  # Was 131072 (use 64k instead of 131k)
```

**Solution 3: Both Together**

```python
N_GPU_LAYERS = 22
N_CTX = 65536
```

Check GPU memory before:
```bash
nvidia-smi
# Look for "Free" memory in your GPU row
```

---

### Model File Not Found

**Error:** `FileNotFoundError: Model not found at /path/to/models/cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf`

**Solution:**

1. Check file exists:
```bash
ls -lh models/
# Should show: cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf (14.1 GB)
```

2. If missing, download it:
```bash
huggingface-cli download cerebras/Qwen3-Coder-REAP-25B-A3B \
  cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf \
  --local-dir ./models \
  --local-dir-use-symlinks False
```

3. Restart server:
```bash
./start_python_server.sh
```

---

### Port 8081 Already in Use

**Error:** `Address already in use` or `port 8081 is already allocated`

**Solution:**

1. Kill existing process:
```bash
pkill -f "qwen_server.py"
sleep 2
```

2. Or find what's using it:
```bash
lsof -ti:8081 | xargs kill -9
```

3. Start server:
```bash
./start_python_server.sh
```

**Alternative:** Use different port

Edit `python-server/config.py`:
```python
SERVER_PORT = 8082  # Or any other unused port
```

Then restart and update client config.

---

### Server Timeout / Health Check Fails

**Error:** `Server health check failed` or `Server is not ready`

**Cause:** Model loading taking too long

**Solution:**

1. Check logs while server is starting:
```bash
tail -f python_server.log
```

2. Look for lines like:
```
load_tensors: loading model tensors
```

3. Wait longer (can take 60+ seconds on first run)

4. If still fails after 2 minutes, check GPU:
```bash
nvidia-smi
# Check if GPU memory is being used
```

---

## API & Tool Calling Issues

### Tool Calls Not Being Executed

**Problem:** Model returns text description instead of actual tool calls

**Cause 1: Tools Not Provided**

Check request includes `tools` array:
```json
{
  "messages": [...],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "bash",
        "description": "Execute bash commands",
        "parameters": {...}
      }
    }
  ]
}
```

**Cause 2: Model Not Calling Tools**

Try explicit request:
```
"List the files using the bash tool"
```

vs.

```
"List the files"
```

**Cause 3: Malformed Tool Definition**

Ensure tool has:
- ✅ `type: "function"`
- ✅ `function.name: string`
- ✅ `function.description: string`
- ✅ `function.parameters: object`
- ✅ `function.parameters.properties: object`
- ✅ `function.parameters.required: array`

---

### Tool Call Response Parsing Error

**Error:** `FAILURE: 'todos' is string` or malformed arguments

**Cause:** Tool arguments not properly JSON-encoded

**Fix in your client:**

```python
# WRONG
tool_calls = {
    "arguments": {"key": "value"}  # Plain dict
}

# CORRECT
import json
tool_calls = {
    "arguments": json.dumps({"key": "value"})  # JSON string
}
```

---

### OpenCode Integration Not Working

**Error:** OpenCode doesn't connect or tool calls fail

**Check 1: Server is Running**
```bash
curl http://localhost:8081/health
# Should return 200 OK
```

**Check 2: OpenCode Config**

File: `~/.config/opencode/opencode.json`

```json
{
  "provider": {
    "llama_local": {
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": "http://192.168.1.YOUR_IP:8081/v1",
        "apiKey": "sk-no-key-required"
      },
      "models": {
        "cerebras-qwen3": {...}
      }
    }
  }
}
```

**Check 3: Correct IP Address**

Find your server's IP:
```bash
hostname -I
# Or
ifconfig | grep "inet " | grep -v 127.0.0.1
```

Use the **non-loopback** IP (e.g., `192.168.1.186`, not `127.0.0.1`)

**Check 4: Network Connectivity**

From OpenCode machine:
```bash
curl http://192.168.1.YOUR_IP:8081/health
# Should work
```

If not:
- Check firewall
- Check both machines on same network
- Try `ping 192.168.1.YOUR_IP`

---

## Performance Issues

### Server Very Slow

**Problem:** Responses take 30+ seconds

**Cause 1: CPU Only (No GPU Acceleration)**

Check:
```bash
nvidia-smi
# If your GPU doesn't appear, GPU layers aren't working
```

Fix:
```python
# Verify in config.py
N_GPU_LAYERS = 26  # Should be > 0
```

Then check logs for:
```
load_tensors: layer XX assigned to device CUDA0
```

**Cause 2: Swapping to Disk**

Check system RAM:
```bash
free -h
# If "Swap" is being used, you're out of RAM
```

Solution:
```python
N_GPU_LAYERS = 26  # Push more to GPU
N_CTX = 65536      # Reduce context
N_BATCH = 256      # Already reduced
```

**Cause 3: Large Context Window**

If using N_CTX = 131072 and slow:
```python
N_CTX = 65536  # Halve context
```

---

### High Memory Usage

**Problem:** Using >15GB RAM + swap

**Cause:** Large context + KV cache quantization

**Solution:**

```python
# Current setup
N_CTX = 131072  # 131k context = 3.4GB KV cache

# Reduce to
N_CTX = 65536   # 64k context = 1.7GB KV cache
```

Check memory:
```bash
watch -n1 nvidia-smi
# Watch VRAM before/after change
```

---

## Logging & Debugging

### Enable Debug Logging

Edit `python-server/config.py`:
```python
LOG_LEVEL = "DEBUG"  # Was "INFO"
VERBOSE = True       # Was False
```

Restart and check logs:
```bash
tail -f python_server.log
# Much more detailed output
```

### Check Server Logs

```bash
# Last 50 lines
tail -50 python_server.log

# Last 100 lines with timestamps
tail -100 python_server.log | head -20

# Follow in real-time
tail -f python_server.log
```

### Clear Old Logs

```bash
# Backup old logs
mv python_server.log python_server.log.bak

# Restart server (creates new log)
./start_python_server.sh
```

---

## Model Behavior Issues

### Model Repeating Same Response

**Problem:** Model generates same text over and over

**Cause:** Repeat penalty too low

**Fix:**

```python
REPEAT_PENALTY = 1.15  # Increase from 1.1
```

Or:
```python
REPEAT_PENALTY = 1.2   # Even higher
```

Restart and test:
```bash
pkill -f "qwen_server.py"
./start_python_server.sh
```

---

### Model Always Calling Tools

**Problem:** Model calls tools even when not requested

**Cause:** Temperature too high or request ambiguous

**Fix:**

```python
TEMPERATURE = 0.5  # Lower from 0.7 (more deterministic)
```

Or specify in request:
```json
{
  "temperature": 0.3,
  "messages": [...]
}
```

---

### Model Responses Not Deterministic

**Problem:** Same question gives different answers

**Normal behavior!** But if you want consistency:

```python
TEMPERATURE = 0.2  # Lower = more deterministic
TOP_P = 0.7        # Lower = narrower choices
```

For absolute determinism:
```python
TEMPERATURE = 0.0  # Always same response
```

---

## Getting Help

### Before Reporting Issue

1. Check logs:
```bash
tail -100 python_server.log
```

2. Run test suite:
```bash
cd python-server
python3 test_tool_calling.py
```

3. Verify health:
```bash
curl http://localhost:8081/health
```

4. Check basic request:
```bash
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"cerebras-qwen3","messages":[{"role":"user","content":"test"}]}'
```

### Include in Bug Report

- Full error message and stack trace
- Last 50 lines of logs
- Your hardware (GPU, VRAM, CPU)
- Python version: `python3 --version`
- Exact command that fails
- Steps to reproduce

### Resources

- **Model Info:** https://huggingface.co/cerebras/Qwen3-Coder-REAP-25B-A3B
- **llama.cpp Issues:** https://github.com/ggerganov/llama.cpp/issues
- **llama-cpp-python:** https://github.com/abetlen/llama-cpp-python
- **OpenCode:** https://github.com/anomalyco/opencode

---

**Last Updated:** February 2026
