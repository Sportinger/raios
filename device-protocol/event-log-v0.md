# raiOS Event Log V0

`event.log.v0` is the first RAM-only event/audit surface for ADR 0004 memory.
It records bounded `current_boot` protocol evidence without creating persistent
memory or a durable audit ledger.

The implementation source of truth is `seed-kernel/src/event_log.rs`, with
recording hooks in `seed-kernel/src/agent_protocol.rs`.

This document also describes the current denied `provider_minimal` export
preflight records and the positive local-only binding records emitted by the
real pinned OpenAI `ask` path. Denial events carry structured hash-valued
bindings but never satisfy positive gates. Negative gate selftests are local
test infrastructure and do not write synthetic positive bindings into the
global event log.

## Scope

V0 records:

- read-only `raios.agent.v0` responses
- `capability_denied` outcomes for known mutating methods
- `raios.module_load_gate.v0` bindings for denied module/service load attempts
- local-only `raios.module_computed_grant_reference.v0` bindings for valid
  current-boot module computed-grant hash references
- local-only `raios.module_audit_rollback_reference.v0` bindings for valid
  current-boot module audit/rollback hash references
- `capability_denied` outcomes for provider context export attempts
- local provider request envelope creation on the real direct OpenAI request path
- positive provider request binding records after pinned/WebPKI provider trust
- positive export-audit binding records after pinned/WebPKI provider trust
- checked binding consumption records for local gate evaluation without export
- read-only negative gate selftest responses that exercise the same predicate
  without mutating the global event log
- read-only final injection gate diagnostics that keep body attachment blocked
- read-only final injection negative selftest responses that exercise the final
  authorization predicate without mutating the global event log
- read-only module load-gate retained-reference selftest responses that
  exercise negative retained-reference candidates without mutating the global
  event log
- read-only module load-gate audit/rollback selftest responses that exercise
  missing and mismatched audit/rollback candidates without mutating the global
  event log
- provider request-binding denial records for `provider_minimal`
- provider context export-denial audit records with
  `outcome: denied_no_provider_write`

V0 does not record raw request payloads, provider prompts, API keys, Wi-Fi
secrets, boot-log text, or arbitrary response bodies. Provider request envelope
events may record local-only request body hashes, but never the body or prompt
itself.

## Methods

The serial protocol exposes one canonical method and one alias:

```text
memory.recent_events [limit]
audit.events [limit]
```

Both return the same response envelope with `body.method` set to
`memory.recent_events`. `limit` is optional, defaults to the kernel default, and
is capped by the RAM ring capacity.

## event.log.v0

The response result shape is:

```json
{
  "schema": "event.log.v0",
  "record_schema": "audit.event.v0",
  "scope": "current_boot",
  "retention": "ram_ring",
  "persistence": "none",
  "provider_export": "disabled",
  "bounded": true,
  "limit": 32,
  "capacity": 64,
  "event_count": 12,
  "returned": 12,
  "dropped_before_sequence": 0,
  "events": []
}
```

`event_count` is the total number of events recorded in the current boot.
`dropped_before_sequence` is `0` until the ring overwrites an older event; after
that, events with a smaller sequence are no longer retained.

## audit.event.v0

Each record is compact and non-secret:

```json
{
  "schema": "audit.event.v0",
  "id": "event.current_boot.00000012",
  "scope": "current_boot",
  "sequence": 12,
  "kind": "agent_protocol.read_response",
  "source_method": "memory.context",
  "source_transport": "serial-console",
  "classification": "public",
  "outcome": "response",
  "requested_capability": "cap.memory.context.read",
  "risk": "observe",
  "subject": "agent.session.serial",
  "resource": "current_boot",
  "reason": "granted_read",
  "evidence": ["computed_capability_grant"],
  "created_at": {
    "clock": "sequence_only",
    "millis": null
  },
  "bindings": {},
  "persistence": "none"
}
```

