#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)
cd "$REPO_ROOT"

TOOLCHAIN="nightly-2024-10-15"
TARGET="$REPO_ROOT/seed-kernel/x86_64-seed.json"

if ! rustup toolchain list | grep -q "$TOOLCHAIN"; then
  rustup toolchain install "$TOOLCHAIN" --component rust-src --component llvm-tools-preview
fi

cargo "+${TOOLCHAIN}" -Zbuild-std=core,compiler_builtins,alloc \
  check --target "$TARGET" -p seed-kernel
