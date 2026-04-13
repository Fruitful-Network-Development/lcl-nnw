#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

mkdir -p \
  "$ROOT_DIR/data/models" \
  "$ROOT_DIR/data/sessions" \
  "$ROOT_DIR/data/caches" \
  "$ROOT_DIR/data/vector_store" \
  "$ROOT_DIR/ops/logs"

if [[ ! -f "$ROOT_DIR/ops/env/.env" && -f "$ROOT_DIR/ops/env/.env.example" ]]; then
  cp "$ROOT_DIR/ops/env/.env.example" "$ROOT_DIR/ops/env/.env"
  echo "Created ops/env/.env from template."
fi

echo "Bootstrap complete."