`bindings` is optional and must be omitted for ordinary read events. When
present, it contains compact public binding schemas, statuses, gate markers, and
hashes only. Denial bindings are non-authorizing even when they carry exact
packet hashes.

Denied records use:

```text
kind: agent_protocol.capability_denied
outcome: capability_denied
reason: missing_evidence
evidence: missing_required_evidence, capability_denied
```

`module.load_ephemeral` and `service.load_ephemeral` denials add
`module_load_gate_evaluated`, `computed_capability_grant_reference_checked`,
`durable_audit_record_required`, `rollback_plan_required`,
`rollback_bindings_required`, `service_inventory_unchanged`, and
`load_not_attempted` evidence and attach a `raios.module_load_gate.v0` binding.
If valid audit/rollback hash-reference evidence was retained earlier in the
same boot, the denial snapshots that reference as non-authorizing current-boot
evidence only after the live gate validates the retained reference against the
retained computed-grant event, a prior denied load event, canonical hashes, and
the `ram_only:` service-slot id.

Most denial responses include `event_id` and `audit_event_id`, both pointing at
the current-boot denial record id. `provider.context_export` is stricter: its
`event_id` points at the capability-denial event, while `audit_event_id` points
at a separate export-denial-audit event. These IDs are evidence handles, not
durable memory.

`memory.context` responses also include `context_event_id` and `audit_event_id`
for the local read/projection event. The local read event alone does not
authorize provider export.

`provider.context_export provider_minimal` denials record
`requested_capability: cap.provider.context_export` and `risk: export`. These
records prove that no provider write was attempted; they are not positive export
bindings.

## Module Load Gate Event

Denied module loading is recorded as a normal current-boot audit event with a
structured non-authorizing binding:

```json
{
  "schema": "audit.event.v0",
  "kind": "agent_protocol.capability_denied",
  "source_method": "module.load_ephemeral",
  "classification": "public",
  "outcome": "capability_denied",
  "requested_capability": "cap.module.load_ephemeral",
  "risk": "modify_ram",
  "subject": "agent.session.serial",
  "resource": "live_service_graph",
  "reason": "missing_evidence",
  "evidence": [
    "missing_required_evidence",
    "capability_denied",
    "module_load_gate_evaluated",
    "computed_capability_grant_reference_checked",
    "durable_audit_record_required",
    "rollback_plan_required",
    "rollback_bindings_required",
    "service_inventory_unchanged",
    "load_not_attempted"
  ],
  "bindings": {
    "schema": "raios.module_load_gate.v0",
    "status": "denied_missing_evidence",
    "load_mode": "ram_only",
    "requested_capability": "cap.module.load_ephemeral",
    "risk": "modify_ram",
    "target": "live_service_graph",
    "gate_state": {
      "module_manifest": "missing",
      "candidate_artifact": "missing",
      "vm_test_report": "missing",
      "local_attestation": "missing",
      "computed_capability_grant": "missing | retained_hash_reference_only",
      "local_approval": "missing",
      "rollback_plan": "missing | retained_hash_reference_only_not_installed | rejected_retained_reference",
      "durable_audit_record": "missing | retained_hash_reference_only_not_durable | rejected_retained_reference",
      "loader": "unavailable",
      "service_slot": "unallocated",
      "artifact_loaded": false,
      "service_started": false,
      "persistence": "none",
      "can_load": false
    },
    "retained_computed_grant_reference": {
      "state": "missing | present",
      "schema": "raios.module_computed_grant_reference.v0",
      "status": "missing | retained_hash_reference_load_still_denied"
    },
    "retained_audit_rollback_reference": {
      "state": "missing | present | rejected",
      "schema": "raios.module_audit_rollback_reference.v0",
      "status": "missing | retained_hash_reference_load_still_denied | rejected"
    },
    "audit_rollback_requirements": {
      "schema": "raios.module_load_gate_audit_rollback_requirements.v0",
      "status": "required_missing",
      "writes_enabled": false,
      "durable_audit_record": {
        "schema": "raios.audit_record.v0",
        "state": "missing | retained_hash_reference_only_not_durable | rejected_retained_reference"
      },
      "rollback_plan": {
        "schema": "raios.rollback_plan.v0",
        "state": "missing | retained_hash_reference_only_not_installed | rejected_retained_reference"
      }
    },
    "evidence": {
      "computed_capability_grant_hash": "null | sha256:<retained grant hash>",
      "manifest_hash": "null | sha256:<retained manifest hash>",
      "artifact_hash": "null | sha256:<retained artifact hash>",
      "vm_test_report_hash": "null | sha256:<retained report hash>",
      "local_attestation_hash": "null | sha256:<retained attestation hash>",
      "audit_record_hash": "null | sha256:<retained audit record hash>",
      "rollback_plan_hash": "null | sha256:<retained rollback plan hash>",
      "local_approval_hash": "null | sha256:<retained approval hash>",
      "pre_load_service_inventory_hash": "null | sha256:<retained inventory hash>",
      "cleanup_actions_hash": "null | sha256:<retained cleanup hash>",
      "ram_only_service_slot_id": "null | ram_only:<service slot id>",
      "service_inventory_change": "none",
      "load_attempted": false
    }
  },
  "persistence": "none"
}
```

