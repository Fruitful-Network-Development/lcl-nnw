# Bootstrap

From repo root:

```bash
bash ops/scripts/bootstrap.sh
cp ops/env/.env.example ops/env/.env   # if not already created
cp runtimes/llama_cpp/.env.example runtimes/llama_cpp/.env
```

Then validate gateway compiles:

```bash
cd gateway
cargo check
```

Run gateway status flow:

```bash
cd gateway
cargo run
```
