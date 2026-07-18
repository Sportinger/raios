# ADR 0002: Agent Self-Description And Live-Built Modules

## Status
Draft for the next protocol/design pass.

ADR 0003 defines the long-term target: an always-on core with a
live-rebuildable service world. This ADR remains the V0 safety and protocol pass
for self-description, local attestation, manifests, and test reports. Where this
document starts with workstation-side or VM-tested artifacts, that is an
implementation ramp, not the final product boundary.

The first implementation of this ADR should be read-only. `system.snapshot`,
`system.capabilities`, `system.boot_log`, `device.graph`, `problem.list`, and
`service.inventory` can be exposed through console or an agent dispatcher before
any live-loading exists. Mutating methods such as `module.load_ephemeral`,
`module.persist`, and `module.rollback` should be present in the protocol as
denied-by-default until a matching VM test report and local attestation record
exist.

## Context
raiOS is meant to be a small bootable agent host, not a Linux
distribution with AI tools preinstalled. The useful distinction is not that the
system has fewer components. The useful distinction is that the OS can describe
itself to an agent in a structured way and can constrain agent actions through a
native capability protocol.

A large external signed module ecosystem is not a good assumption for the early
product. The expected development loop is faster and more local:

```text
raiOS boots -> reports structured state -> agent diagnoses -> agent builds an
extension locally -> VM harness tests it -> raiOS can load or persist the exact
tested artifact under capability rules.
```

In this model, modules are not app-store packages. They are local, agent-built
artifacts with manifests, hashes, test results, and audit records.

## Decision
Build the next raiOS agent architecture around two core primitives:

1. Machine-readable self-description from the OS.
2. Local-first live-built extension artifacts controlled by capability gates and
   VM tests.

Do not require a global signing ecosystem for early modules. Instead, treat an
extension as acceptable only when the local system can answer:

- What does this artifact claim to provide?
- Which capabilities does it request?
- Which artifact hash was tested?
- Which VM or harness tests passed?
- Is it being loaded temporarily or persisted?
- Can it be rolled back?

Acceptance is based on local attestation, not agent self-assertion. The agent can
build and propose an artifact, but raiOS should only accept an artifact hash
that is tied to a manifest hash, base image hash, VM/harness report hash, and
local approval record. The same provider response that built an artifact must not
be sufficient to certify it as safe to load.

This keeps agent iteration fast while still avoiding an unrestricted shell or
arbitrary kernel patch path inside raiOS.

## Core Thesis
Ubuntu gives an agent broad existing power. raiOS should give an agent a
smaller but clearer model of the system.

The agent should not need to infer system state from a random pile of logs and
files. It should be able to ask for structured facts:

```text
system.describe
system.snapshot
system.capabilities
system.boot_log
device.graph
problem.list
```

The OS then answers with stable schemas that expose device state, boot state,
network state, loaded modules, known gaps, and available actions.

## Capability Model
Agent actions should be grouped by risk and effect:

```text
observe     Read facts, logs, inventory, status.
diagnose    Probe devices or run bounded checks.
simulate    Build or test in a VM/harness without changing the live system.
modify_ram  Load a temporary runtime artifact for this boot only.
persist     Change boot config, persistent module set, or stored policy.
hardware    Touch real hardware state with risk outside the VM.
```

raiOS should avoid a generic "run arbitrary command" capability. The host
builder may use normal development tools, but the OS boundary should remain a
small catalog of explicit capabilities with input schemas, output schemas, risk
levels, approval rules, and audit behavior.

Manifest capabilities are requests, not grants. raiOS computes effective grants
from local policy, the current boot state, user approval, and test results. A V0
grant should be shaped like this:

```json
{
  "id": "cap.input.emit_key",
  "subject": "module:usb-hid-keyboard",
  "resource": "input.keyboard",
  "scope": "current_boot",
  "grant_source": "local_approval",
  "duration": "ephemeral",
  "approval_required": true,
  "audit_level": "full"
}
```

`granted_caps` must never be trusted from the manifest itself. The manifest may
request capabilities; only local policy and acceptance evidence compute the
effective grants.

## Live-Built Module Meaning
In this architecture, "module" means any agent-built extension artifact. It can
be one of several levels:

| Kind | Runs where | Artifact format | Allowed first | Required gate |
| --- | --- | --- | --- | --- |
| workstation-side capability | dev workstation | script/binary plus manifest | yes | local policy + audit |
| guest diagnostic | raiOS sandbox | small guest artifact plus manifest | later | VM smoke + read-only caps |
| ram-only driver | raiOS kernel/runtime | low-level artifact plus manifest | later | matching VM report + approval |
| persistent driver | boot image/config | low-level artifact plus manifest | last | matching VM report + rollback |