This event is evidence that the gate refused to load. If a valid
`raios.module_computed_grant_reference.v0` was retained earlier in the same
boot, the denied load event snapshots its event id and hashes, reports
`computed_capability_grant: retained_hash_reference_only`, and still blocks with
`retained_computed_grant_reference_not_authorizing`. It is not a durable audit
record and cannot satisfy a future load grant by itself.

The same binding also exposes
`raios.module_load_gate_audit_rollback_requirements.v0`. That requirement
shape names the future `raios.audit_record.v0` and `raios.rollback_plan.v0`
bindings, reports retained hash-reference-only states when available, disables
writes, and does not create durable state.

When the latest retained `raios.module_audit_rollback_reference.v0` fails the
live predicate, the load-gate binding retains only its event id, schema,
`status: rejected`, reason, and non-authorizing flags. The accepted
`audit_record_hash`, `rollback_plan_hash`, approval, inventory, cleanup, and
slot evidence fields remain `null` so a rejected retained record cannot be
mistaken for valid gate evidence.

## Module Computed Grant Reference Event

When `module.grant_diagnostic` receives a valid current-boot hash reference,
Stage-0 records one local-only RAM event. This event retains hashes only; it
does not retain artifact bytes, manifest JSON, VM-report JSON, or attestation
JSON.

```json
{
  "schema": "audit.event.v0",
  "kind": "module.computed_grant_reference.retained",
  "source_method": "module.grant_diagnostic",
  "classification": "local_only",
  "outcome": "retained_hash_reference_load_still_denied",
  "requested_capability": "cap.module.grant_diagnostic.read",
  "risk": "observe",
  "resource": "live_service_graph",
  "reason": "computed_grant_reference_valid_for_current_boot",
  "bindings": {
    "schema": "raios.module_computed_grant_reference.v0",
    "status": "retained_hash_reference_load_still_denied",
    "scope": "current_boot",
    "classification": "local_only",
    "requested_capability": "cap.module.load_ephemeral",
    "load_mode": "ram_only",
    "grants_capability": false,
    "grants_load_now": false,
    "authorizes_guest_load": false,
    "can_load_now": false,
    "service_inventory_change": "none",
    "load_attempted": false,
    "hashes": {
      "computed_capability_grant_hash": "sha256:<64 hex chars>",
      "manifest_hash": "sha256:<64 hex chars>",
      "artifact_hash": "sha256:<64 hex chars>",
      "vm_test_report_hash": "sha256:<64 hex chars>",
      "local_attestation_hash": "sha256:<64 hex chars>"
    }
  },
  "persistence": "none"
}
```

The latest retained reference is visible through
`module.grant_diagnostic` as `retained_reference`. It remains diagnostic
evidence only and cannot satisfy the future durable audit or loader gates.

## Module Audit/Rollback Reference Event

