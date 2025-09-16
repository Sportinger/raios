# Section Report 2.1 – Boot & Frame (Scaffold)

## Scope
Initial kernel bootstrap for runbook section 2.1 covering Limine-facing artifacts, Rust `no_std` crate scaffolding, early-boot logging, and framebuffer bring-up.

## Artifacts
- `seed-kernel/` Rust crate targeting custom `x86_64-seed` spec with linker script and Limine config.
- Serial logger (`seed-kernel/src/serial.rs`) used for early boot diagnostics and panic mirroring.
- Framebuffer negotiation + double-buffer utilities (`seed-kernel/src/framebuffer.rs`) with color fills/present.
- Build helper: `./scripts/build-seed-kernel.sh` invoking the pinned nightly toolchain with `-Zbuild-std`.

## Exit criteria snapshot
- Kernel crate builds successfully via `cargo +nightly-2024-10-15 -Zbuild-std=core,compiler_builtins,alloc check --target seed-kernel/x86_64-seed.json -p seed-kernel`.
- Serial logging available for panic paths (writes to COM1).
- Framebuffer double buffer fills and presents a placeholder overlay rectangle.

## Follow-up
- Layer font rendering / hello text atlas on top of the present pipeline.
- Add CI harness step to invoke `scripts/build-seed-kernel.sh` and archive `seed-kernel.elf` artifact.
