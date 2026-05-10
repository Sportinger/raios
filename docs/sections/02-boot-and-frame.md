# Section Report 2.1 – Boot & Frame (Scaffold)

## Scope
Initial kernel bootstrap for runbook section 2.1 covering Limine-facing artifacts, Rust `no_std` crate scaffolding, early-boot logging, and framebuffer bring-up.

Status update: the boot path now reaches a visible framebuffer status UI in QEMU.
The current operational snapshot lives in `docs/PROJECT_STATUS.md`.

## Artifacts
- `seed-kernel/` Rust crate targeting custom `x86_64-seed` spec with linker script and Limine config.
- Serial logger (`seed-kernel/src/serial.rs`) used for early boot diagnostics and panic mirroring.
- Framebuffer negotiation + direct framebuffer drawing utilities (`seed-kernel/src/framebuffer.rs`) with color fills/text rendering.
- Build helper: `./scripts/build-seed-kernel.sh` invoking the pinned nightly toolchain with `-Zbuild-std`.

## Exit criteria snapshot
- Kernel crate builds successfully via `scripts/build-seed-kernel.ps1` on Windows or `scripts/build-seed-kernel.sh` in Linux/WSL.
- Serial logging available for panic paths (writes to COM1).
- Framebuffer draws the Stage-0 live status UI in QEMU.

## Follow-up
- Continue TLS/HTTPS provider work now that RDRAND entropy, USB-HID input, and e1000 networking are verified.
- Add CI harness step to invoke `scripts/build-seed-kernel.sh` and archive `seed-kernel.elf` artifact.
