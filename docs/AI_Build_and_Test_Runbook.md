# AI Build & Test Runbook — Stage‑0 “Seed OS” → Stage‑1 (VM-first)

> This is a **no‑code checklist** the AI can execute step by step to bring up the seed OS in a VM, connect to the cloud, install modules/OTA over WebSocket, and validate everything with deterministic tests. Each task has clear **artifacts** and **exit criteria**. Ticks (☑) indicate completion.

---

## 0) Invariants (lock once, don’t drift)
- [x] Target: **QEMU VM** (x86_64) with **OVMF/UEFI**, devices: **virtio-net**, **virtio-input**, **virtio-rng** (locked in `docs/invariant-choices.md`).
- [x] Boot: **Limine**, handoff to **Rust** kernel (locked in `docs/invariant-choices.md`).
- [x] Display: **GOP framebuffer**, **BGRA8888**, **double buffer**, **immutable atlases**; ops: **fill, blit, present** (locked in `docs/invariant-choices.md`).
- [x] Input: **raw key/mouse** events (down/up/move/scroll, UTF‑8 text), batched every **8–16 ms** (locked in `docs/invariant-choices.md`).
- [x] Network: **DHCP + DNS**, **TLS with SPKI pin**, **WebSocket**, **JSON envelope** `{v,t,id,ts,body}` (locked in `docs/invariant-choices.md`).
- [x] Seed includes **Wasm runtime** (single active module), **full outbound net allowed**, logs mirrored to serial + WS (locked in `docs/invariant-choices.md`).
- [x] OTA: **A/B slots** on ESP, **WS‑streamed OTA** (Ed25519 signed, BLAKE3 chunk hashes), **kexec** fast handoff, rollback safe (locked in `docs/invariant-choices.md`).
- [x] Trust: **offline root → short‑lived online signer**; device auth via **token + device‑key challenge** (locked in `docs/invariant-choices.md`).
- [x] Recovery: **panic switch** to safe mode; **remote lockdown**; deterministic CI via VM harness (locked in `docs/invariant-choices.md`).

**Exit:** A one‑page “choices sheet” is stored with the build artifacts.

---

## 1) Keys, registry, and observability (day‑0)
**Goal:** Set up trust & release rails before touching the VM.

- [x] Create **offline root key** (air‑gapped) and **online signing key** (rotatable) (`keys/dev/*`).
- [x] Define **signature format** (Ed25519) and **hashes** (BLAKE3 per chunk; SHA‑256 for SPKI pins) (`ota/cli`, `ota-tools`).
- [x] Provision a **content‑addressed registry** for modules & OTA bundles (store manifest, hash, signature, metadata) (`registry/core`, CLI + tests).
- [ ] Stand up a **WS control service** with log sink. It understands: `hello`, `inventory_request/response`, `module_install/start/stop`, `ota_begin/chunk/commit`, `lockdown`, and log mirroring. (_Progress: `fake-cloud/server` serves WebSocket, handles `hello` + `ota_begin`, verifies manifests, and can publish into the registry; remaining commands/log sink pending._)

**Artifacts:** `root.pub`, `online.pub`, signer cert (online signed by root), empty registry, WS service reachable.
**Exit:** A dummy signed blob verifies end‑to‑end via the WS control plane.

---

## 2) Stage‑0 bring‑up (device)
**Goal:** Boot → pixel → DHCP/DNS → pinned‑TLS WS → hello → Wasm module can draw & talk → OTA works.

### 2.1 Boot & frame
- [x] Boot via OVMF → **Limine** → kernel (`seed-kernel`, `scripts/package-stage0.sh`, Limine vendor build staged under `release/`).
- [x] Claim GOP framebuffer; allocate backbuffer; implement `present()` (`seed-kernel/src/framebuffer.rs`).
- [x] Serial logger + on‑screen **overlay** for warn/error (`seed-kernel/src/serial.rs`, `seed-kernel/src/main.rs`, text blitter).

**Exit:** “hello pixel” overlay visible; serial logs confirm resolution & pitch (achieved by `seed-kernel/src/main.rs` hello banner + serial output).

