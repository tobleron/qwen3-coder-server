# llama-cpp-python Server - File Index

## Files in this Directory

### Core Server Files

#### `qwen_server.py` (8.0 KB)
Main FastAPI server implementation
- FastAPI application setup
- OpenAI-compatible endpoints
- Tool call post-processing
- Health check and model listing
- Full async/await support

**Key Functions**:
- `POST /v1/chat/completions` - Main inference endpoint
- `GET /health` - Server health check
- `GET /v1/models` - List available models
- `_process_tool_calls()` - XML→JSON conversion

#### `tool_parser.py` (5.3 KB)
Qwen3 tool call parser
- Regex-based XML parsing
- Converts XML to OpenAI JSON format
- Based on official HuggingFace implementation
- Reusable module

**Key Classes**:
- `Qwen3CoderToolParser` - Main parser class
- Exports: `parse_tool_calls()`, `has_tool_calls()`, `extract_tool_calls_and_text()`

#### `config.py` (2.1 KB)
Configuration management
- Model parameters
- Server settings (host, port, workers)
- Inference tuning (temperature, context, batch size)
- RTX 3060 optimized defaults

**Key Variables**:
- `N_GPU_LAYERS = 26` - GPU offloading
- `N_CTX = 32768` - Context window
- `N_BATCH = 256` - Batch size
- `TEMPERATURE = 0.7` - Default temperature

### Startup and Testing

#### `../start_python_server.sh` (6.0 KB)
Server startup script
- Creates/activates Python virtual environment
- Installs dependencies with CUDA support
- Checks model file exists
- Monitors server startup
- Implements health check polling

**Key Features**:
- Kills existing processes
- Colored output for clarity
- 120-second startup timeout
- Comprehensive error checking

#### `test_tool_calling.py` (9.8 KB)
Comprehensive test suite
- Server health verification
- Simple chat test
- Tool calling test
- Multiple tools test
- Color-coded output
- CI/CD compatible exit codes

**Test Coverage**:
- Basic connectivity
- Chat without tools
- Single tool execution
- Multiple tool definitions

### Documentation

#### `QUICK_START.md` (1.4 KB)
Quick reference guide
- 30-second setup instructions
- Common commands
- Quick troubleshooting
- File changes summary

#### `../PYTHON_SERVER_SETUP.md` (8.7 KB)
Complete setup documentation
- Detailed installation guide
- Configuration instructions
- API endpoint reference
- Troubleshooting section
- Performance characteristics
- Advanced configuration

#### `../IMPLEMENTATION_SUMMARY.md` (9.5 KB)
Implementation overview
- Problem analysis
- Solution architecture
- File descriptions
- Configuration improvements
- Quick start instructions
- Known limitations
- Future improvements

#### `../IMPLEMENTATION_CHECKLIST.md` (5.8 KB)
Step-by-step implementation checklist
- 9 phases with checkmarks
- Verification commands
- Troubleshooting section
- Success criteria
- Estimated time: 20-30 minutes

#### `INDEX.md` (this file)
File index and reference
- File descriptions
- Key functions/variables
- Quick links
- File sizes

### Dependencies

#### `requirements.txt` (105 bytes)
Python package dependencies
```
llama-cpp-python[server]==0.3.0
fastapi==0.104.1
uvicorn==0.24.0
pydantic==2.5.0
python-multipart==0.0.6
```

### Auto-Generated (created by startup script)

#### `venv/` (directory)
Python virtual environment
- Created by: `start_python_server.sh`
- Contains: Python interpreter, packages
- Size: ~1.2 GB (depends on dependencies)

#### `../python_server.log`
Server log file
- Created by: `start_python_server.sh`
- Contains: Startup messages, model loading, requests/responses
- View with: `tail -f ../python_server.log`

## Directory Structure

