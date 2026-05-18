# raiOS Module Manifest V0

`raios.module_manifest.v0` is the first durable description of an
agent-proposed extension artifact. It is not a trust grant. It describes what an
artifact claims to be, which capabilities it requests, and which hashes must be
bound into VM reports and local attestations before any load decision can be
made.

Stage-0 still returns `capability_denied` for `module.load_ephemeral` and
`module.persist`. A valid manifest only makes an artifact eligible for the
evidence flow:

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
