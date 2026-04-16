# CLI UI

The CLI is a thin access app over the gateway orchestration API.

## Responsibilities
- Collect user input and session/profile preferences.
- Call gateway endpoints:
  - `POST /v1/chat/completions`
  - `POST /v1/embeddings`
  - `GET /models`
  - `GET /health`
- Render normalized responses.

## Non-responsibilities
- No direct model runtime calls.
- No routing/backend selection logic in CLI.
- No local profile resolution beyond passing user intent to gateway.

See `docs/architecture.md` for canonical request/response contracts.
