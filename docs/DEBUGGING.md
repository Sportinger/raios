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
`release\esp\kernel\kernel.elf` and writes `release\raios-stage0.img`.

For local-only provider testing, a default OpenAI key can be embedded from the
current process environment without touching the tracked ESP staging directory.
Without a configured pin, the normal build still fails closed at the TLS trust
gate:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release -Image release\raios-stage0-local-openai.img -UseTempEsp -EmbedOpenAiApiKeyFromEnv
```

This requires `OPENAI_API_KEY` to be set. The resulting image contains the key,
so do not commit or share that local image. The packaging script refuses to
embed a provider key into `release\esp` or the default `release\raios-stage0.img`;
see `docs\SECRETS.md`.

To exercise the preferred normal positive trust path, also embed the current
OpenAI SPKI SHA-256 pin from the process environment:

```powershell
$env:OPENAI_API_KEY = "<local key or fake smoke key>"
$env:OPENAI_SPKI_SHA256 = "<64 hex chars>"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release -Image release\raios-stage0-local-openai.img -UseTempEsp -EmbedOpenAiApiKeyFromEnv -EmbedOpenAiSpkiPinFromEnv
```

For legacy leaf-certificate pinning, embed the current OpenAI leaf certificate
SHA-256 pin instead:

```powershell
$env:OPENAI_API_KEY = "<local key or fake smoke key>"
$env:OPENAI_CERT_SHA256 = "<64 hex chars>"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release -Image release\raios-stage0-local-openai.img -UseTempEsp -EmbedOpenAiApiKeyFromEnv -EmbedOpenAiCertPinFromEnv
```

Leaf-certificate pins are intentionally rotation-sensitive. Prefer SPKI pinning
for normal pinned-trust testing.

To exercise the old unverified provider-response smoke path, build a local image
with the explicit development override:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release -Image release\raios-stage0-local-openai.img -UseTempEsp -EmbedOpenAiApiKeyFromEnv -AllowUnverifiedOpenAiTls
```

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

Run headless with a QEMU xHCI controller plus USB keyboard/mouse attached:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555 -Headless -UsbXhciInput
```

Run the bare-metal-style VM profile with USB keyboard/pointer and e1000
networking:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-baremetal-vm.ps1 -StopExisting
```

The runner uses:

- QEMU: `C:\Program Files\qemu\qemu-system-x86_64.exe`
- firmware code: `C:\Program Files\qemu\share\edk2-x86_64-code.fd`
- firmware vars copy from `release\ovmf_vars.fd`
- image: `release\raios-stage0.img`
- display: GTK with the host cursor hidden over the guest area by default, but
  without automatic mouse grab, so raiOS shows one pointer and the QEMU window
  can still be moved or closed. Add `-MouseGrab` for grab-on-hover while raiOS
  draws its own pointer. Press `Ctrl+Alt+G` to release a grabbed QEMU mouse.
- serial log: `%TEMP%\raios-stage0.serial.txt`
- `-UsbXhciInput` adds `qemu-xhci`, `usb-kbd`, and `usb-tablet` by default.
  The tablet is still USB HID, but it reports absolute pointer coordinates, so
  the raiOS cursor stays aligned with the QEMU window after focus changes. Add
  `-RelativeMouse` to use QEMU's relative `usb-mouse` boot device instead.
- default networking is an emulated Intel e1000 device attached to QEMU
  user-mode networking.
- `-MonitorTcpPort <port>` exposes the QEMU HMP monitor for commands such as
  `sendkey h`.

With `-SerialMode tcp`, the serial device is exposed at
`127.0.0.1:<SerialTcpPort>` and still writes a QEMU chardev log to the serial
log path.

With `-Headless`, the runner uses `-display none` instead of GTK. This is useful
for serial-only harness tests.

Tail the serial log:

```powershell
Get-Content $env:TEMP\raios-stage0.serial.txt -Wait
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
s.sendall(b"help\rstatus\rdevices\rlog\rprovider\ropenai\r")
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

Write a raiOS boot USB from an elevated Administrator PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\write-stage0-usb.ps1 -DiskNumber <N> -ConfirmErase "ERASE DISK <N>"
```

The write command erases the selected USB disk.

