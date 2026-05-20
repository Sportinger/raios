# Project Status

Last verified locally: 2026-05-20 on Windows with QEMU 11 via headless Shadow
VM smoke covering deterministic `provider_minimal` packet/field-list evidence,
explicit provider request-binding denial and export-denial audit records, the
denied `provider.context_export` gate, the local redaction projection, read-only
memory context, the RAM-only current-boot event log with structured denial
bindings, the runtime `raios.provider_request_envelope.v0` marker on the real
OpenAI request path, positive local-only request/export audit binding records on
the SPKI pinned OpenAI path, checked current-boot binding consumption with
single-use rejection, a local-only negative gate selftest for stale/dropped,
previous-boot-or-unretained, substituted-schema, substituted-positive-record,
and mismatched-hash cases, the separate fail-closed
`raios.provider_context_injection_gate.v0` diagnostic, local-only negative
final-injection authorization selftests, the fail-closed
`raios.module_load_gate.v0` denial with event-log binding for denied
`module.load_ephemeral`, read-only `module.manifest_diagnostic`,
`module.manifest_diagnostic_selftest`, `module.artifact_diagnostic`,
`module.artifact_diagnostic_selftest`, `module.grant_diagnostic`, and
`module.grant_diagnostic_selftest` manifest, candidate-artifact, and
computed-grant hash-reference diagnostics, local-only current-boot retention of
valid manifest, artifact, and computed-grant hash references, the denied module
load gate reporting retained manifest, artifact, and computed-grant references
without authorizing loading, guest
audit/rollback hash-reference diagnostics that retain valid references only as
local-only current-boot evidence, the denied module load gate reporting retained
audit/rollback references as non-authorizing hash evidence only after live
current-boot predicate validation, rejection of a wrong-schema retained
audit/rollback reference in the live denied load gate, and local-only negative
manifest, artifact, retained-reference, plus audit/rollback evidence gate
selftests plus
`module.audit_rollback_diagnostic_selftest` guest hash-reference diagnostics,
and guest `module.service_slot_diagnostic` RAM-only service-slot reservation
hash-reference diagnostics that retain valid reservations as local-only
current-boot evidence without allocating a slot or loading artifacts, plus the
denied module load gate live-validating that retained reservation as
non-authorizing service-slot evidence and local-only service-slot gate
selftests for rejected retained reservations.
Direct OpenAI pin-mismatch plus SPKI pinned-trust smokes using a fake local API
key remain previously verified from the prior handoff.

Latest host-tool verification: 2026-05-20 on Windows with
`cargo test --locked -p ota-tools -p registry-core -p registry-tools -p fake-cloud-server`
covering OTA/registry tooling plus the non-authorizing
`raios.computed_capability_grant.v0` diagnostic, host-side
`raios.module_audit_rollback_diagnostic.v0` audit/rollback candidates, and
negative manifest/artifact/report/attestation/audit/rollback evidence cases.

Latest guest-protocol verification: 2026-05-20 on Windows with
`vm-harness\shadow-vm-smoke.ps1`, report
`release\vm-reports\shadow-20260520-182402-20552.json` with 807/807
predicates, covering absent/accepted/stale/mismatched/invalid module-manifest
hash-reference diagnostics, RAM-only retention of valid manifest and
candidate-artifact references, live denied load-gate visibility of retained
manifest and artifact hash evidence, negative manifest/artifact-reference gate
selftests, absent/accepted/stale/mismatched/wrong-policy module computed-grant
hash-reference diagnostics plus RAM-only retention of a valid computed-grant
hash reference and its visibility in the denied module load gate while live
loading remains denied, negative retained-reference gate selftests, negative
retained audit/rollback reference gate selftests,
missing/mismatched durable audit plus rollback evidence selftests, and guest
audit/rollback hash-reference diagnostics over `raios.audit_record.v0` and
`raios.rollback_plan.v0` candidates, including RAM-only retention of a valid
audit/rollback reference, live rejection of a wrong-schema retained
audit/rollback reference, and valid retained audit/rollback visibility in the
denied module load gate, plus RAM-only service-slot reservation diagnostics and
selftests over retained computed-grant/audit/rollback event ids, canonical
reservation hashes, pre-load service-inventory hashes, and `ram_only:` slot ids,
including live denied load-gate visibility of valid retained service-slot
reservation evidence without allocation and local-only negative service-slot
gate selftests.

## Verified Boot State

