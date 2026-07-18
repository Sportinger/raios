# ADR 0003: Always-On Core And Live-Rebuildable World

## Status
Draft for the long-term raiOS architecture.

## Context
The Stage-0 MVP currently proves that raiOS can boot directly into a small
agent host with framebuffer UI, input, e1000 networking, DHCP, DNS, TLS/HTTPS,
and direct OpenAI API calls. That is useful, but it is not the final shape.

The long-term goal is not simply a tiny OS that can talk to an AI provider. The
goal is a living OS whose main design center is live AI-driven extension:

```text
boot -> describe itself -> accept bounded agent actions -> load new behavior
-> test it -> switch to it -> rollback if it fails
```

The user-visible target is that most system evolution happens without a visible
reboot. The VM remains important, but as a shadow safety pass and regression
environment, not as the only place where changes can be tried.

## Decision
raiOS should evolve toward a permanent minimal core plus a live-rebuildable
world above it.

The permanent core is not the whole OS. It is the survival layer:

```text
CPU and interrupt minimum
memory and object ownership minimum
scheduler minimum
IPC/message bus
capability table
module/service loader
crash detector
rollback supervisor
system snapshot root
minimal local control console
minimal agent control link
```

Everything else should become a service or module that can be stopped, replaced,
or rolled back:

```text
framebuffer shell
console shell
input stack
USB service
network stack
Wi-Fi service
provider/OpenAI adapter
filesystem service
diagnostics
agent tools
driver experiments
builder service
shadow VM service
```

The final model is:

```text
always-on core
  -> protected recovery control plane
  -> live service graph
  -> agent workspace
  -> shadow VM/test world
  -> persistence/rollback layer
```

## Stage-0 Ramp
The current codebase is still a statically linked Stage-0 kernel. That is fine
for the ramp. The first step toward this decision is not dynamic loading; it is
to describe the current static components as if they were services:

```text
core.boot
core.memory
core.serial
core.scheduler
core.snapshot_root
svc.ui.framebuffer
svc.console
svc.input
drv.usb.xhci
drv.net.e1000
svc.net.ipv4
drv.wifi.avastar_probe
svc.provider.openai_direct
```

Each entry should have a stable service id, kind, health, last error,
capabilities, `replaceable`, and `core_owned`. This static inventory is the
bridge between today's monolith and tomorrow's live service graph.

## Core And AI Control
The core should not depend on the normal UI, normal network stack, or normal
provider adapter to recover the system. If those services crash, the core must
still be able to:

```text
emit a system snapshot
report crashed services
disable bad modules
restart last-good services
load a recovery artifact
rollback persistent state
accept a small set of control messages
```

For the end state, remote AI recovery requires a protected agent lifeline. This
lifeline is part of the trusted recovery base, but should be smaller and more
static than the normal provider service.

The architecture therefore has two AI paths:

| Path | Purpose | Replaceable | Contains HTTP/TLS/API provider logic |
| --- | --- | --- | --- |
| Normal agent service | Rich daily AI interaction and tool use | yes | yes |
| Recovery agent lifeline | Last-resort control when the live world is broken | only by core-generation update | minimal, pinned, conservative |

The normal agent service can use the full network/provider stack and can be
hot-swapped like any other service. The recovery lifeline exists so the system
is not blind if the normal service graph is down. It may use a very small pinned
provider route, a local physical link, or both. It should only carry recovery
messages, not general chat or arbitrary tool execution.

The core itself should still avoid becoming a large HTTP application. The
lifeline is a protected recovery component with a tiny protocol surface, guarded
by the core and restartable from a known-good image.

The current direct OpenAI implementation is a normal agent-service candidate,
not the recovery lifeline. It depends on the ordinary network/provider path and
currently performs synchronous HTTPS work. Until a separate minimal recovery
protocol and trust state exist, it must not be treated as the trusted recovery
base.

## Live Rebuild Model
Live evolution depends on separating code from state.

Services must not expose arbitrary global pointers. They expose handles,
capabilities, and versioned state objects:

```text
service net.v1 owns state object net.state.v1
service net.v2 starts next to net.v1
migrator converts net.state.v1 -> net.state.v2
core switches network handles to net.v2
net.v1 remains available for rollback until stable
```

