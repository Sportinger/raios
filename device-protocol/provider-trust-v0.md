# Provider Trust V0

Status: fail-closed gate implemented; positive pin verification is still blocked
on TLS verifier input access.

SeedOS Stage-0 may talk to an AI provider only after the provider peer identity
is verified. Until then, provider requests may carry direct user prompts, but
they must not carry automatic system snapshots, tool schemas, persistence
requests, recovery actions, or module-loading evidence.

## Trust States

Provider trust is represented as an explicit state, not inferred from "TLS
connected" logs.

```json
{
  "schema": "seedos.provider_trust.v0",
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
| `unknown` | No TLS attempt has completed in this boot. | no |
| `tls_certificate_verification_bypassed` | TLS was established with `NoVerify`. | no |
| `pin_config_missing` | A fail-closed verifier is active but has no configured pin. | no |
| `pin_config_invalid` | A configured provider pin is not a 64-character SHA-256 hex string. | no |
| `pin_verifier_unavailable` | A syntactically valid pin exists, but the active TLS crate does not expose enough verifier input to check it. | no |
| `pin_mismatch` | A fail-closed verifier rejected the peer certificate or SPKI. | no |
| `pinned_spki_verified` | The peer leaf SPKI SHA-256 matched the configured provider pin. | yes, after redaction |
| `pinned_cert_verified` | The peer leaf certificate SHA-256 matched the configured provider pin. | yes, after redaction |
| `webpki_verified` | A configured trust anchor, time source, and hostname check passed. | yes, after redaction |

Fail-closed states must return a visible provider error and must not silently
fall back to `NoVerify`.

## Current Implementation

The Stage-0 kernel now models OpenAI provider trust explicitly through
`seed-kernel/src/provider_trust.rs` and reports it in `system.snapshot.v0`.
The normal build fails closed before copying the API key or writing an HTTPS
request body when trust is not verified.

Current states:

- no configured pin: `pin_config_missing`
- invalid configured pin: `pin_config_invalid`
- valid configured pin but no verifier access yet: `pin_verifier_unavailable`
- explicit local development bypass:
  `tls_certificate_verification_bypassed`

The unverified path is available only through the named build/package switch
`-AllowUnverifiedOpenAiTls`, which sets
`SEEDOS_ALLOW_UNVERIFIED_OPENAI_TLS=1` for that kernel build. It must not be used
for normal provider or control-plane work.

## Remaining Blocker

The repository still pins `embedded-tls = "=0.17.0"` with `alloc` only.
Inspection of the local Cargo registry source shows:

- `embedded_tls::blocking::NoVerify` is the active verifier in
  `seed-kernel/src/openai.rs`.
- `embedded_tls::blocking::TlsVerifier` exists and receives the handshake
  transcript, optional CA, and `CertificateRef`.
- `CertificateRef` stores the parsed certificate entries behind `pub(crate)`
  fields, and the `handshake` module is private.
- A downstream verifier can implement the trait, but cannot inspect the leaf DER
  certificate or SPKI bytes needed for SHA-256 pinning.
- The built-in `embedded_tls::webpki::CertVerifier` path is not a small provider
  pin. It requires enabling the `webpki` feature, supplying trust anchors, a
  clock implementation, and accepting its current chain limitations. The 0.17.0
  verifier source also documents `TODO: Support intermediates...`.

Because the certificate material is not exposed to downstream code, a positive
SPKI/certificate pinner is not implementable as a narrow kernel-only patch
against `embedded-tls` 0.17.0. The current kernel therefore denies the normal
provider path instead of silently falling back to `NoVerify`.

## Minimal Code Plan

1. Add a verifier input API to the TLS dependency:
   - preferred: upstream or locally patch `embedded-tls` so `TlsVerifier` can
     iterate certificate entries as public borrowed DER slices;
   - acceptable local ramp: expose a minimal `CertificateRef::leaf_der() ->
     Option<&[u8]>` without exposing mutable internals.
2. Add a tiny no-alloc DER helper in `seed-kernel/src/tls_io.rs` or a narrow new
   trust module to extract SubjectPublicKeyInfo from the leaf certificate.
3. Compute SHA-256 over either:
   - the leaf SPKI DER for `pinned_spki_verified`; or
   - the full leaf certificate DER for `pinned_cert_verified` as the first
     simpler vertical slice.
4. Replace the explicit development-only `openai.rs` `NoVerify` path with
   `OpenAiPinnedVerifier` in the normal provider path.
5. Keep missing pin, parse failure, mismatch, and TLS verifier failure as
   explicit provider errors before API-key copy or HTTPS write.
6. Keep `provider::Snapshot` and `system.snapshot.v0` exposing trust state and
   pin kind.
7. Update `vm-harness/openai-direct-smoke.ps1` to require a positive marker such
   as:

```text
openai: TLS provider trust verified: pinned_spki sha256:<pin-id>
```

or, for the first cert-pin slice:

```text
openai: TLS provider trust verified: pinned_cert sha256:<pin-id>
```

8. Keep the current `NoVerify` path only behind an explicitly named development
   build flag or remove it from the normal provider path.

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