- Repository path: `C:\Users\admin\Documents\raios2`
- Boot image: `release/raios-stage0.img`
- Firmware vars seed: `release/ovmf_vars.fd`
- Bootloader: Limine 10 UEFI binary at `release/esp/EFI/BOOT/BOOTX64.EFI`
- Config file: `limine.conf` at ESP root and `EFI/BOOT/limine.conf`
- Kernel path inside image: `/kernel/kernel.elf`

The image boots in QEMU using the Windows PowerShell runner:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

For interactive serial commands, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555
```

For a QEMU xHCI inventory run, add `-UsbXhciInput`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555 -Headless -UsbXhciInput
```

For the bare-metal-style VM profile with USB keyboard, USB mouse, RDRAND, and
e1000 networking, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-baremetal-vm.ps1 -StopExisting
```

Expected xHCI inventory lines in that mode:

```text
usb-xhci: controller @ 00:03.0 detected
usb-xhci: hci 0x0100, ports 8, connected 2
usb-hid: boot keyboard ready on slot 1 endpoint 0x81
usb-hid: boot mouse ready on slot 2 endpoint 0x81
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
```

Expected visible framebuffer UI:

```text
AI  CONSOLE                                      SET
RAIOS
DIRECT AI HOST
NET CONFIGURED   INPUT READY   USB READY   RNG READY
CHAT
TYPE MESSAGE AND PRESS ENTER
```

Expected useful serial lines:

```text
Seed kernel: early init start
Limine loaded base revision: 3
HHDM offset=0xffff800000000000
Framebuffer response revision: 1
Framebuffer negotiated via Limine
status FRAMEBUFFER: READY - 1280x800 PITCH 5120
status ENTROPY: READY - FILL 64/64 TOTAL 64 SRC RDRAND
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
e1000: device 00:02.0 id=0x100e mmio=0x81040000 size=131072 mac 52:54:00:12:34:56
e1000 network initialised; DHCP polling enabled
DHCP lease acquired: ip 10.0.2.15/24 gw 10.0.2.2 dns ["10.0.2.3"]
status NETWORK: CONFIGURED - IP 10.0.2.15/24 GW 10.0.2.2
status INPUT: READY - USB HID KEYBOARD + POINTER
```

Console commands verified over TCP serial and USB-HID keyboard input:

```text
help
status
devices
log
provider
openai
setup
ask <text>
```

The framebuffer UI defaults to an AI chat mode. The `CONSOLE` tab keeps the
debug console visible, and the `SET` tab opens provider settings. `setup` also
opens the in-VM OpenAI/API-key menu. API-key entry is masked, held only in guest
RAM, and not printed into the console or serial output. For local-only testing,
the build scripts can also embed `OPENAI_API_KEY` into a separate non-default
image with `-EmbedOpenAiApiKeyFromEnv`.

Direct OpenAI trust-gate smoke over TCP serial:

```text
> provider
PROVIDER: OPENAI    API KEY: SET
ROUTE: OPENAI DIRECT
TLS TRUST: pin_config_missing
> ask direct provider smoke
OPENAI TLS TRUST DENIED: pin_config_missing
```

Direct OpenAI SPKI pinned-trust smoke is verified with a temporary image built
from a process-local fake API key and a current `OPENAI_SPKI_SHA256` pin.
Expected positive trust lines:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_spki sha256:<pin-id>
openai: HTTPS request sent
OPENAI HTTP
```

