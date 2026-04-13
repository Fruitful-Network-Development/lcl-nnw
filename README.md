# lcl-nnw

Offline-first local LLM scaffold focused on a **single v1 runtime**: `llama.cpp`.

## What this repository contains

- `gateway/`: thin Rust routing/orchestration stub.
- `model_registry/`: model/profile manifests (metadata only, no model binaries).
- `runtimes/llama_cpp/`: local launcher wrapper for `llama-server`.
- `runtimes/llama_cpp/llama.cpp/`: **vendored** `llama.cpp` source tree tracked in Git.
- `ops/scripts/bootstrap.sh`: creates expected local runtime directories.
- `ops/env/local-model-paths.example`: canonical local GGUF path examples.

## What this repository does not contain

- model binaries (`.gguf`, `.safetensors`, etc.)
- runtime caches/sessions/logs
- automatic model download code
- any runtime requirement for internet access

## v1 dependency decisions

- **Keep `llama.cpp` vendored (not submodule).** This keeps cloud review/refinement self-contained without requiring submodule init.
- **Do not include sentence-transformers in v1.** Embedding runtime wiring is intentionally deferred to keep first pass minimal.

## Quick start

```bash
bash ops/scripts/bootstrap.sh
cp runtimes/llama_cpp/.env.example runtimes/llama_cpp/.env
# edit runtimes/llama_cpp/.env and set LLAMA_CPP_SERVER_BIN + LLAMA_CPP_MODEL_PATH
bash runtimes/llama_cpp/start.sh
```

Optional gateway compile check:

```bash
cd gateway
cargo check
```
