# rubox - Rust-based Chat Application with llama.cpp

A lightweight Rust application that provides interactive chat and multi-model comparison using the llama.cpp inference engine.

## Features

- **Interactive Chat Mode**: Seamless conversation with a single selected model
- **Multi-Model Mode**: Compare responses from multiple models side-by-side
- **llama.cpp Integration**: Direct integration with llama.cpp for local inference
- **Orange/Red Theme**: Terminal colors matching the Ollama_LAB aesthetic
- **Configurable Models**: Easy model registry management via `rubox_config.json`
- **Automatic Ollama Detection**: Stops Ollama service if running to avoid port conflicts
- **Chat History**: All conversations automatically saved as markdown files
- **File Organization**: Structured output for prompts, responses, and chats

## Project Structure

```
rubox/
├── Cargo.toml                 # Rust dependencies
├── rubox_config.json          # Configuration file
├── src/
│   ├── main.rs                # Entry point and CLI handling
│   ├── config.rs              # Configuration loading and structs
│   ├── llm_client.rs          # HTTP client for llama.cpp API
│   ├── server_manager.rs      # llama.cpp server lifecycle management
│   ├── chat.rs                # Interactive chat mode logic
│   ├── multi_model.rs         # Multi-model comparison mode
│   └── ui.rs                  # Terminal UI utilities
├── models/                    # GGUF model storage (symlinked)
├── output/                    # Output directory
│   └── _prompts/              # Saved prompts
├── Chat/                      # Chat history (markdown files)
├── tmp_md/                    # Temporary response files
└── third_party/
    └── llama.cpp/             # Symlink to llama.cpp installation
```

## Building

```bash
cd rubox
cargo build --release
```

The binary will be at `target/release/rubox`.

## Configuration

Edit `rubox_config.json` to customize:

- **LLM Settings**: API URL, default model, temperature, context window
- **Model Registry**: Symbolic names mapping to GGUF file paths
- **User Name**: Name displayed in chat history
- **Colors**: ANSI color codes for terminal output
- **Cleanup**: Age threshold for temporary file deletion

## Usage

### List Available Models
```bash
./target/release/rubox --list
```

### Chat Mode (Single Model)
```bash
# With prompt file
echo "What is Rust?" > prompt_input.txt
./target/release/rubox
# Select model 1 for chat mode

# With CLI argument
./target/release/rubox --prompt "Explain quantum computing"
```

### Multi-Model Mode (Compare Models)
```bash
./target/release/rubox
# Select multiple models: "1,2,3"
```

## Key Behaviors

### Chat Mode
1. Start a conversation with an initial prompt
2. Send the entire conversation history to the model
3. Display model response and wait for user input
4. Type `@exit` to save the chat and exit
5. Chat saved to `Chat/Chat_YYYYMMDD_HHMMSS.md`

### Multi-Model Mode
1. Send the same prompt to each selected model sequentially
2. Save individual responses to `tmp_md/`
3. Save prompt to `output/_prompts/`
4. Combine all responses in `output/Results_YYYYMMDD_HHMMSS.md`
5. Auto-clean temporary files older than 3 days

### Ollama Integration
- Automatically detects if Ollama service is running
- Stops Ollama to avoid port 11434 conflicts
- llama.cpp runs on port 8081 (configurable)

## Models

Pre-configured models (update paths as needed):
- **qwen3-vl**: Qwen3-VL-8B-Instruct (7.5GB, multimodal capable)
- **gemma**: Google Gemma 3 4B (4.1GB)
- **lfm**: LFM 2.5 1.2B (2.3GB, lightweight)

## Technical Details

- **Language**: Rust 2021 edition
- **Async Runtime**: Tokio
- **HTTP Client**: Reqwest with JSON support
- **CLI**: Clap with derive macros
- **Config Format**: JSON (serde)
- **Server Communication**: REST API (llama.cpp /v1 format)
- **Color Library**: ANSI escape codes

## Dependencies

- tokio: Async runtime
- reqwest: HTTP client
- serde/serde_json: Configuration and data serialization
- clap: CLI argument parsing
- chrono: Timestamp generation
- anyhow: Error handling
- colored: Terminal colors (optional, using ANSI codes)

## Performance Characteristics

- **Startup**: ~2-5 seconds (model independent)
- **Model Loading**: 30-120 seconds (depends on model size and GPU)
- **First Response**: Varies by model and hardware
- **Subsequent Responses**: Real-time generation streamed to terminal
- **Memory**: Depends on model context window (default 8192 tokens)

## Troubleshooting

### llama-server not found
Ensure llama.cpp is built at `third_party/llama.cpp/build/bin/llama-server`

### Model file not found
Check paths in `rubox_config.json` and `models/` directory

### API connection refused
Verify llama-server started successfully and is listening on configured port

### Port already in use
The application tries to stop Ollama automatically. Verify no other services use port 8081.

## Future Enhancements

- Streaming responses for real-time output
- Multimodal support (image prompts with Qwen3-VL)
- Web UI frontend
- Session persistence and resumption
- Grammar constraints and structured output
- Batch processing for large prompt files

## License

Created as an evolution of Ollama_LAB and rugged-cli projects.