### 2.2 Devices & time/entropy
- [ ] Init **virtio‑rng**; don’t start net until entropy healthy (RDRAND fallback). (_Progress: `seed-kernel/src/entropy.rs` seeds a boot pool via RDRAND, attaches virtio for refills, and now performs TSC-throttled top-ups; `seed-kernel/src/time.rs` calibrates TSC via PIT with bounded retries; `seed-kernel/src/main.rs` gates virtio-net boot until `entropy::is_ready()` while scheduling follow-up work; `seed-kernel/src/virtio/rng.rs` configures the legacy PCI transport + queue; `seed-kernel/src/net.rs` caches the probed virtio-net device pending driver wiring. Remaining work: real virtio-net driver and continuous refresh service._)
- [ ] Bring up **virtio‑net**; run **DHCPv4**; learn **DNS**.
- [ ] Init **virtio‑input** (kbd + pointer); timestamp events.

**Exit:** IP acquired; keyboard/mouse events observed in logs with timestamps.

### 2.3 Control channel
- [ ] Open **TLS** to cloud with **SPKI pin**; then **WebSocket**.
- [ ] Send **`hello` (minimal)** → `{device_id, seed_ver, display{w,h}, input, mac, uptime_ms}`.
- [ ] Reply to **`inventory_request`** with **rich inventory** (CPU flags, RAM, PCI/virtio features, storage layout).

**Exit:** Device appears in console; cloud shows minimal info; rich inventory retrievable on demand.

### 2.4 Wasm (single module)
- [ ] Host API (event‑driven): `on_start`, `on_input(batch)`, `on_timer(id)`, `on_ws_msg(chan,bytes)`, `on_shutdown`; calls: `fill`, `blit`, `present`, `atlas_upload/delete`, `kv_read/write/list/delete`, `ws_open/send/close`, `now_ms`, `set_timer`, `log`.
- [ ] **Install & start** a “hello‑ui” module that draws a test page and echoes input.

**Exit:** Module draws atlas assets and reacts to input in real‑time.

### 2.5 Enrollment & auth
- [ ] First‑boot **self‑enrollment**; cloud issues `device_token` bound to device key; persist to DATA.
- [ ] On WS open, perform **token + device‑key challenge** handshake.

**Exit:** Reboots reconnect automatically without user action.

### 2.6 OTA over WebSocket (A/B + kexec)
- [ ] Receive **OTA manifest** (Ed25519 by online key + attached cert from offline root).
- [ ] Stream OTA chunks (**BLAKE3** per chunk) to **inactive slot**; verify whole‑image hash.
- [ ] Mark slot **pending**; **kexec** the new kernel; Stage‑1 sets **success flag** in DATA; finalize slot.

**Exit:** OTA completes with success; rollback proven by simulating a failed boot (no success flag).

### 2.7 Safety
- [ ] Implement **panic switch** (boot flag or `/data/SAFE`) to block modules/updates.
- [ ] Implement **remote `lockdown`**: halt modules, close outbound sockets; keep control WS.

**Exit:** Panic & lockdown drills pass; logs visible in console.

---

## 3) Cloud control plane (parallel)
- [ ] Device registry + enrollment UI.
- [ ] WS server implementing the envelope & commands. (_Progress: `fake-cloud/server` WebSocket stub handles `hello` + `ota_begin`, performs manifest verification, and can publish to the registry; remaining commands, logging, and inventory flows pending._)
- [x] Signing service; **registry** for modules/OTAs (content‑addressed) (`ota/cli` binaries, deterministic keygen, and `registry/*` crates + CLI/tests).
- [ ] **Canary rollout** & auto‑revert logic.
- [ ] Basic **AI loop stub** (planner → codegen → sign → stage), even if it only ships the “hello‑ui” module first.

**Exit:** Cloud can fully manage Stage‑0 without SSH/console.

---

