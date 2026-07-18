# ADR 0012: Secret Vault Storage And Key Custody

Date: 2026-07-10 · Status: active

## Status

Accepted by the raiOS owner on 2026-07-10 through the autonomous goal that
references `docs/_archive/2026-07-18_genesis-shell-execution-plan-2026-07-10.md`.

This decision authorizes implementation of the bounded store, cryptographic
envelope, key wrappers, and two-consumer broker below. Acceptance is a design
decision only. It is not evidence that a structured store, encrypted secret,
recovery unlock, TPM unseal, physical target, or durable secret write exists.

## Context

Provider API keys and WiFi passphrases are currently RAM-only. Ordinary raiOS
memory deliberately rejects durable secret plaintext. M13 already selects a
structured, crash-consistent, encrypted store on a dedicated raiOS partition as
the durable-storage direction, but no M13 structured-store runtime or Secret
Vault exists yet.

The current repository has AHCI sector I/O but no generic final bounded-region
API and no NVMe driver. Existing persistence validators and the USB handoff are
tied to other media and must not become an accidental vault target. TPM2 ACPI
discovery and status planning also exist without TPM command transport,
seal/unseal, or positive owner-sealed evidence.

## Decision

### Exact physical target and denials

The only permitted physical target is an already provisioned, dedicated,
identity-checked raiOS data partition on an internal storage device. Every open
and every write revalidates the exact controller, port, device, GPT disk and
partition identity, raiOS partition type/label, store UUID, generation, start,
length, and operation bounds.

The generic M13 partition marker is GPT type GUID
`5eedda7a-c0de-4a55-9a15-000000000013` with the exact UTF-16 label
`RAIOS_STRUCTURED_STORE`. A real already-provisioned target retains its own
unique GPT partition GUID and is admitted only when that GUID is included in
the approved device fingerprint; raiOS never rewrites it to a test value. The
disposable QEMU fixture uses its separately documented deterministic disk and
partition GUIDs solely to exercise this marker and must never be accepted as a
physical-target identity.

The following are never Vault targets:

- the boot USB stick or another removable boot medium;
- `SEED_ESP_A`, `SEED_ESP_B`, or any EFI system partition;
- `SEED_DATA/RECLOG`, `ARTSTOR`, or the existing rollback/audit regions;
- Windows system, data, recovery, or other foreign partitions;
- the immutable recovery core; and
- an ambiguous, missing, unmarked, identity-mismatched, or out-of-bounds device.

This work does not create, resize, format, repartition, or relabel a physical
disk. A missing approved partition fails closed and never redirects writes to
USB, Windows, RECLOG, ARTSTOR, or any convenient substitute.

`physical_target_driver_supported` is a separate required gate. QEMU-AHCI
evidence cannot prove the Surface's internal target writable, and the current
absence of an NVMe driver may keep this gate false. That blocks only a claim of
physical persistence; it does not authorize a fallback target or prevent pure
logic and disposable-QEMU evidence from being built.

### One structured store

The durable backend is one append-only, log-structured M13 store shared by
future typed namespaces. The Vault is its first security-sensitive namespace,
not a separate raw secret region.

Each transaction is represented by:

```text
PREPARE -> one or more DATA frames -> COMMIT
                                  -> later TOMBSTONE for deletion
```

Replay accepts only complete, hash-linked, readback-verified committed
transactions. PREPARE or DATA without COMMIT is ignored after reboot. A corrupt
committed chain locks the affected namespace instead of selecting unverified
older bytes. Capacity exhaustion, integer overflow, stale generation, identity
mismatch, and bounds failure return explicit denial.

The store uses two superblock copies. Each carries version, store UUID,
geometry, generation, active-selection data, and an unkeyed SHA-256 corruption
check. A replacement is flushed and read back before the older copy can be
superseded. These superblocks are hash-checked, not signed: their hashes detect
corruption and are not described as identity authority or authenticity proof.
After unlock, AEAD tags and the committed hash chain authenticate Vault records
under the Vault Master Key.

