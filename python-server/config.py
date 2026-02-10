"""
Configuration for Qwen3 Coder llama-cpp-python server
"""

import os
from pathlib import Path

# Base paths
ROOT_DIR = Path(__file__).parent.parent
MODEL_DIR = ROOT_DIR / "models"

# Model configuration
MODEL_NAME = "Cerebras-Qwen3-Coder-25B"
MODEL_PATH = MODEL_DIR / "cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf"
MODEL_ID = "cerebras-qwen3"

# Server configuration
SERVER_HOST = "0.0.0.0"
SERVER_PORT = 8081
SERVER_WORKERS = 1

# Inference parameters (from HuggingFace generation_config.json)
N_GPU_LAYERS = 26  # Offload 26 layers to GPU
N_CTX = 131072  # Context window (restored to 131k)
N_BATCH = 512  # Batch size
N_UBATCH = 512  # Micro-batch size
N_THREADS = 8
TEMPERATURE = 0.7
MAX_TOKENS = 16384
TOP_P = 0.8  # Official: 0.8 (was 0.9)
TOP_K = 20  # Official: 20 (was 40)
REPEAT_PENALTY = 1.1   # Slightly above official 1.05 - needed for Q4_K_M quantization stability
PRESENCE_PENALTY = 0.0
FREQUENCY_PENALTY = 0.0

# Performance Features
FLASH_ATTN = True
MLOCK = True
NO_MMAP = True
CACHE_TYPE_K = 2  # GGML_TYPE_Q4_0 (q8_0 too large for 12GB VRAM)
CACHE_TYPE_V = 2  # GGML_TYPE_Q4_0

# Features
USE_JINJA = True
ENABLE_TOOL_CALLING = True

# Logging
LOG_LEVEL = "INFO"
VERBOSE = False


def get_model_path() -> str:
    """Get the full path to the model file"""
    if not MODEL_PATH.exists():
        raise FileNotFoundError(f"Model not found at {MODEL_PATH}")
    return str(MODEL_PATH)


def get_server_url() -> str:
    """Get the server URL"""
    return f"http://{SERVER_HOST}:{SERVER_PORT}/v1"


def validate_config() -> bool:
    """Validate configuration"""
    if not MODEL_PATH.exists():
        print(f"ERROR: Model file not found at {MODEL_PATH}")
        return False

    if not (0 <= TEMPERATURE <= 2.0):
        print(f"ERROR: Temperature must be between 0 and 2.0, got {TEMPERATURE}")
        return False

    if N_CTX < 512:
        print(f"ERROR: Context window too small (minimum 512), got {N_CTX}")
        return False

    return True


# Print configuration summary
if __name__ == "__main__":
    print("Qwen3 Coder Server Configuration")
    print("=" * 50)
    print(f"Model: {MODEL_NAME}")
    print(f"Model Path: {MODEL_PATH}")
    print(f"Server: {get_server_url()}")
    print(f"GPU Layers: {N_GPU_LAYERS}")
    print(f"Context Window: {N_CTX}")
    print(f"Batch Size: {N_BATCH}")
    print(f"Micro-batch Size: {N_UBATCH}")
    print(f"Tool Calling: {ENABLE_TOOL_CALLING}")
    print("=" * 50)
