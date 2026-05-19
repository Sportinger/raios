# Provider Context Export V0

`raios.provider_context_export.v0` is the first explicit gate for sending a
redacted `raios.agent_context.v0` packet to an AI provider. It does not enable
automatic provider context injection.

Stage-0 currently emits packet/field-list evidence and denial-audit evidence,
including hash-valued denial bindings in `event.log.v0`, for standalone
`provider.context_export` attempts. The direct OpenAI `ask` path now emits a
real `raios.provider_request_envelope.v0` before any provider write; however,
`provider.context_export` itself must not create or fake that envelope. When
pinned/WebPKI trust is positive on the real request path, Stage-0 records
positive `raios.provider_request_binding.v0` and
`raios.provider_context_export_audit_binding.v0` events. Automatic context
injection still remains disabled, so the OpenAI request body does not receive
provider-minimal context and `provider.context_export` continues to deny
standalone export attempts.

## Method

```text
provider.context_export [provider_minimal]
```

`provider.export_context` is accepted as an alias. The only V0 profile is
`provider_minimal`.

## Denial Shape

The method returns an error envelope:

```json
{
  "v": "raios.agent.v0",
  "t": "error",
  "body": {
    "method": "provider.context_export",
    "event_id": "event.current_boot.00000009",
    "audit_event_id": "event.current_boot.00000011",
    "code": "capability_denied",
    "schema": "raios.provider_context_export.v0",
    "request": {
      "provider": "OPENAI",
      "route": "OPENAI DIRECT",
      "profile": "provider_minimal",
      "profile_supported": true,
      "context_schema": "raios.agent_context.v0",
      "projection_schema": "raios.provider_context_projection.v0",
      "export_schema": "raios.provider_context_export.v0",
      "requested_capability": "cap.provider.context_export",
      "export_scope": "single_provider_request"
    },
    "gate_state": {
      "provider_trust_state": "pin_config_missing",
      "provider_trust_positive": false,
      "redaction_projection": "present",
      "field_classification": "present",
      "packet_evidence_binding": "present",
      "exported_field_list_binding": "present",
      "omitted_field_list_binding": "present",
      "provider_request_binding": "missing",
      "provider_request_binding_denial": "present_denied_not_bound",
      "provider_export_audit_binding": "missing",
      "provider_export_denial_audit": "present_denied_no_provider_write",
      "provider_write": "not_attempted",
      "can_export": false
    },
    "provider_request_binding_denial": {
      "schema": "raios.provider_request_binding_denial.v0",
      "id": "provider_request_binding_denial.current_boot.00000010",
      "attempted_request_id": "provider_request_attempt.current_boot.00000010",
      "event_id": "event.current_boot.00000010",
      "status": "denied_not_bound",
      "satisfies_export_gate": false,
      "provider_write": "not_attempted"
    },
    "export_denial_audit": {
      "schema": "raios.provider_context_export_denial_audit.v0",
      "id": "provider_context_export_denial_audit.current_boot.00000011",
      "event_id": "event.current_boot.00000011",
      "status": "denied_no_provider_write",
      "satisfies_export_gate": false,
      "positive_export_authorization": false,
      "denial_event_id": "event.current_boot.00000009",
      "provider_write": "not_attempted"
    },
    "blocked_by": [
      {
        "gate": "provider_trust",
        "state": "pin_config_missing",
        "reason": "provider_trust_not_positive"
      },
      {
        "gate": "provider_request_binding",
        "state": "missing",
        "reason": "provider_request_binding_missing"
      },
      {
        "gate": "provider_context_export_audit_binding",
        "state": "missing",
        "reason": "provider_context_export_audit_binding_missing"
      },
      {
        "gate": "provider_write_path",
        "state": "disabled",
        "reason": "automatic_context_injection_disabled"
      }
    ],
    "evidence": {
      "local_projection_method": "memory.context provider_minimal",
      "local_projection_locator": "snapshot.current.provider_minimal",
      "packet_canonicalization": "raios.provider_minimal.packet.canonical.v0",
      "projected_packet_hash": "sha256:<64 hex chars>",
      "exported_field_list_hash": "sha256:<64 hex chars>",
      "omitted_field_list_hash": "sha256:<64 hex chars>",
      "provider_request_binding_status": "missing",
      "provider_request_binding_denial_id": "provider_request_binding_denial.current_boot.00000010",
      "provider_request_binding_denial_event_id": "event.current_boot.00000010",
      "export_audit_binding_status": "missing",
      "export_denial_audit_id": "provider_context_export_denial_audit.current_boot.00000011",
      "export_denial_audit_event_id": "event.current_boot.00000011",
      "export_denial_audit_satisfies_export_gate": false,
      "denial_event_is_export_binding": false,
      "denial_event_id": "event.current_boot.00000009"
    }
  }
}
```

The denial event, request-binding-denial event, and export-denial-audit event
are distinct current-boot events. None of them is a positive export binding.

## Positive Binding Requirements

The positive `raios.provider_request_binding.v0` binds one exact packet hash to
one real `raios.provider_request_envelope.v0` before any provider write. A
placeholder request, denial record, request-attempt id, or
planned-but-not-dispatched request does not satisfy this gate.

The positive `raios.provider_context_export_audit_binding.v0` binds that request
binding, the packet hash, the exported and omitted field-list hashes, and the
positive provider trust state. It is distinct from:

- the local `memory.context provider_minimal` read/projection event
- the `provider.context_export` denial event
- the request-binding-denial event

A positive export decision must reject:

- `raios.provider_request_binding_denial.v0`
- `raios.provider_context_export_denial_audit.v0`
- any binding with `status` beginning with `denied_`
- any binding with `satisfies_export_gate: false`
- any binding with `positive_export_authorization: false`

In the current slice, even a positive export audit binding still carries
`satisfies_current_boot_export_gate: false` because
`automatic_context_injection` is explicitly `disabled`. That final gate must be
enabled separately before context can be attached to a provider body.

## Required Gates

Provider context export requires all of these gates:

- `positive_provider_trust`: `pinned_spki_verified`, `pinned_cert_verified`, or
  `webpki_verified`
- `raios.provider_context_projection.v0`
- `projected_packet_hash`
- `exported_field_list_hash`
- `omitted_field_list_hash`
- `raios.provider_request_envelope.v0` for exactly one local provider request
- `raios.provider_request_binding.v0` for exactly one real provider request
- `raios.provider_context_export_audit_binding.v0`
- `audit.event.v0`

Until all context-export gates and the separate final injection gate pass,
provider-minimal context attachment must remain false. Standalone
`provider.context_export` attempts must continue to report `provider_write:
not_attempted`.

## Invariants

- Raw `system.snapshot`, boot logs, local-only details, secret values, provider
  prompt text, network topology, TCP diagnostics, and unclassified memory
  context are never exported by this method.
- The only V0 export profile is `provider_minimal`.
- Denial-audit records are evidence that export did not happen; they cannot
  satisfy export gates.
- The provider request path must not attach context automatically until a
  separate final injection gate consumes a positive request binding and positive
  export audit binding.
