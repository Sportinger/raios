# P2-WAVE2-DESIGN

## Routing correction

`agent_protocol_recovery*` is entirely **RETIRE**, not RELOCATE: 47 files / 39,193 baseline lines. No recovery diagnostic function, table, emitter, or type should enter `raios-core`. The real `seed-kernel/src/recovery_lifeline.rs` is outside that family and remains kernel-owned.

The same rule excludes:

- Ten `agent_protocol_module_write_boundary*` files: 9,575 lines.
- RETIRE-routed `hello_service/*` files.
- Recovery/module/hello event variants made unreachable by those deletions.

P3 must delete these before adjacent P2 relocation so dead vocabulary is not preserved as a host API.

## Ordered waves

| Wave | Family and source files | Relocate to `raios-core` | Est. kernel reduction | Focused VM | Selftest tables ported to Cargo |
|---|---|---|---:|---|---|
| **P2-W2** | Module reference decisions: `agent_protocol_module_reference.rs`, `_approval.rs`, `_attestation.rs`, `_grant.rs`, `_audit.rs`, `_service_slot.rs` | `module_reference`, `module_promotion_evidence` | Pure input/check/result records; reference evaluators; ordered failure predicates; case fixtures; status/reason constants; canonical hashes that accept neutral scalar inputs. Keep dispatch, serial emission, event lookup/recording in kernel. | **~6,500** | `full-module-evidence` | Manifest/artifact/VM-report, approval, attestation, grant, audit/rollback and service-slot reference cases. Keep one guest aggregate predicate: `module.reference_core_sanity`. |
| **P2-W3** | Module allocation/fact seam: `agent_protocol_module_service_slot_allocator.rs`, `_projection.rs`, `_loader_identity.rs`, `_loader_artifact_hash_binding.rs`, `_loader_fact.rs`; remaining neutral names in `_module_types.rs` | `module_service_slot`, `module_loader_facts` | Slot-selection/projection functions; allocator authority decision table; identity and artifact-hash checks; loader-fact constraint tables; DTOs still used by these evaluators. Kernel retains registry mutation, event IDs, snapshots and serial rendering. | **~8,000** | `full-module-load-gate` + relevant `full-module-selftests` fragment | Allocator fact/prerequisite/authority/allocation-intent cases; loader identity, artifact-hash and eight loader-fact tables. Keep `module.loader_facts_core_sanity`. |
| **P2-W4** | Loader runtime: `agent_protocol_module_loader_runtime.rs` and `agent_protocol_module_loader_runtime/{snapshot,eval,evidence_core,evidence_live_load,selftest,render}.rs` | `module_loader_runtime` | Neutral snapshot/check types; execution-boundary evaluator; descriptor/load-plan/page-map/entrypoint state transitions; ordered denial table; canonical evidence/hash projections; all reference cases. Kernel retains actual candidate-byte access, page mapping, entrypoint transfer, service registry mutation, event-log reads/writes and serial rendering. | **~8,500** | `full-module-load-gate` | Runtime and live-load reference tables, including descriptor intake through commit-result cases. Keep `module.loader_runtime_core_sanity`. |
| **P2-W5** | Event vocabulary/ring: `event_log_types.rs`, `event_log.rs`, `_evidence.rs`, `_module_checks.rs`, `_provider_selftest.rs`, `_hello_selftest.rs` — **only after P3 prunes retired variants** | `event_log` | Surviving `EventId`, `Event`, `EventBindings` and binding records; fixed-capacity ring operations; recent-window selection; pure check/consume evaluators; event evidence projection; surviving module/provider tables. Kernel retains the global mutex, runtime snapshot acquisition and thin `record_*`/`latest_*` adapters. | **~7,500** | `common` + `provider-memory` | Surviving provider binding/injection cases and module checks. The hello rollback-inspect table is deleted with its retired P3 surface, not ported. Keep `event_log.ring_core_sanity`. |
| **P2-W6** | Memory remainder: current `agent_protocol_memory.rs`, after P3 recovery/hello/module-write-boundary removals and after W5 supplies core event values | extend `memory_context`; add `memory_projection` only if the existing module would become unwieldy | Pure `record::Value` construction for memory profile/context/query/trace/recent-events; event-binding-to-value projection; classification/omission tables; deterministic trace selection. Kernel retains snapshot gathering, durable-store reads, dispatch and final serial framing. | **~4,000** | `memory-durable` | Projection/reference fixtures for surviving event bindings, profile budgets and trace selection. All recovery-lattice projection code is deleted by P3. Keep `memory.context_core_sanity`. |
| **P2-W7** | Provider remainder: `agent_protocol_provider.rs` plus surviving provider checks removed from kernel by W5 | extend `provider_context` | Public export packet construction; durable-denial value construction; binding/injection decision projections; canonical projection hashes; optional scalar-to-record helpers. Kernel retains live trust/provider/Wi-Fi collection, key/transport behavior, event adapter, dispatch and serial framing. | **~1,300** | `provider-memory` | Export packet, final-authorization, smuggling and durable-denial fixtures. Keep `provider.context_core_sanity`. |
| **P2-W8** | Surviving hello modules: `hello_service/{constants,hash_support,lifecycle_binding,records,state_records,emitters}.rs`, after all RETIRE rows are removed | `hello_service_model` | Surviving lifecycle/state records; lifecycle binding evaluator; canonical hashes; pure response `Value` builders. Kernel retains `hello_service.rs`, `runtime.rs`, `state_machine.rs`, current-boot state and dispatch. Do not move rollback/storage diagnostic material merely because it currently shares `emitters.rs` or `records.rs`. | **~3,500** | `quick` | Surviving lifecycle/hot-swap/state-transition reference tables. Retired rollback/preflight tables are deleted. Keep `hello.lifecycle_core_sanity`. |

