#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)
IMG="$REPO_ROOT/release/seedos-stage0.img"
OVMF_CODE=${OVMF_CODE:-/usr/share/OVMF/OVMF_CODE.fd}
OVMF_VARS=${OVMF_VARS:-/usr/share/OVMF/OVMF_VARS.fd}

if [[ ! -f "$IMG" ]];
then
  echo "Seed image not found. Run scripts/package-stage0.sh first." >&2
  exit 1
fi

VARS_COPY=$(mktemp "$REPO_ROOT/release/ovmf_vars.XXXXXX.fd")
cp "$OVMF_VARS" "$VARS_COPY"
cleanup() {
  rm -f "$VARS_COPY"
}
trap cleanup EXIT

qemu-system-x86_64 \
  -machine q35 \
  -m 512M \
  -cpu ${QEMU_CPU_MODEL:-qemu64} \
  -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
  -drive if=pflash,format=raw,file="$VARS_COPY" \
  -drive if=none,id=disk0,format=raw,file="$IMG" \
  -device virtio-blk-pci,drive=disk0 \
  -netdev user,id=net0 \
  -device virtio-net-pci,netdev=net0,disable-modern=on,disable-legacy=off \
  -device virtio-rng-pci,disable-modern=on,disable-legacy=off \
  -device virtio-keyboard-pci \
  -device virtio-mouse-pci \
  -serial stdio \
  -display none \
  -no-reboot \
  -smp 2
