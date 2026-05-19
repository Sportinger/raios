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
    "predicate_count": 297,
    "predicate_passed_count": 297,
    "predicate_failed_count": 0
  },
  "commands": [
    "describe",
    "snapshot",
    "caps",
    "services",
    "problems",
    "agent memory.profile",
    "agent memory.context diagnostic",
    "agent memory.context provider_minimal",
    "agent provider.context_export provider_minimal",
    "agent provider.context_gate provider_minimal",
    "agent provider.context_gate_selftest provider_minimal",
    "agent provider.context_injection_gate provider_minimal",
    "agent provider.context_injection_gate_selftest provider_minimal",
    "agent memory.query",
    "agent memory.trace snapshot.current",
    "agent memory.recent_events",
    "agent audit.events 8",
    "agent memory.record_observation",
    "agent memory.propose_policy",
    "agent memory.supersede_fact",
    "agent memory.redact",
    "agent memory.compact",
    "module.load_ephemeral",
    "agent audit.events 8"
  ],
  "predicates": [
    {
      "name": "protocol:memory_context_schema",
      "expected": "serial_contains:\"schema\": \"raios.agent_context.v0\"",
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

The current predicate set includes the read-only agent protocol, provider trust
problem visibility, `memory.context` event ids, the local
`provider_minimal` redaction projection with packet/field-list hashes, the
denied `provider.context_export` gate with provider writes still
`not_attempted`, positive request/export binding gates still missing, the
read-only `provider.context_gate` missing-binding state, the local-only
`provider.context_gate_selftest` negative predicate cases, the separate
`provider.context_injection_gate` missing-final-authorization state, the
  `provider.context_injection_gate_selftest` negative final-authorization cases,
denial audit records that do not satisfy those gates, structured event-log
denial bindings with packet and field-list hashes, negative checks for positive
provider binding schemas, positive export authorization, and fake provider
request envelopes from `provider.context_export`, query/trace locators, RAM-only
event/audit reads, denied memory mutations, and denied module loading through
`raios.module_load_gate.v0`. The module-load assertions also verify the
current-boot audit event binding, full missing-evidence list, null
manifest/artifact/report/attestation hashes, unchanged service inventory, and
`load_attempted: false`.
