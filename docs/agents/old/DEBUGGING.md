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

For an explicit pin-only rotation window, embed a second standby SPKI pin. The
verifier will accept either the active SPKI pin or the standby SPKI pin, but the
trust metadata still reports `pin_only_no_webpki_chain_validation` and
`not_validated_stage0`:

```powershell
$env:OPENAI_API_KEY = "<local key or fake smoke key>"
$env:OPENAI_SPKI_SHA256 = "<active 64 hex chars>"
$env:OPENAI_SPKI_SHA256_NEXT = "<standby 64 hex chars>"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release -Image release\raios-stage0-local-openai.img -UseTempEsp -EmbedOpenAiApiKeyFromEnv -EmbedOpenAiSpkiPinFromEnv -EmbedOpenAiSpkiRotationPinFromEnv
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

The normal visible release configuration requests Genesis at 1920x1080x32. A
successful boot reports exact `status FRAMEBUFFER: READY - 1920x1080 PITCH 7680`.
Use `-MouseGrab` when host/guest pointer alignment matters during interaction;
press `Ctrl+Alt+G` to release the grab.

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

Run headless with a prepared GPT persistence image attached as xHCI USB Mass
Storage:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -Headless -UsbXhciInput -UsbStorageImage $env:TEMP\raios-usb-msc-test.img
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
- display: GTK with the host cursor hidden over the guest area by default, so
  raiOS shows its scaled legacy pointer. Add `-MouseGrab` for grab-on-hover;
  this keeps the same absolute tablet device. Press `Ctrl+Alt+G` to release a
  grabbed QEMU mouse.
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

Shadow VM smoke timing note: the full
`vm-harness\shadow-vm-smoke.ps1` run can be much longer than the compile step on
this Windows/QEMU setup. When running it through an agent tool, use a command
timeout of at least 30 minutes and pass `-TimeoutSeconds 300` if the default
45-second per-command serial timeout is too tight. Do not treat a 10-minute
outer tool timeout as a guest or protocol failure by itself; inspect the
generated `release\vm-reports\shadow-*.json` and the temp `serial.log`.
The entry script dispatches into focused `shadow-vm-smoke-profile-*.ps1`
profile slices, so profile-specific failures should be debugged in the matching
slice rather than in one monolithic harness file. The harness opens a fresh
serial TCP connection for each agent command, drains buffered bytes after the
expected marker, and then closes the connection; this avoids treating a stale
host-side TCP stream as a guest protocol failure during long full-profile runs.

If a full smoke fails with a host-side TCP write exception or a truncated long
serial command after all predicates up to the previous command passed, FIRST
classify per the AGENTS.md Failure Classification Rule (check whether the QEMU
process is still alive and whether the serial log tail shows a cleanly
completed response), record the verdict in `docs/status/STATUS.md`, and only
then rerun with smaller chunks plus write delay. RESOLVED 2026-07-04: the old
giant mid-profile `agent audit.events 256` scrape was replaced by bounded
per-boundary scrapes (`audit.events 24/64/96` close to the records they
prove); the full profile passed 7814/7814 predicates with 334 commands
(`release\vm-reports\shadow-20260704-184615-9224.json`). The 2026-07-03
failures are classified in the PROJECT_STATUS failure log as host-harness
audit-window failures, not guest regressions. One remaining host-transport
mode is a mid-run QEMU process exit (connect attempts find no listener; the
same-day failed run `shadow-20260704-183440-16492.json` burned its whole
timeout reconnecting); packet M0-2 instruments the harness to classify this
structurally (qemu_exited / listener_missing_process_alive /
connect_timeout_listener_present).
The 2026-07-01 full module-loader report
`release\vm-reports\shadow-20260701-150922-9752.json` used
`-TimeoutSeconds 300 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10 -SerialTcpPort 4579`;
an earlier same-slice run on port 4578 closed the host TCP serial write after
183/183 predicates and 13 commands had passed, and rerunning on a fresh port
with smaller delayed writes passed. Earlier runs with a 180-second command
window timed out while waiting for final markers in long module-loader
responses.
The 2026-07-02 target-region discovery full report
`release\vm-reports\shadow-20260702-174421-7208.json` used the same delayed
serial-write pattern on port 4591 and passed 6789/6789 predicates in
`duration_ms: 553963`.
The earlier 2026-07-02 durable append-authority preflight full report
`release\vm-reports\shadow-20260702-171942-19692.json` used the same delayed
serial-write pattern on port 4587 and passed 6780/6780 predicates in
`duration_ms: 559199`.

Stage-0 serial command-mode input echoes bytes to the serial log without
forcing framebuffer redraws during long pasted commands; this keeps long
hash-reference recovery diagnostics on the real serial path without paying a
full UI render for every input chunk. The 2026-07-03 focused recovery report
`release\vm-reports\shadow-20260703-072638-30256.json` passed 2799/2799
predicates with 142 executed commands in `duration_ms: 222896` after the
source-bound side-effect gate reference update.

For fast iteration, run the same real QEMU/serial path with the quick profile:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile quick
```

