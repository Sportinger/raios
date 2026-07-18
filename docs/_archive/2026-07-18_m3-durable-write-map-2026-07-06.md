# M3 First Durable Write — Design Map (2026-07-06)

Read-only scoping analysis (packet M3-1): the RAIOS_AUDITRB_V0 region
contract, the complete denial-chain table with file:line evidence, the
minimal single-use hello-scoped grant design, the append + readback +
verify + apply transaction flow, a 5-slice plan, expected harness needle
flips (denied → authorized, scoped only), and the authority-leak risks.
Cardinal rule: no shared writes_enabled / generic boundary flips —
everything outside the exact scoped path stays fail-closed.

**Region Contract**

`docs/image-layout-v0.md` does not define `RAIOS_AUDITRB_V0`. It explicitly keeps the current Stage-0 image as a single FAT32 boot image and read-only at runtime: [docs/image-layout-v0.md](C:/Users/admin/Documents/raios2/docs/image-layout-v0.md:6), [docs/image-layout-v0.md](C:/Users/admin/Documents/raios2/docs/image-layout-v0.md:53), [docs/image-layout-v0.md](C:/Users/admin/Documents/raios2/docs/image-layout-v0.md:314). So M3’s audit/rollback target is a harness-provisioned dedicated current-boot test disk, not the future normal `SEED_DATA` persistence path.

Current on-disk contract:

- Harness creates a 1 MiB raw target image and writes ASCII `RAIOS_AUDITRB_V0` at byte 0: [vm-harness/shadow-vm-smoke.ps1](C:/Users/admin/Documents/raios2/vm-harness/shadow-vm-smoke.ps1:105).
- Runner wires it as IDE raw drive `raiosauditrollback0` on `ide.2,unit=0`: [scripts/run-stage0-qemu.ps1](C:/Users/admin/Documents/raios2/scripts/run-stage0-qemu.ps1:67).
- Kernel constants define 512-byte sectors, marker LBA `0`, append region start LBA `1`, region count `1`: [seed-kernel/src/ahci.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/ahci.rs:47), [seed-kernel/src/ahci.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/ahci.rs:55).
- Stable target IDs are `append.audit_ledger.current_boot` / `raios.audit_record.v0` and `append.rollback_store.current_boot` / `raios.rollback_transaction.v0`: [agent_protocol_module_write_boundary_storage_layout.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_storage_layout.rs:173).
- Sector image format is already planned by hello rollback code: audit record at offset `0`, rollback transaction immediately after it, zero padding to 512 bytes: [rollback_writer_gate.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_writer_gate.rs:1218). Current harness needles assert total record length `480` and padding `32`: [shadow-vm-smoke-profile-quick.ps1](C:/Users/admin/Documents/raios2/vm-harness/shadow-vm-smoke-profile-quick.ps1:2842).

**Denial Chain**

| Area | Current gate | Denial / evidence |
|---|---|---|
| Generic mutation denial | [agent_protocol.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol.rs:662), [agent_protocol_policy.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_policy.rs:20) | Mutating methods emit `capability_denied`; `service.rollback_apply` maps to `cap.service.rollback_apply.current_boot`. |
| Durable audit availability | [availability.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_availability.rs:198), [availability.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_availability.rs:230) | Missing durable audit ledger and rollback store; reasons include `durable_audit_ledger_missing_and_rollback_store_missing`, `denied_missing_durable_write_policy`. |
| Durable write policy | [write_policy.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_write_policy.rs:221), [write_policy.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_write_policy.rs:255) | Missing durable write policy and rollback install policy; `authorizes_write: false`. |
| Storage / block write authority | [storage_layout.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_storage_layout.rs:228), [storage_layout.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_storage_layout.rs:1032) | Target label scan proves LBA1 region, bounds, non-scratch, no metadata overlap; still `authorizes_append: false`, `writes_enabled: false`, `write_attempted: false`. |
| Append engine | [append_engine.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_append_engine.rs:304), [append_engine.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_append_engine.rs:346) | Missing audit ledger append engine and rollback store transaction engine; requires append-only, flush, replay, recovery separation. |
| Append contract | [append_contract.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_append_contract.rs:500), [append_contract.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_append_contract.rs:670) | Scratch range ready but not durable authority; target region ready but missing media/durable audit authority. |
| Payload hash | [append_payload_hash.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_append_payload_hash.rs:490) | Audit and rollback payload hashes are evidence only; `authorizes_append_intent: false`, `authorizes_write: false`. |
| Append intent | [append_intent.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_append_intent.rs:453) | Missing append intents or evidence-only intents; no writer authority. |
| Write boundary | [boundary.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_boundary.rs:1053), [boundary.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_write_boundary_boundary.rs:1218) | Aggregates all gates; current final denial is `denied_missing_durable_write_boundary` / `durable_audit_write_missing_and_rollback_install_missing`, or `denied_write_path_unimplemented`. |
| Hello transaction append | [constants.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/constants.rs:205), [constants.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/constants.rs:347), [constants.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/constants.rs:363) | `denied_missing_durable_append_authority`, `denied_missing_rollback_transaction_append_authority`, then `blocked_by_transaction_append_authority_denial_gate`. |
| Hello rollback apply | [state_machine.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/state_machine.rs:416), [runtime.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/runtime.rs:249), [constants.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/constants.rs:136) | Always records apply as denied: `denied_missing_rollback_apply_authority`; apply source evidence may verify, but `rollback_apply_authorized` remains false. |

