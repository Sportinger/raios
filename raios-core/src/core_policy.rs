use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::{
    boot_control::{core_policy_boot_binding, BootControlDecision, BootSlotId, ParsedBootControl},
    sha256_bytes,
};

pub const CORE_POLICY_RECORD_LEN: usize = 128;
pub const CORE_POLICY_PAYLOAD_LEN: usize = 64;
pub const CORE_POLICY_SIGNATURE_LEN: usize = 64;
pub const CORE_POLICY_MAX_EXECUTABLE_LEN: u64 = 64 * 1024 * 1024;
pub const CORE_POLICY_MODULE_PATH: &[u8] = b"/raios/core-policy.bin";
pub const OWNER_CORE_POLICY_KEY_IS_PLACEHOLDER: bool = false;

pub const OWNER_CORE_POLICY_PUBLIC_KEY_SEC1: [u8; 65] = [
    0x04, 0x55, 0x74, 0x87, 0x82, 0x75, 0x2d, 0xd4, 0x05, 0xad, 0x49, 0xca, 0x53, 0xae, 0xf1, 0xb0,
    0xea, 0xd6, 0xf9, 0x3d, 0x44, 0x75, 0x99, 0xcf, 0x89, 0x8d, 0x36, 0xc6, 0x1b, 0x9f, 0x7a, 0x7d,
    0xde, 0x15, 0x13, 0x62, 0x94, 0x0b, 0x43, 0x2c, 0x0f, 0x6a, 0x44, 0x9e, 0xab, 0x05, 0xca, 0x08,
    0x06, 0x84, 0x6d, 0xea, 0x34, 0x19, 0xe7, 0xd4, 0x84, 0x27, 0xd2, 0x52, 0xce, 0xdd, 0x45, 0x46,
    0xf6,
];
pub const OWNER_CORE_POLICY_PUBLIC_KEY_SHA256: [u8; 32] = [
    0x87, 0xa2, 0x9c, 0x8a, 0x63, 0xdb, 0xd9, 0x83, 0xd9, 0xb7, 0x51, 0xe8, 0xbf, 0xc5, 0x52, 0x63,
    0xba, 0x08, 0xd3, 0x34, 0xcd, 0x9d, 0xa3, 0x12, 0x3e, 0x86, 0x1b, 0x7f, 0xe1, 0x2e, 0x08, 0x1f,
];

const CORE_POLICY_MAGIC: &[u8; 8] = b"RAIOSCP1";
const CORE_POLICY_VERSION: u32 = 1;
const CORE_POLICY_SIGNATURE_DOMAIN: &[u8] = b"raios.core_policy.signature.v1\0";
const CORE_POLICY_ID_DOMAIN: &[u8] = b"raios.core_policy.id.v1\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorePolicySlot {
    A,
    B,
}

