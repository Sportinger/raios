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
ENTROPY      WAITING
VIRTIO-RNG   DEGRADED
VIRTIO-NET   WAITING
INPUT        WAITING
```

Expected useful serial lines:

```text
Seed kernel: early init start
Limine loaded base revision: 3
Framebuffer response revision: 1
Framebuffer negotiated via Limine
status FRAMEBUFFER: READY - 1280x800 PITCH 5120
status ENTROPY: WAITING - FILL 0/64 TOTAL 0 SRC NONE
virtio-rng (legacy) @ 00:03.0 detected
virtio-rng request timed out; entropy source disabled
status VIRTIO-RNG: DEGRADED - ATTACHED, WAITING FOR DATA
```

Serial commands verified over TCP serial:

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

Fix or bypass the current virtio-rng entropy timeout:

- virtio-rng is detected and configured, but no entropy is returned before the
  timeout.
- net/input bring-up remains deferred until entropy becomes ready or those paths
  are made safe to probe without entropy.
- after entropy is unblocked, verify virtio-net/DHCP status rows and serial
  `devices` output.

## Known Gaps

- Windows now has a minimal image repackaging path:
  `scripts\package-stage0.ps1` creates `release\seedos-stage0.img` from
  `release\esp`.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
- virtio-rng is detected, but the entropy request currently times out. The UI
  shows this as `VIRTIO-RNG DEGRADED`; net/input bring-up remains deferred until
  entropy can become ready or those paths stop depending on it.
- virtio-net probing exists, but the UI does not yet show link/DHCP state.
- No provider auth, HTTPS, TLS, or API client exists inside the OS yet.
- No signed module runtime exists yet.

## Do Not Regress

- Do not rename `limine.conf` back to `limine.cfg`.
- Do not remove Limine request start/end markers.
- Do not link the kernel lower-half.
- Do not assume Linux packaging tools are available on this Windows host.
- Do not delete or overwrite `release/seedos-stage0.img` unless the replacement
  has booted in QEMU.
