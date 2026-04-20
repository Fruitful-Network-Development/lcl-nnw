#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONFIG_DIR="$ROOT_DIR/.claw"

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to read .claw settings." >&2
  exit 1
fi

eval "$(
  python3 - "$ROOT_DIR" "$CONFIG_DIR/settings.json" "$CONFIG_DIR/settings.local.json" <<'PY'
import json
import shlex
import sys
from pathlib import Path

root = Path(sys.argv[1])
paths = [Path(path) for path in sys.argv[2:]]

def merge(base, overlay):
    for key, value in overlay.items():
        if isinstance(base.get(key), dict) and isinstance(value, dict):
            merge(base[key], value)
        else:
            base[key] = value
    return base

merged = {}
for path in paths:
    if not path.exists():
        continue
    contents = path.read_text().strip()
    if not contents:
        continue
    merge(merged, json.loads(contents))

lcl = merged.get("lcl", {})
llama = lcl.get("llamaServer", {})
provider = lcl.get("provider", {})

model_path = llama.get("modelPath")
if not model_path:
    raise SystemExit("missing lcl.llamaServer.modelPath in .claw/settings.json")

model_path = Path(model_path)
if not model_path.is_absolute():
    model_path = root / model_path

values = {
    "LLAMA_SERVER_BIN": llama.get("bin", "llama-server"),
    "LLAMA_SERVER_HOST": llama.get("host", "127.0.0.1"),
    "LLAMA_SERVER_PORT": str(llama.get("port", 8080)),
    "LLAMA_SERVER_CTX_SIZE": str(llama.get("ctxSize", 8192)),
    "LLAMA_SERVER_MODEL_PATH": str(model_path),
    "LCL_PROVIDER_BASE_URL": provider.get("baseUrl", ""),
}

for key, value in values.items():
    print(f"{key}={shlex.quote(value)}")
PY
)"

if [[ ! -f "$LLAMA_SERVER_MODEL_PATH" ]]; then
  echo "Configured model path does not exist: $LLAMA_SERVER_MODEL_PATH" >&2
  exit 1
fi

cd "$ROOT_DIR"
echo "Starting llama-server at ${LLAMA_SERVER_HOST}:${LLAMA_SERVER_PORT}"
echo "  Model   $LLAMA_SERVER_MODEL_PATH"
if [[ -n "$LCL_PROVIDER_BASE_URL" ]]; then
  echo "  API     $LCL_PROVIDER_BASE_URL"
fi
exec "$LLAMA_SERVER_BIN" \
  -m "$LLAMA_SERVER_MODEL_PATH" \
  --host "$LLAMA_SERVER_HOST" \
  --port "$LLAMA_SERVER_PORT" \
  -c "$LLAMA_SERVER_CTX_SIZE"
