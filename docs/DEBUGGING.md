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
completed response), record the verdict in `docs/PROJECT_STATUS.md`, and only
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
can make an otherwise valid build look stuck.

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

For request modes that are allowed to start the direct provider path, the smoke
also expects a local-only pre-write marker:

```text
OPENAI_PROVIDER_REQUEST_ENVELOPE {"schema":"raios.provider_request_envelope.v0", ...}
```

That marker must report `provider_write: not_attempted`, include body and
envelope hashes, and omit raw prompt text, `Content-Length`, API keys, and
Authorization values.

On pinned/WebPKI positive trust paths with a matching request-body and envelope
hash, the smoke also expects:

```text
OPENAI_PROVIDER_REQUEST_BINDING {"schema":"raios.provider_request_binding.v0", ...}
OPENAI_PROVIDER_EXPORT_AUDIT_BINDING {"schema":"raios.provider_context_export_audit_binding.v0", ...}
OPENAI_PROVIDER_CONTEXT_INJECTION_GATE {"schema":"raios.provider_context_injection_gate.v0", ...}
```

Those markers must stay absent for pin mismatch and the unverified development
TLS override. The export-audit marker is positive audit evidence, and the
injection-gate marker is a blocked prewrite diagnostic; both keep
`automatic_context_injection` `disabled`,
`satisfies_current_boot_export_gate` remains `false`, and the request body still
does not include provider-minimal context. Provider-minimal context hashes must
include the packet hash, exported/omitted field-list hashes,
`redaction_policy_hash`, `field_classification_hash`, and `token_budget_hash`;
provider-minimal, gate/export, and injection-gate responses must keep
`current.recovery_lifeline_status` and
`recovery.lifeline.status.current_boot` local-only and omitted from provider
context.
Pinned-trust markers must also expose
`raios.provider_trust_verifier_metadata.v0` with the verifier id, exact-host
policy, pin policy, and the explicit Stage-0 chain/time policy. Positive
pinned-trust markers must also carry
`raios.provider_trust_verifier_decision.v0` with `stage: certificate_verify`,
`outcome: verified`, and a leaf/SPKI verified reason; the no-pin/no-trust
snapshot and provider-minimal context should show `stage: pin_config`,
`outcome: rejected`, `reason: pin_config_missing`, and
`pin_rotation_policy: missing_active_pin`. A configured standby SPKI pin should
show `pin_rotation_policy: active_spki_plus_rotation_spki`; malformed standby
config must fail closed as `pin_config_invalid`.

Pinned-trust direct smokes also exercise the checked local gate:

```text
agent provider.context_gate provider_minimal
agent provider.context_export provider_minimal
agent provider.context_export provider_minimal
```

The first command must report `raios.provider_context_export_gate_state.v0` with
`binding_validation_status: valid`. The first export command consumes the
retained positive binding pair for local gate evaluation only and records
`raios.provider_context_binding_consumption.v0`; it still returns
`capability_denied`. The second export command must reject the same pair with
`binding_already_consumed`.

The Shadow VM smoke also exercises local-only negative gate selftests:

```text
agent provider.context_gate_selftest provider_minimal
agent provider.context_injection_gate provider_minimal
agent provider.context_injection_gate_selftest provider_minimal
```

The export-gate selftest emits
`raios.provider_context_gate_negative_selftest.v0`, does not mutate the global
event log, does not create request envelopes or positive binding records, and
checks stale/dropped ids, previous-boot-or-unretained ids, substituted
denial/positive records, request/body/binding/context hash mismatches,
redaction/classification/budget/trust-evidence hash mismatches, and
trust-bypass records.

`provider.context_injection_gate` emits
`raios.provider_context_injection_gate.v0`; it names the final authorization
schema `raios.provider_context_injection_authorization.v0`, reports that
authorization as missing, and keeps `can_attach_context: false`.

The final-injection selftest emits
`raios.provider_context_injection_gate_negative_selftest.v0`, keeps provider
write and body attachment disabled, and checks missing, stale, substituted,
body-hash mismatched, trust-downgraded, and unauthorized body-attachment final
authorization candidates.

The Shadow VM smoke also exercises the denied module load gate:

```text
module.load_ephemeral
agent audit.events 8
```

The expected response schema is `raios.module_load_gate.v0`. It must report the
manifest, candidate artifact, VM report, local attestation, computed grant,
local approval, durable audit record, rollback plan, loader, and ram-only
service slot as missing or unavailable, with `can_load: false`,
`service_inventory_change: none`, and `load_attempted: false`. The follow-up
`audit.events` read must show a matching `raios.module_load_gate.v0` event
binding.

The native agent command envelope slice is intentionally limited to a small
read-only target allowlist:

```text
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=system.describe requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=system.snapshot requested_capability=cap.system.snapshot.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=system.boot_log requested_capability=cap.system.boot_log.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=system.capabilities requested_capability=cap.system.capabilities.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=device.graph requested_capability=cap.device.graph.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.inventory requested_capability=cap.service.inventory.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.health requested_capability=cap.service.health.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.rollback_preview requested_capability=cap.service.rollback_preview.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=recovery.rollback_inspect requested_capability=cap.recovery.rollback_inspect.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_availability requested_capability=cap.module.grant_diagnostic.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_write_policy requested_capability=cap.module.grant_diagnostic.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_storage_layout requested_capability=cap.module.grant_diagnostic.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_engine requested_capability=cap.module.grant_diagnostic.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_contract requested_capability=cap.module.grant_diagnostic.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_payload_hash requested_capability=cap.module.grant_diagnostic.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_intent requested_capability=cap.module.grant_diagnostic.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_write_boundary requested_capability=cap.module.grant_diagnostic.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=problem.list requested_capability=cap.problem.list.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.inventory requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.health requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=recovery.rollback_inspect requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_availability requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_write_policy requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_storage_layout requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_engine requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_contract requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_payload_hash requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_intent requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_write_boundary requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=bad target_method=system.describe requested_capability=cap.system.describe.read classification=local_only
agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.load_ephemeral requested_capability=cap.module.load_ephemeral classification=local_only
agent audit.events 68
```

The valid envelopes must return `raios.agent_command_envelope.v0` with
`accepted: true`, current-boot `event_id`/`audit_event_id`,
`dispatches_existing_agent_method: true`, and then emit the normal
`system.describe`, `system.snapshot`, `system.boot_log`,
`system.capabilities`, `device.graph`, `service.inventory`,
`service.health`, `service.rollback_preview`, `recovery.rollback_inspect`, or
`module.audit_rollback_availability`, `module.audit_rollback_write_policy`,
`module.audit_rollback_storage_layout`,
`module.audit_rollback_append_engine`,
`module.audit_rollback_append_contract`,
`module.audit_rollback_append_payload_hash`,
`module.audit_rollback_append_intent`,
`module.audit_rollback_write_boundary`, or `problem.list` response. A
target/capability mismatch must return `reason: requested_capability_denied`
without dispatching `service.inventory`, `service.health`,
`service.rollback_preview`, `recovery.rollback_inspect`, or
`module.audit_rollback_availability`/`module.audit_rollback_write_policy`/
`module.audit_rollback_storage_layout`/`module.audit_rollback_append_engine`/
`module.audit_rollback_append_contract`/
`module.audit_rollback_append_payload_hash`/
`module.audit_rollback_append_intent`/
`module.audit_rollback_write_boundary`.
Bad-schema and over-capable envelopes
must return the same envelope schema with `accepted: false` and must not
dispatch `module.load_ephemeral`; `audit.events` must show thirty-one local-only
`raios.agent_command_envelope.decision` events with
`raios.agent_command_envelope.audit_binding.v0`. The envelope response and
audit binding must keep provider writes, candidate-byte loading, persistence,
durable audit writes, rollback install, parallel dispatch, and broad mutation
disabled.

`system.honesty_report.owner_key_provisioning` must report the key lifecycle
posture without exporting generated key material: persistent install policy
`generate_hardware_bound_owner_key_on_persistent_install`, RAM boot policy
`ephemeral_current_boot_key_only`, `automatic_generation_intended: true`,
`automatic_generation_performed: true`,
`hardware_binding_probe_source: acpi_tpm2_table`,
`hardware_binding_probe_performed: true`,
`hardware_binding_probe_status: tpm2_acpi_absent` in the focused QEMU profile,
`hardware_binding_probe_reason: TPM2 ACPI table missing`,
`tpm2_acpi_table_present: false`,
`tpm2_status_read_plan_available: false`,
`tpm2_status_register_kind: none`,
`tpm2_status_register_phys: 0`,
`tpm2_status_register_width_bytes: 0`,
`hardware_binding_evidence_present: false`,
`persistent_owner_key_generated: false`,
`ram_boot_ephemeral_key_generated: true`,
`ram_boot_ephemeral_key_id: owner_key.ram_candidate.current_boot`,
`ram_boot_ephemeral_key_handle: owner_key.handle.current_boot.ram0`,
`ram_boot_ephemeral_key_algorithm: ram_32_byte_entropy_seed_sha256_fingerprint`,
`ram_boot_ephemeral_key_secret_len: 32`,
`ram_boot_ephemeral_key_material_classification: secret`,
`ram_boot_ephemeral_key_fingerprint: sha256:<64 lowercase hex>`,
`owner_key_material_exported: false`,
`ram_ephemeral_candidate_generated_persistent_hardware_binding_missing`, and no
owner-seal, persistent-install, load, or durable-write authority.

For the real Surface Pro 4 TPM capture, type `ownerkey` on the console/serial
path. It prints the same owner-key snapshot in short form: RAM handle and
`sha256:` fingerprint, TPM2 ACPI present/phys/length/revision, TPM2 interface
kind/start/control/status, and `OWNER AUTH: SEAL NO PERSIST NO LOAD NO DURABLE
NO`. It also prints the planned read-only TPM status register, if CRB/TIS
details expose one. On the focused QEMU profile this must show
`TPM2 ACPI: PRESENT NO PHYS 0x0000000000000000 LEN 0 REV 0` and
`TPM2 STATUS: tpm2_acpi_absent REASON TPM2 ACPI table missing` and
`TPM2 STATUS READ: PLAN NO KIND none PHYS 0x0000000000000000 WIDTH 0 REASON TPM2 ACPI table missing`.
Its nested `owner_key_evidence_input` must consume `core.entropy`, report
`entropy_evidence_present: true`, `entropy_status: ready`, RDRAND observed,
pool capacity 64 with total collected at least 32,
`hardware_binding_source: tpm_or_platform_seal`,
`hardware_binding_probe_source: acpi_tpm2_table`,
`hardware_binding_probe_performed: true`,
`hardware_binding_probe_status: tpm2_acpi_absent` in the focused QEMU profile,
`hardware_binding_probe_reason: TPM2 ACPI table missing`,
`acpi_rsdp_present: true`, `acpi_root_table_valid: true`,
`tpm2_acpi_table_present: false`, `tpm2_acpi_table_length: 0`,
`tpm2_acpi_table_revision: 0`,
`tpm2_status_read_plan_available: false`,
`tpm2_status_register_kind: none`, `tpm2_status_register_phys: 0`,
`tpm2_status_register_width_bytes: 0`,
`hardware_binding_evidence_present: false`,
`tpm_binding_state: tpm2_acpi_absent`,
`ram_boot_ephemeral_input_ready: true`,
`persistent_install_input_ready: false`, `ram_candidate_generated: true`,
`ram_candidate_id: owner_key.ram_candidate.current_boot`,
`ram_candidate_handle: owner_key.handle.current_boot.ram0`, a
`ram_candidate_fingerprint` matching the provisioning `sha256:` fingerprint,
and no key-generation, owner-seal, persistent-install, load, or durable-write
authority.

