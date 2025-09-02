#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob

# Ensure Cargo on PATH (hooks don't load your shell rc files)
export PATH="$HOME/.cargo/bin:$PATH"

# Read .ai/guardrails.(yaml|yml)
read_cfg() {
  local lang="$1" key="$2" file=""
  for file in .ai/guardrails.yaml .ai/guardrails.yml; do
    [[ -f "$file" ]] || continue
    if command -v yq >/dev/null 2>&1; then
      yq -r ".${lang}.${key} // \"\"" "$file" 2>/dev/null
    else
      # naive fallback, good enough for simple key: value lines
      awk "/^${lang}:/{f=1} f&&/${key}:/{print substr(\$0,index(\$0,\":\")+2); exit}" "$file" 2>/dev/null
    fi
    return
  done
}

# Strip exactly one *wrapping* quote pair if present, and any CRs
strip_wrapping_quotes() {
  local s="$*"
  s="${s//$'\r'/}"                 # remove CR
  if [[ ${#s} -ge 2 ]]; then
    if [[ ${s:0:1} == '"' && ${s: -1} == '"' ]]; then s="${s:1:-1}"; fi
    if [[ ${s:0:1} == "'" && ${s: -1} == "'" ]]; then s="${s:1:-1}"; fi
  fi
  printf '%s' "$s"
}

# Execute the command by writing it to a temp script (avoids eval/quoting hell)
run_cmd_if_set() {
  local cmd="$1"
  [[ -z "${cmd:-}" ]] && return 0
  cmd="$(strip_wrapping_quotes "$cmd")"
  local tmp
  tmp="$(mktemp)"
  printf '%s\n' "$cmd" > "$tmp"
  bash "$tmp"
  local rc=$?
  rm -f "$tmp"
  return $rc
}

py_lint=$(read_cfg python lint)
node_lint=$(read_cfg node lint)
go_lint=$(read_cfg go lint)
rust_lint=$(read_cfg rust lint)

has() { command -v "$1" >/dev/null 2>&1; }

# Python
if [[ -f pyproject.toml || -f requirements.txt || -n "$(echo **/*.py 2>/dev/null)" ]]; then
  if [[ -n "${py_lint:-}" ]]; then run_cmd_if_set "$py_lint";
  elif has ruff; then ruff check .; else echo "ruff not found; skipping python lint"; fi
fi

# Node/TypeScript
if [[ -f package.json ]]; then
  if [[ -n "${node_lint:-}" ]]; then run_cmd_if_set "$node_lint";
  else npx -y eslint . || npx -y @biomejs/biome check . || echo "eslint/biome unavailable"; fi
fi

# Go
if [[ -f go.mod ]]; then
  if [[ -n "${go_lint:-}" ]]; then run_cmd_if_set "$go_lint";
  elif has golangci-lint; then golangci-lint run; else go vet ./... || true; fi
fi

# Rust
if [[ -f Cargo.toml ]]; then
  if [[ -n "${rust_lint:-}" ]]; then run_cmd_if_set "$rust_lint";
  elif has cargo; then bash -lc "cargo clippy --no-deps --deny warnings -q" || echo "clippy not installed"; fi
fi

# Generic hygiene
if command -v shellcheck   >/dev/null 2>&1; then shellcheck **/*.sh || true; fi
if command -v hadolint     >/dev/null 2>&1; then hadolint **/Dockerfile* || true; fi
if command -v markdownlint >/dev/null 2>&1; then markdownlint . || true; fi
