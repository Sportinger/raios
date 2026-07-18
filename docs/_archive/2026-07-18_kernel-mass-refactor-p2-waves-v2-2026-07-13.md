# P2 Relocation-Wave Redesign — Current Tree

**Packet:** `WAVE-REDESIGN`  
**Measured tree:** current main after `3b70b9b` recovery-family retirement and `7de058a` routing correction  
**Current `seed-kernel/src`:** **176,346 physical Rust lines**  
**Program target:** **≤120,000 lines with zero capability regression**  
**Scope:** design only; no files changed

## 1. Corrections to the stale plan

The existing `kernel-mass-refactor-p2-waves-2026-07-13.md` must not be executed as written.

1. The `agent_protocol_recovery*` family is already gone. Commit `3b70b9b` removed 45,576 lines from the repository; no P2 wave should plan that deletion again.
2. The module write-boundary and hello rollback authority chain are **RELOCATE**, not RETIRE. Commit `7de058a` explicitly corrected that route.
3. Wave 1 already relocated substantial module-load-gate, provider-context, and memory-context logic into `raios-core`.
4. `agent_protocol_module_load_gate_selftest_eval.rs` already converts kernel snapshots into `raios_core::module_load_gate` inputs and calls the core evaluators. P2 must delete remaining duplicate evaluators/tables and narrow adapters; it must not create a second core load gate.
5. The former `~206k → ~106k` trajectory is obsolete. The relevant starting point is **176,346**.
6. P2 remains byte-identical relocation. Any reduction requiring changed serial vocabulary, reordered fields, renamed reasons, relaxed event-ID grammar, or updated goldens belongs to P4.

## 2. Measured family inventory

Counts are physical Rust source lines from the current tree.

| Family | Current kernel lines | What `raios-core` already hosts | Actual remaining P2 delta |
|---|---:|---|---|
| Module reference/approval/attestation/grant/audit/service-slot | **7,847** | `module_load_gate.rs` already has reference DTOs, ordered evaluators, canonical hashes, service-slot validation, and host tests. `module_types.rs` already has check/input/selftest records and counts. `promotion_attestation.rs` verifies the promotion signature. | Remove kernel duplicate parsing-independent checks, hash functions, mutation tables, valid-case constructors, and exhaustive guest matrices. Retain live event lookup, signature-envelope byte acquisition, serial dispatch, and final emission. |
| Service-slot allocator, projection, loader identity/artifact/facts | **8,467** | Generic `record::Value`; load-gate service-slot records/evaluation; neutral module types. No allocator or loader-fact core module yet. | Relocate pure snapshot evaluators, prerequisite/authority tables, source-evidence checks, projection builders, identity/hash-binding checks, and loader-fact chains. |
| Loader runtime | **11,058** | Load-gate reference chain and module types only. No loader-runtime evaluator exists in core. | Relocate pure snapshot/evaluation state machine, ordered denials, evidence projections, hashes, and reference matrices. Keep byte acquisition, executable mapping, entrypoint transfer, service registry mutation, and event-log access in the kernel. |
| `event_log*` | **14,865** | `record::Value` supports `EventSequence`; `parse_current_boot_event_sequence` already parses event IDs. No core ring/event model exists. | Relocate surviving event types, fixed-capacity ring mechanics, event-ID validation, window selection, bindings, pure evidence checks, and selftest tables. Keep the global lock and runtime snapshot acquisition in kernel. |
| `agent_protocol_memory.rs` | **11,272** | `memory_context.rs` already owns method/profile parsing, token budgets, mutation classification, limits, and `MemoryContextPlan`. `memory_record*` owns typed memory records/resolution. | Do not repeat planning/budget logic. Relocate surviving event-to-record projections, classification/omission tables, trace/recent-event selection, response `Value` builders, and reference fixtures. |
| `agent_protocol_provider.rs` | **2,438** | `provider_context.rs` already owns public/omitted field tables, trust posture, packet construction, hashes, redaction/classification/budget hashes, and host tests. `scoped_provider_export.rs` owns the export authority decision. | Delete duplicate projection and hash paths; move only any remaining pure denial/audit record builders into existing core modules. Keep live provider/Wi-Fi/trust/key collection, transport, dispatch, and serial framing in kernel. |
| `module_evidence.rs` | **4,592** | Load-gate consumes the reference chain, but the retained evidence store/model remains kernel-side. | Relocate immutable evidence records, selection/matching functions, reference hashes, and pure retained-chain evaluation. Keep the current-boot store, lock, insertion, and snapshot acquisition in kernel. |
| `agent_protocol_module_types.rs` | **2,702** | `raios-core::module_types` already contains the neutral load-gate records and constants. Kernel file already re-exports part of it. | Move remaining neutral DTOs/constants beside their owning core modules. Reduce the kernel file to re-exports plus genuinely kernel-owned snapshot handles. |
| Load-gate render/selftest family, excluding 37-line re-export facade | **10,795** | `module_load_gate.rs` already contains the real evaluators, canonical hashes, and host tests. Kernel `selftest_eval` already transliterates and calls them. | Port remaining reference-case tables and paper-trace fixtures to Cargo tests; replace exhaustive guest output with thin integration sanity. Relocate pure `Value` projections where byte-identical. Keep serial sink/framing and runtime snapshot conversion. |
| Hello service surviving model, including rollback | **21,106** | `scoped_rollback_apply.rs` already owns scoped apply, authorized append, verified apply, retained-inspection checks, constants, and seven host tests. Other scoped append modules already own durable append authority. | Split pure lifecycle/rollback evaluators, record builders, hashes, and test matrices from actual state acquisition, media I/O, durable append, scoped apply wiring, and service state mutation. |
| Module write-boundary family | **11,130** | Existing scoped append evaluators cover real durable record families, but there is no core module for the diagnostic write-boundary snapshot chain. | Relocate pure snapshot evaluators and their tables as one authority-foundation wave. Keep storage discovery, RECLOG/media snapshots, append calls, event retention, and dispatch kernel-side. |

