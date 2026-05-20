# raiOS Computed Capability Grant V0

`raios.computed_capability_grant.v0` is the first host-side diagnostic for a
future `cap.module.load_ephemeral` grant. It is a computed evidence candidate,
not execution authority.

The diagnostic hashes and validates an exact tuple:

```text
raios.module_manifest.v0
candidate artifact bytes
raios.vm_test_report.v0
raios.local_attestation.v0
requested_capability: cap.module.load_ephemeral
load_mode: ram_only
subject
resource: live_service_graph
scope: current_boot
```

Even when the tuple is valid, Stage-0 still cannot load code. The diagnostic
must report:

```text
grants_capability: false
grants_load_now: false
authorizes_guest_load: false
can_load_now: false
service_inventory_change: none
load_attempted: false
loader: unavailable
service_slot: unallocated
```

`computed_candidate_present: true` only means the host tool validated the
manifest/artifact/report/attestation tuple and computed a stable diagnostic
hash. It does not satisfy the live `module.load_ephemeral` gate. That mutating
method still returns `raios.module_load_gate.v0` with no load authority until
durable audit, rollback, a loader, and ram-only service-slot evidence exist.

## Canonical Hash

The canonical hash uses
`raios.computed_capability_grant.canonical.v0` and binds the request and evidence
fields:

```text
requested_capability
load_mode
subject
resource
scope
manifest_sha256
candidate_artifact_sha256
vm_test_report_sha256
local_attestation_sha256
grants_load_now=false
authorizes_guest_load=false
service_inventory_change=none
load_attempted=false
```

The output field is:

```json
{
  "computed_capability_grant_hash": "sha256:..."
}
```

## Host Tool

Compute the diagnostic with:

```powershell
cargo run -p registry-tools -- grant-diagnostic `
  --manifest .\candidate.manifest.json `
  --artifact .\candidate.bin `
  --vm-report .\release\vm-reports\shadow-....json `
  --local-attestation .\release\attestations\attest-....json `
  --approval "APPROVE RAM_ONLY <tuple-prefix>"
```

The tool exits non-zero for rejected evidence unless `--allow-invalid` is
provided. Rejected diagnostics still keep `grants_load_now: false` and
`load_attempted: false`.

## Guest Hash-Reference Diagnostic

Stage-0 can now inspect a computed grant hash reference through a read-only
serial method:

```text
agent module.grant_diagnostic
agent module.grant_diagnostic <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
```

The result schema is `raios.module_computed_grant_diagnostic.v0`. The method
does not accept artifact bytes, manifest JSON, VM-report JSON, or attestation
JSON. It only recomputes the canonical grant hash from the supplied hashes and
reports whether the reference is absent, valid for `current_boot`, stale, or
mismatched.

A valid reference is retained in the RAM-only current-boot event log as a
local-only `raios.module_computed_grant_reference.v0` binding. The response
also reports a `retained_reference` object with the event id, echoed hashes,
and `status: retained_hash_reference_load_still_denied`. The retained record is
evidence for diagnostics only; it is not a durable audit record and does not
grant execution.

If `module.load_ephemeral` is called later in the same boot, its denied
`raios.module_load_gate.v0` response snapshots the retained reference as
`retained_computed_grant_reference` and reports
`computed_capability_grant: retained_hash_reference_only`. The denial reason is
still `retained_computed_grant_reference_not_authorizing` until durable audit,
rollback, loader, and ram-only service-slot evidence exists.

A valid reference reports:

```text
validation_status: valid_hash_reference_load_still_denied
retained_reference.status: retained_hash_reference_load_still_denied
computed_candidate_present: true
grants_capability: false
grants_load_now: false
authorizes_guest_load: false
can_load_now: false
load_attempted: false
service_inventory_change: none
```

`agent module.grant_diagnostic_selftest` emits
`raios.module_computed_grant_diagnostic_selftest.v0`, covering absent,
accepted-current-boot, stale previous-boot, mismatched manifest-hash, and
wrong-policy/hash cases. This is local test infrastructure over the real
hash-reference predicate; it does not create a loader, allocate a service slot,
or mutate the service inventory.

## Required Negative Cases

The host policy rejects at least:

```text
vm_report_manifest_hash_mismatch
vm_report_artifact_hash_mismatch
local_attestation_report_hash_mismatch
local_attestation_hash_mismatch
manifest_granted_caps_non_empty
approval_phrase_mismatch
local_attestation_grants_load_now_true
```

These cases are covered by `registry-core` unit tests. They are host-side tests:
they do not mutate the guest, write provider context, allocate a service slot,
or change `service.inventory.v0`.

The guest-side Shadow VM smoke additionally checks the read-only hash-reference
diagnostic and selftest while keeping the mutating load gate denied.

## Boundary

The normal module load gate and recovery artifact loading stay separate.
Recovery uses `raios.recovery.v0` and must not treat this diagnostic as a
recovery trust record.
