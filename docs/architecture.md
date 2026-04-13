# Architecture

## Layers
1. **Interface shell**: `third_party/claw-code/` (integration surface).
2. **Gateway/orchestrator**: `gateway/` (stable routing endpoint).
3. **Model registry**: `model_registry/` (models/profiles/quant defaults).
4. **Runtimes**: `runtimes/` backend-specific launch/config stubs.
5. **Retrieval**: `retrieval/` indexing/chunk/index pipeline skeleton.
6. **Operational glue**: `ops/` env, scripts, compose placeholders.
7. **Runtime data**: `data/` (models/sessions/cache/vector store).

## Design notes
- The gateway is intentionally thin and backend-agnostic.
- `model_registry` is source-controlled metadata only (no weights).
- Backends can be swapped by changing policy/adapter wiring in gateway and runtime scripts.
