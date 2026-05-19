# Provider Request Envelope V0

`raios.provider_request_envelope.v0` is the local pre-write record for one real
provider request. It exists so future provider context export can bind a
redacted `provider_minimal` packet to the exact outbound request that would
carry it, without storing raw prompts or API keys.

Stage-0 now emits this envelope on the real direct OpenAI `ask` path before DNS,
TCP, TLS, API-key copy, or HTTPS write. `provider.context_export` must continue
to deny export and must not create this record by itself.

This envelope is not a positive provider context export binding. It records the
local request shape and exact body hash. When provider trust is positive on the
same request path, Stage-0 records
`raios.provider_request_binding.v0` and
`raios.provider_context_export_audit_binding.v0` records against this envelope.
Context attachment still remains disabled in the current slice.

## Creation Point

The envelope may be created only by the direct provider request path after a
local request id has been allocated and before any provider write is attempted.
It must not be created by `provider.context_export`, by a standalone memory
projection, or by a test-only placeholder.

Valid pre-write state:

```text
provider_write: not_attempted
request_body_ready: local_only
api_key_value: not_copied
context_attached_to_provider_body: false
```

The envelope records the planned wire shape and local routing facts. It is not
evidence that bytes left the machine.

## Shape

```json
{
  "schema": "raios.provider_request_envelope.v0",
  "id": "provider_request_envelope.current_boot.00000042",
  "scope": "current_boot",
  "classification": "local_only",
  "persistence": "none",
  "status": "local_prewrite_envelope",
  "provider_write": "not_attempted",
  "source": {
    "method": "ask",
    "capability": "cap.provider.request",
    "risk": "export",
    "code_path": "seed-kernel/src/openai.rs"
  },
  "provider": {
    "selected": "OPENAI",
    "route": "OPENAI DIRECT",
    "host": "api.openai.com",
    "port": 443,
    "method": "POST",
    "path": "/v1/responses",
    "model": "gpt-5.4"
  },
  "request_body": {
    "schema": "openai.responses.request.redacted.v0",
    "user_prompt": "present_redacted",
    "max_output_tokens": 128,
    "store": false,
    "context_attached_to_provider_body": false,
    "body_sha256": "sha256:<64 hex chars>"
  },
  "secret_state": {
    "api_key_state": "set_or_missing",
    "authorization_header": "redacted",
    "api_key_value": "not_recorded"
  },
  "provider_minimal_context": {
    "attached": false,
    "binding_status": "not_bound"
  },
  "trust_snapshot": {
    "provider_trust_state": "pin_config_missing",
    "provider_trust_positive": false,
    "development_tls_bypass": false
  },
  "evidence": {
    "canonicalization": "raios.provider_request_envelope.canonical.v0",
    "envelope_hash": "sha256:<64 hex chars>"
  }
}
```

## Classification

Public fields may later be exported only through a positive export binding:

- schema ids
- provider family and canonical route labels
- model id
- redaction/profile schema ids
- future packet and field-list hashes when a positive request binding exists
- coarse API-key state markers
- trust-state markers

Local-only fields remain local:

- envelope id and local event ids
- source method and code path
- full provider endpoint tuple as part of the complete envelope
- request-body hash
- envelope hash
- provider write state

Secret fields must never be recorded:

- raw user prompt
- API key value
- Authorization header value
- Wi-Fi secrets or other unrelated local secrets

Do not include `Content-Length`; it can leak prompt length. If body bytes are
hashed, that hash is `local_only` because short prompts can be guessed.

## Positive Binding Predicate

`raios.provider_request_binding.v0` may bind context to a provider request only
when all of these are true:

- the envelope schema is `raios.provider_request_envelope.v0`
- the envelope was created by the real provider request path
- `provider_write` was `not_attempted` at binding time
- the request envelope hash matches the bound envelope
- the request body hash matches the exact body prepared for the write
- the provider-minimal packet, exported-field-list, and omitted-field-list
  hashes match the projection being exported
- provider trust is positive at binding time and not a development bypass
- the binding is single-use and has not already been consumed
- automatic context injection remains a separate final gate

Denial records, request-attempt ids, planned requests, stale event ids, dropped
RAM-ring events, or records from a previous boot do not satisfy this predicate.
The checked consumption gate also rejects substituted binding schemas, mismatched
request-body or envelope hashes, mismatched binding hashes, non-positive trust
records, development TLS bypass records, and already consumed pairs. A consumed
pair is evidence that the local gate evaluated the binding; it is not evidence
that context was attached to a provider body.

## Runtime Marker

The current Stage-0 slice writes a local serial marker:

```text
OPENAI_PROVIDER_REQUEST_ENVELOPE {"schema":"raios.provider_request_envelope.v0", ...}
```

The marker is local-only diagnostic evidence. It includes body and envelope
hashes but does not include raw prompt text, `Content-Length`, API keys, or
Authorization header values. The direct OpenAI smoke checks that the marker
appears on real request paths and remains absent when the trust gate refuses to
start a provider request.

On pinned/WebPKI positive trust paths, the runtime also emits local-only
diagnostic markers for the positive binding records:

```text
OPENAI_PROVIDER_REQUEST_BINDING {"schema":"raios.provider_request_binding.v0", ...}
OPENAI_PROVIDER_EXPORT_AUDIT_BINDING {"schema":"raios.provider_context_export_audit_binding.v0", ...}
```

Those markers must remain absent for development TLS bypass, pin mismatch, and
other non-positive trust states. They bind hashes only and do not imply that
provider-minimal context was attached to the request body.