The first positive RAM-only service slice is deliberately narrower than general
module loading:

```text
module.load_ephemeral svc.demo.nope
module.load_ephemeral external:svc.demo.hello
service.descriptor_source_trust_selftest
service.artifact_reference_trust_selftest
service.artifact_load_plan_preflight_selftest
recovery.rollback_inspect_source_reference_selftest
module.load_ephemeral svc.demo.hello
services
service.health svc.demo.hello
service.stop svc.demo.hello
service.health svc.demo.hello
service.start svc.demo.hello
service.restart svc.demo.hello
service.hot_swap svc.demo.hello.reset_state
service.health svc.demo.hello
service.hot_swap external:svc.demo.hello
service.health svc.demo.hello
service.hot_swap svc.demo.hello
service.hot_swap svc.demo.hello.v2
service.rollback_preview svc.demo.hello
service.health svc.demo.hello
recovery.rollback_materialize_dry_run svc.demo.hello
recovery.rollback_inspect svc.demo.hello
service.rollback_apply svc.demo.hello
recovery.rollback_inspect svc.demo.hello
service.health svc.demo.hello
service.hot_swap svc.demo.hello
service.drop svc.demo.hello
service.health svc.demo.hello
module.load_ephemeral host_bound:svc.demo.hello
services
service.health svc.demo.hello
service.stop svc.demo.hello
service.drop svc.demo.hello
agent audit.events 72
```

The two wrong-target commands must still return `raios.module_load_gate.v0`
with `capability_denied`. The positive command must return
`raios.ram_only_hello_service.v0` with
`raios.current_boot_load_request.v0`,
`raios.current_boot_load_descriptor.v0`, and
`load_descriptor.current_boot.svc.demo.hello.v0`. The load response must expose
the descriptor source locator
`current_image.descriptor_source.svc.demo.hello.v0`, source kind
`current_image_descriptor_source`, `validated: true`, and a `sha256:` source
hash, and the source text must carry the canonical key/value fields used by the
validator, including `source_locator` and `source_kind`. The current-image
source must also expose a `raios.descriptor_source_signature_envelope.v0`
object with `algorithm: ecdsa_p256_sha256_asn1_der`,
`verification_phase: runtime_before_descriptor_selection`, matching payload
hash, SHA-256 hashes for the envelope/public key/signature, and
`signature_verified: true`; the envelope must not authorize external artifact
loading or persistent install. The same response must expose a
`raios.builtin_artifact_identity.v0` object for `builtin:svc.demo.hello` with a
`sha256:` identity hash and a
`raios.builtin_artifact_identity_signature_envelope.v0` whose payload hash
matches the identity hash and whose signature verifies; the identity/envelope
must not authorize external artifact load, executable page mapping,
persistence, or rollback. The identity must include a
`raios.builtin_artifact_content_binding.v0` content binding with a `sha256:`
binding hash, `seed-kernel/src/hello_service.rs` source locator, verified trust
envelope linkage, and external artifact load, executable mapping, and
persistence all disabled. The identity must also include a
`raios.builtin_artifact_reference.v0` artifact reference with a `sha256:`
reference hash, `sha256:` artifact byte hash, content-binding hash linkage,
`seed-kernel/artifacts/svc.demo.hello.builtin.artifact` locator, verified trust
envelope linkage, and artifact byte intake, code loading, executable mapping,
and persistence all disabled. The load response and nested descriptor must also
expose `raios.current_boot_artifact_load_plan_preflight.v0` with a `sha256:`
preflight hash that binds the selected descriptor source, artifact identity,
content binding, artifact reference, artifact bytes, and
`ram_only:svc.demo.hello` service-slot intent while denying candidate-byte
execution, executable mapping, persistence, durable audit, rollback, and broad
mutation. `services` must show
`svc.demo.hello` only while loaded and cite the same
descriptor id/source/kind/validation/hash/signature envelope plus the same
artifact identity hash/signature envelope, content binding hash, and artifact
reference hash plus artifact load-plan preflight id/hash/status and
service-slot activation id/hash/status/active state. `service.health
svc.demo.hello` must return `raios.ram_only_hello_service.health.v0`, report
healthy while loaded/running, stopped while loaded/not running, running again
after `service.start`, still running after `service.restart` with the same
loaded generation, still unchanged after denied `service.hot_swap
svc.demo.hello.reset_state` and denied `service.hot_swap
external:svc.demo.hello`, advanced by one generation after accepted
`service.hot_swap svc.demo.hello`, advanced again with visible `version: "v2"`
after `service.hot_swap svc.demo.hello.v2`, back to `version: "v1"` after the
final `service.hot_swap svc.demo.hello`, and missing after drop, and cite the
active descriptor source hash, signature envelope,
artifact identity/content/reference/preflight evidence, and service-slot
activation evidence while loaded. Stop keeps the same activation hash with
`stopped_current_boot`; start and restart keep the same activation hash with
`active_current_boot`; restart records `last_action: "restart"` and reason
`restarted_loaded_service`; hot-swap records `last_action: "hot_swap"` and
reason `hot_swapped_builtin_service`; the Hello state record
`raios.ram_only_hello_service_state.v0` must start at counter 1, advance through
start/restart to counter 3, stay unchanged across the denied reset-state
hot-swap and accepted v1/v2 hot-swaps, expose a denied
`raios.ram_only_hello_service_state_migration.v0` with pre-counter 3,
post-counter 0, `state_preserved: false`, and `accepted: false` for
`svc.demo.hello.reset_state`, and expose an accepted migration record with
matching pre/post state hash and counter for the v1->v2 transition. Accepted
hot-swaps must also expose
`raios.ram_only_hello_service_hot_swap_probation.v0` with
`active_current_boot_probation` status, previous/new descriptor and artifact
identity hashes, previous/new generation, previous/new state hash/counter, and
the matching state-migration hash while candidate bytes, executable mapping,
persistent state, durable audit, rollback install, and rollback apply stay
denied. `service.rollback_preview svc.demo.hello` must return
`raios.ram_only_hello_service_rollback_preview.v0`, bind the v1->v2 probation
hash, expose rollback-target and current-candidate descriptor/artifact
identity/generation/state/migration facts, keep apply/install/write surfaces
denied, and a follow-up health probe must prove the active v2 generation and
state are unchanged. `service.rollback_apply svc.demo.hello` must return
structured `capability_denied`, bind the rollback-preview hash, probation hash,
current state hash/counter, rollback target, current candidate, and migration
hash, expose
`raios.ram_only_hello_service_rollback_transaction_preflight.v0` with a
`sha256:` preflight hash over the apply-denial hash, preview/probation/state
evidence, target/current candidate facts, requested rollback-apply capability,
and missing rollback-transaction, durable-audit-write, and persistent-install
authorities, expose
`raios.ram_only_hello_service_rollback_write_authority_gate.v0` with a
`sha256:` gate hash over the preflight hash, required `raios.audit_record.v0`
and `raios.rollback_transaction.v0` schemas, unavailable durable-audit-write,
rollback-store-write, and transaction-append authority, and disabled write/apply
side effects, expose
`raios.ram_only_hello_service_rollback_append_intent_gate.v0` with a `sha256:`
gate hash over the write-authority gate hash, preflight hash,
apply-denial/preview/probation/state evidence, target/current candidate facts,
required durable schemas, unavailable append/durable-store authority, and
disabled append/write/apply side effects, expose
`raios.ram_only_hello_service_rollback_payload_envelope_gate.v0` with a
`sha256:` gate hash over the append-intent gate hash, write-authority gate hash,
preflight hash, apply-denial/preview/probation/state evidence, target/current
candidate facts, proposed `raios.rollback_transaction.v0` payload
schema/id/hash, payload provenance hash, required durable schemas, unavailable
transaction-writer/durable-store/append authority, and disabled
append/write/apply side effects, expose
`raios.ram_only_hello_service_rollback_transaction_writer_storage_authority_gate.v0`
with a `sha256:` gate hash over the payload-envelope gate hash, payload and
provenance hashes, append-intent gate hash, write-authority gate hash,
preflight hash, apply-denial/preview/probation/state evidence, target/current
candidate facts, required durable schemas, unavailable
transaction-writer/durable-audit-store/rollback-store/append authority, and
disabled append/write/apply side effects, plus the shared
`raios.module_audit_rollback_append_contract.v0` foundation status for
`storage.authority.audit_rollback.current_boot`,
`append.audit_ledger.current_boot`, and `append.rollback_store.current_boot`
as the rollback transaction append target, expose
`raios.ram_only_hello_service_rollback_durable_writer_policy_preflight.v0` plus
`raios.ram_only_hello_service_rollback_durable_append_transaction_authorization_gate.v0`
binding append-record, sector-plan, target-region write/readback, audit-ledger,
rollback-store, LBA1/512-byte span, test-media, an accepted current-boot
no-write append-engine candidate, missing durable-audit writer, missing
rollback-store writer, and missing transaction append writer evidence with all
authorize/write/append flags false, keep actual
rollback apply,
descriptor mutation, generation mutation, running-state mutation, RAM-only state
mutation, persistent install, durable audit write, rollback-store write,
transaction append, external bytes, candidate execution, executable mapping,
provider auto-load, and broad mutation denied or not attempted, and a follow-up
health probe must prove the active v2 descriptor, generation, running state, and
state hash are unchanged; drop cites
the same activation hash with
`cleared_current_boot` before cleanup and clears the state counter.
`audit.events` must show
`raios.ram_only_hello_service.lifecycle` and
`raios.ram_only_hello_service.health` records whose evidence/bindings cite the
same load descriptor, validated source hash, signature envelope hash, and
signature verification state plus the verified artifact identity hash and
signature envelope plus the artifact content binding hash and trust signature
state plus artifact reference hash, byte hash, trust signature state, artifact
load-plan preflight hash, accepted status, RAM-only service-slot id, and
service-slot activation hash/status plus Hello state hash/counter, denied
reset-state migration evidence, and accepted v2 state-migration
hash/preserved-state evidence plus the accepted hot-swap probation hash/status
and previous/new descriptor, artifact identity, generation, state, and
migration facts, plus the rollback-preview event binding with the same
probation hash and explicit no-apply/no-install evidence, plus the
rollback-apply denial event binding with the same preview/probation/state hashes
plus rollback transaction preflight, write-authority gate, append-intent gate,
payload-envelope gate, payload/provenance, and writer/storage authority gate
hashes, shared writer/storage foundation schema/owner/status fields,
`storage.authority.audit_rollback.current_boot`,
`append.audit_ledger.current_boot`, `append.rollback_store.current_boot`,
the transaction writer owner, append-target-owner and transaction-writer
readiness facts denied by `persistence_device_write_path_missing`, durable
writer-policy preflight fields, durable append/transaction authorization gate
fields, append-engine readiness decision fields, required durable schemas,
explicit unavailable write/store/append authority,
no-apply, no-durable-write, no-rollback-store-write, no-transaction-append,
and no-mutation evidence.

`service.descriptor_source_trust_selftest` must return
`raios.descriptor_source_trust_selftest.v0`, expose a stable diagnostic id and
hash, pass five read-only cases for the valid envelope plus tampered payload,
locator/kind, public-key hash, and signature, and keep descriptor byte intake,
external artifact load, persistence, durable audit, rollback, and broad mutation
denied.

`service.artifact_reference_trust_selftest` must return
`raios.builtin_artifact_reference_trust_selftest.v0`, expose a stable
diagnostic id and hash, cite the validated artifact reference plus the verified
artifact identity trust envelope, pass five read-only cases for the valid
reference plus tampered artifact byte hash, content-binding hash, reference
hash, and trust payload hash, and keep artifact byte intake, artifact load,
executable mapping, persistence, durable audit, rollback, broad mutation, and
global event-log mutation denied.

