# Project Status

Last verified locally: 2026-05-17 on Windows with QEMU 11.

## Verified Boot State

- Repository path: `C:\Users\admin\Documents\raios2`
- Boot image: `release/raisos-stage0.img`
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
AI  CONSOLE                                      SET
RAISOS
DIRECT AI HOST
NET CONFIGURED   INPUT READY   USB READY   RNG READY
CHAT
TYPE MESSAGE AND PRESS ENTER
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

The framebuffer UI defaults to an AI chat mode. The `CONSOLE` tab keeps the
debug console visible, and the `SET` tab opens provider settings. `setup` also
opens the in-VM OpenAI/API-key menu. API-key entry is masked, held only in guest
RAM, and not printed into the console or serial output. For local-only testing,
the build scripts can also embed `OPENAI_API_KEY` into a separate non-default
image with `-EmbedOpenAiApiKeyFromEnv`.

Direct OpenAI trust-gate smoke over TCP serial:

```text
> provider
PROVIDER: OPENAI    API KEY: SET
ROUTE: OPENAI DIRECT
TLS TRUST: pin_config_missing
> ask direct provider smoke
OPENAI TLS TRUST DENIED: pin_config_missing
```

Direct OpenAI pinned-trust smoke is also verified with a temporary image built
from a process-local fake API key and a current `OPENAI_CERT_SHA256` leaf
certificate pin. Expected positive trust lines:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_cert sha256:<pin-id>
openai: HTTPS request sent
OPENAI HTTP
```

## Current Architecture Decision

Do not run or port the Codex CLI inside Stage-0.

Stage-0 should grow a small native agent host:

- framebuffer UI
- serial/keyboard/mouse input
- USB/input and PCI device inventory
- network status
- explicit capability-gated agent tools

Codex/OpenAI integrations should use a small native provider boundary. The OS
boundary should stay small and auditable; a full host CLI is not part of
Stage-0.

See `docs/architecture-decisions/0001-raisos-agent-protocol.md`.

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
  root-port devices, enumerates HID boot keyboards, relative boot mice, and QEMU
  HID tablets, and feeds reports into the same input queue as PS/2.
- if no USB keyboard or pointer is active, the event loop periodically re-probes
  xHCI so a keyboard plugged in after boot can be picked up without rebooting.
- the USB status line includes `EV`, `ERR`, and `TCC` counters for HID input
  reports and interrupt transfer diagnostics on bare metal.
- the USB-XHCI row now includes keyboard and mouse readiness.
- the framebuffer renderer is double-buffered to avoid visible full-screen
  redraw flicker, and pointer movement now updates only a small cursor overlay
  instead of forcing a full UI redraw.
- the visible QEMU GTK profile uses `usb-tablet` absolute pointer input by
  default and hides the host cursor over the guest area without automatic mouse
  grab, so only the raisOS pointer is visible and remains aligned after focus
  changes; `-RelativeMouse` or `-MouseGrab` switches back to relative
  `usb-mouse` for stricter boot-mouse testing.
- the visible UI now defaults to a chat-first surface with `AI`, `CONSOLE`, and
  `SET` modes. Serial commands continue to use the command interpreter so VM
  harnesses remain deterministic.
- USB/PS2 keyboard input now carries special keys into the UI: Tab and arrow
  keys move a visible focus ring through the top navigation, chat/console input,
  and settings actions; Enter activates the focused item and Esc backs out of
  settings/API-key entry.
- the Surface Pro 4 internal WLAN target has been selected as Marvell AVASTAR
  88W8897 (`11ab:2b38`, Linux reference driver family `mwifiex_pcie`). Stage-0
  now probes PCI for that device and exposes it as a Wi-Fi status chip/log line,
  and the settings menu can record a RAM-only SSID and WPA passphrase. Firmware
  upload, WPA, and packet transport are not implemented yet.
- a VM-local `setup` menu now records a RAM-only OpenAI API key without echoing
  the key back into the serial log.
- `ask <text>` now stays inside the guest. In the normal build it requires the
  VM API key state and then fails closed at provider trust before API-key copy or
  HTTPS write unless a syntactically valid provider pin is configured. With
  `-EmbedOpenAiCertPinFromEnv`, the first positive verifier slice checks the
  OpenAI leaf certificate SHA-256 pin and the TLS 1.3 P-256 ECDSA
  `CertificateVerify` proof before copying the API key or writing HTTPS. With
  the explicit development override
  `-AllowUnverifiedOpenAiTls`, it resolves `api.openai.com`, opens TCP 443
  through e1000, performs TLS 1.3 with `NoVerify`, sends an HTTPS Responses API
  request, parses `output_text`, and prints the provider response.
- the provider trust state is visible in console/provider status,
  `system.snapshot.v0`, `problem.list`, and `service.inventory`; the default
  trust problem is `provider.tls_pin_config_missing`, while a successful pinned
  handshake reports `pinned_cert_verified`.
- the development serial relay and old host-framing path have been removed from
  the runtime path.
- the next milestone is moving from leaf-certificate pinning to a more durable
  trust mechanism: SPKI pinning first, then possibly WebPKI once trust anchors,
  time, hostname checks, and chain handling are specified.

## Known Gaps

- Windows now has a minimal image repackaging path:
  `scripts\package-stage0.ps1` creates `release\raisos-stage0.img` from
  `release\esp`.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
- Network failure/timeout states and packet counters are still minimal.
- Keyboard input uses a minimal US/Linux keycode mapping; no layout selection,
  modifier completeness, or text editing beyond Backspace exists yet.
- Bare-metal support is experimental. Minimal direct xHCI USB-HID boot keyboard,
  mouse, hub traversal, and a limited no-input USB hotplug rescan exist, but full
  detach/reconfigure handling and broad NIC coverage do not exist yet, so real
  hardware may still boot to the UI but lack input/network unless it matches the
  implemented paths.
- Wi-Fi support currently detects the Surface Pro 4 Marvell AVASTAR 88W8897
  target and stores RAM-only SSID/WPA configuration for the current boot. The
  next implementation step is a Marvell PCIe firmware-upload path before 802.11
  association or WPA2 can work.
- Bare-metal USB preparation scripts exist, but writing a USB disk is destructive
  and must be done with an explicit disk number and confirmation string.
- API key entry exists in the VM, but the key is RAM-only and not persisted in
  the default image. A local test image can embed the key explicitly, but must
  not be committed or shared.
- Stage-0 has verified DNS/TCP/TLS/HTTPS for `api.openai.com:443` behind both
  the explicit unverified development override and the first positive
  leaf-certificate pin verifier. Leaf-certificate pins rotate with provider
  certificates, so SPKI pinning or WebPKI remains a required hardening step.
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
- Do not delete or overwrite `release/raisos-stage0.img` unless the replacement
  has booted in QEMU.
