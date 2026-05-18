# raiOS VM Test Report V0

`raios.vm_test_report.v0` is the first machine-readable evidence artifact for
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
  "schema": "raios.vm_test_report.v0",
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
  "hardware_profile": {
    "profile": "raios.shadow_vm.q35_xhci.v0",
    "machine": "q35",
    "memory": "512M",
    "cpu": "max",
    "firmware": "edk2-x86_64",
    "boot_drive": "ide_raw_image",
    "display": "none",
    "serial": "tcp_chardev_with_log",
    "input": [
      "qemu-xhci",
      "usb-kbd",
      "usb-tablet"
    ],
    "network": "none"
  },
  "qemu": {
    "args_canonical_json": "[...]",
    "args_sha256": "..."
  },
  "evidence_binding": {
    "base_image_sha256": "...",
    "candidate_artifact_sha256": null,
    "candidate_manifest_sha256": null,
    "hardware_profile_sha256": "...",
    "qemu_args_sha256": "...",
    "serial_log_sha256": "...",
    "predicate_count": 16,
    "predicate_passed_count": 16,
    "predicate_failed_count": 0
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
      "passed": true,
      "actual": "found"
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

`qemu.args_sha256` is computed from `qemu.args_canonical_json`, a compressed JSON
array of the exact runner arguments, instead of a whitespace-joined command
line. That keeps argument binding unambiguous even when a path contains spaces.

Predicate `actual` is `"found"` on success. On failure it contains the tail of
the serial log, capped for report readability, so a denied activation can still
explain which marker was missing without opening the full log.
