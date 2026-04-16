# Architecture

## Layers
1. **Interface surfaces**: `ui/cli/` and `ui/web/` (current user-facing entry points).
2. **Gateway/orchestrator**: `gateway/` (stable routing endpoint).
3. **Model registry**: `model_registry/` (models/profiles/quant defaults).
4. **Runtimes**: `runtimes/` backend-specific launch/config stubs, including vendored `runtimes/llama_cpp/llama.cpp`.
5. **Retrieval**: `retrieval/` indexing/chunk/index pipeline skeleton.
6. **Operational glue**: `ops/` env, scripts, compose placeholders.
7. **Runtime data**: `data/` (models/sessions/cache/vector store).

## Notes on `third_party/claw-code/`

`third_party/claw-code/` is not a required current integration surface in this scaffold. Treat it as future/planned unless and until it is added explicitly.

## Design notes
- The gateway is intentionally thin and backend-agnostic.
- `model_registry` is source-controlled metadata only (no weights).
- Backends can be swapped by changing policy/adapter wiring in gateway and runtime scripts.
- Bootstrap is standardized via `bash ops/scripts/bootstrap.sh` from the repo root.