The legacy leaf-certificate pinned-trust smoke remains supported with
`OPENAI_CERT_SHA256`. Expected positive trust lines:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_cert sha256:<pin-id>
openai: HTTPS request sent
OPENAI HTTP
```

## Current Architecture Decision

Do not run or port the Codex CLI inside Stage-0.

Stage-0 should grow a small native agent host:

- framebuffer UI
- serial/keyboard/mouse input
- USB/input and PCI device inventory
- network status
- explicit capability-gated agent tools

Codex/OpenAI integrations should use a small native provider boundary. The OS
boundary should stay small and auditable; a full host CLI is not part of
Stage-0.

See `docs/architecture-decisions/0001-raios-agent-protocol.md`.

## Exact Next Task

Define the first guest VM-test-report hash-reference diagnostic:

- specify a local-only current-boot VM report reference record as evidence, not
  load authority
- accept only canonical report hash/reference input, not report JSON that
  pretends to be trusted runtime state
- bind the retained report reference to retained manifest, artifact, and
  computed-grant evidence
- make the live load gate report retained VM-report hash evidence only after a
  current-boot predicate validates the retained event
- keep the loader unavailable and recovery artifact loading separate from the
  normal module gate

The verified foundation for that task is:

- Virtio has been removed from the Stage-0 kernel runtime and VM runner path.
- RDRAND seeds entropy in the bare-metal-style VM profile.
- Intel e1000 configures RX/TX rings, negotiates DHCP through smoltcp, and shows
  IP/gateway state in the framebuffer UI and serial console.
- a PS/2/i8042 polling fallback is present for first bare-metal keyboard tests
  on machines that expose legacy keyboard compatibility. It is only reported as
  ready after an acknowledge from the keyboard or real scancode input.
- a polled xHCI path now inventories USB controllers, resets directly attached
  root-port devices, enumerates HID boot keyboards, relative boot mice, and QEMU
  HID tablets, and feeds reports into the same input queue as PS/2.
- if no USB keyboard or pointer is active, the event loop periodically re-probes
  xHCI so a keyboard plugged in after boot can be picked up without rebooting.
- the USB status line includes `EV`, `ERR`, and `TCC` counters for HID input
  reports and interrupt transfer diagnostics on bare metal.
- the USB-XHCI row now includes keyboard and mouse readiness.
- the framebuffer renderer is double-buffered to avoid visible full-screen
  redraw flicker, and pointer movement now updates only a small cursor overlay
  instead of forcing a full UI redraw.
- the visible QEMU GTK profile uses `usb-tablet` absolute pointer input by
  default and hides the host cursor over the guest area without automatic mouse
  grab, so only the raiOS pointer is visible and remains aligned after focus
  changes; `-RelativeMouse` or `-MouseGrab` switches back to relative
  `usb-mouse` for stricter boot-mouse testing.
- the visible UI now defaults to a chat-first surface with `AI`, `CONSOLE`, and
  `SET` modes. Serial commands continue to use the command interpreter so VM
  harnesses remain deterministic.
- USB/PS2 keyboard input now carries special keys into the UI: Tab and arrow
  keys move a visible focus ring through the top navigation, chat/console input,
  and settings actions; Enter activates the focused item and Esc backs out of
  settings/API-key entry.
- the Surface Pro 4 internal WLAN target has been selected as Marvell AVASTAR
  88W8897 (`11ab:2b38`, Linux reference driver family `mwifiex_pcie`). Stage-0
  now probes PCI for that device and exposes it as a Wi-Fi status chip/log line,
  and the settings menu can record a RAM-only SSID and WPA passphrase. Firmware
  upload, WPA, and packet transport are not implemented yet.
- a VM-local `setup` menu now records a RAM-only OpenAI API key without echoing
  the key back into the serial log.
- `ask <text>` now stays inside the guest. In the normal build it requires the
  VM API key state and then fails closed at provider trust before API-key copy or
  HTTPS write unless a syntactically valid provider pin is configured. With
  `-EmbedOpenAiSpkiPinFromEnv`, the preferred verifier slice checks the OpenAI
  leaf SubjectPublicKeyInfo SHA-256 pin and the TLS 1.3 P-256 ECDSA
  `CertificateVerify` proof before copying the API key or writing HTTPS. With
  `-EmbedOpenAiCertPinFromEnv`, the first positive verifier slice checks the
  OpenAI leaf certificate SHA-256 pin and the TLS 1.3 P-256 ECDSA
  `CertificateVerify` proof before copying the API key or writing HTTPS. With
  the explicit development override
  `-AllowUnverifiedOpenAiTls`, it resolves `api.openai.com`, opens TCP 443
  through e1000, performs TLS 1.3 with `NoVerify`, sends an HTTPS Responses API
  request, parses `output_text`, and prints the provider response.
- the provider trust state is visible in console/provider status,
  `system.snapshot.v0`, `problem.list`, and `service.inventory`; the default
  trust problem is `provider.tls_pin_config_missing`, while a successful pinned
  handshake reports `pinned_spki_verified` or `pinned_cert_verified`.
- `raios.agent.v0` exposes read-only serial methods for `system.describe`,
  `system.snapshot`, `system.capabilities`, `system.boot_log`, `device.graph`,
  `problem.list`, `service.inventory`, `memory.profile`, `memory.context`,
  `memory.query`, `memory.trace`, `memory.recent_events`, `audit.events`,
  `module.grant_diagnostic`, and `module.grant_diagnostic_selftest`.
- mutating or potentially mutating methods such as `module.load_ephemeral`,
  `service.restart`, `config.apply`, `provider.configure`, and `wifi.configure`
  return structured `capability_denied` until manifest, VM test report, local
  attestation, computed capability grant, approval, audit, and rollback evidence
  exist.
- `module.load_ephemeral` and `service.load_ephemeral` now return
  `raios.module_load_gate.v0`, which reports the manifest, exact artifact, VM
  report, local attestation, computed grant, local approval, durable audit,
  rollback plan, loader, and ram-only service slot gates; the current state is
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.manifest_diagnostic` now exposes a read-only guest diagnostic for a
  module-manifest hash reference. It accepts no manifest JSON, artifact bytes,
  or unsigned service code and validates only the canonical
  `raios.module_manifest_reference.v0` hash over the manifest hash, requested
  capability, load mode, subject, resource, and current-boot scope.