The first live-built artifact type should be a workstation-side capability module. The
second should be a read-only guest diagnostic or helper. Low-level guest modules
need a separate ABI/isolation decision before implementation.

## Minimal Module Manifest
Every extension artifact should carry a manifest shaped like this:

```json
{
  "manifest_version": "raios.module.v0",
  "name": "usb-hid-keyboard",
  "kind": "driver",
  "target": "raios-stage0",
  "abi": "raios-driver-v0",
  "built_by": "agent-session-id",
  "provides": ["input.keyboard"],
  "requested_caps": ["usb.xhci.read_events", "input.emit_key"],
  "granted_caps": [],
  "risk": "hardware",
  "base_image_hash": "blake3:...",
  "manifest_hash": "blake3:...",
  "artifact_hash": "blake3:...",
  "test_report_hash": null,
  "tests": [
    "qemu_usb_keyboard_detected",
    "key_event_roundtrip"
  ],
  "load_mode": "ram_only",
  "rollback_id": null
}
```

For early development, the important fields are `provides`, `requested_caps`,
`granted_caps`, `risk`, `base_image_hash`, `manifest_hash`, `artifact_hash`,
`test_report_hash`, `tests`, `load_mode`, and `rollback_id`. Global publisher
identity can be added later, but it is not the root trust mechanism for the MVP.

## Protocol Shape
The protocol should carry development cycles, not only chat messages.

ADR 0002 defines the logical protocol independent of transport. V0 should reuse
the existing `device-protocol` JSON envelope and carry it through the direct
provider/tool boundary. The same logical envelope can later move to WebSocket or
another transport:

```json
{
  "v": "raios.agent.v0",
  "t": "request",
  "id": "req-001",
  "ts": 0,
  "body": {
    "method": "system.snapshot",
    "params": {}
  }
}
```

Responses and errors should keep the same `id`:

```json
{
  "v": "raios.agent.v0",
  "t": "error",
  "id": "req-001",
  "ts": 0,
  "body": {
    "code": "capability_denied",
    "message": "module.load_ephemeral requires matching vm_test_report",
    "required": ["vm_test_report", "local_approval"]
  }
}
```

Initial read-only messages:

```text
system.describe
system.snapshot
system.capabilities
system.boot_log
device.graph
problem.list
```

Initial build/test messages:

```text
module.propose
module.build_result
module.test_request
module.test_result
module.load_ephemeral
module.persist
module.rollback
```

For V0 these mutating methods may be implemented as explicit denials. A useful
first response is a structured `capability_denied` that names the missing
manifest hash, VM test report hash, local attestation hash, approval record, or
capability grant. This lets the agent learn the workflow without receiving
write access too early.

The first stable V0 capabilities should be:

```text
cap.system.snapshot.read
cap.system.boot_log.read
cap.system.capabilities.read
cap.device.graph.read
cap.problem.list.read
cap.module.propose
cap.vm_test.report.read
```

No V0 live-loading capability should be granted until the VM test report and
local attestation schemas exist.

## Snapshot V0
The smallest useful `system.snapshot.v0` should expose current facts only:

```json
{
  "schema": "system.snapshot.v0",
  "os": {
    "name": "raiOS",
    "stage": "stage-0",
    "kernel_build_id": "...",
    "image_hash": "blake3:..."
  },
  "status": {
    "framebuffer": "ready",
    "entropy": "ready",
    "rng": "rdrand",
    "network": "configured",
    "input": "ready",
    "usb_xhci": "keyboard_mouse_ready"
  },
  "network": {
    "ip": "10.0.2.15",
    "gateway": "10.0.2.2",
    "dns": ["10.0.2.3"]
  },
  "provider": {
    "selected": "openai",
    "api_key_state": "set"
  },
  "problems": [
    {
      "id": "usb_hid.missing",
      "severity": "info",
      "summary": "xHCI inventory exists, USB HID keyboard and mouse are ready"
    }
  ],
  "capabilities": [
    "cap.system.snapshot.read",
    "cap.system.boot_log.read",
    "cap.module.propose"
  ]
}
```

Fields that leave the machine through a provider adapter need classification:
`public`, `local_only`, or `secret`. The provider boundary must redact
`local_only` and `secret` fields unless local policy explicitly allows them.
Without classification, a field may be displayed locally but must not be
automatically attached to provider requests.

Provider context injection is gated by transport trust. The current direct
provider adapter must fail closed with certificate verification or provider/SPKI
pinning before `system.snapshot.v0` is sent outside the machine. Until then,
provider requests may remain plain prompts.

