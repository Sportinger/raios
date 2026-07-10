//! Typed Secret Vault payload records over the one structured-store namespace.
//!
//! This layer only encodes and decodes fixed V1 ciphertext records. It neither
//! selects a device nor writes a frame, decrypts a value, exposes plaintext, or
//! authorizes a consumer. Callers must pass replay-verified committed records;
//! nonce metadata is unavailable unless they explicitly prove complete ordered
//! history rather than merely the active replay view.

use alloc::vec::Vec;
use raios_core::{
    secret_vault::{
        RecoveryKekContext, RecoveryVmkWrapperV1, SecretConsumer, SecretEnvelopeV1, SecretKind,
        SecretOperation, SecretTargetBinding, SecretTargetKind, RECOVERY_WRAPPER_VERSION,
        SECRET_ENVELOPE_VERSION,
    },
    sha256_bytes,
    structured_store::{CommittedRecord, RecordKey, RecordValue, StoreIdentity},
};

pub(crate) const VAULT_NAMESPACE: [u8; 16] = *b"vault-----------";
pub(crate) const WIFI_PASSPHRASE_RECORD_ID: [u8; 16] = *b"wifi-passphrase-";
pub(crate) const PROVIDER_API_KEY_RECORD_ID: [u8; 16] = *b"provider-key----";
pub(crate) const RECOVERY_WRAPPER_RECORD_ID: [u8; 16] = *b"recovery-wrapper";

pub(crate) const SECRET_ENVELOPE_PAYLOAD_LEN: usize = 456;
pub(crate) const RECOVERY_WRAPPER_PAYLOAD_LEN: usize = 152;
pub(crate) const MAX_RETAINED_VAULT_COMMITS: usize = 4096;

const PAYLOAD_MAGIC: &[u8; 8] = b"RAIOSVL1";
const PAYLOAD_VERSION: u16 = 1;
const PAYLOAD_KIND_SECRET_ENVELOPE: u8 = 1;
const PAYLOAD_KIND_RECOVERY_WRAPPER: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VaultRecordId {
    WifiPassphrase,
    ProviderApiKey,
    RecoveryWrapper,
}

impl VaultRecordId {
    pub(crate) const fn key(self) -> RecordKey {
        RecordKey {
            namespace: VAULT_NAMESPACE,
            record_id: match self {
                Self::WifiPassphrase => WIFI_PASSPHRASE_RECORD_ID,
                Self::ProviderApiKey => PROVIDER_API_KEY_RECORD_ID,
                Self::RecoveryWrapper => RECOVERY_WRAPPER_RECORD_ID,
            },
        }
    }

