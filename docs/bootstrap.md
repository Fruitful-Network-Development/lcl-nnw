# Bootstrap (minimal)

From repo root:

```bash
bash ops/scripts/bootstrap.sh
cp runtimes/llama_cpp/.env.example runtimes/llama_cpp/.env
```

Then edit `runtimes/llama_cpp/.env`:

- `LLAMA_CPP_SERVER_BIN` (path to `llama-server` binary)
- `LLAMA_CPP_MODEL_PATH` (local GGUF under `data/models/`)

Start runtime:

```bash
bash runtimes/llama_cpp/start.sh
```

Optional gateway check:

```bash
cd gateway
cargo check
```
