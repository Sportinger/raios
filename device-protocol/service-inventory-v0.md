# raiOS Service Inventory V0

`service.inventory.v0` is the static service graph for the current monolithic
Stage-0 kernel. It describes permanent-core responsibilities and replaceable
world components before raiOS has dynamic service loading.

This inventory is descriptive evidence, not load permission. Mutating service
lifecycle methods remain denied until a module manifest, computed capability
grant, VM test report, local attestation, audit record, and rollback plan exist.

## Current Transport

The serial agent method is:

```text
service.inventory
```

It is returned inside the `raios.agent.v0` response envelope:

```json
{
  "schema": "service.inventory.v0",
  "services": []
}
```

## Service Entry

Current emitted fields:

```json
{
  "id": "svc.provider.openai_direct",
  "kind": "service",
  "health": "degraded",
  "replaceable": true,
  "core_owned": false,
  "last_error": "TLS provider pin is missing",
  "capabilities": ["cap.system.snapshot.read"]
}
```

`trust_source` is specified below as the intended evidence field, but the current
kernel model and serial emitter do not expose it yet. Consumers must treat trust
source as unavailable on the wire unless a later response includes it.

## Field Semantics

`id` is a stable service identifier. V0 ids are namespaced by ownership:

- `core.*` is permanent-core functionality.
- `svc.*` is a replaceable service candidate.
- `drv.*` is a replaceable driver candidate.

`kind` is one of `core`, `service`, or `driver`.

`health` is a compact state derived from the same typed runtime facts used by
`system.snapshot.v0`:

- `healthy`: required evidence is present and no current error is known.
- `starting`: the component is probing, waiting, or processing a request.
- `degraded`: the component exists but has a current trust, runtime, or protocol
  problem.
- `missing`: required runtime evidence is absent or required local
  configuration is missing.

`last_error` is either `null` or a concise, current reason for `degraded` or
`missing`. It must not contain secrets. For provider services it may report a
trust gap even when network and API-key configuration exist; today
`svc.provider.openai_direct` remains `degraded` while the default trust state is
`pin_config_missing`.

`capabilities` lists capability names associated with the service's observable
surface. These are not self-granted authority for future modules. Local policy
must compute effective grants.

`replaceable` indicates that the final architecture expects the component to be
loadable or hot-swappable outside the survival core. In Stage-0 this is only a
static boundary marker.

`core_owned` indicates that the permanent core owns the responsibility and it
cannot be replaced by normal service loading. A service must not be both
`replaceable: true` and `core_owned: true`.

`trust_source`, when exposed in a future envelope, names the evidence source used
to compute the inventory or health row. Expected values include
`static_kernel_core`, `runtime_network_snapshot`, and
`runtime_provider_snapshot_tls_state`.

## Required V0 Service IDs

The initial static inventory must include these stable ids:

```text
core.boot
core.memory
core.serial
core.scheduler
core.entropy
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

## Current Health Sources

The current kernel maps service health as follows:

- `core.boot`, `core.memory`, `core.serial`, `core.scheduler`,
  `core.snapshot_root`, and `svc.console` are static Stage-0 entries and report
  `healthy` while the kernel can emit the inventory.
- `svc.ui.framebuffer` follows the Limine framebuffer status.
- `core.entropy` follows the entropy pool status.
- `svc.input` follows the consolidated input status.
- `drv.usb.xhci` follows the USB-xHCI status.
- `drv.net.e1000` and `svc.net.ipv4` follow the current e1000/IPv4 network
  status.
- `drv.wifi.avastar_probe` follows the current Marvell AVASTAR probe status.
- `svc.provider.openai_direct` is `missing` when the RAM-only API key is absent,
  `starting` while a direct request is pending, `degraded` on request errors,
  `degraded` when provider trust is missing, invalid, unavailable, mismatched,
  or explicitly bypassed for development, and `healthy` only after
  `pinned_cert_verified`, `pinned_spki_verified`, or `webpki_verified`.

## Invariants

- Inventory data must be derived from typed status facts, not scraped from human
  log text.
- Provider keys, prompts, and other secret material must not appear in
  `last_error`, `capabilities`, or `trust_source`.
- `core_owned` entries may expose read-only observation capabilities, but normal
  service lifecycle operations must not be allowed to replace them.
- Mutating lifecycle methods such as `service.start`, `service.stop`,
  `service.restart`, and `service.load_ephemeral` must continue to return
  `capability_denied` until the evidence chain exists.