    fn from_key(key: RecordKey) -> Result<Self, VaultStoreDenied> {
        if key.namespace != VAULT_NAMESPACE {
            return Err(VaultStoreDenied::NamespaceMismatch);
        }
        match key.record_id {
            WIFI_PASSPHRASE_RECORD_ID => Ok(Self::WifiPassphrase),
            PROVIDER_API_KEY_RECORD_ID => Ok(Self::ProviderApiKey),
            RECOVERY_WRAPPER_RECORD_ID => Ok(Self::RecoveryWrapper),
            _ => Err(VaultStoreDenied::RecordIdUnsupported),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VaultStoreDenied {
    StoreIdentityInvalid,
    NamespaceMismatch,
    RecordIdUnsupported,
    CommittedRecordInvalid,
    CommittedPayloadHashMismatch,
    PayloadLengthInvalid,
    PayloadMagicInvalid,
    PayloadVersionUnsupported,
    PayloadKindInvalid,
    PayloadReservedBytesNonzero,
    EnvelopeVersionUnsupported,
    EnvelopeSlotMismatch,
    EnvelopeStoreUuidMismatch,
    EnvelopeBindingInvalid,
    EnvelopeRecordVersionMismatch,
    EnvelopeCiphertextPaddingNonzero,
    WrapperVersionUnsupported,
    WrapperStoreUuidMismatch,
    WrapperContextInvalid,
    NonceHistoryIncomplete,
    NonceHistoryTooLong,
    NonceHistoryIdentityMismatch,
    NonceHistoryOrderInvalid,
    NonceHistoryRecordVersionInvalid,
    NonceHistoryPreviousRecordHashMismatch,
    NonceHistoryWrapperGenerationInvalid,
    RetainedSecretNonceDuplicate,
    RetainedRecoveryWrapperNonceDuplicate,
}

impl VaultStoreDenied {
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::StoreIdentityInvalid => "vault_store_identity_invalid",
            Self::NamespaceMismatch => "vault_namespace_mismatch",
            Self::RecordIdUnsupported => "vault_record_id_unsupported",
            Self::CommittedRecordInvalid => "vault_committed_record_invalid",
            Self::CommittedPayloadHashMismatch => "vault_committed_payload_hash_mismatch",
            Self::PayloadLengthInvalid => "vault_payload_length_invalid",
            Self::PayloadMagicInvalid => "vault_payload_magic_invalid",
            Self::PayloadVersionUnsupported => "vault_payload_version_unsupported",
            Self::PayloadKindInvalid => "vault_payload_kind_invalid",
            Self::PayloadReservedBytesNonzero => "vault_payload_reserved_bytes_nonzero",
            Self::EnvelopeVersionUnsupported => "vault_envelope_version_unsupported",
            Self::EnvelopeSlotMismatch => "vault_envelope_slot_mismatch",
            Self::EnvelopeStoreUuidMismatch => "vault_envelope_store_uuid_mismatch",
            Self::EnvelopeBindingInvalid => "vault_envelope_binding_invalid",
            Self::EnvelopeRecordVersionMismatch => "vault_envelope_record_version_mismatch",
            Self::EnvelopeCiphertextPaddingNonzero => "vault_envelope_ciphertext_padding_nonzero",
            Self::WrapperVersionUnsupported => "vault_wrapper_version_unsupported",
            Self::WrapperStoreUuidMismatch => "vault_wrapper_store_uuid_mismatch",
            Self::WrapperContextInvalid => "vault_wrapper_context_invalid",
            Self::NonceHistoryIncomplete => "vault_nonce_history_incomplete",
            Self::NonceHistoryTooLong => "vault_nonce_history_too_long",
            Self::NonceHistoryIdentityMismatch => "vault_nonce_history_identity_mismatch",
            Self::NonceHistoryOrderInvalid => "vault_nonce_history_order_invalid",
            Self::NonceHistoryRecordVersionInvalid => "vault_nonce_history_record_version_invalid",
            Self::NonceHistoryPreviousRecordHashMismatch => {
                "vault_nonce_history_previous_record_hash_mismatch"
            }
            Self::NonceHistoryWrapperGenerationInvalid => {
                "vault_nonce_history_wrapper_generation_invalid"
            }
            Self::RetainedSecretNonceDuplicate => "vault_retained_secret_nonce_duplicate",
            Self::RetainedRecoveryWrapperNonceDuplicate => {
                "vault_retained_recovery_wrapper_nonce_duplicate"
            }
        }
    }
}

/// A record borrowed from a structured-store replay path. Constructing it
/// checks the public committed-record shape and payload hash again; it does not
/// substitute for the structured store's own replay verification.
#[derive(Clone, Copy)]
pub(crate) struct ReplayVerifiedVaultRecord<'a> {
    identity: StoreIdentity,
    record: &'a CommittedRecord,
}

impl<'a> ReplayVerifiedVaultRecord<'a> {
    pub(crate) fn from_committed(
        identity: StoreIdentity,
        record: &'a CommittedRecord,
    ) -> Result<Self, VaultStoreDenied> {
        validate_store_identity(identity)?;
        VaultRecordId::from_key(record.key)?;
        if record.transaction_id == 0
            || record.record_version == 0
            || record.commit_frame_sha256 == [0; 32]
        {
            return Err(VaultStoreDenied::CommittedRecordInvalid);
        }
        match &record.value {
            RecordValue::Present(payload) if sha256_bytes(payload) != record.payload_sha256 => {
                Err(VaultStoreDenied::CommittedPayloadHashMismatch)
            }
            RecordValue::Present(_) => Ok(Self { identity, record }),
            RecordValue::Tombstone if record.payload_sha256 == sha256_bytes(&[]) => {
                Ok(Self { identity, record })
            }
            RecordValue::Tombstone => Err(VaultStoreDenied::CommittedPayloadHashMismatch),
        }
    }

    pub(crate) fn record_id(self) -> VaultRecordId {
        // The constructor validated this exact mapping.
        match self.record.key.record_id {
            WIFI_PASSPHRASE_RECORD_ID => VaultRecordId::WifiPassphrase,
            PROVIDER_API_KEY_RECORD_ID => VaultRecordId::ProviderApiKey,
            RECOVERY_WRAPPER_RECORD_ID => VaultRecordId::RecoveryWrapper,
            _ => unreachable!("validated vault record id"),
        }
    }

    pub(crate) const fn identity(self) -> StoreIdentity {
        self.identity
    }

    pub(crate) const fn transaction_id(self) -> u64 {
        self.record.transaction_id
    }

