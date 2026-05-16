# SeedOS VM Test Report V0

`seedos.vm_test_report.v0` is the first machine-readable evidence artifact for
Shadow-VM checks. It is produced by:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1
```

The report is not a live-loading approval by itself. It is only one required
input for future `module.load_ephemeral` or `module.persist` decisions.
Stage-0 still returns `capability_denied` for those methods.

## Current Sandbox Policy

The first harness profile is deliberately narrow:

```text
QEMU headless
temporary image by default
no shared folders
no host filesystem mounts
network disabled unless -Network is passed
serial TCP command channel only
hard timeout per boot/predicate
QEMU process killed after the run
```

Candidate `-ManifestPath` and `-ArtifactPath` values are validated and hashed
into the report, but they are not injected into the guest yet. An artifact path
without a manifest is rejected. That avoids pretending that untrusted code is
safely runnable before the guest module ABI and loader policy exist.

## Shape

The JSON report contains:

```json
{
  "schema": "seedos.vm_test_report.v0",
  "result": "passed",
  "run_id": "shadow-...",
  "sandbox_policy": {
    "network": "disabled",
    "shared_folders": "none",
    "host_filesystem_mounts": "none"
  },
  "base_image": {
    "path": "...",
    "sha256": "...",
    "temporary": true
  },
  "candidate_artifact": {
    "path": null,
    "sha256": null
  },
  "candidate_manifest": {
    "path": null,
    "sha256": null,
    "validation": null
  },
  "qemu": {
    "args_sha256": "..."
  },
  "evidence_binding": {
    "base_image_sha256": "...",
    "candidate_artifact_sha256": null,
    "candidate_manifest_sha256": null,
    "qemu_args_sha256": "...",
    "serial_log_sha256": "..."
  },
  "commands": [
    "describe",
    "snapshot",
    "services",
    "problems",
    "module.load_ephemeral"
  ],
  "predicates": [
    {
      "name": "policy:mutating_load_denied",
      "expected": "serial_contains:\"code\": \"capability_denied\"",
      "passed": true
    }
  ],
  "serial_log": {
    "path": "...",
    "sha256": "..."
  },
  "failures": []
}
```

A sidecar `.sha256` file is written for the report itself so the report can be
referenced without embedding a self-referential hash in the JSON.
