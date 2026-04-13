# Architecture

## Layers
1. **Gateway/orchestrator**: `gateway/` (routing endpoint stub).
2. **Model registry**: `model_registry/` (models/profiles/quant defaults).
3. **Runtime**: `runtimes/llama_cpp/` launcher + vendored `llama.cpp` source.
4. **Retrieval placeholder**: `retrieval/` reserved skeleton only.
5. **Operational glue**: `ops/` env and bootstrap scripts.
6. **Runtime data**: `data/` local models/sessions/cache/vector store directories.

## Design notes
- v1 keeps one runtime path (`llama.cpp`) to reduce moving parts.
- `model_registry` is metadata only; weights remain local and ignored.
- sentence-transformers/embedding runtime are intentionally out of v1 scope.
