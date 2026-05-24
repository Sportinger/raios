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

Shadow VM smoke timing note: the full
`vm-harness\shadow-vm-smoke.ps1` run can be much longer than the compile step on
this Windows/QEMU setup. When running it through an agent tool, use a command
timeout of at least 30 minutes and pass `-TimeoutSeconds 180` if the default
45-second per-command serial timeout is too tight. Do not treat a 10-minute
outer tool timeout as a guest or protocol failure by itself; inspect the
generated `release\vm-reports\shadow-*.json` and the temp `serial.log`.
The entry script dispatches into focused `shadow-vm-smoke-profile-*.ps1`
profile slices, so profile-specific failures should be debugged in the matching
slice rather than in one monolithic harness file.

Stage-0 serial command-mode input echoes bytes to the serial log without
forcing framebuffer redraws during long pasted commands; this keeps long
hash-reference recovery diagnostics on the real serial path without paying a
full UI render for every input chunk. The 2026-05-24 focused recovery report
`release\vm-reports\shadow-20260524-140503-24772.json` passed 2725/2725
predicates with 142 executed commands in `duration_ms: 159960`.

For fast iteration, run the same real QEMU/serial path with the quick profile:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -Profile quick
```

`-Profile quick` covers boot readiness, core read-only agent methods,
provider-minimal context/export gates, denied module loading, denied recovery
artifact loading, and RAM-only audit visibility. It intentionally skips the
exhaustive module/recovery negative matrix; the default `-Profile full` remains
the pre-commit/release evidence path.

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
does not include provider-minimal context.

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
denial/positive records, request/body/binding/context hash mismatches, and
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
`raios.module_computed_grant_diagnostic.v0`, and
`raios.module_computed_grant_diagnostic_selftest.v0`. The manifest-reference
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
kernel both facts are `missing`, `local_only`, non-durable, and
non-authorizing; device identity, partition inventory, write-path availability,
layout regions, append slots, and recovery separation must not be treated as
write or append authority.

The audit/rollback append-engine readiness diagnostic emits
`raios.module_audit_rollback_append_engine.v0` and the selftest emits
`raios.module_audit_rollback_append_engine_selftest.v0`. It reports typed
`raios.audit_ledger_append_engine.v0` and
`raios.rollback_store_transaction_engine.v0` current-boot facts. In the current
kernel both facts are `missing`, `local_only`, non-durable, and
non-authorizing; append-only behavior, flush support, replay support,
storage-layout binding, write-policy binding, and recovery separation must not
be treated as write or append authority.

The audit/rollback append-contract diagnostic emits
`raios.module_audit_rollback_append_contract.v0` and the selftest emits
`raios.module_audit_rollback_append_contract_selftest.v0`. It reports typed
`raios.audit_ledger_append_envelope.v0` and
`raios.rollback_store_transaction_envelope.v0` current-boot facts. In the
current kernel both facts are `missing`, `local_only`, non-durable, and
non-authorizing; the diagnostic consumes the storage-layout and append-engine
facts, names required storage-layout, append-engine, write-policy,
availability, and provenance bindings for future append envelopes, and
`append_engine_missing` must remain true while writes and rollback installs
remain disabled.

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
`allocates_service_slot: false`.

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
