# ADR 0004: System Memory And Agent Context Selection

## Status
Draft for the memory and context architecture pass.

## Context
raiOS should not treat memory as a chatbot feature bolted onto the UI. The
Tamagotchi model is stronger than that: the local system is a bounded life-form
with one machine, one user, one history, and a small set of durable rules.

The useful memory for an agent is therefore not the full transcript of every
conversation. It is typed system state with provenance:

```text
identity
policy
current facts
events
decisions
problems
service state
capability grants and denials
test evidence
rollback history
derived summaries and indexes
```

The hard constraint is context size. If raiOS stores every boot event, module
proposal, test report, crash, and conversation, the memory will quickly become
far too large to put into a provider request. Sending more context is also a
security risk: local addresses, hardware details, prompts, keys, hashes, and
unclassified logs must not leak simply because they matched a search query.

This ADR defines memory as a local system substrate and defines how an agent
receives only the relevant projection for a task.

## Decision
raiOS should build a local system memory model where authoritative memory is
typed, evidence-bound, and capability-gated. Provider prompts should receive a
small `agent_context.v0` packet assembled by a local context broker, not raw
memory.

The core rule is:

```text
memory is not prompt stuffing
memory is a local fact/evidence system with task-scoped projections
```

The agent may ask for memory, but it does not directly scan or own all memory.
It receives context through explicit protocol methods:

```text
memory.profile
memory.context
memory.query
memory.trace
memory.recent_events
memory.decisions
memory.problems
```

Mutating memory methods should exist as vocabulary but remain denied until
policy, audit, and persistence exist:

```text
memory.record_observation
memory.propose_policy
memory.supersede_fact
memory.redact
memory.compact
```

For V0, read-only memory can be derived from the existing `system.snapshot.v0`,
`service.inventory`, `problem.list`, ADRs, and boot log. Persistent memory
writes remain denied until the persistence and rollback layer exists.

## Memory Authority Levels
Not all memory has the same authority. If records disagree, the agent and local
policy must prefer the strongest applicable source.

| Level | Source | Authority | Example |
| --- | --- | --- | --- |
| `core_ledger` | permanent core or future capability ledger | highest | capability grant, rollback root, trust anchor |
| `evidence` | VM reports, local attestation, manifests, hashes | high | tested artifact hash passed smoke |
| `current_snapshot` | live typed system state | high for current boot | e1000 DHCP configured |
| `decision` | ADRs and explicit user approvals | high for intent | do not port Codex CLI into Stage-0 |
| `service_state` | service-owned typed state | medium | provider adapter phase idle |
| `event` | append-only observation | medium | TLS pin mismatch occurred |
| `summary` | derived compaction over records | low | provider trust work is next |
| `semantic_index` | embeddings, keywords, graph edges | locator only | likely relevant records |
| `chat_history` | conversation text | lowest | user discussed editor preferences |

Derived summaries and retrieval indexes are never authority. They are pointers
back to source records. A mutating decision must cite authoritative records,
not only a summary or vector hit.

## Memory Store Shape
A durable memory record should be small, typed, and traceable:

```json
{
  "schema": "raios.memory_record.v0",
  "id": "mem.event.00000042",
  "kind": "event",
  "entity": "svc.provider.openai_direct",
  "predicate": "provider_trust_denied",
  "value": {
    "reason": "pin_config_missing"
  },
  "classification": "public",
  "authority": "event",
  "boot_id": "boot:...",
  "sequence": 42,
  "source": {
    "method": "system.snapshot",
    "record_id": "snapshot:..."
  },
  "evidence": [
    "problem:provider.tls_pin_config_missing"
  ],
  "tags": [
    "provider",
    "tls",
    "trust"
  ],
  "supersedes": [],
  "created_at": {
    "clock": "boot_relative",
    "ticks": 12345
  }
}
```

Facts should be asserted and superseded, not overwritten in place. Deletion is a
redaction transaction, not silent removal. This lets the system explain why an
agent believed something and when that belief stopped being valid.

## Memory Classes
The memory system should reuse the snapshot redaction classes:

```text
public      may be included in provider context after provider trust is verified
local_only  local agent context only unless a specific policy grants export
secret      never expose the value; expose only state markers such as set/missing
```

