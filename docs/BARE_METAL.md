# Bare-Metal Bring-Up

SeedOS is still a VM-first MVP. Bare-metal testing currently means: boot the
same Limine/UEFI stage-0 payload from a USB stick and verify framebuffer plus
device inventory and minimal keyboard input. Network and provider calls are not
expected to work on real hardware yet.

## Current Bare-Metal Expectations

Likely to work on a UEFI x86_64 machine:

- UEFI removable-media boot through `EFI\BOOT\BOOTX64.EFI`.
- Limine handoff to the Rust kernel.
- Limine framebuffer status UI.
- RDRAND entropy on modern CPUs.
- xHCI controller detection in the `USB-XHCI` status row.
- Keyboard input from virtio in QEMU, PS/2/i8042 fallback where present, or a
  directly attached USB HID boot keyboard on xHCI.
- The `USB-XHCI` row shows `HID READY`, `HID NONE`, or `HID ERROR`. `HID NONE`
  means the controller was detected, but no directly attached boot keyboard was
  enumerated.

Expected gaps:

- USB-HID support is minimal: direct root-port boot keyboards only. USB hubs,
  non-boot HID report descriptors, hotplug, and layout selection are not
  implemented yet.
- PS/2/i8042 fallback is conservative: it no longer marks input ready merely
  because an i8042-compatible status port exists.
- No real NIC drivers yet; only virtio-net exists.
- No in-OS HTTPS/TLS/OpenAI client yet.
- No persistence or secure secret store yet.
- The OpenAI bridge still runs on the Windows host in the VM workflow.

## List USB Disks

Run from normal PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\list-usb-disks.ps1
```

Use `-IncludeAll` only for inspection. Do not choose a Windows boot/system disk.

## Write A Boot USB

Run from an elevated Administrator PowerShell. Replace `<N>` with the USB disk
number from `list-usb-disks.ps1`.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\write-stage0-usb.ps1 -DiskNumber <N> -ConfirmErase "ERASE DISK <N>"
```

This erases the selected USB disk, creates a 512 MB FAT32 boot partition, and
copies `release\esp` to it.

For a local-only OpenAI-default USB:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\write-stage0-usb.ps1 -DiskNumber <N> -ConfirmErase "ERASE DISK <N>" -EmbedOpenAiApiKeyFromEnv
```

That embeds `OPENAI_API_KEY` into the local kernel copied to the USB. Do not
share that USB or its image.

## First Boot Checklist

1. Boot via the machine's UEFI boot menu.
2. Pick the USB entry, ideally the explicit UEFI entry.
3. Confirm the Stage-0 status UI appears.
4. Try typing `help`.
5. Check the `USB-XHCI` row. `READY` means the controller was detected and the
   port count was read.
6. Check the `INPUT` row. `USB HID BOOT KEYBOARD` means the direct USB keyboard
   path is active.
7. If input is missing, try a direct keyboard connection without a hub, another
   USB port, or firmware legacy USB keyboard support. A hub or non-boot HID
   device still needs more USB stack work.
