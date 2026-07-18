# P3 retirement design: agent_protocol_module_write_boundary*

Packet: REFACTOR-P3-DESIGN. Date: 2026-07-12. Status: BLOCKED pending the
hello-service cut line.

This is a deletion design only. No source, harness, image, build, QEMU run,
stage, or commit is part of this packet. The inventory routes this family to
RETIRE because these generic methods only describe the old denied shared
audit/rollback writer; M7/M9 now use own-pinned scoped append evaluators.

## 1. Complete deletion manifest

### Ten files

Current PowerShell counts (Get-Content.Count and Get-Item.Length):

| file | lines | bytes | action |
| --- | ---: | ---: | --- |
| seed-kernel/src/agent_protocol_module_write_boundary.rs | 25 | 1,430 | delete facade |
| seed-kernel/src/agent_protocol_module_write_boundary_append_contract.rs | 1,722 | 69,150 | delete |
| seed-kernel/src/agent_protocol_module_write_boundary_append_engine.rs | 808 | 30,683 | delete |
| seed-kernel/src/agent_protocol_module_write_boundary_append_intent.rs | 1,088 | 42,244 | delete |
| seed-kernel/src/agent_protocol_module_write_boundary_append_payload_hash.rs | 1,095 | 46,063 | delete |
| seed-kernel/src/agent_protocol_module_write_boundary_availability.rs | 442 | 16,179 | delete |
| seed-kernel/src/agent_protocol_module_write_boundary_boundary.rs | 1,854 | 75,478 | delete |
| seed-kernel/src/agent_protocol_module_write_boundary_emit.rs | 76 | 2,501 | delete support |
| seed-kernel/src/agent_protocol_module_write_boundary_storage_layout.rs | 1,865 | 72,629 | delete |
| seed-kernel/src/agent_protocol_module_write_boundary_write_policy.rs | 600 | 22,460 | delete |
| TOTAL | 9,575 | 432,817 | |

### Dispatch and declarations

Remove the import group in seed-kernel/src/agent_protocol.rs lines 68-82,
then remove these complete MethodEntry pairs at lines 573-588:

| rows | method | selftest |
| ---: | --- | --- |
| 573-574 | module.audit_rollback_availability | module.audit_rollback_availability_selftest |
| 575-576 | module.audit_rollback_write_policy | module.audit_rollback_write_policy_selftest |
| 577-578 | module.audit_rollback_storage_layout | module.audit_rollback_storage_layout_selftest |
| 579-580 | module.audit_rollback_append_engine | module.audit_rollback_append_engine_selftest |
| 581-582 | module.audit_rollback_append_contract | module.audit_rollback_append_contract_selftest |
| 583-584 | module.audit_rollback_append_payload_hash | module.audit_rollback_append_payload_hash_selftest |
| 585-586 | module.audit_rollback_append_intent | module.audit_rollback_append_intent_selftest |
| 587-588 | module.audit_rollback_write_boundary | module.audit_rollback_write_boundary_selftest |

Remove seed-kernel/src/main.rs lines 43-52:

    mod agent_protocol_module_write_boundary;
    mod agent_protocol_module_write_boundary_append_contract;
    mod agent_protocol_module_write_boundary_append_engine;
    mod agent_protocol_module_write_boundary_append_intent;
    mod agent_protocol_module_write_boundary_append_payload_hash;
    mod agent_protocol_module_write_boundary_availability;
    mod agent_protocol_module_write_boundary_boundary;
    mod agent_protocol_module_write_boundary_emit;
    mod agent_protocol_module_write_boundary_storage_layout;
    mod agent_protocol_module_write_boundary_write_policy;

### Cross-reference evidence and cut line

The family is not cleanly deletable today. The surviving hello service imports
three of the modules and its children consume their constants, types, snapshots,
and evaluators:

    hello_service.rs
      -> append_contract, append_engine, storage_layout
      -> hello_service/{constants, emitters, records,
         rollback_authority_gates, rollback_writer_bindings,
         rollback_writer_gate, storage_authority_gate}

Exact grep evidence, excluding the ten family files:

