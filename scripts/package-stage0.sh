#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)
ESP_DIR="$REPO_ROOT/release/esp"
IMG="$REPO_ROOT/release/seedos-stage0.img"
BOOTX64_SRC="$REPO_ROOT/vendor/limine/build/share/limine/BOOTX64.EFI"
LIMINE_CD_SRC="$REPO_ROOT/vendor/limine/build/share/limine/limine-uefi-cd.bin"
TMP_BOOTLOADER_DIR=""

if [[ ! -f "$BOOTX64_SRC" ]]; then
  if [[ -f "$ESP_DIR/EFI/BOOT/BOOTX64.EFI" ]]; then
    TMP_BOOTLOADER_DIR=$(mktemp -d)
    cp "$ESP_DIR/EFI/BOOT/BOOTX64.EFI" "$TMP_BOOTLOADER_DIR/BOOTX64.EFI"
    if [[ -f "$ESP_DIR/limine-uefi-cd.bin" ]]; then
      cp "$ESP_DIR/limine-uefi-cd.bin" "$TMP_BOOTLOADER_DIR/limine-uefi-cd.bin"
    fi
    BOOTX64_SRC="$TMP_BOOTLOADER_DIR/BOOTX64.EFI"
    LIMINE_CD_SRC="$TMP_BOOTLOADER_DIR/limine-uefi-cd.bin"
  else
    echo "Limine BOOTX64.EFI not found. Populate vendor/limine or keep release/esp/EFI/BOOT/BOOTX64.EFI available." >&2
    exit 1
  fi
fi

cleanup() {
  if [[ -n "$TMP_BOOTLOADER_DIR" ]]; then
    rm -rf "$TMP_BOOTLOADER_DIR"
  fi
}
trap cleanup EXIT

# Build kernel binary
"$REPO_ROOT/scripts/build-seed-kernel.sh"

# Stage ESP content
rm -rf "$ESP_DIR"
mkdir -p "$ESP_DIR"/EFI/BOOT "$ESP_DIR"/kernel
cp "$BOOTX64_SRC" "$ESP_DIR"/EFI/BOOT/
if [[ -f "$LIMINE_CD_SRC" ]]; then
  cp "$LIMINE_CD_SRC" "$ESP_DIR"/
fi
cp "$REPO_ROOT/seed-kernel/limine/limine.conf" "$ESP_DIR"/
cp "$REPO_ROOT/seed-kernel/limine/limine.conf" "$ESP_DIR"/EFI/BOOT/
cp "$REPO_ROOT/target/x86_64-seed/debug/seed-kernel" "$ESP_DIR"/kernel/kernel.elf

# Create FAT32 disk image
rm -f "$IMG"
truncate -s 64M "$IMG"
mkfs.fat -F32 "$IMG" >/dev/null

mmd -i "$IMG" ::/EFI
mmd -i "$IMG" ::/EFI/BOOT
mmd -i "$IMG" ::/kernel
mcopy -s -i "$IMG" "$ESP_DIR"/EFI/BOOT ::/EFI
mcopy -i "$IMG" "$ESP_DIR"/kernel/kernel.elf ::/kernel
mcopy -i "$IMG" "$ESP_DIR"/limine.conf ::/
