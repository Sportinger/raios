# raiOS Recovery Protocol V0

`raios.recovery.v0` is the minimal recovery-control path for the permanent
raiOS core. It is separate from the normal `raios.agent.v0` chat/provider
surface and exists for the case where the replaceable service world is degraded,
crashed, or unsafe to trust.

V0 is a specification only. Stage-0 does not yet implement this runtime control
path. Until the permanent core has service supervision, rollback state, audit
records, and a trusted recovery transport, every mutating recovery method must
return `capability_denied`.

## Purpose

The recovery path has one job:

```text
live world unsafe -> core stays reachable -> inspect bounded state
-> perform narrow, auditable recovery actions only when local evidence allows
```

It is not a second general agent API. It does not expose chat, shell commands,
provider tool calls, arbitrary file access, or unbounded diagnostics.

## Transport Boundary

The recovery endpoint belongs to the permanent core, not to replaceable services.
It may be exposed over one or more future bounded transports:

- local serial recovery console
- local physical-link recovery channel
- pinned minimal provider route, only after a separate recovery trust gate

The normal OpenAI direct provider service is not a recovery transport. It is a
replaceable service candidate named `svc.provider.openai_direct`, depends on the
normal network/TLS/provider stack, has only normal provider trust rather than
recovery trust, and can itself be crashed, disabled, misconfigured, or rolled
back. A component that may need recovery cannot also be the trusted recovery
lifeline.

## Trust States

Recovery trust is explicit and separate from provider trust:

```json
{
  "schema": "raios.recovery_trust.v0",
  "transport": "serial_local",
  "trust_state": "local_physical_console",
  "core_generation": "sha256:...",
  "policy_generation": "sha256:...",
  "last_good_set": "sha256:...",
  "capabilities_enabled": [
    "cap.recovery.snapshot.read"
  ]
}
```

Allowed `trust_state` values:

| State | Meaning | Mutating recovery allowed |
| --- | --- | --- |
| `unknown` | No trusted recovery channel has been established. | no |
| `local_physical_console` | Local serial or physical console access only. | only after local approval and evidence |
| `pinned_recovery_peer_verified` | A dedicated recovery peer was verified by a recovery pin. | only after policy allows it |
| `provider_route_unverified` | A provider path exists but is not verified as a recovery route. | no |
| `policy_locked` | Core is reachable but policy forbids mutation for this boot. | no |
| `safe_mode` | Non-core modules are disabled and persistent writes are blocked. | bounded current-boot repair only |

Provider trust states such as `pinned_spki_verified` for OpenAI are not
automatically recovery trust states. Recovery authority must bind to the core,
transport, policy generation, and requested action.

## Method Set

V0 defines these methods:

```text
recovery.snapshot
recovery.restart_last_good
recovery.disable_module
recovery.rollback
recovery.load_artifact_by_hash
```

The method names intentionally live outside normal chat commands such as
`ask <text>` and outside normal provider tools.

## recovery.snapshot

`recovery.snapshot` is the only V0 method that may be granted before recovery
mutation exists. It is read-only and reports the core's recovery view.

Required shape:

```json
{
  "schema": "raios.recovery_snapshot.v0",
  "core": {
    "state": "alive",
    "safe_mode": false,
    "core_generation": "sha256:...",
    "policy_generation": "sha256:..."
  },
  "trust": {
    "transport": "serial_local",
    "trust_state": "local_physical_console"
  },
  "live_world": {
    "state": "degraded",
    "last_good_set": "sha256:...",
    "active_set": "sha256:...",
    "boot_success_mark": "missing"
  },
  "crashed_services": [
    {
      "id": "svc.provider.openai_direct",
      "last_version": "sha256:...",
      "crash_count": 1,
      "last_error_id": "provider.tls_pin_config_missing",
      "restartable": false,
      "disabled": false
    }
  ],
  "disabled_modules": [],
  "available_artifacts": [
    {
      "hash": "sha256:...",
      "source": "local_recovery_store",
      "manifest_hash": "sha256:...",
      "vm_report_hash": "sha256:...",
      "attestation_hash": "sha256:..."
    }
  ],
  "allowed_actions": [
    "recovery.snapshot"
  ],
  "denied_actions": [
    {
      "method": "recovery.restart_last_good",
      "reason": "missing_recovery_runtime"
    }
  ]
}
```

The snapshot must not include provider API keys, Wi-Fi passphrases, raw prompts,
raw crash memory, local network addresses, or unredacted logs. Crash records are
identified by stable ids and hashes.

## Mutating Actions

All mutating recovery actions are denied by default. A future grant requires:

