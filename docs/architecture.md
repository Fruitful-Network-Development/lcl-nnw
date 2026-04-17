# Architecture

## Layers
1. **Interface shells**: `ui/cli/` and `ui/web/` are thin clients that only gather user input and render responses.
2. **Gateway/orchestrator API**: `gateway/` exposes a stable HTTP contract for orchestration and adapter dispatch.
3. **Model registry**: `model_registry/` stores profile/model metadata (`profiles/*.toml`) used for routing.
4. **Runtimes**: `runtimes/` backend-specific launch/config stubs.
1. **Interface surfaces**: `ui/cli/` and `ui/web/` (current user-facing entry points).
2. **Gateway/orchestrator**: `gateway/` (stable routing endpoint).
3. **Model registry**: `model_registry/` (models/profiles/quant defaults).
4. **Runtimes**: `runtimes/` backend-specific launch/config stubs, including vendored `runtimes/llama_cpp/llama.cpp`.
5. **Retrieval**: `retrieval/` indexing/chunk/index pipeline skeleton.
6. **Operational glue**: `ops/` env, scripts, compose placeholders.
7. **Runtime data**: `data/` (models/sessions/cache/vector store).

## Gateway orchestration flow
1. Client sends a request to `gateway`.
2. Gateway extracts **profile** (explicit `profile` field, else `model`, else default), **session** (`session_id`, else fallback), and user input.
3. Gateway calls routing policy (`policy + registry + session`) to select backend route details.
4. Gateway dispatches to adapters (`gateway/src/adapters.rs`) using the selected route.
5. Gateway returns a normalized response with route metadata for observability.

## Stable HTTP API contract
The gateway currently exposes four minimal endpoints:

### `GET /health`
Health check.

**Response**
```json
{
  "status": "ok"
}
```

### `GET /models`
Lists profile-to-model mappings from `model_registry/profiles`.

**Response**
```json
{
  "object": "list",
  "data": [
    { "id": "lead", "profile": "chat" }
  ]
}
```

### `POST /v1/chat/completions`
OpenAI-compatible shape (minimal) for chat orchestration.

**Request (minimal fields)**
```json
{
  "profile": "chat",
  "session_id": "sess-123",
  "messages": [
    { "role": "user", "content": "hello" }
  ]
}
```

Also supports:
- `model` as profile alias input.
- `input` as a direct prompt string.

**Response**
```json
{
  "id": "chatcmpl-sess-123",
  "object": "chat.completion",
  "model": "lead",
  "profile": "chat",
  "session_id": "sess-123",
  "route": {
    "profile": "chat",
    "backend": "llama_cpp",
    "endpoint": "http://127.0.0.1:8080"
  },
  "choices": [
    {
      "index": 0,
      "message": { "role": "assistant", "content": "..." },
      "finish_reason": "stop"
    }
  ]
}
```

### `POST /v1/embeddings`
OpenAI-compatible shape (minimal) for embedding orchestration.

**Request**
```json
{
  "profile": "rag",
  "session_id": "sess-123",
  "input": ["chunk-a", "chunk-b"]
}
```

`input` can be either a single string or an array of strings.

**Response**
```json
{
  "object": "list",
  "data": [
    { "object": "embedding", "index": 0, "embedding": [1.0, 0.4, 0.9] }
  ],
  "model": "embed-small",
  "profile": "rag",
  "session_id": "sess-123",
  "route": {
    "profile": "rag",
    "backend": "llama_cpp",
    "endpoint": "http://127.0.0.1:8080"
  }
}
```
## Notes on `third_party/claw-code/`

`third_party/claw-code/` is not a required current integration surface in this scaffold. Treat it as future/planned unless and until it is added explicitly.

## Design notes
- Gateway API is the stable orchestration surface; UIs should not embed routing logic.
- `model_registry` is source-controlled metadata only (no weights).
- Backend adapters can be swapped/extended without changing client contracts.
- Backends can be swapped by changing policy/adapter wiring in gateway and runtime scripts.
- Bootstrap is standardized via `bash ops/scripts/bootstrap.sh` from the repo root.
