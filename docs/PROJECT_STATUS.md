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

Expected visible framebuffer overlay:

```text
SEEDOS STAGE-0
AGENT HOST: STUB
VM MVP: BOOT + FRAME + DEVICE POLL
```

Expected useful serial lines:

```text
Seed kernel: early init start
Limine loaded base revision: 3
Framebuffer response revision: 1
Framebuffer negotiated via Limine
Framebuffer hello overlay drawn
virtio-rng (legacy) @ 00:03.0 detected
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

Replace the static framebuffer overlay with a tiny live status UI:

- left/top title: `SEEDOS STAGE-0`
- status rows for framebuffer, entropy, virtio-rng, virtio-net, input
- serial log mirror of each state transition
- no dependency on DHCP/TLS yet

The first useful milestone after that is a command/input path:

- serial command input first, or keyboard if easier
- minimal commands: `help`, `status`, `devices`, `log`
- responses drawn to framebuffer and serial

## Known Gaps

- Windows has a kernel build script and QEMU runner, but no polished Windows
  image repackaging script yet. The current image is already updated and tested.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
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
