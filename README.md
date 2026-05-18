# raisOS

<p align="center">
  <img src="docs/assets/screenshots/raios-openai-chat.png" alt="raisOS direct provider chat screen" width="920">
</p>

<p align="center">
  <strong>A personal operating system that extends itself.</strong>
</p>

raisOS turns a single computer into a bonded, self-extending environment.
Instead of installing applications, you ask for what you need — and a resident
AI builds it inside a small, fully observable system that knows only your
hardware and only you. Every change is sandboxed before it lands,
capability-gated when it runs, and atomically reversible if it misbehaves.

It is what a Lisp Machine would look like if its primary user were an AI: small
enough for an agent to fully model, writable at every layer, and anchored in an
immutable recovery core that cannot be broken from above.

## What It Is

| raisOS is | raisOS is not |
| --- | --- |
| A personal, bootable OS seed for one machine and one user. | A general-purpose Linux distribution, desktop environment, or app store. |
| A real Stage-0 kernel with framebuffer UI, serial diagnostics, USB input, e1000 DHCP, entropy, RAM-only setup, and direct provider transport. | A finished self-extending runtime with signed modules, persistence, rollback, and recovery already complete. |
| The foundation for a native agent protocol where system state is observable and future actions are capability-gated, testable, and reversible. | A port of Codex CLI into the kernel or a shell where an AI gets arbitrary host authority. |
| A small system designed to be rebuilt, tested, and personalized by an AI under local policy. | A fake cloud agent, mock provider path, silent fallback chain, or demo that pretends missing safety layers already exist. |

## Screenshots

### Console status

<p align="center">
  <img src="docs/assets/screenshots/raios-console-status.png" alt="raisOS console status screen" width="920">
</p>

The console status view exposes boot, framebuffer, entropy, USB, input, Wi-Fi,
and network state without requiring a graphical desktop or host-side helper.

### Provider and Wi-Fi setup

<p align="center">
  <img src="docs/assets/screenshots/raios-settings.png" alt="raisOS provider and Wi-Fi settings screen" width="920">
</p>

The `SET` mode is the first in-guest setup surface for provider status,
RAM-only API key entry, and early Wi-Fi SSID/passphrase capture.

### Direct provider chat

<p align="center">
  <img src="docs/assets/screenshots/raios-openai-chat.png" alt="raisOS direct OpenAI chat screen" width="920">
</p>

The chat view shows the Stage-0 direct provider path rendering a response back
inside the framebuffer UI after DNS, TCP, TLS, HTTPS, and response parsing.

## The Tamagotchi Model

Most operating systems are general-purpose. They carry decades of compatibility,
drivers for hardware you'll never own, and abstractions whose only purpose is
portability. raisOS opts out. It bonds to **one machine** and **one user**, and
trades universality for surface area you and the AI can fully reason about.

The trade pays off in three directions:

- **Less to support.** Only the hardware in the box needs drivers, schedulers,
  and quirks. There is no driver matrix, no probing fallback chain, no
  least-common-denominator path.
- **More to know.** The complete system surface fits inside an agent's working
  context. The AI reasons about your real code, not an abstract OS.
- **Sharper personalization.** Capabilities, policies, and services are
  calibrated to you. The text editor you used yesterday and the one you use
  today might be entirely different programs.

When you change machines, raisOS doesn't port — it re-binds, building a fresh
instance on new hardware while carrying forward your policies, modules, and
history.

## How It Works

raisOS is structured in three rings.

**The permanent core** is a tiny, immutable Rust kernel handed off from UEFI
through Limine. It owns boot, memory, scheduling, the framebuffer, input
devices, the recovery path, and the capability ledger. It is small enough to
audit by hand and write-protected against everything above it. If anything else
fails, the core survives.

**The agent host** runs above the core and speaks the raisOS Agent Protocol —
a typed, capability-gated interface through which an AI can read system state,
propose changes, request resources, and submit candidate services. Every tool
call is logged, scoped to declared capabilities, and refused if it exceeds
them. The host talks to AI providers through pinned-trust HTTPS over an
isolated network service, never directly from the kernel.

**Replaceable services** are everything else: networking, storage, display,
input methods, applications. Each is a signed module that runs in a constrained
capability domain. The AI can inspect them, fork them, rebuild them, and
replace them at runtime.

```mermaid
flowchart TB
    user[You]
    agent[Agent host + Agent Protocol]
    services[Replaceable services]
    core[Permanent core]
    hw[Your hardware]

    user <--> agent
    agent <--> services
    agent <-. capability ledger .-> core
    services --> core
    core --> hw
```

## Building with the AI

You ask, the agent builds. A typical interaction:

> *"I want a text editor with vim keybindings and a Markdown preview pane."*

