#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

mkdir -p \
  "$ROOT_DIR/.claw" \
  "$ROOT_DIR/.claw/sessions" \
  "$ROOT_DIR/data/models" \
  "$ROOT_DIR/data/caches" \
  "$ROOT_DIR/ops/logs"

echo "Bootstrap complete."
echo "  Runtime config   $ROOT_DIR/.claw/settings.json"
echo "  Session store    $ROOT_DIR/.claw/sessions"
echo "  Model cache      $ROOT_DIR/data/models"
if [[ -f "$ROOT_DIR/.claw/settings.local.json.example" ]]; then
  echo "  Local overrides  copy .claw/settings.local.json.example to .claw/settings.local.json if you need machine-specific changes"
fi
