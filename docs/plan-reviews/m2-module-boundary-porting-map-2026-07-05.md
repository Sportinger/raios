# M2 Module-Boundary Porting Map (2026-07-05)

Read-only scoping analysis (packet M2-10) for porting the
`agent_protocol_module_*` emit surface onto `raios_core::record`.
Authoritative hazard rule: the key=value line-hash inputs in
`module_evidence.rs` must NEVER be replaced by JSON canonicalization.

**Porting Map**

| File | Lines | Class | Emits | Hash sites | Coverage |
|---|---:|---|---|---|---|
| `module_evidence.rs` | 4516 | COUPLED | key=value hash input only | owner: module hashes at 542, 3392, 3466, 3537, 3624, 3710, 3809, 3909, 4026, 4147, 4189, 4228, 4302, 4365, 4444; helpers 4538-4589 | module-audit-rollback, full |
| `event_log_module_checks.rs` | 295 | NOT emitter | none | validates line hashes at 33,55,88,125,168,193,235,252,286 | module-audit-rollback, full |
| `agent_protocol_module_reference.rs` | 2298 | COUPLED | JSON + line-hash calls | 720,762,985,991,1155,1156,1162,1823,1829,2036,2037,2043,2053; wrappers 2308-2370 | module-audit-rollback, full |
| `agent_protocol_module_grant.rs` | 642 | COUPLED | JSON + line-hash calls | 352,483,538; wrapper 654-660 | module-audit-rollback diag; full selftest |
| `agent_protocol_module_audit.rs` | 1009 | COUPLED | JSON + line-hash calls | 581,587,594,864,870,876; wrappers 1008-1037 | module-audit-rollback diag; full selftest |
| `agent_protocol_module_attestation.rs` | 1108 | COUPLED | JSON + line-hash calls | 544,550,782,783,789,799,811; wrappers 1042-1136 | module-audit-rollback, full |
| `agent_protocol_module_approval.rs` | 1235 | COUPLED | JSON + line-hash calls | 572,578,825,826,832,842,854,868; wrappers 1133-1265 | module-audit-rollback, full |
| `agent_protocol_module_service_slot.rs` | 660 | COUPLED | JSON + line-hash calls | 384,637; wrapper 683-686 | module-audit-rollback diag; full selftest |
| `agent_protocol_module_write_boundary_append_payload_hash.rs` | 953 | COUPLED | JSON + line-hash calls | 195,207 | module-audit-rollback diag; full selftest |
| `agent_protocol_module_load_gate_selftest_reference_cases.rs` | 1532 | NOT emitter | none | 139,365,705,1079,1521 | full only |
| `agent_protocol_module_load_gate_selftest_eval.rs` | 1416 | NOT emitter | none | 51,119,222,356,504,1228,1238,1249,1295; wrappers 1440-1479 | full only |
| `agent_protocol_module_load_gate_selftest.rs` | 1152 | NOT emitter | none | 943,950,972,978,984,1071,1126 | full only |
| `agent_protocol_module_load_gate_selftest_emit.rs` | 718 | SAFE | JSON only | none | full only |
| `agent_protocol_module_load_gate_render.rs` | 6079 | SAFE | JSON only | none; renders precomputed hashes | quick, module-audit-rollback, full |
| `agent_protocol_module_load_gate.rs` | 17 | NOT emitter | none | none | quick/module-audit-rollback/full dispatch |
| `agent_protocol_module_loader_runtime.rs` | 10308 | SAFE | JSON only | none | module-audit-rollback diag; full selftest/load-gate |
| `agent_protocol_module_loader_identity.rs` | 840 | SAFE | JSON only | none | module-audit-rollback diag; full selftest |
| `agent_protocol_module_loader_fact.rs` | 1230 | SAFE | JSON only | none | module-audit-rollback diag; full selftests |
| `agent_protocol_module_loader_artifact_hash_binding.rs` | 851 | SAFE | JSON only | none | module-audit-rollback diag; full selftest |
| `agent_protocol_module_service_slot_allocator.rs` | 4325 | SAFE | JSON only | none; renders retained reservation hashes | module-audit-rollback diag; full selftest/load-gate |
| `agent_protocol_module_service_slot_allocator_projection.rs` | 1185 | NOT emitter | none | none | quick/full through `module.load_ephemeral` |
| `agent_protocol_module_types.rs` | 2975 | NOT emitter | none | none | indirect everywhere |
| `agent_protocol_module_write_boundary.rs` | 93 | NOT emitter | none | none | dispatch for write-boundary methods |
| `agent_protocol_module_write_boundary_emit.rs` | 106 | SAFE | JSON helper only | none | indirect via write-boundary emitters |
| `agent_protocol_module_write_boundary_boundary.rs` | 1728 | SAFE | JSON only | none; renders precomputed hashes at 255-285,531-533 | module-audit-rollback diag; full selftest |
| `agent_protocol_module_write_boundary_availability.rs` | 363 | SAFE | JSON only | none | module-audit-rollback diag; full selftest |
| `agent_protocol_module_write_boundary_write_policy.rs` | 504 | SAFE | JSON only | none | module-audit-rollback diag; full selftest |
| `agent_protocol_module_write_boundary_storage_layout.rs` | 1960 | SAFE | JSON only | none | quick, module-audit-rollback diag; full selftest |
| `agent_protocol_module_write_boundary_append_engine.rs` | 684 | SAFE | JSON only | none | module-audit-rollback diag; full selftest |
| `agent_protocol_module_write_boundary_append_contract.rs` | 1550 | SAFE | JSON only | none | quick, module-audit-rollback diag; full selftest |
| `agent_protocol_module_write_boundary_append_intent.rs` | 968 | SAFE | JSON only | none; consumes precomputed payload hashes | module-audit-rollback diag; full selftest |

