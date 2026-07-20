#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)
IMG="$REPO_ROOT/release/raios-stage0.img"
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
  -cpu ${QEMU_CPU_MODEL:-max} \
  -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
  -drive if=pflash,format=raw,file="$VARS_COPY" \
  -drive format=raw,if=ide,file="$IMG" \
  -netdev user,id=net0 \
  -device e1000,netdev=net0,mac=52:54:00:12:34:56 \
  -device qemu-xhci,id=xhci \
  -device usb-kbd,bus=xhci.0 \
  -device usb-mouse,bus=xhci.0 \
  -serial stdio \
  -display none \
  -no-reboot \
  -smp 2
