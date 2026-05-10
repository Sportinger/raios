# Build, Run, And Debug

This project currently has two practical environments:

- Windows PowerShell: primary verified local path.
- Linux/WSL: useful later for FAT image tooling and Limine source builds.

## Build Kernel On Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release
```

Output:

```text
target\x86_64-seed\release\seed-kernel
```

The script injects the required kernel linker flags through `RUSTFLAGS`.

## Run VM On Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

The runner uses:

- QEMU: `C:\Program Files\qemu\qemu-system-x86_64.exe`
- firmware code: `C:\Program Files\qemu\share\edk2-x86_64-code.fd`
- firmware vars copy from `release\ovmf_vars.fd`
- image: `release\seedos-stage0.img`
- display: GTK
- serial log: `%TEMP%\seedos-stage0.serial.txt`

Tail the serial log:

```powershell
Get-Content $env:TEMP\seedos-stage0.serial.txt -Wait
```

Stop QEMU:

```powershell
Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
```

## Test Workspace

```powershell
cargo fmt --all -- --check
cargo test --locked -p ota-tools -p registry-core -p registry-tools -p fake-cloud-server
```

These tests should use the normal host target. Do not add a root `.cargo/config`
that forces the entire workspace to the kernel target.

## Boot Chain

Expected chain:

```text
QEMU UEFI firmware -> EFI shell/startup -> EFI/BOOT/BOOTX64.EFI -> limine.conf -> /kernel/kernel.elf -> _start
```

Important files:

- `seed-kernel/limine/limine.conf`
- `release/esp/limine.conf`
- `release/esp/EFI/BOOT/limine.conf`
- `seed-kernel/linker.ld`
- `seed-kernel/src/main.rs`
- `seed-kernel/src/framebuffer.rs`
- `seed-kernel/src/text.rs`

## Known Failure Modes

### Limine says config file not found

Likely cause: using `limine.cfg` with Limine 10.

Fix: use `limine.conf` at ESP root and beside `EFI/BOOT/BOOTX64.EFI`.

### Limine says lower half PHDRs are not allowed

Likely cause: kernel linked around `1M` or linker script not applied.

Fix: link at `0xffffffff80000000` and ensure `linker.ld` is passed to
`rust-lld`.

### Limine only reports one request

Likely cause: Limine request section markers missing or ordered incorrectly.

Fix: keep these sections in `seed-kernel/linker.ld`:

```ld
KEEP(*(.limine_requests_start))
KEEP(*(.limine_requests))
KEEP(*(.limine_requests_end))
```

and keep corresponding Rust statics in `seed-kernel/src/main.rs`.

### Kernel starts then hangs around allocator or early Rust code

Likely cause: SSE/FXSR state not enabled before compiler-generated or library
code uses SIMD instructions.

Fix: `_start` currently enables SSE before entering `early_main`; do not remove
that setup without replacing the generated code assumptions.

### Black QEMU screen but serial log continues

Check the serial log for framebuffer lines:

```text
Framebuffer request: checking response
Framebuffer response revision: 1
Framebuffer negotiated via Limine
Framebuffer hello overlay drawn
```

If those lines are missing, debug Limine requests. If they are present, debug
pixel format, text rendering, or whether the displayed image is stale.

### Workspace tests try to build the kernel target

Likely cause: root `.cargo/config.toml` forcing `target =
"seed-kernel/x86_64-seed.json"`.

Fix: keep kernel target config local to `seed-kernel/.cargo/config.toml` or
inside build scripts, not at the workspace root.

## Image Packaging Notes

The tested image is already present at:

```text
release/seedos-stage0.img
```

Linux/WSL packaging path:

```bash
bash scripts/package-stage0.sh
```

That path expects `mkfs.fat`, `mmd`, and `mcopy`.

Windows currently needs a proper packaging script. Until that exists, treat the
checked-in image as the known-good artifact and only replace it after a visual
QEMU boot and serial log verification.