Unknown fields and unknown record kinds are `local_only` until classified.
Secret values must not be embedded into semantic indexes, summaries, problem
summaries, provider context, or denial messages. For secrets, memory may store a
state marker and a sealed handle, but not a retrievable plaintext value.

## Token Budget Model
raiOS should assume memory is always bigger than context. Every agent request
must have an explicit context budget.

Initial budget targets:

| Context profile | Target | Purpose |
| --- | --- | --- |
| `recovery_minimal` | 512-1500 tokens | core recovery lifeline and severe failure reports |
| `provider_minimal` | 1000-3000 tokens | normal provider calls with current state only |
| `diagnostic` | 3000-8000 tokens | troubleshooting one subsystem |
| `planning` | 8000-16000 tokens | architecture planning or multi-step design |
| `deep_trace` | tool-driven, not automatic | raw logs, test reports, source excerpts |

These are targets, not rights. If the broker cannot fit enough context, it must
say what was omitted and expose follow-up queries. It should not silently stuff
more memory into the prompt.

## Context Packet
The agent receives a context packet, not the memory store:

```json
{
  "schema": "raios.agent_context.v0",
  "purpose": "diagnose_provider_trust",
  "profile": "diagnostic",
  "budget": {
    "target_tokens": 4000,
    "estimated_tokens": 1820
  },
  "authority_order": [
    "core_ledger",
    "evidence",
    "current_snapshot",
    "decision",
    "event",
    "summary"
  ],
  "included": {
    "identity": ["mem.fact.identity.stage0"],
    "policy": ["adr.0001", "adr.0002"],
    "current": ["snapshot.current"],
    "problems": ["problem.provider.tls_pin_config_missing"],
    "retrieved": ["mem.event.00000042"]
  },
  "omitted": [
    {
      "kind": "boot_log",
      "reason": "not_relevant_to_provider_trust"
    },
    {
      "kind": "network_detail",
      "reason": "local_only_for_provider_profile"
    }
  ]
}
```

Every included record should have an id so the agent can ask `memory.trace` for
source evidence. The context packet should prefer stable ids, states, and short
summaries over prose logs.

## Retrieval Strategy
RAG is useful, but it must be the last part of a structured retrieval pipeline,
not the whole memory model.

Context selection should happen in this order:

1. Determine request intent and risk.
2. Select a context profile and token budget.
3. Include mandatory tiny invariants.
4. Include current live facts and current problems.
5. Include directly linked decisions, policies, and capability denials.
6. Retrieve task-specific records.
7. Re-rank by authority, freshness, severity, and evidence links.
8. Apply redaction and provider-trust gates.
9. Emit a bounded context packet with omissions.

The retrieval layer should be hybrid:

| Retrieval type | Role |
| --- | --- |
| structured selectors | exact facts such as service id, capability id, problem id |
| graph traversal | links between service, event, artifact, report, and decision |
| keyword search | stable ids, filenames, device names, error codes |
| semantic search | fuzzy relevance over summaries and non-secret text |
| recency window | recent events for the active task |
| severity boost | high-severity problems and denials outrank casual notes |

Semantic retrieval may find candidate records, but it does not decide authority.
The broker must trace semantic hits back to typed records before including them.

## Always-Included Context
Only a very small memory set should be always included:

```text
raiOS identity and stage
current provider trust state
current capability posture
current problem count and high-severity ids
active task id, if any
memory profile and budget
```

Even architectural principles should be compact. The full README, ADRs, logs,
and histories are fetched by id only when relevant.

## Agent Awareness
An agent becomes memory-aware when it can inspect what context it got and ask
why:

```text
memory.profile
  returns available profiles, budgets, and redaction behavior

memory.context { purpose, profile }
  returns the broker-selected packet

memory.query { filter, query, limit }
  returns candidate record ids and short classified summaries

memory.trace { id }
  returns source links and evidence for one record

memory.decisions { topic }
  returns relevant ADRs and explicit approvals

memory.problems { service_id }
  returns current and recent problem records
```

The model should not have to guess whether context was omitted. The packet must
tell it the profile, budget, and omission reasons.

## Memory Writes
Memory writes are risky because they can poison future behavior. V0 should treat
them like other mutating agent actions.

Rules:

- An agent may propose a memory write; local policy decides whether it lands.
- User preference changes require explicit user approval or a future visible UI
  flow.