`service.artifact_load_plan_preflight_selftest` must return
`raios.current_boot_artifact_load_plan_preflight_selftest.v0`, expose a stable
diagnostic id and hash, cite the accepted artifact load-plan preflight plus the
RAM-only service-slot intent, pass eight read-only cases for the valid
preflight plus tampered descriptor-source hash, artifact identity hash,
content-binding hash, artifact reference hash, artifact byte hash, service-slot
intent, and denial flags, and keep candidate execution, executable mapping,
persistence, durable audit, rollback, broad mutation, and global event-log
mutation denied.

The host-bound positive command must cite
`host_build.descriptor_source.svc.demo.hello.v0`, source kind
`host_bound_descriptor_source`, and `binds_source_locator`,
`binds_source_kind`, and `binds_source_hash` equal to the current-image source
locator/kind/hash. The host-bound health response and health audit event must
cite the host-bound source hash plus the bound current-image source hash and a
host-bound artifact load-plan preflight hash. The path must keep
`signature_envelope: null`, external artifact bytes, candidate-byte execution,
persistence, durable audit writes, rollback installation, and broad mutation
disabled.

After a matching manifest, artifact, Shadow-VM report, and local attestation
exist, compute the host-side grant diagnostic with:

```powershell
cargo run -p registry-tools -- grant-diagnostic `
  --manifest .\candidate.manifest.json `
  --artifact .\candidate.bin `
  --vm-report .\release\vm-reports\shadow-....json `
  --local-attestation .\release\attestations\attest-....json `
  --approval "APPROVE RAM_ONLY <tuple-prefix>"
```

The output schema is `raios.computed_capability_grant.v0`. It may report
`computed_candidate_present: true`, but `grants_capability`,
`grants_load_now`, `can_load_now`, and `load_attempted` must remain false.

After retaining the computed grant reference in the guest, run one denied
`module.load_ephemeral` and use that response's current-boot `event_id` plus
the retained computed-grant `event_id` when building the host-only
audit/rollback diagnostic:

```powershell
cargo run -p registry-tools -- audit-rollback-diagnostic `
  --manifest .\candidate.manifest.json `
  --artifact .\candidate.bin `
  --vm-report .\release\vm-reports\shadow-....json `
  --local-attestation .\release\attestations\attest-....json `
  --approval "APPROVE RAM_ONLY <tuple-prefix>" `
  --computed-grant-hash sha256:<grant-hash> `
  --denial-event-id event.current_boot.<denied-load-id> `
  --retained-reference-event-id event.current_boot.<retained-grant-id> `
  --ram-only-service-slot-id ram_only:svc.example.0001 `
  --pre-load-service-inventory-hash sha256:<inventory-hash> `
  --cleanup-actions-hash sha256:<cleanup-actions-hash>
```

The output schema is `raios.module_audit_rollback_diagnostic.v0` and includes
canonical `raios.audit_record.v0` and `raios.rollback_plan.v0` candidates. It
must still report `durable_audit_written: false`,
`rollback_plan_installed: false`, `can_load_now: false`, and
`load_attempted: false`.

Inside the guest, inspect only the hash reference with:

```text
agent module.manifest_diagnostic
agent module.manifest_diagnostic <manifest_reference_hash> <manifest_hash> [current_boot]
agent module.manifest_diagnostic_selftest
agent module.artifact_diagnostic
agent module.artifact_diagnostic <artifact_reference_hash> <retained_manifest_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <manifest_hash> <computed_grant_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
agent module.artifact_diagnostic_selftest
agent module.vm_report_diagnostic
agent module.vm_report_diagnostic <report_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
agent module.vm_report_diagnostic_selftest
agent module.attestation_diagnostic
agent module.attestation_diagnostic <local_attestation_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_vm_report_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <vm_test_report_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
agent module.attestation_diagnostic_selftest
agent module.approval_diagnostic
agent module.approval_diagnostic <local_approval_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_vm_report_reference_event_id> <retained_local_attestation_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <vm_test_report_reference_hash> <local_attestation_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> [current_boot]
agent module.approval_diagnostic_selftest
agent module.grant_diagnostic
agent module.grant_diagnostic <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
agent module.grant_diagnostic_selftest
agent module.audit_rollback_diagnostic
agent module.audit_rollback_diagnostic <audit_record_hash> <rollback_plan_hash> <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> <pre_load_service_inventory_hash> <cleanup_actions_hash> <denial_event_id> <retained_reference_event_id> <ram_only_service_slot_id> [current_boot]
agent module.audit_rollback_diagnostic_selftest
agent module.service_slot_diagnostic
agent module.service_slot_diagnostic <reservation_hash> <retained_reference_event_id> <retained_audit_rollback_reference_event_id> <computed_grant_hash> <audit_record_hash> <rollback_plan_hash> <pre_load_service_inventory_hash> <ram_only_service_slot_id> [current_boot]
agent module.service_slot_diagnostic_selftest
agent module.service_slot_allocator
agent module.service_slot_allocator_selftest
agent module.loader_runtime
agent module.loader_runtime_selftest
agent module.loader_identity
agent module.loader_identity_selftest
agent module.loader_artifact_hash_binding
agent module.loader_artifact_hash_binding_selftest
agent module.loader_entrypoint_abi
agent module.loader_entrypoint_abi_selftest
agent module.loader_address_space_boundary
agent module.loader_address_space_boundary_selftest
agent module.loader_memory_map_constraints
agent module.loader_memory_map_constraints_selftest
agent module.loader_capability_import_table
agent module.loader_capability_import_table_selftest
agent module.loader_service_slot_binding
agent module.loader_service_slot_binding_selftest
agent module.loader_health_state_hooks
agent module.loader_health_state_hooks_selftest
agent module.loader_rollback_hooks
agent module.loader_rollback_hooks_selftest
agent module.loader_audit_rollback_write_boundary_binding
agent module.loader_audit_rollback_write_boundary_binding_selftest
agent module.audit_rollback_availability
agent module.audit_rollback_availability_selftest
agent module.audit_rollback_write_policy
agent module.audit_rollback_write_policy_selftest
agent module.audit_rollback_storage_layout
agent module.audit_rollback_storage_layout_selftest
agent module.audit_rollback_append_engine
agent module.audit_rollback_append_engine_selftest
agent module.audit_rollback_append_contract
agent module.audit_rollback_append_contract_selftest
agent module.audit_rollback_append_payload_hash
agent module.audit_rollback_append_payload_hash_selftest
agent module.audit_rollback_append_intent
agent module.audit_rollback_append_intent_selftest
agent module.audit_rollback_write_boundary
agent module.audit_rollback_write_boundary_selftest
agent module.load_gate_manifest_selftest
agent module.load_gate_artifact_selftest
agent module.load_gate_vm_report_selftest
agent module.load_gate_attestation_selftest
agent module.load_gate_approval_selftest
agent module.load_gate_retained_selftest
agent module.load_gate_audit_rollback_selftest
agent module.load_gate_service_slot_selftest
agent module.load_gate_loader_runtime_selftest
```

The expected guest schemas are
`raios.module_manifest_reference_diagnostic.v0`,
`raios.module_manifest_reference_diagnostic_selftest.v0`,
`raios.module_candidate_artifact_reference_diagnostic.v0`,
`raios.module_candidate_artifact_reference_diagnostic_selftest.v0`,
`raios.module_vm_test_report_reference_diagnostic.v0`,
`raios.module_vm_test_report_reference_diagnostic_selftest.v0`,
`raios.module_local_attestation_reference_diagnostic.v0`,
`raios.module_local_attestation_reference_diagnostic_selftest.v0`,
`raios.module_local_approval_reference_diagnostic.v0`,
`raios.module_local_approval_reference_diagnostic_selftest.v0`,
`raios.module_computed_grant_diagnostic.v0`,
`raios.module_computed_grant_diagnostic_selftest.v0`,
`raios.module_service_slot_allocator_readiness.v0`, and
`raios.module_service_slot_allocator_readiness_selftest.v0`,
`raios.module_loader_runtime_readiness.v0`, and
`raios.module_loader_runtime_readiness_selftest.v0`,
`raios.module_loader_identity.v0`,
`raios.module_loader_identity_selftest.v0`, and
`raios.module_loader_artifact_hash_binding.v0`,
`raios.module_loader_artifact_hash_binding_selftest.v0`, and
`raios.module_loader_entrypoint_abi.v0`,
`raios.module_loader_entrypoint_abi_selftest.v0`,
`raios.module_loader_address_space_boundary.v0`,
`raios.module_loader_address_space_boundary_selftest.v0`,
`raios.module_loader_memory_map_constraints.v0`,
`raios.module_loader_memory_map_constraints_selftest.v0`,
`raios.module_loader_capability_import_table.v0`,
`raios.module_loader_capability_import_table_selftest.v0`,
`raios.module_loader_service_slot_binding.v0`,
`raios.module_loader_service_slot_binding_selftest.v0`,
`raios.module_loader_health_state_hooks.v0`,
`raios.module_loader_health_state_hooks_selftest.v0`,
`raios.module_loader_rollback_hooks.v0`,
`raios.module_loader_rollback_hooks_selftest.v0`,
`raios.module_loader_audit_rollback_write_boundary_binding.v0`,
`raios.module_loader_audit_rollback_write_boundary_binding_selftest.v0`, and
`raios.module_load_gate_loader_runtime_selftest.v0`. The manifest-reference
schemas must keep `accepts_manifest_json: false`,
`accepts_unsigned_service_code: false`, and `accepts_artifact_bytes: false`; all
of these diagnostics must keep `service_inventory_change: none` and
`load_attempted: false`.

The audit/rollback diagnostic emits
`raios.module_audit_rollback_reference_diagnostic.v0` and
`raios.module_audit_rollback_reference_diagnostic_selftest.v0`. It validates
only canonical hashes and current-boot ids, creates no durable audit records or
rollback plans, allocates no service slot, and keeps `can_load_now: false`.
When the full hash reference is valid, it records only a local-only current-boot
`raios.module_audit_rollback_reference.v0` event binding and reports
`retained_audit_rollback_reference.status:
retained_hash_reference_load_still_denied`.

The service-slot diagnostic emits
`raios.module_service_slot_reservation_diagnostic.v0`. It validates a canonical
reservation hash over retained computed-grant and audit/rollback event ids,
their hashes, the pre-load service-inventory hash, and a `ram_only:` slot id.
When valid, it records only a local-only current-boot
`raios.module_service_slot_reservation.v0` event binding and reports
`retained_service_slot_reservation.status:
retained_hash_reference_load_still_denied`; it still keeps
`allocates_service_slot: false`, `creates_service_inventory_records: false`,
`service_inventory_change: none`, and `load_attempted: false`.

The service-slot allocator readiness diagnostic emits
`raios.module_service_slot_allocator_readiness.v0` and the selftest emits
`raios.module_service_slot_allocator_readiness_selftest.v0`. It consumes the
latest retained service-slot reservation only as local-only current-boot
evidence, records retained current-boot source-evidence for typed allocator
runtime, registry binding, service health-state, and unload cleanup facts, and
reports `raios.ram_only_service_slot_allocator.v0` as
observed-current-boot available once a retained service-slot reservation exists.
With that runtime present it also reports the registry binding, health-state
model, and unload cleanup plan as observed-current-boot available. The
durable-audit write and rollback-install prerequisite gates also become
observed-current-boot available when those allocator facts are available. The
module-loader prerequisite boundary also becomes observed-current-boot available
but non-authorizing, then the diagnostic records
`raios.module_service_slot_allocator_authority_source_evidence.v0`, exposes a
nested `raios.module_service_slot_allocator_authority.v0` boundary, and the
final allocator readiness denial is
`service_slot_allocator_authority_boundary_non_authorizing`. It must keep
`allocates_service_slot: false`, `creates_service_inventory_records: false`,
`can_allocate: false`, `can_load_now: false`, and `load_attempted: false`.

