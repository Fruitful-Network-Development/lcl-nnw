# Architecture

## Primary Shape

This repository is now a Rust Cargo workspace centered on a local agent CLI, not an HTTP gateway.

The active product surface is:

- `lcl` CLI binary from `crates/rusty-claude-cli`
- runtime/session/config/tooling code in `crates/runtime`
- provider/client code in `crates/api`
- slash-command, plugin, MCP, and tool support from the imported Claw workspace crates
- a local OpenAI-compatible backend served by `llama-server`

## Runtime Flow

1. `lcl` loads merged config from `.claw/settings.json` and `.claw/settings.local.json`.
2. The `lcl.provider` namespace projects local provider settings into the upstream provider client layer.
3. Session state is persisted under `.claw/sessions/`.
4. Tool permissions default to `workspace-write`.
5. Requests are sent to the configured OpenAI-compatible endpoint, which is expected to be a local `llama-server`.

## Config And State

Active config precedence is:

1. user config home `settings.json`
2. project `.claw/settings.json`
3. project `.claw/settings.local.json`

Within this repo, the committed project config is the main source of truth for the local runtime.

Active runtime state lives in:

- `.claw/settings.json`
- `.claw/settings.local.json`
- `.claw/sessions/`
- `data/models/`

## Local Provider Contract

The local-only first release assumes:

- `lcl.provider.kind = "openai-compat"`
- `lcl.provider.baseUrl` points at the local `llama-server`
- `lcl.provider.apiKey` is a local placeholder token unless overridden
- `lcl.llamaServer.*` drives the launcher under `runtimes/llama_cpp/start.sh`

## Archived Components

The following paths remain for reference or migration only and are not active architecture:

- `gateway/`
- `model_registry/`
- old lane manifests
- the former gateway HTTP endpoints
- `remote_frontier` runtime settings
