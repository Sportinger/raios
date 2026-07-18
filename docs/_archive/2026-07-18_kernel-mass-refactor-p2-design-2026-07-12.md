# Kernel Mass Refactor P2 Implementation Design (2026-07-12)

Packet: `REFACTOR-P2-DESIGN`.

P1-A is excluded. The loader-runtime split is parked in stash
`p1-kernel-attribution-test` after the `memory-durable` wasm-import-grant probe
froze; no P2 packet below edits `agent_protocol_module_loader_runtime.rs` or
its parked child modules. The design uses the existing `raios-core` crate, not
a new crate: it already has the no-std boundary, host tests, and the proven
kernel re-export pattern.

## Contract and method

P2 moves only pure decisions, neutral data models, hashes, table definitions,
and record-value builders. Kernel code keeps hardware reads, RAM event-log
access, serial framing, provider-key/trust snapshots, durable-store access,
authority consumption, and all live dispatch entry points. The kernel still
executes every existing selftest and emits every existing byte. P2 adds a
mirrored host test for each in-guest reference-case table; P3 may remove an
in-guest duplicate later, but P2 removes none.

The kernel shim shape is deliberately boring:

```rust
// seed-kernel/src/agent_protocol_<family>.rs; the module itself remains private
pub use raios_core::<module>::{PureType, pure_function, PURE_CONSTANT};
```

For a mixed file, the shim re-exports only moved names and retains the
kernel-bound names below it. The private `mod agent_protocol_*` declarations in
`main.rs` keep the effective kernel visibility unchanged. `raios-core` exposes
only neutral types/functions needed by its own host tests and by the shim; no
API grants capability or authority.

## First three family order

| order | P2 family | inventory rows in scope | estimated kernel lines relocated | host-test gain |
| ---: | --- | --- | ---: | --- |
| 1 | `module_load_gate` / `module_types` selftest and evaluator slice | `agent_protocol_module_types.rs`, `agent_protocol_module_load_gate*.rs` except parked loader-runtime content | ~1,800 | mirrors 72+ load-gate reference cases and the module type/evaluator truth tables |
| 2 | `provider` projection and export-gate slice | `agent_protocol_provider.rs`; provider selftest behavior is mirrored, not removed | ~1,100 | tests projection allow/omit tables, canonical hashes, and fail-closed export decisions |
| 3 | `memory` request-selection and bounded-context slice | `agent_protocol_memory.rs` | ~350 | tests mutation denial, profile routing, token budgets, and argument edge cases |

The estimates are relocation estimates, not total family size. The inventory
baseline is authoritative for routing (`module_types` 3,144 lines, load-gate
selftest rows 1,204/708/1,480/1,252, provider 3,002, memory 3,263); P1-B’s
memory reflow is formatting-only and does not change this classification. The
large memory file is intentionally not promised as a wholesale move: its
binding and record emitters are kernel-bound. `event_log*`, `hello_service`,
and all `recovery*` rows are later P2/P3 work, not hidden dependencies of these
three packets.

## Family 1: module load-gate and module types

Target modules: `raios_core::module_types` and
`raios_core::module_load_gate`.

### Pure relocation

Move to `raios_core::module_types`:

- The neutral `Module*ReferenceInput`, `Module*ReferenceCheck`, and
  `Module*SelfTestCase` records at the beginning of
  `agent_protocol_module_types.rs` (manifest, artifact, VM report, local
  attestation, local approval, grant, audit/rollback, and service-slot
  decision records). Their fields are scalars, strings, optional hashes, and
  slices; they do not need AHCI, serial, or an event-log object.
- The load-gate case-count constants and fixed hash/event-id fixture strings
  that are consumed only by the load-gate reference cases. Constants naming
  loader-runtime execution facts stay out because that family is parked.
- The pure status/reason result constructors and hash formulas in
  `agent_protocol_module_load_gate_selftest_eval.rs`, after their inputs are
  expressed in core-owned neutral reference DTOs. Preserve branch order and
  exact status/reason strings.

