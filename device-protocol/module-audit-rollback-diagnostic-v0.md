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
  --denial-event-id event.current_boot.<denied-load-id> `
  --retained-reference-event-id event.current_boot.<retained-grant-id> `
  --ram-only-service-slot-id ram_only:svc.example.0001 `
  --pre-load-service-inventory-hash sha256:<inventory-hash> `
  --cleanup-actions-hash sha256:<cleanup-actions-hash>
```

The command first re-runs the
`raios.computed_capability_grant.v0` tuple validation. A valid audit/rollback
diagnostic must bind the same manifest, artifact, VM report, local attestation,
approval phrase, retained grant hash, retained reference event id, denied load
event id, rollback plan hash, and ram-only service slot id.
When a guest-retained `raios.module_vm_test_report_reference.v0` exists, the
live load gate requires the audit/rollback `vm_test_report_hash` to align with
that retained VM-report reference before reporting the full chain as
non-authorizing evidence.

The tool exits non-zero for rejected evidence unless `--allow-invalid` is
provided.

## Guest Hash-Reference Diagnostic

Stage-0 can now inspect the host candidate as hashes only:

```text
agent module.audit_rollback_diagnostic
agent module.audit_rollback_diagnostic <audit_record_hash> <rollback_plan_hash> <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> <pre_load_service_inventory_hash> <cleanup_actions_hash> <denial_event_id> <retained_reference_event_id> <ram_only_service_slot_id> [current_boot]
agent module.audit_rollback_diagnostic_selftest
```

The response schema is
`raios.module_audit_rollback_reference_diagnostic.v0`. The method does not
accept artifact bytes, manifest JSON, VM-report JSON, attestation JSON, local
approval text, audit-record JSON, or rollback-plan JSON. It recomputes the
canonical grant, rollback-plan, and audit-record hashes from the supplied hash
references and current-boot ids only.

A valid reference reports:

```text
validation_status: valid_hash_reference_load_still_denied
audit_record_hash_reference_present: true
rollback_plan_hash_reference_present: true
durable_audit_written: false
rollback_plan_installed: false
grants_capability: false
grants_load_now: false
authorizes_guest_load: false
can_load_now: false
service_inventory_change: none
load_attempted: false
loader: unavailable
service_slot: unallocated
global_event_log_mutation: valid_hash_reference_retention_only
retained_audit_rollback_reference.schema: raios.module_audit_rollback_reference.v0
retained_audit_rollback_reference.status: retained_hash_reference_load_still_denied
```

A valid reference is retained as one local-only, RAM-only current-boot
`raios.module_audit_rollback_reference.v0` event binding. The retained record
stores hashes and current-boot ids only. It is diagnostic evidence for the
current boot, not durable authority: it does not write durable audit records,
install rollback plans, allocate service slots, load artifacts, or change
`service.inventory.v0`.

The follow-on guest `module.service_slot_diagnostic` path may bind this retained
audit/rollback reference to a canonical
`raios.module_service_slot_reservation.v0` hash reference. That service-slot
reservation record is also non-authorizing and still allocates no slot.

Invalid or absent references keep `mutates_global_event_log: false` and
`global_event_log_mutation: none`.

The guest diagnostic validates hash shape and current-boot id syntax. The later
live `module.load_ephemeral`/`service.load_ephemeral` gate revalidates the
retained record against the RAM event log before treating it as accepted
evidence. That live check requires the referenced retained computed-grant event,
a prior denied load event, canonical computed-grant/rollback/audit hashes, and
a valid `ram_only:` slot id; rejected live references do not expose
audit/rollback hashes as accepted load-gate evidence.

`module.audit_rollback_diagnostic_selftest` emits
`raios.module_audit_rollback_reference_diagnostic_selftest.v0` and covers
absent references, accepted current-boot references, stale scope, previous-boot
event ids, audit/rollback schema mismatches, substituted audit hashes,
rollback-plan hash mismatches, computed-grant hash mismatches, and invalid
ram-only service-slot ids. The selftest remains local test infrastructure and
does not create retained records.

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

These tests prove the host diagnostic fails closed before the guest
hash-reference path can inspect the candidates. They do not create guest loader
state.

## Boundary

The host output may become input to the guest hash-reference diagnostic. A valid
guest reference can be retained as current-boot RAM evidence, but it still
cannot satisfy `module.load_ephemeral`. Recovery artifacts remain under
`raios.recovery.v0`, not this normal module load gate.
