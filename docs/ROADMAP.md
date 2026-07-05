# Roadmap

This file holds direction, the capability milestones, and the compact active
cursor. It must stay under ~250 lines. Verification history, report hashes,
and per-slice evidence live in `docs/PROJECT_STATUS.md` and
`release/vm-reports/`; the full pre-restructure roadmap (1,947 lines,
including all phase definitions and archived evidence blocks) is preserved
verbatim at `docs/archive/roadmap-2026-07-04-pre-restructure.md`.

Restructured 2026-07-04 per
`docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md` and
`docs/architecture-decisions/0005-bare-metal-substrate-and-wasm-isolation.md`
(owner decisions: bare metal only, Wasm isolation first, mechanism before
vocabulary).

## Agent Handoff Cursor

Last updated: 2026-07-05.

Current milestone: **M2 Ceremony Collapse** (see Capability Milestones).

**M1 Testable Core closed 2026-07-05.** Capability sentence verified:
`raios-core` host tests pass in <1s (`cargo test --locked -p raios-core`,
9/9), and a second machine (GitHub Actions) builds AND smokes every
commit — run 28734873106 all green: host tests 15s, pinned kernel build
1m11s, headless QEMU quick profile 5m39s with report artifact. Slices:
M1-1 (`772003b`), M1-2 (`836d622`), M1-3 (`d57243b`), M1-3b
(`9db5321` + CRLF fix `943a9a0` — Windows checkout CRLF conversion broke
the signed source snapshots; forced LF).

**M0 Stabilize closed 2026-07-05.** Evidence: honest committed tree
(`0ee066e`, `9df2044`, `a6a8f56`, `e3984fb`); full profile green
(`shadow-20260704-184615-9224.json`, 7814/7814 predicates, SHA-256
`68c8e160849ca9333867ea6007013b2e49d6f39e4e7e4930b761944967ba96ee`); all
recent failures classified (failure classification log in
`docs/PROJECT_STATUS.md`); harness transport instrumentation landed —
every report now carries a `serial_transport_failure` classification
(qemu_exited / listener_missing_process_alive /
connect_timeout_listener_present), `qemu_process` teardown snapshots, and
a structured `stderr_log` block; a dead VM aborts the run immediately
instead of burning the timeout. Verified: quick profile
`shadow-20260705-094659-19752.json`, 417/417 predicates.

Done in M1 so far (2026-07-05): slice M1-1 — `raios-core` `no_std`
workspace crate with `sha256_bytes`/`sha256_hex`/`ByteSink`;
`descriptor_sources.rs` deduplicated; quick profile
`shadow-20260705-100850-5584.json` 417/417. Slice M1-2 — the pure protocol
parsers (`method_eq`, `method_head_eq`, `parse_sha256_ref`,
`parse_current_boot_event_sequence`) moved into `raios-core` with
truth-table host tests (`cargo test --locked -p raios-core`, 9/9); kernel
keeps thin wrappers/re-exports; quick profile
`shadow-20260705-101746-21240.json` 417/417. Note: the `hello_service.rs`
sha256 duplicate stays until M2 — replacing it invalidates the signed
Hello source snapshot (`artifact_content_source_sha256`); that dedup
belongs to the M2 de-hello-ify slice.

Done in M2 so far (2026-07-05): slice M2-1 — `raios-core::record` exists:
`Value` enum (Null/Bool/U64/Str/Sha256/EventSequence/Array/ordered
Object), ONE serializer `write_json` reproducing the kernel's exact JSON
style (CRLF, two-space indent, kernel escaping table), and
`sha256_of_json` implemented through a hashing `ByteSink` so serializer
and hasher cannot diverge; 14/14 host tests. Kernel untouched. Key
finding for all ports: today's kernel hashers hash `key=value` LINES,
not JSON bytes (`module_evidence.rs:4538-4592`) — every ported gate must
consciously map its old line-hash convention.

Slice M2-2 done (2026-07-05): first emitter port —
`agent_protocol_recovery_artifact_selftest_emit.rs` now builds
`raios_core::record::Value` and serializes through the new `SerialSink`
adapter (`agent_protocol_support.rs`); byte-identical output proven by
the recovery profile (`shadow-20260705-105605-12660.json`, 3644/3644,
859 selftest predicates). Net line delta -1 (the one-time SerialSink
adapter cost is now paid; subsequent ports should delete heavily).

Exact next task:

```text
Port the next batch of small recovery emit modules (~150-250 lines each,
no hash participation) onto raios_core::record via SerialSink, one
worker packet per 2-3 modules; each slice must delete more lines than it
adds and pass the recovery (or matching focused) profile byte-identical.
```

## Capability Milestones

The roadmap's backbone is now capability milestones, not schema phases. A
milestone is done when its capability sentence is true and verified, not when
its evidence is described. Denial-gate and schema-only slices do not advance
milestones (ADR 0005 §3).

### M0 Stabilize (active)

Capability sentence: "The project's own pre-commit gate is green and the
repository history is truthful again."

- Working tree committed in honest, boundary-scoped commits.
- Full Shadow VM profile green; the `audit.events 256` serial failure
  root-caused and classified (guest bug vs host transport).
- Recent real predicate failures (7005/7006, 7380/7381) explained.

### M1 Testable Core

Capability sentence: "Kernel gate/evidence logic runs and passes as ordinary
host `cargo test` in seconds, and a second machine (CI) builds and smokes
every commit."

- Extract a `no_std` library crate from the flat binary crate: types, eval
  logic, hash chains, descriptor parsing, behind a `ByteSink` trait (serial
  impl in the kernel, `Vec<u8>` impl in tests).
- Host `cargo test` coverage for gate truth tables, parse round-trips, and
  hash-chain vectors.
- Minimal CI (GitHub Actions): pinned-toolchain build, image packaging,
  headless QEMU quick profile under TCG, report uploaded as artifact.

### M2 Ceremony Collapse

Capability sentence: "The agent layer is small enough for an agent to fully
model again (~10x smaller), with byte-identical serial output proven by the
existing golden-string harness."

- One typed `Value`/record model + one JSON serializer + one canonical
  hasher over the same structure (emitter/hasher divergence becomes
  impossible).
- Port gates slice-by-slice; every porting slice deletes more lines than it
  adds; harness needles prove byte-identical output.
- Table-driven method dispatch; named `key=value` command arguments;
  shared `CommandBindings` struct replacing per-stage 25-field clones.
- De-hello-ify: `event_log.rs` and gate constants parameterized by a
  `ServiceDescriptor`; `hello_service.rs` split below the AGENTS.md size
  thresholds.
- Target: agent layer under ~20k lines; rustfmt runs clean on all sources.

### M3 First Durable Write

Capability sentence: "raiOS performs its first real, policy-authorized,
durable mutation: an audit/rollback transaction append to the
`RAIOS_AUDITRB_V0` LBA1 region — and a hello hot-swap rollback actually
applies using it."

- Grant the first narrow write authority through the existing gate chain
  (AHCI write/readback already verified; this is authority policy, not
  driver work).
- Real transaction append + readback + hash verification; rollback apply
  transitions from `capability_denied` to a real, evidenced state change
  with the transaction as its record.
- The existing denial edifice becomes a functioning transaction system.

### M4 Wasm Isolation

Capability sentence: "A service runs inside an in-kernel Wasm interpreter
and physically cannot call an authority outside its granted host-function
imports."

- Vendored, pinned `no_std` Wasm interpreter (wasmi-class, no JIT).
- Hello (or echo) compiled to wasm32, loaded as a real module artifact
  through the existing descriptor/attestation chain.
- Capability envelope = linked import surface; a deliberate
  exceed-capability test fails at the boundary, not at a policy string.

### M5 Second Service Proof

Capability sentence: "Adding `svc.demo.echo` costs only a descriptor and a
state machine — no new emitters, hash chains, or harness profiles beyond
generated needles."

This is the acceptance test that M2's refactor and M4's runtime actually
generalize. If a second service still costs tens of thousands of lines, the
architecture is not what the ADRs claim.

### M6 Promotion Loop v0

Capability sentence: "One external, AI-authored artifact travels the full
loop: authored, Shadow-VM verified, capability-granted, promoted into the
live system, and rolled back — with evidence at every step."

This is the project's first true product milestone; everything before it is
substrate.

### M7 and beyond (direction, not yet planned in detail)

Persistent image layout (GPT `SEED_ESP_A/B` + `SEED_DATA` per
`docs/image-layout-v0.md`), durable memory records (ADR 0004 Phase D),
recovery agent lifeline (ADR 0003), provider WebPKI + trusted time,
provider-agnostic adapters, bare-metal Wi-Fi.

## Active Execution Rules

Standing rules live in `AGENTS.md` (Definition of Done, Red Gate Rule,
Commit Discipline, Failure Classification, End-of-Session Checks). Compact
reminders:

- Every slice states what the system can now DO that it could not before.
- No new `raios.*.v0` schemas as hand-rolled emit/hash code; after M2, new
  schemas are record-model entries only.
- Match verification cost to slice risk exactly as before (quick often,
  full rarely, focused when the touched boundary is risky); never skip VM
  evidence for trust/storage/rollback/recovery/authority/descriptor/boot
  changes.
- Execution model: a master agent plans from this roadmap and dispatches
  worker agents with narrow, verifiable tasks split by ownership boundary
  (runtime/loader, provider trust, UI/input, VM harness, docs). Workers
  return one integrated slice with a capability sentence plus verification
  evidence. Parallel dispatch only across non-conflicting boundaries.

## Product Thesis

raiOS is a personal, self-modifying, bare-metal operating system where AI
can change the machine only through evidence-gated, capability-scoped system
transactions that can be rolled back. It is bonded to one machine and one
user, small enough for an agent to fully model, and anchored in an immutable
recovery core. It is not a Linux distribution, does not run on a Linux host
(ADR 0005), and does not port the Codex CLI into the kernel (ADR 0001).

## North Star Architecture

```text
permanent core -> recovery agent lifeline -> live service graph
-> agent workspace -> shadow VM/test world -> persistence/rollback
```

The permanent core holds only survival mechanisms. Everything else —
UI, console, input, USB, networking, provider adapters, diagnostics, agent
tools, builder — becomes a replaceable service, first as Wasm modules
(ADR 0005), long-term as a native service graph with versioned state and
migrators (ADR 0003). The provider/OpenAI path is a service, not the core
identity. The system itself is the memory: typed, classified,
provenance-bound facts feeding budgeted `agent_context.v0` packets
(ADR 0004).

## Planning Gates

Unchanged from the May 2026 plan-review consensus, now with the milestone
overlay:

```text
fail-closed TLS/provider trust        (implemented, pin-only)
-> read-only agent protocol           (implemented)
-> typed system.snapshot.v0           (implemented)
-> static service.inventory.v0        (implemented)
-> capability policy v0               (implemented)
-> read-only memory.context           (implemented)
-> RAM-only event.log.v0              (implemented)
-> module_manifest.v0                 (implemented, non-authorizing)
-> vm_test_report.v0                  (implemented)
-> raios.local_attestation.v0         (implemented, non-authorizing)
-> live loading denied until evidence matches   (M3/M4 make this real)
```

The direct OpenAI path remains a normal provider-service candidate, never
the recovery lifeline.

## Phase Map (legacy)

The former Phase 0–10 structure is retired as the planning backbone (phases
had become taxonomy, not gates — Phase 7 scope shipped inside Phase 6 while
Phase 6's own definition of done was unmet). Full phase definitions remain in
`docs/archive/roadmap-2026-07-04-pre-restructure.md`. Rough mapping: Phases
0–5.14 are the implemented substrate above; Phase 6/7 work continues inside
M3–M5; Phase 8 (recovery lifeline) and Phase 10 (persistence/core handoff)
live in M7+; Phase 9 (Shadow VM acceptance) is realized by M6.

## Documentation Ownership

- `README.md`: product thesis, quickstart, concise current reality only.
- `AGENTS.md`: stable startup checklist, standing engineering rules, durable
  current facts only.
- `docs/ROADMAP.md`: this file — direction, milestones, compact cursor.
- `docs/PROJECT_STATUS.md`: authoritative detailed status, exact next task,
  verification evidence; entries older than two weeks move to
  `docs/archive/`.
- `docs/OWNER_DASHBOARD.md`: one page, plain language, updated every
  session — current capability, gate status, top risk, next task.
- `docs/DEBUGGING.md`: commands, smoke profiles, protocol probes, failure
  modes.

## Blockers And Non-Goals

- Do not add fake persistent memory. Memory stays `current_boot` and
  read-only until M3+ persistence exists.
- Do not send raw `system.snapshot` or boot logs to a provider.
- Do not grant module/service/config mutation beyond the explicit milestone
  gates above.
- Do not add non-authorizing loader boundaries or new denial gates while
  M0–M2 are open.
- Do not treat the direct OpenAI provider path as the recovery lifeline.
- Do not overwrite `release/raios-stage0.img` unless the replacement has
  booted in QEMU.
- No work in `ota/`, `registry/`, `fake-cloud/` without a new ADR
  (ADR 0005 §4).
