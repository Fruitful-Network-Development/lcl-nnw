#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONFIG_FILE="$ROOT_DIR/runtimes/embeddings/config.toml"

if [[ ! -f "$CONFIG_FILE" ]]; then
  echo "No runtimes/embeddings/config.toml found. Copy config.example.toml first."
  exit 1
fi

echo "Embedding backend stub. Integrate your preferred local embedding server here."
echo "Using config: $CONFIG_FILE"