Frames bind namespace, record id, transaction and record version, length,
previous-frame hash, payload hash, and integrity fields. Append completes only
after flush, readback, reparse, COMMIT, and replay of the exact committed record.
The first implementation remains append-only without speculative compaction.

### Cryptographic envelope

The implementation uses direct, exact-version RustCrypto dependencies with the
smallest verified `no_std` feature sets:

- `aes-gcm = 0.10.3`: AES-256-GCM with a 128-bit tag;
- `hkdf = 0.12.4`: HKDF-SHA-256 key separation; and
- `zeroize = 1.8.2`: zeroization of VMK, DEK, plaintext, parser, and temporary
  request buffers.

No AES, GHASH, HKDF, or other cryptography is implemented locally.

The Vault Master Key (VMK) is 32 random bytes from ready core entropy and is
never persisted in plaintext. It is a separate key family from the ADR 0007
promotion-authority/owner key and grants no code-load, capability, storage-
region, recovery, network, or provider authority.

Each record uses a 12-byte random nonce that is duplicate-rejected within its
key epoch. HKDF derives a per-record DEK from the VMK plus store UUID, key epoch,
secret id, record version, and secret kind. The authenticated data binds schema
and version, store UUID, key epoch, secret id and kind, exact consumer and
operation, target binding, monotonically increasing record version, plaintext
length, and previous committed record hash. Nonce uncertainty, duplicate nonce,
stale version, tag/AAD failure, entropy failure, or key-policy mismatch denies.

Ordinary `raios.memory_record` continues to reject secret durability. The Vault
stores only typed ciphertext envelopes and never weakens or bypasses
`secret_never_durable_until_sealed_secret_design`.

### Key wrappers and recovery-key format

The in-RAM VMK handle may be opened by two wrapper families:

`ApprovedCorePolicy` comes only from the opaque, positively verified owner
Core Policy measurement and BOOTCTL join defined by ADR 0014. A caller-supplied
generation, hash, key, or verification boolean cannot construct it. This
policy evidence alone does not unlock or decrypt the Vault; the wrapper,
recovery input, exact store, durable use-audit, and Broker gates below remain
independently required.

1. A TPM wrapper intended for automatic normal-boot unlock. A positive claim
   requires real bounded TPM2 command transport and a successful create, load,
   seal, reboot, and unseal chain bound to current/next/last-good core policy.
   ACPI discovery, interface metadata, or a status-register read is not sealing
   evidence. Until real swtpm or approved hardware evidence exists,
   `tpm_auto_unlock`, `vault_vmk_tpm_sealed`, and `tpm_vmk_wrapper_ready`
   remain `not_proven`. The separate ADR 0007 promotion-authority
   `owner_sealed` state is unaffected and still requires its own ceremony.
2. A recovery wrapper around the same VMK. Its key is a separate random 32-byte
   value shown once through a core-owned Genesis surface and never stored.
   HKDF derives the recovery KEK; AES-256-GCM wraps the VMK with store, epoch,
   and policy AAD.

The recovery-wrapper encoding is canonical. With `recovery_key[32]` as HKDF
IKM, derive `salt = SHA256("raios.vault.recovery.salt.v1" || store_uuid[16] ||
key_epoch_le64)` and use `info = "raios.vault.recovery.kek.v1" ||
wrapper_generation_le64 || core_generation_le64 || policy_id_sha256[32]` to
produce exactly 32 KEK bytes. Each wrapper uses a fresh random stored 12-byte
AES-GCM nonce; duplicate or uncertain nonce reuse under the same recovery KEK
denies before encryption. The wrapper AAD is exactly
`"raios.vault.vmk_wrapper.v1" || store_uuid[16] || key_epoch_le64 ||
wrapper_generation_le64 || core_generation_le64 || policy_id_sha256[32] ||
plaintext_len_le32`, where plaintext length is 32 for V1. All integer encodings
are little-endian and all domain strings are their exact ASCII bytes without a
terminating NUL.

Recovery-key presentation V1 is exactly:

```text
RR1-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-xxxxxxxx-cccc
```

