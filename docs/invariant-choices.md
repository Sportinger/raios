# Invariant Choices Sheet

This sheet separates long-lived design choices from the current Stage-0
implementation state. A choice listed here is not evidence that the feature is
implemented; implemented evidence must come from `docs/PROJECT_STATUS.md`, the VM
runbook, serial logs, tests, or screenshots.

## Current Stage-0 Verified Baseline

- **VM platform**: QEMU x86_64 with OVMF/UEFI firmware is the verified target.
  The current bare-metal-style VM path uses Intel e1000 networking,
  USB-xHCI keyboard/pointer input, and RDRAND entropy.
- **Boot**: Limine hands off to the higher-half Rust kernel.
- **Framebuffer**: The kernel uses the GOP/Limine framebuffer and presents via a
  heap backbuffer. The current UI is a double-buffered framebuffer surface with
  text and a small cursor overlay.
- **Input**: Serial command input, USB-HID keyboard, USB-HID relative mouse,
  QEMU USB-HID tablet, and PS/2 fallback paths exist in the current kernel.
  Keyboard navigation drives the `AI`, `CONSOLE`, and `SET` UI modes.
- **Networking**: Intel e1000 is implemented for the QEMU path. DHCPv4 configures
  IP/gateway/DNS through smoltcp.
- **Provider transport**: `ask <text>` stays inside the guest. The normal build
  fails closed at the TLS trust gate before copying the API key or writing an
  HTTPS request. A local non-default development image can explicitly enable the
  old unverified OpenAI transport for smoke testing.
- **TLS trust state**: The implemented default is `pin_config_missing`, exposed
  through typed snapshot/problem output. Positive certificate verification or
  provider pinning is still blocked on TLS verifier input access.
- **Persistence/update/runtime**: No signed module runtime, Wasm host, A/B OTA
  slot layout, DATA partition success flag, kexec handoff, recovery agent, or
  WebSocket control plane is implemented in Stage-0.

## Chosen Or Planned Invariants

These are the intended durable architecture choices unless superseded by a later
ADR. They remain requirements or gates, not current behavior, until verified.

- **VM platform**: Keep QEMU x86_64 with OVMF/UEFI as the first-class test
  platform. Keep Intel e1000, USB-xHCI input, and RDRAND in the verified VM
  profile. Prefer deterministic timers for CI; local acceleration may be allowed
  only when tests account for the difference.
- **Framebuffer**: Keep a dedicated backbuffer and narrow drawing primitives.
  Immutable atlases remain a planned rendering boundary, but current Stage-0
  text/cursor rendering is not a complete atlas-based UI runtime.
- **Input batching**: Target an 8-16 ms input batching window for future event
  delivery. Current Stage-0 polls and routes keyboard/pointer events, but the
  final batched event contract is not yet a protocol guarantee.
- **Networking and control**: DHCPv4 and DNS are implemented for the e1000 VM
  path. The planned control plane should use fail-closed TLS trust, likely via
  certificate verification or provider/service pinning. A WebSocket overlay and
  strict JSON envelope are planned protocol choices, not current Stage-0 runtime
  behavior.
- **Provider trust**: The fail-closed trust gate is implemented. The next
  hardening gate is positive provider pinning or certificate validation. Until
  that lands, provider responses are useful only for explicitly marked
  development smoke tests and must not be treated as a trusted control plane.
- **Runtime**: A single active Wasm module with explicit host APIs remains a
  planned service-runtime shape. Stage-0 currently has no Wasm module loader and
  no generic outbound socket host API for modules.
- **OTA**: A/B slots, signed OTA manifests, chunk hashing, pending/success
  markers, kexec handoff, and rollback are planned update invariants. Stage-0
  currently uses a single bootable FAT image and does not implement OTA,
  rollback, DATA persistence, or kexec.
- **Safety controls**: Safe mode, panic switch, remote lockdown, and recovery
  lifeline are required future safety mechanisms. Current Stage-0 does not
  implement them; missing capability evidence should remain an explicit denial
  in future protocols.
- **Logging**: Serial remains the authoritative implemented log sink in Stage-0.
  Mirroring runtime logs to a remote/WebSocket channel is planned and must not be
  implied until the control plane exists and is tested.

## Documentation Rule

When updating runbooks or section reports, use explicit status labels:

- `implemented/verified`: proven by the current Stage-0 image, serial output,
  screenshot, or test.
- `partially implemented`: real code exists, but the final invariant is not yet
  satisfied.
- `chosen/planned`: architectural commitment or target behavior with no current
  Stage-0 implementation.
- `blocked/denied`: intentionally unavailable until required trust, manifest,
  test, attestation, or recovery evidence exists.