The module loader-runtime readiness diagnostic emits
`raios.module_loader_runtime_readiness.v0` and the selftest emits
`raios.module_loader_runtime_readiness_selftest.v0`. It consumes retained
module evidence, the retained service-slot allocator source-evidence
projection, and the latest retained
`module.loader_identity` plus `module.loader_artifact_hash_binding`
plus `module.loader_entrypoint_abi`, `module.loader_address_space_boundary`,
`module.loader_memory_map_constraints`,
`module.loader_capability_import_table`,
`module.loader_service_slot_binding`,
`module.loader_health_state_hooks`, `module.loader_rollback_hooks`, and
`module.loader_audit_rollback_write_boundary_binding`
source-evidence events only as local-only current-boot inputs, reports missing
loader identity, artifact-hash binding, entrypoint ABI, address-space and
memory-map isolation, capability import table, service-slot binding,
health/rollback hooks, and audit/rollback write-boundary binding facts, and
keeps `loads_artifact: false`,
`allocates_service_slot: false`, `service_inventory_change: none`,
`can_load_now: false`, and `load_attempted: false`. Each aggregate
loader-runtime fact now carries the addressable source diagnostic method and
source fact locator, and `module.loader_runtime_selftest` exposes a
`source_fact_map` so the aggregate required-fact list can be checked against
the typed source methods. With valid retained allocator source evidence, the
live diagnostic should report
`readiness_status: denied_allocator_authority_not_granted`,
`readiness_reason: service_slot_allocator_authority_boundary_non_authorizing`,
and a nested `raios.module_service_slot_allocator_authority.v0` boundary; the
selftest still includes the negative runtime-missing case. The selftest also
includes observed-current-boot
loader identity, artifact-hash, entrypoint-ABI, address-space, memory-map,
capability-table, service-slot, health-hook, rollback-hook, and write-boundary
source-evidence cases. The denied
`module.load_ephemeral` `loader_runtime_readiness` projection and its compact
audit/event binding reuse the same ten base source facts and add the
receiver-identity load preflight as an eleventh non-authorizing source fact, so
load-denial evidence can distinguish receiver/candidate-bound from missing
M6/M7/provider/owner gates. It also reports
`m6_m7_reverify_input_check`, which must show the receiver-preflight source
fact ready while `m6_reverification_input_present`,
`m7_loader_policy_input_present`, `can_enter_m6_reverify`,
`can_enter_m7_loader_policy`, and `authorizes_load` remain false. The nested
`m6_reverify_input_diagnostic` must additionally show
`receiver_preflight_input_ready: true`,
`receiver_candidate_binding_absent: false`,
`m6_reverification_evidence_present: false`, and
`denied_missing_m6_reverify_evidence`, with `authorizes_load` still false. The
nested `m7_loader_policy_input_diagnostic` must show
`m6_reverify_input_ready_for_loader_policy: false`,
`m7_loader_policy_evidence_present: false`,
`denied_m6_reverify_input_not_ready_for_m7_loader_policy`, and
`authorizes_load` still false. The nested
`provider_trust_input_diagnostic` must show
`m7_loader_policy_input_ready_for_provider_trust: false`,
`m7_loader_policy_evidence_present: false`,
`provider_trust_evidence_present: false`, `provider_trust_positive: false`,
`denied_m7_loader_policy_input_not_ready_for_provider_trust`, and
`authorizes_load` still false. The loader-runtime aggregate also records the
live-load sequence as read-only current-boot source evidence:
`raios.module_loader_load_attempt_boundary.v0`,
`raios.module_loader_artifact_load_boundary.v0`,
`raios.module_loader_executable_mapping_boundary.v0`,
`raios.module_loader_entrypoint_transfer_boundary.v0`, and
`raios.module_loader_service_start_boundary.v0`, followed by
`raios.module_loader_service_health_binding_boundary.v0`,
`raios.module_loader_service_running_state_boundary.v0`,
`raios.module_loader_service_start_audit_boundary.v0`, and
`raios.module_loader_service_unload_cleanup_boundary.v0`, followed by
`raios.module_loader_live_load_commit_boundary.v0`,
`raios.module_loader_commit_audit_boundary.v0`,
`raios.module_loader_commit_rollback_boundary.v0`, and
`raios.module_loader_commit_result_boundary.v0`, followed by
`raios.module_loader_descriptor_acceptance_authority_boundary.v0`,
`raios.module_loader_descriptor_parser_contract_boundary.v0`, and
`raios.module_loader_descriptor_parser_result_boundary.v0`, and
`raios.module_loader_descriptor_schema_validation_boundary.v0`, and
`raios.module_loader_descriptor_capability_validation_boundary.v0`, and
`raios.module_loader_descriptor_load_plan_boundary.v0`, and
`raios.module_loader_executable_load_plan_authority_boundary.v0`, and
`raios.module_loader_executable_load_plan_result_boundary.v0`, and
`raios.module_loader_executable_image_layout_boundary.v0`, and
`raios.module_loader_executable_page_mapping_plan_boundary.v0`, and
`raios.module_loader_executable_page_mapping_boundary.v0`, and
`raios.module_loader_descriptor_executable_page_binding_boundary.v0`, and
`raios.module_loader_executable_entrypoint_binding_boundary.v0`, and
`raios.module_loader_executable_entrypoint_transfer_authorization_boundary.v0`,
and `raios.module_loader_executable_entrypoint_transfer_boundary.v0`, and
`raios.module_loader_executable_entrypoint_handoff_boundary.v0`.
Those
boundaries consume the retained intake, execution, registry, service-slot,
health-hook, rollback-hook, audit/rollback, and loader-fact evidence chain only
as provenance, remain non-authorizing, and keep artifact loading, executable
mapping, entrypoint transfer, service start, health-record creation,
running-state marking, start-audit record writing, unload/cleanup, service
inventory mutation, service-slot allocation, live-load commit, load-commit
audit writing, commit rollback install, load-result recording, durable writes,
rollback installation, descriptor acceptance, parsed descriptor production,
validated descriptor production, descriptor schema validation, descriptor
capability validation, capability-validated descriptor production, executable
page-mapping plan production, executable page mapping, executable image-layout
production, executable load-plan authority, executable load-plan production,
descriptor load-plan production,
capability-validated descriptor executable binding, executable entrypoint
binding, entrypoint transfer authorization, explicit executable entrypoint
transfer, executable entrypoint handoff, descriptor parsing, descriptor-byte
intake, and load attempts false.

The module loader identity diagnostic emits `raios.module_loader_identity.v0`
and the selftest emits `raios.module_loader_identity_selftest.v0`. It makes the
first loader-runtime fact addressable as a local-only current-boot diagnostic,
but does not accept loader descriptors or artifact bytes. The live diagnostic
reports the identity fact as missing until retained module evidence,
the retained service-slot allocator source-evidence projection, and
audit/write-boundary bindings exist. With valid retained allocator source
evidence, the live diagnostic reports
`service_slot_allocator_authority_boundary_non_authorizing` through the
allocator-authority boundary rather than the older static runtime-missing
placeholder. It also records a separate
`raios.module_loader_identity_source_evidence.v0` binding in the current-boot
RAM event log; that binding is local-only, non-authorizing, accepts no loader
descriptor or artifact bytes, and is consumed by `module.loader_runtime` only
as observed source evidence. The selftest covers missing retained evidence,
missing allocator readiness/runtime, missing audit/write boundary, identity
scope/schema/provenance failures, missing
retained-evidence/service-slot/audit-boundary bindings, missing identity, and
all-inputs-present-but-non-authorizing identity evidence.

The module loader artifact-hash binding diagnostic emits
`raios.module_loader_artifact_hash_binding.v0` and the selftest emits
`raios.module_loader_artifact_hash_binding_selftest.v0`. It makes the second
loader-runtime fact addressable as local-only current-boot evidence and adds an
explicit loader-identity binding requirement. It must keep loader descriptor
input, artifact byte input, service inventory mutation, service-slot
allocation, and load attempts disabled. The live diagnostic also records
`raios.module_loader_artifact_hash_binding_source_evidence.v0` in the
current-boot RAM event log; that binding is local-only, non-authorizing,
accepts no loader descriptor or artifact bytes, cites the retained
loader-identity source-evidence event when present, and is consumed by
`module.loader_runtime` only as observed source evidence. The selftest covers
missing retained evidence, allocator readiness/runtime gaps, missing
audit/write boundary, missing loader identity, artifact-hash binding
scope/schema/provenance failures,
retained-evidence/service-slot/audit-boundary/loader-identity binding gaps,
missing artifact-hash binding, and all-inputs-present-but-non-authorizing
artifact-hash binding evidence.

The module loader entrypoint-ABI diagnostic emits
`raios.module_loader_entrypoint_abi.v0` and the selftest emits
`raios.module_loader_entrypoint_abi_selftest.v0`. It makes the third
loader-runtime fact addressable as local-only current-boot evidence and adds an
explicit artifact-hash binding dependency. The live diagnostic records
`raios.module_loader_entrypoint_abi_source_evidence.v0` in the current-boot RAM
event log; that binding is local-only, non-authorizing, accepts no loader
descriptor or artifact bytes, cites the retained artifact-hash source-evidence
event when present, and is consumed by `module.loader_runtime` only as observed
source evidence. It must keep loader descriptor input, artifact byte input,
service inventory mutation, service-slot allocation, and load attempts
disabled.

The next seven module loader fact diagnostics after entrypoint ABI now emit both
read-only current-boot fact schemas and retained source-evidence records for
`raios.module_loader_address_space_boundary.v0`,
`raios.module_loader_memory_map_constraints.v0`,
`raios.module_loader_capability_import_table.v0`,
`raios.module_loader_service_slot_binding.v0`,
`raios.module_loader_health_state_hooks.v0`,
`raios.module_loader_rollback_hooks.v0`, and
`raios.module_loader_audit_rollback_write_boundary_binding.v0`, each with a
matching `_selftest` schema. Their source-evidence records are local-only,
non-authorizing, cite the previous retained loader-fact source-evidence event
id when present, and are consumed by `module.loader_runtime` only as observed
evidence. The loader fact diagnostics are chained facts: each diagnostic requires
retained module evidence, service-slot allocator readiness/runtime,
audit/write-boundary availability, and the previous loader fact before its own
fact can become available. They must keep loader descriptor input, artifact byte
input, service inventory mutation, service-slot allocation, and load attempts
disabled. The selftests cover missing prerequisites, previous-boot,
schema/provenance failures, required binding gaps, missing fact, and
all-inputs-present-but-non-authorizing fact evidence.

The audit/rollback availability diagnostic emits
`raios.module_audit_rollback_availability.v0` and the selftest emits
`raios.module_audit_rollback_availability_selftest.v0`. It reports typed
`raios.durable_audit_ledger.v0` and `raios.rollback_store.v0` current-boot
availability facts. In the current kernel both facts must be `missing`,
`local_only`, non-durable, and non-authorizing; `writes_enabled`,
`creates_durable_audit_records`, `creates_rollback_plans`,
`installs_rollback_plan`, `can_load_now`, and `load_attempted` must remain
false.

The audit/rollback write-policy diagnostic emits
`raios.module_audit_rollback_write_policy.v0` and the selftest emits
`raios.module_audit_rollback_write_policy_selftest.v0`. It reports typed
`raios.durable_audit_write_policy.v0` and `raios.rollback_install_policy.v0`
current-boot policy facts. In the current kernel both facts must be `missing`,
`local_only`, non-durable, and non-authorizing; they must name retained module
evidence and availability facts as required bindings, while `writes_enabled`,
`creates_durable_audit_records`, `creates_rollback_plans`,
`installs_rollback_plan`, `can_load_now`, and `load_attempted` remain false.

