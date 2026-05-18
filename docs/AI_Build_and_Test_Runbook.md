# AI Build & Test Runbook - Stage-0 raisOS To Stage-1

This is a no-code runbook for AI agents and humans working on the raisOS VM-first
path. It preserves the historical Stage-1 goals, but distinguishes chosen or
planned invariants from behavior that is implemented and verified in the current
Stage-0 image.

Status labels used below:

- `implemented/verified`: proven in the current Stage-0 VM path by serial logs,
  screenshots, tests, or documented local runs.
- `partially implemented`: real Stage-0 code exists, but the full invariant or
  final contract is not satisfied.
- `chosen/planned`: an architectural target, not current behavior.
- `blocked/denied`: intentionally unavailable until the required evidence or
  trust boundary exists.

## 0) Invariants And Reality Check

- [x] `implemented/verified` Target VM: QEMU x86_64 with OVMF/UEFI, Limine, Rust
  kernel, Intel e1000 networking, USB-xHCI keyboard/pointer input, and RDRAND in
  the bare-metal-style VM profile.
- [x] `implemented/verified` Display: GOP/Limine framebuffer with heap
  backbuffer and double-buffered Stage-0 UI.
- [x] `partially implemented` Input: serial, USB-HID keyboard, USB-HID pointer,
  QEMU tablet, and PS/2 fallback feed the current UI/console. The final 8-16 ms
  batched event protocol is still planned.
- [x] `partially implemented` Network/provider: DHCPv4, DNS, TCP 443, TLS 1.3,
  HTTPS, and direct OpenAI Responses API calls work in the VM path when the
  explicit unverified development override is built in.
- [x] `implemented/verified` TLS trust gate: the normal provider path fails
  closed before API-key copy or HTTPS write when trust is not verified.
- [x] `implemented/verified` First positive TLS trust slice: OpenAI
  leaf-certificate SHA-256 pinning plus TLS 1.3 P-256 ECDSA
  `CertificateVerify` proof is implemented and VM-smoked. SPKI pinning or
  WebPKI remains planned hardening.
- [ ] `chosen/planned` WebSocket control channel with JSON envelope
  `{v,t,id,ts,body}`.
- [ ] `chosen/planned` Wasm runtime with a single active module and explicit host
  APIs.
- [ ] `chosen/planned` A/B OTA, signed manifests, BLAKE3 chunk checks, kexec
  handoff, success marker, and rollback.
- [ ] `chosen/planned` Offline-root to online-signer trust chain for OTA/module
  artifacts and device-token plus device-key challenge auth.
- [ ] `chosen/planned` Safe mode, panic switch, remote lockdown, and recovery
  lifeline.

Exit: `docs/invariant-choices.md` states both the planned invariants and the
current implemented/verified Stage-0 reality.

## 1) Keys, Registry, And Observability

Goal: define release rails without implying they are active inside Stage-0.

- [x] `implemented/verified in host tools` Dev signing/key tooling and registry
  crates exist for signed artifacts and content-addressed metadata.
- [x] `implemented/verified in host tools` Signature/hash tooling exists in the
  workspace.
- [ ] `chosen/planned` A production-ready control service with complete device
  inventory, module lifecycle, OTA, lockdown, and log mirroring commands.
- [ ] `blocked/denied in Stage-0` Stage-0 does not currently consume signed
  modules or OTA bundles.

Exit: host-side tooling can validate dummy signed artifacts, but Stage-0 must not
be described as OTA/module capable until the guest runtime exists.

## 2) Stage-0 Bring-Up

Goal: boot, show state, accept input, bring up network, and make the direct
provider path explicit.

### 2.1 Boot And Framebuffer

- [x] `implemented/verified` Boot via OVMF -> Limine -> higher-half Rust kernel.
- [x] `implemented/verified` Framebuffer negotiation and backbuffered present.
- [x] `implemented/verified` Serial logger and on-screen Stage-0 UI.

Exit: QEMU shows the raisOS UI and serial logs include framebuffer readiness.

### 2.2 Devices, Time, Entropy, And Network

- [x] `implemented/verified` RDRAND entropy path in the bare-metal-style VM
  profile.
- [x] `implemented/verified` e1000 RX/TX and DHCPv4 configuration through
  smoltcp in QEMU.
- [x] `implemented/verified` DNS is learned from DHCP and used by the direct
  provider path.
- [x] `implemented/verified` USB-HID keyboard/pointer enumeration through xHCI
  in the VM profile.
- [x] `partially implemented` Surface Pro 4 Marvell AVASTAR Wi-Fi target is
  detected and RAM-only SSID/WPA settings can be recorded; firmware upload,
  association, WPA, and Wi-Fi transport are not implemented.

Exit: serial logs show entropy, USB input, e1000, DHCP lease, and input status.

### 2.3 Direct Provider Transport

- [x] `implemented/verified` RAM-only OpenAI API-key entry through `setup`/`SET`.
- [x] `implemented/verified` `ask <text>` stays inside the guest. In the normal
  build it is denied by the TLS trust gate until a valid provider pin is
  configured; with the first cert-pin verifier it checks the pinned OpenAI leaf
  certificate and TLS 1.3 signature proof before HTTPS write. With the explicit
  development override it uses DNS, TCP 443, TLS 1.3, HTTPS, and the OpenAI
  Responses API through an intentionally unverified path.
