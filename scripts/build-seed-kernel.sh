#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)
cd "$REPO_ROOT"

TOOLCHAIN="nightly-2024-10-15"
TARGET="$REPO_ROOT/seed-kernel/x86_64-seed.json"
KERNEL_RUSTFLAGS=(
  "-C" "link-arg=-T$REPO_ROOT/seed-kernel/linker.ld"
  "-C" "relocation-model=static"
  "-C" "code-model=kernel"
  "-C" "force-frame-pointers=yes"
  "-C" "link-arg=--gc-sections"
)

if ! rustup toolchain list | grep -q "$TOOLCHAIN"; then
  rustup toolchain install "$TOOLCHAIN" --component rust-src --component llvm-tools-preview
fi

RUSTFLAGS="${KERNEL_RUSTFLAGS[*]} ${RUSTFLAGS:-}" \
cargo "+${TOOLCHAIN}" -Zbuild-std=core,compiler_builtins,alloc \
  build --locked --target "$TARGET" -p seed-kernel

echo "built target/x86_64-seed/debug/seed-kernel"
