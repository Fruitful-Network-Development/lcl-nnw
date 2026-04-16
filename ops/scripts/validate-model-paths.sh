#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

normalize_model_path() {
  local raw="$1"
  raw="${raw#./}"
  echo "$raw"
}

extract_model_dir() {
  local model_path
  model_path="$(normalize_model_path "$1")"
  if [[ "$model_path" != data/models/* ]]; then
    fail "path must start with data/models/: $1"
  fi

  local remainder="${model_path#data/models/}"
  echo "${remainder%%/*}"
}

is_slug_dir() {
  [[ "$1" =~ ^[a-z0-9][a-z0-9.-]*$ ]]
}

declare -A manifest_dirs=()

while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  path="${line#*=}"
  path="${path%\"}"
  path="${path#\"}"

  dir="$(extract_model_dir "$path")"
  is_slug_dir "$dir" || fail "manifest directory is not lowercase slug: $dir"
  manifest_dirs["$dir"]=1
done < <(sed -n 's/^local_weight_path\s*=\s*"\(.*\)"/local_weight_path="\1"/p' "$ROOT_DIR"/model_registry/models/*.toml)

check_env_model_path() {
  local file="$1"
  local var="$2"
  local value
  value="$(sed -n "s/^${var}=//p" "$file")"
  [[ -n "$value" ]] || fail "missing ${var} in ${file#$ROOT_DIR/}"

  local dir
  dir="$(extract_model_dir "$value")"
  is_slug_dir "$dir" || fail "${var} directory is not lowercase slug in ${file#$ROOT_DIR/}: $dir"

  if [[ -z "${manifest_dirs[$dir]+x}" ]]; then
    fail "${var} uses unknown model directory '${dir}' in ${file#$ROOT_DIR/}"
  fi

  if [[ "$value" != *.gguf ]]; then
    fail "${var} must point to a .gguf file in ${file#$ROOT_DIR/}"
  fi
}

check_env_model_path "$ROOT_DIR/ops/env/local-model-paths.example" "LEAD_MODEL_PATH"
check_env_model_path "$ROOT_DIR/ops/env/local-model-paths.example" "REASONING_MODEL_PATH"
check_env_model_path "$ROOT_DIR/runtimes/llama_cpp/.env.example" "LLAMA_CPP_MODEL_PATH"

echo "Model path validation passed."