- A valid `module.manifest_diagnostic` reference is retained as a local-only
  current-boot `raios.module_manifest_reference.v0` event binding. The retained
  record stores hashes only, appears through `retained_manifest_reference` and
  `audit.events`, and remains non-authorizing with
  `authorizes_guest_load: false`, `can_load_now: false`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained manifest reference before snapshotting it into the denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `module_manifest: retained_hash_reference_only`,
  `retained_module_manifest_reference.state: present`, retained hashes, and
  `retained_module_manifest_reference_not_authorizing`; stale, substituted,
  wrong-schema, or hash-mismatched references are rejected without exposing their
  manifest hashes as accepted evidence.
- `module.load_gate_manifest_selftest` now exposes local-only
  `raios.module_load_gate_manifest_selftest.v0` test infrastructure
  for missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record, and
  hash-mismatch retained manifest-reference candidates without mutating the
  global event log, accepting manifest JSON or artifact bytes, or loading
  artifacts.
- `module.artifact_diagnostic` now exposes a read-only guest diagnostic for a
  candidate-artifact hash reference. It accepts no manifest JSON, artifact
  bytes, or unsigned service code and validates the canonical
  `raios.module_candidate_artifact_reference.v0` hash over retained manifest and
  computed-grant event ids plus manifest, artifact, report, attestation, and
  grant hashes.
- A valid `module.artifact_diagnostic` reference is retained as a local-only
  current-boot `raios.module_candidate_artifact_reference.v0` event binding. The
  retained record stores hashes only, appears through
  `retained_candidate_artifact_reference` and `audit.events`, and remains
  non-authorizing with `artifact_loaded: false`, `can_load_now: false`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained candidate-artifact reference before snapshotting it into the denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `candidate_artifact: retained_hash_reference_only`,
  retained artifact hashes, and
  `retained_candidate_artifact_reference_not_authorizing`; stale, substituted,
  wrong-schema, or hash-mismatched references are rejected without exposing their
  artifact hashes as accepted evidence.
- `module.load_gate_artifact_selftest` now exposes local-only
  `raios.module_load_gate_artifact_selftest.v0` test infrastructure for missing,
  accepted-current-boot-but-denied, stale/dropped, previous-boot-or-unretained,
  wrong-schema, substituted-record, hash-mismatch, manifest-reference mismatch,
  and computed-grant-reference mismatch candidates without mutating the global
  event log or loading artifacts.
- host-side `registry-tools grant-diagnostic` now emits
  `raios.computed_capability_grant.v0` over an exact module manifest,
  candidate artifact, Shadow-VM report, local attestation, approval phrase,
  requested capability, subject, resource, and current-boot scope. The
  diagnostic is evidence only: valid tuples set
  `computed_candidate_present: true`, while `grants_capability`,
  `grants_load_now`,
  `authorizes_guest_load`, `can_load_now`, and `load_attempted` remain false.
- `registry-core` unit tests reject mismatched manifest/artifact/report/
  attestation hashes, non-empty manifest `granted_caps`, wrong approval
  phrases, and `limits.grants_load_now: true` attestations.