Move to `raios_core::module_load_gate`:

- The reference-case mutation enums, `CaseSpec`-equivalent tables, valid
  fixture constructors, mutation application, and expected-result projection
  currently in `agent_protocol_module_load_gate_selftest_reference_cases.rs`.
- The retained, audit/rollback, and service-slot selftest case tables and
  pure candidate mutation/evaluation code in
  `agent_protocol_module_load_gate_selftest.rs`.
- Pure load-gate state/reason predicates in
  `agent_protocol_module_load_gate_render.rs` (the small functions that only
  map a neutral binding to a status/reason). The `emit_*` functions and their
  field ordering do not move in this slice.

The host test uses neutral `u64` event sequences and core reference structs;
the kernel adapter converts `event_log::EventId` and event-log reference
records to those DTOs and maps the core result back to the existing kernel
result structs. This avoids importing `event_log` or `AhciReadOnlyProbe` into
`raios-core` and avoids moving the event-log family prematurely.

### Kernel-bound remainder and shim

Keep in the kernel:

- `agent_protocol_module_load_gate.rs` as the dispatch-facing facade and its
  two existing re-export groups.
- `agent_protocol_module_load_gate_selftest_emit.rs`, including serial
  framing, `agent_protocol_support` macros, field ordering, and all existing
  `emit_module_load_gate_*_selftest` entry points.
- The event-log/AHCI-dependent portions of `agent_protocol_module_types.rs`:
  `EventId`-bearing facts, loader/runtime facts, service-slot allocator facts,
  and any record type whose layout directly embeds a kernel `event_log` type
  or `AhciReadOnlyProbe`.
- `agent_protocol_module_load_gate_render.rs` functions that read event-log
  bindings or call `raw`, `raw_line`, `begin_response`, `end_response`, or
  other serial/record emit helpers.

The exact facade form after the move is:

```rust
// agent_protocol_module_types.rs
pub use raios_core::module_types::{
    ModuleArtifactReferenceCheck, ModuleArtifactReferenceInput,
    ModuleArtifactSelfTestCase, ModuleGrantReferenceCheck, ModuleGrantSelfTestCase,
    ModuleLocalApprovalReferenceCheck, ModuleLocalApprovalReferenceInput,
    ModuleLocalApprovalSelfTestCase, ModuleLocalAttestationReferenceCheck,
    ModuleLocalAttestationReferenceInput, ModuleLocalAttestationSelfTestCase,
    ModuleManifestReferenceCheck, ModuleManifestReferenceInput,
    ModuleManifestSelfTestCase, ModuleVmReportReferenceCheck,
    ModuleVmReportReferenceInput, ModuleVmReportSelfTestCase,
};

// agent_protocol_module_load_gate.rs
pub use raios_core::module_load_gate::{
    evaluate_manifest_reference, evaluate_artifact_reference,
    evaluate_vm_report_reference, evaluate_local_attestation_reference,
    evaluate_local_approval_reference,
};
```

The actual list is generated from the pre-move symbol manifest; no wildcard
re-export is allowed where it could accidentally export a kernel-bound fact.
Existing `crate::agent_protocol_module_types::*` and
`crate::agent_protocol_module_load_gate::*` imports therefore continue to
resolve without changing call-site expressions.

### Host tests

The mirrored cargo tests assert, for every existing in-guest case table:

- the case count and case order are unchanged;
- each mutation selects the same first-failing status and reason;
- valid retained/reference chains remain accepted only when all event
  sequences, schema variants, hashes, and retention markers match;
- substitutions, stale or previous-boot IDs, missing records, and each
  individual hash mismatch remain denied;
- no result has an authorizing side effect, and the loader-runtime cases are
  not imported into this packet.

In-guest `module.load_gate_*_selftest` commands remain unchanged and keep
their current serial bytes. The focused VM close is
`full-module-load-gate` plus the existing `full-module-selftests` fragment for
the mirrored module selftest commands.

## Family 2: provider projection and export gate

