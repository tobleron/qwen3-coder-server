# rubox: Pure llama.cpp Implementation

## Confirmation: Zero Ollama Dependency

**rubox is 100% pure llama.cpp** - all inference and model management goes directly through llama.cpp with no Ollama intermediary.

### What llama.cpp Does In rubox

```
User Input
    ↓
[rubox binary]
    ↓
[TCP port check: 127.0.0.1:8081]
    ↓
[Start llama-server if needed]
    ↓
[POST to http://127.0.0.1:8081/v1/chat/completions]
    ↓
[llama.cpp loads GGUF directly]
    ↓
[Generate response]
    ↓
[Display to user]
```

### What Ollama Does In rubox

**Only one thing:** Stopping it if it's running to prevent port conflicts.

That's it. Zero inference, zero API calls to Ollama, zero dependencies.

## Technical Details

### Configuration Points to llama.cpp Only

**rubox_config.json**:
```json
{
  "llm": {
    "api_url": "http://127.0.0.1:8081/v1",  // llama.cpp port
    "model_name": "qwen3-vl"                 // Model name (not Ollama name)
  },
  "models": {
    "registry": {
      "qwen3-vl": "models/Qwen3-VL-8B-Instruct-UD-Q6_K_XL.gguf",  // Direct path
      "gemma": "models/google_gemma-3-4b-it-Q8_0.gguf",           // Direct path
      "lfm": "models/LFM2.5-1.2B-Instruct-BF16.gguf"              // Direct path
    }
  }
}
```

None of these are Ollama model names. They're direct GGUF paths.

### Server Management: Only llama.cpp

**src/server_manager.rs**:
```rust
// Start ONLY llama-server
Command::new("./third_party/llama.cpp/build/bin/llama-server")
    .args(&[
        "--model", &model_path,          // Direct GGUF path
        "--ctx-size", &context_window,
        "--port", "8081",                // llama.cpp port (NOT 11434)
        "--n-gpu-layers", "-1",          // GPU acceleration
        "--parallel", "4",
        "--log-disable"
    ])
```

No Ollama binary, no Ollama environment variables, no Ollama API calls.

### API Client: llama.cpp Format

**src/llm_client.rs**:
```rust
// POST to llama.cpp endpoint
let url = format!("{}/chat/completions", self.api_url);
// self.api_url = "http://127.0.0.1:8081/v1"

// Standard llama.cpp format
let request = CompletionRequest {
    model: self.model_name.clone(),    // Model name or path
    messages,                           // Conversation history
    temperature: 0.7,
    max_tokens: 4096,
};

self.client.post(url).json(&request).send().await?
```

This is the standard llama.cpp `/v1/chat/completions` API format. Works with any llama.cpp server.

### Ollama Stop: Safety Check Only

**src/server_manager.rs**:
```rust
pub fn stop_ollama_if_running() -> anyhow::Result<()> {
    let output = Command::new("systemctl")
        .arg("is-active")
        .arg("ollama")
        .output();

    if let Ok(output) = output {
        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if status == "active" {
            // ONLY if Ollama is running, stop it
            Command::new("systemctl")
                .arg("stop")
                .arg("ollama")
                .output()?;
            println!("Stopped Ollama service to avoid port conflicts");
        }
    }
    Ok(())
}
```

This checks if Ollama is running and stops it **only for conflict avoidance**. No Ollama API is ever called.

## Dependencies: All llama.cpp Compatible

**Cargo.toml**:
- `tokio` - Async runtime ✅
- `reqwest` - HTTP client (works with any REST API) ✅
- `serde_json` - JSON parsing ✅
- `clap` - CLI arguments ✅
- `chrono` - Timestamps ✅
- `anyhow` - Error handling ✅
- `colored` - Terminal colors ✅

**Zero Ollama dependencies** - no ollama crate, no ollama SDK.

## How It Differs From Ollama_LAB

### Ollama_LAB (bash script)
```bash
# Used Ollama API
curl -X POST "http://localhost:11434/api/generate"
# Ollama managed the models
# Ollama managed the inference server
```

### rubox (Rust)
```rust
// Uses llama.cpp API directly
POST http://127.0.0.1:8081/v1/chat/completions
// rubox manages the llama-server
// rubox loads GGUF models directly
```

## Advantages of Pure llama.cpp

1. **No Ollama overhead** - Direct to inference engine
2. **Full parameter control** - GPU layers, parallel requests, etc.
3. **Simpler setup** - No need to install/run Ollama service
4. **Better performance** - Skip Ollama translation layer
5. **Transparent dependencies** - Only what's needed
6. **Portable** - Works on any system with llama.cpp

## Port Usage

- **8081** - llama.cpp server (rubox default)
- **11434** - Ollama (stopped if running, NOT used)

## Model Loading Flow

```
rubox starts
  ↓
Checks if Ollama running (safety check only)
  ↓
Checks if llama-server already listening on 8081
  ↓
If no: Starts llama-server with:
  - Direct GGUF file path
  - GPU acceleration settings
  - Context window size
  - Parallel request settings
  ↓
Waits for /health endpoint
  ↓
Sends prompt via /v1/chat/completions
  ↓
llama.cpp loads model into memory (first time)
  ↓
llama.cpp generates response
  ↓
rubox receives JSON response
  ↓
Display to user
```

## Configuration vs Ollama

### Ollama Approach
- Models stored in `~/.ollama/models/`
- Model registry in Ollama system
- Models pulled from ollama.ai registry
- Configuration in Ollama config

### rubox Approach
- Models in `./models/` directory
- Model registry in `rubox_config.json`
- Models are local GGUF files
- Full configuration in `rubox_config.json`

## Verification Commands

```bash
# Check the API endpoint (should be 8081, not 11434)
grep api_url rubox_config.json

# Check that no Ollama SDK is used
grep -r "ollama" Cargo.toml  # Should show nothing

# Verify source code only stops Ollama (doesn't call it)
grep -r "11434" src/  # Should show nothing
grep -r "ollama" src/ | grep -v "stop_ollama"  # Should show nothing useful

# Check the binary doesn't depend on ollama
ldd ./target/release/rubox | grep -i ollama  # Should show nothing
```

## Summary

**rubox is pure llama.cpp:**
- ✅ All inference through llama.cpp
- ✅ All models are direct GGUF files
- ✅ All configuration is in rubox_config.json
- ✅ Ollama is only checked (and stopped if running)
- ✅ Zero Ollama API calls
- ✅ Zero Ollama dependencies

This is the implementation you requested: a clean, simple Rust application that uses llama.cpp directly, with all the convenience and power of model comparison from Ollama_LAB, but without any Ollama dependency.