| surviving file | evidence | hits |
| --- | --- | ---: |
| seed-kernel/src/hello_service.rs | imports at lines 7-17 | 7 |
| hello_service/constants.rs | rollback_storage_layout at 387-389 | 2 |
| hello_service/emitters.rs | rollback_storage_layout / rollback_append_contract | 115 |
| hello_service/records.rs | typed fields at 84 and 781 | 2 |
| hello_service/rollback_authority_gates.rs | calls/constants, including 4356-4457 | 68 |
| hello_service/rollback_writer_bindings.rs | family constants in projections | 85 |
| hello_service/rollback_writer_gate.rs | constants in canonical hashes/gates | 31 |
| hello_service/storage_authority_gate.rs | constants in authority projections | 44 |
| agent_protocol.rs | one import group at 68-82 | 1 group |
| main.rs | ten declarations at 43-52 | 10 |

This is the stop-condition dependency. The eventual deletion slice must first
replace the hello diagnostic projection, typed fields, and canonical hash
inputs with the real scoped rollback evidence. It must not delete around this
edge or silently preserve a fake compatibility layer. The broad token
module_audit_rollback in agent_protocol_module_audit.rs is a separate retained
reference diagnostic and is not a family import.

## 2. Harness update manifest

The old focused module-audit-rollback profile is retired with this family; it
must not remain runnable with deleted commands. Full baseline:
shadow-20260712-184856-27972.json, result passed, 7,870 predicates.

| profile fragment | exact consumer | same-slice edit |
| --- | --- | --- |
| full-module-audit-rollback.ps1 | lines 1-686: eight direct commands, eight positive envelope checks, eight wrong-capability checks, eight field blocks | delete all eight method sections and their predicates |
| full-module-selftests.ps1 | lines 487-755: eight selftest commands and all assertions | delete all eight selftest sections |
| persistence.ps1 | lines 638 and 642: generic-target-still-denied:write_policy_selftest and generic-target-still-denied:write_boundary_selftest | delete both commands; preserve real scoped persistence checks |
| quick.ps1 | lines 74, 76, 81 owner strings and lines 1125-1196 generic storage/contract probes | delete generic strings/probes; preserve scoped hello checks |
| shadow-vm-smoke.ps1 | full composition | remove only retired fragment composition |
| recovery composition | no direct write-boundary hit | no edit; verify unchanged |

### Exact predicate and golden-needle index

The following complete index is the source-anchored golden-needle manifest:
every Needle RHS in each listed block is deleted as a unit. Counts were
cross-checked against the 733 matching predicates in the passing full report.

| prefix | source lines | golden needles |
| --- | ---: | ---: |
| protocol:module_audit_rollback_availability_ | 2-25 | 24 |
| protocol:module_audit_rollback_write_policy_ | 78-106 | 26 |
| protocol:module_audit_rollback_storage_layout_ | 132-284 | 151 |
| protocol:module_audit_rollback_append_engine_ | 311-340 | 28 |
| protocol:module_audit_rollback_append_contract_ | 367-446 | 78 |
| protocol:module_audit_rollback_append_payload_hash_ | 473-518 | 44 |
| protocol:module_audit_rollback_append_intent_ | 545-598 | 52 |
| protocol:module_write_boundary_ | 625-686 | 68 |

The eight envelope-denial predicates and exact needles are:

    protocol:module_audit_rollback_availability_envelope_mismatch_no_dispatch
      serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_availability
    protocol:module_audit_rollback_write_policy_envelope_mismatch_no_dispatch
      serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_write_policy
    protocol:module_audit_rollback_storage_layout_envelope_mismatch_no_dispatch
      serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_storage_layout
    protocol:module_audit_rollback_append_engine_envelope_mismatch_no_dispatch
      serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_append_engine
    protocol:module_audit_rollback_append_contract_envelope_mismatch_no_dispatch
      serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_append_contract
    protocol:module_audit_rollback_append_payload_hash_envelope_mismatch_no_dispatch
      serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_append_payload_hash
    protocol:module_audit_rollback_append_intent_envelope_mismatch_no_dispatch
      serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_append_intent
    protocol:module_audit_rollback_write_boundary_envelope_mismatch_no_dispatch
      serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_write_boundary