### Why recovery has no wave

The P0 map is unambiguous:

```text
agent_protocol_recovery* → RETIRE (39,193 lines)
real recovery_lifeline.rs → outside that family; keep in kernel
```

Creating `raios_core::recovery_diagnostics` would preserve exactly the superseded lattice P3 exists to remove.

## P3 conflicts and cut lines

Only `REFACTOR-P3-WB-DELETE` is presently named in the checked-in P3 design. The remaining labels below are orchestration names for the required deletion slices, not claims that matching packet documents already exist.

| P3 slice | Retirement scope | Conflicting P2 waves | Safe ordering |
|---|---|---|---|
| **P3-1 / REFACTOR-P3-WB-DELETE** | Ten `agent_protocol_module_write_boundary*` files | W6 and W8 currently reference their bindings/constants; W2’s audit record may also expose old fields | Delete first, after replacing the blocked hello cut line with real scoped evidence. Never copy these types into core. |
| **P3-2 RECOVERY-DELETE** | All 47 `agent_protocol_recovery*` files and dispatch/harness consumers | W5 event vocabulary, W6 memory projections | Must precede W5/W6. Delete recovery-only `EventBindings`, event helpers and memory emitters in the same coherent deletion or immediately adjacent cleanup. |
| **P3-3 HELLO-DIAGNOSTICS-DELETE** | RETIRE-routed hello command targets, descriptor/preflight and rollback/storage lattice | W8; also unblocks P3-1 | Must precede W8. Split surviving lifecycle records from dead rollback fields before relocation. |
| **P3-4 MODULE-DEAD-EVIDENCE-CLEANUP** | Any module diagnostic fields made obsolete by P3-1 and already superseded by real M6/M7 behavior; route must be confirmed against the P0 rows | W2/W3 | Prune before relocating the containing record/evaluator. Do not broaden this slice to RELOCATE rows. |

Write-set implications:

- W2–W4 should not edit any `agent_protocol_recovery*`, module-write-boundary, or RETIRE hello file.
- W5 must wait for P3-2 because `event_log_types.rs` currently embeds extensive recovery-only vocabulary.
- W6 must wait for P3-1/P3-2/P3-3; the current 11,395-line memory file contains large render sections for all three retired families.
- W8 must wait for P3-3 and P3-1 because `emitters.rs` and `records.rs` mix surviving lifecycle data with retired rollback/write-boundary data.
- P3 owns dispatch/harness deletion. P2 owns byte-identical behavior only for surviving methods.

## Thin guest predicates retained

One aggregate predicate per relocated family is sufficient:

| Family | Guest predicate | Minimum proof |
|---|---|---|
| Module references | `module.reference_core_sanity` | One valid chain and one substituted-hash denial traverse the core evaluator. |
| Module loader facts | `module.loader_facts_core_sanity` | One complete fact set passes; one missing prerequisite returns the established first-failure reason. |
| Loader runtime | `module.loader_runtime_core_sanity` | One accepted dry evaluation and one entrypoint-binding denial; no load or mutation. |
| Event log | `event_log.ring_core_sanity` | Append past capacity, verify retained ordering/window and stale-ID rejection through the kernel adapter. |
| Memory | `memory.context_core_sanity` | One bounded read plan preserves profile, budget and public/local omission posture. |
| Provider | `provider.context_core_sanity` | One public projection matches hashes; one trust/body mismatch remains denied with no body attachment. |
| Hello lifecycle | `hello.lifecycle_core_sanity` | One legal state transition and one illegal transition return the existing result. |

These replace the duplicated in-guest reference matrices; Cargo tests own exhaustive tables. They do not replace focused integration predicates for real persistence, provider trust or runtime behavior.

## Reduction estimate

Estimated remaining P2 reduction:

```text
W2   module references          ~6.5k
W3   allocator/facts            ~8.0k
W4   loader runtime             ~8.5k
W5   event log                  ~7.5k
W6   memory remainder           ~4.0k
W7   provider remainder         ~1.3k
W8   hello surviving model      ~3.5k
                                  -----
total                            ~39.3k kernel lines
```

Using the packet’s planning baseline:

```text
~206.0k current planning count
 -58.7k P3 retirement
  -1.7k wave 1 already landed
 -39.3k waves 2–8
 -------
~106.3k projected seed-kernel/src
```

The live tree currently measures roughly 208k physical lines, so the corresponding live-tree projection is approximately **108k**. Both figures are estimates: P3 deletion will reveal shared adapters that remain necessary, while one-line kernel re-exports and thin sanity predicates add back a few hundred lines.

## Recommended interleaving

For maximum deletion per QEMU run without relocating dead vocabulary:

1. **P3-3 hello diagnostics cut** — establishes the real surviving hello boundary and unblocks the checked-in P3 deletion.
2. **P3-1 module-write-boundary deletion** — approximately 9.6k direct deletion, plus dependent hello cleanup; run its focused persistence/quick evidence.
3. **P3-2 recovery deletion** — approximately 39.2k baseline deletion; run the required recovery/full phase-close evidence because this changes the recovery command surface.
4. **P2-W2 + P2-W3** — same module ownership boundary; one combined `full-module-evidence`/`full-module-load-gate` QEMU close if no authority boundary changes.
5. **P2-W4** — loader runtime deserves its own focused run because it borders executable mapping and live-load behavior.
6. **P2-W5 + P2-W6** — after recovery pruning, relocate the surviving event ring and memory projection together; one `memory-durable` plus `common` focused close.
7. **P2-W7** — provider trust/export boundary gets its own `provider-memory` run.
8. **P2-W8** — relocate only the surviving hello lifecycle model; close with `quick`.
9. Run **full + recovery** only at the P2/P3 sub-milestone close, with source-size, format and secret checks.

The key optimization is deleting P3-2 before touching event or memory: otherwise W5/W6 would spend a host module and Cargo tests preserving tens of thousands of lines that the routing map already says must disappear.