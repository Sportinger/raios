# ADR 0011: Core-Owned Genesis Shell And Bounded Personal Shell

## Status

Accepted by the raiOS owner on 2026-07-10 through the autonomous goal that
references `docs/plan-reviews/genesis-shell-execution-plan-2026-07-10.md`.

This decision authorizes only the trusted shell boundary and the exact six UI
imports below, plus execution of the checked-in personal-shell proof as
non-default `current_boot` test infrastructure. It does not authorize raw
framebuffer access, secret access, network or recovery imports, arbitrary
external shell artifacts, persistent installation, provider auto-load, or any
broader mutation authority.

## Context

The current Stage-0 framebuffer UI is a statically linked kernel renderer that
combines AI conversation, diagnostics, provider setup, WiFi setup, status,
input routing, and presentation. That is a useful bootstrap implementation but
not the final raiOS boundary.

ADR 0003 requires an always-available recovery path beneath the replaceable
world. ADR 0005 makes Wasm the first enforceable service-isolation boundary.
ADR 0008 requires each Wasm instance to receive only its evaluated import
surface. The product also needs a universal first screen from which a user can
configure AI, create a personal shell, inspect typed facts, and recover when
replaceable services fail.

The architecture therefore distinguishes a small trusted Genesis and recovery
surface from the replaceable personal UI without granting a guest the
framebuffer or letting it cover secure prompts.

## Decision

### Trusted shell boundary

The boundary is split across the permanent and protected layers. Permanent L0 owns
only minimal framebuffer/input/compositor and final-present primitives, secure
attention, and recovery-authority primitives. Protected, core-generation-owned L1
owns the Genesis/recovery presentation and typed projections. Provider, WiFi, rich AI,
and personal-shell implementations remain replaceable services.

Together the trusted `ShellHost` boundary consists of:

- the universal Genesis and recovery presentation;
- framebuffer composition and the final present operation;
- secure attention and focus transfer;
- secret, permission, recovery-confirmation, and fatal-error overlays;
- clipping and validation of personal-shell display lists; and
- projections over existing typed system, service, problem, capability,
  provider, and recovery facts.

The `ShellHost` is not a general desktop and does not become a second system
truth store. It invokes existing provider, WiFi, capability, and recovery
mechanisms through shared typed adapters. Provider output may propose actions
but never authorizes them. Offline recovery remains usable without provider or
network access.

The replaceable personal shell occupies service slot `svc.user.shell`. It owns
only its clipped personal surface and sanitized focused input. It never receives
a framebuffer pointer and cannot cover the secure strip or trusted overlays.
F12 and the trusted strip return control to Genesis before an input event can
reach the guest.

### Exact Wasm UI imports

The only imports approved for `svc.user.shell` in this decision are:

| Module | Name | Wasm signature |
| --- | --- | --- |
| `ui` | `viewport` | `() -> i64` |
| `ui` | `context_len` | `() -> i32` |
| `ui` | `context_read` | `(ptr: i32, cap: i32) -> i32` |
| `ui` | `input_len` | `() -> i32` |
| `ui` | `input_read` | `(ptr: i32, cap: i32) -> i32` |
| `ui` | `frame_submit` | `(ptr: i32, len: i32) -> i32` |

`viewport` packs unsigned logical width in the high 32 bits and height in the
low 32 bits. Context and input are immutable bounded packets staged once for a
single invocation. Reads require sufficient guest capacity and copy the whole
packet. Pointer, length, overflow, or guest-memory failures trap before host
state changes.

Each import may be called at most once per invocation. A successful invocation
submits exactly one frame. `frame_submit` first copies at most 16 KiB into host
scratch, then atomically validates at most 256 commands and 4 KiB total text.
Unknown versions or opcodes, malformed or non-UTF-8 payloads, limit violations,
and repeated submission reject the complete frame. Coordinates are clipped to
the personal surface; a rejected frame is never partially presented.

V1 uses a fresh stateless Wasm invocation per render/input event with a fixed
250,000-fuel budget. A trap, fuel exhaustion, memory failure, invalid frame, or
unexpected host call marks the personal attempt unhealthy and returns control
to Genesis. The exact packet and opcode layouts are frozen in section 6 of the
accepted execution plan.

No `secret.*`, network, raw device input, block, framebuffer, pointer, DMA,
time, provider, recovery, capability-decision, or generic host-call import is
part of this grant.

### Grant and linker enforcement

The scoped import evaluator must bind its decision to the concrete service id,
artifact SHA-256, verified descriptor-source and artifact-signature evidence,
attestation or computed-grant evidence where supplied, and the exact ordered
import-list hash. Artifact presence alone is not authority.

The evaluator authorizes the exact six-import list only for the specifically
verified `svc.user.shell` artifact and only when all six linker implementations
exist. Wrong service, wrong artifact or evidence, subset, superset, duplicate,
reorder, unknown import, or missing implementation denies before
instantiation. The authorized list and the list used to build the per-instance
wasmi linker come from the same evaluator output.

Existing services keep their current import surfaces. This decision must not
turn `policy_allows_beyond_env` into a broad global grant.

### Proof artifact

The first personal shell is a checked-in, signed proof guest that exercises the
real descriptor, signature, artifact-hash, wasmi, fuel, import-grant, lifecycle,
and fallback path. It is labeled `current_boot`,
`trust_tier: dev_key_not_owner_sealed`, and `owner_sealed: false`.

The proof may render and consume one sanitized input packet, but remains
non-default test infrastructure. It receives no external artifact bytes, is not
persistently installed, does not auto-load through a provider, and disappears
from dynamic service inventory after exit or failure. The release default
continues to report `Personal shell: not created`.

## Consequences

- Genesis and recovery remain dependable when a personal shell traps.
- A personal shell can be visibly different without receiving raw display or
  secure-input authority.
- The core retains a small compositor and trusted overlays while ordinary
  personal UI becomes replaceable, reconciling the recovery requirement with
  the services-out-of-kernel direction.
- Adding another UI import, persistent shell installation, or arbitrary
  generated-shell intake requires a later owner decision and its own evidence.

## Evidence Required Before Runtime Claims

This ADR records authority to implement the boundary; it is not evidence that
the boundary is running. Runtime claims require host validator/grant tests, a
focused Genesis VM profile proving positive rendering and fail-closed negative
cases, secure-strip overdraw resistance, F12 return, trap/fuel fallback, and a
green recovery regression. Until those reports exist, the current Stage-0 UI
remains the only implemented shell.

## Non-Goals

- Porting a general desktop, terminal, or Codex CLI into the core.
- Treating the direct provider path as the recovery lifeline.
- Giving a personal shell secrets, provider responses, raw logs, unclassified
  memory, or mutation authority.
- Claiming that the proof guest is the user's generated or installed shell.