**Class-B Mapping**

Today, `module_evidence.rs` hashes ordered ASCII lines exactly as `name=value`, lowercase hex for `[u8;32]`, `event.current_boot.NNNNNNNN` for event sequences, `\n` between lines, and no newline after the final line. That path must stay byte-identical.

Port rule: do not make these hashes use JSON or `raios_core::record::sha256_of_json`. Keep `module_evidence` key=value hash functions untouched or move them as a dedicated `canonical_line_hash` helper with golden vectors. Port only the surrounding JSON response rendering to `record::Value`.

**Batch Order**

1. SAFE small write-boundary batch: `write_boundary_emit`, `availability`, `write_policy`, `append_engine`, `append_intent`. Verify: `module-audit-rollback`; full only if selftest emit changed.
2. SAFE storage/contract batch: `storage_layout`, `append_contract`, `write_boundary_boundary`. Verify: quick plus `module-audit-rollback`.
3. SAFE loader/service rendering batch: `loader_identity`, `loader_artifact_hash_binding`, `loader_fact`, `service_slot_allocator`. Verify: `module-audit-rollback`; full for selftests.
4. SAFE large render batch: `loader_runtime`, `load_gate_render`, `load_gate_selftest_emit`. Verify: full profile, because load-gate/selftest coverage is full-only.
5. COUPLED batch: `reference`, `grant`, `audit`, `attestation`, `approval`, `service_slot`, `append_payload_hash`. Verify: `module-audit-rollback` plus full for coupled selftests/load-gate matrices. Keep hash helpers unchanged.
6. No-port batch: `types`, `load_gate`, `write_boundary`, `service_slot_allocator_projection`, `event_log_module_checks`, `load_gate_selftest_*` eval/case files unless their callers need signature changes.

**Risks**

Main risk is accidentally replacing key=value hash inputs with JSON canonicalization. Second risk is porting large renderers before small safe emitters prove the pattern. Third risk is assuming `quick` covers module details; it only samples `storage_layout`, `append_contract`, and `module.load_ephemeral`.