# raiOS Module Manifest V0

`raios.module_manifest.v0` is the first durable description of an
agent-proposed extension artifact. It is not a trust grant. It describes what an
artifact claims to be, which capabilities it requests, and which hashes must be
bound into VM reports and local attestations before any load decision can be
made.

Stage-0 still returns `capability_denied` for `module.load_ephemeral` and
`module.persist`. `module.load_ephemeral` currently returns
`raios.module_load_gate.v0`, where the manifest is only one missing input among
the exact artifact hash, VM report, local attestation, computed grant, local
approval, durable audit, rollback plan, and ram-only service slot. A valid
manifest only makes an artifact eligible for the evidence flow:

```text
manifest + artifact
-> validate manifest and hash binding
-> shadow VM report binds exact manifest/artifact/base image/QEMU args
-> local attestation records approval and rollback mode
-> future kernel policy may consider ram-only load
```

## Required Fields

```json
{
  "schema": "raios.module_manifest.v0",
  "name": "hello-diagnostic",
  "version": "0.1.0",
  "kind": "guest_diagnostic",
  "target": "raios-stage0",
  "abi": "none",
  "built_by": "agent-session-local",
  "provides": ["diagnostic.hello"],
  "requested_caps": ["cap.system.snapshot.read"],
  "granted_caps": [],
  "risk": "observe",
  "load_mode": "proposal_only",
  "artifact_hash": "sha256:...",
  "base_image_hash": null,
  "manifest_hash": null,
  "test_report_hash": null,
  "tests": ["shadow_vm_protocol_smoke"],
  "rollback_id": null
}
```

Rules:

- `granted_caps` must be empty. Local policy computes effective grants.
- `artifact_hash` must match the bytes passed as `-ArtifactPath`.
- `manifest_hash` is normally null because the hash is computed externally.
- `base_image_hash`, when present, must match the Shadow-VM report's
  `base_image.sha256` and may also be checked directly with `-BaseImagePath`.
- `test_report_hash` may be checked directly with `-TestReportPath`, but it
  must remain null for the current local-attestation flow. The attestation binds
  the VM report hash externally to avoid a self-referential cycle where the
  report hashes the manifest and the manifest hashes the report.
- `load_mode` values are `proposal_only`, `vm_test_only`, `ram_only`, or
  `persistent`.
- `risk` values are `observe`, `diagnose`, `simulate`, `modify_ram`, `persist`,
  or `hardware`.

Validate a manifest:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\validate-module-manifest.ps1 -ManifestPath .\candidate.manifest.json -ArtifactPath .\candidate.bin
```

When concrete evidence files already exist, the validator can also verify the
declared base-image and report hashes:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\validate-module-manifest.ps1 -ManifestPath .\candidate.manifest.json -ArtifactPath .\candidate.bin -BaseImagePath .\raios-stage0-shadow.img -TestReportPath .\shadow-report.json
```

The validator emits `raios.module_manifest_validation.v0`. Invalid manifests
fail the harness before QEMU boots.

After a passing `raios.vm_test_report.v0` and a matching
`raios.local_attestation.v0` exist, the host registry tool can compute
`raios.computed_capability_grant.v0` for the exact evidence tuple. The manifest
still does not grant its own `granted_caps`, and the computed diagnostic does
not authorize Stage-0 loading.

## Guest Hash-Reference Diagnostic

The guest exposes the first manifest-only hash-reference diagnostic:

```text
agent module.manifest_diagnostic
agent module.manifest_diagnostic <manifest_reference_hash> <manifest_hash> [current_boot]
agent module.manifest_diagnostic_selftest
```

`module.manifest_diagnostic` emits
`raios.module_manifest_reference_diagnostic.v0`. It accepts only SHA-256 hash
references, never manifest JSON, artifact bytes, signed blobs, registry
records, or service code. The canonical reference hash is:

```text
canonicalization=raios.module_manifest_reference.canonical.v0
schema=raios.module_manifest_reference.v0
requested_capability=cap.module.load_ephemeral
load_mode=ram_only
subject=agent.session.serial
resource=live_service_graph
scope=current_boot
manifest_schema=raios.module_manifest.v0
manifest_sha256=<64 lowercase hex chars>
authorizes_guest_load=false
service_inventory_change=none
load_attempted=false
```

A valid command records one local-only RAM event binding:

```json
{
  "schema": "raios.module_manifest_reference.v0",
  "status": "retained_hash_reference_load_still_denied",
  "classification": "local_only",
  "scope": "current_boot",
  "manifest_schema": "raios.module_manifest.v0",
  "accepts_manifest_json": false,
  "accepts_artifact_bytes": false,
  "accepts_unsigned_service_code": false,
  "authorizes_guest_load": false,
  "can_load_now": false,
  "service_inventory_change": "none",
  "load_attempted": false,
  "hashes": {
    "manifest_reference_hash": "sha256:<64 hex chars>",
    "manifest_hash": "sha256:<64 hex chars>"
  }
}
```

The retained reference is evidence only. Later `module.load_ephemeral` and
`service.load_ephemeral` calls revalidate the retained current-boot event before
reporting `module_manifest: retained_hash_reference_only`; loading remains
denied.
