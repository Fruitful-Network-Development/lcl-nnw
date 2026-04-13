#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ENV_FILE="$ROOT_DIR/runtimes/llama_cpp/.env"

if [[ -f "$ENV_FILE" ]]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
fi

: "${LLAMA_CPP_SERVER_BIN:=llama-server}"
: "${LLAMA_CPP_HOST:=127.0.0.1}"
: "${LLAMA_CPP_PORT:=8080}"
: "${LLAMA_CPP_MODEL_PATH:=./data/models/REPLACE_ME.gguf}"
: "${LLAMA_CPP_CTX_SIZE:=8192}"

cd "$ROOT_DIR"
echo "Starting llama.cpp backend at ${LLAMA_CPP_HOST}:${LLAMA_CPP_PORT}"
"$LLAMA_CPP_SERVER_BIN" \
  -m "$LLAMA_CPP_MODEL_PATH" \
  --host "$LLAMA_CPP_HOST" \
  --port "$LLAMA_CPP_PORT" \
  -c "$LLAMA_CPP_CTX_SIZE"