Selftest predicates and needles are complete by these exact source blocks:

| prefix | source lines | count | exact needle content |
| --- | ---: | ---: | --- |
| protocol:module_audit_rollback_availability_selftest_ | 487-507 | 20 | schema ...availability_selftest.v0; classification local_only; mutates_global_event_log false; creates_durable_audit_records false; creates_rollback_plans false; installs_rollback_plan false; case_count 8; passed true; cases missing_ledger_and_store_current_boot, durable_audit_ledger_previous_boot, durable_audit_ledger_wrong_schema, durable_audit_ledger_provenance_missing, rollback_store_previous_boot, rollback_store_wrong_schema, rollback_store_provenance_missing, available_facts_policy_still_denied; status denied_missing_durable_write_policy; reason durable_write_policy_missing; can_load false; load_attempted false |
| protocol:module_audit_rollback_write_policy_selftest_ | 509-533 | 24 | schema ...write_policy_selftest.v0; common no-write needles; case_count 12; passed true; cases missing_policy_pair_current_boot, durable_write_policy_previous_boot, durable_write_policy_wrong_schema, durable_write_policy_provenance_missing, durable_write_policy_retained_evidence_binding_missing, durable_write_policy_availability_binding_missing, rollback_install_policy_previous_boot, rollback_install_policy_wrong_schema, rollback_install_policy_provenance_missing, rollback_install_policy_retained_evidence_binding_missing, rollback_install_policy_availability_binding_missing, available_policy_facts_writer_still_denied; status denied_write_path_unimplemented; reasons durable_audit_rollback_writer_unimplemented; can_load false; load_attempted false |
| protocol:module_audit_rollback_storage_layout_selftest_ | 535-566 | 31 | schema ...storage_layout_selftest.v0; common no-write needles; case_count 18; passed true; cases missing_storage_inputs_current_boot, persistence_device_previous_boot, persistence_device_wrong_schema, persistence_device_provenance_missing, persistence_device_stable_identity_missing, block_device_identity_missing, persistence_sector_read_path_missing, persistence_block_driver_missing, persistence_partition_inventory_missing, audit_rollback_storage_layout_previous_boot, audit_rollback_storage_layout_wrong_schema, audit_rollback_storage_layout_provenance_missing, storage_layout_device_binding_missing, audit_ledger_layout_region_missing, rollback_store_layout_region_missing, storage_layout_append_slots_missing, storage_layout_recovery_boundary_missing, available_storage_layout_still_non_authorizing; reasons persistence_device_inventory_missing_and_storage_layout_missing, audit_rollback_storage_layout_available; can_load false; load_attempted false |
| protocol:module_audit_rollback_append_engine_selftest_ | 568-596 | 29 | schema ...append_engine_selftest.v0; common no-write needles; case_count 16; passed true; cases missing_append_engine_pair_current_boot, audit_ledger_append_engine_previous_boot, audit_ledger_append_engine_wrong_schema, audit_ledger_append_engine_provenance_missing, audit_ledger_append_engine_storage_layout_binding_missing, audit_ledger_append_engine_write_policy_binding_missing, audit_ledger_append_engine_append_only_missing, audit_ledger_append_engine_flush_support_missing, audit_ledger_append_engine_recovery_boundary_missing, rollback_store_transaction_engine_previous_boot, rollback_store_transaction_engine_wrong_schema, rollback_store_transaction_engine_provenance_missing, rollback_store_transaction_engine_storage_layout_binding_missing, rollback_store_transaction_engine_write_policy_binding_missing, rollback_store_transaction_engine_replay_support_missing, available_append_engines_still_non_authorizing; reasons audit_ledger_append_engine_missing_and_rollback_store_transaction_engine_missing, audit_rollback_append_engine_available; can_load false; load_attempted false |
| protocol:module_audit_rollback_append_contract_selftest_ | 599-636 | 37 | schema ...append_contract_selftest.v0; common no-write needles; case_count 24; passed true; cases missing_append_envelope_pair_current_boot, audit_append_envelope_previous_boot, audit_append_envelope_wrong_schema, audit_append_envelope_provenance_missing, audit_append_envelope_provenance_binding_missing, audit_append_envelope_policy_binding_missing, audit_append_envelope_write_policy_id_missing, audit_append_envelope_availability_binding_missing, audit_append_envelope_availability_id_missing, audit_append_envelope_storage_layout_id_missing, audit_append_envelope_append_engine_id_missing, audit_ledger_storage_layout_missing, rollback_transaction_envelope_previous_boot, rollback_transaction_envelope_wrong_schema, rollback_transaction_envelope_provenance_missing, rollback_transaction_envelope_provenance_binding_missing, rollback_transaction_envelope_policy_binding_missing, rollback_transaction_envelope_write_policy_id_missing, rollback_transaction_envelope_availability_binding_missing, rollback_transaction_envelope_availability_id_missing, rollback_transaction_envelope_storage_layout_id_missing, rollback_transaction_envelope_append_engine_id_missing, rollback_store_storage_layout_missing, available_envelopes_append_engine_still_missing; reasons audit_append_envelope_missing_and_rollback_transaction_envelope_missing, audit_ledger_append_engine_missing; status missing; can_load false; load_attempted false |
| protocol:module_audit_rollback_append_payload_hash_selftest_ | 638-671 | 33 | schema ...payload_hash_selftest.v0; common no-write needles; case_count 20; passed true; cases missing_payload_hash_pair_current_boot, audit_record_payload_hash_previous_boot, audit_record_payload_hash_wrong_schema, audit_record_payload_hash_provenance_missing, audit_record_payload_hash_retained_binding_missing, audit_record_payload_hash_service_slot_binding_missing, audit_record_payload_hash_write_request_binding_missing, audit_record_payload_hash_append_contract_binding_missing, audit_record_payload_hash_target_schema_binding_missing, audit_record_payload_hash_missing, audit_record_retained_audit_rollback_missing, audit_record_service_slot_reservation_missing, audit_record_append_contract_missing, rollback_transaction_payload_hash_previous_boot, rollback_transaction_payload_hash_wrong_schema, rollback_transaction_payload_hash_provenance_missing, rollback_transaction_payload_hash_append_contract_binding_missing, rollback_transaction_payload_hash_missing, rollback_transaction_append_contract_missing, available_payload_hashes_still_non_authorizing; reasons audit_record_append_payload_hash_missing_and_rollback_transaction_append_payload_hash_missing, audit_rollback_append_payload_hash_available; status available; can_load false; load_attempted false |
| protocol:module_audit_rollback_append_intent_selftest_ | 673-706 | 33 | schema ...append_intent_selftest.v0; common no-write needles; case_count 20; passed true; cases missing_append_intent_pair_current_boot, audit_record_append_intent_previous_boot, audit_record_append_intent_wrong_schema, audit_record_append_intent_provenance_missing, audit_record_append_intent_provenance_binding_missing, audit_record_append_intent_append_contract_binding_missing, audit_record_append_intent_append_engine_binding_missing, audit_record_append_intent_storage_layout_binding_missing, audit_record_append_intent_write_policy_binding_missing, audit_record_append_intent_availability_binding_missing, audit_record_append_intent_payload_hash_missing, audit_record_append_intent_append_contract_missing, audit_record_append_intent_payload_hash_envelope_missing, rollback_transaction_append_intent_previous_boot, rollback_transaction_append_intent_wrong_schema, rollback_transaction_append_intent_provenance_missing, rollback_transaction_append_intent_append_contract_binding_missing, rollback_transaction_append_intent_payload_hash_missing, rollback_transaction_append_intent_payload_hash_envelope_missing, available_append_intents_still_non_authorizing; reasons audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing, audit_rollback_append_intent_available; status available; can_load false; load_attempted false |
| protocol:module_write_boundary_selftest_ | 708-755 | 47 | schema ...write_boundary_selftest.v0; classification local_only; mutates_global_event_log false; creates_durable_audit_records false; creates_rollback_plans false; installs_rollback_plan false; loads_recovery_artifact false; case_count 22; passed true; cases missing_manifest_reference, stale_artifact_reference, substituted_vm_report_reference, previous_boot_write_request, write_request_schema_mismatch, missing_computed_grant_reference, local_attestation_hash_mismatch, local_approval_hash_mismatch, audit_record_service_slot_hash_mismatch, rollback_plan_service_slot_hash_mismatch, substituted_service_slot_reference, recovery_artifact_loader_requested, durable_audit_ledger_available_rollback_store_missing, rollback_store_available_durable_audit_ledger_missing, availability_facts_present_policy_still_denied, durable_write_policy_available_rollback_policy_missing, policy_facts_available_append_contract_missing, audit_append_available_rollback_transaction_missing, append_contract_available_append_intent_missing, append_intent_payload_hash_envelope_missing, append_intents_available_writer_still_denied, accepted_current_boot_preconditions_write_still_denied; statuses denied_missing_durable_write_policy, denied_missing_rollback_install_policy, denied_missing_append_contract, denied_missing_append_intent, denied_write_path_unimplemented, denied_missing_durable_write_boundary; reasons durable_write_policy_missing, rollback_install_policy_missing, audit_append_envelope_missing_and_rollback_transaction_envelope_missing, rollback_transaction_envelope_missing, audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing, audit_record_append_payload_hash_envelope_missing, durable_audit_rollback_writer_unimplemented, durable_audit_write_missing_and_rollback_install_missing; can_load false; load_attempted false |

