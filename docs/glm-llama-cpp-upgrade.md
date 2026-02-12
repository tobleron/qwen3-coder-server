# GLM llama-cpp-python upgrade notes

These steps describe how to keep the existing Qwen setup while rebuilding `llama-cpp-python` so a GLM `Q4_*` GGUF (or an imminent Q3 build) can be loaded.

1. **Use the same Python runtime as the Qwen server.**
   - If you already run Qwen inside a Miniforge/conda env (e.g., `python=3.11` or `3.12`), clone that environment for GLM: `conda create -n qwen-glm python=3.11`.
   - Activate it before installing dependencies so the GLM server reuses the same stacks (`pip install -r python-server/requirements.txt`).

2. **Rebuild `llama-cpp-python` against the latest `llama.cpp`.**
   - Within the activated env, install the wheel from source so it pulls the patched `llama.cpp` (supporting newer quant types):
     ```bash
     pip install --force-reinstall --no-cache-dir "git+https://github.com/abetlen/llama-cpp-python.git@master#egg=llama-cpp-python[server]"
     ```
   - This rebuild ensures `llama_cpp` now carries up-to-date architecture tables; it won’t change the `llama-cpp-python` version already running for Qwen because both endpoints share the same virtualenv.

3. **Use a supported GLM GGUF.**
   - Download an officially supported `Q4_*` GGUF for GLM (Unsloth/official repos keep these up to date). Place it under `models/`.
   - Update `python-server/glm_config.py` (if needed) to point to the new file.

4. **Verify startup without touching the Qwen port.**
   - Start the GLM server via `./start_glm_server.sh` (or `./start_model_menu.sh` option 2). It listens on `localhost:8082`.
   - Confirm `/health` looks good and run `python-server/verify_reliability.py --base-url http://localhost:8082/v1 --model glm-4.7-flash` against the new endpoint.

5. **Why Qwen keeps working**
   - The new `llama_cpp` binary is backward compatible: it still loads the `cerebras_Qwen3-Coder-REAP-25B-A3B-Q4_K_M.gguf` you already run on port 8081.
   - Only the GLM server uses the `glm_config.py` path and the new GGUF, so the upgrade is additive and doesn’t disrupt the Qwen workflow.
