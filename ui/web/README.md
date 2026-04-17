# Web UI

The web client is a thin access app over the gateway orchestration API.

## Responsibilities
- Capture prompts, context, and session/profile hints.
- Call gateway endpoints:
  - `POST /v1/chat/completions`
  - `POST /v1/embeddings`
  - `GET /models`
  - `GET /health`
- Display normalized chat/embedding responses.

## Non-responsibilities
- No browser-side runtime/backend coupling.
- No duplicated route policy logic in frontend code.
- No direct calls to runtime engines.

See `docs/architecture.md` for the stable orchestration API contract.