- `module.grant_diagnostic` now exposes a read-only guest diagnostic for a
  computed-grant hash reference. It accepts no artifact bytes and validates only
  the `raios.computed_capability_grant.canonical.v0` hash over manifest,
  artifact, VM-report, and local-attestation hashes. A valid reference sets
  `computed_candidate_present: true` but still keeps
  `grants_capability: false`, `grants_load_now: false`,
  `authorizes_guest_load: false`,
  `can_load_now: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- A valid `module.grant_diagnostic` reference is now retained as a local-only
  current-boot `raios.module_computed_grant_reference.v0` event binding. The
  retained record stores hashes only, appears through `retained_reference` and
  `audit.events`, and remains non-authorizing with
  `grants_capability: false`, `grants_load_now: false`,
  `authorizes_guest_load: false`, `can_load_now: false`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now snapshot the latest
  retained computed-grant reference into their denied
  `raios.module_load_gate.v0` response and event binding. With a retained
  reference, the gate reports
  `computed_capability_grant: retained_hash_reference_only`,
  `retained_computed_grant_reference.state: present`, retained hashes, and
  `retained_computed_grant_reference_not_authorizing`, while still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_gate_retained_selftest` now exposes local-only
  `raios.module_load_gate_retained_reference_selftest.v0` test infrastructure
  for the denied load gate's retained-reference predicate. It covers missing,
  accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record, and
  hash-mismatch candidates without mutating the global event log, creating
  retained records, loading artifacts, or mutating service inventory.
- `module.load_ephemeral` and `service.load_ephemeral` also expose
  `raios.module_load_gate_audit_rollback_requirements.v0` in the denied
  response and event binding. The requirement schema names
  `raios.audit_record.v0`, `raios.rollback_plan.v0`, retained grant/reference
  ids, local approval, rollback-plan hash, and ram-only service-slot id as
  required but missing, with writes disabled and `can_load: false`.
- `module.load_gate_audit_rollback_selftest` now exposes local-only
  `raios.module_load_gate_audit_rollback_selftest.v0` test infrastructure for
  missing/stale/previous-boot/wrong-schema/substituted retained
  audit/rollback references, retained computed-grant/audit/rollback hash
  mismatches, retained service-slot mismatch, missing durable audit, missing
  rollback plan, matching-but-still-denied audit/rollback evidence,
  audit/rollback schema mismatches, retained grant hash mismatch,
  manifest/artifact/VM-report/local-attestation mismatches, local approval
  mismatch, rollback hash mismatch, rollback artifact mismatch, and rollback
  service-slot mismatch. It creates no retained references, durable audit
  records, rollback plans, service slots, event-log records, or loads.
- `registry-tools audit-rollback-diagnostic` now emits
  `raios.module_audit_rollback_diagnostic.v0` with nested
  `raios.audit_record.v0` and `raios.rollback_plan.v0` candidates. It binds
  the retained computed-grant hash, retained-reference event id, denied load
  event id, local approval, ram-only service-slot id, rollback plan hash,
  manifest, artifact, VM report, and local attestation while keeping
  `durable_audit_written: false`, `rollback_plan_installed: false`,
  `can_load_now: false`, and `load_attempted: false`.
- `registry-core` unit tests now reject audit/rollback candidate mismatches for
  retained grant hash, manifest, artifact, report, attestation, approval,
  rollback hash, and service-slot ids.
- `module.audit_rollback_diagnostic` now exposes
  `raios.module_audit_rollback_reference_diagnostic.v0` as a guest
  hash-reference diagnostic. It accepts only hashes and current-boot ids for the
  audit record, rollback plan, computed grant, retained reference, denied load
  event, manifest, artifact, VM report, local attestation, local approval,
  pre-load service inventory, cleanup actions, and ram-only service slot. A
  valid reference reports `valid_hash_reference_load_still_denied`, records one
  local-only current-boot `raios.module_audit_rollback_reference.v0` event
  binding, and still keeps `durable_audit_written`,
  `rollback_plan_installed`, `can_load_now`, and `load_attempted` false.
- `module.audit_rollback_diagnostic_selftest` covers absent, accepted
  current-boot, stale, previous-boot event id, wrong-schema, substituted audit
  hash, rollback hash mismatch, computed-grant hash mismatch, and invalid
  ram-only service-slot cases without creating audit records, rollback plans,
  service slots, retained references, or service inventory changes.
- `module.service_slot_diagnostic` now exposes
  `raios.module_service_slot_reservation_diagnostic.v0` as a guest
  hash-reference diagnostic. It binds a reservation hash to the retained
  computed-grant reference id, retained audit/rollback reference id, computed
  grant hash, audit-record hash, rollback-plan hash, pre-load service-inventory
  hash, and `ram_only:` slot id. A valid reference records only a local-only
  current-boot `raios.module_service_slot_reservation.v0` event binding and
  keeps `allocates_service_slot`, `creates_service_inventory_records`,
  `can_load_now`, and `load_attempted` false.