When `module.audit_rollback_diagnostic` receives a valid current-boot
audit/rollback hash reference, Stage-0 records one local-only RAM event. This
event retains hashes and current-boot ids only; it does not retain durable audit
JSON, rollback-plan JSON, artifact bytes, manifests, VM reports, attestations,
or local approval text.

```json
{
  "schema": "audit.event.v0",
  "kind": "module.audit_rollback_reference.retained",
  "source_method": "module.audit_rollback_diagnostic",
  "classification": "local_only",
  "outcome": "retained_hash_reference_load_still_denied",
  "requested_capability": "cap.module.grant_diagnostic.read",
  "risk": "observe",
  "resource": "live_service_graph",
  "reason": "audit_rollback_reference_valid_for_current_boot",
  "bindings": {
    "schema": "raios.module_audit_rollback_reference.v0",
    "status": "retained_hash_reference_load_still_denied",
    "scope": "current_boot",
    "classification": "local_only",
    "requested_capability": "cap.module.load_ephemeral",
    "load_mode": "ram_only",
    "durable_audit_written": false,
    "rollback_plan_installed": false,
    "grants_capability": false,
    "grants_load_now": false,
    "authorizes_guest_load": false,
    "can_load_now": false,
    "service_inventory_change": "none",
    "load_attempted": false,
    "denial_event_id": "event.current_boot.<denied-load-id>",
    "retained_computed_grant_reference_event_id": "event.current_boot.<retained-grant-id>",
    "ram_only_service_slot_id": "ram_only:svc.example.0001",
    "hashes": {
      "audit_record_hash": "sha256:<64 hex chars>",
      "rollback_plan_hash": "sha256:<64 hex chars>",
      "computed_capability_grant_hash": "sha256:<64 hex chars>",
      "manifest_hash": "sha256:<64 hex chars>",
      "artifact_hash": "sha256:<64 hex chars>",
      "vm_test_report_hash": "sha256:<64 hex chars>",
      "local_attestation_hash": "sha256:<64 hex chars>",
      "local_approval_hash": "sha256:<64 hex chars>",
      "pre_load_service_inventory_hash": "sha256:<64 hex chars>",
      "cleanup_actions_hash": "sha256:<64 hex chars>"
    }
  },
  "persistence": "none"
}
```

The latest retained audit/rollback reference is visible through
`module.audit_rollback_diagnostic` as `retained_audit_rollback_reference` and
through later denied `module.load_ephemeral` bindings. It remains diagnostic
evidence only and cannot satisfy durable audit, rollback installation, service
slot allocation, or loader gates.

## Provider Request Envelope Event

The real direct OpenAI `ask` path records one local-only pre-write envelope event
after request id allocation and before DNS/TCP/TLS/API-key copy/HTTPS write:

```json
{
  "schema": "audit.event.v0",
  "id": "event.current_boot.00000010",
  "scope": "current_boot",
  "sequence": 10,
  "kind": "provider_request.envelope_created",
  "source_method": "ask",
  "source_transport": "serial-console",
  "classification": "local_only",
  "outcome": "local_prewrite_envelope",
  "requested_capability": "cap.provider.request",
  "risk": "export",
  "subject": "agent.session.serial",
  "resource": "svc.provider.openai_direct",
  "reason": "provider_request_envelope_created_before_write",
  "evidence": [
    "provider_request_envelope_created",
    "request_body_hash",
    "envelope_hash",
    "provider_write_not_attempted"
  ],
  "created_at": {"clock": "sequence_only", "millis": null},
  "bindings": {
    "schema": "raios.provider_request_envelope.v0",
    "status": "local_prewrite_envelope",
    "satisfies_current_boot_export_gate": false,
    "provider_write": "not_attempted",
    "context_attached_to_provider_body": false,
    "request_id": 1,
    "request_body_hash": "sha256:<64 hex chars>",
    "envelope_hash": "sha256:<64 hex chars>",
    "trust_snapshot": {
      "provider_trust_state": "unknown",
      "provider_trust_positive": false,
      "development_tls_bypass": false
    }
  },
  "persistence": "none"
}
```

