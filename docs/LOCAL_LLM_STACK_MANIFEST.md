# Local LLM Coding Stack — Principal Manifest

## 1. Project Identity

**Project name:** `local-llm-coder`

**Purpose:**
Create a local, modular, single-user coding stack that runs on the current laptop first, supports CPU-only inference, and can later swap in a stronger local machine or remote model backend without rewriting the interface or engine.

## 2. Intended End State

At the end of the initial process, the project exists as a Git repository with:

- a reproducible local runtime for a small/quantized coding-capable LLM
- a stable middle layer that abstracts the model backend
- a simple user-facing interface for prompts and code-focused tasks
- configuration files that let the active model/backend be changed without changing core code
- a clean place for later integration with agent harnesses, editor tooling, or a remote GPU server

## 3. Scope of Phase 1

Phase 1 is **not** about building a multi-agent system, cluster, training pipeline, or full autonomous development environment.

Phase 1 **is** about proving these things:

1. the model can run locally on the laptop
2. the interface to the model is modular
3. the repo is organized so the next layer can be added cleanly
4. the system is useful for short coding prompts, explanation, summarization, and small repository assistance

## 4. Operational Constraints

This project is designed around a laptop-first, CPU-first baseline.

Implications:

- prefer quantized GGUF models
- prefer runtimes that can operate well on CPU
- keep context size modest at first
- keep model weights outside core source control
- treat GPU support as an upgrade path, not a dependency

## 5. Principal Architecture

```text
User
  -> Interface Layer
  -> Engine Layer
  -> Provider Adapter Layer
  -> Runtime Backend
  -> Model Weights
```

### Layer meanings

**Interface Layer**
- CLI first
- later: editor integration, simple TUI, or web UI

**Engine Layer**
- prompt assembly
- session state
- task mode selection
- repository context loading

**Provider Adapter Layer**
- one stable internal contract
- local runtime can be swapped without changing the interface
- future remote server can be swapped in without changing the interface

**Runtime Backend**
- initial target: local `llama.cpp` server or direct local invocation
- future targets: stronger local machine, network server, or API-compatible proxy

**Model Weights**
- local GGUF files
- never committed into the main repo

## 6. Principal Repository Shape

```text
local-llm-coder/
├── README.md
├── PROJECT_STATE.md
├── .gitignore
├── .env.example
├── config/
│   ├── active.yaml
│   ├── models/
│   │   ├── cpu-demo.yaml
│   │   ├── cpu-balanced.yaml
│   │   └── remote-template.yaml
│   └── prompts/
│       ├── coding.yaml
│       ├── explain.yaml
│       └── summarize.yaml
├── docs/
│   ├── architecture.md
│   ├── decisions.md
│   ├── setup.md
│   └── upgrade-paths.md
├── scripts/
│   ├── bootstrap.sh
│   ├── run_local_server.sh
│   ├── smoke_test.sh
│   └── env_check.sh
├── src/
│   └── local_llm_coder/
│       ├── __init__.py
│       ├── main.py
│       ├── cli.py
│       ├── engine/
│       │   ├── session.py
│       │   ├── tasks.py
│       │   └── prompts.py
│       ├── providers/
│       │   ├── base.py
│       │   ├── llamacpp.py
│       │   ├── remote_openai.py
│       │   └── mock.py
│       ├── repo/
│       │   ├── scan.py
│       │   ├── files.py
│       │   └── snippets.py
│       └── util/
│           ├── config.py
│           ├── paths.py
│           └── logging.py
├── tests/
│   ├── test_config.py
│   ├── test_provider_contract.py
│   ├── test_cli.py
│   └── test_smoke.py
├── runtime/
│   ├── vendor/
│   │   └── llama.cpp/
│   └── state/
│       ├── logs/
│       ├── sessions/
│       └── cache/
├── models/
│   ├── README.md
│   └── .gitkeep
└── integrations/
    ├── editor/
    └── claw/
```

## 7. What Each Principal Area Must Contain

### `config/`
The source of truth for selecting:
- provider
- model path or server URL
- context limits
- temperature and inference defaults
- task presets

### `src/local_llm_coder/providers/`
The most important engineering seam.

Every backend must satisfy the same internal contract:
- `generate(prompt, system=None, options=None)`
- `healthcheck()`
- `model_info()`

### `runtime/vendor/llama.cpp/`
A contained runtime dependency location.

This avoids blending third-party runtime code with your own application logic.

### `models/`
Local model storage only.

This directory is operational state, not source code.

### `integrations/claw/`
Reserved for later work.

This is where any bridge, shim, or wrapper for a claw-style harness would live after the local baseline is stable.

## 8. Phase 1 Deliverables

A correct Phase 1 result means all of the following exist:

- a repo with the structure above
- one working local backend adapter
- one active config that points to a local model
- one CLI command that accepts a prompt and returns output
- one smoke test that proves the end-to-end path works
- one setup document that a future agent or developer can follow without guessing

## 9. Definition of Done

The initial demo is complete when:

1. `env_check.sh` passes
2. the local backend starts successfully
3. `smoke_test.sh` returns a model response
4. the CLI can run a short coding prompt
5. changing `config/active.yaml` can switch backend or model profile without code edits
6. the repo is clean enough that a second implementation pass can add editor integration or a stronger backend without reorganizing everything

## 10. Explicit Non-Goals for the Initial Demo

Do **not** expand Phase 1 into:

- distributed inference
- multi-GPU scheduling
- training or fine-tuning
- complex multi-agent delegation
- full IDE plugin development
- cloud orchestration
- generalized autonomous software engineering

## 11. Upgrade Paths Preserved by This Design

This structure keeps open the ability to later add:

- more RAM on the laptop
- a stronger desktop/server backend on the local network
- a remote HTTP model endpoint
- a claw-style harness integration
- codebase indexing and retrieval
- editor-side workflows
- task-specific prompt presets and coding modes

## 12. Principal Outcome

The result is not just “a local model running.”

The result is a **cleanly-shaped local AI coding project** with:
- a stable architecture
- a reproducible demo
- a controlled future upgrade path
- a clear separation between your code, third-party runtime code, and local model assets