Target module: `raios_core::provider_context`.

### Pure relocation

Move or re-express as core-neutral inputs/outputs:

- `ProjectionFieldSpec`, `PROVIDER_MINIMAL_INCLUDED_FIELDS`, and
  `PROVIDER_MINIMAL_OMITTED_FIELDS`, preserving exact order, classification,
  action, and reason strings.
- `provider_context_export_method`, `provider_context_export_profile`,
  `provider_context_export_arg`, `provider_context_block_reason`, and
  `provider_trust_positive`.
- The provider projection field-list, redaction-policy,
  field-classification, token-budget, and canonical packet hash functions.
  Dynamic status/provider/service/problem values enter through a neutral
  `ProviderProjectionInput`; the core function does not read a kernel
  snapshot.
- The public export-packet record builder and ID-membership checks, with the
  current durable-context data passed as a neutral view. The record builder
  returns `raios_core::record::Value`; the kernel remains responsible for
  serial framing.

### Kernel-bound remainder and shim

Keep in `agent_protocol_provider.rs`:

- provider, trust, Wi-Fi, UI, service-inventory, system-status, and durable
  memory snapshot collection;
- `live_provider_trust_honesty`, all `emit_provider_*` entry points, and
  serial/record rendering that consumes kernel snapshots;
- event-log gate lookup/consumption and denial-audit recording;
- provider-key, transport, and authority behavior.

The shim is:

```rust
// seed-kernel/src/agent_protocol_provider.rs
pub use raios_core::provider_context::{
    provider_context_export_method, provider_context_export_profile,
    provider_context_block_reason, provider_trust_positive,
    ProjectionFieldSpec, PROVIDER_MINIMAL_INCLUDED_FIELDS,
    PROVIDER_MINIMAL_OMITTED_FIELDS,
};
```

If a current helper’s signature contains `SystemSnapshot`, `provider::Snapshot`,
`event_log::EventId`, or a kernel-only `DurableMemoryContext`, it stays as a
thin kernel adapter and calls the core function with a copied neutral input.
No provider trust decision is moved in a way that can authorize a request or
export.

### Host tests

Mirror the provider reference tables from `agent_protocol_provider.rs` and
`event_log_provider_selftest.rs` as core tests; do not delete or alter the
in-guest tables. Assert that:

- every included and omitted field has the same order, classification, action,
  and reason;
- the canonical projection, field-list, redaction, classification, and
  budget hashes remain identical;
- unsupported profiles, missing trust, non-positive trust, stale/substituted
  binding IDs, body-hash mismatches, field-list mismatches, and trust-evidence
  mismatches remain denied;
- the positive pin-only posture still authorizes neither provider request nor
  provider export;
- the export packet contains only the public fixture IDs and never attaches a
  body without final authorization.

The focused VM close is `provider-memory`; the existing
`provider-memory-full` profile is the family integration check when the
packet changes the full projection path.

## Family 3: memory request and bounded-context logic

Target module: `raios_core::memory_context`.

### Pure relocation

Move:

- `MEMORY_MUTATION_METHODS` and `memory_mutation_method`;
- `memory_context_profile`, `memory_context_target_tokens`,
  `memory_context_estimated_tokens`, and `memory_method_arg`;
- `event_limit_arg` and the checked/saturating `parse_usize_arg` behavior;
- a small neutral `MemoryContextPlan`/budget result used by the kernel
  emitter to choose the same profile, limit, and omission posture.

These items are pure string/number policy. They do not inspect hardware,
provider state, event-log entries, recovery state, or durable records.

### Kernel-bound remainder and shim

Keep `emit_memory_profile`, `emit_memory_context`, `emit_memory_query`,
`emit_memory_trace`, `emit_recent_events`, and
`emit_memory_capability_denied` in the kernel. They consume `SystemSnapshot`,
UI runtime state, provider/recovery snapshots, `memory_store`, `event_log`,
serial writers, and binding types. Keep the binding macros and all event/value
renderers in the kernel; macro scope and byte ordering are not worth widening
for this small pure slice.