`-Profile quick` covers boot readiness, core read-only agent methods,
provider-minimal context/export gates, denied module loading, denied recovery
artifact loading, and RAM-only audit visibility. It intentionally skips the
exhaustive module/recovery negative matrix; the default `-Profile full` remains
the pre-commit/release evidence path.

Use the focused provider-memory profile for provider-minimal redaction,
provider export denial, and provider context injection-gate changes:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile provider-memory -TimeoutSeconds 180 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10
```

`-Profile provider-memory` runs the common provider-minimal context/export
checks plus `provider.context_injection_gate provider_minimal` and the terminal
`provider.context_injection_gate_selftest provider_minimal` omission-hash
negative case, without the long full module/recovery matrix.

For the broader provider-memory assertions used by the full profile without the
module/recovery matrix, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile provider-memory-full -TimeoutSeconds 180 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10
```

`provider-memory-full` runs the large `provider.context_gate_selftest` as its
terminal command, matching the full profile ordering. Use the smaller
`provider-memory` profile for the terminal injection-gate selftest.

For focused Hello rollback transaction-append dry-run coverage, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile hello-rollback-dry-run -TimeoutSeconds 300 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10
```

`-Profile hello-rollback-dry-run` keeps the real QEMU/serial path, loads the
built-in Hello service, hot-swaps to v2, runs rollback preview, proves
`recovery.rollback_inspect_source_reference_selftest` accepts the valid retained
source-reference RAM-audit candidate and rejects stale, wrong-variant,
substituted, and authorizing candidates,
`recovery.rollback_inspect` is read-only before target-sector materialization,
proves `service.rollback_apply svc.demo.hello` reports missing retained
materializer evidence without target-region writes, runs
`recovery.rollback_materialize_dry_run svc.demo.hello` as current-boot test
infrastructure, proves `recovery.rollback_inspect` returns the verified
target-sector hashes/offsets and retains
`raios.recovery_rollback_inspect_source_reference.v0`, validates that source
reference against retained current-boot RAM audit source/audit events, proves
rollback apply reports a missing inspect source reference before that read-only
inspection, then runs the still-denied rollback apply dry-run over retained
materializer evidence plus the validated inspect source reference while
write/append/apply authority remains false. The same slice also runs at the end
of `-Profile full`.

For focused module audit/rollback availability and write-boundary coverage, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile module-audit-rollback -TimeoutSeconds 360 -SerialWriteChunkSize 16 -SerialWriteDelayMilliseconds 10
```

`-Profile module-audit-rollback` keeps the real QEMU/serial path, runs the
common boot/provider probes, the existing module evidence profile, and the
module audit/rollback write-boundary profile without the full
provider/recovery/hello matrix.

