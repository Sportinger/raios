# Provider Trust V0

Status: fail-closed gate implemented; positive `pinned_spki_verified` and
legacy `pinned_cert_verified` vertical slices implemented for OpenAI.

raiOS Stage-0 may talk to an AI provider only after the provider peer identity
is verified. Until then, provider requests may carry direct user prompts, but
they must not carry automatic system snapshots, tool schemas, persistence
requests, recovery actions, or module-loading evidence.

## Trust States

Provider trust is represented as an explicit state, not inferred from "TLS
connected" logs.

```json
{
  "schema": "raios.provider_trust.v0",
  "provider": "openai",
  "endpoint": "api.openai.com:443",
  "transport": "tls13",
  "trust_state": "pin_config_missing",
  "pin_kind": null,
  "pin_id": null,
  "verified_at_ms": null,
  "capabilities_enabled": []
}
```

Allowed `trust_state` values:

| State | Meaning | Provider context allowed |
| --- | --- | --- |
| `unknown` | A syntactically valid pin exists, but no TLS attempt has completed in this boot. | no |
| `tls_certificate_verification_bypassed` | TLS was established with `NoVerify`. | no |
| `pin_config_missing` | A fail-closed verifier is active but has no configured pin. | no |
| `pin_config_invalid` | A configured provider pin is not a 64-character SHA-256 hex string. | no |
| `pin_verifier_unavailable` | A syntactically valid pin exists, but the verifier cannot validate the presented certificate/signature shape. | no |
| `pin_mismatch` | A fail-closed verifier rejected the peer certificate, host, pin, or TLS signature proof. | no |
| `pinned_spki_verified` | The peer leaf SPKI SHA-256 matched the configured provider pin. | yes, after redaction |
| `pinned_cert_verified` | The peer leaf certificate SHA-256 matched the configured provider pin and its TLS 1.3 `CertificateVerify` signature was valid. | yes, after redaction |
| `webpki_verified` | A configured trust anchor, time source, and hostname check passed. | yes, after redaction |

Fail-closed states must return a visible provider error and must not silently
fall back to `NoVerify`.

## Current Implementation

The Stage-0 kernel models OpenAI provider trust explicitly through
`seed-kernel/src/provider_trust.rs` and reports it in `system.snapshot.v0`.
The normal build fails closed before copying the API key or writing an HTTPS
request body unless trust is either already verified or a syntactically valid
OpenAI SPKI or leaf-certificate pin is configured for the handshake attempt.
When both pins are configured, the SPKI pin is the active provider pin.

Current states:

- no configured pin: `pin_config_missing`
- invalid configured pin: `pin_config_invalid`
- valid configured pin before handshake completion: `unknown`
- valid configured SPKI pin matching the OpenAI leaf SubjectPublicKeyInfo
  SHA-256 plus TLS 1.3 `CertificateVerify` signature proof:
  `pinned_spki_verified`
- valid configured leaf-certificate pin matching the OpenAI leaf certificate
  SHA-256 plus TLS 1.3 `CertificateVerify` signature proof:
  `pinned_cert_verified`
- wrong pin, wrong host, or failed certificate/signature proof: `pin_mismatch`
- unsupported verifier input such as a non-P-256 leaf public key:
  `pin_verifier_unavailable`
- explicit local development bypass:
  `tls_certificate_verification_bypassed`

The unverified path is available only through the named build/package switch
`-AllowUnverifiedOpenAiTls`, which sets
`RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS=1` for that kernel build. It must not be used
for normal provider or control-plane work.

## Implemented Verifier Slices

`seed-kernel/src/openai_trust.rs` implements the normal-path pin verifier:

- endpoint host is fixed to `api.openai.com`
- the preferred configured pin is `RAIOS_OPENAI_SPKI_SHA256`
- the compatibility configured pin is `RAIOS_OPENAI_CERT_SHA256`
- the SPKI pin is SHA-256 of the full DER SubjectPublicKeyInfo TLV
- the legacy certificate pin is SHA-256 of the full DER leaf certificate
- the verifier extracts the leaf P-256 public key from SubjectPublicKeyInfo
- TLS 1.3 `CertificateVerify` is checked with ECDSA P-256/SHA-256 over the
  embedded-tls handshake transcript
- the API key is copied only after the trust state becomes
  `pinned_spki_verified` or `pinned_cert_verified`

The repository carries a narrow local patch of `embedded-tls` 0.17.0 under
`vendor/embedded-tls-0.17.0` so downstream verifier code can read the leaf
certificate DER and certificate-verify signature bytes. This is deliberately a
small verifier-input patch, not a forked TLS policy layer.

The positive SPKI VM smoke requires this marker:

```text
openai: TLS provider trust verified: pinned_spki sha256:<pin-id>
```

The legacy leaf-certificate VM smoke requires this marker:

```text
openai: TLS provider trust verified: pinned_cert sha256:<pin-id>
```

The `NoVerify` path remains only behind the explicit development build flag
`-AllowUnverifiedOpenAiTls`.

## Remaining Work

- Decide whether Stage-0 should keep the vendored verifier-input patch, move it
  upstream, or upgrade to a TLS crate/version with the required public verifier
  inputs.
- Add broader certificate algorithm support or make unsupported algorithms a
  permanent explicit denial.
- Add a WebPKI path only after trust anchors, time, hostname verification, and
  chain/intermediate handling are specified and tested.

## Acceptance Criteria

- A successful HTTPS provider request logs both `openai: TLS 1.3 established`
  and a separate provider trust marker.
- The smoke harness fails if the trust marker is absent.
- With no pin configured, the provider request fails closed before sending the
  API key or HTTP body.
- With a wrong pin, the provider request fails closed before sending the API key
  or HTTP body.
- `trust_state` is visible in serial/provider snapshot output.
- Automatic system context remains disabled unless trust is verified and the
  context redaction policy allows each field.
- `provider.context_export provider_minimal` remains denied until positive
  trust is paired with a real provider request binding and a distinct positive
  export audit binding. The projected packet and field-list hashes, plus
  denial-audit records, may be present before those gates pass.
- Positive pinned/WebPKI request paths emit
  `raios.provider_request_binding.v0` and
  `raios.provider_context_export_audit_binding.v0` local-only markers after
  request body/envelope hash validation and before API-key copy or HTTPS write.
- Development TLS bypass and pin mismatch do not emit positive request/export
  audit binding markers.