Common means the exact literal needles in that profile block:
classification local_only, mutates_global_event_log false,
creates_durable_audit_records false, creates_rollback_plans false,
installs_rollback_plan false, can_load false, and load_attempted false.
The source spans and counts are part of the complete manifest; no replacement
generic assertion is intended.

Expected full-profile drop: 733 predicates, resulting estimate 7,137:

    availability 45
    write_policy 51
    storage_layout 183
    append_engine 58
    append_contract 116
    append_payload_hash 78
    append_intent 86
    write_boundary 116
    total 733

## 3. Supersession table

The replacement is not another generic diagnostic. Each real writer calls one
own-pinned raios-core evaluator and uses the shared scan -> plan -> write
readback -> reparse -> rescan path only after that evaluator authorizes the
exact method, target, schema, marker, and policy.

| retired method family | real M7/M9 surface | existing green evidence |
| --- | --- | --- |
| availability (+ selftest) | all scoped append evaluators: seed_data, promotion_transaction, recovery_action, recovery_load, artifact_persist, memory_record, repromotion, and scoped_rollback_authorized_append | persistence:*, m8-lifeline:*, memory-durable:*, hello_rollback_dry_run:*; reports shadow-persistence-reboot-20260708-023535-14288.json, shadow-20260710-034549-24228.json, shadow-20260712-184533-27876.json |
| write_policy (+ selftest) | evaluator-owned policy and authority inputs; no shared writes_enabled flip | persistence:*, m8-lifeline:*, memory-durable:*, hello_rollback_dry_run:* |
| storage_layout (+ selftest) | caller-owned region and readback checks in durable_store.rs, boot_control.rs, artifact_store.rs, repromotion.rs, and hello scoped rollback | persistence-reboot:*, hello_rollback_dry_run:*, memory-durable:* |
| append_engine (+ selftest) | shared real reclog append implementation, authorized by each caller evaluator | persistence:*, m8-lifeline:*, memory-durable:*, repromotion-* |
| append_contract (+ selftest) | typed frame/record contract at each scoped call; no generic audit/rollback pair | persistence:*, m8-lifeline:*, memory-durable:*, hello_rollback_dry_run:* |
| append_payload_hash (+ selftest) | canonical frame hash plus post-write readback hash in real callers | persistence:*, memory-durable:*, hello_rollback_dry_run:* |
| append_intent (+ selftest) | evaluator decision plus caller-built typed record/frame is the intent boundary | persistence:*, m8-lifeline:*, memory-durable:*, repromotion-* |
| write_boundary (+ selftest) | deliberately split among the scoped evaluators and their callers; no broad replacement exists | persistence-reboot:*, m8-lifeline:*, memory-durable:*, hello_rollback_dry_run:*; full report shadow-20260712-184856-27972.json |