This event is not a `raios.provider_request_binding.v0`. It does not authorize
context export, and `provider.context_export` must not create it.

## Positive Provider Binding Events

After a real direct OpenAI request envelope exists and provider trust becomes
positive through pinned SPKI, pinned leaf certificate, or future WebPKI
verification, Stage-0 records current-boot positive binding evidence before
API-key copy or HTTPS write.

Development TLS bypass, pin mismatch, missing pins, stale ids, and denial
records must not create these positive schemas.

### Request Binding Event

```json
{
  "schema": "audit.event.v0",
  "id": "event.current_boot.00000012",
  "scope": "current_boot",
  "kind": "provider_context_export.request_binding_bound",
  "source_method": "ask",
  "classification": "local_only",
  "outcome": "bound",
  "requested_capability": "cap.provider.context_export",
  "risk": "export",
  "reason": "provider_minimal_context_bound_to_real_request_envelope",
  "bindings": {
    "schema": "raios.provider_request_binding.v0",
    "status": "bound",
    "satisfies_request_binding_gate": true,
    "satisfies_current_boot_export_gate": false,
    "provider_write_at_binding": "not_attempted",
    "context_attached_to_provider_body": false,
    "request_envelope_event_id": "event.current_boot.00000011",
    "request_body_hash": "sha256:<64 hex chars>",
    "request_envelope_hash": "sha256:<64 hex chars>",
    "request_binding_hash": "sha256:<64 hex chars>",
    "hashes": {
      "projected_packet_hash": "sha256:<64 hex chars>",
      "exported_field_list_hash": "sha256:<64 hex chars>",
      "omitted_field_list_hash": "sha256:<64 hex chars>"
    }
  },
  "persistence": "none"
}
```

The request binding satisfies only the request-binding gate. It does not by
itself satisfy the current-boot export gate.

### Export Audit Binding Event

```json
{
  "schema": "audit.event.v0",
  "id": "event.current_boot.00000013",
  "scope": "current_boot",
  "kind": "provider_context_export.audit_binding_bound",
  "source_method": "ask",
  "classification": "local_only",
  "outcome": "authorized_for_single_provider_request",
  "requested_capability": "cap.provider.context_export",
  "risk": "export",
  "reason": "provider_minimal_context_export_audit_bound_without_body_attachment",
  "bindings": {
    "schema": "raios.provider_context_export_audit_binding.v0",
    "status": "authorized_for_single_provider_request",
    "satisfies_export_audit_binding_gate": true,
    "satisfies_current_boot_export_gate": false,
    "positive_export_authorization": true,
    "automatic_context_injection": "disabled",
    "provider_write_at_binding": "not_attempted",
    "context_attached_to_provider_body": false,
    "request_binding_event_id": "event.current_boot.00000012",
    "export_audit_binding_hash": "sha256:<64 hex chars>"
  },
  "persistence": "none"
}
```

The export audit binding is positive evidence for the audit-binding gate, but
Stage-0 still reports the overall current-boot export gate as unsatisfied while
automatic context injection is disabled. The OpenAI request body remains free of
provider-minimal context in this slice.

### Binding Consumption Event

When `provider.context_export provider_minimal` evaluates a retained positive
binding pair, Stage-0 may record a local-only consumption event:

```json
{
  "schema": "audit.event.v0",
  "kind": "provider_context_export.binding_consumption_checked",
  "source_method": "provider.context_export",
  "classification": "local_only",
  "outcome": "checked_not_exported",
  "requested_capability": "cap.provider.context_export",
  "risk": "export",
  "reason": "provider_binding_consumed_without_body_attachment",
  "bindings": {
    "schema": "raios.provider_context_binding_consumption.v0",
    "status": "consumed_for_gate_evaluation",
    "satisfies_current_boot_export_gate": false,
    "automatic_context_injection": "disabled",
    "provider_write": "not_attempted",
    "context_attached_to_provider_body": false,
    "request_binding_event_id": "event.current_boot.00000012",
    "export_audit_binding_event_id": "event.current_boot.00000013",
    "request_binding_hash": "sha256:<64 hex chars>",
    "export_audit_binding_hash": "sha256:<64 hex chars>"
  },
  "persistence": "none"
}
```

