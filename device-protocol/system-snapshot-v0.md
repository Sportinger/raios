# raiOS System Snapshot V0

`system.snapshot.v0` is the first typed, read-only machine snapshot exposed by
`raios.agent.v0`. It reports current-boot facts only. It is not an authority for
mutating decisions, persistence, provider trust, or module loading.

The current Stage-0 kernel emits this schema from the serial-accessible
`system.snapshot` method. The output is intentionally compact and mirrors the
existing framebuffer/console status model until richer per-service state exists.

Provider context injection is disabled until both gates are true:

- the provider transport is positively verified by certificate verification or
  provider/SPKI pinning; a fail-closed missing-pin state is not enough for
  provider context injection
- every outbound snapshot field is classified and redacted by this document or a
  later compatible policy

## Current Shape

The current kernel output is a JSON response body result with this shape:

```json
{
  "schema": "system.snapshot.v0",
  "os": {
    "name": "raiOS",
    "product": "raiOS",
    "stage": "stage-0"
  },
  "status": {
    "framebuffer": "ready",
    "entropy": "ready",
    "usb_xhci": "ready",
    "wifi": "detected",
    "network": "configured",
    "input": "ready"
  },
  "details": {
    "framebuffer": {"state": "ready", "detail": "1280x800 PITCH 5120"},
    "entropy": {"state": "ready", "detail": "FILL 64/64 TOTAL 64 SRC RDRAND"},
    "usb_xhci": {"state": "ready", "detail": "..."},
    "wifi": {"state": "detected", "detail": "..."},
    "network": {"state": "configured", "detail": "IP 10.0.2.15/24 GW 10.0.2.2"},
    "input": {"state": "ready", "detail": "USB HID KEYBOARD + POINTER"}
  },
  "provider": {
    "selected": "OPENAI",
    "route": "OPENAI DIRECT",
    "api_key_state": "set",
    "direct_phase": "idle",
    "direct_endpoint": "https://api.openai.com/v1/responses",
    "direct_model": "gpt-5.4",
    "trust_state": "pin_config_missing",
    "pin_kind": null,
    "pin_id": null,
    "development_bypass": false
  },
  "capabilities": [
    "cap.system.describe.read",
    "cap.system.snapshot.read"
  ],
  "problems": [
    {
      "id": "provider.tls_pin_config_missing",
      "severity": "high",
      "summary": "OpenAI direct transport is fail-closed until a provider pin is configured"
    }
  ]
}
```

Field additions should prefer explicit state fields over prose-only details.
Until then, `details.*.detail` remains a local diagnostic string and must be
treated conservatively at the provider boundary.

## Classification Levels

Every snapshot field has one of these classifications:

```text
public      safe to include in provider context after the TLS/provider trust gate
local_only  useful to local agents or logs, but omitted from provider context by default
secret      never disclose the value; expose only a boolean/state marker if needed
```

Classification is schema-level and applies even when a particular runtime value
looks harmless. Unknown or newly added fields are `local_only` until classified
here or in a later schema revision.

## Field Classification