- `module.service_slot_diagnostic_selftest` covers absent, accepted
  current-boot, stale, mismatched reservation hash, and invalid `ram_only:`
  service-slot cases without mutating the global event log, creating retained
  reservation records, allocating slots, loading artifacts, or changing service
  inventory.
- `module.load_ephemeral` and `service.load_ephemeral` now also validate the
  latest retained audit/rollback reference before snapshotting it into the
  denied `raios.module_load_gate.v0` response and event binding. The live
  predicate checks that the retained reference binds the latest retained
  computed-grant reference, a prior denied load event, canonical computed-grant,
  rollback-plan, and audit-record hashes, and a valid `ram_only:` service-slot
  id. With a valid retained reference, the gate reports
  `durable_audit_record: retained_hash_reference_only_not_durable`,
  `rollback_plan: retained_hash_reference_only_not_installed`,
  `retained_audit_rollback_reference.state: present`, retained audit/rollback
  hashes, `retained_audit_record_reference_not_durable`, and
  `retained_rollback_plan_reference_not_installed`, while still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`. A retained reference that points at a wrong-schema
  event or mismatched hashes is reported as `rejected_retained_reference`, and
  its audit/rollback hashes are not exposed as accepted gate evidence.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained `raios.module_service_slot_reservation.v0` before snapshotting it
  into the same denied gate. The live predicate checks the retained grant and
  audit/rollback event ids, referenced event variants, canonical reservation
  hash, computed-grant/audit/rollback hashes, pre-load service-inventory hash,
  and `ram_only:` slot id. A valid reservation reports
  `service_slot: retained_hash_reference_only_not_allocated`,
  `retained_service_slot_reservation.state: present`, and
  `service_slot_reservation_hash` while keeping
  `allocates_service_slot: false`, `can_load: false`,
  `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_gate_service_slot_selftest` now exposes local-only
  `raios.module_load_gate_service_slot_selftest.v0` test infrastructure for
  missing, accepted-current-boot, stale/dropped, wrong-schema, substituted,
  computed-grant-hash, audit-hash, rollback-hash, inventory-hash,
  service-slot, and reservation-hash retained service-slot reservation cases.
  Rejected cases report `rejected_retained_reference` and keep accepted
  `service_slot_reservation_hash` evidence absent.
- `module.grant_diagnostic_selftest` covers absent, accepted-current-boot,
  stale previous-boot, mismatched manifest-hash, and wrong-policy computed
  grant references without loading artifacts or mutating service inventory.
- `vm-harness\shadow-vm-smoke.ps1` verifies the read-only agent protocol,
  provider trust problem visibility, static service inventory, and denied module
  load behavior, then writes a `raios.vm_test_report.v0` report.
- `memory.profile`, `memory.context`, `memory.query`, and `memory.trace` now
  expose a local read-only `current_boot` memory context slice. The
  `memory.context` result schema is `raios.agent_context.v0`, includes
  current-boot `context_event_id`/`audit_event_id` handles for the local read,
  and provider export is explicitly disabled.
- `memory.context provider_minimal` now emits a local-only
  `raios.provider_context_projection.v0` preview with explicit
  `public`/`local_only`/`secret` field classification, included and omitted
  field lists, deterministic `packet_evidence` hashes for the canonical packet
  plus exported and omitted field lists, a nested redacted
  `raios.agent_context.v0` packet, and `can_export: false` until positive
  provider trust and a distinct provider export audit binding exist.
- `provider.context_export provider_minimal` now exposes the first
  `raios.provider_context_export.v0` gate. It returns structured
  `capability_denied`, records `cap.provider.context_export` with risk
  `export`, reports `provider_write: not_attempted`, reports packet and
  field-list evidence bindings as present, keeps the positive provider request
  binding and export audit binding gates missing, and emits separate
  current-boot denial evidence as
  `raios.provider_request_binding_denial.v0` and
  `raios.provider_context_export_denial_audit.v0`.
- `event.log.v0` now carries structured `bindings` for those denial events:
  both include the canonical provider-minimal packet hash plus exported and
  omitted field-list hashes, and both explicitly report
  `satisfies_current_boot_export_gate: false`.
- The real OpenAI `ask` path now emits a local-only
  `OPENAI_PROVIDER_REQUEST_ENVELOPE` serial marker with schema
  `raios.provider_request_envelope.v0` after request id allocation and before
  DNS/TCP/TLS/API-key copy/HTTPS write. It records redacted request-body shape,
  body hash, envelope hash, trust snapshot, `provider_write: not_attempted`,
  and `context_attached_to_provider_body: false` without raw prompt text,
  `Content-Length`, API keys, or Authorization values.
- On the `pinned_spki_verified` direct OpenAI path, after TLS proof and matching
  request-body hash validation but before API-key copy or HTTPS write, Stage-0
  now records local-only positive
  `raios.provider_request_binding.v0` and
  `raios.provider_context_export_audit_binding.v0` events. They bind the exact
  request-body hash, request-envelope hash, provider-minimal packet hash,
  exported-field-list hash, and omitted-field-list hash. The request binding
  satisfies only `satisfies_request_binding_gate: true`; the export audit
  binding sets `positive_export_authorization: true`, but both retain
  `satisfies_current_boot_export_gate: false`,
  `automatic_context_injection: disabled`, and
  `context_attached_to_provider_body: false`.
- `provider.context_gate provider_minimal` now exposes a read-only
  `raios.provider_context_export_gate_state.v0` diagnostic over retained
  current-boot binding records. It can validate one matching positive request
  binding plus export-audit binding pair while keeping `can_export: false`.
- `provider.context_gate_selftest provider_minimal` exposes local-only test
  infrastructure over the same gate predicate. It does not mutate the global
  event log, create request envelopes, create positive binding records, or
  attempt provider writes. The Shadow VM smoke now covers stale/dropped event
  ids, previous-boot-or-unretained ids, denial-schema substitution,
  positive-record substitution, request/body/binding hash mismatches, context
  hash mismatches, and trust-bypass records.
- `provider.context_export provider_minimal` now consumes one valid retained
  positive binding pair for local gate evaluation only, records
  `raios.provider_context_binding_consumption.v0`, and still returns
  `capability_denied` because `automatic_context_injection` remains disabled.
  A second attempt against the same pair is rejected as
  `binding_already_consumed`.
- `provider.context_injection_gate provider_minimal` now exposes the separate
  `raios.provider_context_injection_gate.v0` diagnostic. It reports final
  authorization as missing, requires
  `raios.provider_context_injection_authorization.v0`, keeps
  `automatic_context_injection: disabled`, and reports
  `can_attach_context: false`.
- `provider.context_injection_gate_selftest provider_minimal` exposes local-only
  test infrastructure over the final injection predicate. It does not mutate the
  global event log, create real request envelopes, create positive binding
  records, create final authorization records, attempt provider writes, or
  attach context. The Shadow VM smoke now covers missing, stale/dropped,
  wrong-schema, substituted-positive-record, final body-hash mismatch, trust
  downgrade, and body-attachment-without-final-authorization cases.
- On positive pinned/WebPKI OpenAI request paths, Stage-0 now emits a local-only
  `OPENAI_PROVIDER_CONTEXT_INJECTION_GATE` marker after request/export binding
  evidence and before API-key copy or HTTPS write. The marker binds the request
  body hash, request-envelope hash, and provider-minimal context hashes while
  keeping provider write not attempted and body attachment false.
- `provider.context_export` still does not create a request envelope; the
  Shadow VM smoke checks that denied export cannot fake one.
- `memory.query` and `memory.trace` include
  `snapshot.current.provider_minimal` as the stable locator for the redacted
  current-status projection.
- `memory.recent_events` and `audit.events [limit]` expose a bounded RAM-only
  `event.log.v0` ring containing compact `audit.event.v0` records for agent
  protocol reads, known `capability_denied` outcomes, provider request-binding
  denials, provider export-denial audits with hash-valued denial bindings, and
  the `raios.module_load_gate.v0` denial binding.
- denied memory/module/service/config methods include current-boot `event_id`
  and `audit_event_id` handles, while all durable audit, persistence, policy
  mutation, redaction mutation, and rollback behavior remains denied.
- memory mutation methods (`memory.record_observation`,
  `memory.propose_policy`, `memory.supersede_fact`, `memory.redact`, and
  `memory.compact`) return structured `capability_denied` with missing audit and
  persistence evidence.
- `vm-harness\shadow-vm-smoke.ps1` now verifies memory-context schemas,
  context event ids, the local `provider_minimal` redaction projection, the
  provider-minimal packet/field-list hashes, the denied
  `provider.context_export` gate with hash bindings present, positive request
  and export audit bindings still missing, denial-audit records present but not
  satisfying export gates, provider writes still not attempted, memory
  query/trace, event log schemas, audit alias, memory mutation denials with
  event ids, the read-only `provider.context_gate` missing-binding state, the
  `provider.context_gate_selftest` negative predicate cases, the separate
  `provider.context_injection_gate` missing-final-authorization state, the
  `provider.context_injection_gate_selftest` negative final-authorization cases,
  the read-only module manifest, candidate-artifact, and computed-grant
  diagnostics and selftests, the module audit/rollback hash-reference
  diagnostics and selftests, retained `raios.module_manifest_reference.v0`,
  `raios.module_candidate_artifact_reference.v0`,
  `raios.module_computed_grant_reference.v0`, and
  `raios.module_audit_rollback_reference.v0` event bindings, and the denied
  module load gate including retained manifest, retained artifact, retained
  computed-grant, plus retained audit/rollback reference state in the response
  and event-log binding, live wrong-schema retained audit/rollback rejection,
  plus negative manifest-reference, artifact-reference, retained-reference,
  retained audit/rollback reference, and audit/rollback requirement selftests,
  service-slot reservation diagnostics and selftests, live denied load-gate
  visibility of valid retained service-slot reservation evidence, and negative
  retained service-slot reservation gate selftests.
  Latest report:
  `release\vm-reports\shadow-20260520-182402-20552.json` with 807/807
  predicates.
- `vm-harness\openai-direct-smoke.ps1 -ExpectPinMismatch` was run against a
  local image built with a fake API key and intentionally wrong SPKI pin. It
  verified the real request envelope marker appears on the `ask` path, omits raw
  prompt/Content-Length/Authorization values, then fails at pin mismatch before
  HTTPS request data is sent and without positive request/export audit binding
  markers.
- `vm-harness\openai-direct-smoke.ps1 -ExpectSpkiPinnedTrust` was run against a
  local image built with a fake API key and the current OpenAI SPKI pin. It
  verified the real request envelope marker, positive request binding marker,
  positive export audit binding marker, and blocked injection-gate marker appear
  before the HTTPS write path, that marker body/envelope/binding/context hashes
  match, that
  `provider.context_gate` validates the retained pair, that
  `provider.context_export` consumes it once for local gate evaluation, and that
  the second consumption attempt returns `binding_already_consumed`, while
  provider-minimal context remains unattached.
- the development serial relay and old host-framing path have been removed from
  the runtime path.
- the next trust milestone is WebPKI or broader certificate algorithm support
  once trust anchors, time, hostname checks, and chain handling are specified.

## Known Gaps

- Windows now has a minimal image repackaging path:
  `scripts\package-stage0.ps1` creates `release\raios-stage0.img` from
  `release\esp`.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
- Network failure/timeout states and packet counters are still minimal.
- Keyboard input uses a minimal US/Linux keycode mapping; no layout selection,
  modifier completeness, or text editing beyond Backspace exists yet.
- Bare-metal support is experimental. Minimal direct xHCI USB-HID boot keyboard,
  mouse, hub traversal, and a limited no-input USB hotplug rescan exist, but full
  detach/reconfigure handling and broad NIC coverage do not exist yet, so real
  hardware may still boot to the UI but lack input/network unless it matches the
  implemented paths.
- Wi-Fi support currently detects the Surface Pro 4 Marvell AVASTAR 88W8897
  target and stores RAM-only SSID/WPA configuration for the current boot. The
  next implementation step is a Marvell PCIe firmware-upload path before 802.11
  association or WPA2 can work.
- Bare-metal USB preparation scripts exist, but writing a USB disk is destructive
  and must be done with an explicit disk number and confirmation string.
- API key entry exists in the VM, but the key is RAM-only and not persisted in
  the default image. A local test image can embed the key explicitly, but must
  not be committed or shared.
- Stage-0 has verified DNS/TCP/TLS/HTTPS for `api.openai.com:443` behind the
  explicit unverified development override, the preferred SPKI pin verifier, and
  the legacy leaf-certificate pin verifier. SPKI pinning still depends on the
  leaf using the currently supported P-256 ECDSA `CertificateVerify` path;
  broader algorithm support or WebPKI remains a hardening step.
- The OpenAI JSON response parser is intentionally minimal and only extracts the
  first `output_text` string.
- QEMU TCP serial is single-client in practice; do not run two serial clients
  against the same port at the same time.
- No signed module runtime exists yet.

## Do Not Regress

- Do not rename `limine.conf` back to `limine.cfg`.
- Do not remove Limine request start/end markers.
- Do not link the kernel lower-half.
- Do not assume Linux packaging tools are available on this Windows host.
- Do not delete or overwrite `release/raios-stage0.img` unless the replacement
  has booted in QEMU.
