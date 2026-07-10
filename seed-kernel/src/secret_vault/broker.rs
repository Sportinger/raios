//! Exact-purpose Secret Vault broker.
//!
//! This module owns Vault operation policy, but not media I/O.  It accepts
//! only replay-verified typed records, prepares only the two fixed record
//! kinds, and issues only named one-use leases.  A future native-consumer
//! bridge must consume those leases without adding a generic plaintext API.

use alloc::vec::Vec;

use raios_core::{
    core_policy::{VerifiedCorePolicy, VerifiedSafeLastGoodCorePolicy},
    scoped_secret_use::{
        authorize_secret_use, SecretUseAuthorization, SecretUseDenial, SecretUseEvidence,
        SecretUseRequest,
    },
    secret_vault::{
        decrypt_secret, encrypt_secret, FreshRecordNonce, RecoveryKey, SecretBinding,
        SecretConsumer, SecretConsumerError, SecretEnvelopeV1, SecretKind, SecretOperation,
        SecretPlaintext, SecretTargetBinding, SecretVaultError, VaultMasterKey, OPENAI_API_HOST,
    },
    structured_store::{CommittedRecord, StoreIdentity},
};

use crate::entropy;

#[cfg(test)]
use raios_core::scoped_secret_use::SecretBootScope;

use super::{
    audit::{DurableProviderUseReceipt, DurableWifiUseReceipt},
    keyring::{ApprovedCorePolicy, KeyringDenied, RecoveryWrapperSlot, RecoveryWrapperSlots},
    store::{
        reconstruct_retained_nonce_metadata, DecodedVaultRecord, ReplayHistoryCompleteness,
        ReplayVerifiedVaultRecord, RetainedNonceMetadata, VaultRecordId, VaultStoreDenied,
    },
};

/// A one-operation mutation permit.  Its constructor is visible only to the
/// `secret_vault` facade; UI, serial, Wasm, and native consumers cannot mint
/// one by naming a trusted-looking origin.
pub(crate) struct VaultMutation {
    _facade_only: (),
}

impl VaultMutation {
    pub(super) const fn genesis_secure_overlay() -> Self {
        Self { _facade_only: () }
    }

    pub(super) const fn explicit_recovery() -> Self {
        Self { _facade_only: () }
    }
}

/// An opaque use permit minted only by moving a durable provider-audit
/// receipt into the broker. The broker never edits this evidence: it rejects
/// a permit whose record or key epoch is no longer current.
pub(crate) struct VaultUseAuthority {
    evidence: SecretUseEvidence,
}

impl VaultUseAuthority {
    fn from_durable_provider_receipt(receipt: DurableProviderUseReceipt) -> Self {
        Self {
            evidence: receipt.into_secret_use_evidence(),
        }
    }

    fn from_durable_wifi_receipt(receipt: DurableWifiUseReceipt) -> (Self, [u8; 32]) {
        let (evidence, target_binding_sha256) = receipt.into_secret_use_evidence();
        (Self { evidence }, target_binding_sha256)
    }
}

/// A complete, transaction-ordered Vault replay.  The only constructor is at
/// the `secret_vault` composition boundary, where the real structured-store
/// replay supplies every committed Vault record.  It reconstructs the nonce
/// history before a broker can prepare any new encryption.
pub(crate) struct VaultCompleteReplay<'a> {
    identity: StoreIdentity,
    records: &'a [CommittedRecord],
    retained_nonces: RetainedNonceMetadata,
}

