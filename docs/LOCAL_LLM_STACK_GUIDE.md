# Local LLM Coding Stack — Principal Guide

## 1. Objective

Build a first local LLM coding stack that:

- runs on the current laptop first
- uses a CPU-friendly local runtime
- keeps the backend swappable
- exposes one simple interface for coding tasks
- creates a repo that future work can extend without restructuring

This guide is intentionally aimed at a **small, correct, modular demo** rather than an ambitious first pass.

## 2. The Core Decision

The correct initial target is:

- **local CPU inference first**
- **quantized model first**
- **CLI interface first**
- **provider abstraction first**
- **Claw-style integration later, not first**

Reason:

If you try to begin with the full stack all at once, it becomes unclear whether failures come from:
- the model
- the runtime
- the abstraction layer
- the agent harness
- the interface
- or the host machine

So the project should be built in this order:

```text
1. Local runtime works
2. Adapter contract works
3. CLI works
4. Config switching works
5. Optional harness/editor integrations come later
```

## 3. Chosen Baseline Shape

### Baseline backend
Use `llama.cpp` as the initial runtime.

### Baseline application layer
Build a thin project-owned adapter and CLI around it.

### Baseline model class
Use a small or modest quantized coding-capable model in GGUF format.

### Baseline interface
Use a terminal CLI.

### Baseline success test
A single command should accept a coding prompt and return a useful answer.

## 4. Why This Is the Right First Shape

This gives you four things immediately:

1. you learn whether local inference on this hardware is tolerable
2. you keep your own repo in control of the abstraction layer
3. you avoid binding the whole project to one upstream harness too early
4. you preserve the option to route to a stronger backend later

## 5. Recommended Project Build Order

### Stage A — Create the repo skeleton
Create the repo and lay down the final directory shape before writing logic.

Run from the parent directory where you want the repo to live:

```bash
mkdir -p ~/dev/local-llm-coder
cd ~/dev/local-llm-coder
mkdir -p config/models config/prompts docs scripts \
  src/local_llm_coder/engine \
  src/local_llm_coder/providers \
  src/local_llm_coder/repo \
  src/local_llm_coder/util \
  tests runtime/vendor runtime/state/logs runtime/state/sessions runtime/state/cache \
  models integrations/editor integrations/claw
```

Then initialize Git:

```bash
cd ~/dev/local-llm-coder
git init
```

### Stage B — Add principal project files
Create the first root files:

```bash
cd ~/dev/local-llm-coder
touch README.md PROJECT_STATE.md .gitignore .env.example
```

Minimum `.gitignore` content:

```bash
cd ~/dev/local-llm-coder
cat > .gitignore <<'EOF'
models/*
!models/README.md
!models/.gitkeep
runtime/state/
.env
__pycache__/
.pytest_cache/
.venv/
EOF
```

### Stage C — Vendor or clone the runtime
Clone `llama.cpp` into the vendor area.

```bash
cd ~/dev/local-llm-coder/runtime/vendor
git clone https://github.com/ggml-org/llama.cpp.git
```

### Stage D — Build the local runtime
Build `llama.cpp` from its own directory.

```bash
cd ~/dev/local-llm-coder/runtime/vendor/llama.cpp
cmake -B build
cmake --build build -j
```

If your system needs dependencies first:

```bash
cd ~/dev/local-llm-coder
sudo apt update
sudo apt install -y build-essential cmake curl git python3 python3-venv python3-pip
```

### Stage E — Add a model profile, not hardcoded paths
Create a model config file.

```bash
cd ~/dev/local-llm-coder
cat > config/models/cpu-demo.yaml <<'EOF'
provider: llamacpp
mode: local_server
model_path: ./models/REPLACE_ME.gguf
host: 127.0.0.1
port: 8080
n_ctx: 4096
temperature: 0.2
max_tokens: 512
EOF
```

Create the active config:

```bash
cd ~/dev/local-llm-coder
cat > config/active.yaml <<'EOF'
profile: cpu-demo
EOF
```

### Stage F — Add project-owned provider contract
Create the provider base contract.

```bash
cd ~/dev/local-llm-coder
cat > src/local_llm_coder/providers/base.py <<'EOF'
from abc import ABC, abstractmethod

class Provider(ABC):
    @abstractmethod
    def healthcheck(self):
        raise NotImplementedError

    @abstractmethod
    def model_info(self):
        raise NotImplementedError

    @abstractmethod
    def generate(self, prompt, system=None, options=None):
        raise NotImplementedError
EOF
```