The agent drafts a service, declares the capabilities it needs (one framebuffer
region, keyboard input, a file handle for one document), and submits the
candidate to the **Shadow VM** — a parallel execution environment that runs the
service against synthetic inputs and records evidence: syscalls made,
capabilities used, memory touched, time spent, anything reached outside the
declared scope. The recording is signed and human-readable.

If the evidence matches the declaration, the service is promoted into your live
system. If it doesn't, it never runs. Either way, the candidate, its evidence,
and its result are preserved, so promotion is auditable and rollback is one
transaction away.

Nothing the AI generates can touch the recovery core. Nothing can exceed its
declared capabilities at runtime. Nothing lands without a record.

## The Recovery Lifeline

Because the AI has write access to almost everything, the parts it *cannot*
touch matter most. The permanent core lives in a read-only region and contains:

- The boot path
- The capability ledger and policy engine
- The Shadow VM and evidence verifier
- A minimal recovery shell with serial and framebuffer console
- An immutable rollback transaction log

If a deployed service corrupts a higher layer, the core boots cleanly into the
recovery shell, replays the rollback log to the last good state, and hands
control back to the agent. The path from "the AI broke something" to "back to
working" is measured in seconds and is impossible to break from above.

## Providers and Trust

raisOS is provider-agnostic by design. The agent host can speak to any provider
that supports a typed completion API: OpenAI, Anthropic, local inference
services, or a self-hosted model. Provider trust is anchored in pinned
certificates managed through the capability ledger, not baked into the kernel
image, so rotations are an in-system transaction rather than an image rebuild.

The default build ships with no embedded credentials. Providers are provisioned
through the `SET` mode at first boot; keys live in a sealed memory region and
never appear on disk or in logs.

## Quick Start

Build a freshly bound image for the machine in front of you:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release -Image release\raisos.img
```

Boot it in a VM to try it before writing to hardware:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

Inside the running system, type `setup` to provision a provider. From there,
ask the agent for what you need.

For bare-metal installation onto the bonded machine, see `docs/BARE_METAL.md`.
The write script is destructive and requires explicit disk selection plus a
confirmation string.

## Principles

raisOS holds a small set of architectural principles that override convenience:

- **The core is small and immutable.** Everything else is replaceable.
- **Capabilities are declared and enforced.** Code that asks for more is
  refused; code that takes more without asking is impossible.
- **Evidence precedes promotion.** Candidate services run in the Shadow VM
  before they touch the live system.
- **Rollback is a first-class operation.** Every promotion is a transaction.
- **The kernel does not parse the internet.** TLS, HTTPS, and protocol parsing
  live in replaceable services with bounded capabilities.
- **The AI is a user, not an authority.** It proposes; the capability ledger
  disposes.

## Current Reality

This section is honest about what exists in the repository today versus the
vision above.

What boots and works in the VM right now:

- UEFI handoff via Limine into a Rust kernel
- Framebuffer chat UI with `AI`, `CONSOLE`, and `SET` modes
- Input from serial, USB-HID keyboard, USB-HID mouse, QEMU USB-HID tablet, and
  PS/2 fallback, with a small framebuffer cursor overlay and Tab/arrow-key
  focus ring
- Intel e1000 NIC brought up via DHCP
- Entropy seeded from RDRAND
- Direct OpenAI transport with verified DNS, TCP, TLS 1.3, HTTPS, and Responses
  API behavior
- A fail-closed provider trust gate that refuses to write HTTPS or copy the API
  key unless a valid leaf-certificate pin is configured
- `SET` mode and a `setup` command that accept an API key into a sealed RAM
  region without echoing it to the serial log
- Detection of the Surface Pro 4 Marvell AVASTAR 88W8897 Wi-Fi NIC on PCI, plus
  RAM-only SSID and passphrase capture in the settings UI

What is described above but not yet implemented:

- The Agent Protocol as a typed, capability-gated interface (the current
  transport is direct OpenAI, not a protocol)
- The capability ledger and policy engine
- The Shadow VM and evidence verifier
- Signed replaceable modules and the runtime to load and isolate them
- Persistence, the rollback transaction log, and the recovery shell as
  described
- The permanent core as a write-protected, audited boundary
- TLS and HTTPS as a replaceable service rather than kernel-resident code
- Wi-Fi firmware upload, association, WPA, and packet transport for the
  detected Marvell target
- Provider-agnostic trust beyond the first OpenAI cert-pin slice
- Re-binding to new hardware as a supported operation

The repository today is the **seed** of the system described above: a bootable
Stage-0 that proves the machine can come up, render itself, accept input, reach
the network, and talk to a provider end-to-end. The architecture above is the
direction every subsequent change is steering toward.

For the exact next task and current verified state, see
`docs/PROJECT_STATUS.md`. For the phased plan, see `docs/ROADMAP.md`. For the
foundational architecture decision, see
`docs/architecture-decisions/0001-raisos-agent-protocol.md`.
