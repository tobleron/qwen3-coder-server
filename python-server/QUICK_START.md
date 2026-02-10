# Quick Start Guide

## 30-Second Setup

```bash
# 1. Start the server
cd /home/r2/Desktop/rubox
./start_python_server.sh

# 2. Wait for "Server is ready!" message
# (Takes 30-40 seconds first time)

# 3. Test it works
curl http://localhost:8081/health
```

## Use with OpenCode

```json
// In ~/.config/opencode/opencode.json
"baseURL": "http://192.168.1.186:8081/v1"
// (Replace IP with your desktop's IP)
```

Now ask OpenCode to analyze code or run commands - tool calling should work!

## Common Commands

```bash
# View logs
tail -f ../python_server.log

# Test tool calling
python3 test_tool_calling.py

# Stop server
pkill -f "python.*qwen_server.py"

# Check GPU usage
nvidia-smi -l 1
```

## What Changed

| Before (C++) | After (Python) |
|---|---|
| `start_qwen_server.sh` | `start_python_server.sh` |
| XML tool calls | ✅ JSON tool calls |
| OpenCode shows XML text | ✅ OpenCode executes tools |
| Manual parsing needed | ✅ Automatic conversion |

## Troubleshooting

**"Connection refused"**
- Server not running? Run `./start_python_server.sh`
- Different port? Check it's 8081

**"Out of memory"**
- Reduce `N_GPU_LAYERS` in `config.py` from 26 to 20
- Restart server

**"Tool calls not working"**
- Run test: `python3 test_tool_calling.py`
- Check logs: `tail -f ../python_server.log`

## More Info

See `../PYTHON_SERVER_SETUP.md` for complete documentation.
