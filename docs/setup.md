# Setup (current repository state)

This repo already contains tracked source scaffolding and a vendored `llama.cpp` tree at:

- `runtimes/llama_cpp/llama.cpp`

No submodule initialization is required for the current default flow.

## 1) Bootstrap local directories

```bash
bash ops/scripts/bootstrap.sh
```

## 2) Configure local runtime values

```bash
cp runtimes/llama_cpp/.env.example runtimes/llama_cpp/.env
```

Set:

- `LLAMA_CPP_SERVER_BIN`
- `LLAMA_CPP_MODEL_PATH`

Use `ops/env/local-model-paths.example` as the source-of-truth template for model paths.

## 3) Start local llama.cpp server

```bash
bash runtimes/llama_cpp/start.sh
```

## 4) (Optional) validate gateway build

```bash
cd gateway
cargo check
```
