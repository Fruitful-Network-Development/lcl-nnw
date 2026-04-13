# home-llm (lcl-nnw)

Modular local/home LLM stack scaffold with a stable architecture:

- **Interface shell**: Claw Code lives in `third_party/claw-code/` as the control surface.
- **Stable routing layer**: Rust gateway in `gateway/` is the single orchestration endpoint.
- **Inference backend**: initial runtime target is llama.cpp (`runtimes/llama_cpp/`).
- **Model registry**: source-controlled manifests in `model_registry/`.
- **Retrieval skeleton**: minimal indexing/chunk pipeline scaffolding in `retrieval/`.

## Why Claw is shell-only in this architecture

Claw Code is treated as an integration shell/interface, not the inference backend. This keeps orchestration and backend policy under project control and preserves backend swap-ability over time.

## Gateway role

The Rust gateway provides:
- config loading path from `model_registry/`
- profile-to-model selection stubs
- health/status entry flow
- clear adapter hook location for backend implementations

## Initial model selections (manifest references only)

No model weights are downloaded by this repo.

- Lead: `Qwen2.5-Coder-7B-Instruct` (default quant: `Q4_K_M`)
- Reasoning: `DeepSeek-R1-Distill-Qwen-7B` (default quant: `Q4_K_M`)
- Alternate general/coder: `Llama-3.1-8B-Instruct` (default quant: `Q4_K_M`)
- Embeddings: `bge-small-en-v1.5`

See `model_registry/models/*.toml` and `model_registry/profiles/*.toml`.

## Quick bootstrap

```bash
bash ops/scripts/bootstrap.sh
cp runtimes/llama_cpp/.env.example runtimes/llama_cpp/.env
cd gateway && cargo check
```

Run gateway status stub:

```bash
cd gateway && cargo run
```

## Repository map

```text
third_party/      upstream/integration surfaces
gateway/          rust routing/orchestration scaffold
model_registry/   model/profile/quantization manifests
runtimes/         backend-specific runtime launch/config stubs
prompts/          system/routing/tool prompt surfaces
retrieval/        chunk/index/pipeline skeleton
ui/               cli/web placeholders
ops/              env, scripts, compose, logs
data/             runtime-generated state (gitignored)
```

## Additional docs

- `docs/architecture.md`
- `docs/bootstrap.md`
- `docs/LOCAL_LLM_STACK_GUIDE.md`
- `docs/LOCAL_LLM_STACK_MANIFEST.md`
