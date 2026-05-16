# SeedOS Capability Policy V0

`system.capabilities.v0` is the first local capability catalog for
`seedos.agent.v0`. It documents what Stage-0 may expose to an agent, what is
only requested by manifests, and why all mutating methods currently fail closed.

V0 is intentionally small:

```text
read-only self-description is granted for the current boot
all mutation is denied unless a later policy can prove local evidence
```

The current implementation source of truth is `seed-kernel/src/agent_protocol.rs`.
This document specifies the policy that file currently exposes over
`system.capabilities` and `capability_denied` responses.

## Policy Invariants

- Capabilities are explicit names, not a generic shell or command runner.
- A capability grant is computed by SeedOS local policy. It is never trusted from
  a manifest, provider response, agent claim, or test report by itself.
- Manifest `requested_caps` are requests only. Manifest `granted_caps` must stay
  empty for V0 artifacts.
- Missing evidence denies the action. Denial is structured and visible as
  `capability_denied`, not a silent fallback.
- Provider context injection is separate from local capability grants and remains
  gated by TLS trust hardening.
- Persistent writes, hardware mutation, live loading, and service control remain
  denied until manifest, VM report, local attestation, computed grant, local
  approval, rollback, and audit records exist.

## Risk Classes

| Risk | Meaning | V0 result |
| --- | --- | --- |
| `observe` | Read local facts, logs, inventory, capabilities, and problems. | Granted for selected current-boot methods. |
| `diagnose` | Run bounded probes or checks that may touch devices but should not change state. | Not granted in Stage-0 V0. |
| `simulate` | Build or test outside the live system, usually in a VM harness. | Evidence may be produced outside the guest; not a guest mutation grant. |
| `modify_ram` | Change current-boot runtime state, load ephemeral code, or alter live services. | Denied. |
| `persist` | Change boot image, saved config, module set, policy, or rollback state. | Denied. |
| `hardware` | Touch physical device state with effects outside a VM or beyond observation. | Denied. |

## Grants Versus Requests

Requests:

- appear in `seedos.module_manifest.v0` as `requested_caps`
- describe what an artifact or agent wants
- do not authorize execution
- may be used by policy, VM harnesses, and review tools to decide what evidence
  is required

Grants:

- appear in `system.capabilities.v0` with `granted: true`
- are computed by local policy for a subject, resource, scope, and duration
- are scoped to `current_boot` for V0 read-only methods
- must be auditable before any future mutating grant exists

A future mutating grant should bind at least:

```json
{
  "id": "cap.module.load_ephemeral",
  "subject": "module:<manifest-name>",
  "resource": "live_service_graph",
  "scope": "current_boot",
  "duration": "ephemeral",
  "grant_source": "local_policy",
  "evidence": [
    "seedos.module_manifest.v0",
    "seedos.vm_test_report.v0",
    "seedos.local_attestation.v0",
    "local_approval",
    "rollback_plan"
  ],
  "audit_level": "full"
}
```

This shape is descriptive only. Stage-0 V0 does not grant it.

## Initial Read-Only Grants

These capabilities are granted for the current boot and map directly to
read-only protocol methods:

| Capability | Method | Risk | Scope | Meaning |
| --- | --- | --- | --- | --- |
| `cap.system.describe.read` | `system.describe` | `observe` | `current_boot` | Read OS and protocol identity. |
| `cap.system.snapshot.read` | `system.snapshot` | `observe` | `current_boot` | Read typed Stage-0 status facts. |
| `cap.system.boot_log.read` | `system.boot_log` | `observe` | `current_boot` | Read the local serial boot log ring. |
| `cap.system.capabilities.read` | `system.capabilities` | `observe` | `current_boot` | Read the current capability catalog. |
| `cap.service.inventory.read` | `service.inventory` | `observe` | `current_boot` | Read static service inventory. |
| `cap.device.graph.read` | `device.graph` | `observe` | `current_boot` | Read known device graph facts. |
| `cap.problem.list.read` | `problem.list` | `observe` | `current_boot` | Read known local problems and gaps. |

`system.snapshot` also reports `capability_denied.for_all_mutating_methods` so an
agent can discover that mutation is intentionally unavailable.

## Denied-By-Default Methods

The following methods are present as protocol vocabulary but must return
`capability_denied` in Stage-0 V0:

```text
module.propose
module.build_result
module.test_request
module.test_result
module.load_ephemeral
module.persist
module.rollback
service.load_ephemeral
service.restart
service.start
service.stop
config.apply
apply_config
provider.configure
wifi.configure
draw_text
probe_device
download_signed_module
run_module_test
```

The denial must name the missing evidence set:

```text
seedos.module_manifest.v0
seedos.vm_test_report.v0
local_attestation.v0
computed_capability_grant
local_approval
rollback_plan
```

V0 does not distinguish between "method known but currently unsafe" and "method
will be allowed soon" by returning partial success. The durable behavior is:
known mutating method -> structured denial; unknown method -> unknown command.

## Audit Expectation

Read-only methods may be recorded in the serial log and protocol transcript.
That is sufficient for V0 observation.

Before any mutating grant can exist, SeedOS must emit or persist an audit record
that can explain the decision later. The minimum record should include:

```text
agent session id
request id
method
requested capability
subject
resource
risk
manifest hash
artifact hash
VM report hash
local attestation hash
approval decision
load mode
rollback pointer or rollback denial reason
policy decision: granted | denied
decision reason
timestamp or boot-relative sequence
```

For persistence, the audit record must also bind the previous-good boot pointer,
pending artifact set, boot-success marker plan, rollback trigger, and safe-mode
rule that disables non-core modules and persistent writes.

## Provider Boundary

Capability grants are local execution authority. They are not permission to send
the same data to a provider.

Fields that leave the machine still need redaction classification:
`public`, `local_only`, or `secret`. Until TLS certificate verification or
provider/SPKI pinning is fail-closed, provider requests must not automatically
attach `system.snapshot.v0` or capability/audit context. Local serial access may
show read-only facts that the provider path is not yet allowed to receive.

## Open Questions

- What exact subject identifiers should V0 use for local users, provider
  sessions, workstation-side tools, guest diagnostics, and live services?
- Where should audit records live before persistent storage and rollback slots
  exist?
- Which low-risk diagnostic capability, if any, is the first candidate for a
  non-observe grant?
- Should `module.propose` remain denied in the guest while workstation-side
  proposal tooling accepts manifests outside the SeedOS runtime?