These rows overlap architecturally but not in the stated line counts. In particular, hello rollback is counted inside the hello family, not the module write-boundary family.

## 3. Relocation invariants

Every wave must preserve these rules:

- No method, schema, field, status, reason, ordering, hash input, or capability result changes.
- The kernel continues to acquire mutable/global/hardware state.
- Core evaluators receive immutable snapshots and return decisions or typed values.
- Core code never writes media, mutates the event ring, changes service state, maps executable pages, or transfers control.
- Kernel wrappers must be thin conversions, dispatch, scoped-apply wiring, and serial framing.
- Exhaustive case tables move to host `cargo test`; one or a few guest sanity predicates retain integration proof.
- P2 goldens remain byte-identical. A required golden update stops that wave and reroutes the change to P4.

### Transliteration rule

For every selftest table moved from guest code:

1. **Quote-original:** capture the exact current case name, mutation, expected status/reason, ordered first failure, and hash input.
2. **Paper-trace:** manually follow the current kernel evaluator and write the expected decision without consulting the proposed port.
3. **Transliterate:** encode that same case against the core DTO/evaluator.
4. Compare kernel and core decisions before deleting the original table.

This follows the `3e7a090` correction: semantic resemblance is insufficient.

Hash grammars must remain literal:

- Preserve every `"name="` separator and field order.
- Preserve absent/null spellings and exact scalar formatting.
- Event IDs remain `event.current_boot.` followed by exactly eight decimal digits.
- Do not normalize event IDs, accept variable-width suffixes, or substitute numeric-only hashes.
- Do not derive a new “cleaner” hash grammar during relocation.

## 4. Ordered P2 waves

### P2-W2 — Finish the existing module-load-gate relocation

**Kernel source:** reference, approval, attestation, grant, audit, and service-slot files; load-gate reference cases and `selftest_eval`.

**Pure kernel-side logic still present:**

- Duplicate parse-independent reference matching and ordered denial checks.
- Valid-input constructors and mutation application.
- `*_CASES` tables for manifest, artifact, VM report, attestation, approval, grant, audit/rollback, and service slot.
- Duplicate canonical hash wrappers.
- Pure conversion of checks into bindings and response records.

**Entanglement:**

- Parsing and live event lookup are interleaved with evaluation.
- Several functions both build a `Value` and emit it.
- Attestation evaluation is mixed with acquisition/decoding of signature-envelope bytes.
- Guest selftests mix snapshot fabrication, mutation, evaluation, and serial rendering.