This is the main seam that protects the rest of the project from backend churn.

### Stage G — Implement the first backend adapter
The first concrete adapter should target `llama.cpp` in one mode only.

Use **one mode first**:
- local HTTP server mode

Do not support multiple runtime modes on day one.

Your first adapter should only need to:
- read the active profile
- call the local server
- return plain text output
- expose `healthcheck()`

### Stage H — Start the local runtime in a repeatable way
Create a script so the runtime is not started manually each time.

```bash
cd ~/dev/local-llm-coder
cat > scripts/run_local_server.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODEL_PATH="$ROOT_DIR/models/REPLACE_ME.gguf"
SERVER_BIN="$ROOT_DIR/runtime/vendor/llama.cpp/build/bin/llama-server"

cd "$ROOT_DIR"
"$SERVER_BIN" \
  -m "$MODEL_PATH" \
  --host 127.0.0.1 \
  --port 8080 \
  -c 4096
EOF
chmod +x scripts/run_local_server.sh
```

### Stage I — Build the CLI only after the adapter exists
Your CLI should be thin.

Its responsibilities:
- accept prompt text
- optionally accept a task preset
- load active config
- call the provider adapter
- print the response

It should **not** contain model-specific logic.

### Stage J — Add a smoke test
Create one script whose only job is to prove the end-to-end path works.

```bash
cd ~/dev/local-llm-coder
cat > scripts/smoke_test.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
python3 -m src.local_llm_coder.main prompt "Write a Python function that reverses a string."
EOF
chmod +x scripts/smoke_test.sh
```

## 6. The Correct File Tree Mental Model

Think about the repo as five distinct ownership zones.

```text
1. Your project logic
2. Your project configuration
3. Third-party runtime code
4. Local model assets
5. Runtime state and logs
```

Mapped to directories:

```text
src/           -> your logic
config/        -> your switching and presets
runtime/vendor -> third-party runtime
models/        -> local assets
runtime/state/ -> logs, cache, sessions
```

That separation is what keeps the repo understandable.

## 7. What Not To Do On Day One

Do not begin by adding:

- multiple providers
- retrieval/indexing
- editor plugins
- Claw integration
- remote server support
- tool execution
- background agents
- complex memory or session orchestration

Those are valid later, but they blur the baseline.

## 8. Where Claw Code Fits

Claw Code should be treated as a **later integration target**, not the foundation of the first demo.

Why:

- your first task is proving local inference and your project-owned abstraction
- once that works, you can decide whether Claw is worth bridging into your stack
- if you use it first, you risk designing your whole architecture around upstream assumptions

So the right role for it is:

```text
Phase 1: local runtime + adapter + CLI
Phase 2: task presets + repository context helpers
Phase 3: optional Claw bridge under integrations/claw/
```

## 9. Minimal Required Internal Modules

At the end of the first pass, these are the only modules that truly matter:

```text
src/local_llm_coder/
├── main.py
├── cli.py
├── providers/
│   ├── base.py
│   └── llamacpp.py
├── util/
│   ├── config.py
│   └── paths.py
└── engine/
    └── prompts.py
```

Everything else can be skeletal.

## 10. The First Useful End-to-End User Flow

The intended first user flow is:

```text
1. Start local model server
2. Run CLI prompt
3. Receive answer
4. Swap config profile later without changing code
```

That is the proof that the architecture is correct.

## 11. How To Judge Whether the Demo Is Valuable

The first demo is valuable if it answers these questions:

- Can the laptop tolerate local inference at all?
- Is the latency acceptable for short coding tasks?
- Does the abstraction layer feel stable?
- Can you switch model/backend by config rather than code?
- Does the repo already look like something future work can inhabit cleanly?

If the answer is yes, the demo succeeded.

## 12. What To Upgrade First If Performance Is Weak

If the demo works but feels weak, upgrade in this order:

1. use a smaller or better quantized model
2. reduce context and generation length defaults
3. add laptop RAM if memory pressure is the issue
4. move the backend to a stronger machine on the local network while keeping the same adapter contract
5. only then consider more advanced harnesses or GPU-oriented builds

## 13. Principal End State Summary

The intended project state is:

- one repo
- one clean provider seam
- one working local backend
- one CLI
- one active config
- one smoke test
- one preserved path toward stronger local or remote backends

That is the right first milestone.