The eight `xxxxxxxx` groups encode the 32 random key bytes as hexadecimal.
`cccc` is the first two bytes of SHA-256 over
`"raios-recovery-v1" || key`. Input accepts uppercase or lowercase hex but
requires the exact prefix, group count, separators, length, and checksum.
Genesis shows the value once and requires one complete re-entry confirmation
before committing the wrapper, then zeroizes display, input, and parser buffers.
The checksum detects transcription errors; it adds no entropy and is not a
password KDF. A low-entropy human password remains unsupported until a separate
memory-hard-KDF decision exists.

The mandatory automated positive path is a two-boot disposable-QEMU test that
creates the recovery wrapper, stores fake sentinel credentials, reopens the
exact store, unlocks with RR1, and exercises only the authorized consumers.
Missing TPM tooling does not block that path and is never replaced by fake TPM
success.

### Exact Secret Broker surface

Vault V1 supports only two secret kinds and two exact consumers:

| Secret kind | Consumer | Allowed purpose and target |
| --- | --- | --- |
| `wifi_passphrase` | trusted native WiFi supplicant | one association attempt for the bound SSID/BSSID/security profile |
| `provider_api_key` | `svc.provider.openai_direct` | one already trust-authorized request to exact host `api.openai.com` |

The broker exposes set/replace, forget, status, `use_for_wifi`, and
`use_for_provider`. It has no generic `get_secret`, reveal, copy, export, debug,
or Wasm interface. A successful use produces one bounded ephemeral plaintext
lease for the exact consumer, purpose, target, boot scope, service generation,
record version, key epoch, trust decision, and store/audit evidence, then
zeroizes it.

Wrong kind, consumer, purpose, host, BSSID/security binding, scope, generation,
version, epoch, trust, tag, or evidence denies before decryption where possible
and always before consumer use. Personal shells, AI output, diagnostics, serial,
recovery artifact loaders, and Wasm receive no plaintext or secret import.

Write-side authority is equally narrow. Set/replace, forget, and recovery-key
unlock originate only from explicit trusted Genesis/Recovery actions. Normal-
boot TPM unlock may occur automatically only after the exact positive TPM-
wrapper policy/evidence gate; no other automatic unlock path exists. Secret
plaintext enters only through the core-owned secure overlay;
`wifi_passphrase` is capped at 63 bytes and `provider_api_key` at 256 bytes.
Personal shells, providers, AI output, diagnostics, and ordinary services
cannot invoke these mutations.

SAFE/recovery may unlock a valid wrapper but never automatically connects.
Outbound use there requires one explicit trusted Genesis recovery action. A
provider response cannot trigger a Vault write, unlock, forget, or secret use.

## Consequences

- Recovery-key unlock can be built and verified independently of TPM support.
- TPM auto-unlock and physical internal-SSD persistence remain honest separate
  gates instead of being inferred from QEMU, ACPI, or an available block driver.
- System rollback does not roll credential versions backward; key wrappers track
  current/next/last-good core generations separately from monotonic Vault data.
- Forget appends a committed tombstone and removes future use authority. It does
  not falsely claim old flash cells were physically erased.
- The default image and boot stick contain no secret, VMK, recovery key, wrapper,
  Vault ciphertext, or test sentinel.

## Evidence Required Before Runtime Claims

This ADR does not grant a positive runtime claim. Such claims require focused
structured-store and Secret Vault profiles proving transaction replay,
readback, torn-write and corruption behavior, two-boot RR1 unlock, exact scoped
consumer use, sentinel absence from logs/UI/context/reports/artifacts, crash
containment, SAFE explicit-connect behavior, and the named fail-closed negative
matrix. Physical persistence additionally requires
`physical_target_driver_supported` plus identity-bound evidence on the approved
partition. TPM auto-unlock additionally requires real TPM seal/reboot/unseal
evidence.

## Non-Goals

- Formatting, partitioning, resizing, or choosing a physical disk.
- Storing arbitrary tokens, cookies, SSH keys, chat, personal-shell secrets, or
  generic blobs.
- Password-based unlock without an approved memory-hard KDF.
- Migrating existing memory, artifact, audit, or rollback stores in this slice.
- Treating a superblock hash as a signature or authority proof.