**Mechanics:**

- Extend existing `raios_core::module_load_gate`; do not create `module_reference`.
- Extend `raios_core::module_types` only for DTOs actually shared by the existing evaluator.
- Keep one kernel snapshot-conversion adapter per live method.
- Convert emit functions to call core `Value` builders only where output remains byte-identical.
- Port every remaining reference table into the existing `module_load_gate` test module.

**Estimated net kernel reduction:** **6,000 lines** after approximately 300–500 lines of wrappers/sanity checks.

**Focused VM:** `module-audit-rollback`, which loads both `full-module-evidence` and the load-gate coverage used by the full composition.

**Selftests ported:** all manifest/artifact/report/attestation/approval/grant/audit/service-slot tables, including first-failure ordering and hash substitution cases.

**Guest proof retained:** valid chain, substituted hash denial, signature denial, and service-slot mismatch through the kernel adapter.

**Golden dependency:** none. Any golden change reroutes the offending rendering cleanup to P4.

**Trajectory:** **176,346 → 170,346**

---

### P2-W3 — Retained module evidence and remaining module DTOs

**Kernel source:** `module_evidence.rs`, remaining neutral content in `agent_protocol_module_types.rs`.

**Pure kernel-side logic still present:**

- Evidence record DTOs and immutable binding records.
- Retained-chain selection and matching.
- Reference hash construction.
- Present/missing/stale/substituted evidence classification.
- Pure snapshot projections.

**Entanglement:**

- The evidence store and its lock live beside immutable record logic.
- Event IDs and insertion order are acquired during mutation but later consumed as neutral scalar inputs.
- Some response builders reach directly into the global store.

**Mechanics:**

- Add `raios_core::module_evidence`.
- Move immutable evidence records, selectors, matchers, and hashes there.
- Extend `module_types` only for truly common records; place family-specific types in `module_evidence`.
- Kernel keeps the store, lock, append/retain operations, and converts its snapshot into a core slice.
- Reduce `agent_protocol_module_types.rs` to re-exports and kernel-only handles.

**Estimated net kernel reduction:** **4,800 lines**.

**Focused VM:** `module-audit-rollback`.

**Selftests ported:** retained chain, missing predecessor, stale/current-boot mismatch, substituted ID/hash, wrong schema/provenance, and ordered lookup cases.

**Guest proof retained:** one retained valid chain and one stale/substituted rejection.

**Golden dependency:** none.

**Trajectory:** **170,346 → 165,546**

---

### P2-W4 — Service-slot allocator, projection, and loader facts

**Kernel source:** allocator, allocator projection, loader identity, artifact-hash binding, and loader-fact files.

**Pure kernel-side logic still present:**

- Allocator fact/prerequisite/authority/intent DTOs.
- Candidate and observed-fact evaluators.
- First-failure status/reason tables.
- Missing/available snapshot constructors.
- Identity and artifact-hash binding checks.
- Eight loader-fact prerequisite chains.
- Source-evidence projections and selftest matrices.

**Entanglement:**

- Snapshot acquisition repeatedly calls the event log.
- Evaluation and `Value` emission are interleaved.
- The allocator file mixes diagnostic projection, authority evaluation, source-evidence lookup, and exhaustive cases.
- No actual service-slot allocation occurs in this family; mutation remains denied.

**Mechanics:**

- Add `raios_core::module_service_slot_allocator`.
- Add `raios_core::module_loader_facts`.
- Put projection builders beside their evaluator instead of creating a generic projection framework.
- Kernel gathers retained evidence/event IDs into immutable snapshots and calls core.
- Keep actual registry allocation/mutation absent or kernel-owned when later implemented.

**Estimated net kernel reduction:** **7,000 lines** after thin snapshot adapters.

**Focused VM:** `module-audit-rollback`; also include the `full-module-load-gate` script at the wave close because these facts feed loader admission.

**Selftests ported:** allocator fact, prerequisite, authority, allocation-intent, authority-input, authority-decision, registry-write-commit-gate, identity, artifact-hash, and all loader-fact tables.

**Guest proof retained:** complete observed snapshot remains non-authorizing; one missing prerequisite returns the existing first-failure reason.

