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

Add the first host bridge/protocol path:

- virtio-rng entropy now works through physical DMA address translation and
  dynamic legacy virtqueue layout.
- legacy virtio-net now configures RX/TX queues, negotiates DHCP through
  smoltcp, and shows IP/gateway state in the framebuffer UI and serial console.
- modern virtio-input now uses explicit kernel MMIO mappings, queues keyboard
  events, and feeds a minimal US keymap into the same command console as serial.
- the next milestone is a tiny host-side bridge over an explicit message
  protocol, starting outside the kernel so provider credentials stay on the host.

## Known Gaps

- Windows now has a minimal image repackaging path:
  `scripts\package-stage0.ps1` creates `release\seedos-stage0.img` from
  `release\esp`.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
- Network failure/timeout states and packet counters are still minimal.
- Keyboard input uses a minimal US/Linux keycode mapping; no layout selection,
  modifier completeness, or text editing beyond Backspace exists yet.
- No provider auth, HTTPS, TLS, or API client exists inside the OS yet.
- No signed module runtime exists yet.

## Do Not Regress

- Do not rename `limine.conf` back to `limine.cfg`.
- Do not remove Limine request start/end markers.
- Do not link the kernel lower-half.
- Do not assume Linux packaging tools are available on this Windows host.
- Do not delete or overwrite `release/seedos-stage0.img` unless the replacement
  has booted in QEMU.