The audit/rollback storage-layout diagnostic emits
`raios.module_audit_rollback_storage_layout.v0` and the selftest emits
`raios.module_audit_rollback_storage_layout_selftest.v0`. It reports typed
`raios.persistence_device_inventory.v0` and
`raios.audit_rollback_storage_layout.v0` current-boot facts. In the current
kernel the persistence device fact observes QEMU AHCI and emits
`raios.ahci_controller_probe.v0` ABAR/port plus read-only IDENTIFY DEVICE and
Sector-0 evidence plus empty MBR partition inventory for the QEMU HARDDISK, but
only read-only block-driver readiness is available for audit/rollback. The VM
smoke harness also attaches a temporary scratch disk on a separate AHCI/IDE
port; when LBA0 carries `RAIOS_SCRATCH_V0`, the kernel emits
`raios.scratch_block_region_write_readback.v0`, writes and reads back only LBA1
of that labeled scratch region, and reports `scratch_write_readback_verified`.
The scratch evidence also reports device identity, `region_start_lba: 1`,
`region_lba_count: 1`, `region_within_device_bounds: true`,
`boot_port_overlap: false`, `metadata_lba_overlap: false`,
`no_boot_or_partition_metadata_overlap: true`, and
`block_write_authority_available: true`. The companion
`raios.scratch_block_write_authority.v0` object authorizes only the
current-boot scratch write/readback proof. That scratch evidence is test
infrastructure only: it must retain `authorizes_audit_rollback: false`,
`authorizes_append: false`, and `writes_enabled: false`. The diagnostic also emits
`raios.audit_rollback_target_region_discovery.v0` from a separate
VM-harness-labeled `RAIOS_AUDITRB_V0` disk. The kernel must read that label and
the LBA1 target region only, report `candidate_region_present: true`,
`candidate_region_is_scratch: false`,
`candidate_overlaps_boot_metadata: false`,
`candidate_overlaps_scratch: false`, and `durable_region_available: true`,
while keeping `authorizes_append: false`, durable audit writes,
rollback-store writes, and `write_attempted: false`. The diagnostic also emits
`raios.block_write_path_authority_gate.v0` with
`block_write_path.authority.audit_rollback.current_boot`, the
`block_driver.ahci.read_only.current_boot` source driver, and `mbr_empty`
partition-inventory evidence; it must remain `missing`, `available: false`,
`authorizes_media_write: false`, `authorizes_append: false`, `writes_enabled:
false`, and `write_attempted: false`. Write-path availability, layout regions,
append slots, and recovery separation are still missing, `local_only`,
non-durable, and non-authorizing; none of those facts may be treated as write
or append authority.

The transaction-writer readiness diagnostic also includes a nested
`target_region_writer_contract` object with schema
`raios.audit_rollback_target_region_writer_contract.v0`, id
`target_region_writer_contract.audit_rollback.current_boot`, status
`target_region_ready_not_write_authority`, reason
`target_region_read_only_missing_media_write_authority`, source discovery
`target_region.audit_rollback.current_boot`, the non-scratch LBA1/512-byte
target span, `append.audit_ledger.current_boot` / `raios.audit_record.v0`,
`append.rollback_store.current_boot` / `raios.rollback_transaction.v0`,
`target_range_ready: true`, and all write/append flags false.
Nested under that contract, `media_write_policy_preflight` must use schema
`raios.audit_rollback_target_region_media_write_policy_preflight.v0`, id
`target_region_media_write_policy_preflight.audit_rollback.current_boot`,
status `denied_missing_media_write_authority_and_durable_audit_policy`, reason
`target_region_contract_ready_policy_or_write_authority_missing`, source
contract schema/id/status/reason fields, owner/target/span/schema verification
booleans, missing media-write authority and durable-audit-policy reasons, and
all media-write/append/write-attempt flags false.
`service.rollback_apply svc.demo.hello` must also repeat that preflight under
`durable_append_authority_preflight.target_region_media_write_policy_preflight`
with a `preflight_hash`, then expose
`durable_append_authority_preflight.media_write_authority_gate` using schema
`raios.ram_only_hello_service_rollback_media_write_authority_gate.v0`, id
`hello_rollback_media_write_authority_gate.current_boot.svc.demo.hello.v0`,
status `denied_missing_durable_audit_policy`, reason
`target_region_test_media_write_verified_durable_audit_policy_missing`, the
durable append preflight hash, the durable append preflight's own
target-region write/readback hash binding, the policy preflight hash, the
target-region write/readback dry-run hash, source-contract/target-span facts,
`test_infrastructure_media_write_authority_available: true`, missing
durable-audit-policy reason, and all durable media-write/append flags false
while recording the target-region write/readback attempt. `agent audit.events
58` must expose the matching RAM audit binding fields before the durable append
authority denial.

The same rollback response and RAM audit binding should now also expose
`raios.ram_only_hello_service_rollback_target_region_write_readback_dry_run.v0`
as current-boot/local-only/test-infrastructure evidence. It must bind the
append-sector plan hash, policy-preflight hash, planned sector-image hash,
readback sector-image hash,
target region id `target_region.audit_rollback.current_boot`, LBA1/512 bytes,
label-found and
target-range-ready booleans, `write_completed: true`,
`readback_completed: true`, and `readback_matches_planned_image: true`, while
keeping `authorizes_media_write`, `authorizes_append`,
`writes_durable_audit_log`, `writes_rollback_store`,
`appends_rollback_transaction`, and `installs_rollback_state` false.

The audit/rollback append-engine readiness diagnostic emits
`raios.module_audit_rollback_append_engine.v0` and the selftest emits
`raios.module_audit_rollback_append_engine_selftest.v0`. It reports typed
`raios.audit_ledger_append_engine.v0` and
`raios.rollback_store_transaction_engine.v0` current-boot facts. In the current
kernel both facts are `missing`, `local_only`, non-durable, and
non-authorizing; append-only behavior, flush support, replay support,
storage-layout binding, write-policy binding, and recovery separation must not
be treated as write or append authority.

The Hello rollback append-engine readiness decision emits
`raios.ram_only_hello_service_rollback_append_engine_readiness_decision.v0`
under `service.rollback_apply svc.demo.hello`. It consumes the durable
append/transaction authorization gate plus the writer-policy, append-record,
sector-plan, target-region write/readback, audit-ledger, rollback-store,
target-span, and test-media evidence. In the current kernel it reports
`status: available`, reason `transaction_append_engine_ready`,
`append_engine_available: true`, `durable_audit_writer_available: true`,
`rollback_store_writer_available: true`,
`transaction_append_writer_available: true`, `ready: true`, and all
authorize/write/append flags false; durable append authority remains denied and
the same fields are retained on the rollback-apply RAM audit event.

The Hello rollback durable append-authority decision emits
`raios.ram_only_hello_service_rollback_durable_append_authority_decision.v0`
under the same rollback-apply response and RAM audit binding. It binds the
durable append preflight, durable writer policy, append-engine readiness
decision, media-write authority gate, target-region media-write policy
preflight, target-region write/readback hash, and LBA1/512-byte target span. In
the current kernel writer policy, append engine, media-write gate, and
test-media authority are ready, but durable audit policy and durable append
authority are false and all authorize/write/append flags remain false.

The Hello rollback durable audit-policy decision emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_decision.v0` under
that same durable append-authority preflight and RAM audit binding. It binds the
durable append-authority decision hash, target-region media-write policy
preflight hash, media-write authority gate hash, target-region write/readback
hash, and LBA1/512-byte target span. In the current kernel the media-write
policy evidence is verified as current-boot test infrastructure, but durable
audit policy remains unavailable and all write/append flags remain false.

The Hello rollback durable audit-policy candidate emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_candidate.v0` under
that same durable append-authority preflight and RAM audit binding. It binds the
durable audit-policy decision hash, canonical audit-record image hash,
target-region media-write policy preflight hash, target-region write/readback
hash, audit-record schema, and LBA1/512-byte target span. In the current kernel
the candidate is available as current-boot evidence only; durable audit policy,
durable append authority, media writes, durable audit writes, rollback-store
writes, transaction append, rollback application, and installed rollback state
remain denied.

The Hello rollback durable audit-policy acceptance gate emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_acceptance_gate.v0`
under that same durable append-authority preflight and RAM audit binding. It
consumes the durable audit-policy candidate and binds candidate/decision/audit
record image/media policy/target-region write-readback hashes,
audit-record schema, and LBA1/512-byte span. In the current kernel candidate
and media policy evidence are verified, but durable policy ledger and write
authority remain unavailable, so durable audit policy, durable append
authority, media writes, durable audit writes, rollback-store writes,
transaction append, rollback application, and installed rollback state remain
denied.

The Hello rollback durable audit-policy ledger candidate emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_candidate.v0`
under that same durable append-authority preflight and RAM audit binding. It
binds the acceptance-gate hash, durable audit-policy candidate hash, durable
audit-policy decision hash, canonical audit-record image hash, target-region
media-write policy preflight hash, target-region write/readback hash,
audit-record schema, and LBA1/512-byte span. In the current kernel the ledger
candidate is current-boot/local-only/read-only evidence only; write authority,
real durable policy ledger, durable audit policy, durable append authority,
media writes, durable audit writes, rollback-store writes, transaction append,
rollback application, and installed rollback state remain denied.

The Hello rollback durable audit-policy ledger-aware acceptance result emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_aware_acceptance_result.v0`
under that same durable append-authority preflight and RAM audit binding. It
consumes the read-only ledger candidate and binds ledger-candidate,
acceptance-gate, durable audit-policy candidate, durable audit-policy decision,
audit-record image, media policy, and target-region write/readback hashes plus
the same audit-record schema and LBA1/512-byte span. In the current kernel
ledger evidence is verified, but write authority, real durable policy ledger,
durable audit policy, durable append authority, media writes, durable audit
writes, rollback-store writes, transaction append, rollback application, and
installed rollback state remain denied.

The current Hello rollback durable policy-ledger availability fact emits
`raios.ram_only_hello_service_rollback_durable_policy_ledger_availability.v0`
under the same response and RAM audit binding. It binds the write-authority
availability hash plus the ledger-aware result, ledger-candidate,
target-region media-write policy, target-region write/readback, audit/rollback
target ids and schemas, and LBA1/512-byte span. The expected current state is
verified write-authority/ledger/media/target evidence with
`durable_policy_ledger_available: false`; no durable write or append authority
is opened.

The current Hello rollback durable policy-ledger availability dry-run emits
`raios.ram_only_hello_service_rollback_durable_policy_ledger_availability_dry_run.v0`
under the same response and RAM audit binding. It binds the policy-ledger
availability hash, write-authority availability hash, ledger-aware result,
ledger-candidate, media policy, target-region write/readback, transaction-append
authority-denial gate, transaction append-availability decision, audit/rollback
target ids and schemas, and LBA1/512-byte span as current-boot test-media-only
evidence. The expected state verifies the transaction denial gate and target
span while keeping durable policy ledger, durable audit policy, durable append
authority, transaction append, writes, append, rollback application, and
installed rollback state unavailable/false.

The current Hello rollback durable audit-policy availability fact emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_availability.v0`
under the same response and RAM audit binding. It consumes that policy-ledger
availability evidence and binds the policy-ledger availability hash,
write-authority availability hash, ledger-aware result, ledger-candidate,
target-region media-write policy, target-region write/readback, audit/rollback
target ids and schemas, and LBA1/512-byte span. The expected current state is
verified policy-ledger/write-authority/ledger/media/target evidence with
`durable_audit_policy_available: false`; no durable write or append authority
is opened.