impl CorePolicySlot {
    const fn boot_slot(self) -> BootSlotId {
        match self {
            Self::A => BootSlotId::A,
            Self::B => BootSlotId::B,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorePolicyPayload {
    slot: CorePolicySlot,
    core_generation: u64,
    executable_len: u64,
    executable_sha256: [u8; 32],
}

impl CorePolicyPayload {
    pub const fn slot(&self) -> CorePolicySlot {
        self.slot
    }

    pub const fn core_generation(&self) -> u64 {
        self.core_generation
    }

    pub const fn executable_len(&self) -> u64 {
        self.executable_len
    }

    pub const fn executable_sha256(&self) -> [u8; 32] {
        self.executable_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedCorePolicy {
    payload: CorePolicyPayload,
    policy_id_sha256: [u8; 32],
    owner_key_sha256: [u8; 32],
}

impl VerifiedCorePolicy {
    pub const fn slot(&self) -> CorePolicySlot {
        self.payload.slot
    }

    pub const fn core_generation(&self) -> u64 {
        self.payload.core_generation
    }

    pub const fn executable_len(&self) -> u64 {
        self.payload.executable_len
    }

    pub const fn executable_sha256(&self) -> [u8; 32] {
        self.payload.executable_sha256
    }

    pub const fn policy_id_sha256(&self) -> [u8; 32] {
        self.policy_id_sha256
    }

    pub const fn owner_key_sha256(&self) -> [u8; 32] {
        self.owner_key_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorePolicyDenied {
    BlobSizeMismatch,
    BadMagic,
    UnsupportedVersion,
    InvalidSlot,
    ReservedBytesNonzero,
    GenerationInvalid,
    ExecutableLengthInvalid,
    ExecutableHashInvalid,
    OwnerKeyPlaceholder,
    OwnerKeyFingerprintMismatch,
    OwnerKeyInvalid,
    SignatureInvalid,
    SignatureNotLowS,
    ExecutingKernelLengthMismatch,
    ExecutingKernelSha256Mismatch,
    BootControl(&'static str),
    BootSlotMismatch,
    BootGenerationMismatch,
}

impl CorePolicyDenied {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::BlobSizeMismatch => "core_policy_blob_size_mismatch",
            Self::BadMagic => "core_policy_bad_magic",
            Self::UnsupportedVersion => "core_policy_version_unsupported",
            Self::InvalidSlot => "core_policy_slot_invalid",
            Self::ReservedBytesNonzero => "core_policy_reserved_bytes_nonzero",
            Self::GenerationInvalid => "core_policy_generation_invalid",
            Self::ExecutableLengthInvalid => "core_policy_executable_length_invalid",
            Self::ExecutableHashInvalid => "core_policy_executable_hash_invalid",
            Self::OwnerKeyPlaceholder => "core_policy_owner_key_placeholder",
            Self::OwnerKeyFingerprintMismatch => "core_policy_owner_key_fingerprint_mismatch",
            Self::OwnerKeyInvalid => "core_policy_owner_key_invalid",
            Self::SignatureInvalid => "core_policy_signature_invalid",
            Self::SignatureNotLowS => "core_policy_signature_not_low_s",
            Self::ExecutingKernelLengthMismatch => "executing_kernel_length_mismatch",
            Self::ExecutingKernelSha256Mismatch => "executing_kernel_sha256_mismatch",
            Self::BootControl(reason) => reason,
            Self::BootSlotMismatch => "core_policy_slot_mismatch",
            Self::BootGenerationMismatch => "core_policy_generation_mismatch",
        }
    }
}

pub fn verify_current_core_policy(
    blob: &[u8],
    executable: &[u8],
    decision: &BootControlDecision,
    authoritative_record: Option<&ParsedBootControl>,
) -> Result<VerifiedCorePolicy, CorePolicyDenied> {
    verify_current_core_policy_with_root(
        blob,
        executable,
        decision,
        authoritative_record,
        &OWNER_CORE_POLICY_PUBLIC_KEY_SEC1,
        OWNER_CORE_POLICY_PUBLIC_KEY_SHA256,
        OWNER_CORE_POLICY_KEY_IS_PLACEHOLDER,
    )
}

fn verify_current_core_policy_with_root(
    blob: &[u8],
    executable: &[u8],
    decision: &BootControlDecision,
    authoritative_record: Option<&ParsedBootControl>,
    owner_public_key_sec1: &[u8],
    owner_public_key_sha256: [u8; 32],
    owner_key_is_placeholder: bool,
) -> Result<VerifiedCorePolicy, CorePolicyDenied> {
    let payload = parse_payload(blob)?;
    let actual_len =
        u64::try_from(executable.len()).map_err(|_| CorePolicyDenied::ExecutableLengthInvalid)?;
    if payload.executable_len != actual_len {
        return Err(CorePolicyDenied::ExecutingKernelLengthMismatch);
    }
    if payload.executable_sha256 != sha256_bytes(executable) {
        return Err(CorePolicyDenied::ExecutingKernelSha256Mismatch);
    }

    if owner_key_is_placeholder || owner_public_key_sec1.iter().all(|byte| *byte == 0) {
        return Err(CorePolicyDenied::OwnerKeyPlaceholder);
    }
    if owner_public_key_sha256 == [0; 32]
        || sha256_bytes(owner_public_key_sec1) != owner_public_key_sha256
    {
        return Err(CorePolicyDenied::OwnerKeyFingerprintMismatch);
    }
    let owner_key = VerifyingKey::from_sec1_bytes(owner_public_key_sec1)
        .map_err(|_| CorePolicyDenied::OwnerKeyInvalid)?;
    let signature = Signature::from_slice(&blob[CORE_POLICY_PAYLOAD_LEN..])
        .map_err(|_| CorePolicyDenied::SignatureInvalid)?;
    if signature.normalize_s().is_some() {
        return Err(CorePolicyDenied::SignatureNotLowS);
    }
    let payload_bytes: &[u8; CORE_POLICY_PAYLOAD_LEN] = blob[..CORE_POLICY_PAYLOAD_LEN]
        .try_into()
        .map_err(|_| CorePolicyDenied::BlobSizeMismatch)?;
    let message = core_policy_signature_message(payload_bytes);
    owner_key
        .verify(&message, &signature)
        .map_err(|_| CorePolicyDenied::SignatureInvalid)?;

    let authoritative_record = authoritative_record.ok_or(CorePolicyDenied::BootControl(
        "boot_control_authoritative_record_missing",
    ))?;
    let binding = core_policy_boot_binding(decision, authoritative_record)
        .map_err(CorePolicyDenied::BootControl)?;
    if binding.slot != payload.slot.boot_slot() {
        return Err(CorePolicyDenied::BootSlotMismatch);
    }
    if binding.generation != payload.core_generation {
        return Err(CorePolicyDenied::BootGenerationMismatch);
    }

    Ok(VerifiedCorePolicy {
        payload,
        policy_id_sha256: policy_id(owner_public_key_sha256, &blob[..CORE_POLICY_PAYLOAD_LEN]),
        owner_key_sha256: owner_public_key_sha256,
    })
}

fn parse_payload(blob: &[u8]) -> Result<CorePolicyPayload, CorePolicyDenied> {
    if blob.len() != CORE_POLICY_RECORD_LEN {
        return Err(CorePolicyDenied::BlobSizeMismatch);
    }
    let payload = &blob[..CORE_POLICY_PAYLOAD_LEN];
    if &payload[..8] != CORE_POLICY_MAGIC {
        return Err(CorePolicyDenied::BadMagic);
    }
    if read_u32(payload, 8) != CORE_POLICY_VERSION {
        return Err(CorePolicyDenied::UnsupportedVersion);
    }
    let slot = match payload[12] {
        1 => CorePolicySlot::A,
        2 => CorePolicySlot::B,
        _ => return Err(CorePolicyDenied::InvalidSlot),
    };
    if payload[13..16] != [0; 3] {
        return Err(CorePolicyDenied::ReservedBytesNonzero);
    }
    let core_generation = read_u64(payload, 16);
    if core_generation == 0 {
        return Err(CorePolicyDenied::GenerationInvalid);
    }
    let executable_len = read_u64(payload, 24);
    if executable_len == 0 || executable_len > CORE_POLICY_MAX_EXECUTABLE_LEN {
        return Err(CorePolicyDenied::ExecutableLengthInvalid);
    }
    let mut executable_sha256 = [0u8; 32];
    executable_sha256.copy_from_slice(&payload[32..64]);
    if executable_sha256 == [0; 32] {
        return Err(CorePolicyDenied::ExecutableHashInvalid);
    }
    Ok(CorePolicyPayload {
        slot,
        core_generation,
        executable_len,
        executable_sha256,
    })
}

pub fn encode_core_policy_payload(
    slot: CorePolicySlot,
    core_generation: u64,
    executable_len: u64,
    executable_sha256: [u8; 32],
) -> Result<[u8; CORE_POLICY_PAYLOAD_LEN], CorePolicyDenied> {
    if core_generation == 0 {
        return Err(CorePolicyDenied::GenerationInvalid);
    }
    if executable_len == 0 || executable_len > CORE_POLICY_MAX_EXECUTABLE_LEN {
        return Err(CorePolicyDenied::ExecutableLengthInvalid);
    }
    if executable_sha256 == [0; 32] {
        return Err(CorePolicyDenied::ExecutableHashInvalid);
    }
    let mut payload = [0u8; CORE_POLICY_PAYLOAD_LEN];
    payload[..8].copy_from_slice(CORE_POLICY_MAGIC);
    payload[8..12].copy_from_slice(&CORE_POLICY_VERSION.to_le_bytes());
    payload[12] = match slot {
        CorePolicySlot::A => 1,
        CorePolicySlot::B => 2,
    };
    payload[16..24].copy_from_slice(&core_generation.to_le_bytes());
    payload[24..32].copy_from_slice(&executable_len.to_le_bytes());
    payload[32..64].copy_from_slice(&executable_sha256);
    Ok(payload)
}

pub fn core_policy_signature_message(payload: &[u8; CORE_POLICY_PAYLOAD_LEN]) -> [u8; 95] {
    let mut message = [0u8; 95];
    message[..CORE_POLICY_SIGNATURE_DOMAIN.len()].copy_from_slice(CORE_POLICY_SIGNATURE_DOMAIN);
    message[CORE_POLICY_SIGNATURE_DOMAIN.len()..].copy_from_slice(payload);
    message
}

pub fn assemble_core_policy_record(
    payload: &[u8; CORE_POLICY_PAYLOAD_LEN],
    signature: &[u8; CORE_POLICY_SIGNATURE_LEN],
) -> [u8; CORE_POLICY_RECORD_LEN] {
    let mut record = [0u8; CORE_POLICY_RECORD_LEN];
    record[..CORE_POLICY_PAYLOAD_LEN].copy_from_slice(payload);
    record[CORE_POLICY_PAYLOAD_LEN..].copy_from_slice(signature);
    record
}

fn policy_id(owner_key_sha256: [u8; 32], payload: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(CORE_POLICY_ID_DOMAIN);
    hash.update(owner_key_sha256);
    hash.update(payload);
    let digest = hash.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boot_control::{
        BootAttempt, BootPayloadSlot, BootPayloadSlotState, BootPosture, BootStorageSlot,
    };
    use p256::ecdsa::{signature::Signer, SigningKey};

    const TEST_PRIVATE_SCALAR: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 7,
    ];

    fn signing_key() -> SigningKey {
        SigningKey::from_slice(&TEST_PRIVATE_SCALAR).unwrap()
    }

    fn root() -> ([u8; 65], [u8; 32]) {
        let encoded = signing_key().verifying_key().to_encoded_point(false);
        let mut key = [0u8; 65];
        key.copy_from_slice(encoded.as_bytes());
        (key, sha256_bytes(&key))
    }

    #[test]
    fn pinned_owner_root_parses_and_matches_its_fingerprint() {
        assert!(!OWNER_CORE_POLICY_KEY_IS_PLACEHOLDER);
        VerifyingKey::from_sec1_bytes(&OWNER_CORE_POLICY_PUBLIC_KEY_SEC1).unwrap();
        assert_eq!(
            sha256_bytes(&OWNER_CORE_POLICY_PUBLIC_KEY_SEC1),
            OWNER_CORE_POLICY_PUBLIC_KEY_SHA256
        );
    }

    fn boot(
        slot: BootSlotId,
        generation: u64,
        posture: BootPosture,
    ) -> (BootControlDecision, ParsedBootControl) {
        let record = ParsedBootControl {
            payload_len: 52,
            seq: 9,
            storage_sha256: [0x11; 32],
            payload_sha256: [0x22; 32],
            active: slot,
            last_good: slot,
            pending: BootSlotId::None,
            safe_mode: posture == BootPosture::Safe,
            boot_attempt: BootAttempt {
                slot,
                generation,
                success_marked: true,
            },
            slot_a: BootPayloadSlot {
                generation: if slot == BootSlotId::A { generation } else { 1 },
                state: BootPayloadSlotState::Good,
                failure_count: 0,
            },
            slot_b: BootPayloadSlot {
                generation: if slot == BootSlotId::B { generation } else { 1 },
                state: BootPayloadSlotState::Good,
                failure_count: 0,
            },
        };
        let decision = BootControlDecision {
            selected_storage_slot: Some(BootStorageSlot::A),
            selected_payload_slot: slot,
            authoritative_bootctl_slot: Some(BootStorageSlot::A),
            seq: Some(record.seq),
            posture,
            pending_consumed: false,
            would_mark_good: false,
            failure_count: None,
            reason: "test",
        };
        (decision, record)
    }

    fn blob(executable: &[u8], slot: CorePolicySlot, generation: u64) -> [u8; 128] {
        let payload = encode_core_policy_payload(
            slot,
            generation,
            executable.len() as u64,
            sha256_bytes(executable),
        )
        .unwrap();
        let signature: Signature = signing_key().sign(&core_policy_signature_message(&payload));
        let signature = signature.normalize_s().unwrap_or(signature);
        let signature_bytes: [u8; CORE_POLICY_SIGNATURE_LEN] = signature.to_bytes().into();
        assemble_core_policy_record(&payload, &signature_bytes)
    }

    fn verify(
        blob: &[u8],
        executable: &[u8],
        decision: &BootControlDecision,
        record: &ParsedBootControl,
    ) -> Result<VerifiedCorePolicy, CorePolicyDenied> {
        let (key, fingerprint) = root();
        verify_current_core_policy_with_root(
            blob,
            executable,
            decision,
            Some(record),
            &key,
            fingerprint,
            false,
        )
    }

    #[test]
    fn exact_record_verifies_and_binds_normal_boot() {
        let executable = b"real whole Limine executable bytes";
        let (decision, record) = boot(BootSlotId::A, 7, BootPosture::Normal);
        let verified = verify(
            &blob(executable, CorePolicySlot::A, 7),
            executable,
            &decision,
            &record,
        )
        .unwrap();

        assert_eq!(verified.slot(), CorePolicySlot::A);
        assert_eq!(verified.core_generation(), 7);
        assert_eq!(verified.executable_sha256(), sha256_bytes(executable));
        assert_ne!(verified.policy_id_sha256(), [0; 32]);
    }

    #[test]
    fn format_errors_and_payload_tamper_fail_closed() {
        let executable = b"kernel";
        let (decision, record) = boot(BootSlotId::A, 7, BootPosture::Normal);
        let valid = blob(executable, CorePolicySlot::A, 7);
        assert_eq!(
            verify(&valid[..127], executable, &decision, &record),
            Err(CorePolicyDenied::BlobSizeMismatch)
        );

        let mut bad_reserved = valid;
        bad_reserved[13] = 1;
        assert_eq!(
            verify(&bad_reserved, executable, &decision, &record),
            Err(CorePolicyDenied::ReservedBytesNonzero)
        );

        let mut tampered = valid;
        tampered[32] ^= 1;
        assert!(matches!(
            verify(&tampered, executable, &decision, &record),
            Err(CorePolicyDenied::ExecutingKernelSha256Mismatch)
                | Err(CorePolicyDenied::SignatureInvalid)
        ));
    }

    #[test]
    fn wrong_root_and_signature_tamper_are_rejected() {
        let executable = b"kernel";
        let (decision, record) = boot(BootSlotId::A, 7, BootPosture::Normal);
        let mut signed = blob(executable, CorePolicySlot::A, 7);
        signed[127] ^= 1;
        assert!(matches!(
            verify(&signed, executable, &decision, &record),
            Err(CorePolicyDenied::SignatureInvalid) | Err(CorePolicyDenied::SignatureNotLowS)
        ));

        let other = SigningKey::from_slice(&[9u8; 32]).unwrap();
        let encoded = other.verifying_key().to_encoded_point(false);
        let fingerprint = sha256_bytes(encoded.as_bytes());
        let valid = blob(executable, CorePolicySlot::A, 7);
        assert_eq!(
            verify_current_core_policy_with_root(
                &valid,
                executable,
                &decision,
                Some(&record),
                encoded.as_bytes(),
                fingerprint,
                false
            ),
            Err(CorePolicyDenied::SignatureInvalid)
        );
    }

    #[test]
    fn kernel_hash_slot_and_generation_mismatches_are_rejected() {
        let executable = b"kernel";
        let (decision, record) = boot(BootSlotId::A, 7, BootPosture::Probation);
        let valid = blob(executable, CorePolicySlot::A, 7);
        assert_eq!(
            verify(&valid, b"other!", &decision, &record),
            Err(CorePolicyDenied::ExecutingKernelSha256Mismatch)
        );
        assert_eq!(
            verify(
                &blob(executable, CorePolicySlot::B, 7),
                executable,
                &decision,
                &record
            ),
            Err(CorePolicyDenied::BootSlotMismatch)
        );
        assert_eq!(
            verify(
                &blob(executable, CorePolicySlot::A, 8),
                executable,
                &decision,
                &record
            ),
            Err(CorePolicyDenied::BootGenerationMismatch)
        );
    }

    #[test]
    fn safe_and_placeholder_roots_never_authorize() {
        let executable = b"kernel";
        let (safe, record) = boot(BootSlotId::A, 7, BootPosture::Safe);
        assert_eq!(
            verify(
                &blob(executable, CorePolicySlot::A, 7),
                executable,
                &safe,
                &record
            ),
            Err(CorePolicyDenied::BootControl(
                "boot_control_posture_not_authorizing"
            ))
        );

        let (normal, record) = boot(BootSlotId::A, 7, BootPosture::Normal);
        assert_eq!(
            verify_current_core_policy_with_root(
                &blob(executable, CorePolicySlot::A, 7),
                executable,
                &normal,
                Some(&record),
                &[0; 65],
                [0; 32],
                true,
            ),
            Err(CorePolicyDenied::OwnerKeyPlaceholder)
        );
    }
}
