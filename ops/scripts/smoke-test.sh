#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONFIG_DIR="$ROOT_DIR/.claw"

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to read .claw settings." >&2
  exit 1
fi

cd "$ROOT_DIR"

run_lcl() {
  cargo run -p rusty-claude-cli --bin lcl -- "$@"
}

provider_base_url="$(
  python3 - "$CONFIG_DIR/settings.json" "$CONFIG_DIR/settings.local.json" <<'PY'
import json
import sys
from pathlib import Path

def merge(base, overlay):
    for key, value in overlay.items():
        if isinstance(base.get(key), dict) and isinstance(value, dict):
            merge(base[key], value)
        else:
            base[key] = value
    return base

merged = {}
for raw_path in sys.argv[1:]:
    path = Path(raw_path)
    if not path.exists():
        continue
    contents = path.read_text().strip()
    if not contents:
        continue
    merge(merged, json.loads(contents))

print(merged.get("lcl", {}).get("provider", {}).get("baseUrl", ""))
PY
)"

echo "== lcl doctor =="
run_lcl doctor
echo

echo "== lcl status =="
run_lcl status
echo

if [[ -n "$provider_base_url" ]] && curl --silent --fail "${provider_base_url%/}/models" >/dev/null 2>&1; then
  echo "== lcl prompt =="
  run_lcl --output-format json prompt "Reply with OK only."
  echo
else
  echo "== lcl prompt =="
  echo "Skipping prompt smoke test because ${provider_base_url:-<unset>} is not reachable."
fi