The shim is:

```rust
// seed-kernel/src/agent_protocol_memory.rs
pub use raios_core::memory_context::{
    memory_context_profile, memory_context_target_tokens,
    memory_context_estimated_tokens, memory_method_arg, event_limit_arg,
    memory_mutation_method, MemoryContextPlan, MEMORY_MUTATION_METHODS,
};
```

The module’s kernel-facing functions remain at their existing paths and keep
their existing response bytes. The core module must not expose event-log or
serial types merely to make a larger move appear possible.

### Host tests

Assert that mutation names remain denied, read profiles route identically,
unsupported/unknown profile arguments keep the existing fallback or denial,
target and estimated token budgets stay exact, event limits clamp/parse as
before, whitespace and malformed numeric arguments cannot panic, and the
plan never grants memory mutation or provider export. The existing
`memory-durable` VM profile remains the proof for real durable behavior and
the kernel’s current-boot integration.

## Packet decomposition and non-overlapping write sets

The four packets are sequential because the first packet establishes the
neutral reference vocabulary and registers all three core modules once. This
avoids putting `raios-core/src/lib.rs` in three write sets. Each worker owns
exactly the following files; generated VM reports are evidence, not hand-edited
packet files. Shared status/dashboard/roadmap documents are owned by the
orchestrator at phase close and are excluded from every worker write set.

### P2-A — core host modules and one-time registration

Write set:

- `raios-core/src/lib.rs`
- `raios-core/src/module_types.rs` (new)
- `raios-core/src/module_load_gate.rs` (new)
- `raios-core/src/provider_context.rs` (new)
- `raios-core/src/memory_context.rs` (new)

This packet owns the one-time `lib.rs` declarations for all four new modules,
but contains no seed-kernel edits. The module implementations and their
mirrored cargo tests are kept neutral and compile under the existing core
no-std boundary. P2-B, P2-C, and P2-D consume these fixed core APIs and do not
edit any P2-A file.

Checks:

- core symbol manifest records every intended public type/function/constant;
  no kernel-only type appears in a core signature;
- run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-source-size.ps1`;
- run `cargo fmt --all -- --check` and `cargo test -p raios-core`;
- verify the new modules contain no non-test `std`, hardware, serial,
  event-log, provider-key, durable-store, or authority access.

Verification tier: host-only foundation check. No VM is needed because this
packet cannot change the guest image until one of the following family packets
switches a kernel call site to the already-tested core API.

### P2-B — module load-gate selftests

Write set:

- `seed-kernel/src/agent_protocol_module_types.rs`
- `seed-kernel/src/agent_protocol_module_load_gate.rs`
- `seed-kernel/src/agent_protocol_module_load_gate_selftest.rs`
- `seed-kernel/src/agent_protocol_module_load_gate_selftest_eval.rs`
- `seed-kernel/src/agent_protocol_module_load_gate_selftest_reference_cases.rs`
- `seed-kernel/src/agent_protocol_module_load_gate_render.rs`
- `seed-kernel/src/agent_protocol_module_load_gate_selftest_emit.rs`

Checks:

- pre/post symbol manifest proves every moved function/const exists exactly
  once and every kernel-bound entry point remains callable;
- compare all existing module-load-gate selftest case counts, names, status,
  reason, and field-order constants;
- run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-source-size.ps1`;
- run `cargo fmt --all -- --check` and `cargo test -p raios-core`;
- compare affected in-guest serial responses byte-for-byte with the pinned
  pre-P2 baseline; then run `full-module-load-gate` and
  `full-module-selftests`.

Verification tier: focused module profiles at packet close; no full profile
inside the packet unless the Red Gate requires repair.

### P2-C — provider projection

Write set:

- `seed-kernel/src/agent_protocol_provider.rs`

Checks:

- symbol/const conservation for every projection table entry and every
  existing provider dispatch function;
- compare canonical packet, field-list, redaction, classification, and
  token-budget hashes and affected serial output;
