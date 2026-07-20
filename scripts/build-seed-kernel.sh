#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)
cd "$REPO_ROOT"

TOOLCHAIN="nightly-2024-10-15"
TARGET="$REPO_ROOT/seed-kernel/x86_64-seed.json"
INVENTORY_SCRIPT="$REPO_ROOT/scripts/unsafe-inventory.py"
INVENTORY_BASELINE="$REPO_ROOT/docs/architecture/unsafe-inventory-baseline-v2.json"
KERNEL_RUSTFLAGS=(
  "-C" "link-arg=-T$REPO_ROOT/seed-kernel/linker.ld"
  "-C" "relocation-model=static"
  "-C" "code-model=kernel"
  "-C" "force-frame-pointers=yes"
  "-C" "link-arg=--gc-sections"
)

python -B "$INVENTORY_SCRIPT" --check "$INVENTORY_BASELINE"

if ! rustup toolchain list | grep -q "$TOOLCHAIN"; then
  rustup toolchain install "$TOOLCHAIN" --component rust-src --component llvm-tools-preview
fi

RUSTFLAGS="${KERNEL_RUSTFLAGS[*]} ${RUSTFLAGS:-}" \
cargo "+${TOOLCHAIN}" -Zbuild-std=core,compiler_builtins,alloc \
  build --locked --target "$TARGET" -p seed-kernel

TARGET_ROOT=${CARGO_TARGET_DIR:-"$REPO_ROOT/target"}
if [[ "$TARGET_ROOT" != /* ]]; then
  TARGET_ROOT="$REPO_ROOT/$TARGET_ROOT"
fi
ARTIFACT="$TARGET_ROOT/x86_64-seed/debug/seed-kernel"
EVIDENCE="$TARGET_ROOT/x86_64-seed/debug/seed-kernel.unsafe-inventory-build.json"
python -B "$INVENTORY_SCRIPT" --build-evidence --check "$INVENTORY_BASELINE" \
  --artifact "$ARTIFACT" --profile debug --evidence-out "$EVIDENCE"
echo "built $ARTIFACT"
