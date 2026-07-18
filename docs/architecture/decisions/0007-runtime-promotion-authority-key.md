# ADR 0007: Runtime Promotion Authority Key

Date: 2026-07-06 · Status: active

## Status

Accepted for M6B-2 dev-key grant authority; owner-key sealing pending.

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

The current M6B key is the development promotion key. Its private scalar is a
known in-repo test fixture, and `PROMOTION_AUTHORITY_IS_PLACEHOLDER` remains
true as an honest label. For M6B-2 only, the owner deliberately gives this key
full capability-grant function when a local attestation signature verifies,
labeled `trust_tier=dev_key_not_owner_sealed`. It does not authorize load.
Replacing it with owner-generated promotion key K is the later sealing
ceremony.

Rotation is rebuild plus reflash: replacing the pinned public key requires a new
image, and old images keep trusting only the key they were built with.

## Owner Ratification Points

1. The owner generates promotion key K outside the repository and keeps the
   private key off the build tree, boot image, logs, and provider context.
2. The owner reviews and ratifies the pinned public key bytes and SHA-256
   fingerprint that replace the development key.
3. The owner explicitly approves the later sealing ceremony that replaces the
   development key with owner key K before any owner-sealed grant or load
   authorization is claimed.

## M6B-2 Enforcement Precondition

Ratified owner decision for this session: the dev/placeholder promotion key is
deliberately given full **grant** function for M6B-2, labeled
`trust_tier=dev_key_not_owner_sealed`. There is no placeholder hard-refuse
guard in this slice. `grants_capability` may flip true when the candidate's
current-boot evidence chain is complete and consistent and the local
attestation carries a P-256 promotion signature that verified against the
pinned dev key.

The load boundary remains denied: `can_load_now`, `authorizes_guest_load`,
service-slot allocation, loader dispatch, durable audit, rollback store/write,
and rollback application stay false or unavailable until M6C/M6D. Replacing
the const with owner key K and authorizing any load remain the later sealing
ceremony and promotion/rollback milestones, not M6B-2.

## Consequences

- Runtime promotion has a single pinned trust root instead of trusting keys from
  candidate input.
- Build-time descriptor signatures remain scoped to checked-in artifacts and do
  not authorize external code.
- Until the owner-key sealing ceremony happens, M6B may grant only under the
  development trust tier `dev_key_not_owner_sealed`; guest load, service-slot
  allocation, artifact load, service start, and load attempts remain denied.