**Minimal Grant Design**

Do not flip global `writes_enabled`, generic module write boundary, scratch authority, or read-only boot driver flags.

The minimal M3 grant is a single-use, hello-scoped transaction append authority over:

- method: `service.rollback_apply`
- service: `svc.demo.hello`
- target region: `target_region.audit_rollback.current_boot`
- marker: `RAIOS_AUDITRB_V0`
- exact span: LBA `1`, count `1`, byte count `512`
- exact sector image hash, audit record hash, rollback transaction hash, offsets, and zero-padding plan
- exact current hot-swap probation / rollback preview state
- retained inspect source and target sector inspection hashes

The gate values that should flip only in that scoped path:

- `transaction_append_available: false -> true`
- `missing_transaction_append_authority: true -> false`
- `authorizes_media_write`, `authorizes_append`, `authorizes_transaction_append`: `false -> true`
- `writes_durable_audit_log`, `writes_rollback_store`, `appends_rollback_transaction`: `false -> true`
- `blocked_by_authority_denial_gate: true -> false`
- `rollback_apply_authorized: false -> true`
- `applies_rollback: false -> true`, but only after readback and sector inspection pass

Everything else stays fail-closed: scratch remains test-only, boot media remains read-only, external unsigned intake remains denied, rollback application without the exact transaction proof remains denied, and generic module audit/rollback diagnostics should not become broad write authority.

**Transaction Flow**

1. `service.rollback_apply` validates hello probation/preview state and retained descriptor/artifact evidence.
2. Reuse `hello_rollback_append_record_dry_run` and sector planner to build the audit record + rollback transaction image: [rollback_writer_gate.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_writer_gate.rs:1104), [rollback_writer_gate.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_writer_gate.rs:1218).
3. Verify target region discovery: label found, LBA1/count1, 512-byte sector, non-scratch, no boot/partition/GPT overlap.
4. Evaluate the scoped grant against the exact sector plan and hashes.
5. Perform the real append by calling the existing target-region AHCI path, not a new driver path: [ahci.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/ahci.rs:771). The current hello path already exercises this as “dry-run” evidence: [rollback_authority_gates.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_authority_gates.rs:3651).
6. Read back and inspect LBA1 with expected sector hash, audit hash, rollback hash, offsets, and zero padding: [ahci.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/ahci.rs:810), [ahci.rs](C:/Users/admin/Documents/raios2/seed-kernel/src/ahci.rs:2221).
7. Only after inspection succeeds, apply the hello rollback state transition and emit the applied state citing the transaction hash, sector write/readback hash, and inspection hash.
8. On any mismatch or missing evidence, keep current service state and return explicit denial/failure.

**Slice Plan**

1. Scope evaluator: add the narrow positive authorization decision over existing evidence. Verify with unit tests plus `hello-rollback-dry-run` profile.
2. Authorized append: route the positive gate into the existing target LBA1 write/readback path. Verify `hello-rollback-dry-run`.
3. Verified apply: after readback/inspection, mutate hello state and emit applied evidence. Verify `hello-rollback-dry-run`.
4. Module profile alignment: keep generic `module.audit_rollback_*` broad writes denied, but add/check the scoped hello append evidence in `module-audit-rollback`.
5. Closure: run focused close profiles, then full Shadow VM profile and secret scan.

Needle changes expected:

- `vm-harness/shadow-vm-smoke-profile-hello-rollback-dry-run.ps1`: lines around 193, 205, 223, 238, 253, 294, 306, 350-353, 372, 385 currently expect blocked/no-append/no-apply; these must flip for the authorized apply path.
- `vm-harness/shadow-vm-smoke-profile-quick.ps1`: lines 2817-2825, 2848-2852, 2862-2863, 2928-2934, 2948 currently assert denied/missing/no append; update only for the scoped hello success.
- `vm-harness/shadow-vm-smoke-profile-full-module-audit-rollback.ps1`: lines 233-235 and 420-436 should stay denied for generic module authority unless that profile is extended with separate scoped-hello success needles. Do not silently turn generic append targets available.

**Risks**

- Biggest risk: flipping shared `writes_enabled`, `authorizes_append`, block driver write support, or module write-boundary availability globally. That would grant more than M3 intends.
- Scratch evidence must never become durable authority.
- The target label sector LBA0 and any boot/GPT/partition metadata must stay unwritable.
- The LBA1 contract is currently one sector, so this is a first transaction slot, not a general multi-entry log.
- Current target write/readback is named “dry-run” while it already writes the target test disk. M3 should make the authority-bearing write explicit so future agents do not confuse test-media evidence with durable policy.
- Hello source/descriptor changes will affect the attestation/signing chain. Any edits to hello artifacts or descriptor identity need the existing re-sign/build flow, not hand-edited hashes.