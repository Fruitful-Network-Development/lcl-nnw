# llama.cpp runtime scaffold

This folder defines the backend contract for running local generation with `llama.cpp`.

## Intent
- Keep backend-specific launch details isolated from gateway logic.
- Make startup reproducible via scripts.
- Avoid downloading model files in-repo.

## Expected binary
Set `LLAMA_CPP_SERVER_BIN` in `runtimes/llama_cpp/.env.example`.

## Expected model location
Model paths should point into `data/models/` and be referenced from `model_registry/models/*.toml`.

## Startup
```bash
bash runtimes/llama_cpp/start.sh
```
