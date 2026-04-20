# Upstream Provenance

This repository vendors selected Rust workspace components from the upstream Claw Code project to power the local `lcl` CLI.

## Source

- Repository: `https://github.com/ultraworkers/claw-code`
- Branch at import time: `main`
- Imported commit: `50e3fa3a834a7a5a603d69c372a2c4c190b7104b`
- Import date: April 20, 2026

## Imported Rust Workspace Material

- Root workspace files:
  - `Cargo.toml`
  - `Cargo.lock`
  - `MOCK_PARITY_HARNESS.md`
  - `mock_parity_scenarios.json`
- Workspace support scripts:
  - `scripts/`
- Rust crates:
  - `crates/api`
  - `crates/commands`
  - `crates/compat-harness`
  - `crates/mock-anthropic-service`
  - `crates/plugins`
  - `crates/runtime`
  - `crates/rusty-claude-cli`
  - `crates/telemetry`
  - `crates/tools`

## LCL-Owned Adaptations

- Renamed the primary binary from `claw` to `lcl`.
- Added an `lcl` namespace to runtime config for local provider and `llama-server` settings.
- Defaulted the local provider flow to an OpenAI-compatible `llama-server`.
- Reworked repo scripts and docs around `.claw/` state and a CLI-first workflow.

## Explicit Non-Imports

- The upstream Python/reference workspace was not imported.
- The legacy HTTP gateway in this repository was not replaced with an upstream daemon because upstream Claw exposes a CLI/runtime workspace rather than an HTTP server.
