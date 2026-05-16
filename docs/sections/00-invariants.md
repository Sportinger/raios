# Section Report 0 - Invariants

## Scope

Recognition of build invariants and repo layout commitments from
`docs/AI_Build_and_Test_Runbook.md`, with explicit separation between chosen
architecture and current Stage-0 evidence.

## Implemented And Verified In Stage-0

- Target VM: QEMU x86_64 with OVMF/UEFI is the verified environment.
- Boot path: Limine hands off to the higher-half Rust kernel.
- Display path: GOP/Limine framebuffer with a heap backbuffer and double-buffered
  Stage-0 UI.
- Input path: serial commands, USB-HID keyboard, USB-HID pointer/tablet, and
  PS/2 fallback feed the UI/console in the current VM profile.
- Network path: Intel e1000 plus DHCPv4 configures IP/gateway/DNS in QEMU.
- Provider path: `ask <text>` stays in the guest. The normal build fails closed
  at provider trust before API-key copy or HTTPS write; an explicit development
  image can still exercise in-guest DNS, TCP 443, TLS 1.3, HTTPS, and the
  Responses API with RAM-only API-key state.
- Logging: serial log is the authoritative implemented log sink.

## Partially Implemented Or Known Gaps

- Positive TLS verification is not implemented yet. Provider pinning or
  certificate verification is the next trust gate after the current fail-closed
  default.
- Input events exist, but the final 8-16 ms batched event protocol is not yet a
  stable external contract.
- Wi-Fi target detection and RAM-only SSID/WPA entry exist for the Surface Pro 4
  Marvell AVASTAR path, but firmware upload, association, WPA, and packet
  transport do not exist.
- OpenAI response parsing is minimal and extracts the first `output_text`.

## Chosen Or Planned Invariants

These remain architectural targets and must not be reported as implemented until
there is direct Stage-0 evidence:

- Positive provider/control trust using certificate verification or
  provider/service pinning.
- WebSocket control transport with strict JSON envelope `{v,t,id,ts,body}`.
- Single active Wasm module runtime with explicit host APIs.
- Signed module lifecycle guarded by manifest, computed capabilities, VM test
  report, local attestation, and audit records.
- A/B OTA slots, signed manifests, BLAKE3 chunk verification, kexec handoff,
  success markers, and rollback.
- Offline-root to online-signer trust chain for update/module artifacts.
- Device enrollment using token plus device-key challenge.
- Panic switch, safe mode, remote lockdown, and recovery agent lifeline.
- Remote log mirroring over the future control channel.

## Exit Evidence

- Choices sheet: `docs/invariant-choices.md`.
- Current verified state: `docs/PROJECT_STATUS.md`.
- Run/debug commands and expected serial output: `docs/DEBUGGING.md`.

Any future section report should use the labels `implemented/verified`,
`partially implemented`, `chosen/planned`, or `blocked/denied` rather than
marking future architecture as complete.