This event means the pair was consumed for local gate evaluation only. It is not
an export record and cannot by itself make
`satisfies_current_boot_export_gate` true.

## Negative Gate Selftest Response

`provider.context_gate_selftest provider_minimal` emits
`raios.provider_context_gate_negative_selftest.v0`. It is not an
`audit.event.v0` record and it does not persist synthetic events into the global
RAM ring. Its required flags are:

```text
test_infrastructure: true
mutates_global_event_log: false
creates_provider_request_envelope: false
creates_positive_binding_records: false
provider_write: not_attempted
automatic_context_injection: disabled
context_attached_to_provider_body: false
```

The Shadow VM harness expects the selftest to pass cases for stale/dropped
event ids, previous-boot-or-unretained event ids, denial-schema substitution,
positive-record substitution, wrong variants, mismatched request/body/binding
hashes, mismatched provider-minimal context hashes, and trust-bypass records.
These cases are evidence that the predicate fails closed; they are not provider
export authority.

## Final Injection Gate Diagnostic

`provider.context_injection_gate provider_minimal` emits
`raios.provider_context_injection_gate.v0`. It is a read-only diagnostic, not an
`audit.event.v0` authority. It reports:

```text
final_authorization_schema: raios.provider_context_injection_authorization.v0
final_authorization: missing
final_prewrite_body_check: not_attempted
automatic_context_injection: disabled
context_attached_to_provider_body: false
provider_write: not_attempted
can_attach_context: false
```

Positive pinned/WebPKI OpenAI request paths emit a local
`OPENAI_PROVIDER_CONTEXT_INJECTION_GATE` marker before API-key copy or HTTPS
write. The marker binds request and provider-minimal context hashes but still
has `status: blocked`.

`provider.context_injection_gate_selftest provider_minimal` emits
`raios.provider_context_injection_gate_negative_selftest.v0`. Like the export
gate selftest, it is a response-only test artifact, not an `audit.event.v0`
record. It must keep:

```text
test_infrastructure: true
mutates_global_event_log: false
creates_provider_request_envelope: false
creates_positive_binding_records: false
creates_final_authorization_records: false
provider_write: not_attempted
automatic_context_injection: disabled
context_attached_to_provider_body: false
can_attach_context: false
```

The current cases cover `final_injection_authorization_missing`,
`final_injection_authorization_stale_or_dropped_event_id`,
`final_injection_authorization_wrong_schema_or_variant`,
`final_injection_authorization_substituted_record`,
`final_prewrite_body_hash_mismatch`,
`final_provider_trust_downgraded_before_write`, and
`body_attachment_without_final_authorization`.

## Provider Export Denial Events

Stage-0 records denial evidence before any provider write. These records are
current-boot evidence only. They do not persist memory, do not prove that
context left the machine, and do not satisfy positive export gates.

### Request Binding Denial Event

The request-binding-denial event records that a provider request binding was not
created because no real provider request envelope exists:

```json
{
  "schema": "audit.event.v0",
  "id": "event.current_boot.00000010",
  "scope": "current_boot",
  "sequence": 10,
  "kind": "provider_context_export.request_binding_denied",
  "source_method": "provider.context_export",
  "source_transport": "serial-console",
  "classification": "public",
  "outcome": "denied_not_bound",
  "requested_capability": "cap.provider.context_export",
  "risk": "export",
  "subject": "agent.session.serial",
  "resource": "current_boot",
  "reason": "provider_request_binding_requires_real_request_envelope",
  "evidence": [
    "provider_request_binding_denied",
    "projected_packet_hash",
    "provider_write_not_attempted"
  ],
  "created_at": {"clock": "sequence_only", "millis": null},
  "bindings": {
    "schema": "raios.provider_request_binding_denial.v0",
    "status": "denied_not_bound",
    "satisfies_current_boot_export_gate": false,
    "provider_write": "not_attempted",
    "hashes": {
      "packet_canonicalization": "raios.provider_minimal.packet.canonical.v0",
      "projected_packet_hash": "sha256:<64 hex chars>",
      "exported_field_list_hash": "sha256:<64 hex chars>",
      "omitted_field_list_hash": "sha256:<64 hex chars>"
    }
  },
  "persistence": "none"
}
```