```
/home/r2/Desktop/rubox/
│
├── start_python_server.sh          ← Server startup (use this!)
├── PYTHON_SERVER_SETUP.md          ← Complete documentation
├── IMPLEMENTATION_SUMMARY.md       ← What was implemented
├── IMPLEMENTATION_CHECKLIST.md     ← Step-by-step guide
├── python_server.log               ← (auto-created) Server logs
│
└── python-server/                  ← This directory
    ├── qwen_server.py              ← FastAPI server
    ├── tool_parser.py              ← XML→JSON converter
    ├── config.py                   ← Configuration
    ├── requirements.txt            ← Dependencies
    ├── test_tool_calling.py        ← Test suite
    ├── QUICK_START.md              ← Quick reference
    ├── INDEX.md                    ← This file
    │
    ├── venv/                       ← (auto-created) Virtual environment
    │   ├── bin/
    │   │   ├── python              ← Python interpreter
    │   │   ├── pip                 ← Package manager
    │   │   └── activate            ← Activation script
    │   ├── lib/                    ← Installed packages
    │   └── ...
    │
    └── __pycache__/                ← (auto-created) Python cache
```

## Quick Reference

### Start Server
```bash
./start_python_server.sh
```

### Run Tests
```bash
cd python-server
python3 test_tool_calling.py
```

### View Logs
```bash
tail -f python_server.log
```

### Check Health
```bash
curl http://localhost:8081/health
```

### Stop Server
```bash
pkill -f "python.*qwen_server.py"
```

## File Sizes

| File | Size | Type |
|------|------|------|
| qwen_server.py | 8.0 KB | Python code |
| tool_parser.py | 5.3 KB | Python code |
| config.py | 2.1 KB | Python code |
| test_tool_calling.py | 9.8 KB | Python code |
| requirements.txt | 105 B | Text |
| QUICK_START.md | 1.4 KB | Markdown |
| INDEX.md | This | Markdown |
| start_python_server.sh | 6.0 KB | Bash script |
| PYTHON_SERVER_SETUP.md | 8.7 KB | Markdown |
| IMPLEMENTATION_SUMMARY.md | 9.5 KB | Markdown |
| IMPLEMENTATION_CHECKLIST.md | 5.8 KB | Markdown |

**Total Documentation**: ~40 KB
**Total Code**: ~25 KB
**Virtual Environment**: ~1.2 GB (auto-created)
**Log File**: Grows with usage

## Key Implementation Details

### Tool Call Conversion Process

```
Model Output (XML):
<tool_call>
  <function=bash>
    <parameter=command>ls -la</parameter>
  </function>
</tool_call>

↓ tool_parser.py (Qwen3CoderToolParser)

API Response (JSON):
{
  "choices": [{
    "message": {
      "content": null,
      "tool_calls": [{
        "type": "function",
        "function": {
          "name": "bash",
          "arguments": "{\"command\": \"ls -la\"}"
        }
      }]
    },
    "finish_reason": "tool_calls"
  }]
}

↓ OpenCode

✅ Tool Execution (NOT just display)
```

### Configuration Tuning (RTX 3060)

```python
# GPU
N_GPU_LAYERS = 26          # 26 of 49 layers on GPU
N_THREADS = 8              # CPU threads

# Memory & Speed
N_CTX = 32768              # 32k token context
N_BATCH = 256              # Token batch
N_UBATCH = 128             # Micro-batch

# Model Output
TEMPERATURE = 0.7          # Default temp
MAX_TOKENS = 8192          # Max output
TOP_P = 0.9                # Nucleus sampling
TOP_K = 40                 # Top-K sampling
```

## Maintenance

### Cleaning Up

```bash
# Remove virtual environment (frees ~1.2GB)
rm -rf python-server/venv

# Clear logs
> python_server.log

# Next startup will recreate venv automatically
```

### Updating Configuration

Edit `python-server/config.py` and restart:
```bash
pkill -f "python.*qwen_server.py"
./start_python_server.sh
```

### Viewing Full Logs

```bash
# Last 50 lines
tail -50 python_server.log

# Follow in real-time
tail -f python_server.log

# Search for errors
grep -i error python_server.log

# Count messages
wc -l python_server.log
```

## Next Steps

1. **Read**: Start with `QUICK_START.md` or `IMPLEMENTATION_CHECKLIST.md`
2. **Run**: Execute `./start_python_server.sh`
3. **Test**: Run `python3 python-server/test_tool_calling.py`
4. **Configure**: Update OpenCode baseURL to your server IP
5. **Use**: Start asking OpenCode to analyze and execute code!

---

**Questions?** See the appropriate documentation file above.
**Troubleshooting?** Check `PYTHON_SERVER_SETUP.md` Phase 8.
**Complete Overview?** See `IMPLEMENTATION_SUMMARY.md`.

---

**Version**: 1.0.0
**Last Updated**: February 2026
**Status**: ✅ Production Ready
