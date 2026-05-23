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
  "profile": "quick",
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
    "args_sha256": "...",
    "serial_write": {
      "chunk_bytes": 256,
      "inter_chunk_delay_ms": 0
    }
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
    "..."
  ],
  "executed_commands": [
    {
      "command": "describe",
      "name": "command:describe",
      "expected_marker": "RAIOS_AGENT_END system.describe",
      "response_offset": 5249,
      "duration_ms": 363,
      "sent": true,
      "passed": true
    }
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

`profile` names the harness slice (`quick`, `recovery`, or `full`). `commands`
is derived from the actual serial commands that were sent during the run, not
from a static profile inventory. `executed_commands` is the authoritative
per-command evidence list; each entry is recorded only after the serial write
returns, and includes the expected marker, response offset, duration, `sent`,
and pass/fail result. A command that was sent but timed out remains present with
`passed: false`; a connection failure before any write is not promoted into
`commands`.

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
`raios.module_load_gate.v0`. It also checks the read-only
`raios.module_manifest_reference_diagnostic.v0` absent-reference state, a valid
manifest hash-reference command that records
`raios.module_manifest_reference.v0` in the current-boot event log, and
`raios.module_manifest_reference_diagnostic_selftest.v0` cases for accepted,
absent, stale, mismatched, and invalid manifest references. The current
predicate set also checks the read-only
`raios.module_candidate_artifact_reference_diagnostic.v0` absent-reference
state, a valid artifact hash-reference command that records
`raios.module_candidate_artifact_reference.v0` in the current-boot event log,
and `raios.module_candidate_artifact_reference_diagnostic_selftest.v0` cases for
accepted, absent, stale, mismatched, malformed, and grant-mismatched artifact
references. The current predicate set also checks the read-only
`raios.module_computed_grant_diagnostic.v0` absent-reference state, a valid
full hash-reference command that records
`raios.module_computed_grant_reference.v0` in the current-boot event log, and
`raios.module_computed_grant_diagnostic_selftest.v0` cases for accepted,
absent, stale, mismatched, and wrong-policy computed grant references. It also
checks the read-only
`raios.module_vm_test_report_reference_diagnostic.v0` absent-reference state, a
valid VM-report hash-reference command that records
`raios.module_vm_test_report_reference.v0` in the current-boot event log, and
`raios.module_vm_test_report_reference_diagnostic_selftest.v0` cases for
accepted, absent, stale, mismatched, computed-grant-mismatched, and
non-current-boot event-id references. It also
checks `raios.module_load_gate_manifest_selftest.v0` cases for
missing, accepted-current-boot-but-denied, stale/dropped,
previous-boot-or-unretained, wrong-schema, substituted-record, and mismatched
manifest-reference candidates. The current predicate set also checks
`raios.module_load_gate_artifact_selftest.v0` cases for missing,
accepted-current-boot-but-denied, stale/dropped, previous-boot-or-unretained,
wrong-schema, substituted-record, artifact-reference hash mismatch,
manifest-reference mismatch, and computed-grant-reference mismatch candidates.
The current predicate set also checks
`raios.module_load_gate_vm_report_selftest.v0` cases for missing,
accepted-current-boot-but-denied, stale/dropped, previous-boot-or-unretained,
wrong-schema, substituted-record, VM-report-reference hash mismatch,
manifest-reference mismatch, artifact-reference mismatch,
computed-grant-reference mismatch, and VM-report-hash mismatch candidates. The
current predicate set also checks
`raios.module_load_gate_retained_reference_selftest.v0` cases for missing,
accepted-current-boot-but-denied, stale/dropped, previous-boot-or-unretained,
wrong-schema, substituted-record, and mismatched-hash retained-reference
candidates. It also checks
`raios.module_load_gate_audit_rollback_selftest.v0` cases for missing durable
audit, missing rollback plan, matching audit/rollback evidence still denied by
missing loader and service slot, audit/rollback schema mismatches, missing,
stale, previous-boot, wrong-schema, and substituted retained audit/rollback
references, retained computed-grant, audit, and rollback hash mismatches,
retained service-slot mismatch, retained grant hash mismatch,
manifest/artifact/VM-report/local-attestation mismatches, local approval
mismatch, rollback-plan hash mismatch, rollback artifact mismatch, and rollback
service-slot mismatch. It also checks
`raios.module_load_gate_service_slot_selftest.v0` cases for missing,
accepted-current-boot, stale/dropped, wrong-schema, substituted,
computed-grant/audit/rollback hash mismatches, inventory mismatch, slot
mismatch, and reservation-hash mismatch retained service-slot reservations. It
also checks
`raios.module_service_slot_reservation_diagnostic.v0` absent and valid
hash-reference commands, RAM-only retention as
`raios.module_service_slot_reservation.v0`, and
`raios.module_service_slot_reservation_diagnostic_selftest.v0` absent,
accepted, stale, mismatched-hash, and invalid-slot cases. The
module-load assertions verify the current-boot audit event binding, full
missing-evidence list, audit/rollback requirement schema, retained
manifest hash evidence when a valid manifest reference was retained, retained
artifact hash evidence when a valid candidate-artifact reference was retained,
retained grant/report/attestation hashes when a valid grant reference was
retained, retained VM-report reference state and hashes when a valid VM-report
reference was retained, live rejection of a wrong-schema retained audit/rollback reference,
retained audit/rollback reference state and hashes when a valid audit/rollback
reference was retained, live retained service-slot reservation state and
reservation hash when a valid reservation was retained, unchanged service
inventory, and `load_attempted: false`, plus separate recovery artifact
identity/trust/VM-test/local-approval retained hash-reference diagnostics that
remain non-authorizing. The latest verified report is
`release/vm-reports/shadow-20260523-161602-8028.json` with 4500/4500
predicates.

## Guest Hash-Reference Diagnostic

The report itself is not accepted by the guest as trusted JSON in this slice.
Instead, the guest can retain a current-boot hash reference to report evidence:

```text
agent module.vm_report_diagnostic
agent module.vm_report_diagnostic <report_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
agent module.vm_report_diagnostic_selftest
agent module.load_gate_vm_report_selftest
```

The retained schema is `raios.module_vm_test_report_reference.v0`; the
diagnostic schema is
`raios.module_vm_test_report_reference_diagnostic.v0`; the canonicalization is
`raios.module_vm_test_report_reference.canonical.v0`. The method records only a
local-only RAM event binding for valid current-boot hash references and keeps
`accepts_vm_report_json: false`, `authorizes_guest_load: false`,
`can_load_now: false`, and `load_attempted: false`.