- run the size checker, `cargo fmt --all -- --check`, and
  `cargo test -p raios-core`;
- run focused `provider-memory`, with `provider-memory-full` if the full
  projection response changed.

Verification tier: focused provider profile at packet close.

### P2-D — memory policy seam

Write set:

- `seed-kernel/src/agent_protocol_memory.rs`

Checks:

- function/const conservation for method routing, profiles, budgets, and
  argument parsers; no memory response entry point disappears;
- run the size checker, `cargo fmt --all -- --check`, and
  `cargo test -p raios-core`;
- compare `memory.profile`, `memory.context`, `memory.query`,
  `memory.trace`, and `memory.recent_events` serial bytes;
- run focused `memory-durable`.

Verification tier: focused memory profile at packet close.

### P2 phase close

After P2-A through P2-D, the orchestrator reads the complete diff, confirms
that all three focused reports are green, runs the `full` profile, runs the
secret scan, and performs the repository end-of-session source-size and
format checks. The full report must say `result: passed` and be newer than the
last P2 commit. P2 does not remove in-guest tables, change schemas, compact
vocabulary, or weaken any denial; those belong to later phases.

## Risks and close-reading gates

- **Trait/impl orphan rules.** A core crate cannot implement a trait for a
  kernel-owned type. Move a type and its pure impl together, or keep the impl
  in the kernel adapter. Do not make `raios-core` depend on `seed-kernel`.
- **`no_std`.** New modules compile under `#![cfg_attr(not(test), no_std)]`.
  They may use the existing `alloc` crate only where a record value requires
  it; no filesystem, threading, time, networking, or `std` imports are valid
  in non-test code. Host-only fixtures belong under `#[cfg(test)]`.
- **Macro scope.** `raw`, `raw_line`, `begin_response`, binding-field macros,
  and any textually scoped helper stay in kernel files. Moving a macro or
  widening it to `pub` is not a relocation shortcut.
- **Visibility widening.** Cross-crate moved items must be `pub` in
  `raios-core`, but the kernel shim exports only the exact old names and its
  module remains private. No kernel-only event, AHCI, provider-key, or
  authority type becomes a core public type. A wildcard re-export is rejected
  when it would expose such a type.
- **Byte identity.** Preserve literal values, table order, branch order,
  field order, and status/reason spelling. Core tests prove pure meaning;
  focused VM reports prove the existing serial bytes and codegen-sensitive
  boot behavior.
- **Uncertain module rows.** The inventory marks
  `agent_protocol_module_load_gate.rs`, the load-gate selftest support/eval/
  reference rows, and the module allocator projection row `UNCERTAIN` for
  dispatch ownership. On close reading, any helper whose caller or output
  path cannot be proved pure is routed around: leave it kernel-bound, record
  the symbol in the packet handoff, and do not infer a new export. The same
  rule applies to `agent_protocol_module_types.rs` fields containing
  `event_log::EventId` or `AhciReadOnlyProbe`.
- **Provider selftest coupling.** The provider selftest table currently builds
  an in-memory `EventLog`; only its behavior is mirrored in core in P2-C.
  Moving the event-log implementation itself is a later `event_log` family
  packet.
- **Memory classification.** The file’s apparent pure binding/record helpers
  are coupled to kernel event-binding enums and serial output. They are
  `UNCERTAIN` until a neutral input/output boundary is demonstrated; the
  design routes around them and relocates only the policy seam above.
- **Parked loader family.** Any dependency on the parked loader-runtime
  modules, including loader-runtime selftest constants or response assembly,
  stops the packet and is reported for the dedicated attribution slice.

## Out of scope noticed

No QEMU, build, source, harness, dispatch-table, `event_log`, recovery, hello,
P3 deletion, P4 vocabulary, schema, authority, persistence, or loader-runtime
change is part of this design packet. Existing untracked release preview
artifacts are unrelated and must remain untouched. The required implementation
worker checks are specified above but are not run by this design-only packet.
