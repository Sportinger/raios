# SeedOS Local Attestation V0

`seedos.local_attestation.v0` records that a local user approved the exact
manifest, artifact, and Shadow-VM report tuple for a future load gate. It still
does not load code in Stage-0.

Create one after a passing VM report:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\create-local-attestation.ps1 `
  -ManifestPath .\candidate.manifest.json `
  -ArtifactPath .\candidate.bin `
  -VmReportPath .\release\vm-reports\shadow-....json `
  -Approval "APPROVE RAM_ONLY <artifact-hash-prefix>"
```

The approval phrase must match the artifact hash prefix printed by the tool's
error message, for example:

```text
APPROVE RAM_ONLY 012345abcdef
```

The attestation binds:

```text
manifest sha256
artifact sha256
VM report sha256
base image sha256
QEMU args sha256
load mode
approval phrase hash
rollback plan
```

Output is written to `release\attestations` with a `.sha256` sidecar. Generated
attestations are local evidence artifacts and are ignored by Git.

The current attestation result is:

```text
evidence_recorded_load_still_denied_in_stage0
```

That wording is intentional: until the guest loader and kernel policy exist, an
attestation is evidence, not execution permission.