The same pattern applies to UI, input, provider adapters, diagnostics, and
eventually drivers.

The core needs these primitives:

```text
load_service_ephemeral
start_service
pause_service
snapshot_service_state
migrate_service_state
switch_handle
rollback_handle
kill_service
persist_service_set
```

V0 live loading is not allowed merely because these primitives are named. Before
`load_service_ephemeral` can succeed, raiOS needs at least a service id,
manifest, computed capability grants, VM test report, local attestation record,
health check, and audit record. Missing evidence should produce a structured
denial rather than a partial load.

## Agent Workspace
raiOS should eventually contain its own agent workspace rather than only being
edited from an external development machine. The workspace is a structured
system object, not a POSIX-like shell requirement:

```text
source/proposal DAG
module manifests
capability requests
artifact hashes
test reports
state migrators
audit records
rollback pointers
```

A builder service may compile, assemble, or receive generated artifacts. It is
not part of the permanent core. If the builder crashes, the core can restart or
replace it. If the builder produces a bad artifact, the supervisor can drop it.

## Shadow VM Role
The shadow VM remains a core part of the safety model, but not a prerequisite
for every live experiment.

Use levels:

```text
read-only diagnostic module: may load live with low-risk caps
UI/tool module: may load live with rollback
network/provider service: live-load plus shadow test preferred
driver touching MMIO/interrupts: shadow VM required before live load
persistent boot service: shadow VM plus approval plus rollback slot required
core-generation update: shadow VM plus handoff test plus recovery fallback
```

On hardware that supports virtualization, the shadow world can run locally. On
small machines it can run externally during development. The protocol should not
care where it runs; reports are bound to image hash, artifact hash, hardware
profile, and snapshot preconditions.

## Toward No Visible Reboot
"No reboot" means no visible full restart of the user's system. Internally,
raiOS may still pause, quiesce, restart services, or perform a core-generation
handoff.

Most updates should use service hot-swap:

```text
load v2 next to v1
copy or migrate state
atomically switch handles
watch health
keep v1 as rollback
garbage collect old version later
```

Deep core updates require generation handoff:

```text
load core generation N+1
freeze non-core services
snapshot root objects
migrate core metadata
switch root dispatch tables
resume services
retain generation N for rollback until stable
```

Blocking service work is a design risk for this model. If a provider request,
TLS handshake, driver poll, or filesystem write can monopolize the cooperative
loop, it is not recovery-safe. Such paths should be moved behind service health
state, timeouts, and eventually preemptible or restartable service boundaries
before being used for recovery or persistence.

Some hardware and firmware boundaries may still require a real reset, such as
changing the CPU boot mode, firmware state, or early memory map assumptions. The
design goal is to make that rare by keeping the permanent core extremely small.

## Consequences
This changes the long-term roadmap:

- The provider path is a service, not the definition of the OS.
- The module system is not an app store; it is the live service mechanism.
- The recovery agent lifeline becomes part of the trusted base.
- Services need versioned state and migrators from the beginning.
- VM tests are safety evidence, not the only development surface.
- The capability model must cover live handles, state migration, rollback, and
  persistence.

The hard rule is:

```text
If a component must be replaceable by the AI, it cannot be a hidden dependency
of the survival core.
```

## Implementation Direction
The next protocol and code work should be shaped by this end state even when the
first implementation is smaller:

1. Define the core/world boundary in docs and code ownership.
2. Expose `system.snapshot.v0` and service inventory as machine-readable data.
3. Add a service registry even before true dynamic loading exists.
4. Add ephemeral module/service slots with explicit capabilities.
5. Add health checks, crash records, and last-good service pointers.
6. Add versioned state objects and a first trivial state migrator.
7. Add a protected recovery control path separate from the rich provider path.
8. Add shadow VM reports as acceptance evidence for risky service changes.
9. Add persistence and rollback slots.
10. Only then attempt core-generation handoff.

## Non-Goals
This decision does not mean:

- Putting a large OpenAI client, browser, shell, or package manager in the core.
- Letting AI write arbitrary kernel memory.
- Removing tests because live loading exists.
- Requiring a public signed module ecosystem.
- Pretending every firmware or hardware transition can be made live.
