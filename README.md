# lcl-nnw

Minimal offline-first local LLM stack scaffold.

This repository vendors `llama.cpp` source directly under:

- `runtimes/llama_cpp/llama.cpp`

This repository does **not** store model weights in Git. Put local GGUF model files under:

- `data/models/`

Canonical model directory naming convention:
- lowercase slug directories (letters, numbers, dots, and hyphens)
- example: `data/models/qwen2.5-coder-7b-instruct/`

What is included in Git:
- the repo scaffold
- vendored `llama.cpp` source
- model manifests and local path examples
- bootstrap scripts
- local runtime/data directory structure

What is not included in Git:
- GGUF model binaries
- Hugging Face downloads
- Python package caches
- system toolchains

Suggested local lead model path:
`data/models/qwen2.5-coder-7b-instruct/Qwen2.5-Coder-7B-Instruct.Q4_K_M.gguf`

Suggested local reasoning model path:
`data/models/deepseek-r1-distill-qwen-7b/DeepSeek-R1-Distill-Qwen-7B.Q4_K_M.gguf`