The current Hello rollback durable audit-policy availability dry-run emits
`raios.ram_only_hello_service_rollback_durable_audit_policy_availability_dry_run.v0`
under the same response and RAM audit binding. It binds the audit-policy
availability hash, policy-ledger availability dry-run hash, policy-ledger
availability hash, write-authority availability hash, ledger-aware result,
ledger-candidate, media policy, target-region write/readback, transaction-append
authority-denial gate, transaction append-availability decision, audit/rollback
target ids and schemas, and LBA1/512-byte span as current-boot test-media-only
evidence. The expected state verifies the transaction denial gate and target
span while keeping durable audit policy, durable append authority, transaction
append, writes, append, rollback application, and installed rollback state
unavailable/false.

The current Hello rollback durable append-authority availability dry-run emits
`raios.ram_only_hello_service_rollback_durable_append_authority_availability_dry_run.v0`
under the same response and RAM audit binding. It binds the append-authority
availability hash, audit-policy availability dry-run hash, audit-policy
availability hash, policy-ledger availability dry-run hash, policy-ledger
availability hash, write-authority availability hash, ledger-aware result,
ledger-candidate, media policy, target-region write/readback,
transaction-append authority-denial gate, transaction append-availability
decision, audit/rollback target ids and schemas, and LBA1/512-byte span as
current-boot test-media-only evidence. The expected state verifies the
transaction denial gate and target span while keeping durable append authority,
durable audit policy, transaction append, writes, append, rollback application,
and installed rollback state unavailable/false.

The durable policy write-authority decision now consumes that dry-run hash in
`raios.ram_only_hello_service_rollback_durable_policy_write_authority_decision.v0`
alongside the transaction-append dry-run, target-sector inspection,
write-authority availability, audit-policy availability, append-authority
availability, transaction-denial gate, transaction append-availability decision,
audit/rollback target ids and schemas, and the LBA1/512-byte span while keeping
all durable write, append, transaction-append, and rollback-apply flags false.

The top-level
`raios.ram_only_hello_service_rollback_apply.v0` denial hash now consumes that
retained durable policy write-authority decision hash plus the retained
`raios.recovery_rollback_inspect_source_reference.v0` hash. The rollback apply
response and RAM audit binding expose
`source_durable_policy_write_authority_decision_hash`,
`source_recovery_rollback_inspect_source_reference_hash`,
`retained_durable_policy_write_authority_decision_verified`, and
`retained_recovery_rollback_inspect_source_reference_validated`; these fields
must be present while durable media writes, durable audit writes,
rollback-store writes, transaction append, and rollback application remain
false/denied.
The read-only `recovery.rollback_apply_authorization_diagnostic` reference
tuple now carries the same sourced rollback-apply denial hash, durable policy
write-authority decision hash, and recovery inspect-source reference hash. A
valid reference may be retained as current-boot/local-only RAM audit evidence,
but it must still report no recovery command dispatch, no rollback apply, no
durable writes, and no load attempt.
The read-only `recovery.disable_module_target_binding_diagnostic` reference
tuple now carries those same three source hashes from the retained apply
authorization event. A valid reference may be retained, but it must still report
no module disable, no recovery command dispatch, no durable writes, and no load
attempt.

The current Hello rollback durable append-authority availability fact emits
`raios.ram_only_hello_service_rollback_durable_append_authority_availability.v0`
under that same response and RAM audit binding. It consumes the durable
audit-policy availability evidence and binds the audit-policy availability hash,
policy-ledger availability hash, write-authority availability hash,
ledger-aware result, ledger-candidate, target-region media-write policy,
target-region write/readback, audit/rollback target ids and schemas, and
LBA1/512-byte span. The expected current state is verified
audit-policy/policy-ledger/write-authority/ledger/media/target evidence with
`durable_append_authority_available: false`; no durable write or append
authority is opened.

The current Hello rollback transaction-append availability decision emits
`raios.ram_only_hello_service_rollback_transaction_append_availability_decision.v0`
under that same response and RAM audit binding. It consumes the durable
append-authority availability evidence and binds the audit-policy availability
hash, append-engine readiness hash, durable writer-policy preflight hash,
target-region media-write policy, target-region write/readback,
audit/rollback target ids and schemas, and LBA1/512-byte span. The expected
current state is verified append-authority availability, audit-policy
availability, append-engine, writer-policy, media, target, and test-media
evidence with `transaction_append_available: false`; no durable write, append,
or transaction-append authority is opened.

The current Hello rollback transaction-append authority-denial gate emits
`raios.ram_only_hello_service_rollback_transaction_append_authority_denial_gate.v0`
under that same response and RAM audit binding. It consumes the
transaction-append availability decision and binds the durable append-authority
availability hash, audit-policy availability hash, append-engine readiness
hash, durable writer-policy preflight hash, target-region media-write policy,
target-region write/readback, audit/rollback target ids and schemas, and
LBA1/512-byte span. The expected current state is verified availability
decision, append-engine, writer-policy, media, target, and test-media evidence
with `missing_transaction_append_authority: true`; no media-write, append,
transaction-append, durable-audit, rollback-store, or write-attempt authority is
opened. The next narrow debugging target is the test-media-only rollback
transaction-append dry-run blocked by this gate.

The audit/rollback append-contract diagnostic emits
`raios.module_audit_rollback_append_contract.v0` and the selftest emits
`raios.module_audit_rollback_append_contract_selftest.v0`. It reports typed
`raios.audit_ledger_append_envelope.v0` and
`raios.rollback_store_transaction_envelope.v0` current-boot facts. In the
current kernel both facts are `missing`, `local_only`, non-durable, and
non-authorizing; the diagnostic consumes the storage-layout and append-engine
facts, names required storage-layout, append-engine, write-policy,
availability, and provenance bindings for future append envelopes, and
retains the same fail-closed block write-path authority gate from the
storage-layout diagnostic. `append_engine_missing` must remain true while
writes and rollback installs remain disabled.

The audit/rollback append payload-hash diagnostic emits
`raios.module_audit_rollback_append_payload_hash.v0` and the selftest emits
`raios.module_audit_rollback_append_payload_hash_selftest.v0`. It reports typed
`raios.audit_record_append_payload_hash_envelope.v0` and
`raios.rollback_transaction_append_payload_hash_envelope.v0` current-boot
facts. In the current kernel the envelopes are derived only from retained
audit/rollback candidates, retained service-slot reservation evidence, the
pre-load write-request shape, and bound append-contract ids; they are still
`missing` until append-contract facts exist and must remain `local_only`,
non-durable, and non-authorizing.

The audit/rollback append-intent diagnostic emits
`raios.module_audit_rollback_append_intent.v0` and the selftest emits
`raios.module_audit_rollback_append_intent_selftest.v0`. It reports typed
`raios.audit_record_append_intent.v0` and
`raios.rollback_transaction_append_intent.v0` current-boot facts. In the current
kernel both facts are `missing`, `local_only`, non-durable, and
non-authorizing; the diagnostic consumes the bound append-contract facts and
append payload-hash envelope readiness, and names required append-contract,
append-engine, storage-layout, write-policy, availability, payload-hash, and
provenance bindings for future append requests.
`append_intent_missing` must remain true while writes and rollback installs
remain disabled.

The write-boundary diagnostic emits
`raios.module_audit_rollback_write_boundary.v0` and the selftest emits
`raios.module_audit_rollback_write_boundary_selftest.v0`. It consumes only the
retained current-boot module evidence chain plus the retained service-slot
reservation plus the audit/rollback availability, write-policy, storage-layout,
append-engine readiness through the append contract, append-contract facts, and
append payload-hash envelopes, and append-intent facts, emits
`raios.module_pre_load_audit_rollback_write_request.v0` and
`raios.module_audit_rollback_write_denial_evidence.v0`, and keeps
`writes_enabled: false`, `creates_durable_audit_records: false`,
`creates_rollback_plans: false`, `installs_rollback_plan: false`,
`loads_artifact: false`, and `loads_recovery_artifact: false`. The current live
kernel must still report
`durable_audit_write_missing`, `rollback_install_missing`,
`storage_layout_missing`, `append_engine_missing`, and
`append_intent_missing`; append payload-hash envelopes must not be treated as
durable audit or rollback-store authority.

The recovery artifact load boundary is a separate denied path:

```text
recovery.load_artifact
module.load_recovery_artifact
```

It emits `raios.recovery_artifact_load_boundary.v0` with a
`raios.recovery_artifact_load_denial_evidence.v0` binding, uses
`cap.recovery.load_artifact` rather than `cap.module.load_ephemeral`, reports
missing `raios.recovery_artifact_identity.v0`,
`raios.recovery_artifact_trust.v0`, `raios.recovery_artifact_vm_test.v0`,
`raios.recovery_artifact_local_approval.v0`,
`raios.recovery_artifact_loader.v0`, and
`raios.recovery_artifact_rollback_evidence.v0`, and keeps
`loads_recovery_artifact: false`, `loads_normal_module: false`,
`normal_module_load_path_used: false`, `service_inventory_change: none`, and
`load_attempted: false`.

The read-only binding diagnostic is:

```text
recovery.identity_diagnostic
recovery.identity_diagnostic <identity_reference_hash> <artifact_hash> [current_boot]
recovery.identity_diagnostic_selftest
recovery.trust_diagnostic
recovery.trust_diagnostic <trust_reference_hash> <retained_identity_event_id> <identity_reference_hash> <artifact_hash> <trust_hash> [current_boot]
recovery.trust_diagnostic_selftest
recovery.vm_test_diagnostic
recovery.vm_test_diagnostic <vm_test_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <identity_reference_hash> <trust_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> [current_boot]
recovery.vm_test_diagnostic_selftest
recovery.local_approval_diagnostic
recovery.local_approval_diagnostic <local_approval_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> [current_boot]
recovery.local_approval_diagnostic_selftest
recovery.loader_diagnostic
recovery.loader_diagnostic <loader_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <retained_local_approval_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <local_approval_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> <loader_hash> [current_boot]
recovery.loader_diagnostic_selftest
recovery.rollback_evidence_diagnostic
recovery.rollback_evidence_diagnostic <rollback_evidence_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <retained_local_approval_event_id> <retained_loader_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <local_approval_reference_hash> <loader_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> <loader_hash> <rollback_evidence_hash> [current_boot]
recovery.rollback_evidence_diagnostic_selftest
recovery.lifeline_request_diagnostic
recovery.lifeline_request_diagnostic <lifeline_request_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <retained_local_approval_event_id> <retained_loader_event_id> <retained_rollback_evidence_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <local_approval_reference_hash> <loader_reference_hash> <rollback_evidence_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> <loader_hash> <rollback_evidence_hash> [current_boot]
recovery.lifeline_request_diagnostic_selftest
recovery.lifeline_protocol_diagnostic
recovery.lifeline_protocol_diagnostic_selftest
recovery.lifeline_command_vocabulary
recovery.lifeline_command_vocabulary_selftest
recovery.loader_runtime_isolation
recovery.loader_runtime_isolation_selftest
recovery.rollback_transaction_engine
recovery.rollback_transaction_engine_selftest
recovery.durable_audit_rollback_persistence
recovery.durable_audit_rollback_persistence_selftest
recovery.memory_provenance
recovery.memory_provenance_selftest
recovery.load_binding
recovery.load_binding_selftest
```

The identity/trust/VM-test/local-approval/loader/rollback-evidence/lifeline-request diagnostics emit
`raios.recovery_artifact_identity_diagnostic.v0`,
`raios.recovery_artifact_trust_diagnostic.v0`,
`raios.recovery_artifact_vm_test_diagnostic.v0`,
`raios.recovery_artifact_local_approval_diagnostic.v0`,
`raios.recovery_artifact_loader_diagnostic.v0`, and
`raios.recovery_artifact_rollback_evidence_diagnostic.v0`, plus
`raios.recovery_lifeline_request_diagnostic.v0`. Valid references are retained
only as local-only current-boot hash evidence, accept no artifact bytes,
VM-test JSON, approval text, loader descriptors, rollback evidence JSON, or
lifeline request JSON, and do not authorize recovery loading.

