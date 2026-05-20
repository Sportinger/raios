# raiOS Module VM Test Report Reference V0

`raios.module_vm_test_report_reference.v0` is a local-only, RAM-only
current-boot hash reference for VM-test-report evidence. It is evidence for the
denied module load gate, not report JSON, not a signature, and not authority to
load guest code.

## Guest Diagnostic

```text
agent module.vm_report_diagnostic
agent module.vm_report_diagnostic <report_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
agent module.vm_report_diagnostic_selftest
```

With no arguments the diagnostic reports the VM-test-report reference as absent.
With arguments it validates only hashes and current-boot event ids. The guest
does not accept VM-report JSON, artifact bytes, manifest JSON, signed blobs, or
service code through this method.

The diagnostic response schema is
`raios.module_vm_test_report_reference_diagnostic.v0`. A valid command records a
local-only `module.vm_test_report_reference.retained` event with a
`raios.module_vm_test_report_reference.v0` binding, then returns it as
`retained_vm_test_report_reference`. That binding keeps:

```text
classification: local_only
accepts_vm_report_json: false
accepts_artifact_bytes: false
authorizes_guest_load: false
can_load_now: false
service_inventory_change: none
load_attempted: false
```

## Canonical Hash

The canonicalization is
`raios.module_vm_test_report_reference.canonical.v0`.

The hash input is exactly:

```text
canonicalization=raios.module_vm_test_report_reference.canonical.v0
schema=raios.module_vm_test_report_reference.v0
requested_capability=cap.module.load_ephemeral
load_mode=ram_only
subject=agent.session.serial
resource=live_service_graph
scope=current_boot
retained_manifest_reference_event_id=<event.current_boot.NNNNNNNN>
retained_artifact_reference_event_id=<event.current_boot.NNNNNNNN>
retained_reference_event_id=<event.current_boot.NNNNNNNN>
manifest_reference_sha256=<hex>
artifact_reference_sha256=<hex>
manifest_sha256=<hex>
candidate_artifact_sha256=<hex>
computed_capability_grant_sha256=<hex>
vm_test_report_sha256=<hex>
local_attestation_sha256=<hex>
accepts_vm_report_json=false
accepts_artifact_bytes=false
loads_artifact=false
authorizes_guest_load=false
service_inventory_change=none
load_attempted=false
```

`retained_reference_event_id` is the retained
`raios.module_computed_grant_reference.v0` event id. The VM-report reference is
accepted only when the referenced manifest, candidate-artifact, and
computed-grant events are the latest live current-boot retained records and all
hashes match those retained records.

## Load Gate Behavior

After a valid reference is retained, `module.load_ephemeral` and
`service.load_ephemeral` still return `capability_denied`. The denied
`raios.module_load_gate.v0` response reports:

```text
vm_test_report: retained_hash_reference_only
retained_vm_test_report_reference.state: present
retained_vm_test_report_reference.schema: raios.module_vm_test_report_reference.v0
retained_vm_test_report_reference.status: retained_hash_reference_load_still_denied
reason: retained_vm_test_report_reference_not_authorizing
vm_test_report_reference_hash: sha256:<reference hash>
vm_test_report_hash: sha256:<report hash>
can_load: false
load_attempted: false
```

If the latest retained VM-report reference is stale, wrong-schema, substituted,
hash-mismatched, or no longer matches the retained manifest, artifact, or
computed-grant references, the live gate reports
`vm_test_report: rejected_retained_reference` and does not expose the report hash
as accepted evidence.

## Selftests

`module.vm_report_diagnostic_selftest` emits
`raios.module_vm_test_report_reference_diagnostic_selftest.v0`. It is local test
infrastructure only and must not mutate the global event log or create retained
records.

`module.load_gate_vm_report_selftest` emits
`raios.module_load_gate_vm_report_selftest.v0`. It covers missing,
accepted-current-boot-but-denied, stale/dropped,
previous-boot-or-unretained, wrong-schema, substituted-record,
reference-hash-mismatch, manifest-reference mismatch, artifact-reference
mismatch, computed-grant-reference mismatch, and VM-report-hash mismatch cases.

Both selftests keep:

```text
mutates_global_event_log: false
creates_retained_vm_test_report_reference_records: false
accepts_vm_report_json: false
accepts_artifact_bytes: false
loads_artifact: false
service_inventory_change: none
load_attempted: false
can_load: false
```
