# Setup

## Requirements

- Rust toolchain with `cargo`
- `llama-server` installed locally
- a GGUF model file available at the path configured in `.claw/settings.json` or `.claw/settings.local.json`

This repo no longer uses the gateway-first env workflow. Runtime settings come from `.claw/`, not `ops/env/.env`.

## Bootstrap

From the repo root:

```bash
bash ops/scripts/bootstrap.sh
```

That prepares:

- `.claw/sessions/`
- `data/models/`
- `data/caches/`
- `ops/logs/`

## Configure The Local Runtime

Shared defaults live in `.claw/settings.json`.

If you need machine-specific overrides, copy the example file and edit it:

```bash
cp .claw/settings.local.json.example .claw/settings.local.json
```

The repo-specific `lcl` namespace supports:

- `lcl.provider.kind`
- `lcl.provider.baseUrl`
- `lcl.provider.apiKey`
- `lcl.llamaServer.bin`
- `lcl.llamaServer.host`
- `lcl.llamaServer.port`
- `lcl.llamaServer.ctxSize`
- `lcl.llamaServer.modelPath`

## Start llama-server

```bash
bash runtimes/llama_cpp/start.sh
```

That script reads merged `.claw/settings.json` plus `.claw/settings.local.json`, resolves the configured model path, and launches `llama-server` with the configured host, port, and context size.

## Run lcl

Interactive mode:

```bash
bash ops/scripts/dev.sh
```

One-shot mode:

```bash
cargo run -p rusty-claude-cli --bin lcl -- prompt "Explain the workspace layout."
```

Useful local diagnostics:

```bash
cargo run -p rusty-claude-cli --bin lcl -- doctor
cargo run -p rusty-claude-cli --bin lcl -- status
```

## Smoke Test

With `llama-server` running:

```bash
bash ops/scripts/smoke-test.sh
```

The smoke script checks:

- `lcl doctor`
- `lcl status`
- a prompt round-trip when the configured OpenAI-compatible endpoint is reachable