It emits `raios.recovery_artifact_load_binding.v0` and
`raios.recovery_artifact_load_binding_selftest.v0`, requires retained
`recovery_artifact_identity_event_id`, `recovery_artifact_trust_event_id`,
`recovery_vm_test_event_id`, `recovery_local_approval_event_id`,
`recovery_loader_event_id`, and `recovery_rollback_evidence_event_id`, and
binds retained identity, trust, VM-test, local-approval, loader, and
rollback-evidence ids when their current-boot hash-reference chain matches. It
keeps normal module append-intent,
append-payload, writer, service-slot, and `module.load_ephemeral` facts
non-authorizing for recovery loads. Payload-hash envelopes remain non-authority
inputs only, and a fully retained chain still stops at
`recovery_lifeline_protocol_missing`.

The lifeline-request diagnostic emits
`raios.recovery_lifeline_request_diagnostic.v0` and
`raios.recovery_lifeline_request_diagnostic_selftest.v0`. It consumes the six
retained recovery evidence event ids and their hashes, rejects missing, stale,
previous-boot, wrong-schema, substituted, and mismatched chains, records a valid
request only as local-only current-boot hash evidence, and keeps
`loads_recovery_loader`, `loads_recovery_artifact`, `creates_durable_records`,
`installs_rollback_plan`, `allocates_service_slot`, and `load_attempted` false.

The lifeline-protocol diagnostic emits
`raios.recovery_lifeline_protocol_state.v0` and
`raios.recovery_lifeline_protocol_state_selftest.v0`. It consumes the retained
lifeline request event id plus the six recovery evidence event ids bound by
that request, rejects missing, stale, previous-boot, wrong-schema,
substituted, and mismatched lifeline request/evidence chains before reporting
protocol gaps, and exposes typed local-only missing facts for
`raios.recovery_lifeline_protocol_state.v0`,
`raios.recovery_lifeline_command_vocabulary.v0`,
`raios.recovery_loader_runtime_isolation.v0`,
`raios.recovery_rollback_transaction_engine.v0`,
`raios.durable_audit_rollback_persistence.v0`, and
`raios.recovery_memory_provenance.v0`. It never accepts a direct OpenAI
provider path as the recovery lifeline, and keeps recovery loader execution,
artifact loading, durable writes, rollback installs, service-slot allocation,
and lifeline behavior disabled.

The lifeline command-vocabulary diagnostic emits
`raios.recovery_lifeline_command_vocabulary.v0` and
`raios.recovery_lifeline_command_vocabulary_selftest.v0`. It reuses the
retained lifeline request and recovery evidence chain, rejects missing, stale,
previous-boot, wrong-schema, substituted, and mismatched request/protocol-state
inputs before exposing command readiness, and defines command ids such as
`recovery.lifeline.status`, `recovery.lifeline.rollback_preview`,
`recovery.lifeline.rollback_apply`, `recovery.lifeline.disable_module`,
`recovery.lifeline.restart_last_good`, and
`recovery.lifeline.load_artifact_by_hash`. It reports each argument-envelope
schema and required capability, but keeps `accepts_lifeline_command_envelope`,
`command_execution_enabled`, loader execution, artifact loading, durable
writes, rollback installs, service-slot allocation, and service inventory
changes disabled.

The loader runtime-isolation diagnostic emits
`raios.recovery_loader_runtime_isolation.v0` and
`raios.recovery_loader_runtime_isolation_selftest.v0`. It reuses the retained
lifeline request/evidence chain and command-vocabulary envelope, rejects
missing, stale, previous-boot, wrong-schema, substituted, and mismatched
request/protocol-state/command-vocabulary inputs before loader readiness, and
reports missing local-only facts for loader address-space boundary, entrypoint
ABI, memory-map constraints, capability import table, artifact hash binding,
provider separation, normal-module separation, rollback transaction engine,
durable audit/rollback persistence, and recovery memory provenance. It accepts
no loader descriptor, artifact bytes, or lifeline command envelope, and keeps
loader execution, command dispatch, artifact loading, durable writes, rollback
installs, service-slot allocation, and service inventory changes disabled.

The rollback transaction-engine diagnostic emits
`raios.recovery_rollback_transaction_engine.v0` and
`raios.recovery_rollback_transaction_engine_selftest.v0`. It reuses the
retained lifeline request/evidence chain, command-vocabulary envelope, and
loader runtime-isolation boundary, rejects missing, stale, previous-boot,
wrong-schema, substituted, and mismatched request/protocol-state/
command-vocabulary/loader-isolation inputs before rollback readiness, and
reports missing local-only facts for rollback target selection, transaction
id/provenance, last-good binding, disabled-module set binding, artifact hash
binding, replay preconditions, recovery-only capability import, atomic
apply/abort semantics, durable audit/rollback persistence, and recovery memory
provenance. It accepts no rollback transaction envelope, rollback plan JSON,
lifeline command envelope, loader descriptor, artifact bytes, or direct OpenAI
recovery shortcut, and keeps rollback preview/apply, loader execution, artifact
loading, durable writes, rollback installs, service-slot allocation, and service
inventory changes disabled.

The durable audit/rollback persistence diagnostic emits
`raios.durable_audit_rollback_persistence.v0` and
`raios.durable_audit_rollback_persistence_selftest.v0`. It consumes the
retained lifeline request/evidence chain, command-vocabulary envelope, loader
runtime-isolation boundary, and rollback transaction-engine boundary, rejects
missing, stale, previous-boot, wrong-schema, substituted, and mismatched
request/protocol-state/command-vocabulary/loader-isolation/rollback-engine
inputs before persistence readiness, and reports missing local-only facts for
persistence-device inventory, durable storage-layout identity, audit append-log
identity, rollback-store identity, transaction replay cursor, last-good
checkpoint binding, write ordering, crash consistency, integrity root/hash
chain, and recovery-memory provenance. It accepts no persistence device JSON,
storage layout JSON, recovery memory record, rollback transaction envelope,
lifeline command envelope, loader descriptor, artifact bytes, or direct OpenAI
recovery shortcut, and keeps durable writes, rollback replay, recovery-memory
writes, rollback preview/apply, loader execution, artifact loading, rollback
installs, service-slot allocation, and service inventory changes disabled.

The recovery memory-provenance diagnostic emits
`raios.recovery_memory_provenance.v0` and
`raios.recovery_memory_provenance_selftest.v0`. It consumes the retained
lifeline request/evidence chain, command-vocabulary envelope, loader
runtime-isolation boundary, rollback transaction-engine boundary, and durable
audit/rollback persistence boundary, rejects missing, stale, previous-boot,
wrong-schema, substituted, and mismatched request/protocol-state/
command-vocabulary/loader-isolation/rollback-engine/persistence inputs before
memory readiness, and reports missing local-only facts for source record ids,
source schema hashes, classification, authority level, rollback-transaction
binding, last-good checkpoint binding, recovery-only export profile, redaction
state, replay window, and audit linkage. It accepts no memory record JSON,
exports no provider context, writes no recovery memory, and keeps rollback
preview/apply, loader execution, artifact loading, durable writes, rollback
replay, service-slot allocation, and lifeline command dispatch disabled.

A valid `module.manifest_diagnostic` hash-reference command records a local-only
current-boot `raios.module_manifest_reference.v0` event binding and reports
`retained_manifest_reference.status: retained_hash_reference_only`. This
retained reference stores only hashes and is not load authority:
`authorizes_guest_load`, `can_load_now`, and `load_attempted` must remain false.

A valid `module.artifact_diagnostic` hash-reference command records a local-only
current-boot `raios.module_candidate_artifact_reference.v0` event binding. It
binds retained manifest and computed-grant event ids plus manifest, artifact,
report, attestation, and grant hashes; it accepts no artifact bytes and still
keeps `artifact_loaded: false`, `can_load_now: false`, and
`load_attempted: false`.

A valid `module.vm_report_diagnostic` hash-reference command records a
local-only current-boot `raios.module_vm_test_report_reference.v0` event
binding. It binds retained manifest, candidate-artifact, and computed-grant
event ids plus manifest/reference/artifact/report/attestation hashes; it
accepts no VM-report JSON and still keeps `can_load_now: false`,
`service_inventory_change: none`, and `load_attempted: false`.

A valid `module.attestation_diagnostic` hash-reference command records a
local-only current-boot `raios.module_local_attestation_reference.v0` event
binding. It binds retained manifest, candidate-artifact, VM-report, and
computed-grant event ids plus reference/artifact/report/attestation hashes; it
accepts no local-attestation JSON and still keeps `can_load_now: false`,
`service_inventory_change: none`, and `load_attempted: false`.

A valid `module.approval_diagnostic` hash-reference command records a
local-only current-boot `raios.module_local_approval_reference.v0` event
binding. It binds retained manifest, candidate-artifact, VM-report,
local-attestation, and computed-grant event ids plus reference/artifact/report/
attestation/approval hashes; it accepts no local approval text and still keeps
`can_load_now: false`, `service_inventory_change: none`, and
`load_attempted: false`.

A valid `module.grant_diagnostic` full hash-reference command records a
local-only current-boot `raios.module_computed_grant_reference.v0` event binding
and the diagnostic response reports `retained_reference.status:
retained_hash_reference_load_still_denied`. This retained reference is still
non-authorizing: `grants_capability`, `grants_load_now`,
`authorizes_guest_load`, `can_load_now`, and `load_attempted` must remain
false.

