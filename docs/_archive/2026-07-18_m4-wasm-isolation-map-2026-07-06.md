# M4 Wasm Isolation — Design Map (2026-07-06)

Read-only scoping analysis (packet M4-1b): interpreter recommendation
(wasmi =0.31.2, vendored, no_std+alloc, no JIT), the artifact-chain
handoff map (where attested bytes would reach an interpreter, guarded by
the existing gate set), the capability-envelope design (the wasmi Linker
built from the computed grant IS the capability check — non-granted
imports fail at instantiation), the wasm32 guest build plan, safety
rules (heap not stack, fuel metering, non-panicking host imports), and a
6-slice plan.

**Recommendation**
Use `wasmi = "=0.31.2"` first, vendored and pinned with `default-features = false`.

License: likely dual `MIT OR Apache-2.0`. No JIT, Rust-native interpreter, intended `no_std + alloc` path. Fits `nightly-2024-10-15` unless a transitive crate raised MSRV unexpectedly. Rough vendoring: expect about 12-20 crates and a few MB of source; verify with `cargo vendor --versioned-dirs`.

Reject for M4: Wasmtime/Wasmer are too large/std/JIT-oriented; WAMR/wasm3 add C/FFI porting risk; tinywasm may be smaller but is less proven for this trust boundary.

Vendoring-time uncertainties to verify: exact 0.31.2 feature names for `no_std`, fuel API names, resource limiter API in `no_std`, transitive license set, transitive `std` leaks, panic/unsafe footprint, stack usage.

**Handoff Map**
Current state: raw artifact bytes do not become runnable guest bytes. [seed-kernel/build.rs](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:63) reads `seed-kernel/artifacts/svc.demo.hello.builtin.artifact`; [build.rs](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:131) hashes bytes into `artifact_bytes_sha256`; [descriptor_sources.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/descriptor_sources.rs:188) stores only hash/reference facts.

Key hook sites:
- [build.rs](C:/Users/admin/Documents/raios2/seed-kernel/build.rs:63): generalize artifact path/source-set/signature generation for a real `.wasm` artifact.
- [descriptor_sources.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/descriptor_sources.rs:276): artifact identity/reference flags currently say no external bytes, no code load, no executable mapping.
- [preflight.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/preflight.rs:23): preflight binds `artifact_bytes_sha256`; [preflight.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/preflight.rs:271) explicitly denies candidate execution, external bytes, candidate-byte load, and executable pages.
- [agent_protocol.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol.rs:360): dispatcher exposes loader-runtime diagnostics; [agent_protocol.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol.rs:499) still routes generic `module.load_ephemeral` to denial.
- [agent_protocol_module_loader_runtime.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_loader_runtime.rs:21): gathers retained manifest/artifact/report/attestation/grant/approval/audit/slot evidence; [same file](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_loader_runtime.rs:109) models artifact byte intake; [same file](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_loader_runtime.rs:251) models executable mapping; [same file](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_loader_runtime.rs:8950) ends at `defined_non_executable`.

Existing gates: manifest reference, candidate-artifact reference, VM report, local attestation, computed grant, local approval, audit/rollback reference, service-slot reservation, allocator authority, loader identity, artifact hash binding, entrypoint ABI, address-space/memory-map constraints, capability import table, service-slot binding, health/rollback/write-boundary hooks.

**Capability Envelope**
First imports:
- `env.log(ptr: i32, len: i32) -> ()`: read bounded guest memory, max 256 bytes, serial line only.
- `env.counter_get() -> i64`: returns a u64-shaped current-boot counter.

Import resolution is the capability check. Build a wasmi `Linker` from the computed grant. Only granted host funcs are defined. If a module imports `env.storage_write`, `env.net_send`, or even `env.log` without grant, instantiation/link fails before entrypoint. That is the enforcement boundary.

**Guest Build**
Add one tiny workspace crate, no allocator:
- target: `wasm32-unknown-unknown`
- crate type: `cdylib`
- `#![no_std]`, `panic = "abort"`
- export e.g. `raios_service_main() -> i32`
- imports only `env.log` and `env.counter_get`

Local toolchain note: `wasm32-unknown-unknown` is available for `nightly-2024-10-15` but not installed here. Install with:
`rustup target add wasm32-unknown-unknown --toolchain nightly-2024-10-15`

Generalize current `build.rs` descriptor/artifact path, source-set hash, P-256 signature envelope, and artifact reference generation. The M4 artifact bytes should be embedded as real attested Wasm bytes, then handed to wasmi as `&'static [u8]` after gates pass. Do not map them executable.

**Safety**
Keep wasmi `Module`, `Store`, linker state, guest memory, and buffers on heap/alloc, not large stack values. The prior multi-MB stack lesson applies.

Set hard limits: max module bytes, max linear memory pages, max instances/tables, max log length, max fuel per call. Enable wasmi fuel metering; fuel exhaustion is a service trap, not kernel panic.

Host functions must not panic. Validate guest ptr/len through wasmi memory APIs and return traps/errors. Guest panic should compile to `unreachable`/trap and mark service failed. Kernel panic handler halts the machine, so host import code must be boring and defensive.

**Slice Plan**
1. Vendor wasmi 0.31.2 no_std only. Verify kernel build and `cargo fmt --all -- --check`.
2. Add wasm guest crate and artifact attestation generalization. Verify wasm target build plus descriptor/hash selftests.
3. Instantiate attested module with no imports, no service mutation. Verify focused loader-runtime profile.
4. Link `env.log` and `env.counter_get` from computed grant. Verify positive log and missing-import link failure.
5. Route current-boot demo service through interpreter, inventory/health still RAM-only. Verify quick plus module-audit-rollback.
6. Harden traps: malformed wasm, bad hash, over-memory, fuel exhaustion, guest panic. Verify full profile before claiming M4.

**Risks**
Biggest risks: wasmi 0.31.2 feature drift vs `no_std`, stack/alloc pressure, accidental `std` transitive feature, host import panic, and confusing interpreter validation with native executable mapping. The lazy path is one interpreter, one demo module, two imports, one exceed-capability link failure.