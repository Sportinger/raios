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

Expected visible framebuffer UI:

```text
SEEDOS STAGE-0
AGENT HOST: LIVE STATUS
FRAMEBUFFER  READY
ENTROPY      READY
VIRTIO-RNG   READY
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

Host bridge smoke verified over TCP serial:

```text
> ask ping from vm harness
SEEDOS_BRIDGE_REQ 1 70696E672066726F6D20766D206861726E657373
BRIDGE REQUEST 1 SENT
BRIDGE RESPONSE 1: HOST BRIDGE OK: ping from vm harness
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
- a tiny serial host bridge now accepts `ask <text>`, emits
  `SEEDOS_BRIDGE_REQ`, receives an STX-framed `SEEDOS_BRIDGE_RESP`, and renders
  the answer in the VM console.
- a VM-local `setup` menu now records provider selection and a RAM-only API key
  without echoing the key back into the serial log.
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
- The host bridge is a development echo responder only; it is not a provider
  adapter and does not carry auth, tools, or policy yet.
- Provider selection and API key entry exist in the VM, but the key is RAM-only,
  not persisted in the image, and not yet wired to a real provider request path.
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
