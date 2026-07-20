//! Fixed-capacity Secret Vault V1 cryptographic records.
//!
//! This module owns only authenticated envelopes and key derivation. It does
//! not select storage, generate randomness, reveal plaintext, or authorize a
//! consumer. Nonces come from the caller and are accepted only with complete
//! per-epoch duplicate-check evidence.

use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Nonce, Tag,
};
use embedded_io::Write;
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

pub const SECRET_ENVELOPE_VERSION: u16 = 1;
pub const RECOVERY_WRAPPER_VERSION: u16 = 1;
pub const MAX_WIFI_PASSPHRASE_LEN: usize = 63;
pub const MAX_PROVIDER_API_KEY_LEN: usize = 256;
pub const MAX_SECRET_LEN: usize = MAX_PROVIDER_API_KEY_LEN;
pub const OPENAI_API_HOST: &str = "api.openai.com";

const RECORD_AAD_DOMAIN: &[u8] = b"raios.secret_vault.envelope.v1";
const RECORD_DEK_SALT_DOMAIN: &[u8] = b"raios.vault.record.salt.v1";
const RECORD_DEK_INFO_DOMAIN: &[u8] = b"raios.vault.record.dek.v1";
const WIFI_TARGET_DOMAIN: &[u8] = b"raios.vault.wifi_target.v1";
const WIFI_SECURITY_WPA2_PSK_CCMP: &[u8] = b"wpa2_psk_ccmp";
const PROVIDER_TARGET_DOMAIN: &[u8] = b"raios.vault.provider_target.v1";
const RECOVERY_SALT_DOMAIN: &[u8] = b"raios.vault.recovery.salt.v1";
const RECOVERY_INFO_DOMAIN: &[u8] = b"raios.vault.recovery.kek.v1";
const RECOVERY_WRAPPER_AAD_DOMAIN: &[u8] = b"raios.vault.vmk_wrapper.v1";

const RECORD_AAD_CAPACITY: usize = 192;
const DEK_INFO_CAPACITY: usize = 64;
const RECOVERY_INFO_CAPACITY: usize = 96;
const RECOVERY_AAD_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SecretKind {
    WifiPassphrase = 1,
    ProviderApiKey = 2,
}