    pub(crate) const fn record_version(self) -> u64 {
        self.record.record_version
    }

    pub(crate) fn decode(self) -> Result<DecodedVaultRecord, VaultStoreDenied> {
        let slot = self.record_id();
        match &self.record.value {
            RecordValue::Tombstone => Ok(DecodedVaultRecord::Tombstone(slot)),
            RecordValue::Present(payload) => match slot {
                VaultRecordId::WifiPassphrase | VaultRecordId::ProviderApiKey => {
                    decode_secret_envelope(slot, payload, self.identity, self.record.record_version)
                        .map(|envelope| match slot {
                            VaultRecordId::WifiPassphrase => {
                                DecodedVaultRecord::WifiPassphrase(envelope)
                            }
                            VaultRecordId::ProviderApiKey => {
                                DecodedVaultRecord::ProviderApiKey(envelope)
                            }
                            VaultRecordId::RecoveryWrapper => unreachable!("secret slot"),
                        })
                }
                VaultRecordId::RecoveryWrapper => decode_recovery_wrapper(payload, self.identity)
                    .map(DecodedVaultRecord::RecoveryWrapper),
            },
        }
    }
}

/// Decoded ciphertext-only Vault data. This enum intentionally contains no
/// decrypt, plaintext, device, append, or consumer-use operation.
pub(crate) enum DecodedVaultRecord {
    WifiPassphrase(SecretEnvelopeV1),
    ProviderApiKey(SecretEnvelopeV1),
    RecoveryWrapper(RecoveryVmkWrapperV1),
    Tombstone(VaultRecordId),
}

