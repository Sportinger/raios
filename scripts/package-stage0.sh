#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)
ESP_DIR="$REPO_ROOT/release/esp"
IMG="$REPO_ROOT/release/seedos-stage0.img"

# Build kernel binary
"$REPO_ROOT/scripts/build-seed-kernel.sh"

# Stage ESP content
rm -rf "$ESP_DIR"
mkdir -p "$ESP_DIR"/EFI/BOOT "$ESP_DIR"/kernel
cp "$REPO_ROOT/vendor/limine/build/share/limine/BOOTX64.EFI" "$ESP_DIR"/EFI/BOOT/
cp "$REPO_ROOT/vendor/limine/build/share/limine/limine-uefi-cd.bin" "$ESP_DIR"/
cp "$REPO_ROOT/seed-kernel/limine/limine.cfg" "$ESP_DIR"/
cp "$REPO_ROOT/target/x86_64-seed/debug/seed-kernel" "$ESP_DIR"/kernel/seed-kernel.elf

# Create FAT32 disk image
rm -f "$IMG"
truncate -s 64M "$IMG"
mkfs.fat -F32 "$IMG" >/dev/null

mmd -i "$IMG" ::/EFI
mmd -i "$IMG" ::/EFI/BOOT
mmd -i "$IMG" ::/kernel
mcopy -s -i "$IMG" "$ESP_DIR"/EFI/BOOT ::/EFI
mcopy -i "$IMG" "$ESP_DIR"/kernel/seed-kernel.elf ::/kernel
mcopy -i "$IMG" "$ESP_DIR"/limine.cfg ::/