impl SecretKind {
    pub const fn max_len(self) -> usize {
        match self {
            Self::WifiPassphrase => MAX_WIFI_PASSPHRASE_LEN,
            Self::ProviderApiKey => MAX_PROVIDER_API_KEY_LEN,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SecretConsumer {
    NativeWifiSupplicant = 1,
    OpenAiDirect = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SecretOperation {
    AssociateBoundBss = 1,
    RequestExactHost = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SecretTargetKind {
    WifiBoundBss = 1,
    ProviderExactHost = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecretTargetBinding {
    kind: SecretTargetKind,
    hash: [u8; 32],
}

impl SecretTargetBinding {
    /// Binds one validated WPA2-PSK/CCMP profile without retaining its SSID.
    pub fn for_wifi_wpa2_psk_ccmp(ssid: &[u8], bssid: [u8; 6]) -> Result<Self, SecretVaultError> {
        if ssid.is_empty() || ssid.len() > 32 {
            return Err(SecretVaultError::InvalidWifiSsidLength);
        }
        if bssid == [0; 6] || bssid[0] & 1 != 0 {
            return Err(SecretVaultError::InvalidWifiBssid);
        }

        let mut hash = Sha256::new();
        hash.update(WIFI_TARGET_DOMAIN);
        hash.update((ssid.len() as u16).to_le_bytes());
        hash.update(ssid);
        hash.update(bssid);
        hash.update(WIFI_SECURITY_WPA2_PSK_CCMP);
        Ok(Self {
            kind: SecretTargetKind::WifiBoundBss,
            hash: digest_array(hash),
        })
    }

    /// Accepts only the V1 OpenAI direct-provider host.
    pub fn for_openai_host(host: &str) -> Result<Self, SecretVaultError> {
        if host != OPENAI_API_HOST {
            return Err(SecretVaultError::ProviderHostMismatch);
        }
        let mut hash = Sha256::new();
        hash.update(PROVIDER_TARGET_DOMAIN);
        hash.update((host.len() as u16).to_le_bytes());
        hash.update(host.as_bytes());
        Ok(Self {
            kind: SecretTargetKind::ProviderExactHost,
            hash: digest_array(hash),
        })
    }

    /// Reconstructs untrusted stored metadata. It grants no use authority;
    /// AEAD verification and `scoped_secret_use` must still accept it.
    pub const fn from_stored(kind: SecretTargetKind, hash: [u8; 32]) -> Self {
        Self { kind, hash }
    }

    pub const fn kind(self) -> SecretTargetKind {
        self.kind
    }

    pub const fn hash(self) -> [u8; 32] {
        self.hash
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecretBinding {
    pub store_uuid: [u8; 16],
    pub key_epoch: u64,
    pub secret_id: [u8; 16],
    pub record_version: u64,
    pub kind: SecretKind,
    pub consumer: SecretConsumer,
    pub operation: SecretOperation,
    pub target: SecretTargetBinding,
    pub previous_record_hash: Option<[u8; 32]>,
}

/// Caller-supplied nonce plus the complete retained nonce set for its epoch.
pub struct FreshRecordNonce<'a> {
    pub nonce: [u8; 12],
    pub retained_epoch_nonces: &'a [[u8; 12]],
    pub retained_set_complete: bool,
}

pub struct FreshWrapperNonce<'a> {
    pub nonce: [u8; 12],
    pub retained_kek_nonces: &'a [[u8; 12]],
    pub retained_set_complete: bool,
}

/// VMK ownership is unique and its bytes are erased on drop.
pub struct VaultMasterKey([u8; 32]);

impl VaultMasterKey {
    pub fn take_from(source: &mut [u8; 32]) -> Self {
        let key = Self(*source);
        source.zeroize();
        key
    }
}

impl Zeroize for VaultMasterKey {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for VaultMasterKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Recovery-key ownership is unique and its bytes are erased on drop.
pub struct RecoveryKey([u8; 32]);

impl RecoveryKey {
    pub fn take_from(source: &mut [u8; 32]) -> Self {
        let key = Self(*source);
        source.zeroize();
        key
    }
}

impl Zeroize for RecoveryKey {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for RecoveryKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Fixed-capacity plaintext taken from a mutable secure-input buffer.
///
/// No public byte accessor exists. The final broker adapter must add only the
/// two exact native-consumer calls rather than a generic reveal operation.
pub struct SecretPlaintext {
    kind: SecretKind,
    bytes: [u8; MAX_SECRET_LEN],
    len: usize,
}

/// Errors from the two fixed native secret consumers.
///
/// These operations deliberately have no byte-returning counterpart. A
/// plaintext can become only the bounded Marvell WPA2 command or the exact
/// OpenAI authorization header, then its owner zeroizes on return.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretConsumerError {
    WrongKind {
        expected: SecretKind,
        actual: SecretKind,
    },
    WifiCommand(crate::marvell_wifi_supplicant::SupplicantError),
    OpenAiAuthorizationWriteFailed,
}

impl SecretPlaintext {
    pub fn take_from(kind: SecretKind, source: &mut [u8]) -> Result<Self, SecretVaultError> {
        let len = source.len();
        if len == 0 {
            source.zeroize();
            return Err(SecretVaultError::EmptySecret);
        }
        if len > kind.max_len() {
            source.zeroize();
            return Err(SecretVaultError::SecretTooLong { kind, length: len });
        }

        let mut bytes = [0u8; MAX_SECRET_LEN];
        bytes[..len].copy_from_slice(source);
        source.zeroize();
        Ok(Self { kind, bytes, len })
    }

    pub const fn kind(&self) -> SecretKind {
        self.kind
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    /// Consumes exactly one WiFi passphrase into the bounded NXP WPA2-PSK/CCMP
    /// supplicant command. It exposes no passphrase slice to the caller.
    pub fn into_wpa2_psk_ccmp_pmk_set(
        self,
        seq: u16,
        bssid: [u8; 6],
        ssid: &[u8],
        out: &mut [u8],
    ) -> Result<usize, SecretConsumerError> {
        if self.kind != SecretKind::WifiPassphrase {
            return Err(SecretConsumerError::WrongKind {
                expected: SecretKind::WifiPassphrase,
                actual: self.kind,
            });
        }
        crate::marvell_wifi_supplicant::build_supplicant_pmk_set(
            seq,
            bssid,
            ssid,
            self.as_bytes(),
            out,
        )
        .map_err(SecretConsumerError::WifiCommand)
    }

    /// Consumes exactly one provider key into the V1 direct-OpenAI
    /// authorization header. The writer receives only the protocol bytes;
    /// this API never returns a key slice, length, buffer, or callback.
    pub fn into_openai_authorization_header<W: Write>(
        self,
        writer: &mut W,
    ) -> Result<(), SecretConsumerError> {
        if self.kind != SecretKind::ProviderApiKey {
            return Err(SecretConsumerError::WrongKind {
                expected: SecretKind::ProviderApiKey,
                actual: self.kind,
            });
        }
        writer
            .write_all(b"Authorization: Bearer ")
            .and_then(|()| writer.write_all(self.as_bytes()))
            .and_then(|()| writer.write_all(b"\r\n"))
            .map_err(|_| SecretConsumerError::OpenAiAuthorizationWriteFailed)
    }
}

impl Zeroize for SecretPlaintext {
    fn zeroize(&mut self) {
        self.bytes.zeroize();
        self.len.zeroize();
    }
}

impl Drop for SecretPlaintext {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Storage-ready fixed-capacity ciphertext. It deliberately has no `Debug`.
pub struct SecretEnvelopeV1 {
    pub version: u16,
    pub binding: SecretBinding,
    pub plaintext_len: u16,
    pub aad_hash: [u8; 32],
    pub nonce: [u8; 12],
    pub ciphertext: [u8; MAX_SECRET_LEN],
    pub tag: [u8; 16],
}

impl SecretEnvelopeV1 {
    pub fn ciphertext(&self) -> Option<&[u8]> {
        self.ciphertext.get(..self.plaintext_len as usize)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretVaultError {
    EmptySecret,
    SecretTooLong { kind: SecretKind, length: usize },
    PlaintextKindMismatch,
    InvalidWifiSsidLength,
    InvalidWifiBssid,
    ProviderHostMismatch,
    InvalidRecordVersion,
    InvalidStoreUuid,
    InvalidKeyEpoch,
    InvalidSecretId,
    InvalidTargetBinding,
    StaleRecordVersion,
    NonceHistoryUncertain,
    DuplicateNonce,
    EncodingOverflow,
    KdfExpandFailed,
    EncryptionFailed,
    UnsupportedEnvelopeVersion,
    StoreUuidMismatch,
    KeyEpochMismatch,
    SecretIdMismatch,
    RecordVersionMismatch,
    SecretKindMismatch,
    ConsumerMismatch,
    OperationMismatch,
    TargetMismatch,
    PreviousRecordHashMismatch,
    InvalidCiphertextLength,
    AadHashMismatch,
    AuthenticationFailed,
    UnsupportedRecoveryWrapperVersion,
    WrapperGenerationMismatch,
    CoreGenerationMismatch,
    PolicyIdMismatch,
    InvalidRecoveryWrapperLength,
}

pub fn encrypt_secret(
    vmk: &VaultMasterKey,
    plaintext: SecretPlaintext,
    binding: SecretBinding,
    latest_committed_version: Option<u64>,
    fresh_nonce: FreshRecordNonce<'_>,
) -> Result<SecretEnvelopeV1, SecretVaultError> {
    validate_binding_shape(&binding)?;
    if plaintext.kind != binding.kind {
        return Err(SecretVaultError::PlaintextKindMismatch);
    }
    if binding.record_version == 0 {
        return Err(SecretVaultError::InvalidRecordVersion);
    }
    if latest_committed_version.is_some_and(|latest| binding.record_version <= latest) {
        return Err(SecretVaultError::StaleRecordVersion);
    }
    if !fresh_nonce.retained_set_complete {
        return Err(SecretVaultError::NonceHistoryUncertain);
    }
    if fresh_nonce
        .retained_epoch_nonces
        .contains(&fresh_nonce.nonce)
    {
        return Err(SecretVaultError::DuplicateNonce);
    }

    let aad = build_record_aad(&binding, plaintext.len)?;
    let aad_hash = Sha256::digest(aad.as_bytes()).into();
    let dek = derive_record_dek(vmk, &binding)?;
    let cipher =
        Aes256Gcm::new_from_slice(&dek.bytes).map_err(|_| SecretVaultError::EncryptionFailed)?;
    let mut scratch = SecretScratch([0u8; MAX_SECRET_LEN]);
    scratch.0[..plaintext.len].copy_from_slice(plaintext.as_bytes());
    let tag = cipher
        .encrypt_in_place_detached(
            Nonce::from_slice(&fresh_nonce.nonce),
            aad.as_bytes(),
            &mut scratch.0[..plaintext.len],
        )
        .map_err(|_| SecretVaultError::EncryptionFailed)?;
    let mut tag_bytes = [0u8; 16];
    tag_bytes.copy_from_slice(&tag);
    let mut ciphertext = [0u8; MAX_SECRET_LEN];
    ciphertext[..plaintext.len].copy_from_slice(&scratch.0[..plaintext.len]);

    Ok(SecretEnvelopeV1 {
        version: SECRET_ENVELOPE_VERSION,
        binding,
        plaintext_len: plaintext.len as u16,
        aad_hash,
        nonce: fresh_nonce.nonce,
        ciphertext,
        tag: tag_bytes,
    })
}

/// Decrypts only after every caller-expected binding matches exactly.
///
/// The returned fixed-capacity owner has no public plaintext accessor and
/// zeroizes itself on drop.
pub fn decrypt_secret(
    vmk: &VaultMasterKey,
    envelope: &SecretEnvelopeV1,
    expected: &SecretBinding,
    minimum_record_version: u64,
) -> Result<SecretPlaintext, SecretVaultError> {
    if envelope.version != SECRET_ENVELOPE_VERSION {
        return Err(SecretVaultError::UnsupportedEnvelopeVersion);
    }
    if envelope.binding.record_version < minimum_record_version {
        return Err(SecretVaultError::StaleRecordVersion);
    }
    validate_binding_shape(&envelope.binding)?;
    validate_binding_shape(expected)?;
    compare_bindings(&envelope.binding, expected)?;

    let len = envelope.plaintext_len as usize;
    if len == 0 || len > envelope.binding.kind.max_len() {
        return Err(SecretVaultError::InvalidCiphertextLength);
    }
    let aad = build_record_aad(&envelope.binding, len)?;
    let expected_aad_hash: [u8; 32] = Sha256::digest(aad.as_bytes()).into();
    if envelope.aad_hash != expected_aad_hash {
        return Err(SecretVaultError::AadHashMismatch);
    }

    let dek = derive_record_dek(vmk, &envelope.binding)?;
    let cipher = Aes256Gcm::new_from_slice(&dek.bytes)
        .map_err(|_| SecretVaultError::AuthenticationFailed)?;
    let mut plaintext = SecretPlaintext {
        kind: envelope.binding.kind,
        bytes: [0u8; MAX_SECRET_LEN],
        len,
    };
    plaintext.bytes[..len].copy_from_slice(&envelope.ciphertext[..len]);
    cipher
        .decrypt_in_place_detached(
            Nonce::from_slice(&envelope.nonce),
            aad.as_bytes(),
            &mut plaintext.bytes[..len],
            Tag::from_slice(&envelope.tag),
        )
        .map_err(|_| SecretVaultError::AuthenticationFailed)?;
    Ok(plaintext)
}

fn validate_binding_shape(binding: &SecretBinding) -> Result<(), SecretVaultError> {
    if binding.store_uuid == [0; 16] {
        return Err(SecretVaultError::InvalidStoreUuid);
    }
    if binding.key_epoch == 0 {
        return Err(SecretVaultError::InvalidKeyEpoch);
    }
    if binding.secret_id == [0; 16] {
        return Err(SecretVaultError::InvalidSecretId);
    }
    if binding.record_version == 0 {
        return Err(SecretVaultError::InvalidRecordVersion);
    }
    if binding.target.hash == [0; 32] {
        return Err(SecretVaultError::InvalidTargetBinding);
    }
    Ok(())
}

fn compare_bindings(
    actual: &SecretBinding,
    expected: &SecretBinding,
) -> Result<(), SecretVaultError> {
    if actual.store_uuid != expected.store_uuid {
        return Err(SecretVaultError::StoreUuidMismatch);
    }
    if actual.key_epoch != expected.key_epoch {
        return Err(SecretVaultError::KeyEpochMismatch);
    }
    if actual.secret_id != expected.secret_id {
        return Err(SecretVaultError::SecretIdMismatch);
    }
    if actual.record_version != expected.record_version {
        return Err(SecretVaultError::RecordVersionMismatch);
    }
    if actual.kind != expected.kind {
        return Err(SecretVaultError::SecretKindMismatch);
    }
    if actual.consumer != expected.consumer {
        return Err(SecretVaultError::ConsumerMismatch);
    }
    if actual.operation != expected.operation {
        return Err(SecretVaultError::OperationMismatch);
    }
    if actual.target != expected.target {
        return Err(SecretVaultError::TargetMismatch);
    }
    if actual.previous_record_hash != expected.previous_record_hash {
        return Err(SecretVaultError::PreviousRecordHashMismatch);
    }
    Ok(())
}

struct FixedBytes<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> FixedBytes<N> {
    const fn new() -> Self {
        Self {
            bytes: [0u8; N],
            len: 0,
        }
    }

    fn append(&mut self, value: &[u8]) -> Result<(), SecretVaultError> {
        let end = self
            .len
            .checked_add(value.len())
            .ok_or(SecretVaultError::EncodingOverflow)?;
        if end > N {
            return Err(SecretVaultError::EncodingOverflow);
        }
        self.bytes[self.len..end].copy_from_slice(value);
        self.len = end;
        Ok(())
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

fn build_record_aad(
    binding: &SecretBinding,
    plaintext_len: usize,
) -> Result<FixedBytes<RECORD_AAD_CAPACITY>, SecretVaultError> {
    let mut aad = FixedBytes::new();
    aad.append(RECORD_AAD_DOMAIN)?;
    aad.append(&SECRET_ENVELOPE_VERSION.to_le_bytes())?;
    aad.append(&binding.store_uuid)?;
    aad.append(&binding.key_epoch.to_le_bytes())?;
    aad.append(&binding.secret_id)?;
    aad.append(&[binding.kind as u8])?;
    aad.append(&[binding.consumer as u8])?;
    aad.append(&[binding.operation as u8])?;
    aad.append(&[binding.target.kind as u8])?;
    aad.append(&binding.target.hash)?;
    aad.append(&binding.record_version.to_le_bytes())?;
    aad.append(&(plaintext_len as u32).to_le_bytes())?;
    match binding.previous_record_hash {
        Some(hash) => {
            aad.append(&[1])?;
            aad.append(&hash)?;
        }
        None => {
            aad.append(&[0])?;
            aad.append(&[0; 32])?;
        }
    }
    Ok(aad)
}

struct DerivedKey {
    bytes: [u8; 32],
}

struct SecretScratch([u8; MAX_SECRET_LEN]);

impl Drop for SecretScratch {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for DerivedKey {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

fn derive_record_dek(
    vmk: &VaultMasterKey,
    binding: &SecretBinding,
) -> Result<DerivedKey, SecretVaultError> {
    let mut salt_hash = Sha256::new();
    salt_hash.update(RECORD_DEK_SALT_DOMAIN);
    salt_hash.update(binding.store_uuid);
    salt_hash.update(binding.key_epoch.to_le_bytes());
    let salt = digest_array(salt_hash);

    let mut info = FixedBytes::<DEK_INFO_CAPACITY>::new();
    info.append(RECORD_DEK_INFO_DOMAIN)?;
    info.append(&binding.secret_id)?;
    info.append(&binding.record_version.to_le_bytes())?;
    info.append(&[binding.kind as u8])?;

    let hkdf = Hkdf::<Sha256>::new(Some(&salt), &vmk.0);
    let mut dek = DerivedKey { bytes: [0u8; 32] };
    hkdf.expand(info.as_bytes(), &mut dek.bytes)
        .map_err(|_| SecretVaultError::KdfExpandFailed)?;
    Ok(dek)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryKekContext {
    pub store_uuid: [u8; 16],
    pub key_epoch: u64,
    pub wrapper_generation: u64,
    pub core_generation: u64,
    pub policy_id_sha256: [u8; 32],
}

/// Opaque recovery KEK; wrapper code consumes it only through exact helpers.
pub struct RecoveryKek {
    key: DerivedKey,
}

pub fn derive_recovery_kek(
    recovery_key: &RecoveryKey,
    context: &RecoveryKekContext,
) -> Result<RecoveryKek, SecretVaultError> {
    validate_recovery_context(context)?;
    let mut salt_hash = Sha256::new();
    salt_hash.update(RECOVERY_SALT_DOMAIN);
    salt_hash.update(context.store_uuid);
    salt_hash.update(context.key_epoch.to_le_bytes());
    let salt = digest_array(salt_hash);

    let mut info = FixedBytes::<RECOVERY_INFO_CAPACITY>::new();
    info.append(RECOVERY_INFO_DOMAIN)?;
    info.append(&context.wrapper_generation.to_le_bytes())?;
    info.append(&context.core_generation.to_le_bytes())?;
    info.append(&context.policy_id_sha256)?;

    let hkdf = Hkdf::<Sha256>::new(Some(&salt), &recovery_key.0);
    let mut kek = RecoveryKek {
        key: DerivedKey { bytes: [0u8; 32] },
    };
    hkdf.expand(info.as_bytes(), &mut kek.key.bytes)
        .map_err(|_| SecretVaultError::KdfExpandFailed)?;
    Ok(kek)
}

pub struct RecoveryWrapperAad(FixedBytes<RECOVERY_AAD_CAPACITY>);

impl RecoveryWrapperAad {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

pub fn build_recovery_wrapper_aad(
    context: &RecoveryKekContext,
    plaintext_len: u32,
) -> Result<RecoveryWrapperAad, SecretVaultError> {
    validate_recovery_context(context)?;
    if plaintext_len != 32 {
        return Err(SecretVaultError::InvalidRecoveryWrapperLength);
    }
    let mut aad = FixedBytes::new();
    aad.append(RECOVERY_WRAPPER_AAD_DOMAIN)?;
    aad.append(&context.store_uuid)?;
    aad.append(&context.key_epoch.to_le_bytes())?;
    aad.append(&context.wrapper_generation.to_le_bytes())?;
    aad.append(&context.core_generation.to_le_bytes())?;
    aad.append(&context.policy_id_sha256)?;
    aad.append(&plaintext_len.to_le_bytes())?;
    Ok(RecoveryWrapperAad(aad))
}

/// Persistent AES-GCM wrapper around exactly one 32-byte VMK.
///
/// It deliberately has no `Debug` implementation.
pub struct RecoveryVmkWrapperV1 {
    pub version: u16,
    pub context: RecoveryKekContext,
    pub plaintext_len: u32,
    pub nonce: [u8; 12],
    pub ciphertext: [u8; 32],
    pub tag: [u8; 16],
}

pub fn wrap_vmk_for_recovery(
    recovery_key: &RecoveryKey,
    vmk: &VaultMasterKey,
    context: RecoveryKekContext,
    fresh_nonce: FreshWrapperNonce<'_>,
) -> Result<RecoveryVmkWrapperV1, SecretVaultError> {
    validate_recovery_context(&context)?;
    if !fresh_nonce.retained_set_complete {
        return Err(SecretVaultError::NonceHistoryUncertain);
    }
    if fresh_nonce.retained_kek_nonces.contains(&fresh_nonce.nonce) {
        return Err(SecretVaultError::DuplicateNonce);
    }

    let kek = derive_recovery_kek(recovery_key, &context)?;
    let aad = build_recovery_wrapper_aad(&context, 32)?;
    let cipher = Aes256Gcm::new_from_slice(&kek.key.bytes)
        .map_err(|_| SecretVaultError::EncryptionFailed)?;
    let mut scratch = KeyScratch(vmk.0);
    let tag = cipher
        .encrypt_in_place_detached(
            Nonce::from_slice(&fresh_nonce.nonce),
            aad.as_bytes(),
            &mut scratch.0,
        )
        .map_err(|_| SecretVaultError::EncryptionFailed)?;
    let ciphertext = scratch.0;
    let mut tag_bytes = [0u8; 16];
    tag_bytes.copy_from_slice(&tag);
    Ok(RecoveryVmkWrapperV1 {
        version: RECOVERY_WRAPPER_VERSION,
        context,
        plaintext_len: 32,
        nonce: fresh_nonce.nonce,
        ciphertext,
        tag: tag_bytes,
    })
}

pub fn unwrap_vmk_from_recovery(
    recovery_key: &RecoveryKey,
    wrapper: &RecoveryVmkWrapperV1,
    expected: &RecoveryKekContext,
) -> Result<VaultMasterKey, SecretVaultError> {
    if wrapper.version != RECOVERY_WRAPPER_VERSION {
        return Err(SecretVaultError::UnsupportedRecoveryWrapperVersion);
    }
    validate_recovery_context(&wrapper.context)?;
    validate_recovery_context(expected)?;
    compare_recovery_contexts(&wrapper.context, expected)?;
    if wrapper.plaintext_len != 32 {
        return Err(SecretVaultError::InvalidRecoveryWrapperLength);
    }

    let kek = derive_recovery_kek(recovery_key, &wrapper.context)?;
    let aad = build_recovery_wrapper_aad(&wrapper.context, wrapper.plaintext_len)?;
    let cipher = Aes256Gcm::new_from_slice(&kek.key.bytes)
        .map_err(|_| SecretVaultError::AuthenticationFailed)?;
    let mut scratch = KeyScratch(wrapper.ciphertext);
    cipher
        .decrypt_in_place_detached(
            Nonce::from_slice(&wrapper.nonce),
            aad.as_bytes(),
            &mut scratch.0,
            Tag::from_slice(&wrapper.tag),
        )
        .map_err(|_| SecretVaultError::AuthenticationFailed)?;
    let vmk = VaultMasterKey(scratch.0);
    scratch.0.zeroize();
    Ok(vmk)
}

fn validate_recovery_context(context: &RecoveryKekContext) -> Result<(), SecretVaultError> {
    if context.store_uuid == [0; 16] {
        return Err(SecretVaultError::InvalidStoreUuid);
    }
    if context.key_epoch == 0 {
        return Err(SecretVaultError::InvalidKeyEpoch);
    }
    if context.wrapper_generation == 0 {
        return Err(SecretVaultError::WrapperGenerationMismatch);
    }
    if context.core_generation == 0 {
        return Err(SecretVaultError::CoreGenerationMismatch);
    }
    if context.policy_id_sha256 == [0; 32] {
        return Err(SecretVaultError::PolicyIdMismatch);
    }
    Ok(())
}

fn compare_recovery_contexts(
    actual: &RecoveryKekContext,
    expected: &RecoveryKekContext,
) -> Result<(), SecretVaultError> {
    if actual.store_uuid != expected.store_uuid {
        return Err(SecretVaultError::StoreUuidMismatch);
    }
    if actual.key_epoch != expected.key_epoch {
        return Err(SecretVaultError::KeyEpochMismatch);
    }
    if actual.wrapper_generation != expected.wrapper_generation {
        return Err(SecretVaultError::WrapperGenerationMismatch);
    }
    if actual.core_generation != expected.core_generation {
        return Err(SecretVaultError::CoreGenerationMismatch);
    }
    if actual.policy_id_sha256 != expected.policy_id_sha256 {
        return Err(SecretVaultError::PolicyIdMismatch);
    }
    Ok(())
}

struct KeyScratch([u8; 32]);

impl Drop for KeyScratch {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

fn digest_array(hash: Sha256) -> [u8; 32] {
    hash.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const STORE_UUID: [u8; 16] = *b"vault-store-test";
    const SECRET_ID: [u8; 16] = *b"wifi-secret-test";
    const NONCE: [u8; 12] = [0x21; 12];

    fn vmk(byte: u8) -> VaultMasterKey {
        let mut source = [byte; 32];
        let key = VaultMasterKey::take_from(&mut source);
        assert_eq!(source, [0; 32]);
        key
    }

    fn recovery_key(byte: u8) -> RecoveryKey {
        let mut source = [byte; 32];
        let key = RecoveryKey::take_from(&mut source);
        assert_eq!(source, [0; 32]);
        key
    }

    fn wifi_target(ssid: &[u8]) -> SecretTargetBinding {
        SecretTargetBinding::for_wifi_wpa2_psk_ccmp(ssid, [0x02, 0x11, 0x22, 0x33, 0x44, 0x55])
            .unwrap()
    }

    fn wifi_binding() -> SecretBinding {
        SecretBinding {
            store_uuid: STORE_UUID,
            key_epoch: 7,
            secret_id: SECRET_ID,
            record_version: 11,
            kind: SecretKind::WifiPassphrase,
            consumer: SecretConsumer::NativeWifiSupplicant,
            operation: SecretOperation::AssociateBoundBss,
            target: wifi_target(b"TestNetwork"),
            previous_record_hash: Some([0x44; 32]),
        }
    }

    fn plaintext(kind: SecretKind, value: &[u8]) -> SecretPlaintext {
        let mut source = [0u8; MAX_SECRET_LEN];
        source[..value.len()].copy_from_slice(value);
        SecretPlaintext::take_from(kind, &mut source[..value.len()]).unwrap()
    }

    #[test]
    fn named_wifi_consumer_writes_only_the_pinned_supplicant_command() {
        let mut from_secret = [0u8; 160];
        let mut direct = [0u8; 160];
        let expected = crate::marvell_wifi_supplicant::build_supplicant_pmk_set(
            9,
            [0x02, 0x11, 0x22, 0x33, 0x44, 0x55],
            b"TestNetwork",
            b"test-passphrase",
            &mut direct,
        )
        .unwrap();
        let actual = plaintext(SecretKind::WifiPassphrase, b"test-passphrase")
            .into_wpa2_psk_ccmp_pmk_set(
                9,
                [0x02, 0x11, 0x22, 0x33, 0x44, 0x55],
                b"TestNetwork",
                &mut from_secret,
            )
            .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(&from_secret[..actual], &direct[..expected]);
    }

    #[test]
    fn named_openai_consumer_never_exposes_a_key_or_writes_for_the_wrong_kind() {
        let mut bytes = [0u8; 64];
        let initial_len = bytes.len();
        let mut writer = &mut bytes[..];
        assert_eq!(
            plaintext(SecretKind::WifiPassphrase, b"test-passphrase")
                .into_openai_authorization_header(&mut writer),
            Err(SecretConsumerError::WrongKind {
                expected: SecretKind::ProviderApiKey,
                actual: SecretKind::WifiPassphrase,
            })
        );
        assert_eq!(writer.len(), initial_len);

        let mut writer = &mut bytes[..];
        plaintext(SecretKind::ProviderApiKey, b"test-provider-key")
            .into_openai_authorization_header(&mut writer)
            .unwrap();
        let written = initial_len - writer.len();
        assert_eq!(
            &bytes[..written],
            b"Authorization: Bearer test-provider-key\r\n"
        );
    }

    fn encrypted_wifi() -> (VaultMasterKey, SecretEnvelopeV1, SecretBinding) {
        let vmk = vmk(0x17);
        let binding = wifi_binding();
        let envelope = encrypt_secret(
            &vmk,
            plaintext(SecretKind::WifiPassphrase, b"test-passphrase"),
            binding,
            Some(10),
            FreshRecordNonce {
                nonce: NONCE,
                retained_epoch_nonces: &[[0x20; 12]],
                retained_set_complete: true,
            },
        )
        .unwrap();
        (vmk, envelope, binding)
    }

    #[test]
    fn aes256_gcm_matches_nist_known_answer() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let mut block = [0u8; 16];
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let tag = cipher
            .encrypt_in_place_detached(Nonce::from_slice(&nonce), b"", &mut block)
            .unwrap();
        assert_eq!(
            block,
            [
                0xce, 0xa7, 0x40, 0x3d, 0x4d, 0x60, 0x6b, 0x6e, 0x07, 0x4e, 0xc5, 0xd3, 0xba, 0xf3,
                0x9d, 0x18,
            ]
        );
        assert_eq!(
            tag.as_slice(),
            &[
                0xd0, 0xd1, 0xc8, 0xa7, 0x99, 0x99, 0x6b, 0xf0, 0x26, 0x5b, 0x98, 0xb5, 0xd4, 0x8a,
                0xb9, 0x19,
            ]
        );
    }

    #[test]
    fn hkdf_sha256_matches_rfc5869_case_one() {
        let ikm = [0x0b; 22];
        let salt = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
        ];
        let info = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9];
        let mut okm = [0u8; 42];
        Hkdf::<Sha256>::new(Some(&salt), &ikm)
            .expand(&info, &mut okm)
            .unwrap();
        assert_eq!(
            okm,
            [
                0x3c, 0xb2, 0x5f, 0x25, 0xfa, 0xac, 0xd5, 0x7a, 0x90, 0x43, 0x4f, 0x64, 0xd0, 0x36,
                0x2f, 0x2a, 0x2d, 0x2d, 0x0a, 0x90, 0xcf, 0x1a, 0x5a, 0x4c, 0x5d, 0xb0, 0x2d, 0x56,
                0xec, 0xc4, 0xc5, 0xbf, 0x34, 0x00, 0x72, 0x08, 0xd5, 0xb8, 0x87, 0x18, 0x58, 0x65,
            ]
        );
    }

    #[test]
    fn round_trip_uses_exact_binding() {
        let (vmk, envelope, binding) = encrypted_wifi();
        let plaintext = decrypt_secret(&vmk, &envelope, &binding, 11).unwrap();
        assert_eq!(plaintext.kind(), SecretKind::WifiPassphrase);
        assert_eq!(plaintext.as_bytes(), b"test-passphrase");
    }

    #[test]
    fn tag_ciphertext_and_aad_tampering_deny() {
        let (vmk, mut envelope, binding) = encrypted_wifi();
        envelope.tag[0] ^= 1;
        assert_eq!(
            decrypt_secret(&vmk, &envelope, &binding, 11).err().unwrap(),
            SecretVaultError::AuthenticationFailed
        );

        let (vmk, mut envelope, binding) = encrypted_wifi();
        envelope.ciphertext[0] ^= 1;
        assert_eq!(
            decrypt_secret(&vmk, &envelope, &binding, 11).err().unwrap(),
            SecretVaultError::AuthenticationFailed
        );

        let (vmk, mut envelope, binding) = encrypted_wifi();
        envelope.aad_hash[0] ^= 1;
        assert_eq!(
            decrypt_secret(&vmk, &envelope, &binding, 11).err().unwrap(),
            SecretVaultError::AadHashMismatch
        );
    }

    #[test]
    fn every_wrong_binding_has_a_stable_typed_denial() {
        let cases: &[(fn(&mut SecretBinding), SecretVaultError)] = &[
            (
                |b| b.store_uuid[0] ^= 1,
                SecretVaultError::StoreUuidMismatch,
            ),
            (|b| b.key_epoch += 1, SecretVaultError::KeyEpochMismatch),
            (|b| b.secret_id[0] ^= 1, SecretVaultError::SecretIdMismatch),
            (
                |b| b.record_version += 1,
                SecretVaultError::RecordVersionMismatch,
            ),
            (
                |b| b.kind = SecretKind::ProviderApiKey,
                SecretVaultError::SecretKindMismatch,
            ),
            (
                |b| b.consumer = SecretConsumer::OpenAiDirect,
                SecretVaultError::ConsumerMismatch,
            ),
            (
                |b| b.operation = SecretOperation::RequestExactHost,
                SecretVaultError::OperationMismatch,
            ),
            (
                |b| b.target = wifi_target(b"OtherNetwork"),
                SecretVaultError::TargetMismatch,
            ),
            (
                |b| b.previous_record_hash = None,
                SecretVaultError::PreviousRecordHashMismatch,
            ),
        ];

        for (mutate, expected_error) in cases {
            let (vmk, envelope, mut expected) = encrypted_wifi();
            mutate(&mut expected);
            assert_eq!(
                decrypt_secret(&vmk, &envelope, &expected, 11)
                    .err()
                    .unwrap(),
                *expected_error
            );
        }
    }

    #[test]
    fn duplicate_uncertain_and_stale_nonce_version_states_deny() {
        let vmk = vmk(0x17);
        let binding = wifi_binding();
        let make_plaintext = || plaintext(SecretKind::WifiPassphrase, b"test-passphrase");

        assert_eq!(
            encrypt_secret(
                &vmk,
                make_plaintext(),
                binding,
                Some(10),
                FreshRecordNonce {
                    nonce: NONCE,
                    retained_epoch_nonces: &[NONCE],
                    retained_set_complete: true,
                },
            )
            .err()
            .unwrap(),
            SecretVaultError::DuplicateNonce
        );
        assert_eq!(
            encrypt_secret(
                &vmk,
                make_plaintext(),
                binding,
                Some(10),
                FreshRecordNonce {
                    nonce: NONCE,
                    retained_epoch_nonces: &[],
                    retained_set_complete: false,
                },
            )
            .err()
            .unwrap(),
            SecretVaultError::NonceHistoryUncertain
        );

        let mut stale = binding;
        stale.record_version = 10;
        assert_eq!(
            encrypt_secret(
                &vmk,
                make_plaintext(),
                stale,
                Some(10),
                FreshRecordNonce {
                    nonce: NONCE,
                    retained_epoch_nonces: &[],
                    retained_set_complete: true,
                },
            )
            .err()
            .unwrap(),
            SecretVaultError::StaleRecordVersion
        );
    }

    #[test]
    fn stale_and_unknown_envelope_versions_deny_before_use() {
        let (vmk, mut envelope, binding) = encrypted_wifi();
        assert_eq!(
            decrypt_secret(&vmk, &envelope, &binding, 12).err().unwrap(),
            SecretVaultError::StaleRecordVersion
        );
        envelope.version = 2;
        assert_eq!(
            decrypt_secret(&vmk, &envelope, &binding, 11).err().unwrap(),
            SecretVaultError::UnsupportedEnvelopeVersion
        );
    }

    #[test]
    fn exact_secret_length_bounds_and_source_zeroization_hold() {
        let mut wifi_max = [b'w'; MAX_WIFI_PASSPHRASE_LEN];
        let wifi = SecretPlaintext::take_from(SecretKind::WifiPassphrase, &mut wifi_max).unwrap();
        assert_eq!(wifi.len(), MAX_WIFI_PASSPHRASE_LEN);
        assert_eq!(wifi_max, [0; MAX_WIFI_PASSPHRASE_LEN]);

        let mut wifi_too_long = [b'w'; MAX_WIFI_PASSPHRASE_LEN + 1];
        assert!(matches!(
            SecretPlaintext::take_from(SecretKind::WifiPassphrase, &mut wifi_too_long),
            Err(SecretVaultError::SecretTooLong { .. })
        ));
        assert_eq!(wifi_too_long, [0; MAX_WIFI_PASSPHRASE_LEN + 1]);

        let mut provider_max = [b'p'; MAX_PROVIDER_API_KEY_LEN];
        let provider =
            SecretPlaintext::take_from(SecretKind::ProviderApiKey, &mut provider_max).unwrap();
        assert_eq!(provider.len(), MAX_PROVIDER_API_KEY_LEN);
        assert_eq!(provider_max, [0; MAX_PROVIDER_API_KEY_LEN]);
    }

    #[test]
    fn plaintext_and_key_owners_zeroize_on_drop() {
        let mut plaintext = plaintext(SecretKind::WifiPassphrase, b"temporary-value");
        plaintext.zeroize();
        assert_eq!(plaintext.bytes, [0; MAX_SECRET_LEN]);
        assert_eq!(plaintext.len, 0);

        let mut vmk = vmk(0x5a);
        vmk.zeroize();
        assert_eq!(vmk.0, [0; 32]);

        let mut recovery = recovery_key(0xa5);
        recovery.zeroize();
        assert_eq!(recovery.0, [0; 32]);
    }

    fn recovery_context() -> RecoveryKekContext {
        RecoveryKekContext {
            store_uuid: STORE_UUID,
            key_epoch: 7,
            wrapper_generation: 9,
            core_generation: 12,
            policy_id_sha256: [0x33; 32],
        }
    }

    fn recovery_wrapper() -> (RecoveryKey, RecoveryVmkWrapperV1, RecoveryKekContext) {
        let recovery = recovery_key(0x42);
        let vmk = vmk(0x17);
        let context = recovery_context();
        let wrapper = wrap_vmk_for_recovery(
            &recovery,
            &vmk,
            context,
            FreshWrapperNonce {
                nonce: [0x31; 12],
                retained_kek_nonces: &[[0x30; 12]],
                retained_set_complete: true,
            },
        )
        .unwrap();
        (recovery, wrapper, context)
    }

    #[test]
    fn recovery_wrapper_round_trip_and_tamper_denial_hold() {
        let (recovery, wrapper, context) = recovery_wrapper();
        let unwrapped = unwrap_vmk_from_recovery(&recovery, &wrapper, &context).unwrap();
        assert_eq!(unwrapped.0, [0x17; 32]);

        let (recovery, mut wrapper, context) = recovery_wrapper();
        wrapper.tag[0] ^= 1;
        assert_eq!(
            unwrap_vmk_from_recovery(&recovery, &wrapper, &context)
                .err()
                .unwrap(),
            SecretVaultError::AuthenticationFailed
        );
    }

    #[test]
    fn recovery_wrapper_context_and_nonce_denials_are_typed() {
        let (recovery, wrapper, context) = recovery_wrapper();
        let cases: &[(fn(&mut RecoveryKekContext), SecretVaultError)] = &[
            (
                |c| c.store_uuid[0] ^= 1,
                SecretVaultError::StoreUuidMismatch,
            ),
            (|c| c.key_epoch += 1, SecretVaultError::KeyEpochMismatch),
            (
                |c| c.wrapper_generation += 1,
                SecretVaultError::WrapperGenerationMismatch,
            ),
            (
                |c| c.core_generation += 1,
                SecretVaultError::CoreGenerationMismatch,
            ),
            (
                |c| c.policy_id_sha256[0] ^= 1,
                SecretVaultError::PolicyIdMismatch,
            ),
        ];
        for (mutate, expected) in cases {
            let mut wrong = context;
            mutate(&mut wrong);
            assert_eq!(
                unwrap_vmk_from_recovery(&recovery, &wrapper, &wrong)
                    .err()
                    .unwrap(),
                *expected
            );
        }

        let recovery = recovery_key(0x42);
        let vmk = vmk(0x17);
        assert_eq!(
            wrap_vmk_for_recovery(
                &recovery,
                &vmk,
                context,
                FreshWrapperNonce {
                    nonce: [0x31; 12],
                    retained_kek_nonces: &[[0x31; 12]],
                    retained_set_complete: true,
                },
            )
            .err()
            .unwrap(),
            SecretVaultError::DuplicateNonce
        );
        assert_eq!(
            wrap_vmk_for_recovery(
                &recovery,
                &vmk,
                context,
                FreshWrapperNonce {
                    nonce: [0x31; 12],
                    retained_kek_nonces: &[],
                    retained_set_complete: false,
                },
            )
            .err()
            .unwrap(),
            SecretVaultError::NonceHistoryUncertain
        );
    }

    #[test]
    fn recovery_kek_and_aad_follow_adr_0012_byte_contract() {
        let context = recovery_context();
        let key = recovery_key(0x42);
        let kek = derive_recovery_kek(&key, &context).unwrap();
        assert_eq!(
            kek.key.bytes,
            [
                0x95, 0xf4, 0x99, 0x90, 0xd0, 0x6e, 0x6a, 0x0d, 0xc5, 0x5e, 0x77, 0x83, 0x9c, 0x2b,
                0xa4, 0x07, 0x39, 0xff, 0x01, 0x67, 0x8c, 0x92, 0x76, 0x71, 0xaf, 0x7d, 0x99, 0x05,
                0x18, 0xaa, 0x19, 0x26,
            ]
        );

        let aad = build_recovery_wrapper_aad(&context, 32).unwrap();
        let mut expected = std::vec::Vec::new();
        expected.extend_from_slice(RECOVERY_WRAPPER_AAD_DOMAIN);
        expected.extend_from_slice(&context.store_uuid);
        expected.extend_from_slice(&context.key_epoch.to_le_bytes());
        expected.extend_from_slice(&context.wrapper_generation.to_le_bytes());
        expected.extend_from_slice(&context.core_generation.to_le_bytes());
        expected.extend_from_slice(&context.policy_id_sha256);
        expected.extend_from_slice(&32u32.to_le_bytes());
        assert_eq!(aad.as_bytes(), expected);
    }
}