For focused recovery work, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile recovery -TimeoutSeconds 180
```

`-Profile recovery` keeps the real QEMU/serial path and the full recovery
lifeline chain, but skips the long provider selftest, memory mutation, and
normal module-loading diagnostic matrix. The harness writes serial commands in
chunks by default (`-SerialWriteChunkSize 256 -SerialWriteDelayMilliseconds 0`);
increase `-SerialWriteDelayMilliseconds` only if a local serial transport starts
dropping command bytes.

Shadow VM reports derive `commands` from the actual `Send-AgentCommand` calls
observed during the run. Each report also includes `executed_commands` entries
with the command, predicate name, expected marker, response offset, duration,
`sent`, and pass/fail result. Commands are added only after the serial write
returns; a connection failure before writing must not appear as an executed
command. Do not maintain a separate static command inventory in the report; it
drifts from the real serial path.

If the report failure is only `Timed out connecting to QEMU serial TCP port
4565`, first check for a stale `qemu-system-x86_64` process or an occupied
serial port, stop stale QEMU processes, and rerun the smoke. The TCP serial path
is single-client in practice, so concurrent harnesses or manual serial clients
can make an otherwise valid build look stuck. The shadow harness now takes a
named per-port mutex before packaging or starting QEMU and rejects a concurrent
smoke on the same port instead of letting `-StopExisting` terminate the older
run. Manual QEMU/serial clients are outside that mutex; use a distinct port or
stop them first.

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

Bare-metal support is experimental. Start with `docs/architecture/hardware/bare-metal.md`.

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

> The retired per-schema provider probe catalogue is preserved in _archive/2026-07-18_DEBUGGING_history.md.

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

### A profile you rewrote from the design manifest fails the first time it actually runs

Symptom: a profile that has not been executed for a while fails on expectations that a
recent slice wrote — and the KERNEL turns out to be right every time. Hit 2026-07-14 on
`quick -Network`, which had not run since 2026-07-13 16:00 while two P4 slices rewrote its
hello block from the semantic manifest and closed on `full`+`recovery`+`persistence` only.

Cause: expectations were derived from the DESIGN, not from observed output. Three drifted
at once: the generic `capability_denial` moved to the v1 envelope (event_id top-level, code
under `facts`) while the profile still read `.body`; health after a hot-swap honestly gained
`state_migration` evidence; and the host-bound descriptor's honest `rejected` status
(unsigned, hash-bound by design) was asserted as `verified`.

Rules that follow:
1. **A slice that rewrites a profile's expectations must RUN that profile in the same
   slice.** `full` green is not a substitute — profiles do not overlap as much as they look.
2. When a needle fails, get the ACTUAL response out of the serial log before touching the
   needle. A code reading is not evidence: on this same day, reasoning from
   `rollback_apply_verified(pre_apply_snapshot.state_migration, …)` produced the wrong
   conclusion (that the migration survives rollback); the live response said
   `migration: null`, because the rollback UNDID it.
3. Batch-check before you re-run. One `quick` run costs minutes; grep every same-class needle
   against one captured serial log instead of discovering them one boot at a time.
4. A needle fix must be at least as strict as what it replaces. Each of the three fixes added
   assertions (v1 schema/family/empty-grants; the all-zero signature hash proving the
   host-bound path cannot become a signed loader path).
5. Watch for ONE method answering in TWO shapes: `service.hot_swap` renders the v0 `body`
   envelope for its hello-family denial and the v1 envelope for the generic one. Read each
   response in its own shape; do not "fix" the helper to accept both.

### A service silently stops loading after you re-signed a descriptor (the BOM trap)

Symptom: the build is GREEN, but in the VM a built-in service no longer loads. `module.load_ephemeral
svc.demo.hello` comes back as a generic `module.load_gate` DENIAL instead of a `hello.lifecycle`
load. Nothing in the dispatch, the predicate, or the emitter changed. Hit 2026-07-13.

Cause: the descriptor file grew a UTF-8 BOM (`EF BB BF`). PowerShell's
`Set-Content -Encoding utf8` on Windows PowerShell 5.1 PREPENDS a BOM. If you rewrite a `.desc`
with it while iterating attestation pins, you corrupt the descriptor's bytes.

Why the build does not catch it: `build.rs` hashes WHATEVER BYTES IT FINDS, so a BOM is just three
more bytes to hash — the pin asserts still pass and the signature still verifies (you re-signed the
corrupted bytes). The kernel is stricter: `key_value_text_is_canonical()` rejects a BOM, so
`validate_builtin_hello_artifact_identity()` returns false, `load_request()` returns `None`, and the
dispatcher falls through to the generic denial. **A byte-attested file's build check cannot catch a
change to the bytes it hashes.**

Diagnosis: `head -c 8 <file> | xxd`. Canonical descriptors start with the first key, not `efbbbf`.
Fix: strip the three bytes at byte level and re-sign. Never write a byte-attested file through
PowerShell text encoding — use `[System.IO.File]::WriteAllBytes`, or
`[System.IO.File]::WriteAllText` (UTF-8, no BOM).

Related: editing attested hello sources legitimately moves the source-set pins (ADR 0013). Iterate
them build-as-oracle (`left:` = the pin in the descriptor, `right:` = the value computed from the
real sources), re-sign with `descriptor-resign` after each, and NEVER weaken `build.rs`. The tool
mints a fresh dev key on every sign — that is by design; the (desc, pub, sig) triple stays
self-consistent and the key is not a trust anchor.

### A harness script dies with "cannot call a method on a null-valued expression" pointing at a line that is demonstrably fine

Symptom: a PowerShell harness script fails in `Write-Report` (shadow-vm-smoke-support.ps1)
with `InvokeMethodOnNull`, and the reported line is `$Predicates.ToArray()`. Chasing it
leads to an imaginary scope bug: `$Predicates` is NOT null. Hit 2026-07-13 on
`shadow-vm-persistence-reboot.ps1`.

TWO traps, and you must know both or you will lose hours:

1. **PowerShell misattributes the line inside a big multi-line hashtable literal.** The
   real failure was `@($script:VisualEvidence.ToArray())` a few lines below; the error was
   reported against a neighbouring `$Predicates` line. NEVER trust the reported line inside
   an `[ordered]@{ ... }` literal. Probe instead: paste the "failing" expression at the TOP
   of the function and print it. If it evaluates fine there, the reported line is a lie —
   look for another METHOD call (`.ToArray()`, not `.Count`; `$null.Count` returns 0 without
   throwing, so only method calls raise `InvokeMethodOnNull`) on a variable the caller never
   initialized.

2. **An unguarded `Write-Report` in a top-level `finally` eats the primary exception.** Its
   throw REPLACES the exception already unwinding, so every failure of the script arrives
   disguised as a report-write error. `shadow-vm-smoke.ps1` guards this call; any script that
   does not, must. Guard it, and print the primary failure in the `catch` of the main body at
   the moment it happens — do not let the rethrow race the finally.

Root cause class: **support drift.** `shadow-vm-smoke-support.ps1` is dot-sourced and reads
caller-owned script-scope state (`$Predicates`, `$ExecutedCommands`, `$script:VisualEvidence`,
...). When support grows a new one, every dot-sourcing script must initialize it.
`shadow-vm-smoke.ps1` was updated; `shadow-vm-persistence-reboot.ps1` was not, and its 119
passing predicates went dark for weeks. When adding state to support.ps1, grep for every
script that dot-sources it.

### build.rs dies with an artifact/identity sha256 assert in a fresh checkout

Symptom: `seed-kernel\build.rs` panics on an `assertion left == right failed`
comparing two `sha256:...` values, in a FRESH clone or `git worktree add` tree,
while the primary working tree builds fine.

Likely cause: `core.autocrlf` smudged a byte-attested text file on checkout
(LF -> CRLF), changing its pinned hash. Observed 2026-07-12 with
`seed-kernel/artifacts/svc.demo.hello.builtin.artifact` (249 -> 256 bytes)
failing the `artifact_reference_sha256` assert.

Fix: every byte-attested path must be listed with `-text` in `.gitattributes`
(hello source set, `seed-kernel/descriptors/**`, `seed-kernel/artifacts/**`).
For an already-smudged tree, copy the file byte-exact from the primary
worktree or re-checkout after the `.gitattributes` fix.

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

### Surface WiFi connection stops after password entry

The guided flow now reports the real bounded connection stage in Settings:

```text
LINK: register_rings
LINK: mac_control
LINK: supplicant_profile
LINK: supplicant_pmk
LINK: associate
LINK: wait_port_release
LINK: link_ready
```

For WPA2, `associate` success is not enough. `link_ready` is authorized only
after event `0x002b` (`PORT_RELEASE`); only then is the Marvell `WifiPhy`
attached and DHCP polling enabled. Useful serial lines are:

```text
marvell wifi: bounded association sequence started
marvell wifi: association accepted; waiting for secure port release
marvell wifi: secure port released; data link and DHCP enabled
authenticated Marvell WiFi link attached; DHCP polling enabled
DHCP lease acquired: ip ...
```

`firmware_supplicant_unavailable`, `security_unsupported`, command/port-release
timeouts, rejected association status, or stale response sequence are explicit
fail-closed results. Do not interpret `KEY SET` or association acceptance alone
as a network link.

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
