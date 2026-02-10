# rubox Quick Start Guide

## One-Time Setup

1. **Build the project**:
   ```bash
   cd /home/r2/Desktop/rubox
   cargo build --release
   ```

2. **Verify llama.cpp symlink**:
   ```bash
   ls -la third_party/llama.cpp
   ```
   Should show: `llama.cpp -> /home/r2/Desktop/llama.cpp`

3. **Verify models are symlinked**:
   ```bash
   ls -la models/
   ```
   Should show three .gguf files (qwen3-vl, gemma, lfm)

4. **Test the binary**:
   ```bash
   ./target/release/rubox --help
   ./target/release/rubox --list
   ```

## Running rubox

### Chat with a Single Model

1. **Create a prompt**:
   ```bash
   echo "Your question here" > prompt_input.txt
   ```

2. **Start rubox**:
   ```bash
   ./target/release/rubox
   ```

3. **Select a model** (e.g., press `1` for qwen3-vl, then Enter)

4. **Chat**: Type your follow-up messages, press Enter to send

5. **Exit**: Type `@exit` to save the chat and exit

6. **Chat is saved**: Check `Chat/Chat_*.md`

### Compare Multiple Models

1. **Create a prompt**:
   ```bash
   echo "Your question here" > prompt_input.txt
   ```

2. **Start rubox**:
   ```bash
   ./target/release/rubox
   ```

3. **Select multiple models** (e.g., type `1,2,3` then Enter)

4. **Wait**: Each model will be queried sequentially

5. **Results saved**: Check `output/Results_*.md`

## Using CLI Arguments

```bash
# Use a specific prompt without prompt_input.txt
./target/release/rubox --prompt "Explain quantum entanglement"

# Override the default model
./target/release/rubox --model gemma

# Use a specific GGUF file
./target/release/rubox --model models/custom.gguf

# List available models
./target/release/rubox --list
```

## Output Files

After running rubox, check these directories:

- **Chat/** - Interactive chat history (single model)
  - File: `Chat_YYYYMMDD_HHMMSS.md`

- **output/** - Multi-model results and prompts
  - Responses: `Results_YYYYMMDD_HHMMSS.md`
  - Prompts: `_prompts/Prompt_DD_MM_YYYY_HH_MM_SS.md`

- **tmp_md/** - Temporary files (auto-cleaned after 3 days)

## Customization

### Change Default Model

Edit `rubox_config.json`:
```json
"models": {
  "default": "models/google_gemma-3-4b-it-Q8_0.gguf"
}
```

### Add a New Model

1. Place GGUF file in `models/` directory
2. Add to registry in `rubox_config.json`:
   ```json
   "registry": {
     "mymodel": "models/mymodel.gguf"
   }
   ```

### Change Color Scheme

Edit color codes in `rubox_config.json` under `ui`:
```json
"color_orange": "\u001b[38;5;208m"
```

### Adjust Temperature

Change in `rubox_config.json`:
```json
"base_temp": 0.7
```

## Tips

- **Keep conversations focused**: Longer context reduces response speed
- **Use model sizes wisely**:
  - LFM (1.2B): Fast, basic tasks
  - Gemma (4B): Good balance, coding
  - Qwen3-VL (8B): Most capable, slower
- **Batch comparisons**: Multi-model mode is great for finding best model for your task
- **Check output files**: All responses are saved automatically

## Troubleshooting

### "llama-server not found"
```bash
ls /home/r2/Desktop/llama.cpp/build/bin/llama-server
```
If missing, ensure llama.cpp is built. Check the main repository.

### "Model file not found"
```bash
ls models/
```
Ensure model symlinks exist and point to valid files.

### Port 8081 already in use
- rubox tries to stop Ollama automatically
- Check: `lsof -i :8081` or `netstat -tlnp | grep 8081`

### Very slow startup
- First run might require model loading (20-120 seconds)
- Subsequent runs reuse the loaded server (fast)
- Check CPU/GPU usage during startup

## Next Steps

1. Explore different models and compare their outputs
2. Customize the color scheme to your preference
3. Add your own GGUF models to the registry
4. Check `MEMORY.md` for implementation notes