The serial response may include a richer
`raios.provider_request_binding_denial.v0` object with the same packet and
field-list hashes. The RAM event binding is still a denial record and must not
be accepted where `raios.provider_request_binding.v0` is required.

### Export Denial Audit Event

The export-denial-audit event records that the export path was evaluated and no
provider write was attempted:

```json
{
  "schema": "audit.event.v0",
  "id": "event.current_boot.00000011",
  "scope": "current_boot",
  "sequence": 11,
  "kind": "provider_context_export.denial_audit",
  "source_method": "provider.context_export",
  "source_transport": "serial-console",
  "classification": "public",
  "outcome": "denied_no_provider_write",
  "requested_capability": "cap.provider.context_export",
  "risk": "export",
  "subject": "agent.session.serial",
  "resource": "current_boot",
  "reason": "provider_context_export_not_authorized",
  "evidence": [
    "provider_request_binding_denied",
    "projected_packet_hash",
    "exported_field_list_hash",
    "omitted_field_list_hash",
    "provider_write_not_attempted"
  ],
  "created_at": {"clock": "sequence_only", "millis": null},
  "bindings": {
    "schema": "raios.provider_context_export_denial_audit.v0",
    "status": "denied_no_provider_write",
    "satisfies_current_boot_export_gate": false,
    "positive_export_authorization": false,
    "provider_write": "not_attempted",
    "hashes": {
      "packet_canonicalization": "raios.provider_minimal.packet.canonical.v0",
      "projected_packet_hash": "sha256:<64 hex chars>",
      "exported_field_list_hash": "sha256:<64 hex chars>",
      "omitted_field_list_hash": "sha256:<64 hex chars>"
    }
  },
  "persistence": "none"
}
```

The export-denial-audit event id must be distinct from the local
`memory.context provider_minimal` read/projection event, the
`provider.context_export` denial event, and the request-binding-denial event:

```text
export_denial_audit_event_id != local_projection_event_id
export_denial_audit_event_id != denial_event_id
export_denial_audit_event_id != request_binding_denial_event_id
```

Positive request/export binding records use different positive schemas:
`raios.provider_request_binding.v0` and
`raios.provider_context_export_audit_binding.v0`. Denial schemas,
`denied_*` statuses, and `positive_export_authorization: false` never satisfy
provider export gates. In this slice, even positive binding records still carry
`satisfies_current_boot_export_gate: false` because automatic context injection
is disabled. The current compact RAM ring is not sufficient to prove durable
positive export authority by itself.

## Invariants

- The log is append-only within the current boot.
- The log is a fixed-size RAM ring and may drop old records.
- Records are evidence locators for current-boot behavior, not persistent
  authority.
- Provider export is disabled.
- Positive provider binding events must keep
  `provider_write_at_binding: not_attempted`,
  `context_attached_to_provider_body: false`, and
  `satisfies_current_boot_export_gate: false` in this slice.
- Binding consumption events must keep `provider_write: not_attempted`,
  `automatic_context_injection: disabled`, and
  `context_attached_to_provider_body: false`.
- Module computed-grant reference events must keep `grants_capability: false`,
  `grants_load_now: false`, `authorizes_guest_load: false`,
  `can_load_now: false`, and `load_attempted: false`.
- Future persistent audit records must bind hashes, approvals, rollback state,
  and durable timestamps separately. This V0 log is not that ledger.
