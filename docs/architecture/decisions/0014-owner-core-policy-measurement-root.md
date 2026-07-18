# ADR 0014: Owner Core Policy Measurement Root

Date: 2026-07-10 · Status: active

## Status

Accepted by the raiOS owner on 2026-07-10 through the explicit authority
grant for a new owner Core Policy trust root.

This decision authorizes the software-pinned P-256 key, fixed policy record,
post-link signing, and runtime verification described below. Acceptance does
not prove that a policy record exists, that the running image verified one, or
that Secret Vault unlock, decryption, or use is authorized.

## Context

ADR 0012 binds a recovery wrapper to `core_generation` and
`policy_id_sha256`, but the kernel has no authoritative source for an
`ApprovedCorePolicy`. BOOTCTL records only mutable slot and generation state;
its unkeyed payload hash covers BOOTCTL metadata, not `kernel.elf`, and is not
a signature or policy identity.

The current boot carrier also loads one `/kernel/kernel.elf`. BOOTCTL evaluates
a logical A/B state machine, but Limine does not select that file from BOOTCTL
and deterministic ESP-A/B firmware selection is not proven. The first policy
binding must therefore authenticate the exact executable file Limine supplied
and its declared logical slot/generation without claiming a working A/B boot
selector.

## Decision

### Separate owner software key

Core Policy uses a distinct owner P-256 key family. It is not the descriptor,
distribution, development promotion, ADR 0007 promotion-owner, OTA, Vault VMK,
or recovery-key family and grants none of those authorities.

The raw 32-byte private scalar lives outside the repository by default at
`%LOCALAPPDATA%\raiOS\keys\core-policy-owner.p256.secret`. Creation is explicit
and atomic, refuses an existing file or any path inside the workspace, and
must replace inherited ACLs with entries for only the current Windows user SID
and LocalSystem SID. ACL failure deletes the new key. The private scalar never
enters the repository, image, environment, command output, logs, reports, or
provider context.

The corresponding uncompressed 65-byte SEC1 public key is the tracked runtime
pin. Host signing must derive the public key from the private scalar and
byte-compare it with that pin before signing. A local derived public-key copy
may be written under the ignored `target/core-policy/` directory for
fingerprint inspection, but it is not runtime input. The trust label is
`owner_software_pinned`; `hardware_rooted` and `owner_sealed` remain false.

### Exact V1 record

The only accepted record is exactly 128 bytes at
`boot():/raios/core-policy.bin`. The Limine module is non-required: absence
keeps Vault policy authority denied while the core continues booting.

The first 64 bytes are the canonical signed payload:

| Offset | Length | Field |
| ---: | ---: | --- |
| `0` | `8` | ASCII `RAIOSCP1` |
| `8` | `4` | version `1`, unsigned little-endian `u32` |
| `12` | `1` | logical payload slot: `1` = A, `2` = B |
| `13` | `3` | reserved; all bytes must be zero |
| `16` | `8` | nonzero core generation, little-endian `u64` |
| `24` | `8` | exact raw Limine executable-file length, little-endian `u64`, range `1..=67,108,864` |
| `32` | `32` | nonzero SHA-256 of the complete raw Limine executable file |

Bytes `64..128` are the raw 64-byte IEEE-P1363 P-256 ECDSA signature
`r[32] || s[32]`. DER, a carried public key, a fingerprint field, stored
verification booleans, trailing bytes, and non-canonical high-S signatures are
rejected.

The signed message is exactly 95 bytes:

```text
ASCII "raios.core_policy.signature.v1\0" (31 bytes, including NUL)
|| payload[0..64]
```

The policy identity is independent of signature encoding and is exactly:

```text
SHA256(
  ASCII "raios.core_policy.id.v1\0" (24 bytes, including NUL)
  || SHA256(pinned_owner_public_key_sec1[65])
  || payload[0..64]
)
```

