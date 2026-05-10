# Roadmap

## Product Thesis

SeedOS/RaiOS2 should be a tiny bootable environment whose primary interface is an
AI agent host. The OS should be small enough to understand, boot quickly in a VM,
and expose narrow, auditable capabilities to an AI provider or local bridge.

This is not a Linux distribution and not a place to run the full Codex CLI in the
kernel. Codex is useful as a development tool and as a product reference; the OS
should implement its own minimal protocol surface.

## Phase 0: Bootable Visual MVP

Status: done for the current VM MVP.

Goal:

```text
UEFI -> Limine -> Rust kernel -> framebuffer overlay -> serial diagnostics
```

Done:

- Limine UEFI boot path working.
- Higher-half kernel linking fixed.
- Limine framebuffer request working.
- Direct framebuffer drawing working.
- Serial diagnostics working.
- virtio-rng detected.
- Live status rows for framebuffer, entropy, virtio-rng, virtio-net, and input.
- Minimal Windows image packaging path.

## Phase 1: Minimal Agent Host UI

Goal:

```text
Boot -> status UI -> command input -> visible responses
```

Scope:

- framebuffer text UI
- serial command input
- optional keyboard input
- device/status model in memory
- commands: `help`, `status`, `devices`, `log`

Definition of done:

- QEMU window shows live state, not only a fixed splash.
- Serial input can request status.
- State transitions are mirrored in serial logs.

## Phase 2: Network Visibility

Goal:

```text
virtio-net visible -> DHCP attempt -> IP/DNS/gateway state shown
```

Scope:

- virtio-net status in UI
- DHCP progress and timeout states
- packet counters
- DNS stub visibility if already present in code

Definition of done:

- UI shows whether network is unavailable, probing, configured, or failed.
- Serial log gives enough data to debug without a graphical screenshot.

## Phase 3: Host Bridge

Goal:

```text
VM agent protocol -> host bridge -> provider/API/CLI on host
```

Scope:

- tiny message protocol over serial, virtio-console, or user-mode TCP
- host process translates requests to development-time tools
- no secrets stored in the kernel
- every agent action maps to an explicit tool/capability

Definition of done:

- VM can ask the host bridge for a simple response.
- The bridge can be swapped later for direct HTTPS/provider adapters.

## Phase 4: Provider Integration

Goal:

```text
Prompt -> provider adapter -> response rendered in SeedOS
```

Scope:

- provider config flow
- OpenAI/ChatGPT/Codex-style adapter first
- API key/pairing handled outside the kernel at first
- rendered response in framebuffer UI

Definition of done:

- User can boot the VM and get one AI response rendered in the OS.
- Failure modes are visible: missing auth, network unavailable, provider error.

## Phase 5: Capability And Module System

Goal:

```text
AI proposes action -> capability check -> signed module/config -> test -> apply
```

Scope:

- narrow tool catalog
- signed module download/install
- module test harness
- audit log
- rollback path

Definition of done:

- AI can request a bounded change.
- The OS can deny, test, apply, and log it without arbitrary execution.
