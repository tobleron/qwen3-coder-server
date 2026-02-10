# Comprehensive Implementation Task: Cerebras-Qwen3-Coder (REAP) Integration

This task defines the "Pure Default" implementation for the Cerebras-Qwen3-Coder-25B model using `llama-cpp-python` and OpenCode.ai. It eliminates all workarounds and improvisations in favor of official model specifications and production-grade parsing logic.

## 1. Core Model Configuration (The "Book" Defaults)
Based on official HuggingFace `generation_config.json` and model card specifications.

### Sampling Parameters
- **Temperature:** 0.7
- **Top P:** 0.8
- **Top K:** 20
- **Repetition Penalty:** 1.05
- **Context Window:** 131,072 (131k)
- **Max Output Tokens:** 16,384

### Performance Defaults (Optimized for 12GB+ VRAM)
- **GPU Layers:** 26 (Balanced for 25B model on consumer hardware)
- **Flash Attention:** Enabled
- **KV Cache:** Q4_0 (Provides 131k context within available memory)
- **Mlock:** Enabled (Prevents weight swapping)
- **Mmap:** Disabled (Forces full load into RAM/VRAM)

## 2. Protocol & Communication (ChatML Standard)
The model follows the ChatML format. All turn boundaries must be strictly enforced.

### Official Stop Tokens
- `<|im_end|>` (Primary turn terminator)
- `<|endoftext|>` (System fallback)
- `<|im_start|>user` (Prevents hallucinating next turn)
- `<|im_start|>system` (Prevents hallucinating instructions)

### Chat Template
Use the internal GGUF template (which maps to ChatML) BUT with `use_jinja=True` to ensure the `llama-cpp-python` engine handles the role mapping correctly.

## 3. Tool Calling Architecture (The "Healing" Parser)
Implement the vLLM-standard `qwen3_coder` parsing logic. This must handle both wrapped and unwrapped function calls to be resilient against template inconsistencies.

### Required Tokens for Extraction
- **Start:** `<tool_call>` (Optional fallback to `<function=`)
- **End:** `</tool_call>` (Optional fallback to `</function>`)
- **Function Name:** `<function=NAME>`
- **Parameters:** `<parameter=NAME>VALUE</parameter>`

### Implementation Logic (Pure Python)
1. **Regex Scanning:** Use non-greedy regex to find all function blocks.
2. **Type Preservation:** If a parameter value looks like a JSON array `[...]` or object `{...}`, parse it with `json.loads()` to pass real objects to OpenCode.
3. **Normalization:** Convert XML output into valid OpenAI Tool Call JSON objects.
4. **Text Cleaning:** Any text *before* the first tool call is yielded as assistant content; everything from the first tool tag onwards is intercepted.

## 4. OpenCode.ai Integration (`opencode.json`)
The client configuration must be aligned with the server's capabilities.

### Recommended Project/Global Config
```json
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "llama_local": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Llama Server Local",
      "options": {
        "baseURL": "http://<SERVER_IP>:8081/v1",
        "apiKey": "sk-no-key-required"
      },
      "models": {
        "cerebras-qwen3": {
          "name": "Cerebras Qwen3 Coder 25B",
          "toolsEnabled": true,
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

## 5. Deployment Checklist
1. **Reset Config:** Update `python-server/config.py` with the exact 131k context and 1.05 penalty.
2. **Robust Parser:** Implement the vLLM-style regex patterns in `tool_parser.py`.
3. **Turn Control:** Ensure `stop_tokens` are passed to EVERY completion request.
4. **Streaming Filter:** Verify the look-ahead buffer in `qwen_server.py` prevents `<tool_call>` tags from leaking as raw text.
5. **OpenCode Refresh:** Update the `opencode.json` on the Mac to include `reasoning: true` and `toolsEnabled: true`.

---
*Status: READY FOR IMPLEMENTATION*