**Golden dependency:** none.

**Trajectory:** **165,546 → 158,546**

---

### P2-W5 — Loader-runtime evaluator

**Kernel source:** loader-runtime root plus `snapshot`, `eval`, `evidence_core`, `evidence_live_load`, `selftest`, and pure portions of `render`.

**Pure kernel-side logic still present:**

- Runtime snapshot/check/result types.
- Ordered prerequisite and execution-boundary evaluation.
- Descriptor/load-plan/page-map/entrypoint state decisions.
- Evidence-reference matching and hashes.
- Dry/live-load result classification.
- Exhaustive reference cases.

**Entanglement:**

- `snapshot.rs` reads event-log state while constructing neutral facts.
- `evidence_*` mixes decision construction with event retention.
- `render.rs` mixes pure `Value` construction with serial emission.
- Live-load evaluation sits adjacent to actual candidate-byte, mapping, entrypoint, and registry paths.

**Mechanics:**

- Add `raios_core::module_loader_runtime`.
- Move only immutable snapshot DTOs, evaluation, evidence projections, hashes, and test tables.
- Kernel keeps candidate-byte access, page allocation/mapping, entrypoint transfer, service start, inventory mutation, and event writes.
- Split the call sequence explicitly:

```text
kernel acquire snapshot
    -> core evaluate
    -> kernel scoped execution/apply
    -> core project result
    -> kernel record event + emit
```

**Estimated net kernel reduction:** **8,000 lines**.

**Focused VM:** the full module load-gate composition via `full`’s `full-module-load-gate` fragment; use `module-audit-rollback` as the named focused profile if the harness is not given a standalone load-gate profile.

**Selftests ported:** missing retained evidence, allocator authority, every loader fact, stale/scope/schema/provenance/binding failures, dry-ready case, artifact-load denial, executable-map denial, entrypoint-transfer denial, service-start denial, and final defined-non-executable case.

**Guest proof retained:** one dry evaluation and one entrypoint-binding denial; no load or mutation.

**Golden dependency:** none. Moving serial formatting from `render.rs` is P4 if byte identity cannot be retained.

**Trajectory:** **158,546 → 150,546**

---

### P2-W6 — Write-boundary and hello rollback authority foundation

This is the required rerouted **RELOCATE** wave.

**Kernel source:**

- All ten `agent_protocol_module_write_boundary*` files: **11,130 lines**.
- Pure evaluator/hash/table portions of hello rollback authority, writer gate, bindings, storage authority, rollback records, and rollback-related emitters.

**Pure kernel-side logic still present:**

- Availability, write-policy, storage-layout, append-engine, append-contract, payload-hash, append-intent, and final boundary snapshot evaluators.
- Hello rollback media-write, durable-append, policy, ledger, writer, preflight, payload-envelope, and authority decisions.
- Pure rollback preview/apply record construction and hashes.
- Dry-run and inspection decisions over already-acquired snapshots.
- Extensive missing/stale/substituted/wrong-schema/binding/availability matrices.

**What core already owns:**

- `scoped_rollback_apply` owns scoped apply, authorized append, verified apply, and retained-inspection decisions.
- Scoped append modules own real durable append authority for their record families.
- Therefore this wave must not recreate scoped apply or invent a shared generic write-authority evaluator.

**Mechanics:**

- Add `raios_core::module_write_boundary` for the module diagnostic snapshot chain.
- Add `raios_core::hello_rollback_foundation` for hello-specific pure foundation decisions only.
- Reuse `scoped_rollback_apply` for final scoped apply/authorized-append/verified-apply calls.
- Do not create a generic shared write boundary capable of authorizing unrelated record families.
- Kernel keeps:

  - RECLOG/media/storage discovery;
  - sector reads and readback acquisition;
  - event lookup/retention;
  - current service state and rollback target acquisition;
  - durable append invocation;
  - scoped-apply wiring;
  - service-state mutation after successful apply.

### Byte-identical authority input preservation

The core input structs must be transliterations of the current snapshots, not redesigned abstractions. Each authority input remains sourced from the same kernel observation:

| Authority input | Preserved acquisition |
|---|---|
| Method/service ID | Existing dispatch method and fixed hello service ID |
| Target region ID/marker | Existing storage-layout snapshot |
| LBA start/count/byte count | Existing media/partition snapshot |
| Audit and rollback target IDs/schemas | Existing append-contract snapshot |
| Policy/availability/engine state | Existing write-boundary snapshots |
| Payload and provenance hashes | Existing hash functions with identical `"name="` grammar |
| Retained evidence/event IDs | Existing current-boot event lookup |
| Probation and preview status | Existing hello service snapshot |
| Scoped apply inputs | Constructed from the same fields currently passed to `scoped_rollback_apply` |
| Readback/inspection evidence | Acquired by kernel after I/O, then evaluated in core |

No core evaluator acquires an event, reads a sector, appends a record, or mutates the hello service.

**Host tests that pin the chain:**

- Existing `scoped_rollback_apply` tests continue pinning:

  - valid scoped apply;
  - wrong method/service/region;
  - append authorization;
  - verified apply;
  - retained applied-rollback inspection.

- New `module_write_boundary` host tables pin every existing module selftest case.
- New `hello_rollback_foundation` tables pin the current authority, writer, storage, payload, dry-run, and inspection matrices.
- Cross-module host tests assert that the exact foundation decision fields supplied to `scoped_rollback_apply` match the current kernel paper trace.
- Hash tests pin literal field order, `"name="` separators, and eight-digit event IDs.

**Estimated net kernel reduction:** **13,000 lines**, including both the write-boundary family and the pure hello rollback foundation, after roughly 700–1,000 lines of kernel acquisition/apply adapters.

**Focused VM:** `module-audit-rollback` for the module boundary, then `m6d-rollback` for hello rollback. This wave crosses a real storage/rollback/authority boundary and must not batch its verification with another wave.

**Selftests ported:** every write-boundary table plus hello rollback authority, durable policy, ledger, append, storage, writer, payload-envelope, readback, inspection, preview, and apply-decision tables.

**Guest proof retained:** one current-boot denied module boundary, one rollback preview, one applied/denied scoped rollback path as supported by the current implementation, and real readback evidence through kernel wiring.

**Golden dependency:** none. Any changed reason/field/order is P4 and blocks this P2 wave until restored.

**Trajectory:** **150,546 → 137,546**

---

### P2-W7 — Event model and fixed-capacity ring

**Kernel source:** `event_log_types.rs`, `event_log.rs`, evidence and surviving module/provider/hello selftests.

**Pure kernel-side logic still present:**

- `EventId`, event/binding records, and binding accessors.
- Event-ID formatting and validation.
- Fixed-capacity ring append, overwrite, lookup, and recent-window selection.
- Pure evidence projection and binding consistency checks.
- Module/provider/hello event selftest tables.

**Entanglement:**

- Global mutex access is mixed with ring operations.
- Record helpers both mutate the ring and construct evidence.
- `event_log_evidence.rs` mixes lookup, check, and output building.
- Family-specific event checks share the central event vocabulary.

**Mechanics:**

- Add `raios_core::event_log`.
- Move types and a state-parameterized fixed-capacity ring implementation.
- Reuse existing current-boot sequence parsing from core.
- Kernel retains the single global mutex and exposes thin `record`, `latest`, and `snapshot` adapters.
- Family modules produce typed event payloads; kernel assigns sequence IDs and mutates the ring.

**Estimated net kernel reduction:** **9,500 lines**.

**Focused VM:** `quick`, followed by `provider-memory` because provider denial/binding events are security-relevant.

**Selftests ported:** capacity overwrite, retained ordering, recent limits, stale/missing ID, exact event-ID grammar, binding mismatch, and surviving module/provider/hello evidence cases.

**Guest proof retained:** append past capacity, retained ordering/window, exact eight-digit ID parsing, and stale-ID rejection through the locked kernel adapter.

**Golden dependency:** none. Event vocabulary compaction is P4.

**Trajectory:** **137,546 → 128,046**

---

### P2-W8 — Memory projection remainder

**Kernel source:** `agent_protocol_memory.rs`.

**Pure kernel-side logic still present:**