- [x] `partially implemented` Response parsing extracts the first `output_text`.
- [x] `implemented/verified` Provider trust state is visible through typed
  status/snapshot output and the Shadow VM smoke expects
  `provider.tls_pin_config_missing` by default.
- [x] `implemented/verified` Leaf-certificate provider pinning is implemented
  for the OpenAI direct path. SPKI pinning and WebPKI are still future gates.

Exit: the VM denies normal direct provider use until trust is verified. A local
pinned image can prove the normal trust path; a local development image can
still obtain a direct provider response for transport smoke testing only when
built with the explicit unverified TLS override.

### 2.4 Control Channel

- [ ] `chosen/planned` Open fail-closed TLS to a control endpoint, then a
  WebSocket transport.
- [ ] `chosen/planned` Send `hello` with device/display/input/network facts.
- [ ] `chosen/planned` Reply to inventory requests through a typed protocol.

Exit: not currently satisfied. Do not describe Stage-0 as WebSocket-controlled.

### 2.5 Wasm Runtime

- [ ] `chosen/planned` Host API for rendering, input, timers, storage, logging,
  and network/control messages.
- [ ] `blocked/denied` Module installation/start must remain denied until
  manifest, capability grants, VM test report, local attestation, and audit
  records exist.

Exit: not currently satisfied. No Wasm runtime is implemented in Stage-0.

### 2.6 Enrollment And Auth

- [ ] `chosen/planned` First-boot self-enrollment.
- [ ] `chosen/planned` Token plus device-key challenge on reconnect.
- [ ] `blocked/denied` Persistent auth state is not implemented in Stage-0.

Exit: not currently satisfied.

### 2.7 OTA, Persistence, And Rollback

- [ ] `chosen/planned` A/B slot image layout.
- [ ] `chosen/planned` Signed OTA manifest and per-chunk verification.
- [ ] `chosen/planned` Pending/success markers and rollback.
- [ ] `blocked/denied` The current Stage-0 image is a single bootable FAT image;
  no DATA partition, OTA write path, kexec handoff, or rollback path is
  implemented.

Exit: not currently satisfied. Do not imply A/B OTA or recovery exists.

### 2.8 Safety And Recovery

- [ ] `chosen/planned` Panic switch/safe mode.
- [ ] `chosen/planned` Remote lockdown.
- [ ] `chosen/planned` Recovery agent lifeline separate from the rich provider
  path.
- [ ] `blocked/denied` Current direct OpenAI chat is not a recovery control
  plane.

Exit: not currently satisfied.

## 3) Cloud Control Plane

- [ ] `chosen/planned` Device registry and enrollment UI.
- [ ] `chosen/planned` Complete WebSocket server implementing the final envelope
  and commands.
- [x] `implemented/verified in host tools` Registry/signing-related workspace
  pieces exist.
- [ ] `chosen/planned` Canary rollout and auto-revert logic.
- [ ] `chosen/planned` AI planner/codegen/sign/stage loop.

Exit: cloud-side work must not be used as evidence that guest Stage-0 can run
modules, accept OTA, or recover.

## 4) Stage-1 Composition

- [ ] Choose future kernel/system-server split and scheduling features.
- [ ] Choose final driver/service set.
- [ ] Ship initial replaceable services only after the service inventory,
  manifest, capability policy, VM-test-report, local-attestation, and audit
  gates exist.

Exit: Stage-1 composition is still future work.

## 5) VM Self-Testing Strategy

- [ ] `chosen/planned` QMP-driven orchestrator, deterministic fake control
  service, fixtures, and golden transcripts.
- [x] `implemented/verified` Current PowerShell VM runners and serial harnesses
  exercise the Stage-0 boot and direct-provider paths.
- [ ] `chosen/planned` Fault injection for OTA, WebSocket reconnects, storage
  failures, and panic/lockdown.
- [ ] `chosen/planned` Built-in self-test module. This depends on the future
  module runtime and is not implemented.

Exit: current VM tests cover the real Stage-0 path; future tests should preserve
that while adding typed reports for planned capabilities.

## 6) Deliverables Checklist

- [x] `implemented/verified` Bootable Stage-0 image:
  `release/raisos-stage0.img`.
- [x] `implemented/verified` Windows build/package/run scripts.
- [x] `implemented/verified` Documentation of current provider-key handling and
  secret scanning.
- [ ] `chosen/planned` ESP+DATA layout, A/B slots, rollback instructions, module
  host API, OTA manifest schema, complete WS envelope, and CI report artifacts.

Exit: publish current Stage-0 artifacts without presenting planned runtime,
update, or recovery capabilities as implemented.

## 7) Expansion Afterwards

- [ ] Small PCs after VM stability.
- [ ] ARM SBC and other device classes later.
- [ ] Optional CBOR payload migration later.
- [ ] Wi-Fi packet transport after firmware upload, association, and WPA work.

## Notes For Agents

- Keep the seed small, but keep the final architecture honest.
- Do not add fake module, OTA, trust, recovery, or persistence fallbacks.
- Missing evidence should produce explicit TODO, known-gap, or
  `capability_denied` behavior.
- Never describe a planned invariant as implemented unless the current Stage-0
  image, tests, and status docs prove it.
