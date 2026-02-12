"""Configuration for GLM-4.7-Flash llama-cpp-python server"""

from pathlib import Path

# Base paths
ROOT_DIR = Path(__file__).parent.parent
MODEL_DIR = ROOT_DIR / "models"

# Model configuration
MODEL_NAME = "GLM-4.7-Flash (23B-A3B-Q4_K_M)"
MODEL_PATH = MODEL_DIR / "GLM-4.7-Flash-REAP-23B-A3B-Q4_K_M.gguf"
MODEL_ID = "glm-4.7-flash"

# Server configuration
SERVER_HOST = "0.0.0.0"
SERVER_PORT = 8081
SERVER_WORKERS = 1

# Inference parameters
N_GPU_LAYERS = 26
N_CTX = 131072
N_BATCH = 512
N_UBATCH = 512
N_THREADS = 8
TEMPERATURE = 0.2
MAX_TOKENS = 16384
TOP_P = 0.8
TOP_K = 20
REPEAT_PENALTY = 1.0
PRESENCE_PENALTY = 0.0
FREQUENCY_PENALTY = 0.0

# Performance features
FLASH_ATTN = True
MLOCK = True
NO_MMAP = True
CACHE_TYPE_K = 2
CACHE_TYPE_V = 2

# Features
USE_JINJA = False
ENABLE_TOOL_CALLING = True

# Logging
LOG_LEVEL = "INFO"
VERBOSE = False


def get_model_path() -> str:
    if not MODEL_PATH.exists():
        raise FileNotFoundError(f"Model not found at {MODEL_PATH}")
    return str(MODEL_PATH)


def get_server_url() -> str:
    return f"http://{SERVER_HOST}:{SERVER_PORT}/v1"


def validate_config() -> bool:
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


if __name__ == "__main__":
    print("GLM-4.7 Server Configuration")
    print("=" * 50)
    print(f"Model: {MODEL_NAME}")
    print(f"Model Path: {MODEL_PATH}")
    print(f"Server: {get_server_url()}")
    print(f"GPU Layers: {N_GPU_LAYERS}")
    print(f"Context Window: {N_CTX}")
    print(f"Tool Calling: {ENABLE_TOOL_CALLING}")
    print("=" * 50)