- Service observations must name the service and source snapshot.
- Provider responses cannot create policy by themselves.
- Summaries are regenerated from source records and can be discarded.
- A record that affects capabilities, provider export, persistence, rollback, or
  trust must be evidence-bound.

Initial mutating methods should return `capability_denied` with the missing
evidence:

```text
memory.record_observation -> denied until event log and audit exist
memory.propose_policy     -> denied until policy ledger exists
memory.compact            -> denied until source retention exists
memory.redact             -> denied until redaction transaction exists
```

## Compaction And Summaries
Compaction is allowed only as a derived cache. A summary must include:

```text
source record ids
source hash or generation
created_by
profile
classification
expiration or invalidation rule
```

If a source record changes, is superseded, or is redacted, dependent summaries
become stale. The broker may still use a stale summary as a hint for retrieval,
but not as an authoritative claim.

## Provider Boundary
Local memory and provider context are separate. A record being readable locally
does not mean it may leave the machine.

Provider context requires all of these gates:

```text
positive provider trust state
known context profile
field classification
redaction applied
token budget applied
audit record for exported context
```

If provider trust is missing, bypassed, failed, or unknown, the broker may still
build local context for the framebuffer or serial agent path, but automatic
provider context injection must remain disabled.

## Current Stage-0 Slice
The first durable slice should not require persistent storage or vector search.
It should be a read-only broker over existing facts:

```text
system.snapshot.v0
system.capabilities.v0
service.inventory.v0
problem.list
system.boot_log
ADR ids and short titles
```

V0 output:

```text
memory.profile
memory.context provider_minimal
memory.context diagnostic
memory.query over current snapshot, problems, capabilities, and ADR titles
memory.trace for included record ids
```

All memory mutation remains denied.

## Implementation Phases

### Phase A: Document Schemas
Define:

```text
raios.memory_record.v0
raios.agent_context.v0
memory.profile
memory.context
memory.query
memory.trace
```

### Phase B: Read-Only Context Broker
Build a broker that assembles context from current `system.snapshot.v0`,
capabilities, problems, service inventory, and ADR metadata. No persistence and
no embeddings are required.

### Phase C: Event Log
Add an append-only current-boot event log with sequence ids. It can be RAM-only
at first, but the schema should already match the future persistent form.

### Phase D: Persistent Memory Layout
After the image/state layout exists, persist event logs, decisions, local
approvals, redactions, and rollback-linked records.

### Phase E: Hybrid Retrieval
Add local keyword and graph retrieval first. Add semantic retrieval only for
non-secret text and derived summaries. The semantic index is a locator, not an
authority.

### Phase F: Provider Context Injection
Enable `provider_minimal` only after the provider transport is positively
verified and the redaction profile is implemented in code.

### Phase G: Compaction And Sleep-Time Work
During idle time, regenerate summaries, detect stale records, suggest policy
cleanups, and prepare small task-specific context packets. These background
tasks may propose changes but must not silently rewrite authoritative memory.

## Failure Modes

| Failure | Required behavior |
| --- | --- |
| Memory grows without bound | context broker enforces budgets and omissions |
| Semantic search finds stale text | trace to source and re-rank by freshness |
| Summary contradicts evidence | evidence wins; summary is stale |
| Provider profile sees local-only field | block export and report redaction error |
| Agent proposes false memory | deny or record as untrusted proposal |
| User preference conflicts with policy | policy/capability ledger wins until changed by approval |
| Record classification is unknown | treat as `local_only` |
| Secret appears in free text | redact, mark source contaminated, do not index semantically |

## Non-Goals
This decision does not mean:

- Sending the whole memory store to the provider.
- Treating RAG results as authoritative facts.
- Letting the agent silently rewrite user preferences or system policy.
- Persisting secrets as text memory.
- Making summaries a replacement for logs, test reports, manifests, or audit
  records.
- Enabling provider context injection before the transport trust and redaction
  gates are implemented.

## Open Questions

- What is the first persistent storage format for append-only memory records?
- Should semantic retrieval use embeddings, lexical BM25-style search, or both
  for the first local implementation?
- Which memory profiles are allowed over the recovery lifeline?
- How should conflicts be presented in the framebuffer UI?
- How should user-visible memory approval work before there is a full settings
  service?
- Should ADRs and docs be imported into memory as records, or referenced by
  stable file path and content hash?
