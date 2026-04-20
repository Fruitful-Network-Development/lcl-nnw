# lcl-nnw

`lcl-nnw` is now a Claw-first local agent repo. The primary surface is `lcl`, a Rust CLI assembled from selectively imported upstream Claw Code workspace crates and adapted for this repository's local `llama-server` runtime.

This cutover intentionally drops the old lane-first HTTP gateway contract. `remote_frontier` is out of scope for the first Claw-based release.

## What Is Active

- `lcl` CLI for interactive and one-shot agent workflows
- `.claw/settings.json` and `.claw/settings.local.json` for runtime config
- local OpenAI-compatible model serving through `llama-server`
- session state under `.claw/sessions`
- model weights under `data/models`

## What Is Archived

- `gateway/` remains in the repo only as archived code
- `model_registry/` lane manifests are no longer the active runtime source of truth
- `ops/env/.env.example` is kept only as a migration note
- `GET /health`, `GET /lanes`, and `POST /v1/chat/completions` are not supported interfaces in this phase

## Quick Start

```bash
bash ops/scripts/bootstrap.sh
bash runtimes/llama_cpp/start.sh
bash ops/scripts/dev.sh
```

That launches the local `lcl` CLI in interactive mode. For a one-shot prompt:

```bash
cargo run -p rusty-claude-cli --bin lcl -- prompt "Summarize this repository."
```

For local verification:

```bash
bash ops/scripts/smoke-test.sh
```

## Config

Committed defaults live in `.claw/settings.json` and point at the local lane values that previously lived in `local_cpu16`:

- model: `qwen2.5-coder-7b-instruct-q4_k_m`
- provider: `http://127.0.0.1:8080/v1`
- API key: `local-dev-token`
- permission mode: `workspace-write`

Optional machine-local overrides belong in `.claw/settings.local.json`. A starter template lives at `.claw/settings.local.json.example`.

## Repo Layout

```text
Cargo.toml                Root Rust workspace
crates/                   Imported and adapted Claw runtime crates
.claw/                    Runtime config and sessions
ops/scripts/              Bootstrap, run, and smoke helpers
runtimes/llama_cpp/       Config-driven llama-server launcher
data/models/              Local GGUF model weights
docs/                     Setup, architecture, and provenance
gateway/                  Archived, unsupported gateway code
model_registry/           Archived lane manifests
```

## Provenance

Upstream import details, pinned commit metadata, and adapted module ownership live in [docs/upstream-provenance.md](docs/upstream-provenance.md).
