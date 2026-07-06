# ADR 0007: Runtime Promotion Authority Key

## Status

Proposed - pending owner ratification.

## Context

M6 introduces the first runtime trust anchor that can eventually authorize
external code. The build and descriptor signing keys prove checked-in build
artifacts; they must not become the authority for code delivered after boot.

The promotion loop needs a distinct key for runtime promotion decisions over
the evidence chain: manifest identity, candidate artifact hash, Shadow VM
report, local attestation, computed grant, audit/rollback evidence, and service
slot intent.

## Decision

The runtime promotion authority public key is pinned in the raiOS image and is
distinct from the build key and descriptor key. The input to verification may
carry a signature only. It must never carry the key used for verification,
because a self-carried key would make self-authorization trivial.

The promotion authority is the local owner-controlled trust anchor for external
code promotion. It is the only key family that may authorize an externally
delivered artifact to move from retained evidence toward capability grant and
live load. The AI, the artifact, the manifest, and provider output are not
promotion authorities.

The current key in M6B-1 is a non-ratified placeholder. Its private scalar is a
known in-repo test fixture, and `PROMOTION_AUTHORITY_IS_PLACEHOLDER` is true.
It grants nothing. Before any grant or promotion slice can authorize, the owner
must replace it with owner-generated promotion key K and the grant path must
hard-refuse while the placeholder flag remains true.

Rotation is rebuild plus reflash: replacing the pinned public key requires a new
image, and old images keep trusting only the key they were built with.

## Owner Ratification Points

1. The owner generates promotion key K outside the repository and keeps the
   private key off the build tree, boot image, logs, and provider context.
2. The owner reviews and ratifies the pinned public key bytes and SHA-256
   fingerprint that replace the placeholder.
3. The owner explicitly approves the first grant/promotion slice after
   `PROMOTION_AUTHORITY_IS_PLACEHOLDER` is false and the focused verification
   report proves placeholder refusal plus real-key verification.

## M6B-2 Enforcement Precondition (hard gate)

The M6B-1 adversarial review confirmed the placeholder verifies correctly and
grants nothing, but also that `PROMOTION_AUTHORITY_IS_PLACEHOLDER` is currently
defined and **not yet referenced by any gating code** — safe today only because
every authority boolean is an unconditional `no()`. Therefore, before M6B-2 (the
first slice that flips any grant/authority boolean to true):

1. The grant path MUST read `PROMOTION_AUTHORITY_IS_PLACEHOLDER` and hard-refuse
   (return a `capability_denied`-style non-authorizing result) while it is true.
   A `signature_verified=true` over the scalar-1 generator-point placeholder must
   never be treated as authority.
2. The placeholder key (`PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SEC1`, the
   NIST P-256 generator point) MUST be replaced by owner-generated key K, and the
   const renamed off `PLACEHOLDER_*`.
3. A focused verification report must prove BOTH: (a) with the flag true / the
   placeholder key, a valid signature still yields grant denied; (b) with the
   real key and flag false, verification authorizes exactly the scoped capability
   while load stays denied until audit/rollback/slot exist.

## Consequences

- Runtime promotion has a single pinned trust root instead of trusting keys from
  candidate input.
- Build-time descriptor signatures remain scoped to checked-in artifacts and do
  not authorize external code.
- Until owner ratification happens, M6B may verify signatures as a mechanism but
  must keep grants, guest load, service-slot allocation, artifact load, service
  start, and load attempts denied.
