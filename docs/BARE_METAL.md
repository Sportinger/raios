# Bare-Metal Bring-Up

raiOS is still a VM-first MVP. Bare-metal testing currently means: boot the
same Limine/UEFI stage-0 payload from a USB stick and verify framebuffer plus
device inventory and minimal keyboard/mouse input. Network and provider calls
are expected in the VM through the e1000 test path; real hardware networking
depends on matching NIC support. On the Surface Pro 4 the built-in Marvell
88W8897 WiFi is DETECTED on the PCIe bus (the driver skeleton — pure firmware
sequencer + inert hardware shell — sees the chip but does not operate it yet;
no firmware download). See docs/marvell-88w8897-wifi-driver-scope.md.

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

- USB-HID support is minimal and works best with a simple boot-protocol keyboard/
  mouse present at boot on a root port. USB HUB enumeration IS implemented in
  `usb.rs` (hub descriptor incl. superspeed, port power/reset/status, the xHCI
  route string, and recursive downstream HID enumeration; failures are captured in
  `hub_last_error`) — so devices behind a hub CAN enumerate. But it is not yet
  robust across all real hubs (on the Surface Pro 4's hub it reached a
  `hub_last_error` and the USB row went DEGRADED), and full hotplug + non-boot HID
  report descriptors + layout selection are still limited. Hub enumeration is
  VM-testable via QEMU `-device usb-hub` + a `usb-kbd` behind it.
- PS/2/i8042 fallback is conservative: it no longer marks input ready merely
  because an i8042-compatible status port exists.
- Intel e1000 exists and is used in the bare-metal-style VM. Broader real
  hardware NIC coverage is still missing (the Surface's Marvell 88W8897 WiFi is
  detected but not yet driven).
- In-OS provider transport has reached OpenAI over DNS/TCP/TLS/HTTPS in QEMU.
  The normal path now has an OpenAI SPKI pin verifier and a legacy
  leaf-certificate pin verifier; without a configured pin it still fails closed
  before API-key copy or HTTPS write.
- Persistence IS implemented (M7 durable promotion two-boot proof; M9 durable
  memory records survive reboot) — but it only ever writes to raiOS's OWN
  GPT-marked reclog region on an AHCI disk, command-triggered (never on boot). On
  a machine without that provisioned region (a normal boot-USB, or a Surface whose
  internal disk isn't AHCI/isn't raiOS-formatted) durable writes fail-closed to
  RAM-only, so a plain observe-boot writes nothing to the host disk. No secure
  secret store yet.

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
share that USB or its image. The normal provider path still fails closed at TLS
trust unless `-EmbedOpenAiSpkiPinFromEnv` supplies a current
`OPENAI_SPKI_SHA256` or `-EmbedOpenAiCertPinFromEnv` supplies a current
`OPENAI_CERT_SHA256`. Add `-AllowUnverifiedOpenAiTls` only for a local
development smoke USB. The USB script refuses `-SkipBuild` with provider
key/trust build flags so they cannot silently be omitted or copied from stale
staging state.

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

## Real-Hardware Observations — Surface Pro 4 (2026-07-08)

First boot on the real Surface Pro 4 at full M2–M12 maturity (booted from a
USB stick, latest kernel). Observed on the Stage-0 status UI:

- Framebuffer, USB (xHCI) READY, RNG READY. The full UI (raiOS / Direct AI Host
  / AI / CONSOLE / SET tabs) rendered.
- **WiFi DETECTED** — the built-in Marvell 88W8897 is found on the PCIe bus
  (QEMU always showed MISSING). Not yet driven (no firmware download).
- Net MISSING (no ethernet on the Surface).
- Input started MISSING, then was brought up to WORKING over a debugging session
  (see below). The Surface has ONE USB-A port (needed for the boot stick), no
  serial port, and the Type Cover is not a USB-HID device.

### USB HID real-hardware bring-up (2026-07-08) — WORKING, with two open edges

Real USB keyboard + mouse now enumerate and deliver input on the Surface's xHCI
(they always worked in QEMU; the gaps below were all real-hardware-only that
QEMU's lenient emulation masks). The fixes, in order, each moved the failure to
the next stage until input worked:

1. **TT / split-transaction slot-context fields** (`write_slot_context`): set the
   TT Hub Slot ID / TT Port Number only for Low/Full-speed devices behind a
   High-speed hub (zero otherwise) — Address Device for a hub child was failing
   with completion code 4 (USB Transaction Error).
2. **Best-effort `SET_PROTOCOL`/`SET_IDLE`**: real keyboards may STALL these
   optional HID requests; aborting enumeration on them dropped the device. Plus
   `CONTROL_BUFFER_LEN` 256 → 1024 for large composite HID config descriptors.
3. **ep0 Max Packet Size correction** (THE keyboard fix): a Full-speed device's
   ep0 `bMaxPacketSize0` may be 8/16/32/**64**; raiOS assumed 8 and the first
   full descriptor read triggered a **Babble (CC3)**. Now it reads the first 8
   bytes, then issues an Evaluate Context to set the real MPS before continuing.
4. **Command-ring wrap** (THE mouse-direct fix): `execute_command` never wrapped
   the command ring; each error-recovery burns 2 slots, so a mouse erroring while
   moving exhausted the 64-slot ring in ~1 s → every later command failed → all
   input died until a replug re-initialised the controller. The ring is now a
   proper cyclic ring with a LINK TRB.

An on-screen `ENUM …` trace (USB HOTPLUG row) reports the exact enumeration stage
+ completion codes + VID:PID + ep0 MPS, and the default boot view is now CONSOLE
so a keyboard-less user sees full diagnostics in one photo (no scrolling / no
input needed). Status row also carries `RCV<n> ICC<cc>` (recovery count + last
interrupt error code) — but note it currently truncates off the right edge on the
1280×800 panel.

**Working now:** keyboard direct + keyboard behind a hub; mouse direct.

**Open TODO (real-hardware-only, silent — no error code — hard to reproduce in
QEMU):**
- **Mouse behind a hub stalls:** works briefly, then the pointer interrupt IN
  (Full-speed, split transactions through the hub TT) silently stops with no
  error CC; only unplug/replug (full USB re-init) revives it. Likely needs the
  interrupt endpoint to be detected-as-stopped and re-armed without a replug.
- **Very fast keyboard input freezes** input (not the `input.rs` RING — it drops
  oldest, no panic; not a `usb::STATE` re-entrancy deadlock). Cause still open.
- Make the `RCV/ICC` (and any new interrupt-endpoint) diagnostics fit on-screen.

Other next hardware-facing work: the Marvell 88W8897 WiFi firmware-download
bring-up (needs the physical device).