## Direct OpenAI Smoke

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1
```

This uses `release\raios-stage0-local-openai.img`, so first package that local
image with `-UseTempEsp -EmbedOpenAiApiKeyFromEnv`. The image contains the key
and must not be committed or shared. By default this smoke checks that the
provider path is denied by the TLS trust gate.

Expected trust-gate lines:

```text
> provider
PROVIDER: OPENAI    API KEY: SET
ROUTE: OPENAI DIRECT
TLS TRUST: pin_config_missing
> ask direct provider smoke
OPENAI TLS TRUST DENIED: pin_config_missing
```

To require a real provider response from a development image built with
`-AllowUnverifiedOpenAiTls`, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1 -ExpectProviderResponse
```

That confirms the guest is using e1000 networking, TLS, HTTPS, and the OpenAI
Responses API directly, but only through an explicit unverified development
override. Serious use must rely on the pinned or verified trust path, not this
development mode.

To require the normal SPKI pinned-trust path, package a local image with both
`OPENAI_API_KEY` and `OPENAI_SPKI_SHA256`, then run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1 -ExpectSpkiPinnedTrust
```

The harness expects:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_spki sha256:<pin-id>
openai: HTTPS request sent
```

To require the legacy leaf-certificate pinned-trust path, package a local image
with both `OPENAI_API_KEY` and `OPENAI_CERT_SHA256`, then run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1 -ExpectPinnedTrust
```

The harness expects:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_cert sha256:<pin-id>
openai: HTTPS request sent
```

For a transport-only smoke, the API key can be a fake non-secret value; the
expected result is then an `OPENAI HTTP` provider error after HTTPS write, not a
model response.

To prove a wrong pin fails before HTTPS write, package with an intentionally
wrong `OPENAI_SPKI_SHA256` or `OPENAI_CERT_SHA256` and run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1 -ExpectPinMismatch
```

## VM Setup Menu

Type `setup` in the VM console to open the provider menu:

```text
1 PROVIDER: OPENAI DIRECT    2 API KEY: MISSING
3 CLEAR API KEY    4 WIFI SSID: NONE
5 WIFI KEY: MISSING    6 CLEAR WIFI    Q EXIT
```

Press `1` to show provider status, press `2` to enter an API key, and press
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
status ENTROPY: READY - FILL 64/64 TOTAL 64 SRC RDRAND
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
e1000: device 00:02.0 id=0x100e mmio=0x81040000 size=131072 mac 52:54:00:12:34:56
e1000 network initialised; DHCP polling enabled
DHCP lease acquired: ip 10.0.2.15/24 gw 10.0.2.2 dns ["10.0.2.3"]
status NETWORK: CONFIGURED - IP 10.0.2.15/24 GW 10.0.2.2
status INPUT: READY - USB HID KEYBOARD + POINTER
```

For USB-HID keyboard/mouse smoke, useful lines include:

```text
usb-xhci: hci 0x0100, ports 8, connected 2
usb-hid: device class 00 subclass 00 protocol 00
usb-hid: boot keyboard interface 0
usb-hid: boot keyboard ready on slot 1 endpoint 0x81
usb-hid: boot mouse ready on slot 2 endpoint 0x81
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
status INPUT: READY - USB HID KEYBOARD + POINTER
usb input batch: 1 events
> help
COMMANDS: help status devices log provider openai setup ask <text>
```

On bare metal, `KBD NONE` or `MOUSE NONE` means the xHCI controller was usable
but the current direct root-port scan did not find that USB HID boot device. In
that case the connected device may be the boot stick, a hub/dock, or a keyboard
or mouse that does not expose boot protocol HID on the root port. If no USB
input is active, Stage-0 periodically logs `usb-hotplug: rescanning xHCI input
devices` and re-probes xHCI, so removing a boot stick and then plugging a USB
keyboard directly can be tested without rebooting. This is still a limited
no-input recovery path, not full USB detach/reconfigure support.

For HID input debugging, the USB status line includes `EV` for successful input
reports, `ERR` for interrupt transfer errors, and `TCC` for the last transfer
completion code. If a keyboard is `READY` but typing does not change `EV`, the
device enumerated but reports are not reaching the input queue yet.

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
release/raios-stage0.img
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
