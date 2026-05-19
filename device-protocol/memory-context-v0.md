# raiOS Memory Context V0

`memory-context-v0` is the first read-only implementation slice of ADR 0004.
It does not create persistent memory. It exposes bounded `current_boot`
context packets over facts that already exist in Stage-0:

```text
system.snapshot.v0
system.capabilities.v0
service.inventory.v0
problem.list.v0
system.boot_log.v0 summary
event.log.v0
ADR metadata
```

The goal is to let an agent ask "what context did I get and why" without
receiving raw logs, secrets, or an unbounded transcript.

## Methods

The serial protocol methods are:

```text
memory.profile
memory.context [diagnostic|planning|provider_minimal]
memory.query
memory.trace [record_id]
memory.recent_events [limit]
audit.events [limit]
```

The current command parser accepts these through `agent <method>` and also has
short local aliases for the read methods. All V0 results are local inspection
results. They are not attached automatically to provider requests.
`audit.events` is an alias for `memory.recent_events`; both expose the same
RAM-only current-boot event ring.

## memory.profile

`memory.profile` returns:

```json
{
  "schema": "memory.profile.v0",
  "scope": "current_boot",
  "profiles": [
    {
      "id": "diagnostic",
      "available": true,
      "target_tokens": 4000,
      "provider_export": false
    },
    {
      "id": "planning",
      "available": true,
      "target_tokens": 8000,
      "provider_export": false
    },
    {
      "id": "provider_minimal",
      "available": false,
      "provider_export": false
    }
  ],
  "mutation_policy": "denied_until_event_log_audit_policy_persistence_and_rollback_exist"
}
```

`provider_minimal` is listed as a planned profile because the provider boundary
needs it later, but it remains unavailable for provider export until:

- provider trust is positive: `pinned_spki_verified`, `pinned_cert_verified`,
  or `webpki_verified`
- a separate provider redaction projection exists
- an audit record can name what left the machine

## memory.context

`memory.context` returns the ADR 0004 context packet schema:

```json
{
  "schema": "raios.agent_context.v0",
  "purpose": "current_boot_system_context",
  "profile": "diagnostic",
  "scope": "current_boot",
  "provider_export": "disabled",
  "source_schemas": [
    "system.snapshot.v0",
    "system.capabilities.v0",
    "service.inventory.v0",
    "problem.list.v0",
    "system.boot_log.v0"
  ],
  "included": {
    "identity": ["mem.fact.identity.stage0"],
    "policy": ["adr.0001", "adr.0004"],
    "current": [
      "snapshot.current",
      "capabilities.current_boot",
      "service.inventory.current",
      "problem.list.current"
    ],
    "summaries": ["boot_log.summary.current"]
  },
  "omitted": []
}
```

The context packet may repeat compact current facts for convenience, but stable
record ids are the primary handle. Raw source material remains behind
`system.snapshot`, `system.boot_log`, `service.inventory`, `problem.list`, or
`memory.trace`.

## Omitted Classes

Every V0 context packet must report omitted classes:

- `raw_boot_log`: use `system.boot_log` or `memory.trace` locally when raw lines
  are needed
- `local_only_details`: prose detail strings can include IPs, PCI ids, request
  ids, hashes, or topology
- `secret_values`: API keys, Wi-Fi passphrases, and raw secret values are never
  included
- `provider_export`: disabled until provider trust, redaction, and audit exist
- `provider_minimal`: currently blocked by either provider trust or the missing
  provider redaction projection

## memory.query

`memory.query` is a static current-boot index in V0. It returns candidate record
ids and short classified summaries. It is not semantic search and must not be
treated as authority without `memory.trace`.

Initial ids include:

```text
mem.fact.identity.stage0
snapshot.current
capabilities.current_boot
service.inventory.current
problem.list.current
boot_log.summary.current
adr.0001
adr.0004
```

The event log is a separate current-boot evidence source. Event ids use the
`event.current_boot.<sequence>` form and can be discovered through
`memory.recent_events` or its `audit.events` alias. These event ids are
locators/evidence records, not persistent memory facts.

## memory.trace

`memory.trace [record_id]` maps known ids to source methods or source files. If
no id is supplied, it returns the default trace set for the current context. If
an id is unknown, the response must say `found: false`.

## Denied Mutations

These methods are protocol vocabulary only and must return
`capability_denied` in Stage-0 V0:

```text
memory.record_observation
memory.propose_policy
memory.supersede_fact
memory.redact
memory.compact
```

The denial must name missing durable evidence such as audit record, policy
ledger, source retention, redaction transaction, persistence layout, and rollback
plan. The denial also includes a current-boot `event_id` and `audit_event_id`.
Memory writes are not allowed to silently land in RAM as if they were durable
memory.

## Invariants

- V0 memory scope is `current_boot`.
- No V0 method persists a memory record.
- Summaries and query hits are locators, not authority.
- Event ids are current-boot evidence locators, not persistent memory authority.
- Secret values are never emitted. State markers such as `set` or `missing` are
  allowed.
- Provider-bound context injection remains disabled even when local
  `memory.context provider_minimal` is requested.
- Unknown fields and unknown record kinds are `local_only` until classified.