- Event-binding-to-`Value` projections.
- Profile/context/query/trace/recent-event response construction.
- Classification and omission tables not already in core.
- Deterministic trace selection and reference fixtures.
- Repeated record fragment construction.

**Entanglement:**

- Live store/event snapshots are gathered inside response rendering.
- `Value` construction is mixed with direct serial fragment emission.
- Durable-store reads and mutation-denial audit wiring sit beside read-only projection logic.

**Mechanics:**

- Extend `memory_context` with surviving projection/selection functions.
- Extend `memory_record` only when an existing record type owns the projection.
- Do not add `memory_projection` unless the moved code cannot reasonably belong to those two existing modules.
- Kernel gathers memory-store and event snapshots, invokes the core plan/projection, performs existing durable-denial audit wiring, and frames output.

**Estimated net kernel reduction:** **7,000 lines**.

**Focused VM:** `memory-durable`.

**Selftests ported:** profile budgets, event limits, classification/omission, trace selection, recent-event projection, durable-denial posture, and output-hash fixtures.

**Guest proof retained:** one bounded context read showing the same profile, token budget, included records, omissions, and `current_boot` posture.

**Golden dependency:** none. Table-driven emit-vocabulary compaction remains P4.

**Trajectory:** **128,046 → 121,046**

---

### P2-W9 — Provider remainder and surviving hello model

This is a small final relocation close, not a new provider design.

**Kernel source:**

- Remaining pure content in `agent_protocol_provider.rs`.
- Hello lifecycle constants, hash support, lifecycle binding, state records, pure record builders, and pure parts of emitters/preflight.
- Excludes rollback foundation already moved in W6 and mutable runtime/state-machine code.

**Pure kernel-side logic still present:**

- Provider denial/audit `Value` construction and remaining duplicate hashes.
- Hello lifecycle/state-migration/probation records and hashes.
- Descriptor/preflight validation over already-acquired bytes/facts.
- Lifecycle binding checks and reference tables.

**Entanglement:**

- Provider live trust/key/network acquisition is mixed with projection.
- Hello emitters contain both pure record builders and serial wrappers.
- Hello state-machine operations acquire/mutate current-boot state and record events.

**Mechanics:**

- Extend existing `provider_context` and `scoped_provider_export`; no new provider module.
- Add `raios_core::hello_service_model`.
- Kernel keeps provider transport/key/trust acquisition, hello runtime/state machine, current-boot state, event writes, dispatch, and serial output.
- Core receives descriptor bytes/hashes as inputs; it never reads candidate bytes itself.

**Estimated net kernel reduction:** **3,500 lines**.

**Focused VM:** `provider-memory`, then `quick` for hello lifecycle.

**Selftests ported:** provider packet/denial/smuggling fixtures; hello lifecycle, hot-swap, migration, probation, descriptor identity, and preflight cases not already covered by W6.

**Guest proof retained:** provider public projection plus trust denial; legal and illegal hello lifecycle transitions.

**Golden dependency:** none.

**Trajectory:** **121,046 → 117,546**

## 5. Projected P2 trajectory

| Close | Net reduction | Projected kernel lines |
|---|---:|---:|
| Current main | — | **176,346** |
| W2 load-gate completion | 6,000 | 170,346 |
| W3 evidence/types | 4,800 | 165,546 |
| W4 allocator/facts | 7,000 | 158,546 |
| W5 loader runtime | 8,000 | 150,546 |
| W6 write/rollback authority | 13,000 | 137,546 |
| W7 event log | 9,500 | 128,046 |
| W8 memory remainder | 7,000 | 121,046 |
| W9 provider + hello model | 3,500 | **117,546** |

Projected margin below target: **2,454 lines**.

The estimates are deliberately net of expected kernel adapters. W6–W8 require measurement after each move because their files mix evaluators, acquisition, and rendering heavily. If any wave lands more than 1,000 lines short of its estimate, split out the next already-identified pure `Value` builders before considering P4 goldens.

## 6. P4 preview — emit-vocabulary compaction

P4 Batch 6 can remove repeated field vocabulary and hand-written serial/object emission after P2 establishes typed records. These are rough removable-line estimates, not P2 credit.

