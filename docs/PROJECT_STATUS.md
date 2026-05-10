# Project Status

Last verified locally: 2026-05-10 on Windows with QEMU 11.

## Verified Boot State

- Repository path: `C:\Users\admin\Documents\raios2`
- Boot image: `release/seedos-stage0.img`
- Firmware vars seed: `release/ovmf_vars.fd`
- Bootloader: Limine 10 UEFI binary at `release/esp/EFI/BOOT/BOOTX64.EFI`
- Config file: `limine.conf` at ESP root and `EFI/BOOT/limine.conf`
- Kernel path inside image: `/kernel/kernel.elf`

The image boots in QEMU using the Windows PowerShell runner:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

For interactive serial commands, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555
```

For a QEMU xHCI inventory run, add `-UsbXhciInput`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555 -Headless -UsbXhciInput
```

For the bare-metal-style VM profile with USB keyboard, USB mouse, RDRAND, and
e1000 networking, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-baremetal-vm.ps1 -StopExisting
```

Expected xHCI inventory lines in that mode:

```text
usb-xhci: controller @ 00:03.0 detected
usb-xhci: hci 0x0100, ports 8, connected 2
usb-hid: boot keyboard ready on slot 1 endpoint 0x81
usb-hid: boot mouse ready on slot 2 endpoint 0x81
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
```

Expected visible framebuffer UI:

```text
SEEDOS STAGE-0
AGENT HOST: LIVE STATUS
FRAMEBUFFER  READY
ENTROPY      READY
USB-XHCI     READY
NETWORK      CONFIGURED
INPUT        READY
```

Expected useful serial lines:

```text
Seed kernel: early init start
Limine loaded base revision: 3
HHDM offset=0xffff800000000000
Framebuffer response revision: 1
Framebuffer negotiated via Limine
status FRAMEBUFFER: READY - 1280x800 PITCH 5120
status ENTROPY: READY - FILL 64/64 TOTAL 64 SRC RDRAND
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
e1000: device 00:02.0 id=0x100e mmio=0x81040000 size=131072 mac 52:54:00:12:34:56
e1000 network initialised; DHCP polling enabled
DHCP lease acquired: ip 10.0.2.15/24 gw 10.0.2.2 dns ["10.0.2.3"]
status NETWORK: CONFIGURED - IP 10.0.2.15/24 GW 10.0.2.2
status INPUT: READY - USB HID KEYBOARD + POINTER
```

Console commands verified over TCP serial and USB-HID keyboard input:

```text
help
status
devices
log
provider
openai
setup
ask <text>
```

`setup` opens an in-VM OpenAI/API-key menu. It can enter an API key with masked
framebuffer input, clear the key, and show provider status. The key is held only
in guest RAM and is not printed into the console or serial output. For
local-only testing, the build scripts can also embed `OPENAI_API_KEY` into a
separate non-default image with `-EmbedOpenAiApiKeyFromEnv`.

Direct OpenAI transport smoke over TCP serial:

```text
> provider
PROVIDER: OPENAI    API KEY: SET
ROUTE: OPENAI DIRECT
> ask direct provider smoke
OPENAI_DIRECT_REQ 1 api.openai.com /v1/responses
OPENAI DIRECT REQUEST 1 STARTED
openai: TLS 1.3 established
openai: HTTPS request sent
OPENAI: <provider response text>
```

## Current Architecture Decision

Do not run or port the Codex CLI inside Stage-0.

Stage-0 should grow a small native agent host:

- framebuffer UI
- serial/keyboard input
- USB/input and PCI device inventory
- network status
- explicit capability-gated agent tools

Codex/OpenAI integrations should use a small native provider boundary. The OS
boundary should stay small and auditable; a full host CLI is not part of
Stage-0.

See `docs/architecture-decisions/0001-seedos-agent-protocol.md`.

## Exact Next Task

Harden and polish the direct provider path:

- Virtio has been removed from the Stage-0 kernel runtime and VM runner path.
- RDRAND seeds entropy in the bare-metal-style VM profile.
- Intel e1000 configures RX/TX rings, negotiates DHCP through smoltcp, and shows
  IP/gateway state in the framebuffer UI and serial console.
- a PS/2/i8042 polling fallback is present for first bare-metal keyboard tests
  on machines that expose legacy keyboard compatibility. It is only reported as
  ready after an acknowledge from the keyboard or real scancode input.
- a polled xHCI path now inventories USB controllers, resets directly attached
  root-port devices, enumerates HID boot keyboards and mice, and feeds reports
  into the same input queue as PS/2.
- the USB-XHCI row now includes keyboard and mouse readiness.
- a VM-local `setup` menu now records a RAM-only OpenAI API key without echoing
  the key back into the serial log.
- `ask <text>` now stays inside the guest: it requires the VM API key state,
  resolves `api.openai.com`, opens TCP 443 through e1000, performs TLS 1.3,
  sends an HTTPS Responses API request, parses `output_text`, and prints the
  provider response.
- the development serial relay and old host-framing path have been removed from
  the runtime path.
- the next milestone is replacing MVP certificate bypass with provider pinning
  or certificate verification, then rendering responses better in the
  framebuffer UI.

## Known Gaps

- Windows now has a minimal image repackaging path:
  `scripts\package-stage0.ps1` creates `release\seedos-stage0.img` from
  `release\esp`.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
- Network failure/timeout states and packet counters are still minimal.
- Keyboard input uses a minimal US/Linux keycode mapping; no layout selection,
  modifier completeness, or text editing beyond Backspace exists yet.
- Bare-metal support is experimental. Minimal direct xHCI USB-HID boot keyboard
  and mouse handling exists, but USB hubs, non-boot HID report parsing, hotplug,
  and broad NIC coverage do not exist yet, so real hardware may still boot to the
  UI but lack input/network unless it matches the implemented paths.
- Bare-metal USB preparation scripts exist, but writing a USB disk is destructive
  and must be done with an explicit disk number and confirmation string.
- API key entry exists in the VM, but the key is RAM-only and not persisted in
  the default image. A local test image can embed the key explicitly, but must
  not be committed or shared.
- Stage-0 uses DNS/TCP/TLS/HTTPS for `api.openai.com:443`, but the MVP TLS path
  currently disables certificate verification and should be hardened before any
  serious use.
- The OpenAI JSON response parser is intentionally minimal and only extracts the
  first `output_text` string.
- QEMU TCP serial is single-client in practice; do not run two serial clients
  against the same port at the same time.
- No signed module runtime exists yet.

## Do Not Regress

- Do not rename `limine.conf` back to `limine.cfg`.
- Do not remove Limine request start/end markers.
- Do not link the kernel lower-half.
- Do not assume Linux packaging tools are available on this Windows host.
- Do not delete or overwrite `release/seedos-stage0.img` unless the replacement
  has booted in QEMU.