Example flow:

```text
Agent -> raiOS: system.snapshot
raiOS -> Agent: xHCI ready, USB HID keyboard and mouse ready

Agent -> raiOS: module.propose richer-usb-hid
raiOS -> Agent: allowed for VM-test only, needs usb.xhci.read_events

Agent/Host: builds artifact
Harness: boots test VM with artifact
Harness -> raiOS/Agent: module.test_result passed, artifact_hash blake3:...

Agent -> raiOS: module.load_ephemeral blake3:...
raiOS -> Agent: loaded for current boot, input.keyboard ready
```

## VM Harness Role
The VM harness is a first-class part of the safety model. It should be able to:

- Boot the current image plus a candidate artifact.
- Inject expected virtual hardware.
- Run smoke tests and protocol tests.
- Capture serial logs, framebuffer status, and protocol output.
- Report a deterministic result back to the agent loop.

The harness does not make live loading risk-free, but it gives raiOS a fast
local check before accepting an artifact into the running boot or persistent
image.

A V0 test report should be machine-readable:

```json
{
  "schema": "raios.vm_test_report.v0",
  "report_hash": "blake3:...",
  "base_image_hash": "blake3:...",
  "candidate_artifact_hash": "blake3:...",
  "candidate_manifest_hash": "blake3:...",
  "qemu_version": "...",
  "qemu_args_hash": "blake3:...",
  "hardware_profile": "qemu-e1000-usb-xhci-v0",
  "commands": ["status", "devices", "ask protocol-smoke"],
  "predicates": [
    "serial_contains:status INPUT: READY",
    "protocol_response:system.snapshot"
  ],
  "result": "passed",
  "expires_at_boot_id": null
}
```

For bare-metal issues, the report should also include snapshot preconditions.
raiOS should refuse a report if the current image hash, artifact hash, hardware
profile, or required preconditions do not match.

## Audit And Reproducibility
raiOS should record enough information to replay or understand a change:

```text
agent session id
request id
requested capability
manifest
artifact hash
test result ids
approval decision
load mode
rollback pointer
```

This is the local replacement for a public signing ecosystem during early
development. The system trusts the exact tested artifact under local policy, not
a broad package marketplace.

The local attestation record should bind together:

```text
manifest hash
artifact hash
base image hash
VM/harness report hash
approval source
approval time
load mode
```

For persistence, the record should also include the previous-good boot pointer,
pending artifact set, boot-success marker, rollback trigger, and safe-mode rule
that disables modules and persistent writes.

## Non-Goals
This decision does not mean:

- Porting Codex CLI into the kernel.
- Building a general Linux-compatible package manager.
- Accepting arbitrary kernel code without a manifest and tests.
- Requiring an external module store or global publisher signing system.
- Blocking bare-metal driver work on the full module runtime.

## Suggested Implementation Phases

### Phase A: Protocol Documentation
Define the schemas and examples first:

```text
device-protocol/agent-v0.md
device-protocol/module-manifest-v0.md
device-protocol/vm-test-harness-v0.md
```

This phase should pin the V0 JSON envelope, request/response/error shape,
capability denial semantics, `system.snapshot.v0`, `module_manifest.v0`, and
`vm_test_report.v0`.

### Phase B: Read-Only Self-Description
Expose current Stage-0 facts through the native provider/tool context:

```text
framebuffer state
entropy state
RDRAND entropy state
e1000/DHCP state
input state
USB/xHCI inventory
provider/setup state
recent boot log
known problems
available capabilities
```

### Phase C: Agent Context Injection
Make the direct provider adapter attach a compact `system.snapshot` to provider
requests so the agent answers with current raiOS context instead of blind chat.

### Phase D: Proposal And Test Loop
Add protocol support for module proposals and VM test results before implementing
dynamic guest loading.

The first implementation target is a workstation-side capability module, not a kernel
driver. `module.propose` produces a manifest/proposal only. Loading remains
denied until a matching test report and local attestation record exist.

### Phase E: Ephemeral Loading
Allow selected low-risk artifacts to load for the current boot only after a
matching VM test result.

### Phase F: Persistence And Rollback
Add explicit approval, image/config persistence, boot slot tracking, and
rollback.

## Open Questions
- How does a bare-metal run report enough facts for the VM harness to reproduce
  the failure?
- Which actions should always require direct user confirmation?
- What isolation format should guest diagnostics use: native Rust ABI, Wasm, or
  another small sandbox?
- What is the eventual low-level module ABI for memory, MMIO, interrupts, panic
  containment, unload, and kill switch behavior?