| Family | Current emit-vocabulary footprint | Rough P4 removable lines | Notes |
|---|---:|---:|---|
| Module references/evidence/service slot | ~2,600 | **1,700–2,100** | Repeated reference object, retained binding, gate-state, policy-result, and selftest-case fields. |
| Allocator and loader facts | ~3,400 | **2,200–2,800** | Repeated source-evidence item, fact/prerequisite/authority item, and missing/available record vocabulary. |
| Loader runtime/load-gate render | ~6,800 | **4,500–5,500** | Largest single table-driven opportunity; most is repeated ordered field emission. |
| Event log/evidence | ~3,000 | **1,800–2,400** | Event/binding projection vocabulary; ring mechanics are not P4. |
| Memory | ~5,000 | **3,200–4,000** | Profile/context/trace/recent-event field repetition and fragment emitters. |
| Provider | ~1,100 | **650–850** | Core already owns much of the projection vocabulary, so remaining gain is limited. |
| Module write boundary | ~4,000 | **2,600–3,200** | Availability/policy/layout/contract/intent/boundary records repeat the same evidence envelope. |
| Hello lifecycle and rollback | ~7,500 | **4,500–5,800** | Broad record vocabulary; authority decisions must remain distinct even if emitted from tables. |
| **Total preview** | **~33,400** | **~21,150–26,650** | Only after byte-identical typed projection exists. |

### Owner ambition choices

**Minimal P4**

- Compact only the largest established vocabularies: load-gate/runtime, memory, and hello rollback.
- Expected additional kernel reduction: **~12k–15k**.
- Likely result: roughly **102k–106k**.
- Lowest golden and review risk.

**Vocabulary v1 P4**

- Convert all surviving protocol emitters to typed field tables/record projections.
- Expected additional kernel reduction: **~21k–27k**.
- Likely result: roughly **91k–96k**.
- Requires a deliberate vocabulary version/golden ceremony and should not be smuggled into P2.

## 7. Verification contract per wave

Each implementation wave closes only after:

1. The moved exhaustive tables pass under `cargo test -p raios-core`.
2. The kernel still compiles with the local `CARGO_HOME` and lane-specific target directory.
3. `cargo fmt --all -- --check` passes.
4. The named focused VM profile passes.
5. Existing serial output/goldens are byte-identical.
6. `scripts/check-source-size.ps1` passes.
7. `scripts/scan-secrets.ps1` passes before commit.
8. The orchestrator reads the entire diff.
9. The exact physical kernel line count is recorded.
10. No source changes remain uncommitted at slice close.

Because W6 changes storage/rollback authority boundaries, it receives its own `module-audit-rollback` and `m6d-rollback` evidence and is not batched with adjacent work.

Run `full` plus `recovery` only at the P2 sub-milestone close, consistent with the aggressive-fast owner cadence. If any focused run fails, classify it in `PROJECT_STATUS.md` before retry under the repository’s failure-classification rule.

## 8. Execution order

1. **W2:** finish the already-started load-gate relocation.
2. **W3:** move retained evidence and collapse the kernel module-types mirror.
3. **W4:** move allocator/projection/loader facts.
4. **W5:** move loader-runtime evaluation.
5. **W6:** move the write-boundary and hello rollback foundation as its own authority wave.
6. **W7:** move event types/ring/evidence after the producing module families use core DTOs.
7. **W8:** move memory projection after core event values exist.
8. **W9:** close the small provider remainder and surviving hello lifecycle model.
9. Run P2 phase-close `full` and `recovery`, source-size, format, and secret checks.
10. Choose minimal or vocabulary-v1 P4 based on the measured P2 result.

## 9. Stop conditions

A P2 wave stops and is redesigned if it requires any of the following:

- A golden or serial-output change.
- A schema/version change.
- A different status/reason or first-failure order.
- Broader event-ID parsing.
- A new generic write-authority abstraction.
- Moving a mutex, hardware I/O, media append, service mutation, executable mapping, or entrypoint transfer into `raios-core`.
- Reimplementing logic already present in `module_load_gate`, `memory_context`, `provider_context`, `scoped_rollback_apply`, or another scoped evaluator.
- Granting authority not present before the relocation.

The shortest safe path is to move the existing pure decision once, preserve the kernel’s acquisition/apply boundary, port the exhaustive table to Cargo, and delete the duplicate guest matrix.