/// Callers cannot derive complete nonce history from `ReplayState.records`:
/// that active view intentionally forgets superseded records. They must make
/// their full committed-log proof explicit here before nonce metadata exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReplayHistoryCompleteness {
    Complete,
    Incomplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RetainedSecretNonce {
    pub(crate) key_epoch: u64,
    pub(crate) nonce: [u8; 12],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RetainedRecoveryWrapperNonce {
    pub(crate) key_epoch: u64,
    pub(crate) wrapper_generation: u64,
    pub(crate) nonce: [u8; 12],
}

/// Nonce metadata reconstructed from every committed Vault record in a
/// complete ordered history. It deliberately is not constructible for an
/// active-only or incomplete replay view.
pub(crate) struct RetainedNonceMetadata {
    secret: Vec<RetainedSecretNonce>,
    recovery_wrapper: Vec<RetainedRecoveryWrapperNonce>,
}

impl RetainedNonceMetadata {
    pub(crate) fn secret(&self) -> &[RetainedSecretNonce] {
        &self.secret
    }

    pub(crate) fn recovery_wrapper(&self) -> &[RetainedRecoveryWrapperNonce] {
        &self.recovery_wrapper
    }

    pub(crate) fn contains_secret_nonce(&self, key_epoch: u64, nonce: [u8; 12]) -> bool {
        self.secret
            .iter()
            .any(|retained| retained.key_epoch == key_epoch && retained.nonce == nonce)
    }

    pub(crate) fn contains_recovery_wrapper_nonce(
        &self,
        key_epoch: u64,
        wrapper_generation: u64,
        nonce: [u8; 12],
    ) -> bool {
        self.recovery_wrapper.iter().any(|retained| {
            retained.key_epoch == key_epoch
                && retained.wrapper_generation == wrapper_generation
                && retained.nonce == nonce
        })
    }
}

/// Reconstructs nonce metadata only from a caller-supplied *complete*, ordered
/// stream of records that the structured-store replay has already verified.
/// Incomplete history denies instead of risking AES-GCM nonce reuse.
pub(crate) fn reconstruct_retained_nonce_metadata(
    history: &[ReplayVerifiedVaultRecord<'_>],
    completeness: ReplayHistoryCompleteness,
) -> Result<RetainedNonceMetadata, VaultStoreDenied> {
    if completeness != ReplayHistoryCompleteness::Complete {
        return Err(VaultStoreDenied::NonceHistoryIncomplete);
    }
    if history.len() > MAX_RETAINED_VAULT_COMMITS {
        return Err(VaultStoreDenied::NonceHistoryTooLong);
    }

    let mut identity = None;
    let mut previous_transaction_id = 0;
    let mut last_record_version = [0u64; 3];
    let mut last_commit_hash = [None; 3];
    let mut last_wrapper_generation = 0;
    let mut secret = Vec::new();
    let mut recovery_wrapper = Vec::new();

    for entry in history {
        if let Some(expected) = identity {
            if expected != entry.identity() {
                return Err(VaultStoreDenied::NonceHistoryIdentityMismatch);
            }
        } else {
            identity = Some(entry.identity());
        }
        if entry.transaction_id() <= previous_transaction_id {
            return Err(VaultStoreDenied::NonceHistoryOrderInvalid);
        }
        previous_transaction_id = entry.transaction_id();

        let slot_index = match entry.record_id() {
            VaultRecordId::WifiPassphrase => 0,
            VaultRecordId::ProviderApiKey => 1,
            VaultRecordId::RecoveryWrapper => 2,
        };
        if entry.record_version() <= last_record_version[slot_index] {
            return Err(VaultStoreDenied::NonceHistoryRecordVersionInvalid);
        }
        last_record_version[slot_index] = entry.record_version();

        match entry.decode()? {
            DecodedVaultRecord::WifiPassphrase(envelope)
            | DecodedVaultRecord::ProviderApiKey(envelope) => {
                if envelope.binding.previous_record_hash != last_commit_hash[slot_index] {
                    return Err(VaultStoreDenied::NonceHistoryPreviousRecordHashMismatch);
                }
                let retained = RetainedSecretNonce {
                    key_epoch: envelope.binding.key_epoch,
                    nonce: envelope.nonce,
                };
                if secret.contains(&retained) {
                    return Err(VaultStoreDenied::RetainedSecretNonceDuplicate);
                }
                secret.push(retained);
            }
            DecodedVaultRecord::RecoveryWrapper(wrapper) => {
                let retained = RetainedRecoveryWrapperNonce {
                    key_epoch: wrapper.context.key_epoch,
                    wrapper_generation: wrapper.context.wrapper_generation,
                    nonce: wrapper.nonce,
                };
                if recovery_wrapper.contains(&retained) {
                    return Err(VaultStoreDenied::RetainedRecoveryWrapperNonceDuplicate);
                }
                if wrapper.context.wrapper_generation <= last_wrapper_generation {
                    return Err(VaultStoreDenied::NonceHistoryWrapperGenerationInvalid);
                }
                last_wrapper_generation = wrapper.context.wrapper_generation;
                recovery_wrapper.push(retained);
            }
            DecodedVaultRecord::Tombstone(_) => {}
        }
        last_commit_hash[slot_index] = Some(entry.record.commit_frame_sha256);
    }

    Ok(RetainedNonceMetadata {
        secret,
        recovery_wrapper,
    })
}

pub(crate) fn encode_secret_envelope(
    slot: VaultRecordId,
    envelope: &SecretEnvelopeV1,
) -> Result<[u8; SECRET_ENVELOPE_PAYLOAD_LEN], VaultStoreDenied> {
    if matches!(slot, VaultRecordId::RecoveryWrapper) {
        return Err(VaultStoreDenied::EnvelopeSlotMismatch);
    }
    validate_secret_envelope(
        slot,
        envelope,
        envelope.binding.store_uuid,
        envelope.binding.record_version,
    )?;

    let mut out = [0u8; SECRET_ENVELOPE_PAYLOAD_LEN];
    write_payload_header(&mut out, PAYLOAD_KIND_SECRET_ENVELOPE);
    out[16..32].copy_from_slice(&envelope.binding.store_uuid);
    write_u64(&mut out, 32, envelope.binding.key_epoch);
    out[40..56].copy_from_slice(&envelope.binding.secret_id);
    write_u64(&mut out, 56, envelope.binding.record_version);
    out[64] = envelope.binding.kind as u8;
    out[65] = envelope.binding.consumer as u8;
    out[66] = envelope.binding.operation as u8;
    out[67] = envelope.binding.target.kind() as u8;
    out[68..100].copy_from_slice(&envelope.binding.target.hash());
    if let Some(previous) = envelope.binding.previous_record_hash {
        out[100] = 1;
        out[104..136].copy_from_slice(&previous);
    }
    write_u16(&mut out, 136, envelope.plaintext_len);
    out[140..172].copy_from_slice(&envelope.aad_hash);
    out[172..184].copy_from_slice(&envelope.nonce);
    out[184..440].copy_from_slice(&envelope.ciphertext);
    out[440..456].copy_from_slice(&envelope.tag);
    Ok(out)
}

pub(crate) fn encode_recovery_wrapper(
    wrapper: &RecoveryVmkWrapperV1,
) -> Result<[u8; RECOVERY_WRAPPER_PAYLOAD_LEN], VaultStoreDenied> {
    validate_recovery_wrapper(wrapper, wrapper.context.store_uuid)?;

    let mut out = [0u8; RECOVERY_WRAPPER_PAYLOAD_LEN];
    write_payload_header(&mut out, PAYLOAD_KIND_RECOVERY_WRAPPER);
    out[16..32].copy_from_slice(&wrapper.context.store_uuid);
    write_u64(&mut out, 32, wrapper.context.key_epoch);
    write_u64(&mut out, 40, wrapper.context.wrapper_generation);
    write_u64(&mut out, 48, wrapper.context.core_generation);
    out[56..88].copy_from_slice(&wrapper.context.policy_id_sha256);
    write_u32(&mut out, 88, wrapper.plaintext_len);
    out[92..104].copy_from_slice(&wrapper.nonce);
    out[104..136].copy_from_slice(&wrapper.ciphertext);
    out[136..152].copy_from_slice(&wrapper.tag);
    Ok(out)
}

fn decode_secret_envelope(
    slot: VaultRecordId,
    bytes: &[u8],
    identity: StoreIdentity,
    committed_record_version: u64,
) -> Result<SecretEnvelopeV1, VaultStoreDenied> {
    validate_store_identity(identity)?;
    parse_payload_header(
        bytes,
        SECRET_ENVELOPE_PAYLOAD_LEN,
        PAYLOAD_KIND_SECRET_ENVELOPE,
    )?;

    let previous_record_hash = match bytes[100] {
        0 if all_zero(&bytes[104..136]) => None,
        1 => Some(read_array::<32>(bytes, 104)),
        _ => return Err(VaultStoreDenied::EnvelopeBindingInvalid),
    };
    if !all_zero(&bytes[101..104]) || !all_zero(&bytes[138..140]) {
        return Err(VaultStoreDenied::PayloadReservedBytesNonzero);
    }
    let binding = raios_core::secret_vault::SecretBinding {
        store_uuid: read_array::<16>(bytes, 16),
        key_epoch: read_u64(bytes, 32),
        secret_id: read_array::<16>(bytes, 40),
        record_version: read_u64(bytes, 56),
        kind: parse_secret_kind(bytes[64])?,
        consumer: parse_secret_consumer(bytes[65])?,
        operation: parse_secret_operation(bytes[66])?,
        target: SecretTargetBinding::from_stored(
            parse_target_kind(bytes[67])?,
            read_array::<32>(bytes, 68),
        ),
        previous_record_hash,
    };
    let envelope = SecretEnvelopeV1 {
        version: read_u16(bytes, 8),
        binding,
        plaintext_len: read_u16(bytes, 136),
        aad_hash: read_array::<32>(bytes, 140),
        nonce: read_array::<12>(bytes, 172),
        ciphertext: read_array::<256>(bytes, 184),
        tag: read_array::<16>(bytes, 440),
    };
    validate_secret_envelope(
        slot,
        &envelope,
        identity.store_uuid,
        committed_record_version,
    )?;
    Ok(envelope)
}

fn decode_recovery_wrapper(
    bytes: &[u8],
    identity: StoreIdentity,
) -> Result<RecoveryVmkWrapperV1, VaultStoreDenied> {
    validate_store_identity(identity)?;
    parse_payload_header(
        bytes,
        RECOVERY_WRAPPER_PAYLOAD_LEN,
        PAYLOAD_KIND_RECOVERY_WRAPPER,
    )?;
    let wrapper = RecoveryVmkWrapperV1 {
        version: read_u16(bytes, 8),
        context: RecoveryKekContext {
            store_uuid: read_array::<16>(bytes, 16),
            key_epoch: read_u64(bytes, 32),
            wrapper_generation: read_u64(bytes, 40),
            core_generation: read_u64(bytes, 48),
            policy_id_sha256: read_array::<32>(bytes, 56),
        },
        plaintext_len: read_u32(bytes, 88),
        nonce: read_array::<12>(bytes, 92),
        ciphertext: read_array::<32>(bytes, 104),
        tag: read_array::<16>(bytes, 136),
    };
    validate_recovery_wrapper(&wrapper, identity.store_uuid)?;
    Ok(wrapper)
}

fn validate_secret_envelope(
    slot: VaultRecordId,
    envelope: &SecretEnvelopeV1,
    store_uuid: [u8; 16],
    committed_record_version: u64,
) -> Result<(), VaultStoreDenied> {
    if envelope.version != SECRET_ENVELOPE_VERSION {
        return Err(VaultStoreDenied::EnvelopeVersionUnsupported);
    }
    if envelope.binding.store_uuid != store_uuid {
        return Err(VaultStoreDenied::EnvelopeStoreUuidMismatch);
    }
    if envelope.binding.store_uuid == [0; 16]
        || envelope.binding.key_epoch == 0
        || envelope.binding.secret_id == [0; 16]
        || envelope.binding.record_version == 0
        || envelope.binding.target.hash() == [0; 32]
    {
        return Err(VaultStoreDenied::EnvelopeBindingInvalid);
    }
    if envelope.binding.record_version != committed_record_version {
        return Err(VaultStoreDenied::EnvelopeRecordVersionMismatch);
    }
    if envelope.plaintext_len == 0
        || envelope.plaintext_len as usize > envelope.binding.kind.max_len()
        || envelope.ciphertext[envelope.plaintext_len as usize..]
            .iter()
            .any(|byte| *byte != 0)
    {
        return Err(VaultStoreDenied::EnvelopeCiphertextPaddingNonzero);
    }

    let slot_matches_binding = match slot {
        VaultRecordId::WifiPassphrase => {
            envelope.binding.kind == SecretKind::WifiPassphrase
                && envelope.binding.consumer == SecretConsumer::NativeWifiSupplicant
                && envelope.binding.operation == SecretOperation::AssociateBoundBss
                && envelope.binding.target.kind() == SecretTargetKind::WifiBoundBss
        }
        VaultRecordId::ProviderApiKey => {
            envelope.binding.kind == SecretKind::ProviderApiKey
                && envelope.binding.consumer == SecretConsumer::OpenAiDirect
                && envelope.binding.operation == SecretOperation::RequestExactHost
                && envelope.binding.target.kind() == SecretTargetKind::ProviderExactHost
        }
        VaultRecordId::RecoveryWrapper => false,
    };
    if !slot_matches_binding {
        return Err(VaultStoreDenied::EnvelopeSlotMismatch);
    }
    Ok(())
}

fn validate_recovery_wrapper(
    wrapper: &RecoveryVmkWrapperV1,
    store_uuid: [u8; 16],
) -> Result<(), VaultStoreDenied> {
    if wrapper.version != RECOVERY_WRAPPER_VERSION {
        return Err(VaultStoreDenied::WrapperVersionUnsupported);
    }
    if wrapper.context.store_uuid != store_uuid {
        return Err(VaultStoreDenied::WrapperStoreUuidMismatch);
    }
    if wrapper.context.store_uuid == [0; 16]
        || wrapper.context.key_epoch == 0
        || wrapper.context.wrapper_generation == 0
        || wrapper.context.core_generation == 0
        || wrapper.context.policy_id_sha256 == [0; 32]
        || wrapper.plaintext_len != 32
    {
        return Err(VaultStoreDenied::WrapperContextInvalid);
    }
    Ok(())
}

fn validate_store_identity(identity: StoreIdentity) -> Result<(), VaultStoreDenied> {
    if identity.store_uuid == [0; 16] || identity.generation == 0 {
        return Err(VaultStoreDenied::StoreIdentityInvalid);
    }
    Ok(())
}

fn write_payload_header(out: &mut [u8], kind: u8) {
    out[..8].copy_from_slice(PAYLOAD_MAGIC);
    write_u16(out, 8, PAYLOAD_VERSION);
    out[10] = kind;
    write_u16(out, 12, out.len() as u16);
}

fn parse_payload_header(
    bytes: &[u8],
    expected_len: usize,
    expected_kind: u8,
) -> Result<(), VaultStoreDenied> {
    if bytes.len() != expected_len {
        return Err(VaultStoreDenied::PayloadLengthInvalid);
    }
    if &bytes[..8] != PAYLOAD_MAGIC {
        return Err(VaultStoreDenied::PayloadMagicInvalid);
    }
    if read_u16(bytes, 8) != PAYLOAD_VERSION {
        return Err(VaultStoreDenied::PayloadVersionUnsupported);
    }
    if bytes[10] != expected_kind {
        return Err(VaultStoreDenied::PayloadKindInvalid);
    }
    if bytes[11] != 0 || read_u16(bytes, 12) as usize != expected_len || !all_zero(&bytes[14..16]) {
        return Err(VaultStoreDenied::PayloadReservedBytesNonzero);
    }
    Ok(())
}

fn parse_secret_kind(value: u8) -> Result<SecretKind, VaultStoreDenied> {
    match value {
        1 => Ok(SecretKind::WifiPassphrase),
        2 => Ok(SecretKind::ProviderApiKey),
        _ => Err(VaultStoreDenied::EnvelopeBindingInvalid),
    }
}

fn parse_secret_consumer(value: u8) -> Result<SecretConsumer, VaultStoreDenied> {
    match value {
        1 => Ok(SecretConsumer::NativeWifiSupplicant),
        2 => Ok(SecretConsumer::OpenAiDirect),
        _ => Err(VaultStoreDenied::EnvelopeBindingInvalid),
    }
}

fn parse_secret_operation(value: u8) -> Result<SecretOperation, VaultStoreDenied> {
    match value {
        1 => Ok(SecretOperation::AssociateBoundBss),
        2 => Ok(SecretOperation::RequestExactHost),
        _ => Err(VaultStoreDenied::EnvelopeBindingInvalid),
    }
}

fn parse_target_kind(value: u8) -> Result<SecretTargetKind, VaultStoreDenied> {
    match value {
        1 => Ok(SecretTargetKind::WifiBoundBss),
        2 => Ok(SecretTargetKind::ProviderExactHost),
        _ => Err(VaultStoreDenied::EnvelopeBindingInvalid),
    }
}

fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> [u8; N] {
    let mut out = [0u8; N];
    out.copy_from_slice(&bytes[offset..offset + N]);
    out
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(read_array::<2>(bytes, offset))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(read_array::<4>(bytes, offset))
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(read_array::<8>(bytes, offset))
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use raios_core::{
        secret_vault::{
            RecoveryKekContext, RecoveryVmkWrapperV1, SecretBinding, SecretConsumer,
            SecretEnvelopeV1, SecretKind, SecretOperation, SecretTargetBinding,
        },
        sha256_bytes,
        structured_store::{CommittedRecord, RecordValue, StoreIdentity},
    };

    use super::*;

    const IDENTITY: StoreIdentity = StoreIdentity {
        store_uuid: *b"vault-store-test",
        generation: 3,
    };

    fn provider_envelope(version: u64, nonce: u8) -> SecretEnvelopeV1 {
        let mut ciphertext = [0u8; 256];
        ciphertext[..3].copy_from_slice(b"enc");
        SecretEnvelopeV1 {
            version: SECRET_ENVELOPE_VERSION,
            binding: SecretBinding {
                store_uuid: IDENTITY.store_uuid,
                key_epoch: 7,
                secret_id: *b"provider-secret-",
                record_version: version,
                kind: SecretKind::ProviderApiKey,
                consumer: SecretConsumer::OpenAiDirect,
                operation: SecretOperation::RequestExactHost,
                target: SecretTargetBinding::from_stored(
                    SecretTargetKind::ProviderExactHost,
                    [0x21; 32],
                ),
                previous_record_hash: None,
            },
            plaintext_len: 3,
            aad_hash: [0x31; 32],
            nonce: [nonce; 12],
            ciphertext,
            tag: [0x41; 16],
        }
    }

    fn wrapper(version: u64, nonce: u8) -> RecoveryVmkWrapperV1 {
        RecoveryVmkWrapperV1 {
            version: RECOVERY_WRAPPER_VERSION,
            context: RecoveryKekContext {
                store_uuid: IDENTITY.store_uuid,
                key_epoch: 7,
                wrapper_generation: version,
                core_generation: 9,
                policy_id_sha256: [0x51; 32],
            },
            plaintext_len: 32,
            nonce: [nonce; 12],
            ciphertext: [0x71; 32],
            tag: [0x81; 16],
        }
    }

    fn committed(
        slot: VaultRecordId,
        transaction_id: u64,
        record_version: u64,
        payload: Vec<u8>,
    ) -> CommittedRecord {
        CommittedRecord {
            key: slot.key(),
            transaction_id,
            record_version,
            commit_frame_sha256: [0x91; 32],
            payload_sha256: sha256_bytes(&payload),
            value: RecordValue::Present(payload),
        }
    }

    #[test]
    fn provider_and_wrapper_records_round_trip_without_plaintext() {
        let provider = provider_envelope(1, 0x12);
        let provider_bytes =
            encode_secret_envelope(VaultRecordId::ProviderApiKey, &provider).unwrap();
        let provider_record =
            committed(VaultRecordId::ProviderApiKey, 1, 1, provider_bytes.to_vec());
        let decoded = ReplayVerifiedVaultRecord::from_committed(IDENTITY, &provider_record)
            .unwrap()
            .decode()
            .unwrap();
        assert!(matches!(decoded, DecodedVaultRecord::ProviderApiKey(_)));

        let recovery_bytes = encode_recovery_wrapper(&wrapper(1, 0x61)).unwrap();
        let recovery_record = committed(
            VaultRecordId::RecoveryWrapper,
            2,
            1,
            recovery_bytes.to_vec(),
        );
        assert!(matches!(
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &recovery_record)
                .unwrap()
                .decode(),
            Ok(DecodedVaultRecord::RecoveryWrapper(_))
        ));
    }

    #[test]
    fn malformed_payload_and_incomplete_history_fail_closed() {
        let provider = provider_envelope(1, 0x12);
        let mut bytes = encode_secret_envelope(VaultRecordId::ProviderApiKey, &provider).unwrap();
        bytes[11] = 1;
        let record = committed(VaultRecordId::ProviderApiKey, 1, 1, bytes.to_vec());
        assert!(matches!(
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &record)
                .unwrap()
                .decode(),
            Err(VaultStoreDenied::PayloadReservedBytesNonzero)
        ));

        let valid = committed(
            VaultRecordId::ProviderApiKey,
            1,
            1,
            encode_secret_envelope(VaultRecordId::ProviderApiKey, &provider)
                .unwrap()
                .to_vec(),
        );
        let history = [ReplayVerifiedVaultRecord::from_committed(IDENTITY, &valid).unwrap()];
        assert!(matches!(
            reconstruct_retained_nonce_metadata(&history, ReplayHistoryCompleteness::Incomplete),
            Err(VaultStoreDenied::NonceHistoryIncomplete)
        ));
    }

    #[test]
    fn complete_history_reconstructs_nonce_metadata_and_rejects_reuse() {
        let first = committed(
            VaultRecordId::ProviderApiKey,
            1,
            1,
            encode_secret_envelope(VaultRecordId::ProviderApiKey, &provider_envelope(1, 0x12))
                .unwrap()
                .to_vec(),
        );
        let mut second_envelope = provider_envelope(2, 0x13);
        second_envelope.binding.previous_record_hash = Some(first.commit_frame_sha256);
        let second = committed(
            VaultRecordId::ProviderApiKey,
            2,
            2,
            encode_secret_envelope(VaultRecordId::ProviderApiKey, &second_envelope)
                .unwrap()
                .to_vec(),
        );
        let history = [
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &first).unwrap(),
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &second).unwrap(),
        ];
        let metadata =
            reconstruct_retained_nonce_metadata(&history, ReplayHistoryCompleteness::Complete)
                .unwrap();
        assert_eq!(metadata.secret().len(), 2);
        assert!(metadata.contains_secret_nonce(7, [0x12; 12]));

        let mut reused_envelope = provider_envelope(2, 0x12);
        reused_envelope.binding.previous_record_hash = Some(first.commit_frame_sha256);
        let reused = committed(
            VaultRecordId::ProviderApiKey,
            2,
            2,
            encode_secret_envelope(VaultRecordId::ProviderApiKey, &reused_envelope)
                .unwrap()
                .to_vec(),
        );
        let reused_history = [
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &first).unwrap(),
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &reused).unwrap(),
        ];
        assert!(matches!(
            reconstruct_retained_nonce_metadata(
                &reused_history,
                ReplayHistoryCompleteness::Complete
            ),
            Err(VaultStoreDenied::RetainedSecretNonceDuplicate)
        ));
    }

    #[test]
    fn wrapper_history_rejects_nonmonotonic_generation_and_duplicate_context() {
        let generation_two = committed(
            VaultRecordId::RecoveryWrapper,
            1,
            1,
            encode_recovery_wrapper(&wrapper(2, 0x61)).unwrap().to_vec(),
        );
        let generation_one = committed(
            VaultRecordId::RecoveryWrapper,
            2,
            2,
            encode_recovery_wrapper(&wrapper(1, 0x62)).unwrap().to_vec(),
        );
        let nonmonotonic = [
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &generation_two).unwrap(),
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &generation_one).unwrap(),
        ];
        assert!(matches!(
            reconstruct_retained_nonce_metadata(&nonmonotonic, ReplayHistoryCompleteness::Complete),
            Err(VaultStoreDenied::NonceHistoryWrapperGenerationInvalid)
        ));

        let first = committed(
            VaultRecordId::RecoveryWrapper,
            1,
            1,
            encode_recovery_wrapper(&wrapper(1, 0x61)).unwrap().to_vec(),
        );
        let duplicate = committed(
            VaultRecordId::RecoveryWrapper,
            2,
            2,
            encode_recovery_wrapper(&wrapper(1, 0x61)).unwrap().to_vec(),
        );
        let duplicate_context = [
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &first).unwrap(),
            ReplayVerifiedVaultRecord::from_committed(IDENTITY, &duplicate).unwrap(),
        ];
        assert!(matches!(
            reconstruct_retained_nonce_metadata(
                &duplicate_context,
                ReplayHistoryCompleteness::Complete
            ),
            Err(VaultStoreDenied::RetainedRecoveryWrapperNonceDuplicate)
        ));
    }
}
