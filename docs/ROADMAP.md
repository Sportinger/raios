# Roadmap

## Agent Handoff Cursor

Last updated: 2026-07-02 by Codex after tightening the provider-context smoke
harness around redaction, field-classification, and token-budget evidence.
Keep this section compact. The authoritative, unabridged current
state is
`docs/PROJECT_STATUS.md`; this file should describe direction and the next
cursor, not repeat the full implementation history.

Current phase: Phase 6, Ephemeral Live Services.

Active execution rule:

- keep the existing evidence gates and fail-closed posture
- stop adding loader-runtime schema-only boundaries unless they directly unblock
  the RAM-only service path
- prove the next slice with one real observable service lifecycle:
  load/start/list/stop/drop, all current-boot and non-persistent
- treat the plan as an AI-parallel OS build, not a traditional serial
  big-team roadmap: split independent agents by ownership boundary, then merge
  only real verified slices

Latest verified implementation slice:

- the provider-minimal context projection, provider request/export binding
  hashes, provider context injection gate, retained provider evidence lists, and
  quick Shadow VM predicates now include `redaction_policy_hash`,
  `field_classification_hash`, and `token_budget_hash`; no-pin/no-trust export
  remains denied and automatic context injection stays disabled
- the full Shadow VM provider-memory slice now expects all 19 provider context
  binding-gate selftest cases, including redaction/classification/budget hash
  mismatches, and the direct OpenAI smoke harness compares those hashes across
  positive request binding, export-audit binding, and blocked injection-gate
  markers when a local pinned-trust image is supplied
- `module.load_ephemeral svc.demo.hello` now loads/starts the built-in
  `svc.demo.hello` current-boot test service through a narrow RAM-only path
  that consumes `raios.current_boot_load_request.v0` and
  `raios.current_boot_load_descriptor.v0` from a validated current-image
  descriptor-source record
- `module.load_ephemeral host_bound:svc.demo.hello` loads/starts the same
  built-in RAM-only service through a host-produced descriptor-source candidate
  that binds the current-image source hash
- descriptor-source validation now parses the built-in source text into checked
  key/value fields for both current-image and host-bound sources instead of
  depending on a complete source-text equality check
- the current-image descriptor-source path now carries a repo-local
  P-256/SHA-256 signature envelope; the build script checks the checked-in
  public key/signature metadata, the kernel verifies the envelope before
  selecting the descriptor source, and load/inventory/health/RAM-audit evidence
  exposes the envelope id/hash and signature verification state
- `service.descriptor_source_trust_selftest` proves that the accepted envelope
  verifies and tampered payload, locator/kind, public-key hash, and signature
  cases fail closed without accepting descriptor or artifact bytes
- the built-in `builtin:svc.demo.hello` artifact now carries a signed
  `raios.builtin_artifact_identity.v0` identity/trust envelope; the build script
  checks the checked-in P-256 signature, the kernel validates it before load,
  and load/inventory/health/RAM-audit evidence exposes the identity id/hash,
  trust-envelope id/hash, signature verification state, and a signed
  `raios.builtin_artifact_content_binding.v0` content/hash binding for the
  checked-in Hello service source snapshot plus a signed repo-local artifact
  byte/reference hash for
  `seed-kernel/artifacts/svc.demo.hello.builtin.artifact`
- `service.artifact_reference_trust_selftest` proves that valid artifact
  reference evidence passes and tampered byte/content/reference/trust evidence
  fails closed without accepting artifact bytes or mutating the event log
- the Hello load path now emits
  `raios.current_boot_artifact_load_plan_preflight.v0`, binding the selected
  descriptor source, artifact identity, content binding, artifact reference,
  artifact bytes, and `ram_only:svc.demo.hello` service-slot intent into one
  accepted current-boot/local-only preflight hash visible in load, inventory,
  health, and RAM-audit evidence
- `service.artifact_load_plan_preflight_selftest` proves that valid preflight
  evidence passes and tampered descriptor/artifact/slot/denial evidence fails
  closed without mutating the event log
- the Hello load path now also emits
  `raios.ram_only_service_slot_activation.v0`, derived from the accepted
  preflight; load/start, inventory, health, stop/drop, and RAM-audit bindings
  expose activation id/hash/status/active state, and drop clears the current
  boot slot while citing the same activation hash
- the host-bound descriptor-source path remains hash-bound to the current-image
  source and does not accept arbitrary descriptor or artifact bytes
- `service.inventory` shows `svc.demo.hello` as healthy/running while loaded;
  `service.health svc.demo.hello` reports healthy, stopped, or missing from the
  same current-boot state; `service.stop svc.demo.hello` marks it stopped;
  `service.drop svc.demo.hello` removes it from inventory; the inventory and
  health records cite
  `load_descriptor.current_boot.svc.demo.hello.v0` plus the descriptor source
  locator/kind/validation/hash and bound source hash when present
- lifecycle and health actions retain
  `raios.ram_only_hello_service.lifecycle` and
  `raios.ram_only_hello_service.health` audit events in the current-boot RAM
  event log with descriptor and validated source-hash evidence
- the hello path accepts no arbitrary external artifact bytes, writes no
  persistent state, writes no durable audit log, installs no rollback plan, and
  grants no broad mutation
- wrong hello targets and external-looking hello targets remain on the denied
  module-load gate
- denied `module.load_ephemeral` / `service.load_ephemeral` remains the live
  policy surface for normal modules
- retained manifest, artifact, VM-test-report, local-attestation,
  local-approval, computed-grant, audit/rollback, service-slot, allocator, and
  loader-runtime evidence is current-boot, local-only, and non-authorizing