| Field path | Class | Provider-redacted behavior |
| --- | --- | --- |
| `schema` | public | keep |
| `os.name` | public | keep |
| `os.product` | public | keep |
| `os.stage` | public | keep |
| `os.kernel_build_id` | local_only | omit unless local policy allows build provenance |
| `os.image_hash` | local_only | omit unless bound to a VM test report or attestation |
| `status.*` | public | keep current coarse state values |
| `details.framebuffer.state` | public | keep |
| `details.framebuffer.detail` | local_only | omit or replace with coarse display class |
| `details.entropy.state` | public | keep |
| `details.entropy.detail` | local_only | omit or replace with source class such as `rdrand` |
| `details.usb_xhci.state` | public | keep |
| `details.usb_xhci.detail` | local_only | omit; may expose coarse keyboard/pointer readiness later |
| `details.wifi.state` | public | keep |
| `details.wifi.detail` | local_only | omit; may contain PCI ids, BARs, SSID state, or setup state |
| `details.network.state` | public | keep |
| `details.network.detail` | local_only | omit; may contain local IP, gateway, DNS, or counters |
| `details.input.state` | public | keep |
| `details.input.detail` | local_only | omit; may reveal attached input topology |
| `network.ip` | local_only | omit by default |
| `network.gateway` | local_only | omit by default |
| `network.dns` | local_only | omit by default |
| `provider.selected` | public | keep provider family only |
| `provider.route` | public | keep if it does not include credentials or account ids |
| `provider.api_key_state` | public | keep only `set`, `missing`, or `unknown`; never the key |
| `provider.direct_phase` | public | keep coarse phase |
| `provider.direct_endpoint` | public | keep canonical provider endpoint only |
| `provider.direct_model` | public | keep model id |
| `provider.trust_state` | public | keep; this is required for fail-closed context policy |
| `provider.pin_kind` | public | keep only stable pin type, not raw pin |
| `provider.pin_id` | public | keep only a short non-secret pin identifier |
| `provider.development_bypass` | public | keep; this must block provider context injection |
| `provider.direct_last_prompt` | secret | never include in snapshot provider context |
| `provider.direct_last_error` | local_only | omit unless reduced to a stable problem id |
| `provider.direct_last_event` | local_only | omit unless reduced to a stable phase |
| `provider.direct_pending_id` | local_only | omit |
| `provider.direct_last_request_id` | local_only | omit |
| `provider.tcp.*` | local_only | omit by default |
| `wifi.ssid` | secret | never include raw SSID in provider context |
| `wifi.passphrase` | secret | never include |
| `wifi.passphrase_state` | public | keep only `set`, `missing`, or `unknown` |
| `capabilities[]` | public | keep capability ids |
| `problems[].id` | public | keep stable problem ids |
| `problems[].severity` | public | keep |
| `problems[].summary` | public | keep only summaries that do not include local addresses, prompts, keys, hashes, or raw logs |

## Redaction Profiles

`local_full` is the profile used for serial console and framebuffer-local
inspection. It may include `public`, `local_only`, and secret state markers, but
must never print secret values. The current `system.snapshot` serial method is a
local profile.

`provider_minimal` is the only allowed profile for automatic provider context
after the provider trust gate. It includes:

- `schema`
- `os.name`, `os.product`, `os.stage`
- `status.*`
- `provider.selected`, `provider.route`, `provider.api_key_state`,
  `provider.direct_phase`, `provider.direct_endpoint`, `provider.direct_model`,
  `provider.trust_state`, `provider.pin_kind`, `provider.pin_id`,
  `provider.development_bypass`
- `capabilities[]`
- `problems[]` after summary scrubbing

`provider_diagnostic` is a future local-policy profile. It may include selected
`local_only` fields only after explicit local approval, a visible audit record,
and a reason scoped to one provider request. It still cannot include `secret`
values.

## Redaction Rules

1. Secret values must never be emitted into provider context, boot logs, problem
   summaries, service inventory, or denial messages. Only state markers such as
   `set` or `missing` are allowed.
2. `local_only` fields are omitted from provider context by default. If a future
   policy allows one, it must name the field path and request scope explicitly.
3. Free-form strings crossing the provider boundary must be scrubbed for API
   keys, WPA passphrases, raw prompts, local IP/gateway/DNS addresses, PCI BAR
   addresses, request ids, and hashes unless the field is explicitly classified
   as public.
4. Problem summaries intended for provider context must be stable descriptions,
   not copied log lines.
5. If transport trust is `tls_certificate_verification_bypassed`, unknown,
   fail-closed, or failed, provider context injection must send no snapshot
   fields. Only `pinned_cert_verified`, `pinned_spki_verified`, or
   `webpki_verified` may unlock the future provider projection.
6. A schema consumer must fail closed: any unclassified field is treated as
   `local_only`, and any unrecognized secret-looking field is treated as
   `secret`.

## Current Gaps

- The serial `system.snapshot` method emits the local profile only; it does not
  yet emit a separate `provider_minimal` projection.
- Network, Wi-Fi, USB, and provider detail fields are still prose strings in the
  current kernel. They are useful locally but intentionally classified as
  `local_only` until typed subfields exist.
- The normal direct OpenAI path now has positive `pinned_spki_verified` and
  legacy `pinned_cert_verified` slices, but automatic snapshot attachment to
  provider requests remains disabled until a `provider_minimal` projection is
  emitted and the provider request path explicitly applies this redaction
  profile.
