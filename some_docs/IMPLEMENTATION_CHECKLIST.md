# Implementation Checklist

Follow this checklist to complete the llama-cpp-python server setup.

## Phase 1: Initial Setup (5 minutes)

- [ ] Navigate to rubox directory: `cd /home/r2/Desktop/rubox`
- [ ] Verify model file exists: `ls -lh models/cerebras_Qwen3-Coder-*.gguf`
- [ ] Make startup script executable: `chmod +x start_python_server.sh`
- [ ] Check Python 3 is installed: `python3 --version` (should be 3.9+)
- [ ] Verify GPU: `nvidia-smi` (should show RTX 3060)

## Phase 2: Server Startup (2-3 minutes)

- [ ] **Start the server**: `./start_python_server.sh`
- [ ] **Wait for output**:
  - Should see "Virtual environment activated"
  - Should see "Installing dependencies"
  - Should see "Model found"
  - Should see "🚀 Starting Qwen3-Coder server..."
  - **Should see "✓ Server is ready!"** (most important)
- [ ] Note the server PID from output
- [ ] Check log file exists: `ls -lh python_server.log`

## Phase 3: Verification (3 minutes)

- [ ] **Test health endpoint** (in new terminal):
  ```bash
  curl http://localhost:8081/health
  ```
  Expected output: `{"status":"ok","model":"cerebras-qwen3",...}`

- [ ] **Test chat endpoint**:
  ```bash
  curl -X POST http://localhost:8081/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{"model":"cerebras-qwen3","messages":[{"role":"user","content":"Hello"}],"max_tokens":100}'
  ```
  Expected: JSON response with message content

- [ ] **View startup logs**:
  ```bash
  tail -20 python_server.log
  ```
  Should show model loading and initialization

## Phase 4: Tool Calling Tests (5 minutes)

- [ ] **Run test suite** (in new terminal):
  ```bash
  cd python-server
  python3 test_tool_calling.py
  ```

- [ ] **All tests should PASS**:
  - [ ] Health Check - PASS
  - [ ] Simple Chat - PASS
  - [ ] Tool Calling - PASS
  - [ ] Multiple Tools - PASS

- [ ] **Check test output** for tool call conversion:
  - Should see "Tool called 1 tool(s)"
  - Should see proper function names
  - Should see JSON arguments

## Phase 5: OpenCode Configuration (2 minutes)

- [ ] **Get your desktop IP**:
  ```bash
  hostname -I
  ```
  Note the first IP address (e.g., 192.168.1.186)

- [ ] **Edit OpenCode config**: `~/.config/opencode/opencode.json`
  - [ ] Update baseURL: `http://YOUR_IP:8081/v1`
  - [ ] Keep model name: `cerebras-qwen3`
  - [ ] Save file

- [ ] **Restart OpenCode** for config to take effect

## Phase 6: OpenCode Testing (5 minutes)

- [ ] **Test simple query**: Ask OpenCode a question
  - Example: "What is 2+2?"
  - Should respond in chat normally

- [ ] **Test tool calling**: Ask OpenCode to analyze code
  - Example: "Analyze the current directory structure"
  - Should SEE tool calls being EXECUTED (not displayed as text)
  - Should show results of tool execution

- [ ] **Verify tool execution works**:
  - Look for bash commands being run
  - Files being created/modified
  - Code being executed
  - NOT just showing XML text

## Phase 7: Monitoring (Ongoing)

- [ ] **Monitor server logs**:
  ```bash
  tail -f python_server.log
  ```

- [ ] **Check GPU usage**:
  ```bash
  nvidia-smi -l 1
  ```
  Should show ~9-10GB VRAM in use

- [ ] **Watch for errors**: No ERROR or WARNING messages (except memory lock during load)

## Phase 8: Troubleshooting (if needed)

### Server won't start
- [ ] Check port 8081 isn't in use: `lsof -i :8081`
- [ ] Kill any existing process: `pkill -f "python.*qwen_server.py"`
- [ ] Check logs: `tail -50 python_server.log`
- [ ] Verify model file: `ls -lh models/cerebras_*.gguf`

### Out of memory
- [ ] Edit `python-server/config.py`
- [ ] Reduce `N_GPU_LAYERS` from 26 to 20
- [ ] Reduce `N_CTX` from 32768 to 16384
- [ ] Restart: `pkill -f "python.*qwen_server.py" && ./start_python_server.sh`

### Tool calls not working
- [ ] Run test: `python3 python-server/test_tool_calling.py`
- [ ] Check if bash tool is being called
- [ ] Verify OpenCode config has correct IP
- [ ] Look for errors in logs: `grep -i error python_server.log`

### Slow responses
- [ ] Check GPU load: `nvidia-smi`
- [ ] Increase `N_GPU_LAYERS` in config.py (if memory allows)
- [ ] Monitor system memory: `free -h`

## Phase 9: Confirmation

When this checklist is complete:

- [ ] Server starts automatically with `./start_python_server.sh`
- [ ] Health endpoint responds: `curl http://localhost:8081/health`
- [ ] Test suite passes: `python3 python-server/test_tool_calling.py`
- [ ] OpenCode connects successfully
- [ ] Tool calls are EXECUTED (not displayed as text)
- [ ] All tool call results appear in OpenCode chat
- [ ] No errors in logs

## Success Criteria

✅ **Implementation is successful when:**

1. Server starts without errors
2. Health endpoint responds with "status": "ok"
3. Test suite shows all PASS
4. OpenCode connects to server
5. Tool calls execute (bash commands run, results appear)
6. No XML text displayed to user
7. Can use OpenCode for code analysis and tool execution

## Next Steps After Checklist

1. ✅ Keep `./start_python_server.sh` running
2. ✅ Use OpenCode normally for coding tasks
3. ✅ Monitor logs for issues: `tail -f python_server.log`
4. ✅ Refer to `PYTHON_SERVER_SETUP.md` for advanced configuration

## Support

**If you get stuck**:
1. Check Phase 8 (Troubleshooting)
2. Look at PYTHON_SERVER_SETUP.md (complete guide)
3. Run test suite: `python3 python-server/test_tool_calling.py`
4. Check logs: `tail -50 python_server.log`

---

**Estimated Total Time**: 20-30 minutes
**Difficulty**: Easy to Medium
**Estimated Success Rate**: >95% with this checklist