```text
trusted recovery transport
matching core and policy generation
service inventory
crash or health evidence
artifact/module hash binding
module manifest when an artifact is involved
VM test report for new or replaced code
local attestation or local approval
rollback target
audit record
```

### recovery.restart_last_good

Restarts the last-good service set for the current boot. It must not start a
service set that lacks a success marker or was superseded by a rollback block.

Allowed only when:

- the last-good pointer is present and hash-verified
- the active service set is crashed, degraded, or explicitly stopped
- the action does not require loading unknown artifacts
- audit storage for the decision is available

V0 result in Stage-0:

```json
{
  "error": "capability_denied",
  "method": "recovery.restart_last_good",
  "missing_evidence": [
    "service_supervisor",
    "last_good_pointer",
    "audit_record",
    "computed_recovery_grant"
  ]
}
```

### recovery.disable_module

Disables a module id or artifact hash for the current boot, and optionally for a
future safe-mode boot only after persistence policy exists.

Allowed only when:

- the target is not `core_owned`
- the target module id or artifact hash is known in service inventory
- disabling it cannot remove the recovery endpoint itself
- the denial or disable decision is auditable

The method must fail closed for unknown targets, core services, and broad
patterns such as "disable all".

### recovery.rollback

Switches persistent state or service set pointers back to a known rollback
target. It is a persistence-risk action and therefore stricter than a current
boot restart.

Allowed only when:

- the rollback target hash is present in the recovery store
- the target has a valid boot-success mark or equivalent local attestation
- the current failed generation is recorded before the switch
- safe mode can disable non-core modules if the rollback also fails

Stage-0 has no persistent recovery layout yet, so this method is specified but
must remain denied.

### recovery.load_artifact_by_hash

Loads an artifact already present in the local recovery store by exact hash. The
request supplies a hash, not a URL, provider text, or downloaded blob.

Allowed only when:

- the artifact bytes are already local
- the artifact hash matches the manifest and recovery-store index
- the manifest requests a recovery-compatible load mode
- the VM report and local attestation bind the same artifact hash, manifest
  hash, base image hash, and policy generation
- local policy computes a recovery grant

This method must not fetch code from OpenAI, a chat response, or an arbitrary
network endpoint. Network retrieval, if ever allowed, is a separate signed
artifact acquisition path and not part of recovery V0.

## Denied-By-Default Rules

Recovery V0 denies on any missing or ambiguous evidence:

- unknown method
- unknown service id or module id
- missing trust state
- provider-only trust state
- missing service inventory
- missing crash or health evidence
- missing last-good pointer
- missing rollback target
- missing manifest, VM report, attestation, or computed grant
- target is `core_owned`
- action would disable the recovery endpoint
- action would write persistence while in `safe_mode`
- request includes free-form code, shell commands, URLs, provider messages, or
  non-hash artifact identifiers

Denials are explicit protocol results. There is no fallback to normal chat,
OpenAI direct, host-side relay, shell execution, or best-effort mutation.

## Capability Names

Initial recovery capability ids:

```text
cap.recovery.snapshot.read
cap.recovery.restart_last_good
cap.recovery.disable_module
cap.recovery.rollback
cap.recovery.load_artifact_by_hash
```

Only `cap.recovery.snapshot.read` is eligible for an early V0 grant. All others
require the full evidence chain above.

## Relationship To Existing V0 Docs

- `system.snapshot.v0` describes the normal current-boot machine snapshot.
- `service.inventory.v0` identifies replaceable services and `core_owned`
  responsibilities.
- `provider-trust-v0.md` gates normal provider context, not recovery authority.
- `module-manifest-v0.md`, `vm-test-report-v0.md`, and
  `local-attestation-v0.md` provide evidence for future artifact-related
  recovery decisions.

Recovery V0 may reuse hashes and service ids from those documents, but it does
not inherit normal agent or provider permissions.
The normal `raios.module_load_gate.v0` denial for `module.load_ephemeral` is
also not recovery authority. Recovery artifact loads must be evaluated through
`recovery.load_artifact_by_hash`, recovery trust state, recovery policy
generation, and recovery audit evidence.

## Open Questions

- What is the first concrete recovery transport: serial-only, a local physical
  link, or a dedicated pinned minimal provider route?
- Where will the permanent core store the last-good pointer, disabled-module
  list, rollback targets, and audit records before a full persistent layout
  exists?
- What exact service-supervisor evidence is required to mark a service as
  crashed rather than merely degraded?
- Should `safe_mode` allow current-boot `restart_last_good`, or should it be
  read-only until explicit local approval is entered?
- Which artifact kinds are recovery-loadable by hash, and which must remain
  normal-world module loads only?
