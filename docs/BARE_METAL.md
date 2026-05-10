# Bare-Metal Bring-Up

SeedOS is still a VM-first MVP. Bare-metal testing currently means: boot the
same Limine/UEFI stage-0 payload from a USB stick and verify framebuffer plus
device inventory and minimal keyboard/mouse input. Network and provider calls
are expected in the VM through the e1000 test path; real hardware networking
depends on matching NIC support.

## Current Bare-Metal Expectations

Likely to work on a UEFI x86_64 machine:

- UEFI removable-media boot through `EFI\BOOT\BOOTX64.EFI`.
- Limine handoff to the Rust kernel.
- Limine framebuffer status UI.
- RDRAND entropy on modern CPUs.
- xHCI controller detection in the `USB-XHCI` status row.
- Keyboard input from a directly attached USB HID boot keyboard on xHCI or
  PS/2/i8042 fallback where present.
- Pointer input from a directly attached USB HID boot mouse on xHCI.
- The `USB-XHCI` row shows separate keyboard and mouse readiness. `KBD NONE` or
  `MOUSE NONE` means the controller was detected, but that direct boot HID
  device was not enumerated.

Expected gaps:

- USB-HID support is minimal: direct root-port boot keyboards only. USB hubs,
  direct root-port boot mice only. USB hubs, non-boot HID report descriptors,
  hotplug, and layout selection are not implemented yet.
- PS/2/i8042 fallback is conservative: it no longer marks input ready merely
  because an i8042-compatible status port exists.
- Intel e1000 exists and is used in the bare-metal-style VM. Broader real
  hardware NIC coverage is still missing.
- In-OS provider work currently reaches OpenAI over DNS/TCP/TLS/HTTPS in QEMU.
  The MVP TLS path still needs certificate verification or provider pinning
  before serious use.
- No persistence or secure secret store yet.

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
6. Check the `INPUT` row. `USB HID KEYBOARD + POINTER` means direct USB keyboard
   and mouse input are active.
7. If input is missing, try a direct keyboard connection without a hub, another
   USB port, or firmware legacy USB keyboard support. A hub or non-boot HID
   device still needs more USB stack work.