No retired method is needed by the real M7/M9 writers. The old diagnostic
schemas are not retained. The hello projection remains BLOCKING until replaced.

## 4. Owner-confirmation checklist

- Delete 9,575 lines and 16 generic diagnostic methods that only describe the
  old unavailable shared writer.
- Keep fail-closed authority: real writes remain authorized only by own-scoped
  evaluator and caller evidence.
- Full evidence intentionally drops from 7,870 to about 7,137 predicates; the
  named checks are retirement, not lost capability.
- Deletion is blocked because live hello code still imports contract, engine,
  and storage-layout surfaces.
- Approve only after the hello path is cut to scoped rollback evidence in the
  same source+harness slice.

## 5. Packet decomposition

Packet count: one conditional deletion packet, not executable while blocked.

REFACTOR-P3-WB-DELETE source write set:

    delete the ten family files
    seed-kernel/src/agent_protocol.rs
    seed-kernel/src/main.rs
    seed-kernel/src/hello_service.rs
    seed-kernel/src/hello_service/{constants,emitters,records,
      rollback_authority_gates,rollback_writer_bindings,rollback_writer_gate,
      storage_authority_gate}.rs

The same slice must edit:

    vm-harness/shadow-vm-smoke-profile-full-module-audit-rollback.ps1
    vm-harness/shadow-vm-smoke-profile-full-module-selftests.ps1
    vm-harness/shadow-vm-smoke-profile-persistence.ps1
    vm-harness/shadow-vm-smoke-profile-quick.ps1
    vm-harness/shadow-vm-smoke.ps1

