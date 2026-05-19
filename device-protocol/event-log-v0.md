# raiOS Event Log V0

`event.log.v0` is the first RAM-only event/audit surface for ADR 0004 memory.
It records bounded `current_boot` protocol evidence without creating persistent
memory or a durable audit ledger.

The implementation source of truth is `seed-kernel/src/event_log.rs`, with
recording hooks in `seed-kernel/src/agent_protocol.rs`.

## Scope

V0 records:

- read-only `raios.agent.v0` responses
- `capability_denied` outcomes for known mutating methods

V0 does not record raw request payloads, provider prompts, API keys, Wi-Fi
secrets, boot-log text, or arbitrary response bodies. It records method names
and compact policy metadata only.

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
  "persistence": "none"
}
```

Denied records use:

```text
kind: agent_protocol.capability_denied
outcome: capability_denied
reason: missing_evidence
evidence: missing_required_evidence, capability_denied
```

Denial responses include `event_id` and `audit_event_id`, both pointing at the
current-boot record id. These IDs are evidence handles, not durable memory.

## Invariants

- The log is append-only within the current boot.
- The log is a fixed-size RAM ring and may drop old records.
- Records are evidence locators for current-boot behavior, not persistent
  authority.
- Provider export is disabled.
- Future persistent audit records must bind hashes, approvals, rollback state,
  and durable timestamps separately. This V0 log is not that ledger.
