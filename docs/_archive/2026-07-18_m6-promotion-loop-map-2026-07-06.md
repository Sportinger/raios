# M6 Promotion Loop v0 — Design Map (2026-07-06)

Read-only scoping analysis (packet M6-1): the stage-by-stage
exists/missing map (authored-externally / shadow-vm-verified /
capability-granted / promoted-live / rolled-back) with file:line, the
external-authorship analysis (the two-phase candidate→identity path the
denial edifice was already built for), the promotion + rollback
transaction design (reuse the M3 durable append/readback/inspect), the
smallest honest v0 scope, and the verdict: M6 is large — split into
sub-milestones M6A (external candidate identity), M6B (verified grant),
M6C (promotion), M6D (rollback); 6-10 focused slices. The hard part is
turning the hash-reference denial chain into a real authority chain
without weakening the trust model — not wasm execution, which exists.

**Stage Map**
| M6 stage | Exists today | Missing for M6 |
|---|---|---|
| `authored-externally` | Echo is a real Wasm current-boot service: descriptor/artifact identity at `seed-kernel/src/echo_service.rs:21`, load/start/health/stop/drop at `echo_service.rs:285`, `333`, `385`, `426`, `461`; Wasm validation/execution through `wasmi` at `seed-kernel/src/wasm_runtime.rs:57`, `157`, `210`. | It is not external. `build.rs` verifies repo-local P-256 signatures and embeds bytes via generated constants: `seed-kernel/build.rs:405`, `420`, `594`. Runtime explicitly denies external bytes/load/persistence/rollback install: `echo_service.rs:186`, `580`; build metadata also says `accepts_external_artifact_bytes=false`: `build.rs:457`, `509`. |
| `shadow-vm-verified` | Harness emits `raios.vm_test_report.v0`: `vm-harness/shadow-vm-smoke-support.ps1:652`, written at `:724`; latest report schema/result are in `release/vm-reports/shadow-20260706-073633-23460.json:2`, `:3`. Gate model already retains VM report refs: `seed-kernel/src/agent_protocol_module_reference.rs:225`, `469`; load gate binds latest refs at `seed-kernel/src/event_log.rs:4073`. | Current module evidence is still hash-reference/diagnostic, not a candidate-specific authorizing VM result. The full-module-evidence profile even uses synthetic hashes such as `333...`: `vm-harness/shadow-vm-smoke-profile-full-module-evidence.ps1:81`. No path accepts report JSON or uses it to authorize load. |
| `capability-granted` | Computed grant/local approval objects exist as retained evidence. Grant diagnostic records a valid hash ref but keeps load denied: `seed-kernel/src/agent_protocol_module_grant.rs:74`, `128`, `228`. Load gate tracks approval and grant refs: `seed-kernel/src/agent_protocol_module_load_gate_render.rs:220`, `2026`. | No positive grant path exists. Diagnostics still say `accepts_artifact_bytes=false`, `artifact_loaded=false`, `service_started=false`, `load_attempted=false`: `agent_protocol_module_grant.rs:92`, `145`. |
| `promoted-live` | Built-ins can load live through explicit handlers. `module.load_ephemeral` dispatches hello/echo positive paths before falling back to denial: `seed-kernel/src/agent_protocol.rs:492`, `506`; echo updates service inventory on load/start: `echo_service.rs:285`, `333`. | Generic candidate promotion is denied. Final load gate emits `capability_denied`: `seed-kernel/src/agent_protocol_module_load_gate_render.rs:4647`; loader runtime readiness has `accepts_artifact_bytes=false`, `loads_artifact=false`, `allocates_service_slot=false`, `can_load_now=false`: `:3159`. |
| `rolled-back` | M3 rollback-apply is real for hello. It verifies LBA1 `RAIOS_AUDITRB_V0` append/readback/inspection: `raios-core/src/scoped_rollback_apply.rs:16`, `261`, `563`, `753`; hello rollback mutates state back after verified apply: `seed-kernel/src/hello_service/state_machine.rs:435`, `520`. | It is hello-scoped. Constants require `svc.demo.hello`: `scoped_rollback_apply.rs:16`. No generic “un-promote external module” rollback payload, service-slot free, or inventory restore exists. |

**External Authorship**
Today’s trust chain is build-time: `build.rs` reads descriptor/artifact/signature files, verifies P-256, computes hashes, then emits Rust constants and `include_bytes!`: `seed-kernel/build.rs:420`, `434`, `487`, `594`. Runtime validates those embedded constants and hard-denies external intake: `seed-kernel/src/descriptor_sources.rs:738`, `865`, `891`.

For M6, an external `.wasm` needs identity after build. There are two honest designs:

1. Runtime intake plus non-build signature authority: accept bounded artifact bytes, compute SHA-256 in guest, parse/validate Wasm with `wasmi::Module::new`, then verify a runtime promotion authority signature over manifest hash, artifact hash, VM report hash, grant, and rollback plan. This creates an external identity without pretending the build key signed it.

2. Two-phase candidate then identity, preferred: first retain external bytes as an inert current-boot candidate, then Shadow-VM verifies the exact hash, then local attestation/grant/approval creates the runtime identity. This matches the existing gate chain better: `module_manifest.v0`, `candidate_artifact_sha256`, `vm_test_report.v0`, `local_attestation.v0`, computed grant, approval, audit, rollback, slot.

The denial edifice was built for option 2. The fields to flip are already visible: hash-only retained refs in `event_log.rs:5388` through `:5506`; audit/rollback “not durable/not installed” at `agent_protocol_module_load_gate_render.rs:158`; service slot allocator `defined_non_authorizing` at `:1761`; loader runtime false fields at `:3159`; final `artifact_loaded=false`, `service_started=false`, `can_load=false` at `:4713`.

**Promotion Transaction**
Promotion should be a durable transaction, not just a RAM mutation. The transaction should bind exact candidate hash, manifest hash, VM report hash, local attestation hash, grant hash, approval, service id, import set, slot id, previous live state, and rollback plan.

Reuse the M3 pattern: append to `RAIOS_AUDITRB_V0`, read back, inspect, then mutate live state only after the evidence passes. The existing mechanism already proves append/readback/inspection for a scoped rollback apply: `raios-core/src/scoped_rollback_apply.rs:563`; the M6 work is to generalize the evaluator beyond hello and define promotion as the authority record for live service graph mutation.

**Rollback**
For v0, rollback can be “un-promote”: stop service, drop the RAM-only candidate instance, remove inventory entry, free the slot, and record the verified rollback transaction. If promotion replaced an existing service later, rollback restores the previous descriptor/state instead.

This reuses the M3 shape but needs a generic target. Current hello rollback restores previous generation/state/descriptor at `seed-kernel/src/hello_service/state_machine.rs:520`; M6 needs the same verified-apply discipline for an external service slot, not hello’s private state machine.

**Smallest Honest V0**
Yes, echo Wasm can be the first “external artifact” only if the promoted bytes arrive at runtime from outside the kernel image. Same Wasm behavior is fine; embedded `include_bytes!` echo is not.

Minimal new machinery:

- bounded runtime candidate byte intake into RAM only
- guest-computed artifact hash and `wasmi::Module::new` validation
- candidate-specific Shadow VM report binding to that exact hash
- local runtime attestation or promotion-authority signature, not build-time P-256
- positive computed grant for the tiny import set already used by echo: `env.log`, `env.counter_get` at `seed-kernel/src/wasm_runtime.rs:521`
- positive current-boot service slot allocator
- promotion transaction append/readback/inspection
- live instantiate from retained candidate bytes
- rollback transaction that un-promotes and removes inventory

No persistent artifact store is required for v0 if the status says `current_boot`. No fake persistence, no fake provider, no “loaded” status before the durable promotion evidence exists.

**Slice Plan**
1. M6A candidate intake: accept bounded external Wasm bytes as inert RAM candidate; compute hash; deny load. Verification: unit/host tests plus focused module-gate VM later, no full profile for this slice alone.

2. M6A Shadow binding: make `-ArtifactPath` drive a candidate-specific `raios.vm_test_report.v0` hash and guest-retained report ref. Verification: focused Shadow profile.

3. M6B local attestation: verify a non-build promotion authority or local attestation over manifest/artifact/report. Verification: host crypto tests and focused gate profile.

4. M6B computed grant: turn exact import/capability grant from diagnostic into authorizing evidence while still denying load until audit/rollback/slot exist. Verification: focused gate profile.

5. M6C service slot allocator: allocate one current-boot external Wasm slot only when full retained evidence is complete. Verification: focused VM, assert no execution yet if promotion tx absent.

6. M6C promotion transaction plus live start: append/readback/inspect promotion record, instantiate Wasm, update inventory/health. Verification: focused M6 promotion profile.

7. M6D rollback transaction plus un-promote: append/readback/inspect rollback record, stop/drop service, remove inventory, free slot. Verification: focused M6 rollback profile, then full profile before claiming M6.

**Verdict**
M6 is large. It should be split into sub-milestones: M6A external candidate identity, M6B verified grant, M6C promotion, M6D rollback. The hardest parts are not Wasm execution, which exists, but turning the current hash-reference denial chain into a real authority chain without weakening the trust model. Honest estimate: 6-10 focused implementation slices, with multiple focused VM profiles and one final full Shadow profile before M6 can be called green.