- the normal-module loader-runtime chain now reaches descriptor/artifact intake,
  execution authorization, service-registry mutation, live-load attempt,
  artifact-load, executable-mapping, entrypoint-transfer, service-start,
  service-health-binding, service-running-state, service-start-audit,
  service-unload-cleanup, live-load-commit, commit-audit, commit-rollback,
  commit-result, descriptor-acceptance authority, descriptor-parser contract,
  descriptor-parser result, descriptor schema-validation, descriptor
  capability-validation, descriptor load-plan, executable load-plan authority,
  executable load-plan result, executable image-layout, executable
  page-mapping plan, executable page-mapping, descriptor/executable-page
  binding, executable entrypoint binding, executable entrypoint transfer
  authorization, executable entrypoint transfer, executable entrypoint handoff,
  and executable entrypoint invocation boundaries
- all lifecycle boundaries report explicit non-authorizing reasons and keep
  descriptor intake, descriptor bytes, parsed descriptor production,
  validated descriptor production, descriptor schema validation, descriptor
  capability validation, capability-validated descriptor production,
  executable load-plan authority, executable load-plan production, executable
  image-layout production, executable page-mapping plan production, executable
  page mapping, capability-validated descriptor binding to executable pages,
  executable entrypoint binding, entrypoint transfer authorization, explicit
  entrypoint transfer, executable entrypoint handoff, executable entrypoint
  invocation, descriptor parsing, artifact bytes, artifact load, executable
  mapping, service start, health record creation, running-state marking,
  start-audit record writing,
  unload/cleanup, live-load commit, load-commit audit writing, commit rollback
  install, result recording, service inventory mutation, service-slot
  allocation, durable audit writes, rollback install, and load attempts false

Latest full verification:

```text
release\vm-reports\shadow-20260702-034736-23492.json
6629/6629 predicates, 243 executed commands, duration_ms: 613395
```

Latest focused verification:

```text
release\vm-reports\shadow-20260702-034303-24400.json
191/191 quick predicates, 31 executed commands, duration_ms: 60437
```

Latest focused verification after the artifact identity slice:

```text
release\vm-reports\shadow-20260702-021750-7868.json
172/172 quick predicates, 29 executed commands, duration_ms: 51268
```

Latest focused verification after the artifact content binding slice:

```text
release\vm-reports\shadow-20260702-022858-26440.json
174/174 quick predicates, 29 executed commands, duration_ms: 51671
```

Latest focused verification after the artifact byte/reference slice:

```text
release\vm-reports\shadow-20260702-023832-25068.json
177/177 quick predicates, 29 executed commands, duration_ms: 53786
```

Latest focused verification after the artifact-reference trust selftest:

```text
release\vm-reports\shadow-20260702-025252-23928.json
178/178 quick predicates, 30 executed commands, duration_ms: 53295
```

Latest focused verification after the artifact load-plan preflight:

```text
release\vm-reports\shadow-20260702-030513-27840.json
181/181 quick predicates, 30 executed commands, duration_ms: 59868
```

Latest focused verification after the artifact load-plan preflight selftest:

```text
release\vm-reports\shadow-20260702-032107-16036.json
182/182 quick predicates, 31 executed commands, duration_ms: 38186
```

Latest focused verification after the service-slot activation slice:

```text
release\vm-reports\shadow-20260702-033352-9800.json
185/185 quick predicates, 31 executed commands, duration_ms: 60174
```

Latest focused verification after the provider context hash-binding slice:

```text
release\vm-reports\shadow-20260702-034303-24400.json
191/191 quick predicates, 31 executed commands, duration_ms: 60437
```

Exact next task:

```text
Continue provider trust/context hardening. Prove positive SPKI/WebPKI TLS
provider trust plus typed request/export authorization before any provider
context export/injection can advance. Keep redaction, classification, and budget
hash evidence bound through the path; keep no-pin/no-trust and development
bypass paths fail-closed; keep candidate bytes non-executing.
```

AI-parallel next wave:

1. Provider trust/context track: harden the direct provider path toward
   SPKI/WebPKI trust and keep context injection gated by typed request/export
   authorization evidence.
2. Runtime artifact track: keep the Hello activation record green; only add
   narrow follow-ups that prove cleanup or trust evidence without executing
   candidate bytes.
3. UI/input track: improve response wrapping, scrolling, and settings controls
   while keeping UI state derived from typed system facts.
4. VM harness/evidence track: keep focused smokes fast and add predicates only
   when they prove positive behavior or necessary fail-closed denials.
5. Recovery/persistence track: keep lifeline, durable audit, rollback, and
   persistence designed from the final trust model; do not implement fake
   persistence or rollback before the evidence chain exists.

Only after provider trust/context and the live-load execution/audit/rollback
evidence chain are real should a later integration cursor consider loading
candidate bytes. Execution must stay built-in/current-boot until those gates
exist.

Documentation ownership:

- `README.md`: product thesis, quickstart, concise current reality only.
- `docs/ROADMAP.md`: phase direction and compact active cursor only.
- `docs/PROJECT_STATUS.md`: authoritative detailed status, exact next task,
  verification evidence, known gaps, and unabridged implementation history.
- `docs/DEBUGGING.md`: commands, smoke profiles, protocol probes, and failure
  modes.

Current blockers and non-goals:

- Do not add fake persistent memory. V0 memory is `current_boot` and read-only.
- Do not send raw `system.snapshot` or boot logs to a provider.
- Do not grant module/service/config mutation before the evidence chain exists.
- Do not add another non-authorizing loader boundary before the hello-service
  slice unless it is the smallest blocker for load/start/list/stop/drop.
- Do not treat the direct OpenAI provider path as the recovery lifeline.
- Do not overwrite `release/raios-stage0.img` unless the replacement has booted
  in QEMU.

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
