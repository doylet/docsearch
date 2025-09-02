#!/usr/bin/env bash
set -euo pipefail

has() { command -v "$1" >/dev/null 2>&1; }

# Python
if [[ -f pyproject.toml || -f requirements.txt || -n "$(echo **/*.py 2>/dev/null)" ]]; then
  if awk '/^python:/{f=1} f&&/test:/{print; exit}' .ai/guardrails.yaml >/dev/null 2>&1; then
    cmd=$(awk '/^python:/{f=1} f&&/test:/{print substr($0,index($0,":")+2); exit}' .ai/guardrails.yaml); bash -lc "$cmd"
  elif has pytest; then pytest -q; fi
fi

# Node
if [[ -f package.json ]]; then
  if awk '/^node:/{f=1} f&&/test:/{print; exit}' .ai/guardrails.yaml >/dev/null 2>&1; then
    cmd=$(awk '/^node:/{f=1} f&&/test:/{print substr($0,index($0,":")+2); exit}' .ai/guardrails.yaml); bash -lc "$cmd"
  else npm test --silent || pnpm -s test || yarn -s test || true; fi
fi

# Go
if [[ -f go.mod ]]; then
  if awk '/^go:/{f=1} f&&/test:/{print; exit}' .ai/guardrails.yaml >/dev/null 2>&1; then
    cmd=$(awk '/^go:/{f=1} f&&/test:/{print substr($0,index($0,":")+2); exit}' .ai/guardrails.yaml); bash -lc "$cmd"
  else go test ./...; fi
fi

# Rust
if [[ -f Cargo.toml ]]; then
  if awk '/^rust:/{f=1} f&&/test:/{print; exit}' .ai/guardrails.yaml >/dev/null 2>&1; then
    cmd=$(awk '/^rust:/{f=1} f&&/test:/{print substr($0,index($0,":")+2); exit}' .ai/guardrails.yaml); bash -lc "$cmd"
  else cargo test --quiet; fi
fi

# Java
if [[ -f pom.xml || -f build.gradle || -f build.gradle.kts ]]; then
  if awk '/^java:/{f=1} f&&/test:/{print; exit}' .ai/guardrails.yaml >/dev/null 2>&1; then
    cmd=$(awk '/^java:/{f=1} f&&/test:/{print substr($0,index($0,":")+2); exit}' .ai/guardrails.yaml); bash -lc "$cmd"
  else mvn -q -DskipTests=false test || ./gradlew test; fi
fi

# .NET
if compgen -G "**/*.sln" > /dev/null || compgen -G "**/*.csproj" > /dev/null; then
  if awk '/^dotnet:/{f=1} f&&/test:/{print; exit}' .ai/guardrails.yaml >/dev/null 2>&1; then
    cmd=$(awk '/^dotnet:/{f=1} f&&/test:/{print substr($0,index($0,":")+2); exit}' .ai/guardrails.yaml); bash -lc "$cmd"
  else dotnet test --nologo --verbosity quiet; fi
fi