The magic, version, and signature domain fix V1 to the Secret Vault core-policy
purpose and whole raw Limine executable-file measurement. V1 carries no
extensible purpose, algorithm, key, or caller-supplied verification flag.

### Runtime approval

Packaging computes and signs the policy only after `kernel.elf` is linked. At
runtime the core hashes the complete raw executable bytes and length returned
by Limine, verifies the fixed record and low-S signature against the compiled
public pin, and computes the policy ID from the same payload.

An opaque `ApprovedCorePolicy` may be constructed only when all of these are
true:

- exactly one well-formed V1 policy record is present;
- its raw-file length and SHA-256 match the complete Limine executable file;
- the signature verifies against the compiled owner Core Policy pin;
- BOOTCTL has one authoritative valid record and evaluates to `Normal` or
  `Probation`;
- BOOTCTL selects logical slot A or B; and
- that selected slot and its nonzero generation exactly match the signed slot
  and generation.

Missing, duplicate, short, oversized, malformed, wrong-domain, wrong-key,
high-S, bad-signature, length/hash-mismatched, all-zero, partial-measurement,
invalid-BOOTCTL, `PersistenceUnavailable`, missing/mismatched selected-slot,
or zero/mismatched generation cases deny `ApprovedCorePolicy`. SAFE also
denies it because BOOTCTL deliberately selects no payload there and the actual
last-good executable relation is not yet proven. These denials do not prevent
the permanent core, Genesis, or local recovery from booting.

The verifier never substitutes BOOTCTL's payload hash, a compile-time constant,
descriptor/development signatures, an unsigned manifest, or a caller-provided
key/hash/boolean. The fixed single-file carrier means the signed slot is a
logical binding only; it does not prove that firmware or Limine selected an ESP
from BOOTCTL.

### Vault authority remains composed

This verified path is the sole runtime source of `ApprovedCorePolicy`. Its
fields remain opaque and callers cannot construct approval from a generation,
hash, or prior `verified` flag.

Approval alone does not unwrap a VMK, unlock the Vault, decrypt or expose a
secret, create a Broker lease, authorize a consumer, write storage, connect a
network, or grant code load. ADR 0012 wrapper replay, recovery-key input, exact
store identity, and the durable per-use audit and consumer evidence remain
independent required gates. TPM automatic unlock remains `not_proven`.

## Claim Boundary

The positive claim is limited to this statement: the complete raw kernel file
supplied by Limine matches an owner-software-key-signed Core Policy record whose
logical slot and generation match the current authoritative Normal/Probation
BOOTCTL decision.

Without Secure Boot or a TPM measured-boot chain, this does not prove firmware,
Limine, or hardware integrity; machine binding; current in-memory executable
integrity after load; physical-tamper resistance; deterministic ESP-A/B
selection; or anti-rollback. A replaced kernel can replace its embedded pin,
and an older signed kernel/policy plus matching mutable BOOTCTL state can still
be replayed. Those facts must remain explicit in runtime evidence and owner
handoff documentation.

## Evidence Required Before Runtime Claims

- Host tooling proves key initialization, ACL enforcement, pin match, exact
  128-byte output, sign/verify, altered-field rejection, high-S rejection, and
  refusal to read a private key from the workspace.
- Runtime tests recompute the real Limine raw-file length/hash and cover the
  exact positive BOOTCTL join plus each fail-closed class above.
- A focused VM report proves that a missing or invalid policy keeps Vault
  authority denied while Genesis/recovery still boots.
- Secret Vault unlock/decrypt/use remains unclaimed until ADR 0012's separate
  wrapper, input, store, audit, consumer, reboot, and leakage evidence passes.

## Consequences

- Recovery wrappers can bind to a real measured Core Policy identity instead
  of an invented hash or BOOTCTL metadata hash.
- Core Policy rotation requires a new pinned public key and newly signed
  policies; it does not silently reuse another raiOS signing family.
- SAFE recovery unlock and hardware-rooted/anti-rollback claims remain later
  work rather than being inferred from a software signature.
