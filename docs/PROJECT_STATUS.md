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

Expected xHCI inventory lines in that mode:

```text
usb-xhci: controller @ 00:06.0 detected
usb-xhci: hci 0x0100, ports 8, connected 2
USB-XHCI: READY 00:06.0 HCI 0100 PORTS 8 CONNECTED 2
```

Expected visible framebuffer UI:

```text
SEEDOS STAGE-0
AGENT HOST: LIVE STATUS
FRAMEBUFFER  READY
ENTROPY      READY
VIRTIO-RNG   READY
USB-XHCI     MISSING
VIRTIO-NET   CONFIGURED
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
status ENTROPY: READY - FILL 64/64 TOTAL 64 SRC VIRTIO-RNG
status USB-XHCI: MISSING - CONTROLLER ABSENT
virtio-rng (legacy) @ 00:03.0 detected
virtio-rng delivered 64 bytes (stored 64)
Entropy pool healthy after virtio-rng refill
virtio-net legacy transport @ 0x6080, mac 52:54:00:12:34:56, rx_q=256, tx_q=256
virtio-net initialised; DHCP polling enabled
DHCP lease acquired: ip 10.0.2.15/24 gw 10.0.2.2 dns ["10.0.2.3"]
status VIRTIO-NET: CONFIGURED - IP 10.0.2.15/24 GW 10.0.2.2
virtio-input: modern device @ 00:04.0 initialised
status INPUT: READY - VIRTIO INPUT QUEUE ACTIVE
```

Console commands verified over TCP serial and QEMU virtio keyboard injection:

```text
help
status
devices
log
bridge
setup
ask <text>
```

`setup` opens an in-VM menu. It can select `ECHO` or `OPENAI`, enter an API key
with masked framebuffer input, clear the key, and show provider status. The key
is held only in guest RAM and is not printed into the console or serial output.
For local-only testing, the build scripts can also embed `OPENAI_API_KEY` into a
separate non-default image with `-EmbedOpenAiApiKeyFromEnv`.

Host bridge smoke verified over TCP serial:

```text
> ask ping from vm harness
SEEDOS_BRIDGE_REQ 1 70696E672066726F6D20766D206861726E657373
BRIDGE REQUEST 1 SENT
BRIDGE RESPONSE 1: HOST BRIDGE OK: ping from vm harness
```

OpenAI host bridge smoke verified over TCP serial:

```text
> ask hi
SEEDOS_BRIDGE_REQ 1 6869
BRIDGE REQUEST 1 SENT
BRIDGE RESPONSE 1: SeedOS console ready.
```

## Current Architecture Decision

Do not run or port the Codex CLI inside Stage-0.

Stage-0 should grow a small native agent host:

- framebuffer UI
- serial/keyboard input
- virtio device inventory
- network status
- explicit capability-gated agent tools

Codex/OpenAI integrations should first run through a host-side bridge or normal
HTTPS/provider adapter. The OS boundary should stay small and auditable.

See `docs/architecture-decisions/0001-seedos-agent-protocol.md`.

## Exact Next Task

Evolve the first host bridge/protocol path:

- virtio-rng entropy now works through physical DMA address translation and
  dynamic legacy virtqueue layout.
- legacy virtio-net now configures RX/TX queues, negotiates DHCP through
  smoltcp, and shows IP/gateway state in the framebuffer UI and serial console.
- modern virtio-input now uses explicit kernel MMIO mappings, queues keyboard
  events, and feeds a minimal US keymap into the same command console as serial.
- a PS/2/i8042 polling fallback is present for first bare-metal keyboard tests
  on machines that expose legacy keyboard compatibility.
- a bare-metal xHCI detector now inventories USB controllers and connected
  ports in the framebuffer UI and `devices`, but it is not a HID keyboard
  driver yet.
- a tiny serial host bridge now accepts `ask <text>`, emits
  `SEEDOS_BRIDGE_REQ`, receives an STX-framed `SEEDOS_BRIDGE_RESP`, and renders
  the answer in the VM console.
- a VM-local `setup` menu now records provider selection and a RAM-only API key
  without echoing the key back into the serial log.
- the Windows host bridge can run as an echo responder or as an OpenAI Responses
  API adapter with `-Provider openai`; the OpenAI adapter reads the host
  `OPENAI_API_KEY`, not the VM-stored key.
- the next milestone is turning the echo bridge into a capability-shaped agent
  protocol with a real host/provider adapter that can use the selected provider.

## Known Gaps

- Windows now has a minimal image repackaging path:
  `scripts\package-stage0.ps1` creates `release\seedos-stage0.img` from
  `release\esp`.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
- Network failure/timeout states and packet counters are still minimal.
- Keyboard input uses a minimal US/Linux keycode mapping; no layout selection,
  modifier completeness, or text editing beyond Backspace exists yet.
- Bare-metal support is experimental. xHCI controller detection exists, but
  USB-HID keyboard handling and real NIC drivers do not exist yet, so real
  hardware may boot to the UI but lack input/network.
- Bare-metal USB preparation scripts exist, but writing a USB disk is destructive
  and must be done with an explicit disk number and confirmation string.
- The host bridge is a development echo responder only; it is not a provider
  adapter and does not carry auth, tools, or policy yet.
- Provider selection and API key entry exist in the VM, but the key is RAM-only,
  not persisted in the default image, and not yet wired to a real provider
  request path. A local test image can embed the key explicitly, but must not be
  committed or shared.
- The OpenAI provider adapter currently runs on the Windows host bridge. Stage-0
  still has no direct HTTPS/TLS provider client inside the OS.
- QEMU TCP serial is single-client in practice; do not run the serial smoke
  client and host bridge against the same port at the same time.
- No HTTPS, TLS, or provider API client exists inside the OS yet.
- No signed module runtime exists yet.

## Do Not Regress

- Do not rename `limine.conf` back to `limine.cfg`.
- Do not remove Limine request start/end markers.
- Do not link the kernel lower-half.
- Do not assume Linux packaging tools are available on this Windows host.
- Do not delete or overwrite `release/seedos-stage0.img` unless the replacement
  has booted in QEMU.
