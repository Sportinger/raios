# Module Candidate Artifact Reference V0

`raios.module_candidate_artifact_reference.v0` is a local-only, RAM-only
current-boot hash reference for the candidate artifact named by an already
retained manifest and computed-grant tuple. It is evidence for the denied load
gate, not authority to load bytes.

Guest commands:

```text
agent module.artifact_diagnostic
agent module.artifact_diagnostic <artifact_reference_hash> <retained_manifest_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <manifest_hash> <computed_grant_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
agent module.artifact_diagnostic_selftest
```

The canonical hash is `raios.module_candidate_artifact_reference.canonical.v0`
over these ordered fields:

```text
canonicalization=raios.module_candidate_artifact_reference.canonical.v0
schema=raios.module_candidate_artifact_reference.v0
requested_capability=cap.module.load_ephemeral
load_mode=ram_only
subject=agent.session.serial
resource=live_service_graph
scope=current_boot
retained_manifest_reference_event_id=<event.current_boot.NNNNNNNN>
retained_reference_event_id=<event.current_boot.NNNNNNNN>
manifest_reference_sha256=<sha256>
manifest_sha256=<sha256>
computed_capability_grant_sha256=<sha256>
candidate_artifact_sha256=<sha256>
vm_test_report_sha256=<sha256>
local_attestation_sha256=<sha256>
accepts_artifact_bytes=false
loads_artifact=false
authorizes_guest_load=false
service_inventory_change=none
load_attempted=false
```

A valid reference records `module.artifact_reference.retained` with
`classification: local_only`. `module.load_ephemeral` may then report
`candidate_artifact: retained_hash_reference_only`, but `artifact_loaded`,
`service_started`, `can_load`, and `load_attempted` remain false.