After a valid reference is retained, `module.load_ephemeral` still denies but
should report `module_manifest: retained_hash_reference_only`,
`retained_module_manifest_reference.state: present`,
`retained_module_manifest_reference_not_authorizing`,
`candidate_artifact: retained_hash_reference_only`,
`retained_candidate_artifact_reference.state: present`,
`retained_candidate_artifact_reference_not_authorizing`,
`vm_test_report: retained_hash_reference_only`,
`retained_vm_test_report_reference.state: present`,
`retained_vm_test_report_reference_not_authorizing`,
`local_attestation: retained_hash_reference_only`,
`retained_local_attestation_reference.state: present`,
`retained_local_attestation_reference_not_authorizing`,
`local_approval: retained_hash_reference_only`,
`retained_local_approval_reference.state: present`,
`retained_local_approval_reference_not_authorizing`,
`computed_capability_grant: retained_hash_reference_only`,
`retained_computed_grant_reference.state: present`, retained hashes, and
`retained_computed_grant_reference_not_authorizing`. After a valid
audit/rollback reference is retained, the same denial should also report
`retained_audit_rollback_reference.state: present`,
`durable_audit_record: retained_hash_reference_only_not_durable`,
`rollback_plan: retained_hash_reference_only_not_installed`,
`durable_audit_write_missing`, and `rollback_install_missing`. Loader, service
slot, service
inventory change, and load attempt state must remain unavailable,
non-authorizing, `none`, and `false`. After a valid service-slot reservation is
retained, the denial should report
`retained_service_slot_reservation.state: present`,
`service_slot: retained_hash_reference_only_not_allocated`,
`retained_service_slot_reservation_not_allocated`, and
`service_slot_reservation_hash`, while still keeping
`allocates_service_slot: false`. With the full retained evidence chain present,
the same denial should now also report
`service_slot_allocator_readiness.schema:
raios.module_service_slot_allocator_readiness.v0`,
`service_slot_allocator: defined_non_authorizing`,
`service_slot_allocator_ready: false`,
`loader_runtime_readiness.schema:
raios.module_loader_runtime_readiness.v0`,
`loader_runtime: blocked_by_service_slot_allocator_authority`,
`readiness_status: denied_allocator_authority_not_granted`,
`readiness_reason: service_slot_allocator_authority_boundary_non_authorizing`,
`allocator_authority_boundary.schema:
raios.module_service_slot_allocator_authority.v0`, and
typed missing loader-runtime facts such as
`raios.module_loader_identity.v0`. It also reports the non-authorizing
live-load sequence through
`raios.module_loader_load_attempt_boundary.v0`,
`raios.module_loader_artifact_load_boundary.v0`,
`raios.module_loader_executable_mapping_boundary.v0`,
`raios.module_loader_entrypoint_transfer_boundary.v0`, and
`raios.module_loader_service_start_boundary.v0`,
`raios.module_loader_service_health_binding_boundary.v0`,
`raios.module_loader_service_running_state_boundary.v0`,
`raios.module_loader_service_start_audit_boundary.v0`, and
`raios.module_loader_service_unload_cleanup_boundary.v0`,
`raios.module_loader_live_load_commit_boundary.v0`,
`raios.module_loader_commit_audit_boundary.v0`,
`raios.module_loader_commit_rollback_boundary.v0`, and
`raios.module_loader_commit_result_boundary.v0`, and
`raios.module_loader_descriptor_acceptance_authority_boundary.v0`, and
`raios.module_loader_descriptor_parser_contract_boundary.v0`, and
`raios.module_loader_descriptor_parser_result_boundary.v0`, and
`raios.module_loader_descriptor_schema_validation_boundary.v0`, and
`raios.module_loader_descriptor_capability_validation_boundary.v0`, and
`raios.module_loader_descriptor_load_plan_boundary.v0`, and
`raios.module_loader_executable_load_plan_authority_boundary.v0`, and
`raios.module_loader_executable_load_plan_result_boundary.v0`, and
`raios.module_loader_executable_image_layout_boundary.v0`, and
`raios.module_loader_executable_page_mapping_plan_boundary.v0`, and
`raios.module_loader_executable_page_mapping_boundary.v0`, and
`raios.module_loader_descriptor_executable_page_binding_boundary.v0`, and
`raios.module_loader_executable_entrypoint_binding_boundary.v0`, and
`raios.module_loader_executable_entrypoint_transfer_authorization_boundary.v0`,
and `raios.module_loader_executable_entrypoint_transfer_boundary.v0`, and
`raios.module_loader_executable_entrypoint_handoff_boundary.v0`, while still
keeping `loads_artifact: false`, `creates_service_inventory_records: false`,
`service_inventory_change: none`, `starts_service: false`,
`creates_service_health_records: false`, `marks_service_running: false`,
`writes_service_start_audit_record: false`, `unloads_service: false`,
`cleans_up_service_slot: false`, `commits_live_load: false`,
`writes_load_commit_audit_record: false`,
`installs_commit_rollback_record: false`, `records_load_result: false`, and
`accepts_loader_descriptor: false`, `accepts_descriptor_bytes: false`,
`produces_parsed_descriptor: false`, `validates_descriptor_schema: false`,
`produces_validated_descriptor: false`,
`validates_descriptor_capabilities: false`,
`produces_capability_validated_descriptor: false`,
`authorizes_executable_load_plan: false`,
`produces_executable_load_plan: false`,
`produces_executable_image_layout: false`,
`produces_executable_page_mapping_plan: false`,
`maps_executable_pages: false`,
`binds_capability_validated_descriptor_to_executable_pages: false`,
`jumps_to_entrypoint: false`, `parses_descriptor_bytes: false`, and
`load_attempted: false`.

The live denied load gate revalidates a retained audit/rollback reference
before reporting those retained states. If the retained record points at a
wrong-schema event, stale/dropped event, substituted record, mismatched
canonical grant/audit/rollback hash, or invalid `ram_only:` service-slot id, the
gate reports `rejected_retained_reference`; the accepted audit/rollback evidence
hash fields stay `null`, and loading remains denied.

The live denied load gate also revalidates a retained manifest reference before
reporting it as manifest evidence. If the retained record is stale, wrong-schema,
substituted, hash-mismatched, or tied to a different retained computed-grant
manifest hash, the manifest gate reports `rejected_retained_reference`, accepted
manifest hash fields stay `null`, and loading remains denied.

The live denied load gate also revalidates a retained candidate-artifact
reference before reporting it as artifact evidence. If the retained record is
stale, wrong-schema, substituted, hash-mismatched, or no longer matches the
retained manifest/computed-grant references, the artifact gate reports
`rejected_retained_reference`, accepted artifact hash fields stay `null`, and
loading remains denied.

The live denied load gate also revalidates a retained VM-test-report reference
before reporting it as report evidence. If the retained record is stale,
wrong-schema, substituted, hash-mismatched, or no longer matches the retained
manifest, candidate-artifact, or computed-grant references, the VM-report gate
reports `rejected_retained_reference`, accepted VM-report hash fields stay
`null`, and loading remains denied.

The live denied load gate also revalidates a retained local-attestation
reference before reporting it as attestation evidence. If the retained record is
stale, wrong-schema, substituted, hash-mismatched, or no longer matches the
retained manifest, candidate-artifact, VM-report, or computed-grant references,
the local-attestation gate reports `rejected_retained_reference`, accepted
attestation hash fields stay `null`, and loading remains denied.

The live denied load gate also revalidates a retained local-approval reference
before reporting it as approval evidence. If the retained record is stale,
wrong-schema, substituted, hash-mismatched, or no longer matches the retained
manifest, candidate-artifact, VM-report, local-attestation, or computed-grant
references, the local-approval gate reports `rejected_retained_reference`,
accepted approval hash fields stay `null`, and loading remains denied.

The live denied load gate also revalidates a retained service-slot reservation
before reporting it as retained service-slot evidence. If the reservation points
at stale, wrong-schema, substituted, hash-mismatched, inventory-mismatched, or
slot-mismatched evidence, the service-slot gate reports
`rejected_retained_reference`, accepted `service_slot_reservation_hash` stays
`null`, and loading remains denied.

`module.load_gate_retained_selftest` emits
`raios.module_load_gate_retained_reference_selftest.v0`. It must keep
`mutates_global_event_log: false`, `creates_retained_reference_records: false`,
`loads_artifact: false`, `service_inventory_change: none`, and
`can_load: false` while covering missing, stale/dropped,
previous-boot-or-unretained, wrong-schema, substituted-record, and
hash-mismatch retained-reference cases.

`module.load_ephemeral` also reports
`raios.module_load_gate_audit_rollback_requirements.v0`, with
`raios.audit_record.v0` and `raios.rollback_plan.v0` still non-durable and
non-installed even when retained hash references exist; record writes remain
disabled. `module.load_gate_audit_rollback_selftest` emits
`raios.module_load_gate_audit_rollback_selftest.v0`; it must keep
`mutates_global_event_log: false`,
`creates_retained_audit_rollback_reference_records: false`,
`creates_durable_audit_records: false`, `creates_rollback_plans: false`,
`allocates_service_slot: false`, `loads_artifact: false`, and
`can_load: false`. It covers missing, stale, previous-boot, wrong-schema, and
substituted retained audit/rollback references; retained
computed-grant/audit/rollback hash mismatches; retained service-slot mismatch;
and the existing missing/mismatched durable audit plus rollback evidence cases.

`module.load_gate_service_slot_selftest` emits
`raios.module_load_gate_service_slot_selftest.v0`; it must keep
`mutates_global_event_log: false`,
`creates_service_slot_reservation_records: false`,
`allocates_service_slot: false`, `creates_service_inventory_records: false`,
`loads_artifact: false`, and `can_load: false`. It covers stale/dropped,
wrong-schema, substituted, computed-grant/audit/rollback hash mismatches,
inventory mismatch, slot mismatch, and reservation-hash mismatch for retained
service-slot reservations; rejected cases must keep
`accepted_service_slot_reservation_hash: false`.

`module.load_gate_loader_runtime_selftest` emits
`raios.module_load_gate_loader_runtime_selftest.v0`; it must keep
`mutates_global_event_log: false`, `accepts_loader_descriptor: false`,
`accepts_artifact_bytes: false`, `loads_artifact: false`,
`allocates_service_slot: false`, `creates_service_inventory_records: false`,
and `can_load: false`. It covers missing/rejected retained evidence,
missing/rejected retained service-slot reservation projection, and the
all-retained-evidence-ready state that remains denied by the non-authorizing
service-slot allocator authority boundary; all cases must keep load attempts
disabled.
It also emits the ten base loader-runtime source facts with
`source_fact_map_complete: true`. The denied load-gate
`loader_runtime_readiness` projection appends the receiver-identity load
preflight source fact as the eleventh entry when reporting real load-denial
readiness.

`module.load_gate_attestation_selftest` emits
`raios.module_load_gate_local_attestation_selftest.v0`; it must keep
`mutates_global_event_log: false`,
`creates_retained_local_attestation_reference_records: false`,
`accepts_local_attestation_json: false`, `accepts_artifact_bytes: false`,
`loads_artifact: false`, `service_inventory_change: none`, and
`can_load: false`. It covers stale/dropped, previous-boot-or-unretained,
wrong-schema, substituted, hash-mismatch, manifest-reference mismatch,
artifact-reference mismatch, VM-report-reference mismatch, and
computed-grant-reference mismatch for retained local-attestation references;
rejected cases must keep `accepted_local_attestation_hash: false`.

`module.load_gate_approval_selftest` emits
`raios.module_load_gate_local_approval_selftest.v0`; it must keep
`mutates_global_event_log: false`,
`creates_retained_local_approval_reference_records: false`,
`accepts_local_approval_text: false`, `accepts_artifact_bytes: false`,
`loads_artifact: false`, `service_inventory_change: none`, and
`can_load: false`. It covers stale/dropped, previous-boot-or-unretained,
wrong-schema, substituted, hash-mismatch, manifest-reference mismatch,
artifact-reference mismatch, VM-report-reference mismatch,
local-attestation-reference mismatch, and computed-grant-reference mismatch for
retained local-approval references; rejected cases must keep
`accepted_local_approval_hash: false`.

`module.load_gate_manifest_selftest` emits
`raios.module_load_gate_manifest_selftest.v0`; it must keep
`mutates_global_event_log: false`,
`creates_manifest_reference_records: false`, `accepts_manifest_json: false`,
`accepts_artifact_bytes: false`, `loads_artifact: false`, and `can_load: false`.
It covers missing, stale/dropped, previous-boot-or-unretained, wrong-schema,
substituted-record, and hash-mismatch retained manifest-reference candidates.

`module.load_gate_artifact_selftest` emits
`raios.module_load_gate_artifact_selftest.v0`; it must keep
`mutates_global_event_log: false`,
`creates_retained_candidate_artifact_reference_records: false`,
`accepts_artifact_bytes: false`, `loads_artifact: false`, and
`can_load: false`. It covers missing, stale/dropped,
previous-boot-or-unretained, wrong-schema, substituted-record, hash-mismatch,
manifest-reference mismatch, and computed-grant-reference mismatch retained
artifact-reference candidates.

`module.load_gate_vm_report_selftest` emits
`raios.module_load_gate_vm_report_selftest.v0`; it must keep
`mutates_global_event_log: false`,
`creates_retained_vm_test_report_reference_records: false`,
`accepts_vm_report_json: false`, `accepts_artifact_bytes: false`,
`loads_artifact: false`, and `can_load: false`. It covers missing,
stale/dropped, previous-boot-or-unretained, wrong-schema, substituted-record,
hash-mismatch, manifest-reference mismatch, artifact-reference mismatch,
computed-grant-reference mismatch, and VM-report-hash mismatch retained
VM-report-reference candidates.

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