impl<'a> VaultCompleteReplay<'a> {
    pub(super) fn from_complete_history(
        identity: StoreIdentity,
        records: &'a [CommittedRecord],
    ) -> Result<Self, VaultBrokerDenied> {
        let mut verified = Vec::new();
        verified
            .try_reserve_exact(records.len())
            .map_err(|_| VaultBrokerDenied::RetainedNonceAllocation)?;
        for record in records {
            verified.push(
                ReplayVerifiedVaultRecord::from_committed(identity, record)
                    .map_err(VaultBrokerDenied::Store)?,
            );
        }
        let retained_nonces =
            reconstruct_retained_nonce_metadata(&verified, ReplayHistoryCompleteness::Complete)
                .map_err(VaultBrokerDenied::Store)?;
        Ok(Self {
            identity,
            records,
            retained_nonces,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VaultLockStatus {
    Locked,
    UnlockedFromRecovery(RecoveryWrapperSlot),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VaultSecretStatus {
    Missing,
    Available { record_version: u64, key_epoch: u64 },
    Forgotten { record_version: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VaultStoreStatus {
    Unbound,
    Bound { generation: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VaultCorePolicyStatus {
    Unbound,
    Bound {
        core_generation: u64,
        policy_id_sha256: [u8; 32],
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VaultBrokerStatus {
    pub(crate) store: VaultStoreStatus,
    pub(crate) core_policy: VaultCorePolicyStatus,
    pub(crate) lock: VaultLockStatus,
    pub(crate) wifi: VaultSecretStatus,
    pub(crate) provider: VaultSecretStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VaultBrokerDenied {
    StoreIdentityInvalid,
    StoreIdentityMismatch,
    RecordNotSecret,
    ReplayOutOfOrder,
    EnvelopeSlotMismatch,
    SecretIdMismatch,
    Locked,
    AlreadyUnlocked,
    SecretMissing,
    SecretForgotten,
    KeyEpochInvalid,
    KeyEpochMismatch,
    CompleteReplayRequired,
    CorePolicyMissing,
    CorePolicyIdentityMismatch,
    RecordVersionExhausted,
    EntropyNotReady,
    RetainedNonceAllocation,
    Store(VaultStoreDenied),
    Keyring(KeyringDenied),
    Crypto(SecretVaultError),
    Use(SecretUseDenial),
    AuditlessUseUnexpectedlyAuthorized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContainedQemuCryptoProof;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContainedQemuCryptoProofStep {
    WifiTag,
    WifiAad,
    WifiKind,
    WifiConsumer,
    WifiOperation,
    WifiTarget,
    WifiDuplicateNonce,
    ProviderTag,
    ProviderAad,
    ProviderKind,
    ProviderConsumer,
    ProviderOperation,
    ProviderTarget,
    ProviderDuplicateNonce,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContainedQemuCryptoProofDenied {
    Locked,
    CompleteReplayRequired,
    WifiMissing,
    ProviderMissing,
    KeyEpochMismatch,
    EnvelopeAuthenticationFailed {
        kind: SecretKind,
        actual: SecretVaultError,
    },
    RetainedNonceAllocation,
    RetainedNonceMissing(SecretKind),
    RecordVersionExhausted(SecretKind),
    UnexpectedCryptoResult {
        step: ContainedQemuCryptoProofStep,
        expected: SecretVaultError,
        actual: Option<SecretVaultError>,
    },
}

/// A typed ciphertext handoff for the structured-store transaction layer.  It
/// contains no media operation and becomes active only after replay verifies
/// the resulting committed record.
pub(crate) struct WifiSecretWrite {
    envelope: SecretEnvelopeV1,
}

impl WifiSecretWrite {
    pub(crate) fn into_store_envelope(self) -> SecretEnvelopeV1 {
        self.envelope
    }
}

/// A typed ciphertext handoff for the fixed OpenAI credential slot.
pub(crate) struct ProviderSecretWrite {
    envelope: SecretEnvelopeV1,
}

impl ProviderSecretWrite {
    pub(crate) fn into_store_envelope(self) -> SecretEnvelopeV1 {
        self.envelope
    }
}

/// A tombstone intent for the fixed WiFi slot.  The structured-store layer
/// remains solely responsible for append, flush, readback, and replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WifiSecretForget {
    pub(crate) record_version: u64,
    pub(crate) previous_record_hash: [u8; 32],
}

impl WifiSecretForget {
    pub(crate) const fn record_id(self) -> VaultRecordId {
        VaultRecordId::WifiPassphrase
    }
}

/// A tombstone intent for the fixed provider slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProviderSecretForget {
    pub(crate) record_version: u64,
    pub(crate) previous_record_hash: [u8; 32],
}

impl ProviderSecretForget {
    pub(crate) const fn record_id(self) -> VaultRecordId {
        VaultRecordId::ProviderApiKey
    }
}

/// One owned plaintext value, bound to the native WiFi supplicant.  Dropping
/// an unused lease zeroizes its plaintext through `SecretPlaintext`.
pub(crate) struct WifiSecretLease {
    plaintext: SecretPlaintext,
    _authorization: SecretUseAuthorization,
}

impl WifiSecretLease {
    /// Consumes this one-use lease into the one NXP WPA2 supplicant command.
    /// No passphrase slice is returned to the caller.
    pub(crate) fn into_wpa2_psk_ccmp_pmk_set(
        self,
        seq: u16,
        bssid: [u8; 6],
        ssid: &[u8],
        out: &mut [u8],
    ) -> Result<usize, SecretConsumerError> {
        self.plaintext
            .into_wpa2_psk_ccmp_pmk_set(seq, bssid, ssid, out)
    }
}

/// One owned plaintext value, bound to `svc.provider.openai_direct` and the
/// exact OpenAI host.  It cannot be cloned or inspected by the broker caller.
pub(crate) struct ProviderSecretLease {
    plaintext: SecretPlaintext,
    _authorization: SecretUseAuthorization,
}

impl ProviderSecretLease {
    /// Consumes this one-use lease into the exact direct-OpenAI authorization
    /// header. No key bytes, key length, or mutable key buffer is returned.
    pub(crate) fn into_openai_authorization_header<W: embedded_io::Write>(
        self,
        writer: &mut W,
    ) -> Result<(), SecretConsumerError> {
        self.plaintext.into_openai_authorization_header(writer)
    }
}

enum SlotState {
    Missing,
    Present {
        envelope: SecretEnvelopeV1,
        commit_hash: [u8; 32],
    },
    Tombstoned {
        record_version: u64,
        commit_hash: [u8; 32],
    },
}

impl SlotState {
    const fn missing() -> Self {
        Self::Missing
    }

    fn status(&self) -> VaultSecretStatus {
        match self {
            Self::Missing => VaultSecretStatus::Missing,
            Self::Present { envelope, .. } => VaultSecretStatus::Available {
                record_version: envelope.binding.record_version,
                key_epoch: envelope.binding.key_epoch,
            },
            Self::Tombstoned { record_version, .. } => VaultSecretStatus::Forgotten {
                record_version: *record_version,
            },
        }
    }

    fn record_version(&self) -> Option<u64> {
        match self {
            Self::Missing => None,
            Self::Present { envelope, .. } => Some(envelope.binding.record_version),
            Self::Tombstoned { record_version, .. } => Some(*record_version),
        }
    }

    fn previous_record_hash(&self) -> Option<[u8; 32]> {
        match self {
            Self::Missing => None,
            Self::Present { commit_hash, .. } | Self::Tombstoned { commit_hash, .. } => {
                Some(*commit_hash)
            }
        }
    }

    fn present(&self) -> Result<&SecretEnvelopeV1, VaultBrokerDenied> {
        match self {
            Self::Present { envelope, .. } => Ok(envelope),
            Self::Missing => Err(VaultBrokerDenied::SecretMissing),
            Self::Tombstoned { .. } => Err(VaultBrokerDenied::SecretForgotten),
        }
    }

    fn install_present(
        &mut self,
        envelope: SecretEnvelopeV1,
        commit_hash: [u8; 32],
    ) -> Result<(), VaultBrokerDenied> {
        self.require_newer_version(envelope.binding.record_version)?;
        *self = Self::Present {
            envelope,
            commit_hash,
        };
        Ok(())
    }

    fn install_tombstone(
        &mut self,
        record_version: u64,
        commit_hash: [u8; 32],
    ) -> Result<(), VaultBrokerDenied> {
        self.require_newer_version(record_version)?;
        *self = Self::Tombstoned {
            record_version,
            commit_hash,
        };
        Ok(())
    }

    fn require_newer_version(&self, incoming: u64) -> Result<(), VaultBrokerDenied> {
        if incoming == 0 || self.record_version().is_some_and(|known| incoming <= known) {
            return Err(VaultBrokerDenied::ReplayOutOfOrder);
        }
        Ok(())
    }

    fn next_record_version(&self) -> Result<u64, VaultBrokerDenied> {
        self.record_version().map_or(Ok(1), |version| {
            version
                .checked_add(1)
                .ok_or(VaultBrokerDenied::RecordVersionExhausted)
        })
    }
}

/// Core-owned Vault state.  It holds the VMK only after a verified recovery
/// unlock and does not own any raw device, partition, or write fallback.
pub(crate) struct VaultBroker {
    identity: Option<StoreIdentity>,
    approved_core_policy: Option<ApprovedCorePolicy>,
    vmk: Option<VaultMasterKey>,
    unlocked_key_epoch: Option<u64>,
    retained_nonces: Option<RetainedNonceMetadata>,
    lock_status: VaultLockStatus,
    wifi: SlotState,
    provider: SlotState,
}

impl VaultBroker {
    pub(crate) const fn new() -> Self {
        Self {
            identity: None,
            approved_core_policy: None,
            vmk: None,
            unlocked_key_epoch: None,
            retained_nonces: None,
            lock_status: VaultLockStatus::Locked,
            wifi: SlotState::missing(),
            provider: SlotState::missing(),
        }
    }

    pub(crate) fn status(&self) -> VaultBrokerStatus {
        VaultBrokerStatus {
            store: self.identity.map_or(VaultStoreStatus::Unbound, |identity| {
                VaultStoreStatus::Bound {
                    generation: identity.generation,
                }
            }),
            core_policy: self.approved_core_policy.map_or(
                VaultCorePolicyStatus::Unbound,
                |policy| VaultCorePolicyStatus::Bound {
                    core_generation: policy.core_generation(),
                    policy_id_sha256: policy.policy_id_sha256(),
                },
            ),
            lock: self.lock_status,
            wifi: self.wifi.status(),
            provider: self.provider.status(),
        }
    }

    /// Exercises fixed fail-closed mutations against both real replayed
    /// envelopes. The helper returns no plaintext and accepts no fault selector.
    pub(crate) fn prove_exact_contained_qemu_crypto_fail_closed(
        &self,
    ) -> Result<ContainedQemuCryptoProof, ContainedQemuCryptoProofDenied> {
        let vmk = self
            .vmk
            .as_ref()
            .ok_or(ContainedQemuCryptoProofDenied::Locked)?;
        let retained = self
            .retained_nonces
            .as_ref()
            .ok_or(ContainedQemuCryptoProofDenied::CompleteReplayRequired)?;
        let wifi = self
            .wifi
            .present()
            .map_err(|_| ContainedQemuCryptoProofDenied::WifiMissing)?;
        let provider = self
            .provider
            .present()
            .map_err(|_| ContainedQemuCryptoProofDenied::ProviderMissing)?;
        if self.unlocked_key_epoch != Some(wifi.binding.key_epoch)
            || self.unlocked_key_epoch != Some(provider.binding.key_epoch)
        {
            return Err(ContainedQemuCryptoProofDenied::KeyEpochMismatch);
        }

        prove_replayed_envelope_fail_closed(
            vmk,
            retained,
            wifi,
            provider.binding.target,
            ContainedQemuCryptoProofStep::WifiTag,
            ContainedQemuCryptoProofStep::WifiAad,
            ContainedQemuCryptoProofStep::WifiKind,
            ContainedQemuCryptoProofStep::WifiConsumer,
            ContainedQemuCryptoProofStep::WifiOperation,
            ContainedQemuCryptoProofStep::WifiTarget,
            ContainedQemuCryptoProofStep::WifiDuplicateNonce,
        )?;
        prove_replayed_envelope_fail_closed(
            vmk,
            retained,
            provider,
            wifi.binding.target,
            ContainedQemuCryptoProofStep::ProviderTag,
            ContainedQemuCryptoProofStep::ProviderAad,
            ContainedQemuCryptoProofStep::ProviderKind,
            ContainedQemuCryptoProofStep::ProviderConsumer,
            ContainedQemuCryptoProofStep::ProviderOperation,
            ContainedQemuCryptoProofStep::ProviderTarget,
            ContainedQemuCryptoProofStep::ProviderDuplicateNonce,
        )?;
        Ok(ContainedQemuCryptoProof)
    }

    /// Binds this broker to an already mounted, identity-checked structured
    /// store.  It never formats, selects, or opens a storage device.
    pub(crate) fn bind_store(&mut self, identity: StoreIdentity) -> Result<(), VaultBrokerDenied> {
        validate_identity(identity)?;
        if self.identity.is_some_and(|bound| bound != identity) {
            return Err(VaultBrokerDenied::StoreIdentityMismatch);
        }
        self.identity = Some(identity);
        Ok(())
    }

    /// Installs one complete ordered structured-store replay after the facade
    /// proved its global transaction order and every per-slot predecessor
    /// chain.  This is ciphertext-only state; it does not unlock or decrypt a
    /// secret.
    pub(crate) fn load_complete_replay(
        &mut self,
        replay: VaultCompleteReplay<'_>,
    ) -> Result<(), VaultBrokerDenied> {
        self.bind_store(replay.identity)?;
        let mut wifi = SlotState::missing();
        let mut provider = SlotState::missing();
        for record in replay.records {
            install_replay_record(replay.identity, record, &mut wifi, &mut provider)?;
        }
        self.wifi = wifi;
        self.provider = provider;
        self.retained_nonces = Some(replay.retained_nonces);
        Ok(())
    }

    /// Retains only the identity copied from the opaque verified Core Policy.
    /// This does not open a wrapper, decrypt a secret, or create use authority.
    pub(crate) fn bind_verified_core_policy(
        &mut self,
        verified: &VerifiedCorePolicy,
    ) -> Result<(), VaultBrokerDenied> {
        self.bind_approved_core_policy(ApprovedCorePolicy::from_verified(verified))
    }

    pub(crate) fn bind_verified_safe_last_good_core_policy(
        &mut self,
        verified: &VerifiedSafeLastGoodCorePolicy,
    ) -> Result<(), VaultBrokerDenied> {
        self.bind_approved_core_policy(ApprovedCorePolicy::from_safe_last_good(verified))
    }

    fn bind_approved_core_policy(
        &mut self,
        policy: ApprovedCorePolicy,
    ) -> Result<(), VaultBrokerDenied> {
        if self
            .approved_core_policy
            .is_some_and(|bound| bound != policy)
        {
            return Err(VaultBrokerDenied::CorePolicyIdentityMismatch);
        }
        self.approved_core_policy = Some(policy);
        Ok(())
    }

    /// Opens the non-exportable VMK only through a readback-verified recovery
    /// wrapper and exact approved policy.  TPM auto-unlock is intentionally
    /// absent from this interface.
    pub(crate) fn unlock_with_recovery_key(
        &mut self,
        recovery_key: &RecoveryKey,
        wrappers: &RecoveryWrapperSlots,
    ) -> Result<(), VaultBrokerDenied> {
        if self.vmk.is_some() {
            return Err(VaultBrokerDenied::AlreadyUnlocked);
        }
        let policy = self
            .approved_core_policy
            .ok_or(VaultBrokerDenied::CorePolicyMissing)?;
        let identity = self
            .identity
            .ok_or(VaultBrokerDenied::StoreIdentityInvalid)?;
        let (vmk, selected) = wrappers
            .unlock_with_recovery(recovery_key, policy)
            .map_err(VaultBrokerDenied::Keyring)?;
        let wrapper = match selected {
            RecoveryWrapperSlot::Current => wrappers.current_wrapper(),
            RecoveryWrapperSlot::LastGood => {
                wrappers
                    .last_good_wrapper()
                    .ok_or(VaultBrokerDenied::Keyring(
                        KeyringDenied::NoMatchingRecoveryWrapper,
                    ))?
            }
        };
        if wrapper.context.store_uuid != identity.store_uuid {
            return Err(VaultBrokerDenied::StoreIdentityMismatch);
        }
        if wrapper.context.key_epoch == 0 {
            return Err(VaultBrokerDenied::KeyEpochInvalid);
        }
        self.vmk = Some(vmk);
        self.unlocked_key_epoch = Some(wrapper.context.key_epoch);
        self.lock_status = VaultLockStatus::UnlockedFromRecovery(selected);
        Ok(())
    }

    /// Accepts only an already captured WiFi plaintext and binds it to one
    /// WPA2-PSK/CCMP BSS.  The result still requires the typed structured-store
    /// append/readback/replay path before this broker treats it as available.
    pub(crate) fn save_or_replace_wifi(
        &self,
        _mutation: VaultMutation,
        ssid: &[u8],
        bssid: [u8; 6],
        plaintext: SecretPlaintext,
    ) -> Result<WifiSecretWrite, VaultBrokerDenied> {
        let target = SecretTargetBinding::for_wifi_wpa2_psk_ccmp(ssid, bssid)
            .map_err(VaultBrokerDenied::Crypto)?;
        let envelope = self.prepare_write(
            VaultRecordId::WifiPassphrase,
            SecretKind::WifiPassphrase,
            SecretConsumer::NativeWifiSupplicant,
            SecretOperation::AssociateBoundBss,
            target,
            plaintext,
        )?;
        Ok(WifiSecretWrite { envelope })
    }

    /// Stores only the direct OpenAI credential and internally fixes its host
    /// to `api.openai.com`; callers cannot select a provider target.
    pub(crate) fn save_or_replace_provider(
        &self,
        _mutation: VaultMutation,
        plaintext: SecretPlaintext,
    ) -> Result<ProviderSecretWrite, VaultBrokerDenied> {
        let target = SecretTargetBinding::for_openai_host(OPENAI_API_HOST)
            .map_err(VaultBrokerDenied::Crypto)?;
        let envelope = self.prepare_write(
            VaultRecordId::ProviderApiKey,
            SecretKind::ProviderApiKey,
            SecretConsumer::OpenAiDirect,
            SecretOperation::RequestExactHost,
            target,
            plaintext,
        )?;
        Ok(ProviderSecretWrite { envelope })
    }

    /// Prepares only the typed tombstone metadata for WiFi.  This operation
    /// never erases old media cells and never performs the media write itself.
    pub(crate) fn forget_wifi(
        &self,
        _mutation: VaultMutation,
    ) -> Result<WifiSecretForget, VaultBrokerDenied> {
        let (record_version, previous_record_hash) = self.prepare_forget(&self.wifi)?;
        Ok(WifiSecretForget {
            record_version,
            previous_record_hash,
        })
    }

    /// Prepares only the typed tombstone metadata for the fixed provider slot.
    pub(crate) fn forget_provider(
        &self,
        _mutation: VaultMutation,
    ) -> Result<ProviderSecretForget, VaultBrokerDenied> {
        let (record_version, previous_record_hash) = self.prepare_forget(&self.provider)?;
        Ok(ProviderSecretForget {
            record_version,
            previous_record_hash,
        })
    }

    /// Creates one lease for an association attempt only when the currently
    /// observed BSS hashes to the stored bound WiFi target.
    pub(crate) fn use_for_wifi(
        &self,
        ssid: &[u8],
        bssid: [u8; 6],
        receipt: DurableWifiUseReceipt,
    ) -> Result<WifiSecretLease, VaultBrokerDenied> {
        let (authority, audited_target_sha256) =
            VaultUseAuthority::from_durable_wifi_receipt(receipt);
        let requested_target = SecretTargetBinding::for_wifi_wpa2_psk_ccmp(ssid, bssid)
            .map_err(VaultBrokerDenied::Crypto)?;
        if requested_target.hash() != audited_target_sha256 {
            return Err(VaultBrokerDenied::Use(SecretUseDenial::TargetMismatch));
        }
        let (plaintext, authorization) = self.authorize_and_decrypt(
            self.wifi.present()?,
            SecretConsumer::NativeWifiSupplicant,
            SecretOperation::AssociateBoundBss,
            requested_target,
            None,
            authority,
        )?;
        Ok(WifiSecretLease {
            plaintext,
            _authorization: authorization,
        })
    }

    /// Checks the requested BSS against the replayed ciphertext metadata
    /// without touching the VMK or plaintext. The facade uses this before it
    /// writes a durable pre-use audit record.
    pub(super) fn require_wifi_target(
        &self,
        ssid: &[u8],
        bssid: [u8; 6],
    ) -> Result<[u8; 32], VaultBrokerDenied> {
        let requested = SecretTargetBinding::for_wifi_wpa2_psk_ccmp(ssid, bssid)
            .map_err(VaultBrokerDenied::Crypto)?;
        if self.wifi.present()?.binding.target != requested {
            return Err(VaultBrokerDenied::Use(SecretUseDenial::TargetMismatch));
        }
        Ok(requested.hash())
    }

    /// Contained-QEMU negative proof only: the BSS and all current WiFi
    /// evidence are exact except the durable pre-use audit bit. Authorization
    /// must stop before decrypting with `AuditBindingMissing`.
    pub(super) fn prove_wifi_auditless_use_denied_for_contained_test(
        &self,
        ssid: &[u8],
        bssid: [u8; 6],
    ) -> Result<(), VaultBrokerDenied> {
        let envelope = self.wifi.present()?;
        let policy = self
            .approved_core_policy
            .ok_or(VaultBrokerDenied::CorePolicyMissing)?;
        let requested_target = SecretTargetBinding::for_wifi_wpa2_psk_ccmp(ssid, bssid)
            .map_err(VaultBrokerDenied::Crypto)?;
        let request = SecretUseRequest {
            kind: envelope.binding.kind,
            consumer: SecretConsumer::NativeWifiSupplicant,
            operation: SecretOperation::AssociateBoundBss,
            bound_target: envelope.binding.target,
            requested_target,
            provider_host: None,
            evidence: SecretUseEvidence {
                vault_unlocked: self.vmk.is_some(),
                boot_scope: raios_core::scoped_secret_use::SecretBootScope::CurrentBoot,
                explicit_recovery_action: false,
                service_generation: policy.core_generation(),
                current_service_generation: policy.core_generation(),
                record_version: envelope.binding.record_version,
                current_record_version: envelope.binding.record_version,
                key_epoch: envelope.binding.key_epoch,
                current_key_epoch: envelope.binding.key_epoch,
                trust_decision_positive: true,
                committed_store_record_verified: self.retained_nonces.is_some(),
                audit_binding_present: false,
            },
        };
        match authorize_secret_use(&request) {
            Err(SecretUseDenial::AuditBindingMissing) => Ok(()),
            Err(denied) => Err(VaultBrokerDenied::Use(denied)),
            Ok(_) => Err(VaultBrokerDenied::AuditlessUseUnexpectedlyAuthorized),
        }
    }

    /// Creates one lease only for a trust-authorized direct request to the
    /// fixed V1 OpenAI host.  No host parameter exists on this API.
    pub(crate) fn use_for_provider(
        &self,
        receipt: DurableProviderUseReceipt,
    ) -> Result<ProviderSecretLease, VaultBrokerDenied> {
        let authority = VaultUseAuthority::from_durable_provider_receipt(receipt);
        let requested_target = SecretTargetBinding::for_openai_host(OPENAI_API_HOST)
            .map_err(VaultBrokerDenied::Crypto)?;
        let (plaintext, authorization) = self.authorize_and_decrypt(
            self.provider.present()?,
            SecretConsumer::OpenAiDirect,
            SecretOperation::RequestExactHost,
            requested_target,
            Some(OPENAI_API_HOST),
            authority,
        )?;
        Ok(ProviderSecretLease {
            plaintext,
            _authorization: authorization,
        })
    }

    /// Contained-QEMU negative proof only: all current provider evidence is
    /// exact except the durable pre-use audit bit, so authorization must stop
    /// before decrypting with `AuditBindingMissing`.
    pub(super) fn prove_provider_auditless_use_denied_for_contained_test(
        &self,
    ) -> Result<(), VaultBrokerDenied> {
        let envelope = self.provider.present()?;
        let policy = self
            .approved_core_policy
            .ok_or(VaultBrokerDenied::CorePolicyMissing)?;
        let requested_target = SecretTargetBinding::for_openai_host(OPENAI_API_HOST)
            .map_err(VaultBrokerDenied::Crypto)?;
        let request = SecretUseRequest {
            kind: envelope.binding.kind,
            consumer: SecretConsumer::OpenAiDirect,
            operation: SecretOperation::RequestExactHost,
            bound_target: envelope.binding.target,
            requested_target,
            provider_host: Some(OPENAI_API_HOST),
            evidence: SecretUseEvidence {
                vault_unlocked: self.vmk.is_some(),
                boot_scope: raios_core::scoped_secret_use::SecretBootScope::CurrentBoot,
                explicit_recovery_action: false,
                service_generation: policy.core_generation(),
                current_service_generation: policy.core_generation(),
                record_version: envelope.binding.record_version,
                current_record_version: envelope.binding.record_version,
                key_epoch: envelope.binding.key_epoch,
                current_key_epoch: envelope.binding.key_epoch,
                trust_decision_positive: true,
                committed_store_record_verified: self.retained_nonces.is_some(),
                audit_binding_present: false,
            },
        };
        match authorize_secret_use(&request) {
            Err(SecretUseDenial::AuditBindingMissing) => Ok(()),
            Err(denied) => Err(VaultBrokerDenied::Use(denied)),
            Ok(_) => Err(VaultBrokerDenied::AuditlessUseUnexpectedlyAuthorized),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_write(
        &self,
        slot: VaultRecordId,
        kind: SecretKind,
        consumer: SecretConsumer,
        operation: SecretOperation,
        target: SecretTargetBinding,
        plaintext: SecretPlaintext,
    ) -> Result<SecretEnvelopeV1, VaultBrokerDenied> {
        if plaintext.kind() != kind {
            return Err(VaultBrokerDenied::Crypto(
                SecretVaultError::PlaintextKindMismatch,
            ));
        }
        let identity = self
            .identity
            .ok_or(VaultBrokerDenied::StoreIdentityInvalid)?;
        let vmk = self.vmk.as_ref().ok_or(VaultBrokerDenied::Locked)?;
        let key_epoch = self.unlocked_key_epoch.ok_or(VaultBrokerDenied::Locked)?;
        let state = self.slot(slot);
        validate_write_epoch(state, key_epoch)?;
        let replay_nonces = self
            .retained_nonces
            .as_ref()
            .ok_or(VaultBrokerDenied::CompleteReplayRequired)?;
        let binding = SecretBinding {
            store_uuid: identity.store_uuid,
            key_epoch,
            secret_id: slot.key().record_id,
            record_version: state.next_record_version()?,
            kind,
            consumer,
            operation,
            target,
            previous_record_hash: state.previous_record_hash(),
        };
        let mut retained = retained_nonces(replay_nonces, key_epoch)?;
        let nonce = fresh_nonce()?;
        let result = encrypt_secret(
            vmk,
            plaintext,
            binding,
            state.record_version(),
            FreshRecordNonce {
                nonce,
                retained_epoch_nonces: &retained,
                retained_set_complete: true,
            },
        )
        .map_err(VaultBrokerDenied::Crypto);
        for retained_nonce in &mut retained {
            retained_nonce.fill(0);
        }
        result
    }

    fn prepare_forget(&self, state: &SlotState) -> Result<(u64, [u8; 32]), VaultBrokerDenied> {
        let _ = self.vmk.as_ref().ok_or(VaultBrokerDenied::Locked)?;
        let key_epoch = self.unlocked_key_epoch.ok_or(VaultBrokerDenied::Locked)?;
        let envelope = state.present()?;
        if key_epoch == 0 {
            return Err(VaultBrokerDenied::KeyEpochInvalid);
        }
        if envelope.binding.key_epoch != key_epoch {
            return Err(VaultBrokerDenied::KeyEpochMismatch);
        }
        Ok((
            state.next_record_version()?,
            state
                .previous_record_hash()
                .ok_or(VaultBrokerDenied::SecretMissing)?,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn authorize_and_decrypt(
        &self,
        envelope: &SecretEnvelopeV1,
        consumer: SecretConsumer,
        operation: SecretOperation,
        requested_target: SecretTargetBinding,
        provider_host: Option<&str>,
        authority: VaultUseAuthority,
    ) -> Result<(SecretPlaintext, SecretUseAuthorization), VaultBrokerDenied> {
        let evidence = authority.evidence;
        if evidence.vault_unlocked != self.vmk.is_some() {
            return Err(VaultBrokerDenied::Use(SecretUseDenial::VaultLocked));
        }
        if evidence.record_version != envelope.binding.record_version
            || evidence.current_record_version != envelope.binding.record_version
        {
            return Err(VaultBrokerDenied::Use(
                SecretUseDenial::RecordVersionMismatch,
            ));
        }
        if evidence.key_epoch != envelope.binding.key_epoch
            || evidence.current_key_epoch != envelope.binding.key_epoch
        {
            return Err(VaultBrokerDenied::Use(SecretUseDenial::KeyEpochMismatch));
        }
        let authorization = authorize_secret_use(&SecretUseRequest {
            kind: envelope.binding.kind,
            consumer,
            operation,
            bound_target: envelope.binding.target,
            requested_target,
            provider_host,
            evidence,
        })
        .map_err(VaultBrokerDenied::Use)?;
        let vmk = self.vmk.as_ref().ok_or(VaultBrokerDenied::Locked)?;
        if self.unlocked_key_epoch != Some(envelope.binding.key_epoch) {
            return Err(VaultBrokerDenied::KeyEpochMismatch);
        }
        let plaintext = decrypt_secret(
            vmk,
            envelope,
            &envelope.binding,
            envelope.binding.record_version,
        )
        .map_err(VaultBrokerDenied::Crypto)?;
        Ok((plaintext, authorization))
    }

    fn slot(&self, slot: VaultRecordId) -> &SlotState {
        match slot {
            VaultRecordId::WifiPassphrase => &self.wifi,
            VaultRecordId::ProviderApiKey => &self.provider,
            VaultRecordId::RecoveryWrapper => unreachable!("recovery wrapper is not a broker slot"),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn prove_replayed_envelope_fail_closed(
    vmk: &VaultMasterKey,
    retained_metadata: &RetainedNonceMetadata,
    envelope: &SecretEnvelopeV1,
    mismatched_target: SecretTargetBinding,
    tag_step: ContainedQemuCryptoProofStep,
    aad_step: ContainedQemuCryptoProofStep,
    kind_step: ContainedQemuCryptoProofStep,
    consumer_step: ContainedQemuCryptoProofStep,
    operation_step: ContainedQemuCryptoProofStep,
    target_step: ContainedQemuCryptoProofStep,
    nonce_step: ContainedQemuCryptoProofStep,
) -> Result<(), ContainedQemuCryptoProofDenied> {
    let baseline = decrypt_secret(
        vmk,
        envelope,
        &envelope.binding,
        envelope.binding.record_version,
    )
    .map_err(
        |actual| ContainedQemuCryptoProofDenied::EnvelopeAuthenticationFailed {
            kind: envelope.binding.kind,
            actual,
        },
    )?;
    drop(baseline);

    let mut corrupt_tag = duplicate_envelope(envelope);
    corrupt_tag.tag[0] ^= 1;
    require_crypto_denial(
        decrypt_secret(
            vmk,
            &corrupt_tag,
            &envelope.binding,
            envelope.binding.record_version,
        ),
        SecretVaultError::AuthenticationFailed,
        tag_step,
    )?;

    let mut corrupt_aad = duplicate_envelope(envelope);
    corrupt_aad.aad_hash[0] ^= 1;
    require_crypto_denial(
        decrypt_secret(
            vmk,
            &corrupt_aad,
            &envelope.binding,
            envelope.binding.record_version,
        ),
        SecretVaultError::AadHashMismatch,
        aad_step,
    )?;

    let mut expected = envelope.binding;
    expected.kind = other_kind(expected.kind);
    require_crypto_denial(
        decrypt_secret(vmk, envelope, &expected, envelope.binding.record_version),
        SecretVaultError::SecretKindMismatch,
        kind_step,
    )?;

    expected = envelope.binding;
    expected.consumer = other_consumer(expected.consumer);
    require_crypto_denial(
        decrypt_secret(vmk, envelope, &expected, envelope.binding.record_version),
        SecretVaultError::ConsumerMismatch,
        consumer_step,
    )?;

    expected = envelope.binding;
    expected.operation = other_operation(expected.operation);
    require_crypto_denial(
        decrypt_secret(vmk, envelope, &expected, envelope.binding.record_version),
        SecretVaultError::OperationMismatch,
        operation_step,
    )?;

    expected = envelope.binding;
    expected.target = mismatched_target;
    require_crypto_denial(
        decrypt_secret(vmk, envelope, &expected, envelope.binding.record_version),
        SecretVaultError::TargetMismatch,
        target_step,
    )?;

    let mut binding = envelope.binding;
    binding.record_version = binding.record_version.checked_add(1).ok_or(
        ContainedQemuCryptoProofDenied::RecordVersionExhausted(binding.kind),
    )?;
    let mut proof_plaintext = *b"raios-contained-qemu";
    let proof_plaintext =
        SecretPlaintext::take_from(binding.kind, &mut proof_plaintext).map_err(|actual| {
            ContainedQemuCryptoProofDenied::UnexpectedCryptoResult {
                step: nonce_step,
                expected: SecretVaultError::DuplicateNonce,
                actual: Some(actual),
            }
        })?;
    let mut retained = retained_nonces(retained_metadata, envelope.binding.key_epoch)
        .map_err(|_| ContainedQemuCryptoProofDenied::RetainedNonceAllocation)?;
    if !retained.contains(&envelope.nonce) {
        for nonce in &mut retained {
            nonce.fill(0);
        }
        return Err(ContainedQemuCryptoProofDenied::RetainedNonceMissing(
            envelope.binding.kind,
        ));
    }
    let result = encrypt_secret(
        vmk,
        proof_plaintext,
        binding,
        Some(envelope.binding.record_version),
        FreshRecordNonce {
            nonce: envelope.nonce,
            retained_epoch_nonces: &retained,
            retained_set_complete: true,
        },
    );
    for nonce in &mut retained {
        nonce.fill(0);
    }
    require_crypto_denial(result, SecretVaultError::DuplicateNonce, nonce_step)
}

fn require_crypto_denial<T>(
    result: Result<T, SecretVaultError>,
    expected: SecretVaultError,
    step: ContainedQemuCryptoProofStep,
) -> Result<(), ContainedQemuCryptoProofDenied> {
    match result {
        Err(actual) if actual == expected => Ok(()),
        Err(actual) => Err(ContainedQemuCryptoProofDenied::UnexpectedCryptoResult {
            step,
            expected,
            actual: Some(actual),
        }),
        Ok(value) => {
            drop(value);
            Err(ContainedQemuCryptoProofDenied::UnexpectedCryptoResult {
                step,
                expected,
                actual: None,
            })
        }
    }
}

fn duplicate_envelope(envelope: &SecretEnvelopeV1) -> SecretEnvelopeV1 {
    SecretEnvelopeV1 {
        version: envelope.version,
        binding: envelope.binding,
        plaintext_len: envelope.plaintext_len,
        aad_hash: envelope.aad_hash,
        nonce: envelope.nonce,
        ciphertext: envelope.ciphertext,
        tag: envelope.tag,
    }
}

const fn other_kind(kind: SecretKind) -> SecretKind {
    match kind {
        SecretKind::WifiPassphrase => SecretKind::ProviderApiKey,
        SecretKind::ProviderApiKey => SecretKind::WifiPassphrase,
    }
}

const fn other_consumer(consumer: SecretConsumer) -> SecretConsumer {
    match consumer {
        SecretConsumer::NativeWifiSupplicant => SecretConsumer::OpenAiDirect,
        SecretConsumer::OpenAiDirect => SecretConsumer::NativeWifiSupplicant,
    }
}

const fn other_operation(operation: SecretOperation) -> SecretOperation {
    match operation {
        SecretOperation::AssociateBoundBss => SecretOperation::RequestExactHost,
        SecretOperation::RequestExactHost => SecretOperation::AssociateBoundBss,
    }
}

fn validate_identity(identity: StoreIdentity) -> Result<(), VaultBrokerDenied> {
    if identity.store_uuid == [0; 16] || identity.generation == 0 {
        return Err(VaultBrokerDenied::StoreIdentityInvalid);
    }
    Ok(())
}

fn install_replay_record(
    identity: StoreIdentity,
    record: &CommittedRecord,
    wifi: &mut SlotState,
    provider: &mut SlotState,
) -> Result<(), VaultBrokerDenied> {
    let replay = ReplayVerifiedVaultRecord::from_committed(identity, record)
        .map_err(VaultBrokerDenied::Store)?;
    let record_version = replay.record_version();
    let commit_hash = record.commit_frame_sha256;
    match replay.decode().map_err(VaultBrokerDenied::Store)? {
        DecodedVaultRecord::WifiPassphrase(envelope) => {
            validate_exact_slot(VaultRecordId::WifiPassphrase, &envelope)?;
            wifi.install_present(envelope, commit_hash)
        }
        DecodedVaultRecord::ProviderApiKey(envelope) => {
            validate_exact_slot(VaultRecordId::ProviderApiKey, &envelope)?;
            provider.install_present(envelope, commit_hash)
        }
        DecodedVaultRecord::Tombstone(VaultRecordId::WifiPassphrase) => {
            wifi.install_tombstone(record_version, commit_hash)
        }
        DecodedVaultRecord::Tombstone(VaultRecordId::ProviderApiKey) => {
            provider.install_tombstone(record_version, commit_hash)
        }
        DecodedVaultRecord::RecoveryWrapper(_) | DecodedVaultRecord::Tombstone(_) => Ok(()),
    }
}

fn validate_exact_slot(
    slot: VaultRecordId,
    envelope: &SecretEnvelopeV1,
) -> Result<(), VaultBrokerDenied> {
    if envelope.binding.secret_id != slot.key().record_id {
        return Err(VaultBrokerDenied::SecretIdMismatch);
    }
    let expected = match slot {
        VaultRecordId::WifiPassphrase => (
            SecretKind::WifiPassphrase,
            SecretConsumer::NativeWifiSupplicant,
            SecretOperation::AssociateBoundBss,
        ),
        VaultRecordId::ProviderApiKey => (
            SecretKind::ProviderApiKey,
            SecretConsumer::OpenAiDirect,
            SecretOperation::RequestExactHost,
        ),
        VaultRecordId::RecoveryWrapper => return Err(VaultBrokerDenied::RecordNotSecret),
    };
    if (
        envelope.binding.kind,
        envelope.binding.consumer,
        envelope.binding.operation,
    ) != expected
    {
        return Err(VaultBrokerDenied::EnvelopeSlotMismatch);
    }
    Ok(())
}

fn validate_write_epoch(state: &SlotState, key_epoch: u64) -> Result<(), VaultBrokerDenied> {
    if key_epoch == 0 {
        return Err(VaultBrokerDenied::KeyEpochInvalid);
    }
    if let SlotState::Present { envelope, .. } = state {
        if envelope.binding.key_epoch != key_epoch {
            return Err(VaultBrokerDenied::KeyEpochMismatch);
        }
    }
    Ok(())
}

fn retained_nonces(
    metadata: &RetainedNonceMetadata,
    key_epoch: u64,
) -> Result<Vec<[u8; 12]>, VaultBrokerDenied> {
    let count = metadata
        .secret()
        .iter()
        .filter(|retained| retained.key_epoch == key_epoch)
        .count();
    let mut nonces = Vec::new();
    nonces
        .try_reserve_exact(count)
        .map_err(|_| VaultBrokerDenied::RetainedNonceAllocation)?;
    for retained in metadata.secret() {
        if retained.key_epoch == key_epoch {
            nonces.push(retained.nonce);
        }
    }
    Ok(nonces)
}

fn fresh_nonce() -> Result<[u8; 12], VaultBrokerDenied> {
    if !entropy::is_ready() {
        return Err(VaultBrokerDenied::EntropyNotReady);
    }
    let mut nonce = [0u8; 12];
    entropy::take(&mut nonce);
    Ok(nonce)
}

#[cfg(test)]
mod tests {
    use raios_core::{
        secret_vault::{SecretBinding, SECRET_ENVELOPE_VERSION},
        sha256_bytes,
        structured_store::{RecordValue, StoreIdentity},
    };

    use super::*;
    use crate::secret_vault::store::encode_secret_envelope;

    const IDENTITY: StoreIdentity = StoreIdentity {
        store_uuid: *b"vault-broker-tst",
        generation: 5,
    };

    fn wifi_envelope(version: u64) -> SecretEnvelopeV1 {
        let mut ciphertext = [0u8; 256];
        ciphertext[..3].copy_from_slice(b"enc");
        SecretEnvelopeV1 {
            version: SECRET_ENVELOPE_VERSION,
            binding: SecretBinding {
                store_uuid: IDENTITY.store_uuid,
                key_epoch: 7,
                secret_id: VaultRecordId::WifiPassphrase.key().record_id,
                record_version: version,
                kind: SecretKind::WifiPassphrase,
                consumer: SecretConsumer::NativeWifiSupplicant,
                operation: SecretOperation::AssociateBoundBss,
                target: SecretTargetBinding::for_wifi_wpa2_psk_ccmp(b"test", [2, 1, 2, 3, 4, 5])
                    .unwrap(),
                previous_record_hash: None,
            },
            plaintext_len: 3,
            aad_hash: [0x41; 32],
            nonce: [0x51; 12],
            ciphertext,
            tag: [0x61; 16],
        }
    }

    fn committed_wifi(version: u64) -> CommittedRecord {
        let payload =
            encode_secret_envelope(VaultRecordId::WifiPassphrase, &wifi_envelope(version))
                .unwrap()
                .to_vec();
        CommittedRecord {
            key: VaultRecordId::WifiPassphrase.key(),
            transaction_id: 9,
            record_version: version,
            commit_frame_sha256: [0x71; 32],
            payload_sha256: sha256_bytes(&payload),
            value: RecordValue::Present(payload),
        }
    }

    #[test]
    fn replayed_ciphertext_updates_only_public_status() {
        let mut broker = VaultBroker::new();
        let records = [committed_wifi(1)];
        broker
            .load_complete_replay(
                VaultCompleteReplay::from_complete_history(IDENTITY, &records).unwrap(),
            )
            .unwrap();
        assert_eq!(
            broker.status(),
            VaultBrokerStatus {
                store: VaultStoreStatus::Bound { generation: 5 },
                core_policy: VaultCorePolicyStatus::Unbound,
                lock: VaultLockStatus::Locked,
                wifi: VaultSecretStatus::Available {
                    record_version: 1,
                    key_epoch: 7,
                },
                provider: VaultSecretStatus::Missing,
            }
        );
        assert!(matches!(
            broker.use_for_wifi(
                b"test",
                [2, 1, 2, 3, 4, 5],
                wifi_use_receipt(b"test", [2, 1, 2, 3, 4, 5]),
            ),
            Err(VaultBrokerDenied::Use(SecretUseDenial::VaultLocked))
        ));
    }

    #[test]
    fn policy_binding_is_idempotent_and_rejects_identity_replacement() {
        let mut broker = VaultBroker::new();
        let policy = ApprovedCorePolicy::for_test(7, [0x44; 32]);
        broker.bind_approved_core_policy(policy).unwrap();
        broker.bind_approved_core_policy(policy).unwrap();
        assert_eq!(
            broker.status().core_policy,
            VaultCorePolicyStatus::Bound {
                core_generation: 7,
                policy_id_sha256: [0x44; 32],
            }
        );
        assert_eq!(
            broker.bind_approved_core_policy(ApprovedCorePolicy::for_test(8, [0x55; 32])),
            Err(VaultBrokerDenied::CorePolicyIdentityMismatch)
        );
    }

    #[test]
    fn tombstone_removes_future_broker_use_without_erasing_history() {
        let mut broker = VaultBroker::new();
        let tombstone = CommittedRecord {
            key: VaultRecordId::WifiPassphrase.key(),
            transaction_id: 10,
            record_version: 2,
            commit_frame_sha256: [0x72; 32],
            payload_sha256: sha256_bytes(&[]),
            value: RecordValue::Tombstone,
        };
        let records = [committed_wifi(1), tombstone];
        broker
            .load_complete_replay(
                VaultCompleteReplay::from_complete_history(IDENTITY, &records).unwrap(),
            )
            .unwrap();
        assert_eq!(
            broker.status().wifi,
            VaultSecretStatus::Forgotten { record_version: 2 }
        );
        assert!(matches!(
            broker.use_for_wifi(
                b"test",
                [2, 1, 2, 3, 4, 5],
                wifi_use_receipt(b"test", [2, 1, 2, 3, 4, 5]),
            ),
            Err(VaultBrokerDenied::SecretForgotten)
        ));
    }

    #[test]
    fn complete_replay_rejects_a_secret_without_its_predecessor() {
        let mut envelope = wifi_envelope(2);
        envelope.binding.previous_record_hash = Some([0x33; 32]);
        let payload = encode_secret_envelope(VaultRecordId::WifiPassphrase, &envelope)
            .unwrap()
            .to_vec();
        let record = CommittedRecord {
            key: VaultRecordId::WifiPassphrase.key(),
            transaction_id: 10,
            record_version: 2,
            commit_frame_sha256: [0x72; 32],
            payload_sha256: sha256_bytes(&payload),
            value: RecordValue::Present(payload),
        };
        assert!(matches!(
            VaultCompleteReplay::from_complete_history(IDENTITY, &[record]),
            Err(VaultBrokerDenied::Store(
                VaultStoreDenied::NonceHistoryPreviousRecordHashMismatch
            ))
        ));
    }

    #[test]
    fn write_rejects_plaintext_for_the_wrong_fixed_slot() {
        let mut source = *b"not-a-wifi-key";
        let plaintext =
            SecretPlaintext::take_from(SecretKind::ProviderApiKey, &mut source).unwrap();
        assert!(matches!(
            VaultBroker::new().prepare_write(
                VaultRecordId::WifiPassphrase,
                SecretKind::WifiPassphrase,
                SecretConsumer::NativeWifiSupplicant,
                SecretOperation::AssociateBoundBss,
                SecretTargetBinding::for_wifi_wpa2_psk_ccmp(b"test", [2, 1, 2, 3, 4, 5]).unwrap(),
                plaintext,
            ),
            Err(VaultBrokerDenied::Crypto(
                SecretVaultError::PlaintextKindMismatch
            ))
        ));
    }

    #[test]
    fn contained_qemu_crypto_proof_uses_replayed_envelopes_and_retained_nonces() {
        let master = VaultMasterKey::take_from(&mut [0x73; 32]);
        let wifi_binding = SecretBinding {
            store_uuid: IDENTITY.store_uuid,
            key_epoch: 7,
            secret_id: VaultRecordId::WifiPassphrase.key().record_id,
            record_version: 1,
            kind: SecretKind::WifiPassphrase,
            consumer: SecretConsumer::NativeWifiSupplicant,
            operation: SecretOperation::AssociateBoundBss,
            target: SecretTargetBinding::for_wifi_wpa2_psk_ccmp(b"proof-wifi", [2, 1, 2, 3, 4, 5])
                .unwrap(),
            previous_record_hash: None,
        };
        let provider_binding = SecretBinding {
            store_uuid: IDENTITY.store_uuid,
            key_epoch: 7,
            secret_id: VaultRecordId::ProviderApiKey.key().record_id,
            record_version: 1,
            kind: SecretKind::ProviderApiKey,
            consumer: SecretConsumer::OpenAiDirect,
            operation: SecretOperation::RequestExactHost,
            target: SecretTargetBinding::for_openai_host(OPENAI_API_HOST).unwrap(),
            previous_record_hash: None,
        };
        let mut wifi_plaintext = *b"proof-passphrase";
        let wifi = encrypt_secret(
            &master,
            SecretPlaintext::take_from(SecretKind::WifiPassphrase, &mut wifi_plaintext).unwrap(),
            wifi_binding,
            None,
            FreshRecordNonce {
                nonce: [0x31; 12],
                retained_epoch_nonces: &[],
                retained_set_complete: true,
            },
        )
        .unwrap();
        let mut provider_plaintext = *b"proof-provider-key";
        let provider = encrypt_secret(
            &master,
            SecretPlaintext::take_from(SecretKind::ProviderApiKey, &mut provider_plaintext)
                .unwrap(),
            provider_binding,
            None,
            FreshRecordNonce {
                nonce: [0x32; 12],
                retained_epoch_nonces: &[[0x31; 12]],
                retained_set_complete: true,
            },
        )
        .unwrap();
        let wifi_payload = encode_secret_envelope(VaultRecordId::WifiPassphrase, &wifi)
            .unwrap()
            .to_vec();
        let provider_payload = encode_secret_envelope(VaultRecordId::ProviderApiKey, &provider)
            .unwrap()
            .to_vec();
        let records = [
            CommittedRecord {
                key: VaultRecordId::WifiPassphrase.key(),
                transaction_id: 9,
                record_version: 1,
                commit_frame_sha256: [0x71; 32],
                payload_sha256: sha256_bytes(&wifi_payload),
                value: RecordValue::Present(wifi_payload),
            },
            CommittedRecord {
                key: VaultRecordId::ProviderApiKey.key(),
                transaction_id: 10,
                record_version: 1,
                commit_frame_sha256: [0x72; 32],
                payload_sha256: sha256_bytes(&provider_payload),
                value: RecordValue::Present(provider_payload),
            },
        ];
        let mut broker = VaultBroker::new();
        broker
            .load_complete_replay(
                VaultCompleteReplay::from_complete_history(IDENTITY, &records).unwrap(),
            )
            .unwrap();
        broker.vmk = Some(master);
        broker.unlocked_key_epoch = Some(7);

        assert_eq!(
            broker.prove_exact_contained_qemu_crypto_fail_closed(),
            Ok(ContainedQemuCryptoProof)
        );
    }

    fn wifi_use_receipt(ssid: &[u8], bssid: [u8; 6]) -> DurableWifiUseReceipt {
        DurableWifiUseReceipt::for_test(
            SecretUseEvidence {
                vault_unlocked: true,
                boot_scope: SecretBootScope::CurrentBoot,
                explicit_recovery_action: false,
                service_generation: 1,
                current_service_generation: 1,
                record_version: 1,
                current_record_version: 1,
                key_epoch: 7,
                current_key_epoch: 7,
                trust_decision_positive: true,
                committed_store_record_verified: true,
                audit_binding_present: true,
            },
            SecretTargetBinding::for_wifi_wpa2_psk_ccmp(ssid, bssid)
                .unwrap()
                .hash(),
        )
    }
}
