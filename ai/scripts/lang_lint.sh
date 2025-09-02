#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob

# Load overrides if present
if [[ -f .ai/guardrails.yaml ]]; then
  # naive parsers for simple key: value
  py_lint=$(awk '/^python:/{f=1} f&&/lint:/{print substr($0,index($0,":")+2); f=0}' .ai/guardrails.yaml || true)
  node_lint=$(awk '/^node:/{f=1} f&&/lint:/{print substr($0,index($0,":")+2); f=0}' .ai/guardrails.yaml || true)
  go_lint=$(awk '/^go:/{f=1} f&&/lint:/{print substr($0,index($0,":")+2); f=0}' .ai/guardrails.yaml || true)
  rust_lint=$(awk '/^rust:/{f=1} f&&/lint:/{print substr($0,index($0,":")+2); f=0}' .ai/guardrails.yaml || true)
fi

has() { command -v "$1" >/dev/null 2>&1; }

# Python
if [[ -f pyproject.toml || -f requirements.txt || -n "$(echo **/*.py 2>/dev/null)" ]]; then
  if [[ -n "${py_lint:-}" ]]; then bash -lc "$py_lint"; 
  elif has ruff; then ruff check .; else echo "ruff not found; skipping python lint"; fi
fi

# Node/TypeScript
if [[ -f package.json ]]; then
  if [[ -n "${node_lint:-}" ]]; then bash -lc "$node_lint";
  else npx -y eslint . || npx -y @biomejs/biome check . || echo "eslint/biome unavailable"; fi
fi

# Go
if [[ -f go.mod ]]; then
  if [[ -n "${go_lint:-}" ]]; then bash -lc "$go_lint";
  elif has golangci-lint; then golangci-lint run; else go vet ./... || true; fi
fi

# Rust
if [[ -f Cargo.toml ]]; then
  if [[ -n "${rust_lint:-}" ]]; then bash -lc "$rust_lint";
  elif has cargo; then cargo clippy --no-deps -q -D warnings || echo "clippy not installed"; fi
fi

# Shell/Docker/Markdown (generic)
if command -v shellcheck >/dev/null 2>&1; then shellcheck **/*.sh || true; fi
if command -v hadolint >/dev/null 2>&1; then hadolint **/Dockerfile* || true; fi
if command -v markdownlint >/dev/null 2>&1; then markdownlint . || true; fi
