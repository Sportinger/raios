# Roadmap

## Product Thesis

SeedOS/RaiOS2 should be a tiny bootable environment whose primary interface is an
AI agent host. The OS should be small enough to understand, boot quickly in a VM,
and expose narrow, auditable capabilities to an AI provider through native
provider adapters.

This is not a Linux distribution and not a place to run the full Codex CLI in the
kernel. Codex is useful as a development tool and as a product reference; the OS
should implement its own minimal protocol surface.

## North Star Architecture

The long-term target is stronger than a small OS with a provider client. SeedOS
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

## Phase 3: Direct Provider Transport

Goal:

```text
VM agent protocol -> in-OS DNS/TCP/TLS/HTTPS -> provider API
```

Scope:

- tiny provider request state machine inside Stage-0
- DNS/TCP visibility for provider endpoints
- TLS/HTTPS client small enough to audit
- API key entry in RAM first, stronger storage later
- every agent action maps to an explicit tool/capability

Definition of done:

- VM can submit a prompt to the provider without a host-side helper.
- The framebuffer and serial console show missing-auth, network, TLS, and
  provider errors clearly.

Current status: the host relay has been removed from the runtime path. The VM
command `ask <text>` uses RAM-only OpenAI API key state, resolves
`api.openai.com`, opens TCP 443 through e1000, performs a TLS 1.3 handshake,
sends an HTTPS OpenAI Responses API request, and prints the first `output_text`
response. Certificate verification is still bypassed in this MVP path; HTTPS
hardening, tool schemas, and capability policy remain.

## Phase 4: Provider Integration

Goal:

```text
Prompt -> provider adapter -> response rendered in SeedOS
```

Scope:

- provider config flow
- OpenAI/ChatGPT/Codex-style adapter first
- API key/pairing handled through a visible VM flow first, with persistence and
  stronger secret storage later
- rendered response in framebuffer UI

Definition of done:

- User can boot the VM and get one AI response rendered in the OS.
- Failure modes are visible: missing auth, network unavailable, provider error.

## Phase 5: Core/World Boundary And Service Inventory

Goal:

```text
running kernel facts -> service graph -> machine-readable system model
```

Scope:

- define which code belongs to the permanent core and which belongs to services
- expose `system.snapshot.v0`
- expose service inventory, health state, and last error per service
- make UI/console/provider/network status consume the same structured model
- add capability names for observation and service lifecycle operations

Definition of done:

- The agent can ask what is running, what is degraded, and which capabilities
  exist without scraping human logs.
- The codebase has an explicit boundary between survival-core responsibilities
  and replaceable service responsibilities.

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

Definition of done:

- A low-risk service can be loaded without reboot, expose one new console command
  or UI panel, then be removed without corrupting the rest of the system.

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
- restart last-good service set
- disable bad module ids
- load recovery artifact by hash
- optional pinned minimal provider route or local physical link

Definition of done:

- If UI, provider service, or another non-core service crashes, the core can
  still expose a snapshot and accept bounded recovery commands.

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

Definition of done:

- Risky service changes and all persistent changes require a matching test
  report before activation.

## Phase 10: Persistence, Rollback, And Core Handoff

Goal:

```text
tested service set -> persist -> boot-success mark -> rollback or core generation handoff
```

Scope:

- persistent service set
- last-good pointer
- safe mode that disables non-core modules and persistent writes
- boot-success marker
- rollback on crash or missing success mark
- experimental core-generation handoff for deep core updates

Definition of done:

- SeedOS can persist a tested live change, recover from a bad persistent change,
  and eventually replace even core generations without a normal user-visible
  reinstall cycle.
