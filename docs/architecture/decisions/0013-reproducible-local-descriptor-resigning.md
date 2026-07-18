# ADR 0013: Reproducible Local Descriptor Re-signing

Date: 2026-07-10 · Status: active

## Status

Accepted by the raiOS owner on 2026-07-10.

## Context

The local `target\descriptor-resign` helper that produced fresh descriptor
signatures is absent. Existing descriptor bytes must not be altered, signatures
must not be forged or reused, and the parked OTA signer is not a substitute.

## Decision

Ship `descriptor-resign` as a standalone, tracked host tool. An explicit
`sign` invocation reads the descriptor's raw bytes unchanged, generates one
fresh P-256 keypair, and writes its SEC1 public key and DER ECDSA signature.
It accepts no private-key input; the private key exists only in process memory
for that invocation and is neither persisted nor printed.

`verify` accepts raw descriptor bytes, the SEC1 public key, and the DER
signature, then reports whether that exact tuple verifies. The tool never
canonicalizes, reserializes, or silently re-signs a descriptor. Builds and
other commands never invoke signing automatically.

Each successful output is development provenance only:
`trust_tier=dev_key_not_owner_sealed`. It grants no runtime, loader,
promotion, provider, OTA, or owner-sealed authority. The existing OTA tool
remains unused for descriptor signing.

## Consequences

- A lost local helper can be rebuilt from tracked source and every produced
  tuple can be independently verified.
- New explicit signing produces a fresh key and signature, so the process is
  reproducible and verifiable but not byte-identical across sign invocations.
- Private-key import, retention, printing, and automatic re-signing require a
  separate ADR and owner decision.
