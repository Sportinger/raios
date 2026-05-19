# Roadmap

## Agent Handoff Cursor

Last updated: 2026-05-19 by Codex after implementing the RAM-only current-boot
event/audit log, recording agent protocol reads and denials, updating protocol
docs and the Shadow VM harness, and running the Shadow VM smoke.

Latest maintenance verification:

- `git diff --check` passed.
- `cargo fmt --all -- --check` passed.
- `cargo test --locked -p ota-tools -p registry-core -p registry-tools -p fake-cloud-server`
  passed.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-seed-kernel.ps1 -Profile release`
  passed.
- `powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1`
  passed and wrote
  `release\vm-reports\shadow-20260519-104330-25636.json` with 80/80
  predicates.

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
  `service.inventory`.
- Mutating or potentially mutating methods currently return structured
  `capability_denied` until manifest, VM test report, local attestation,
  computed grant, approval, audit, and rollback evidence exist.
- `system.snapshot.v0`, `system.capabilities.v0`, `problem.list.v0`,
  `service.inventory.v0`, provider trust docs, module manifest docs, VM test
  report docs, local attestation docs, and recovery protocol docs exist.
- `memory.profile`, `memory.context`, `memory.query`, and `memory.trace` exist
  as local read-only `current_boot` methods. `memory.context` emits
  `raios.agent_context.v0`; provider export remains disabled.
- `memory.recent_events` and `audit.events [limit]` expose a bounded RAM-only
  `event.log.v0` ring for current-boot agent protocol reads and denials.
- Denied agent methods now cite current-boot `event_id` and `audit_event_id`
  values such as `event.current_boot.00000012`.
- Memory mutation methods such as `memory.record_observation`,
  `memory.propose_policy`, `memory.supersede_fact`, `memory.redact`, and
  `memory.compact` return structured `capability_denied`.
- The Shadow VM smoke validates the read-only protocol, memory context schemas,
  provider export denial, event/audit log reads, memory mutation denials with
  event ids, and the denied module load path, then emits
  `raios.vm_test_report.v0` reports.

Current phase: Phase 5.6 is implemented as the first RAM-only current-boot
event/audit foundation. The next durable architecture step is a
`provider_minimal` redaction projection for `raios.agent_context.v0`.

Exact next task:

```text
Define and implement the provider_minimal redaction projection for
raios.agent_context.v0, keeping provider export disabled unless provider trust
is positive and the outbound projection is audit-bound.
```

Start with explicit field classification for the context packet, then emit a
local-only projection that states what would be allowed or omitted for provider
export. Do not attach it to provider requests until positive provider trust and
event/audit binding both exist.

Next three tasks:

1. Specify the `provider_minimal` projection fields for
   `raios.agent_context.v0`, including public/local-only/secret treatment and
   explicit omissions.
2. Implement local read-only projection output and Shadow VM assertions proving
   that provider export remains disabled while trust is not positive.
3. Add event/audit binding for any future provider-bound context export, still
   without sending context automatically to OpenAI.

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
-> local_attestation.v0
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

## Phase 6: Ephemeral Live Services

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
