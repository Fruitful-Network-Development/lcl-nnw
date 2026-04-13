# llama.cpp runtime scaffold

Single-runtime v1 launcher for local `llama-server` usage.

## Expected env file

Copy template:

```bash
cp runtimes/llama_cpp/.env.example runtimes/llama_cpp/.env
```

Set:

- `LLAMA_CPP_SERVER_BIN`
- `LLAMA_CPP_MODEL_PATH`
- optional host/port/context values

## Startup

```bash
bash runtimes/llama_cpp/start.sh
```
