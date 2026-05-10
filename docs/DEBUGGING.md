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

## Package Image On Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release
```

This stages `target\x86_64-seed\release\seed-kernel` into
`release\esp\kernel\kernel.elf` and writes `release\seedos-stage0.img`.

For local-only provider testing, a default OpenAI key can be embedded from the
current process environment without touching the tracked ESP staging directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release -Image release\seedos-stage0-local-openai.img -UseTempEsp -EmbedOpenAiApiKeyFromEnv
```

This requires `OPENAI_API_KEY` to be set. The resulting image contains the key,
so do not commit or share that local image.

## Run VM On Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

Run with interactive serial commands on TCP port 4555:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555
```

Run headless with the same serial TCP port:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555 -Headless
```

The runner uses:

- QEMU: `C:\Program Files\qemu\qemu-system-x86_64.exe`
- firmware code: `C:\Program Files\qemu\share\edk2-x86_64-code.fd`
- firmware vars copy from `release\ovmf_vars.fd`
- image: `release\seedos-stage0.img`
- display: GTK
- serial log: `%TEMP%\seedos-stage0.serial.txt`

With `-SerialMode tcp`, the serial device is exposed at
`127.0.0.1:<SerialTcpPort>` and still writes a QEMU chardev log to the serial
log path.

With `-Headless`, the runner uses `-display none` instead of GTK. This is useful
for serial-only harness tests.

Tail the serial log:

```powershell
Get-Content $env:TEMP\seedos-stage0.serial.txt -Wait
```

Stop QEMU:

```powershell
Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
```

Smoke-test serial commands with Python while QEMU is running in TCP mode:

```powershell
@'
import socket, time
s = socket.create_connection(("127.0.0.1", 4555), timeout=5)
s.settimeout(0.2)
time.sleep(1)
s.sendall(b"help\rstatus\rdevices\rlog\r")
end = time.time() + 3
out = bytearray()
while time.time() < end:
    try:
        out.extend(s.recv(4096))
    except TimeoutError:
        time.sleep(0.1)
print(out.decode("ascii", "replace"))
s.close()
'@ | python -
```

## Bare-Metal USB

Bare-metal support is experimental. Start with `docs/BARE_METAL.md`.

List removable USB disks:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\list-usb-disks.ps1
```

Write a SeedOS boot USB from an elevated Administrator PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\write-stage0-usb.ps1 -DiskNumber <N> -ConfirmErase "ERASE DISK <N>"
```

The write command erases the selected USB disk.

## Host Bridge

Run the development bridge while QEMU is running in TCP serial mode:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\host-bridge.ps1 -Port 4555
```

Run the same bridge against OpenAI instead of the echo responder:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\host-bridge.ps1 -Port 4555 -Provider openai
```

This requires `OPENAI_API_KEY` in the host PowerShell environment. The bridge
uses the Responses API with model `gpt-5.5` by default and returns a short
single-line answer that fits the current Stage-0 console.

One-shot request/response smoke:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\host-bridge.ps1 -Port 4555 -Once -Ask "hello bridge"
```

Full headless QEMU smoke:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\host-bridge-smoke.ps1
```

OpenAI provider smoke:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-bridge-smoke.ps1
```

Expected serial bridge lines:

```text
> ask hello bridge
SEEDOS_BRIDGE_REQ 1 68656C6C6F20627269646765
BRIDGE REQUEST 1 SENT
BRIDGE RESPONSE 1: HOST BRIDGE OK: hello bridge
```

The VM-to-host request is printable for logs. The host-to-VM response is prefixed
with STX (`0x02`) so the kernel routes it through the bridge parser instead of
treating it as user console input.

## VM Setup Menu

Type `setup` in the VM console to open the provider menu:

```text
1 PROVIDER: ECHO    2 API KEY: MISSING
3 CLEAR API KEY    4 STATUS    Q EXIT
```

Press `1` to choose `ECHO` or `OPENAI`, press `2` to enter an API key, and press
Enter to save it. The framebuffer prompt masks API-key input with `*`, and the
kernel does not echo the key to the serial output. The key is RAM-only; rebooting
the VM or choosing clear removes it.

If the kernel was built with `-EmbedOpenAiApiKeyFromEnv`, `setup` starts with
`OPENAI` selected and `API KEY: SET`. The key is embedded in that local kernel
binary/image, not printed to serial output.

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
status FRAMEBUFFER: READY - 1280x800 PITCH 5120
```

If those lines are missing, debug Limine requests. If they are present, debug
pixel format, text rendering, or whether the displayed image is stale.

For the live status UI, useful lines now include:

```text
HHDM offset=0xffff800000000000
status FRAMEBUFFER: READY - 1280x800 PITCH 5120
status ENTROPY: READY - FILL 64/64 TOTAL 64 SRC VIRTIO-RNG
status VIRTIO-RNG: READY - ATTACHED AS ENTROPY SOURCE
virtio-net legacy transport @ 0x6080, mac 52:54:00:12:34:56, rx_q=256, tx_q=256
virtio-net initialised; DHCP polling enabled
DHCP lease acquired: ip 10.0.2.15/24 gw 10.0.2.2 dns ["10.0.2.3"]
status VIRTIO-NET: CONFIGURED - IP 10.0.2.15/24 GW 10.0.2.2
virtio-input: modern device @ 00:04.0 initialised
status INPUT: READY - VIRTIO INPUT QUEUE ACTIVE
```

Modern virtio-input depends on the Limine HHDM response and the kernel MMIO
window in `seed-kernel/src/memory.rs`. If input falls back to missing, check that
Limine reports request count 4, that the HHDM offset line appears, and that PCI
BAR sizing did not reject the virtio common, notify, ISR, or device capability.

### Kernel hits #UD during first DHCP transmit

Likely cause: the custom target enabled CPU features that QEMU's default CPU did
not expose. One verified failure was smoltcp emitting `pshufb` in
`smoltcp::wire::ip::checksum::data` because the target allowed SSSE3.

Fix: keep `seed-kernel/x86_64-seed.json` limited to `+sse,+sse2,+fxsr` unless
the kernel grows CPUID feature gates or the QEMU runner is pinned to a matching
CPU model.

### Workspace tests try to build the kernel target

Likely cause: root `.cargo/config.toml` forcing `target =
"seed-kernel/x86_64-seed.json"`.

Fix: keep kernel target config local to `seed-kernel/.cargo/config.toml` or
inside build scripts, not at the workspace root.

## Image Packaging Notes

The tested image is present at:

```text
release/seedos-stage0.img
```

Windows packaging path:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release
```

Linux/WSL packaging path:

```bash
bash scripts/package-stage0.sh
```

That path expects `mkfs.fat`, `mmd`, and `mcopy`.