Checks: grep proves no live import/MethodEntry/command/predicate remains; the
old focused module-audit-rollback profile no longer exists; replacement
evidence is persistence/persistence-reboot, m8-lifeline, memory-durable, and
hello_rollback_dry_run; run the changed focused profile, then full at close and
secret scan. Do not invent a denial-only replacement profile.

## 6. Risks and out-of-scope observations

- .github/, scripts/, tools/, and ci/ have no direct retired file/method hit,
  but CI consumes quick indirectly.
- docs/DEBUGGING.md has 48 module.audit_rollback_ references and active
  diagnostic prose; update or mark historical later, outside this write set.
- PROJECT_STATUS, ROADMAP, M2/M3/M7 reviews, the session retrospective, and
  archived roadmap contain historical names; do not bulk-delete history.
- seed-kernel/build.rs hashes all 20 hello files. This packet does not delete
  hello files, but the hello cut must preserve the ordered source set.
- The inventory's memory -> legacy recovery_execution dependency is outside
  this first family. Event-log relocation is also outside this packet.

## Required command evidence

No build, QEMU, deletion, stage, or commit was run. Initial git status:

    ?? docs/assets/screenshots/enum-console-shot.png
    ?? docs/assets/screenshots/iommu-wifi-shot.png
    ?? docs/assets/screenshots/raios-genesis-live-preview.img
    ?? docs/assets/screenshots/raios-stage0-preview.img
    ?? docs/assets/screenshots/set-wifi-scan-shot.png
    ?? docs/assets/screenshots/ui-pill-detail-shot.png
    ?? release/usb-write-result.txt

Final status observed after writing this document:

    M Cargo.toml
    M seed-kernel/src/agent_protocol_module_loader_runtime.rs
    ?? docs/plan-reviews/kernel-mass-refactor-p2-design-2026-07-12.md
    ?? docs/plan-reviews/kernel-mass-refactor-p3-design-2026-07-12.md
    ?? docs/assets/screenshots/enum-console-shot.png
    ?? docs/assets/screenshots/raios-genesis-live-preview.img
    ?? docs/assets/screenshots/raios-stage0-preview.img
    ?? docs/assets/screenshots/set-wifi-scan-shot.png
    ?? docs/assets/screenshots/ui-pill-detail-shot.png
    ?? release/usb-write-result.txt
    ?? seed-kernel/src/agent_protocol_module_loader_runtime/

The P2/Cargo/loader-runtime entries appeared concurrently and are out of
scope; they were preserved. No source or harness entry for this P3 packet
family was changed.
