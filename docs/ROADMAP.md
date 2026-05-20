# Roadmap

## Agent Handoff Cursor

Last updated: 2026-05-20 by Codex after adding the first RAM-only service-slot
reservation hash-reference diagnostic while keeping `cap.module.load_ephemeral`
denied. The guest now validates
`raios.module_service_slot_reservation.v0` references through
`module.service_slot_diagnostic`, binding a `ram_only:` slot id to retained
computed-grant and audit/rollback reference event ids, canonical hashes, and the
pre-load service-inventory hash. Valid reservations are retained only as
local-only current-boot evidence and still allocate no slot, create no service
inventory records, load no artifact, and grant no live execution.

Latest maintenance verification:

- `git diff --check` passed.
- `cargo fmt --all -- --check` passed.
- `cargo test --locked -p ota-tools -p registry-core -p registry-tools -p fake-cloud-server`
  passed.
- `cargo test --locked -p registry-core -p registry-tools` passed after adding
  the computed grant diagnostic, audit/rollback diagnostic, and their negative
  evidence tests.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release`
  passed.
- `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1`
  passed and wrote
  `release\vm-reports\shadow-20260520-164131-22208.json` with 593/593
  predicates, including `module.grant_diagnostic`,
  `module.grant_diagnostic_selftest`, `module.audit_rollback_diagnostic`,
  `module.audit_rollback_diagnostic_selftest`,
  `module.service_slot_diagnostic`, `module.service_slot_diagnostic_selftest`,
  retained
  `raios.module_computed_grant_reference.v0` and
  `raios.module_audit_rollback_reference.v0` plus
  `raios.module_service_slot_reservation.v0` audit/event binding coverage, and
  retained-reference state plus live wrong-schema retained audit/rollback
  rejection, negative retained-reference, retained audit/rollback reference,
  and audit/rollback requirement selftests in the denied module load gate.
- `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1 -ExpectPinMismatch`
  passed against a local fake-key image with an intentionally wrong SPKI pin;
  positive request/export audit binding markers stayed absent. The local image
  was deleted and the release kernel was rebuilt without the test environment
  afterward.
- `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1 -ExpectSpkiPinnedTrust`
  passed against a local fake-key image with the current OpenAI SPKI pin; the
  request envelope, positive request binding, and positive export audit binding
  markers appeared, marker hashes matched, `provider.context_gate` validated the
  retained pair, `provider.context_export` consumed it once for local gate
  evaluation, the second consumption attempt was rejected as
  `binding_already_consumed`, and provider-minimal context stayed unattached.
  The local image was deleted and the release kernel was rebuilt without the
  test environment afterward.

This file is the planning handoff. Every agent that changes code, tests,
protocol docs, architecture docs, or verified project state must update this
section before handing off. If the exact next task changes, also update
`docs/PROJECT_STATUS.md`.

Current verified cursor:

- Stage-0 is a bootable Limine/Rust kernel with framebuffer UI, serial console,
  USB-HID/xHCI input, e1000 DHCP, entropy, setup UI, and direct OpenAI transport
  through the guest network path.
- Provider trust is fail-closed by default. OpenAI SPKI SHA-256 pinning is the
  preferred positive verifier slice, and legacy leaf-certificate SHA-256 pinning
  remains supported. The unverified TLS path exists only behind the explicit
  development build switch.
- `raios.agent.v0` exists over the serial console with read-only methods:
  `system.describe`, `system.snapshot`, `system.capabilities`,
  `system.boot_log`, `device.graph`, `problem.list`, and
  `service.inventory`, plus current-boot memory, provider gate, and module
  grant diagnostic reads.
- Mutating or potentially mutating methods currently return structured
  `capability_denied` until manifest, VM test report, local attestation,
  computed grant, approval, audit, and rollback evidence exist.
- `module.load_ephemeral` and `service.load_ephemeral` now use
  `raios.module_load_gate.v0`. The current gate reports missing
  manifest/artifact/report/attestation/grant/approval/audit/rollback evidence,
  loader unavailable, service slot unallocated, `can_load: false`,
  `service_inventory_change: none`, and `load_attempted: false`, and the same
  gate is visible as an `event.log.v0` binding.
- `registry-tools grant-diagnostic` now emits
  `raios.computed_capability_grant.v0` for host-side evidence review. A valid
  tuple sets `computed_candidate_present: true`, but
  `grants_capability`, `grants_load_now`, `authorizes_guest_load`,
  `can_load_now`, and `load_attempted` remain false.
- `registry-core` unit tests cover mismatched manifest/artifact/report/
  attestation hashes, non-empty manifest `granted_caps`, wrong approval
  phrases, and attestations that set `limits.grants_load_now: true`.
- `registry-tools audit-rollback-diagnostic` now emits
  `raios.module_audit_rollback_diagnostic.v0` for host-side evidence review.
  It creates canonical `raios.audit_record.v0` and `raios.rollback_plan.v0`
  candidates that bind retained grant/reference ids, denied load event id,
  local approval, rollback hash, ram-only service-slot id, manifest, artifact,
  VM report, and local attestation. It remains non-authorizing:
  `durable_audit_written`, `rollback_plan_installed`, `can_load_now`, and
  `load_attempted` remain false.
- `registry-core` unit tests cover audit/rollback mismatches for retained grant
  hash, manifest, artifact, report, attestation, approval, rollback hash, and
  service-slot ids.
- `module.audit_rollback_diagnostic` now exposes
  `raios.module_audit_rollback_reference_diagnostic.v0` as a guest
  hash-reference diagnostic over host audit/rollback candidates. It accepts
  only hashes and current-boot ids, recomputes the canonical grant,
  rollback-plan, and audit-record hashes, rejects stale, previous-boot,
  wrong-schema, substituted, mismatched, and invalid service-slot candidates,
  records valid references as local-only current-boot
  `raios.module_audit_rollback_reference.v0` bindings, and keeps
  `durable_audit_written`, `rollback_plan_installed`, `can_load_now`, and
  `load_attempted` false.
- `module.service_slot_diagnostic` now exposes
  `raios.module_service_slot_reservation_diagnostic.v0` as a guest
  hash-reference diagnostic over retained grant and audit/rollback evidence. It
  accepts only hashes and current-boot ids, recomputes the canonical reservation
  hash, rejects stale, mismatched, invalid-slot, and live retained-reference
  mismatch candidates, records valid reservations as local-only current-boot
  `raios.module_service_slot_reservation.v0` bindings, and keeps
  `allocates_service_slot`, `creates_service_inventory_records`,
  `can_load_now`, and `load_attempted` false.
- `module.grant_diagnostic` now exposes
  `raios.module_computed_grant_diagnostic.v0` as a read-only hash-reference
  diagnostic. It accepts no artifact bytes, recomputes the canonical grant hash
  from supplied manifest/artifact/report/attestation hashes, and keeps
  `grants_capability`, `grants_load_now`, `authorizes_guest_load`,
  `can_load_now`, and `load_attempted` false.
- Valid `module.grant_diagnostic` references are retained in the RAM-only
  current-boot event log as local-only
  `raios.module_computed_grant_reference.v0` bindings. The retained record
  stores hashes only, appears through `retained_reference` and `audit.events`,
  and remains non-authorizing.
- `module.grant_diagnostic_selftest` now exposes local-only test infrastructure
  for absent, accepted-current-boot, stale, mismatched, and wrong-policy
  computed grant references without loading artifacts or mutating
  `service.inventory.v0`.
- `module.load_ephemeral` and `service.load_ephemeral` now snapshot the latest
  retained computed-grant reference into their denied
  `raios.module_load_gate.v0` response and event binding. A retained reference
  changes the computed-grant gate state to `retained_hash_reference_only` with
  reason `retained_computed_grant_reference_not_authorizing`, while
  `can_load`, `load_attempted`, and `service_inventory_change` remain false or
  `none`.
- `module.load_ephemeral` and `service.load_ephemeral` now also validate the
  latest retained audit/rollback reference before snapshotting it into the same
  denied response and event binding. The live predicate checks the retained
  computed-grant event, prior denied load event, canonical computed-grant,
  rollback-plan, and audit-record hashes, and the `ram_only:` service-slot id.
  A valid retained audit/rollback reference changes the durable-audit and
  rollback-plan gate states to
  `retained_hash_reference_only_not_durable` and
  `retained_hash_reference_only_not_installed`, while `can_load`,
  `load_attempted`, and `service_inventory_change` remain false or `none`. A
  wrong-schema, substituted, or hash-mismatched retained reference is reported
  as `rejected_retained_reference` and its audit/rollback hashes are not exposed
  as accepted evidence.
- `module.load_gate_retained_selftest` now exposes local-only
  `raios.module_load_gate_retained_reference_selftest.v0` test infrastructure
  for missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record, and
  hash-mismatch retained-reference candidates without mutating the global event
  log or creating retained records.
- `module.load_ephemeral` and `service.load_ephemeral` also expose
  `raios.module_load_gate_audit_rollback_requirements.v0` in the denied
  response and event binding. The requirement schema names
  `raios.audit_record.v0`, `raios.rollback_plan.v0`, retained grant/reference
  ids, retained audit/rollback reference ids, local approval,
  rollback-plan hash, and ram-only service-slot id as required. Retained
  audit/rollback references report hash-reference-only states, not durable
  authority. Writes remain disabled.
- `module.load_gate_audit_rollback_selftest` now exposes local-only
  `raios.module_load_gate_audit_rollback_selftest.v0` test infrastructure for
  missing/stale/previous-boot/wrong-schema/substituted retained
  audit/rollback references, retained computed-grant/audit/rollback hash
  mismatches, retained service-slot mismatch, missing durable audit, missing
  rollback plan, matching-but-still-denied audit/rollback evidence,
  audit/rollback schema mismatches, retained grant hash mismatch,
  manifest/artifact/VM-report/local-attestation mismatches, local approval
  mismatch, rollback hash mismatch, rollback artifact mismatch, and rollback
  service-slot mismatch without mutating the global event log, creating retained
  references, creating audit/rollback records, allocating service slots, or
  loading artifacts.
- `system.snapshot.v0`, `system.capabilities.v0`, `problem.list.v0`,
  `service.inventory.v0`, provider trust docs, module manifest docs, VM test
  report docs, local attestation docs, and recovery protocol docs exist.
- `memory.profile`, `memory.context`, `memory.query`, and `memory.trace` exist
  as local read-only `current_boot` methods. `memory.context` emits
  `raios.agent_context.v0` with local read event ids; provider export remains
  disabled.
- `memory.context provider_minimal` emits a local
  `raios.provider_context_projection.v0` preview with explicit field
  classification, included/omitted field lists, a nested redacted context
  packet, deterministic packet/field-list hashes, and `can_export: false` until
  positive provider trust and a distinct provider export audit binding exist.
- `provider.context_export provider_minimal` exists as a denied-by-default
  provider-boundary gate. It emits `raios.provider_context_export.v0`, records
  `cap.provider.context_export` with risk `export`, reports
  `provider_write: not_attempted`, reports packet/field-list evidence bindings
  as present, keeps positive provider request binding and export audit binding
  missing, and emits separate current-boot denial evidence for request-binding
  denial and export-denial audit.
- `event.log.v0` denial events now carry structured `bindings` with
  provider-minimal packet, exported-field-list, and omitted-field-list hashes,
  while explicitly marking current-boot export-gate satisfaction false.
- `raios.provider_request_envelope.v0` is specified in
  `device-protocol/provider-request-envelope-v0.md` and is now emitted as a
  local-only `OPENAI_PROVIDER_REQUEST_ENVELOPE` marker on the real OpenAI `ask`
  path before DNS/TCP/TLS/API-key copy/HTTPS write. It binds the exact request
  body hash and envelope hash while keeping context attachment false.
- After positive pinned provider trust and matching request-body hash validation
  on the real OpenAI `ask` path, Stage-0 records local-only
  `raios.provider_request_binding.v0` and
  `raios.provider_context_export_audit_binding.v0` evidence before API-key copy
  or HTTPS write. These records bind the request body/envelope hashes to the
  provider-minimal packet/exported/omitted field-list hashes, but
  `satisfies_current_boot_export_gate` remains false because automatic context
  injection is still disabled.
- `provider.context_gate provider_minimal` exposes read-only
  `raios.provider_context_export_gate_state.v0` diagnostics for retained
  positive binding pairs. `provider.context_export provider_minimal` consumes a
  valid pair once for local gate evaluation and records
  `raios.provider_context_binding_consumption.v0`, but still returns
  `capability_denied` and keeps context unattached.
- `provider.context_gate_selftest provider_minimal` exposes local-only
  `raios.provider_context_gate_negative_selftest.v0` test infrastructure over
  the same gate predicate. It covers stale/dropped event ids,
  previous-boot-or-unretained ids, denial-schema substitution,
  positive-record substitution, request/body/binding/context hash mismatches,
  and trust-bypass records without mutating the global event log or creating
  provider request envelopes.
- `provider.context_injection_gate provider_minimal` exposes the separate
  `raios.provider_context_injection_gate.v0` diagnostic. It makes the final
  authorization schema explicit as
  `raios.provider_context_injection_authorization.v0`, reports that
  authorization as missing, and keeps `can_attach_context: false`.
- `provider.context_injection_gate_selftest provider_minimal` exposes local-only
  `raios.provider_context_injection_gate_negative_selftest.v0` test
  infrastructure over the final authorization predicate. It covers missing,
  stale/dropped, wrong-schema, substituted-positive-record, final body-hash
  mismatch, trust downgrade, and body-attachment-without-final-authorization
  cases without mutating the global event log or creating provider writes.
- Positive pinned/WebPKI OpenAI request paths now emit
  `OPENAI_PROVIDER_CONTEXT_INJECTION_GATE` after positive request/export binding
  evidence and before API-key copy or HTTPS write. The marker binds request and
  context hashes but keeps provider write not attempted and body attachment
  false.
- `memory.recent_events` and `audit.events [limit]` expose a bounded RAM-only
  `event.log.v0` ring for current-boot agent protocol reads, denials,
  provider request-binding denials, and provider export-denial audits.
- Denied agent methods now cite current-boot `event_id` and `audit_event_id`
  values such as `event.current_boot.00000012`.
- Memory mutation methods such as `memory.record_observation`,
  `memory.propose_policy`, `memory.supersede_fact`, `memory.redact`, and
  `memory.compact` return structured `capability_denied`.
- The Shadow VM smoke validates the read-only protocol, memory context schemas,
  the local provider-minimal projection, the denied provider context export gate,
  provider export denial, event/audit log reads, memory mutation denials with
  event ids, module computed-grant hash-reference diagnostics, module
  audit/rollback hash-reference diagnostics with retained current-boot
  references, and the denied module load gate, then emits
  `raios.vm_test_report.v0` reports.

Current phase: Phase 6 has host-side computed grant plus audit/rollback
evidence diagnostics, guest-side read-only computed-grant, audit/rollback, and
service-slot reservation hash-reference diagnostics, current-boot retained
computed-grant, audit/rollback, and service-slot reservation bindings, and a
fail-closed module load gate that validates retained grant/audit references
before reporting them as non-authorizing evidence. Negative retained-reference,
audit/rollback, and service-slot reservation selftests are covered. No code
loading exists yet.

Exact next task:

```text
Integrate retained RAM-only service-slot reservation evidence into the denied
module load path.
```

Start from the retained `raios.module_service_slot_reservation.v0` event
binding. Extend the denied `raios.module_load_gate.v0` response and event
binding to revalidate that retained reservation against the retained grant and
audit/rollback chain before reporting it as non-authorizing service-slot
evidence. It must still keep `service_slot: unallocated`,
`allocates_service_slot: false`, `service_inventory_change: none`, and
`load_attempted: false`.

Next three tasks:

1. Snapshot live-validated retained service-slot reservation evidence into the
   denied module load gate.
2. Reject stale/dropped, wrong-schema, substituted, hash-mismatched,
   inventory-mismatched, and slot-mismatched retained reservations.
3. Keep `module.load_ephemeral` and `service.load_ephemeral` denied with
   loader unavailable, `service_inventory_change: none`, and
   `load_attempted: false`.

Current blockers and non-goals:

- Do not add fake persistent memory. V0 memory is `current_boot` and read-only.
- Do not send raw `system.snapshot` or boot logs to a provider.
- Do not grant module/service/config mutation before the evidence chain exists.
- Do not treat the direct OpenAI provider path as the recovery lifeline.
- Do not overwrite `release/raios-stage0.img` unless the replacement has booted
  in QEMU.

Update discipline for future agents:

- Update `Last updated`, `Current phase`, `Exact next task`, `Next three tasks`,
  and verification notes whenever work changes the project state.
- Mark work as done only with evidence: file paths, protocol methods, harness
  reports, command output, or explicit known gaps.
- Keep prose plans tied to stable IDs such as service ids, problem ids,
  capability ids, schema ids, report ids, and ADR ids.
- If a task only verifies behavior and changes no code, still update the
  verification note when that result changes what future agents should trust.

## Product Thesis

raiOS should be a tiny bootable environment whose primary interface is an
AI agent host. The OS should be small enough to understand, boot quickly in a VM,
and expose narrow, auditable capabilities to an AI provider through native
provider adapters.

This is not a Linux distribution and not a place to run the full Codex CLI in the
kernel. Codex is useful as a development tool and as a product reference; the OS
should implement its own minimal protocol surface.

## North Star Architecture

The long-term target is stronger than a small OS with a provider client. raiOS
should become an always-on core plus a live-rebuildable world:

```text
permanent core -> recovery agent lifeline -> live service graph
-> agent workspace -> shadow VM/test world -> persistence/rollback
```

The permanent core should only contain the survival mechanisms: minimal
scheduling, memory/object ownership, IPC, capabilities, service loading, crash
detection, rollback supervision, root system snapshots, and a tiny recovery
control path.

The normal OS surface should be replaceable services: UI, console, input, USB,
networking, Wi-Fi, provider adapters, diagnostics, agent tools, builder service,
and eventually driver experiments. The provider/OpenAI path is therefore a
service, not the core identity of the OS.

System memory is part of this north star. raiOS should not grow a large prompt
dump or generic RAG database. It should make the system itself the memory:
typed facts, events, decisions, problems, capability denials, service state,
test evidence, and rollback records with provenance. Agents should receive
task-scoped `agent_context.v0` packets selected by a local context broker under
token, redaction, and provider-trust budgets. See
`docs/architecture-decisions/0004-system-memory-and-agent-context.md`.

For the final system, most evolution should happen without a visible reboot:

```text
load service v2 next to v1
migrate state
switch handles
watch health
rollback to v1 if needed
persist only after tests and approval
```

If the live world crashes, the core should still be able to report a snapshot,
disable bad modules, restart last-good services, roll back persistent state, and
use a protected recovery agent lifeline. See
`docs/architecture-decisions/0003-always-on-core-and-live-rebuildable-world.md`.

## Planning Gates

The current Stage-0 code proves that direct provider access is possible, but it
does not yet prove the live-rebuildable architecture. The next planning gates are
therefore intentionally narrow:

```text
fail-closed TLS/provider trust
-> read-only agent protocol
-> typed system.snapshot.v0
-> static service.inventory.v0
-> capability policy v0
-> read-only memory.context over real typed facts
-> RAM-only event.log.v0 over reads and denials
-> module_manifest.v0
-> vm_test_report.v0
-> raios.local_attestation.v0
-> live loading remains denied until evidence matches
```

The direct OpenAI path is a normal provider-service candidate. It is not the
recovery lifeline and must not become the trusted control plane for persistence,
OTA, or recovery without the separate gates above.

## Phase 0: Bootable Visual MVP

Status: done for the current VM MVP.

Goal:

```text
UEFI -> Limine -> Rust kernel -> framebuffer overlay -> serial diagnostics
```

Done:

- Limine UEFI boot path working.
- Higher-half kernel linking fixed.
- Limine HHDM request available for kernel mappings.
- Limine framebuffer request working.
- Direct framebuffer drawing working.
- Serial diagnostics working.
- RDRAND entropy path working in the bare-metal-style VM profile.
- Chat-first double-buffered framebuffer UI with compact status for entropy,
  USB-xHCI, network, and input.
- Minimal Windows image packaging path.

## Phase 1: Minimal Agent Host UI

Goal:

```text
Boot -> status UI -> command input -> visible responses
```

Scope:

- framebuffer text UI
- serial command input (`help`, `status`, `devices`, `log`)
- optional keyboard input
- device/status model in memory
- commands: `help`, `status`, `devices`, `log`

Definition of done:

- QEMU window shows live state, not only a fixed splash.
- Serial input can request status.
- State transitions are mirrored in serial logs.

Current status: framebuffer UI, serial commands, entropy, e1000 network
bring-up, DHCP configuration, USB keyboard input, and USB mouse input are
implemented. The remaining work here is mostly UI polish and richer command
behavior.

## Phase 2: Network Visibility

Goal:

```text
e1000 visible -> DHCP attempt -> IP/DNS/gateway state shown
```

Scope:

- network status in UI
- DHCP progress and timeout states
- packet counters
- DNS stub visibility if already present in code

Definition of done:

- UI shows whether network is unavailable, probing, configured, or failed.
- Serial log gives enough data to debug without a graphical screenshot.

Current status: QEMU user-mode DHCP configures `10.0.2.15/24`, gateway
`10.0.2.2`, and DNS `10.0.2.3` locally. Packet counters, failure/timeout states,
and DNS command visibility remain.

## Phase 3: Direct Provider Transport With Trust Gate

Goal:

```text
VM agent protocol -> in-OS DNS/TCP/TLS/HTTPS -> provider API -> verified peer
```

Scope:

- tiny provider request state machine inside Stage-0
- DNS/TCP visibility for provider endpoints
- TLS/HTTPS client small enough to audit
- fail-closed certificate verification or provider/SPKI pinning
- API key entry in RAM first, stronger storage later
- every agent action maps to an explicit tool/capability

Definition of done:

- VM can submit a prompt to the provider without a host-side helper.
- The normal provider path does not use certificate verification bypass.
- Provider trust state is visible through status/snapshot output and VM smoke
  tests check for a verified or pinned TLS marker.
- The framebuffer and serial console show missing-auth, network, TLS, and
  provider errors clearly.

Current status: the host relay has been removed from the runtime path. The VM
command `ask <text>` stays in the guest and fails closed in the normal build
when provider trust is not positively verified. The default visible trust state
is `pin_config_missing`, and the Shadow VM smoke checks that problem. The first
positive verifier slice is implemented for OpenAI SPKI SHA-256 pinning: a local
image built with `-EmbedOpenAiSpkiPinFromEnv` checks the configured pin and the
TLS 1.3 P-256 ECDSA `CertificateVerify` proof before API key copy or HTTPS
write, and `openai-direct-smoke.ps1 -ExpectSpkiPinnedTrust` verifies the marker.
The earlier leaf-certificate SHA-256 pin path remains available through
`-EmbedOpenAiCertPinFromEnv` and `-ExpectPinnedTrust` for compatibility. A local
development image built with
`-AllowUnverifiedOpenAiTls` can still exercise the old unverified path for
transport debugging only. WebPKI, broader certificate algorithm support, and
redacted context projection remain the next trust hardening gates before
provider context injection, tool schemas, or capability policy can be treated as
safe.

## Phase 4: Provider Integration And Redacted Context

Goal:

```text
Prompt + redacted read-only context -> provider adapter -> response rendered in raiOS
```

Scope:

- provider config flow
- OpenAI/ChatGPT/Codex-style adapter first
- API key/pairing handled through a visible VM flow first, with persistence and
  stronger secret storage later
- rendered response in framebuffer UI
- `system.snapshot.v0` context may be attached only after TLS trust and field
  redaction are defined
- no mutating provider tools in this phase

Definition of done:

- User can boot the VM and get one AI response rendered in the OS.
- Failure modes are visible: missing auth, network unavailable, provider error.
- Snapshot fields that can leave the machine are classified as `public`,
  `local_only`, or `secret`, and provider requests include only explicitly
  allowed redacted context.

## Phase 5: Static Service Inventory And Snapshot V0

Goal:

```text
running kernel facts -> typed snapshot -> static service graph -> machine-readable system model
```

Scope:

- define which code belongs to the permanent core and which belongs to services
- expose `system.snapshot.v0`
- expose service inventory, health state, and last error per service
- model the current statically linked kernel components as services before any
  dynamic service loading
- include service id, kind, health, last error, capabilities, `replaceable`, and
  `core_owned`
- make UI/console/provider/network status consume the same structured model
- add capability names for observation and service lifecycle operations

Definition of done:

- The agent can ask what is running, what is degraded, and which capabilities
  exist without scraping human logs.
- The codebase has an explicit boundary between survival-core responsibilities
  and replaceable service responsibilities.
- Existing framebuffer and console status are derived from typed facts, not from
  a second status source.

Initial service names should be stable even while everything is still linked
into the kernel:

```text
core.boot
core.memory
core.serial
core.scheduler
core.entropy
core.snapshot_root
svc.ui.framebuffer
svc.console
svc.input
drv.usb.xhci
drv.net.e1000
svc.net.ipv4
drv.wifi.avastar_probe
svc.provider.openai_direct
```

The first agent protocol methods are read-only:

```text
system.describe
system.snapshot
system.capabilities
system.boot_log
device.graph
problem.list
service.inventory
```

Mutating methods may be documented, but they must initially return
`capability_denied` until manifest, VM-test-report, local attestation, and audit
records exist.

## Phase 5.5: Read-Only System Memory Context

Goal:

```text
typed facts -> bounded context broker -> agent_context.v0
```

Scope:

- expose `memory.profile`
- expose read-only `memory.context` over current snapshot, service inventory,
  problem list, capabilities, boot log summaries, and ADR metadata
- expose `memory.query` and `memory.trace` for included records
- enforce token profiles such as `provider_minimal`, `diagnostic`, and
  `planning`
- make summaries and semantic/RAG hits locators only, never authority
- keep all memory mutation denied until event log, audit, policy, persistence,
  and rollback records exist

Definition of done:

- The agent can ask for task-relevant context without receiving the whole memory
  store or raw logs.
- Context packets report profile, budget, included records, and omitted classes.
- Provider-bound context still obeys provider trust and redaction gates.

## Phase 5.6: RAM-Only Current-Boot Event Log

Goal:

```text
agent protocol behavior -> bounded event.log.v0 -> denial/event evidence ids
```

Status: implemented for agent protocol reads and known denials.

Scope:

- expose `memory.recent_events [limit]`
- expose `audit.events [limit]` as an alias
- record read-only protocol responses with method, capability, classification,
  outcome, and compact evidence
- record `capability_denied` outcomes for memory/module/service/config methods
- include current-boot `event_id` and `audit_event_id` in denial responses
- keep the log RAM-only, bounded, non-secret, and non-provider-exported

Definition of done:

- Shadow VM proves `event.log.v0` and `audit.event.v0` over serial.
- Denied memory and module methods cite event ids.
- No persistent memory, durable audit ledger, or provider export is implied.

## Phase 5.7: Provider-Minimal Redaction Projection

Goal:

```text
agent_context.v0 -> classified provider_minimal projection -> export still denied
```

Status: implemented as a local read-only projection.

Scope:

- mark `provider_minimal` available as a local projection in `memory.profile`
- include local `context_event_id` and `audit_event_id` handles on
  `memory.context` responses
- emit `raios.provider_context_projection.v0` for
  `memory.context provider_minimal`
- classify provider-bound fields as `public`, `local_only`, or `secret`
- include only public product/stage identity, coarse subsystem states, provider
  state markers, capability ids, service ids, stable problem metadata, and
  public record summaries in the nested projected packet
- omit raw `system.snapshot`, boot logs, local-only details, provider prompt
  text, request ids, network topology, Wi-Fi secrets, TCP diagnostics, and
  unclassified context
- keep provider export disabled with explicit blockers for provider trust and
  provider export audit binding

Definition of done:

- Shadow VM proves the projection schema, field classification, explicit
  omissions, local event ids, provider export denial, and query/trace locator.
- OpenAI requests still do not receive automatic context injection.

## Phase 5.8: Provider Context Export Gate

Goal:

```text
provider_minimal projection -> provider_context_export gate -> provider write denied
```

Status: implemented as a denied-by-default protocol gate.

Scope:

- expose `provider.context_export [provider_minimal]` and
  `provider.export_context [provider_minimal]` as provider-boundary methods
- add `cap.provider.context_export` with risk `export` and no V0 grant
- return `raios.provider_context_export.v0` with current-boot `event_id` and
  `audit_event_id`
- report provider trust state, projection presence, field-classification
  presence, packet evidence state, missing request binding, missing export
  audit binding, and `provider_write: not_attempted`
- record the denial in `event.log.v0` as `cap.provider.context_export`
- keep OpenAI requests free of automatic context attachment

Definition of done:

- Shadow VM proves the export schema, capability denial, export risk event,
  missing evidence list, and no provider write attempt.

## Phase 5.9: Provider Context Packet Evidence

Goal:

```text
provider_minimal packet -> canonical evidence hashes -> export still denied
```

Status: implemented for the local projection and denied export gate.

Scope:

- define `raios.provider_minimal.packet.canonical.v0`
- hash the canonical provider-minimal `raios.agent_context.v0` packet
- hash the exported field list separately
- hash the omitted field list separately
- expose those hashes through `raios.provider_context_projection.v0`
- expose those hashes through `raios.provider_context_export.v0`
- report packet and field-list bindings as present while provider writes remain
  `not_attempted`
- keep OpenAI requests free of automatic context attachment

Definition of done:

- Shadow VM proves the projection and export gate both expose
  `projected_packet_hash`, `exported_field_list_hash`, and
  `omitted_field_list_hash`, while request binding and export audit binding
  remain missing.

## Phase 5.10: Provider Export Denial Audit

Goal:

```text
failed provider export -> distinct denial evidence -> export gates still fail
```

Status: implemented for the denied `provider.context_export` path.

Scope:

- keep positive `raios.provider_request_binding.v0` missing until a real
  provider request envelope exists
- keep positive `raios.provider_context_export_audit_binding.v0` missing until
  structured hash-valued audit evidence exists
- emit `raios.provider_request_binding_denial.v0` for the failed binding
  attempt
- emit `raios.provider_context_export_denial_audit.v0` for the no-write export
  decision
- record separate current-boot event ids for the capability denial, request
  binding denial, and export denial audit
- mark denial-audit records with `satisfies_export_gate: false`
- carry hash-valued structured `event.log.v0` bindings on the denial events
  while keeping `satisfies_current_boot_export_gate: false`
- keep `provider_write: not_attempted` and automatic provider context injection
  disabled

Definition of done:

- Shadow VM proves the positive binding gates remain missing, denial records are
  present but cannot satisfy export gates, and the event log contains
  `provider_context_export.request_binding_denied` plus
  `provider_context_export.denial_audit` with packet/field-list hashes.

## Phase 5.11: Provider Request Envelope

Goal:

```text
real provider request path -> local pre-write envelope -> positive binding candidate
```

Status: implemented for the real direct OpenAI `ask` path.

Scope:

- create `raios.provider_request_envelope.v0` only from the real OpenAI request
  path, not from `provider.context_export`
- bind the envelope to the exact request body hash prepared for HTTPS write
- keep raw prompt text, API keys, Authorization values, and Content-Length out
  of the envelope
- keep provider-minimal context attachment blocked unless positive provider
  trust and a positive export audit binding both exist
- fail closed if envelope hashes, packet hashes, boot scope, or event retention
  do not match

Definition of done:

- Shadow VM proves `provider.context_export` does not create a fake request
  envelope.
- Direct OpenAI pin-mismatch smoke proves the envelope schema appears on a real
  provider request path, omits prompt/Content-Length/Authorization values, and
  still fails before HTTPS write on pin mismatch.
- Denied export remains denied until a positive request binding and positive
  export audit binding exist.

## Phase 5.12: Positive Provider Context Binding

Goal:

```text
provider_minimal packet hash -> real request envelope -> positive export audit binding
```

Status: implemented for local-only current-boot binding records; automatic
context injection remains disabled.

Scope:

- create `raios.provider_request_binding.v0` only for a retained current-boot
  `raios.provider_request_envelope.v0`
- bind request-envelope hash, request-body hash, provider-minimal packet hash,
  exported-field-list hash, and omitted-field-list hash
- reject denial schemas, development TLS bypass, stale or dropped event ids,
  previous-boot ids, consumed bindings, and hash mismatches
- create `raios.provider_context_export_audit_binding.v0` only after positive
  provider trust and matching request binding exist
- set `satisfies_request_binding_gate: true` only on the request binding
- set `positive_export_authorization: true` only on the export audit binding
- keep `satisfies_current_boot_export_gate: false`,
  `automatic_context_injection: disabled`, and
  `context_attached_to_provider_body: false`

Definition of done:

- Shadow VM proves standalone `provider.context_export` still cannot fake
  request envelopes or positive bindings.
- Direct OpenAI pin-mismatch smoke proves positive binding markers remain absent
  when provider trust fails.
- Direct OpenAI SPKI pinned-trust smoke proves the real `ask` path emits the
  request envelope, positive request binding, and positive export audit binding
  markers before HTTPS write.
- The OpenAI request body still does not receive automatic provider-minimal
  context.

## Phase 5.13: Checked Current-Boot Binding Consumption Gate

Goal:

```text
positive binding pair -> checked retained chain -> consumed for local gate evaluation
```

Status: implemented for local gate evaluation and negative predicate selftests;
automatic context injection remains disabled.

Scope:

- expose `provider.context_gate provider_minimal` as a read-only diagnostic
  over retained current-boot binding evidence
- validate one `raios.provider_request_binding.v0` with one matching
  `raios.provider_context_export_audit_binding.v0`
- require matching request id, request-envelope event id, request-body hash,
  request-envelope hash, request-binding hash, and provider-minimal
  packet/exported/omitted field-list hashes inside the retained binding pair
- reject development TLS bypass records, non-positive trust records, stale or
  dropped referenced events, wrong variants, already consumed pairs, and body
  attachment records
- expose `provider.context_gate_selftest provider_minimal` as local-only test
  infrastructure that exercises stale/dropped ids,
  previous-boot-or-unretained ids, substituted denial schemas, substituted
  positive records, and request/body/context hash mismatches without mutating
  global event state
- consume a valid pair once through `provider.context_export provider_minimal`
  and record `raios.provider_context_binding_consumption.v0`
- keep `satisfies_current_boot_export_gate: false`,
  `automatic_context_injection: disabled`, `provider_write: not_attempted`, and
  `context_attached_to_provider_body: false`

Definition of done:

- Shadow VM proves the read-only gate reports missing binding evidence without
  creating request envelopes or positive bindings.
- Shadow VM proves the selftest cases reject stale/dropped ids,
  previous-boot-or-unretained ids, substituted schemas, substituted positive
  records, mismatched request/body/binding/context hashes, and trust-bypass
  records while creating no provider request envelopes or positive binding
  records.
- Direct OpenAI pin-mismatch smoke proves positive binding and consumption
  remain absent when trust fails.
- Direct OpenAI SPKI pinned-trust smoke proves marker hashes match, the retained
  pair validates, the first export-gate evaluation consumes it without body
  attachment, and a second attempt is rejected as `binding_already_consumed`.

## Phase 5.14: Final Provider Context Injection Gate

Goal:

```text
checked binding evidence -> explicit injection authorization -> one request body may attach context
```

Status: fail-closed diagnostic and negative authorization selftests implemented;
no context injection is implemented in the current slice.

Scope:

- define a distinct schema for the final injection authorization, separate from
  request binding, export-audit binding, and binding consumption
- expose `provider.context_injection_gate provider_minimal` as a read-only
  diagnostic over the current gate state
- expose `provider.context_injection_gate_selftest provider_minimal` as
  local-only test infrastructure for missing, stale, substituted, body-hash
  mismatched, trust-downgraded, and unauthorized body-attachment final
  authorization candidates
- emit a blocked `OPENAI_PROVIDER_CONTEXT_INJECTION_GATE` marker on positive
  pinned/WebPKI OpenAI request paths before API-key copy or HTTPS write
- require positive provider trust, retained current-boot binding evidence,
  redaction projection hashes, single-use consumption, and a final local policy
  decision before `context_attached_to_provider_body` may become true
- evaluate the current direct OpenAI gate synchronously before HTTPS write; a
  future provider-adapter service boundary may replace that direct path after it
  has equivalent evidence and tests
- require fail-closed harness coverage for missing final authorization, stale
  final authorization, hash mismatch, trust bypass, and body attachment attempts
  without authorization
- keep raw prompt text, API keys, Authorization values, local-only network
  details, and unclassified memory out of all provider context

Definition of done:

- `context_attached_to_provider_body` becomes true only when the final injection
  gate's own schema and evidence pass.
- Direct and Shadow VM harnesses prove denied and positive paths separately.
- The request body contains only the redacted `provider_minimal` projection and
  never raw local-only or secret fields.

## Phase 6: Ephemeral Live Services

Status: started with a denied-by-default `raios.module_load_gate.v0`, a
host-side `raios.computed_capability_grant.v0` diagnostic, and a guest-side
read-only computed-grant hash-reference diagnostic. No artifact loader,
ram-only service slot allocator, durable audit ledger, rollback state, or
positive loading grant exists yet.

Goal:

```text
AI proposes artifact -> capability check -> load for current boot -> drop/kill
```

Scope:

- module/service manifest v0
- ram-only service slot
- service registry
- capability grants are computed by local policy, not self-declared by modules
- health checks and crash records
- audit log for load, start, kill, and unload
- denied-by-default behavior for missing manifest, missing grant, missing test
  report, or missing local attestation

Definition of done:

- A low-risk service can be loaded without reboot, expose one new console command
  or UI panel, then be removed without corrupting the rest of the system.
- Loading requires service inventory, manifest, computed capability grants,
  health reporting, audit records, and an explicit denial path.

## Phase 7: Hot-Swap And State Migration

Goal:

```text
service v1 keeps running -> service v2 loads -> state migrates -> handles switch
```

Scope:

- versioned service state objects
- first state migrator
- handle indirection for service clients
- atomic switch and rollback
- watchdog during the probation period after a switch

Definition of done:

- A simple service can be upgraded live while preserving its state.
- A failed upgrade rolls back to the previous service version without a full
  system restart.

## Phase 8: Recovery Agent Lifeline

Goal:

```text
live world down -> core still reports state -> AI can trigger recovery actions
```

Scope:

- tiny recovery control protocol
- separate from the normal rich provider service
- separate from the direct OpenAI chat path
- restart last-good service set
- disable bad module ids
- load recovery artifact by hash
- optional pinned minimal provider route or local physical link

Definition of done:

- If UI, provider service, or another non-core service crashes, the core can
  still expose a snapshot and accept bounded recovery commands.
- The current `svc.provider.openai_direct` path is not treated as the recovery
  lifeline unless a separate minimal recovery protocol and trust state exist.

## Phase 9: Shadow VM Acceptance

Goal:

```text
candidate artifact -> shadow boot/test -> report hash -> live/persist decision
```

Scope:

- machine-readable VM test report
- image hash, artifact hash, hardware profile, and snapshot precondition binding
- serial/protocol/screenshot predicates
- acceptance policy by risk level
- first implementation may extend the existing serial smoke test before adding
  QMP, power fault injection, or screenshot diffs

Definition of done:

- Risky service changes and all persistent changes require a matching test
  report before activation.
- The first report includes image hash, QEMU args hash, hardware profile,
  commands, predicates, result, and serial log reference.

## Phase 10: Persistence, Rollback, And Core Handoff

Goal:

```text
tested service set -> persist -> boot-success mark -> rollback or core generation handoff
```

Scope:

- image/state layout specification before implementation
- persistent service set
- last-good pointer
- safe mode that disables non-core modules and persistent writes
- boot-success marker
- rollback on crash or missing success mark
- experimental core-generation handoff for deep core updates

Definition of done:

- raiOS can persist a tested live change, recover from a bad persistent change,
  and eventually replace even core generations without a normal user-visible
  reinstall cycle.
- The current single-FAT Stage-0 image remains explicitly documented as the MVP
  layout until an A/B or DATA-backed layout is specified and tested.
