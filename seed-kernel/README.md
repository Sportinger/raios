# Seed Kernel

Rust Stage-0 kernel skeleton built for Limine + UEFI boot flow. This directory currently provides:

- A `no_std` Rust crate targeting `x86_64-unknown-none` with custom linker script and entry point.
- Serial logger targeting COM1 for early diagnostics plus panic reporting.
- Limine framebuffer negotiation with double-buffered blitter primitives and a placeholder overlay fill.
- Pinned toolchain (`nightly-2024-10-15`) plus `build-std` configuration for reproducible builds.
- Limine configuration stub targeting `boot:///kernel/seed-kernel.elf`.

## Building (scaffold)

```
# ensure necessary toolchain components are installed
cd ..  # repo root
rustup toolchain install nightly-2024-10-15 --component rust-src --component llvm-tools-preview
./scripts/build-seed-kernel.sh
```

This currently emits `target/x86_64-seed/debug/seed-kernel.elf`. Image packaging + Limine ESP generation will be wired up in subsequent iterations.

## Next steps

- Wire serial logger and framebuffer bootstrap per runbook section 2.1.
- Automate ESP/ISO assembly with Limine binaries and kernel artifact.
