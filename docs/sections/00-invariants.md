# Section Report 0 – Invariants

## Scope
Recognition of build invariants and repo layout commitments per AI_Build_and_Test_Runbook.md.

## Hard invariants acknowledged
- Target VM: QEMU x86_64 with OVMF/UEFI, Intel e1000 networking, USB-xHCI keyboard/mouse, and RDRAND entropy.
- Boot path: Limine bootloader handing off to a Rust kernel.
- Display path: GOP framebuffer using BGRA8888 format, double-buffered with immutable atlases; drawing ops limited to fill/blit/present.
- Input pipeline: Raw keyboard and pointer events (down/up/move/scroll, UTF-8 text) batched every 8–16 ms.
- Networking: DHCP + DNS, TLS with SPKI pinning, WebSocket transport, strict JSON envelope `{v,t,id,ts,body}` semantics.
- Seed runtime: Single active Wasm module with outbound networking allowed, serial + remote log mirroring.
- OTA: Dual A/B slots, OTA streamed via WebSocket using Ed25519 signatures, BLAKE3 chunk hashing, kexec fast handoff, rollback safe.
- Trust chain: Offline root issuing cert for short-lived online signer; device auth via token plus device-key challenge.
- Safety: Panic switch for safe mode, remote lockdown path, deterministic CI via VM harness.

## Exit evidence
- Choices sheet committed at `docs/invariant-choices.md` documenting any clarifications/assumptions.
- Repository scaffolding will follow the structure mandated in the initial prompt.

