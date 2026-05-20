# raiOS Module Audit/Rollback Diagnostic V0

`raios.module_audit_rollback_diagnostic.v0` is the host-side diagnostic that
builds canonical, non-authorizing candidates for the future module load audit
and rollback gates.

It emits two nested candidate artifacts:

```text
raios.audit_record.v0
raios.rollback_plan.v0
```

These are evidence candidates, not guest state. The host tool does not write a
durable audit ledger, install a rollback plan in the guest, allocate a service
slot, load artifact bytes, or mutate `service.inventory.v0`.

## Host Tool

Compute the diagnostic with:

```powershell
cargo run -p registry-tools -- audit-rollback-diagnostic `
  --manifest .\candidate.manifest.json `
  --artifact .\candidate.bin `
  --vm-report .\release\vm-reports\shadow-....json `
  --local-attestation .\release\attestations\attest-....json `
  --approval "APPROVE RAM_ONLY <tuple-prefix>" `
  --computed-grant-hash sha256:<grant-hash> `
  --denial-event-id event.current_boot.00000031 `
  --retained-reference-event-id event.current_boot.00000027 `
  --ram-only-service-slot-id ram_only:svc.example.0001 `
  --pre-load-service-inventory-hash sha256:<inventory-hash> `
  --cleanup-actions-hash sha256:<cleanup-actions-hash>
```

The command first re-runs the
`raios.computed_capability_grant.v0` tuple validation. A valid audit/rollback
diagnostic must bind the same manifest, artifact, VM report, local attestation,
approval phrase, retained grant hash, retained reference event id, denied load
event id, rollback plan hash, and ram-only service slot id.

The tool exits non-zero for rejected evidence unless `--allow-invalid` is
provided.

## Canonical Rollback Plan Hash

The rollback candidate uses `raios.rollback_plan.canonical.v0` and binds:

```text
schema=raios.rollback_plan.v0
load_mode
scope
artifact_sha256
pre_load_service_inventory_sha256
ram_only_service_slot_id
cleanup_actions_sha256
service_inventory_change=none
load_attempted=false
```

The candidate reports:

```text
installed_in_guest: false
service_inventory_change: none
load_attempted: false
```

## Canonical Audit Record Hash

The audit candidate uses `raios.audit_record.canonical.v0` and binds:

```text
schema=raios.audit_record.v0
requested_capability
load_mode
subject
resource
scope
denial_event_id
retained_reference_event_id
computed_capability_grant_sha256
manifest_sha256
candidate_artifact_sha256
vm_test_report_sha256
local_attestation_sha256
local_approval_sha256
rollback_plan_sha256
ram_only_service_slot_id
grants_load_now=false
authorizes_guest_load=false
service_inventory_change=none
load_attempted=false
```

The candidate reports:

```text
durability: host_diagnostic_not_durable
writes_enabled: false
grants_capability: false
grants_load_now: false
authorizes_guest_load: false
can_load_now: false
service_inventory_change: none
load_attempted: false
```

## Required Negative Cases

`registry-core` tests reject at least:

```text
computed_grant_hash_mismatch
vm_report_manifest_hash_mismatch
vm_report_artifact_hash_mismatch
local_attestation_report_hash_mismatch
local_attestation_hash_mismatch
approval_phrase_mismatch
rollback_plan_hash_mismatch
rollback_service_slot_mismatch
```

These tests prove the host diagnostic fails closed before a future guest
hash-reference path exists. They do not create guest loader state.

## Boundary

The output may become input to a future guest read-only hash-reference
diagnostic. It is not accepted by Stage-0 today and cannot satisfy
`module.load_ephemeral`. Recovery artifacts remain under `raios.recovery.v0`,
not this normal module load gate.