## 4) Stage‑1 composition (after first contact)
- [ ] Decide kernel features for this machine (e.g., **SMP**, **preemptive scheduler**, **x2APIC + TSC‑Deadline**, split kernel/**system‑server**).
- [ ] Choose driver set; bake into kernel or privileged bundles.
- [ ] Ship initial **modules** (UI shell, updater, logger, metrics).
- [ ] Optionally plan future **CBOR** migration (same semantics).

**Exit:** Device runs Stage‑1 stably with the chosen features.

---

## 5) VM self‑testing strategy (how the AI tests the VM)
**Goal:** Deterministic, automated validation without human in the loop.

### 5.1 Harness architecture
- [ ] **Orchestrator** drives QEMU via **QMP** (machine protocol) and consumes **serial logs**.
- [ ] **Fake cloud** service implements the WS/JSON contract for tests (deterministic scripts).
- [ ] **Fixtures** (per test): a manifest describing firmware, disks, net mode, SPKI pin, and the scenario timeline.
- [ ] **Golden transcripts**: expected serial & WS message sequences for each scenario.

**Exit:** One command to run an entire suite; results reported as pass/fail with logs & diffs attached.

### 5.2 Determinism & isolation
- [ ] Run QEMU in a **snapshot/ephemeral** mode; no persistent host writes.
- [ ] Pin CPU model & timers; prefer **TCG** for deterministic CI; allow **KVM** for speed locally.
- [ ] Use **user‑mode networking** for predictable DNS/ports; fixture owns host‑side port forwards.
- [ ] Seed the device RNG with **virtio‑rng**; record test seeds in reports.

**Exit:** Re‑running a test yields identical results (timestamps aside).

### 5.3 Control & fault injection (via QMP)
- [ ] **Power**: reset/power‑cycle during OTA write to test rollback.
- [ ] **Net**: drop packets, sever link, delay/delay‑jitter during WS keepalive.
- [ ] **Input**: inject key/mouse sequences; verify batching (8–16 ms).
- [ ] **Clock**: skew RTC within bounds; ensure TLS pinning unaffected.
- [ ] **Storage**: make ESP/DATA temporarily read‑error to test error paths.

**Exit:** Each fault scenario produces the expected recovery behavior and log markers.

### 5.4 Self‑test Wasm module (“bist”)
- [ ] A signed **built‑in test module** performs: atlas upload → fills/blits → frame present cadence; input echo; WS echo to fake cloud; local KV read/write; outbound net probe.
- [ ] Harness asserts visual checksum (simple hash over the framebuffer) and expected WS/log messages.

**Exit:** `bist` proves rendering, input, storage, and networking in one pass.

### 5.5 Core test suite (minimum)
- [ ] **Boot & Hello**: boot to hello; minimal inventory; keepalive pings OK.
- [ ] **Render Basics**: atlas upload, fills, blits, present without tearing.
- [ ] **Input Batching**: synthetic input bursts coalesce into 8–16 ms batches.
- [ ] **Enrollment/Auth**: first‑boot enrollment; WS challenge response; token persistence across reboot.
- [ ] **OTA Success**: OTA to slot B; kexec; success flag set; slot flip permanent.
- [ ] **OTA Rollback**: corrupt image or kill power pre‑commit; verify fallback.
- [ ] **Lockdown & Panic**: remote lockdown halts module; panic switch forces safe mode.
- [ ] **Reconnect**: kill the WS; verify backoff & buffer behavior.
- [ ] **Offline Mode**: server down; module continues to run; logs persisted and replayed later.

**Exit:** All tests green on CI with artifacts (logs, frame hashes, transcripts).

---

## 6) Deliverables checklist (what to publish each run)
- [ ] Seed OS image (ESP + DATA skeleton), versioned.
- [ ] Seed OS image (ESP + DATA skeleton), versioned. (_Progress: `scripts/package-stage0.sh` produces `release/seedos-stage0.img` with Limine + kernel ESP; DATA partition skeleton pending._)
- [ ] Keys & signer certs (public parts), SPKI pin.
- [ ] WS envelope & message type spec (Markdown).
- [ ] Module Host API spec (Markdown).
- [ ] OTA manifest schema (Markdown).
- [ ] Test fixtures & golden transcripts; CI report (HTML/Markdown).
- [ ] Release notes + rollback instructions.

**Exit:** Release is reproducible and diagnosable from artifacts alone.

---

## 7) Expansion afterwards (post‑VM)
- [ ] Small PCs (e1000 + GOP), then ARM SBC (Pi4 + simplefb), then dev phones.
- [ ] Optional: migrate payload encoding to **CBOR**; add Wi‑Fi later; GPU accel much later.
- [ ] Add canary cohorts, metrics dashboards, cost guards, and Secure/Measured Boot as the fleet grows.

---

### Notes for the AI
- Prefer **deterministic tests** first; speed later.
- Keep **the seed small** and push complexity to the cloud & modules.
- Never ship unsigned code; never accept a key you didn’t pin or certify.
