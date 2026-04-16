# Setup

This repository is already a scaffold with local runtime/data directories.

## Current state

- `runtimes/llama_cpp/llama.cpp` is **vendored directly** in this repository.
- This project is **not currently using Git submodules** for `llama.cpp` or `third_party/claw-code`.
- There is no required `third_party/claw-code` checkout in the current scaffold.

If the project later migrates to submodules, those instructions should be added in a dedicated migration section.

## Bootstrap (single source of truth)

From the repository root:

```bash
bash ops/scripts/bootstrap.sh
```

This creates local runtime directories (for example `data/models/`, `data/sessions/`, and `ops/logs/`) and copies `ops/env/.env.example` to `ops/env/.env` when needed.

## Local model placement

Model weights are local-only and should be placed under `data/models/`.

Example model paths:

- `data/models/qwen2.5-coder-7b-instruct/Qwen2.5-Coder-7B-Instruct.Q4_K_M.gguf`
- `data/models/deepseek-r1-distill-qwen-7b/DeepSeek-R1-Distill-Qwen-7B.Q4_K_M.gguf`

Do not commit GGUF or other large model artifacts to Git.
