# M5 Second Service Proof — Design Map (2026-07-06)

Read-only scoping analysis (packet M5-1): what a real current-boot
service needs (hello as reference, file:line), the generalizes-vs-still-
hardcoded table, the recommendation (extract a shared current_boot_service
shell driven by ServiceDescriptor — NOT a hello copy, which would be
19k+ lines and fail M5), the attestation interaction (moving shared code
out of hello re-signs hello; echo needs a service/load descriptor), a
5-slice plan, and the honest verdict: ~900-1,800 net lines if
generalization succeeds; a copy is a hard fail.

**Hello Surface Today**
- Dispatch: positive `service.*`/`module.load_ephemeral` routes are hardwired to `hello_service::*`; generic routes deny after that. `seed-kernel/src/agent_protocol.rs:329-333`, `491-522`.
- Descriptor: `ServiceDescriptor` exists but only as `HELLO_SERVICE_DESCRIPTOR`; it names ids, aliases, capabilities, slot ids, inventory fields, event kinds. `seed-kernel/src/hello_service/constants.rs:3-83`.
- Runtime state: hello keeps `loaded`, `running`, `generation`, `state_counter`, migration/probation/applied rollback, last event ids. `seed-kernel/src/hello_service/runtime.rs:13-104`.
- Target matching: load/start/health/stop/restart/drop only match hello descriptor/aliases. `seed-kernel/src/hello_service/command_targets.rs:3-18`, `121-145`.
- Lifecycle: load/start/restart/hot-swap/stop/drop/health emit audit events and mutate RAM state. `seed-kernel/src/hello_service/state_machine.rs:16-63`, `65-171`, `274-400`.
- Inventory: static services are generic, but current-boot loaded service projection is hello-only. `seed-kernel/src/service_inventory.rs:47-160`; `seed-kernel/src/agent_protocol_system.rs:620-659`, `664-808`.
- Event log: lifecycle/health/rollback records are `record_hello_*`, use `HELLO_SERVICE_DESCRIPTOR`, and bind `EventBindings::HelloServiceLifecycle`. `seed-kernel/src/event_log.rs:4429-4555`.
- Binding shape: `HelloServiceLifecycleBinding` embeds descriptor/artifact attestation plus hello state/probation/rollback fields. `seed-kernel/src/event_log_types.rs:143-245`; event enum at `3717-3720`.
- Audit emitter: event-log binding emitter is named and schema-mapped as hello. `seed-kernel/src/agent_protocol_memory.rs:643-701`, `1776-1827`.
- Attestation: hello load descriptor/source + artifact identity/reference chain is hardcoded in `descriptor_sources.rs`; validation only knows hello. `seed-kernel/src/descriptor_sources.rs:10-35`, `217-345`, `347-385`, `865-919`.

**Generalizes Vs Hardcoded**
| Area | Generalizes | Still Hardcoded |
|---|---|---|
| Method table | Table-driven dispatch exists | Positive service routes point only to `hello_service`; fallback denies echo |
| Descriptor | `ServiceDescriptor` carries reusable ids/caps/status strings | Type lives in `hello_service/constants.rs`; only one const instance |
| State machine | State fields are mostly generic lifecycle fields | Type/function names and state hashes are hello-specific |
| Inventory | Static inventory loop is generic | loaded current-boot service append is `emit_hello_service_inventory` |
| Event log | Ring/event model is generic | `record_hello_*`, `HelloServiceLifecycleBinding`, hello evidence arrays |
| Emitters | Record-model helpers exist | lifecycle binding schemas/keys use `raios.ram_only_hello_service.*` |
| Descriptor/artifact trust | P-256 verification helpers exist | descriptor/artifact record lookup validates hello only |
| Wasm | Echo already validates/runs inside wasmi envelope | `wasm.echo_probe` is read-only probe, not a service lifecycle mutation |

**Recommendation**
Use path (a): extract a shared `current_boot_service` shell driven by a service descriptor and a small state machine. Do not copy `hello_service/`.

A parallel echo copy is already ~19,156 lines before descriptor-source, event-log, inventory, dispatch, and harness edits. That fails M5.

Echo is not “just the wasm run.” The real service is a native lifecycle shell whose `start` invokes the wasm envelope. The shell owns `loaded/running/generation/run_count/last_run/evidence`, writes lifecycle audit events, and appears in `service.inventory`; the wasm module is the service body executed under `env.log` + `env.counter_get`.

Minimal echo surface: `module.load_ephemeral svc.demo.echo`, `service.start`, `service.health`, `service.inventory`, `service.stop`, `service.restart`, `service.drop`. Defer hot-swap/rollback for echo unless M5 explicitly requires it.

Estimated recommended line cost: ~900-1,800 net plus generated quick-profile needles. If it exceeds ~5k for this minimal surface, M5 should be marked failed: the architecture still has not generalized beyond hello.

**Attestation Notes**
- Echo already has artifact identity descriptor/signature and wasm bytes: `seed-kernel/descriptors/svc.demo.echo.wasm_artifact_identity.desc:1-28`; build verifies signature/hash at `seed-kernel/build.rs:401-505`.
- That descriptor is identity/non-authorizing: it says `executes_artifact=false`, `links_imports=false`, `maps_executable_pages=false`. For M5, keep it identity-only and add a service/load-plan record that authorizes current-boot wasm execution under the fixed import set.
- If echo’s existing artifact identity descriptor fields change, it must be re-signed and `build.rs` assertions updated.
- If echo gets a hello-style `current_boot_load_descriptor`, that new descriptor needs its own P-256 signature. Existing echo wasm artifact signature does not need re-signing unless bytes/identity fields change.
- Extracting generic shell code used by hello means hello’s signed source-set in `build.rs:5-28` must include the moved code, which implies re-signing hello v1/v2 descriptors.

**Slice Plan**
1. Scoping/generic shell: move `ServiceDescriptor`/minimal `ServiceState`/target matching into shared current-boot service code; keep hello output unchanged. Verify with `cargo fmt --all -- --check` + host tests if any pure logic moves.
2. Event-log generalization: rename/add generic service lifecycle binding and generic `record_service_lifecycle/health`; hello uses it, echo can use it. Verify quick profile.
3. Echo descriptor/load plan: add echo service descriptor and load descriptor binding to existing echo artifact identity; no artifact identity re-sign unless fields change. Verify quick profile.
4. Echo wasm start path: split `wasm.echo_probe` positive run into reusable `run_echo_service`; service start records run result/fuel/log evidence and mutates inventory/event log. Verify quick profile with generated echo needles.
5. Lifecycle completion: add echo stop/restart/drop/health inventory assertions; keep rollback/hot-swap denied or absent for echo. Verify quick profile, then full profile before claiming M5.

**Verdict**
M5 is plausible only if the next work deletes/generalizes hello-specific lifecycle plumbing. A second-service copy is a hard fail. The architecture has a reusable descriptor idea, record-model emitters, and a real wasm envelope, but the live positive service path is still hello-hardcoded at dispatch, inventory, event-log binding, descriptor lookup, capability table, and audit schemas.