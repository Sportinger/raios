//! Core-only Secret Vault custody. No service-facing secret accessor lives here.

use alloc::vec::Vec;
use spin::Mutex;

use raios_core::{
    secret_vault::{
        RecoveryKekContext, RecoveryVmkWrapperV1, SecretConsumerError, SecretPlaintext,
        VaultMasterKey, MAX_PROVIDER_API_KEY_LEN,
    },
    sha256_bytes,
    structured_store::{RecordKey, ReplayTail, StoreDenied},
};

use self::{
    audit::{
        append_provider_use_audit, append_wifi_use_audit, ProviderRequestTrust,
        ProviderUseAuditBinding, ProviderUseAuditDenied, WifiUseAuditBinding, WifiUseAuditDenied,
    },
    broker::{
        VaultBroker, VaultBrokerDenied, VaultCompleteReplay, VaultLockStatus, WifiSecretLease,
    },
    keyring::{
        ApprovedCorePolicy, KeyringDenied, RecoveryKeyCandidate, RecoveryKeyText,
        RecoveryWrapperSlots,
    },
    store::{
        encode_recovery_wrapper, encode_secret_envelope, DecodedVaultRecord,
        ReplayVerifiedVaultRecord, VaultRecordId, VaultStoreDenied, VAULT_NAMESPACE,
    },
};
use crate::{
    serial,
    structured_store::{ValidatedRegionIdentity, ValidatedReplayWithHistory},
};

mod audit;
mod broker;
mod keyring;
mod store;

pub(crate) use self::{
    broker::{ProviderSecretLease, VaultBrokerStatus, VaultSecretStatus},
    store::{
        RECOVERY_WRAPPER_PAYLOAD_LEN, SECRET_ENVELOPE_PAYLOAD_LEN as PROVIDER_SECRET_PAYLOAD_LEN,
        SECRET_ENVELOPE_PAYLOAD_LEN as WIFI_SECRET_PAYLOAD_LEN,
    },
};

const CONTAINED_WIFI_SSID: &[u8] = b"RaiWPA2";
const CONTAINED_WIFI_BSSID: [u8; 6] = [0x20, 0x22, 0x33, 0x44, 0x55, 0x66];
const CONTAINED_WIFI_WRONG_BSSID: [u8; 6] = [0x22, 0x22, 0x33, 0x44, 0x55, 0x66];

#[derive(Debug)]
pub(crate) enum VaultFacadeDenied {
    HistoryLocked,
    CorePolicy(&'static str),
    RegionIdentityMismatch,
    RecoveryUnavailable,
    RecoveryAlreadyProvisioned,
    RecoverySessionActive,
    RecoverySessionMissing,
    PersistenceOutcomeUncertain,
    RecoveryWrapperMissing,
    Keyring(KeyringDenied),
    Store(VaultStoreDenied),
    StructuredStore(StoreDenied),
    RecoveryStore(crate::structured_store_c1::RecoveryWrapperStoreDenied),
    ProviderStore(crate::structured_store_c1::ProviderSecretStoreDenied),
    WifiStore(crate::structured_store_c1::WifiSecretStoreDenied),
    Broker(VaultBrokerDenied),
    Audit(ProviderUseAuditDenied),
    WifiAudit(WifiUseAuditDenied),
    Consumer(SecretConsumerError),
    ProviderUseStateChanged,
    WifiUseStateChanged,
    ContainedProviderValidationFailed,
    ContainedWifiValidationFailed,
}

impl VaultFacadeDenied {
    /// Stable, non-secret recovery diagnostics. No submitted bytes, key
    /// material, ciphertext, hashes, or caller-controlled text can enter it.
    pub(crate) const fn recovery_reason(&self) -> &'static str {
        match self {
            Self::HistoryLocked => "history_locked",
            Self::CorePolicy(_) => "core_policy_denied",
            Self::RegionIdentityMismatch => "region_identity_mismatch",
            Self::RecoveryUnavailable => "recovery_unavailable",
            Self::RecoveryAlreadyProvisioned => "already_provisioned",
            Self::RecoverySessionActive => "session_active",
            Self::RecoverySessionMissing => "session_missing",
            Self::PersistenceOutcomeUncertain => "persistence_outcome_uncertain",
            Self::RecoveryWrapperMissing => "wrapper_missing",
            Self::Keyring(KeyringDenied::InvalidRecoveryKeyFormat) => "format",
            Self::Keyring(KeyringDenied::InvalidRecoveryKeyChecksum) => "checksum",
            Self::Keyring(KeyringDenied::RecoveryKeyConfirmationMismatch) => {
                "confirmation_mismatch"
            }
            Self::Keyring(_) => "keyring_denied",
            Self::Store(_) => "vault_record_denied",
            Self::StructuredStore(_) => "structured_store_denied",
            Self::RecoveryStore(_) => "recovery_store_denied",
            Self::ProviderStore(_) => "provider_store_denied",
            Self::WifiStore(_) => "wifi_store_denied",
            Self::Broker(_) => "broker_denied",
            Self::Audit(error) => error.reason(),
            Self::WifiAudit(error) => error.reason(),
            Self::Consumer(_) => "provider_consumer_denied",
            Self::ProviderUseStateChanged => "provider_use_state_changed",
            Self::WifiUseStateChanged => "wifi_use_state_changed",
            Self::ContainedProviderValidationFailed => "contained_provider_validation_failed",
            Self::ContainedWifiValidationFailed => "contained_wifi_validation_failed",
        }
    }

    /// Stable, non-secret reason for the driver marker. It never includes a
    /// submitted SSID/BSSID, secret metadata, ciphertext, or caller text.
    pub(crate) const fn wifi_use_reason(&self) -> &'static str {
        match self {
            Self::HistoryLocked => "history_locked",
            Self::CorePolicy(_) => "core_policy_denied",
            Self::RegionIdentityMismatch => "region_identity_mismatch",
            Self::PersistenceOutcomeUncertain => "persistence_outcome_uncertain",
            Self::WifiStore(_) => "wifi_store_denied",
            Self::WifiAudit(error) => (*error).reason(),
            Self::Consumer(_) => "wifi_consumer_denied",
            Self::WifiUseStateChanged => "wifi_use_state_changed",
            Self::Broker(VaultBrokerDenied::Locked) => "vault_locked",
            Self::Broker(VaultBrokerDenied::SecretMissing) => "secret_missing",
            Self::Broker(VaultBrokerDenied::SecretForgotten) => "secret_forgotten",
            Self::Broker(VaultBrokerDenied::Use(reason)) => wifi_use_denial_reason(*reason),
            Self::Broker(VaultBrokerDenied::Crypto(_)) => "invalid_bound_bss",
            Self::ContainedWifiValidationFailed => "contained_wifi_validation_failed",
            _ => "wifi_vault_denied",
        }
    }
}

const fn wifi_use_denial_reason(
    reason: raios_core::scoped_secret_use::SecretUseDenial,
) -> &'static str {
    use raios_core::scoped_secret_use::SecretUseDenial;

    match reason {
        SecretUseDenial::SecretKindMismatch => "secret_kind_mismatch",
        SecretUseDenial::ConsumerMismatch => "consumer_mismatch",
        SecretUseDenial::OperationMismatch => "operation_mismatch",
        SecretUseDenial::TargetKindMismatch => "target_kind_mismatch",
        SecretUseDenial::TargetMismatch => "target_mismatch",
        SecretUseDenial::ProviderHostMissing => "provider_host_missing",
        SecretUseDenial::ProviderHostMismatch => "provider_host_mismatch",
        SecretUseDenial::UnexpectedProviderHost => "unexpected_provider_host",
        SecretUseDenial::VaultLocked => "vault_locked",
        SecretUseDenial::SafeRecoveryActionMissing => "safe_recovery_action_missing",
        SecretUseDenial::InvalidServiceGeneration => "invalid_service_generation",
        SecretUseDenial::ServiceGenerationMismatch => "service_generation_mismatch",
        SecretUseDenial::InvalidRecordVersion => "invalid_record_version",
        SecretUseDenial::RecordVersionMismatch => "record_version_mismatch",
        SecretUseDenial::InvalidKeyEpoch => "invalid_key_epoch",
        SecretUseDenial::KeyEpochMismatch => "key_epoch_mismatch",
        SecretUseDenial::TrustDecisionMissing => "trust_decision_missing",
        SecretUseDenial::CommittedStoreEvidenceMissing => "store_evidence_missing",
        SecretUseDenial::AuditBindingMissing => "audit_binding_missing",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VaultRecoveryState {
    Unavailable,
    ReadyToProvision,
    AwaitingConfirmation,
    PersistenceOutcomeUncertain,
    Locked,
    Unlocked,
}

/// The only RR1 display handle. It is neither cloneable nor formattable, and
/// dropping it cancels the singleton pending ceremony.
pub(crate) struct RecoveryKeyDisplay {
    text: RecoveryKeyText,
    owns_session: bool,
}

impl RecoveryKeyDisplay {
    pub(crate) fn as_bytes(&self) -> &[u8; keyring::RECOVERY_KEY_TEXT_LEN] {
        self.text.as_bytes()
    }

    /// Destroys the displayed RR1 and transfers only the singleton ceremony
    /// lease to the masked confirmation input path.
    pub(crate) fn begin_confirmation(mut self) -> RecoveryKeyConfirmation {
        self.owns_session = false;
        RecoveryKeyConfirmation { owns_session: true }
    }
}

impl Drop for RecoveryKeyDisplay {
    fn drop(&mut self) {
        if self.owns_session {
            cancel_pending_session();
        }
    }
}

/// Non-secret proof that the one-time display was consumed before re-entry.
pub(crate) struct RecoveryKeyConfirmation {
    owns_session: bool,
}

impl Drop for RecoveryKeyConfirmation {
    fn drop(&mut self) {
        if self.owns_session {
            cancel_pending_session();
        }
    }
}

struct InitialRecoverySession {
    vmk: VaultMasterKey,
    candidate: RecoveryKeyCandidate,
    region: ValidatedRegionIdentity,
    policy: ApprovedCorePolicy,
}

enum ProvisioningState {
    Idle,
    Reserved,
    Awaiting(InitialRecoverySession),
    Confirming,
    PersistenceOutcomeUncertain,
}

struct VaultRuntime {
    broker: VaultBroker,
    region: Option<ValidatedRegionIdentity>,
    approved_policy: Option<ApprovedCorePolicy>,
    wrappers: Option<RecoveryWrapperSlots>,
    replayed_wrapper: Option<RecoveryVmkWrapperV1>,
    replay_loaded: bool,
    vault_history_empty: bool,
    provisioning: ProvisioningState,
    provider_write_outcome_uncertain: bool,
    wifi_write_outcome_uncertain: bool,
}

impl VaultRuntime {
    const fn new() -> Self {
        Self {
            broker: VaultBroker::new(),
            region: None,
            approved_policy: None,
            wrappers: None,
            replayed_wrapper: None,
            replay_loaded: false,
            vault_history_empty: false,
            provisioning: ProvisioningState::Idle,
            provider_write_outcome_uncertain: false,
            wifi_write_outcome_uncertain: false,
        }
    }

    fn recovery_state(&self) -> VaultRecoveryState {
        if matches!(
            self.broker.status().lock,
            VaultLockStatus::UnlockedFromRecovery(_)
        ) {
            return VaultRecoveryState::Unlocked;
        }
        match &self.provisioning {
            ProvisioningState::PersistenceOutcomeUncertain => {
                return VaultRecoveryState::PersistenceOutcomeUncertain;
            }
            ProvisioningState::Reserved
            | ProvisioningState::Awaiting(_)
            | ProvisioningState::Confirming => {
                return VaultRecoveryState::AwaitingConfirmation;
            }
            ProvisioningState::Idle => {}
        }
        if self.wrappers.is_some() {
            VaultRecoveryState::Locked
        } else if self.replay_loaded
            && self.vault_history_empty
            && self.region.is_some()
            && self.approved_policy.is_some()
        {
            VaultRecoveryState::ReadyToProvision
        } else {
            VaultRecoveryState::Unavailable
        }
    }
}

static RUNTIME: Mutex<VaultRuntime> = Mutex::new(VaultRuntime::new());

/// Installs only history read through the identity-revalidated store port.
/// Non-Vault namespaces remain outside the Broker and cannot influence its
/// nonce/predecessor proof.
pub(crate) fn load_verified_replay(
    replay: &ValidatedReplayWithHistory,
) -> Result<VaultBrokerStatus, VaultFacadeDenied> {
    let history = replay
        .committed_history()
        .map_err(|_| VaultFacadeDenied::HistoryLocked)?;
    let mut vault_records = Vec::new();
    vault_records
        .try_reserve_exact(history.len())
        .map_err(|_| VaultFacadeDenied::Broker(VaultBrokerDenied::RetainedNonceAllocation))?;
    vault_records.extend(
        history
            .iter()
            .filter(|record| record.key.namespace == VAULT_NAMESPACE)
            .cloned(),
    );

    let region = replay.identity();
    let identity = region.store;
    if replay.state().identity != identity {
        return Err(VaultFacadeDenied::Broker(
            VaultBrokerDenied::StoreIdentityMismatch,
        ));
    }
    let replayed_wrapper = recovery_wrapper_from_replay(replay)?;
    let complete = VaultCompleteReplay::from_complete_history(identity, &vault_records)
        .map_err(VaultFacadeDenied::Broker)?;

    let mut runtime = RUNTIME.lock();
    if runtime.region.is_some_and(|bound| bound != region) {
        return Err(VaultFacadeDenied::RegionIdentityMismatch);
    }
    runtime
        .broker
        .load_complete_replay(complete)
        .map_err(VaultFacadeDenied::Broker)?;
    runtime.region = Some(region);
    runtime.replay_loaded = true;
    runtime.vault_history_empty =
        vault_records.is_empty() && replay.state().tail == ReplayTail::ZeroFilled;
    runtime.replayed_wrapper = replayed_wrapper;
    runtime.wrappers = None;
    restore_replayed_wrapper(&mut runtime)?;
    Ok(runtime.broker.status())
}

/// Separately binds the already verified executing-core identity. Replay may
/// load without it, but recovery unlock remains denied until both are present.
pub(crate) fn bind_verified_core_policy() -> Result<VaultBrokerStatus, VaultFacadeDenied> {
    let verified = crate::core_policy_runtime::verified().map_err(VaultFacadeDenied::CorePolicy)?;
    let approved = ApprovedCorePolicy::from_verified(verified);
    let mut runtime = RUNTIME.lock();
    runtime
        .broker
        .bind_verified_core_policy(verified)
        .map_err(VaultFacadeDenied::Broker)?;
    if runtime
        .approved_policy
        .is_some_and(|bound| bound != approved)
    {
        return Err(VaultFacadeDenied::Broker(
            VaultBrokerDenied::CorePolicyIdentityMismatch,
        ));
    }
    runtime.approved_policy = Some(approved);
    restore_replayed_wrapper(&mut runtime)?;
    Ok(runtime.broker.status())
}

pub(crate) fn recovery_state() -> VaultRecoveryState {
    RUNTIME.lock().recovery_state()
}

pub(crate) fn begin_initial_recovery() -> Result<RecoveryKeyDisplay, VaultFacadeDenied> {
    let (region, policy) = {
        let mut runtime = RUNTIME.lock();
        match runtime.recovery_state() {
            VaultRecoveryState::ReadyToProvision => {}
            VaultRecoveryState::AwaitingConfirmation => {
                return Err(VaultFacadeDenied::RecoverySessionActive);
            }
            VaultRecoveryState::PersistenceOutcomeUncertain => {
                return Err(VaultFacadeDenied::PersistenceOutcomeUncertain);
            }
            VaultRecoveryState::Locked | VaultRecoveryState::Unlocked => {
                return Err(VaultFacadeDenied::RecoveryAlreadyProvisioned);
            }
            VaultRecoveryState::Unavailable => {
                return Err(VaultFacadeDenied::RecoveryUnavailable);
            }
        }
        let region = runtime
            .region
            .ok_or(VaultFacadeDenied::RecoveryUnavailable)?;
        let policy = runtime
            .approved_policy
            .ok_or(VaultFacadeDenied::RecoveryUnavailable)?;
        runtime.provisioning = ProvisioningState::Reserved;
        (region, policy)
    };

    let vmk = match keyring::generate_vault_master_key() {
        Ok(vmk) => vmk,
        Err(error) => {
            reset_pre_io_provisioning();
            return Err(VaultFacadeDenied::Keyring(error));
        }
    };
    let (candidate, text) = match keyring::generate_recovery_key_candidate() {
        Ok(value) => value,
        Err(error) => {
            reset_pre_io_provisioning();
            return Err(VaultFacadeDenied::Keyring(error));
        }
    };

    let mut runtime = RUNTIME.lock();
    if !matches!(&runtime.provisioning, ProvisioningState::Reserved)
        || runtime.region != Some(region)
        || runtime.approved_policy != Some(policy)
        || !runtime.vault_history_empty
        || runtime.wrappers.is_some()
    {
        runtime.provisioning = ProvisioningState::Idle;
        return Err(VaultFacadeDenied::RecoveryUnavailable);
    }
    runtime.provisioning = ProvisioningState::Awaiting(InitialRecoverySession {
        vmk,
        candidate,
        region,
        policy,
    });
    Ok(RecoveryKeyDisplay {
        text,
        owns_session: true,
    })
}

pub(crate) fn confirm_initial_recovery(
    mut confirmation: RecoveryKeyConfirmation,
    reentered: &mut [u8],
) -> Result<VaultBrokerStatus, VaultFacadeDenied> {
    confirmation.owns_session = false;
    let session = {
        let mut runtime = RUNTIME.lock();
        let prior = core::mem::replace(&mut runtime.provisioning, ProvisioningState::Confirming);
        match prior {
            ProvisioningState::Awaiting(session) => session,
            other => {
                runtime.provisioning = other;
                keyring::zeroize_recovery_input(reentered);
                return Err(VaultFacadeDenied::RecoverySessionMissing);
            }
        }
    };

    let recovery_key = match session.candidate.confirm(reentered) {
        Ok(key) => key,
        Err(error) => {
            reset_pre_io_provisioning();
            return Err(VaultFacadeDenied::Keyring(error));
        }
    };
    let context = RecoveryKekContext {
        store_uuid: session.region.store.store_uuid,
        key_epoch: 1,
        wrapper_generation: 1,
        core_generation: session.policy.core_generation(),
        policy_id_sha256: session.policy.policy_id_sha256(),
    };
    let mut proposed_slots =
        match RecoveryWrapperSlots::provision_initial(&session.vmk, &recovery_key, context) {
            Ok(slots) => slots,
            Err(error) => {
                reset_pre_io_provisioning();
                return Err(VaultFacadeDenied::Keyring(error));
            }
        };
    let payload = match encode_recovery_wrapper(proposed_slots.current_wrapper()) {
        Ok(payload) => payload,
        Err(error) => {
            reset_pre_io_provisioning();
            return Err(VaultFacadeDenied::Store(error));
        }
    };

    {
        let mut runtime = RUNTIME.lock();
        if !matches!(&runtime.provisioning, ProvisioningState::Confirming)
            || runtime.region != Some(session.region)
            || runtime.approved_policy != Some(session.policy)
            || !runtime.vault_history_empty
            || runtime.wrappers.is_some()
        {
            runtime.provisioning = ProvisioningState::Idle;
            return Err(VaultFacadeDenied::RecoveryUnavailable);
        }
        runtime.provisioning = ProvisioningState::PersistenceOutcomeUncertain;
    }

    let replay =
        crate::structured_store_c1::commit_initial_recovery_wrapper(session.region, &payload)
            .map_err(VaultFacadeDenied::RecoveryStore)?;
    let persisted =
        recovery_wrapper_from_replay(&replay)?.ok_or(VaultFacadeDenied::RecoveryWrapperMissing)?;
    proposed_slots
        .confirm_current_readback(&persisted)
        .map_err(VaultFacadeDenied::Keyring)?;
    let _ = load_verified_replay(&replay)?;

    let status = {
        let mut runtime = RUNTIME.lock();
        if runtime.region != Some(session.region)
            || runtime.approved_policy != Some(session.policy)
            || !matches!(
                &runtime.provisioning,
                ProvisioningState::PersistenceOutcomeUncertain
            )
        {
            return Err(VaultFacadeDenied::PersistenceOutcomeUncertain);
        }
        let wrappers = runtime
            .wrappers
            .take()
            .ok_or(VaultFacadeDenied::RecoveryWrapperMissing)?;
        let unlock = runtime
            .broker
            .unlock_with_recovery_key(&recovery_key, &wrappers)
            .map_err(VaultFacadeDenied::Broker);
        runtime.wrappers = Some(wrappers);
        unlock?;
        runtime.provisioning = ProvisioningState::Idle;
        runtime.broker.status()
    };
    serial::write_line("VAULT_BROKER_UNLOCKED source=recovery slot=current");
    Ok(status)
}

pub(crate) fn unlock_with_recovery_key(
    reentered: &mut [u8],
) -> Result<VaultBrokerStatus, VaultFacadeDenied> {
    let recovery_key =
        keyring::parse_recovery_key_input(reentered).map_err(VaultFacadeDenied::Keyring)?;
    let status = {
        let mut runtime = RUNTIME.lock();
        let wrappers = runtime
            .wrappers
            .take()
            .ok_or(VaultFacadeDenied::RecoveryWrapperMissing)?;
        let unlock = runtime
            .broker
            .unlock_with_recovery_key(&recovery_key, &wrappers)
            .map_err(VaultFacadeDenied::Broker);
        runtime.wrappers = Some(wrappers);
        unlock?;
        runtime.broker.status()
    };
    serial::write_line("VAULT_BROKER_UNLOCKED source=recovery slot=current");
    Ok(status)
}

pub(crate) fn status() -> VaultBrokerStatus {
    RUNTIME.lock().broker.status()
}

pub(crate) fn provider_status() -> VaultSecretStatus {
    RUNTIME.lock().broker.status().provider
}

pub(crate) fn wifi_status() -> VaultSecretStatus {
    RUNTIME.lock().broker.status().wifi
}

/// True only while this boot is bound to the exact disposable C1 QEMU store.
/// It exists solely to hide the contained proof UI on every other target.
pub(crate) fn contained_qemu_wifi_test_available() -> bool {
    let Some(region) = RUNTIME.lock().region else {
        return false;
    };
    crate::structured_store_c1::revalidate_qemu_wifi_store_identity(region).is_ok()
}

/// Persists only one WPA2-PSK/CCMP credential bound to the selected live BSS.
/// Once media I/O starts, any unproven outcome remains denied for this boot;
/// only exact readback plus a fresh full replay clears it.
pub(crate) fn save_or_replace_wifi(
    ssid: &[u8],
    bssid: [u8; 6],
    plaintext: SecretPlaintext,
) -> Result<VaultBrokerStatus, VaultFacadeDenied> {
    let (region, expected_committed_version, proposed_version, payload) = {
        let mut runtime = RUNTIME.lock();
        if runtime.wifi_write_outcome_uncertain {
            return Err(VaultFacadeDenied::PersistenceOutcomeUncertain);
        }
        if !runtime.replay_loaded {
            return Err(VaultFacadeDenied::HistoryLocked);
        }
        let expected_committed_version = match runtime.broker.status().wifi {
            VaultSecretStatus::Missing => None,
            VaultSecretStatus::Available { record_version, .. }
            | VaultSecretStatus::Forgotten { record_version } => Some(record_version),
        };
        let envelope = runtime
            .broker
            .save_or_replace_wifi(
                broker::VaultMutation::genesis_secure_overlay(),
                ssid,
                bssid,
                plaintext,
            )
            .map_err(VaultFacadeDenied::Broker)?
            .into_store_envelope();
        let proposed_version = envelope.binding.record_version;
        let payload = encode_secret_envelope(VaultRecordId::WifiPassphrase, &envelope)
            .map_err(VaultFacadeDenied::Store)?;
        let region = runtime.region.ok_or(VaultFacadeDenied::HistoryLocked)?;
        runtime.wifi_write_outcome_uncertain = true;
        (
            region,
            expected_committed_version,
            proposed_version,
            payload,
        )
    };

    let replay = crate::structured_store_c1::commit_wifi_secret(
        region,
        expected_committed_version,
        proposed_version,
        &payload,
    )
    .map_err(VaultFacadeDenied::WifiStore)?;
    let _ = load_verified_replay(&replay)?;

    let mut runtime = RUNTIME.lock();
    let status = runtime.broker.status();
    if runtime.region != Some(region)
        || !matches!(
            status.wifi,
            VaultSecretStatus::Available { record_version, .. }
                if record_version == proposed_version
        )
        || !runtime.wifi_write_outcome_uncertain
    {
        return Err(VaultFacadeDenied::PersistenceOutcomeUncertain);
    }
    runtime.wifi_write_outcome_uncertain = false;
    serial::write_fmt(format_args!(
        "VAULT_WIFI_STORED version={} readback=verified\r\n",
        proposed_version
    ));
    Ok(status)
}

/// Test-media-only save adapter for the repo's existing fixed WPA2 beacon.
/// Neither UI nor harness may supply a target to this path.
pub(crate) fn save_or_replace_contained_qemu_wifi_for_test(
    plaintext: SecretPlaintext,
) -> Result<VaultBrokerStatus, VaultFacadeDenied> {
    let region = RUNTIME
        .lock()
        .region
        .ok_or(VaultFacadeDenied::HistoryLocked)?;
    crate::structured_store_c1::revalidate_qemu_wifi_store_identity(region)
        .map_err(VaultFacadeDenied::WifiStore)?;
    save_or_replace_wifi(CONTAINED_WIFI_SSID, CONTAINED_WIFI_BSSID, plaintext)
}

/// Persists only the fixed direct-OpenAI credential from the trusted Genesis
/// overlay. Once media I/O starts, any unproven outcome remains denied for the
/// rest of this boot; only exact readback plus a fresh full replay clears it.
pub(crate) fn save_or_replace_provider(
    plaintext: SecretPlaintext,
) -> Result<VaultBrokerStatus, VaultFacadeDenied> {
    let (region, expected_committed_version, proposed_version, payload) = {
        let mut runtime = RUNTIME.lock();
        if runtime.provider_write_outcome_uncertain {
            return Err(VaultFacadeDenied::PersistenceOutcomeUncertain);
        }
        if !runtime.replay_loaded {
            return Err(VaultFacadeDenied::HistoryLocked);
        }
        let expected_committed_version = match runtime.broker.status().provider {
            VaultSecretStatus::Missing => None,
            VaultSecretStatus::Available { record_version, .. }
            | VaultSecretStatus::Forgotten { record_version } => Some(record_version),
        };
        let envelope = runtime
            .broker
            .save_or_replace_provider(broker::VaultMutation::genesis_secure_overlay(), plaintext)
            .map_err(VaultFacadeDenied::Broker)?
            .into_store_envelope();
        let proposed_version = envelope.binding.record_version;
        let payload = encode_secret_envelope(VaultRecordId::ProviderApiKey, &envelope)
            .map_err(VaultFacadeDenied::Store)?;
        let region = runtime.region.ok_or(VaultFacadeDenied::HistoryLocked)?;
        runtime.provider_write_outcome_uncertain = true;
        (
            region,
            expected_committed_version,
            proposed_version,
            payload,
        )
    };

    let replay = crate::structured_store_c1::commit_provider_secret(
        region,
        expected_committed_version,
        proposed_version,
        &payload,
    )
    .map_err(VaultFacadeDenied::ProviderStore)?;
    let _ = load_verified_replay(&replay)?;

    let mut runtime = RUNTIME.lock();
    let status = runtime.broker.status();
    if runtime.region != Some(region)
        || !matches!(
            status.provider,
            VaultSecretStatus::Available { record_version, .. }
                if record_version == proposed_version
        )
        || !runtime.provider_write_outcome_uncertain
    {
        return Err(VaultFacadeDenied::PersistenceOutcomeUncertain);
    }
    runtime.provider_write_outcome_uncertain = false;
    serial::write_fmt(format_args!(
        "VAULT_PROVIDER_STORED version={} readback=verified\r\n",
        proposed_version
    ));
    Ok(status)
}

/// The only production WiFi-secret consumer entrypoint. It performs exact
/// target/store/core checks, commits the metadata-only pre-use audit, rechecks
/// all state, then consumes the one-use lease directly into the NXP command.
pub(crate) fn write_wifi_pmk_for_association(
    seq: u16,
    ssid: &[u8],
    bssid: [u8; 6],
    out: &mut [u8],
) -> Result<usize, VaultFacadeDenied> {
    wifi_lease_after_durable_audit(ssid, bssid, false)?
        .into_wpa2_psk_ccmp_pmk_set(seq, bssid, ssid, out)
        .map_err(VaultFacadeDenied::Consumer)
}

/// Contained test infrastructure for the real WiFi lease path. It accepts
/// only the repo's fixed `RaiWPA2` beacon on the exact re-identified C1 store,
/// formats one local command, and claims no radio, association, link, or DHCP.
pub(crate) fn run_contained_qemu_wifi_consumer_test() -> Result<(), VaultFacadeDenied> {
    let (region, _, _, _, _) = current_wifi_use_facts(CONTAINED_WIFI_SSID, CONTAINED_WIFI_BSSID)?;
    crate::structured_store_c1::revalidate_qemu_wifi_store_identity(region)
        .map_err(VaultFacadeDenied::WifiStore)?;

    match current_wifi_use_facts(CONTAINED_WIFI_SSID, CONTAINED_WIFI_WRONG_BSSID) {
        Err(VaultFacadeDenied::Broker(VaultBrokerDenied::Use(
            raios_core::scoped_secret_use::SecretUseDenial::TargetMismatch,
        ))) => {}
        Err(error) => return Err(error),
        Ok(_) => return Err(VaultFacadeDenied::ContainedWifiValidationFailed),
    }
    serial::write_line(
        "VAULT_WIFI_WRONG_BSSID_DENIED reason=target_mismatch test_infrastructure=true",
    );

    {
        let runtime = RUNTIME.lock();
        runtime
            .broker
            .prove_wifi_auditless_use_denied_for_contained_test(
                CONTAINED_WIFI_SSID,
                CONTAINED_WIFI_BSSID,
            )
            .map_err(VaultFacadeDenied::Broker)?;
    }
    serial::write_line(
        "VAULT_WIFI_AUDITLESS_DENIED reason=audit_binding_missing test_infrastructure=true",
    );

    let lease = wifi_lease_after_durable_audit(CONTAINED_WIFI_SSID, CONTAINED_WIFI_BSSID, true)?;
    let mut validator = ContainedWifiCommandValidator::new();
    if !validator.consume(lease)? {
        return Err(VaultFacadeDenied::ContainedWifiValidationFailed);
    }
    serial::write_line(
        "VAULT_WIFI_CONTAINED_CONSUMED target=bound_bss accepted=true test_infrastructure=true",
    );
    Ok(())
}

fn wifi_lease_after_durable_audit(
    ssid: &[u8],
    bssid: [u8; 6],
    test_infrastructure: bool,
) -> Result<WifiSecretLease, VaultFacadeDenied> {
    let (region, policy, record_version, key_epoch, target_binding_sha256) =
        current_wifi_use_facts(ssid, bssid)?;
    crate::structured_store_c1::revalidate_qemu_wifi_store_identity(region)
        .map_err(VaultFacadeDenied::WifiStore)?;
    let binding = WifiUseAuditBinding::from_current_wifi_slot(
        region.store.store_uuid,
        target_binding_sha256,
        record_version,
        key_epoch,
        policy.core_generation(),
        policy.policy_id_sha256(),
        test_infrastructure,
    )
    .map_err(VaultFacadeDenied::WifiAudit)?;
    let receipt = append_wifi_use_audit(binding).map_err(VaultFacadeDenied::WifiAudit)?;

    crate::structured_store_c1::revalidate_qemu_wifi_store_identity(region)
        .map_err(VaultFacadeDenied::WifiStore)?;
    let verified = crate::core_policy_runtime::verified().map_err(VaultFacadeDenied::CorePolicy)?;
    let current_policy = ApprovedCorePolicy::from_verified(verified);
    let runtime = RUNTIME.lock();
    let current_wifi = runtime.broker.status().wifi;
    let current_target_sha256 = runtime
        .broker
        .require_wifi_target(ssid, bssid)
        .map_err(VaultFacadeDenied::Broker)?;
    if runtime.wifi_write_outcome_uncertain
        || !runtime.replay_loaded
        || runtime.region != Some(region)
        || runtime.approved_policy != Some(policy)
        || current_policy != policy
        || !matches!(
            runtime.broker.status().lock,
            VaultLockStatus::UnlockedFromRecovery(_)
        )
        || current_wifi
            != (VaultSecretStatus::Available {
                record_version,
                key_epoch,
            })
        || current_target_sha256 != target_binding_sha256
    {
        return Err(VaultFacadeDenied::WifiUseStateChanged);
    }
    let lease = runtime
        .broker
        .use_for_wifi(ssid, bssid, receipt)
        .map_err(VaultFacadeDenied::Broker)?;
    serial::write_line(
        "VAULT_WIFI_PREUSE_AUDIT_COMMITTED consumer=native_wifi_supplicant operation=associate_bound_bss readback=verified reparse=verified rescan=verified",
    );
    Ok(lease)
}

fn current_wifi_use_facts(
    ssid: &[u8],
    bssid: [u8; 6],
) -> Result<
    (
        ValidatedRegionIdentity,
        ApprovedCorePolicy,
        u64,
        u64,
        [u8; 32],
    ),
    VaultFacadeDenied,
> {
    let verified = crate::core_policy_runtime::verified().map_err(VaultFacadeDenied::CorePolicy)?;
    let current_policy = ApprovedCorePolicy::from_verified(verified);
    let runtime = RUNTIME.lock();
    if runtime.wifi_write_outcome_uncertain {
        return Err(VaultFacadeDenied::PersistenceOutcomeUncertain);
    }
    if !runtime.replay_loaded {
        return Err(VaultFacadeDenied::HistoryLocked);
    }
    if !matches!(
        runtime.broker.status().lock,
        VaultLockStatus::UnlockedFromRecovery(_)
    ) {
        return Err(VaultFacadeDenied::Broker(VaultBrokerDenied::Locked));
    }
    let region = runtime.region.ok_or(VaultFacadeDenied::HistoryLocked)?;
    let policy = runtime
        .approved_policy
        .ok_or(VaultFacadeDenied::CorePolicy("core_policy_missing"))?;
    if policy != current_policy {
        return Err(VaultFacadeDenied::WifiUseStateChanged);
    }
    let (record_version, key_epoch) = match runtime.broker.status().wifi {
        VaultSecretStatus::Available {
            record_version,
            key_epoch,
        } => (record_version, key_epoch),
        VaultSecretStatus::Missing => {
            return Err(VaultFacadeDenied::Broker(VaultBrokerDenied::SecretMissing));
        }
        VaultSecretStatus::Forgotten { .. } => {
            return Err(VaultFacadeDenied::Broker(
                VaultBrokerDenied::SecretForgotten,
            ));
        }
    };
    let target_binding_sha256 = runtime
        .broker
        .require_wifi_target(ssid, bssid)
        .map_err(VaultFacadeDenied::Broker)?;
    Ok((
        region,
        policy,
        record_version,
        key_epoch,
        target_binding_sha256,
    ))
}

/// Production entrypoint: only the provider module's moved, opaque verifier
/// token can start the durable pre-use audit and mint one provider lease.
pub(crate) fn provider_lease_for_verified_request(
    verified: crate::provider_trust::VerifiedOpenAiProviderTrust,
) -> Result<ProviderSecretLease, VaultFacadeDenied> {
    let trust =
        ProviderRequestTrust::from_verified_openai(verified).map_err(VaultFacadeDenied::Audit)?;
    provider_lease_after_durable_audit(trust)
}

/// Contained QEMU test infrastructure for the real provider lease path. It
/// runs only against the exact re-identified C1 fixture and never performs a
/// network request or returns/logs the formatted header.
pub(crate) fn run_contained_qemu_provider_consumer_test() -> Result<(), VaultFacadeDenied> {
    let (region, _, _, _) = current_provider_use_facts()?;
    crate::structured_store_c1::revalidate_qemu_store_identity(region)
        .map_err(VaultFacadeDenied::ProviderStore)?;
    {
        let runtime = RUNTIME.lock();
        runtime
            .broker
            .prove_provider_auditless_use_denied_for_contained_test()
            .map_err(VaultFacadeDenied::Broker)?;
    }
    serial::write_line(
        "VAULT_PROVIDER_AUDITLESS_DENIED reason=audit_binding_missing test_infrastructure=true",
    );

    let mut fixture_evidence = [0u8; 80];
    fixture_evidence[..32].copy_from_slice(&region.device_identity_sha256);
    fixture_evidence[32..48].copy_from_slice(&region.gpt_disk_guid);
    fixture_evidence[48..64].copy_from_slice(&region.store.store_uuid);
    fixture_evidence[64..].copy_from_slice(b"provider-sink-v1");
    let trust = ProviderRequestTrust::contained_qemu_validator(sha256_bytes(&fixture_evidence))
        .map_err(VaultFacadeDenied::Audit)?;
    let lease = provider_lease_after_durable_audit(trust)?;
    let mut validator = ContainedProviderHeaderValidator::new();
    lease
        .into_openai_authorization_header(&mut validator)
        .map_err(VaultFacadeDenied::Consumer)?;
    if !validator.accepted() {
        return Err(VaultFacadeDenied::ContainedProviderValidationFailed);
    }
    serial::write_line(
        "VAULT_PROVIDER_CONTAINED_CONSUMED target=api.openai.com accepted=true test_infrastructure=true",
    );
    Ok(())
}

fn provider_lease_after_durable_audit(
    trust: ProviderRequestTrust,
) -> Result<ProviderSecretLease, VaultFacadeDenied> {
    let (region, policy, record_version, key_epoch) = current_provider_use_facts()?;
    crate::structured_store_c1::revalidate_qemu_store_identity(region)
        .map_err(VaultFacadeDenied::ProviderStore)?;
    let binding = ProviderUseAuditBinding::from_current_provider_slot(
        region.store.store_uuid,
        record_version,
        key_epoch,
        policy.core_generation(),
        policy.policy_id_sha256(),
        trust,
    )
    .map_err(VaultFacadeDenied::Audit)?;
    let receipt = append_provider_use_audit(binding).map_err(VaultFacadeDenied::Audit)?;

    crate::structured_store_c1::revalidate_qemu_store_identity(region)
        .map_err(VaultFacadeDenied::ProviderStore)?;
    let verified = crate::core_policy_runtime::verified().map_err(VaultFacadeDenied::CorePolicy)?;
    let current_policy = ApprovedCorePolicy::from_verified(verified);
    let runtime = RUNTIME.lock();
    let current_provider = runtime.broker.status().provider;
    if runtime.provider_write_outcome_uncertain
        || !runtime.replay_loaded
        || runtime.region != Some(region)
        || runtime.approved_policy != Some(policy)
        || current_policy != policy
        || !matches!(
            runtime.broker.status().lock,
            VaultLockStatus::UnlockedFromRecovery(_)
        )
        || current_provider
            != (VaultSecretStatus::Available {
                record_version,
                key_epoch,
            })
    {
        return Err(VaultFacadeDenied::ProviderUseStateChanged);
    }
    let lease = runtime
        .broker
        .use_for_provider(receipt)
        .map_err(VaultFacadeDenied::Broker)?;
    serial::write_line(
        "VAULT_PROVIDER_PREUSE_AUDIT_COMMITTED consumer=openai_direct operation=request_exact_host readback=verified reparse=verified rescan=verified",
    );
    Ok(lease)
}

fn current_provider_use_facts(
) -> Result<(ValidatedRegionIdentity, ApprovedCorePolicy, u64, u64), VaultFacadeDenied> {
    let verified = crate::core_policy_runtime::verified().map_err(VaultFacadeDenied::CorePolicy)?;
    let current_policy = ApprovedCorePolicy::from_verified(verified);
    let runtime = RUNTIME.lock();
    if runtime.provider_write_outcome_uncertain {
        return Err(VaultFacadeDenied::PersistenceOutcomeUncertain);
    }
    if !runtime.replay_loaded {
        return Err(VaultFacadeDenied::HistoryLocked);
    }
    if !matches!(
        runtime.broker.status().lock,
        VaultLockStatus::UnlockedFromRecovery(_)
    ) {
        return Err(VaultFacadeDenied::Broker(VaultBrokerDenied::Locked));
    }
    let region = runtime.region.ok_or(VaultFacadeDenied::HistoryLocked)?;
    let policy = runtime
        .approved_policy
        .ok_or(VaultFacadeDenied::CorePolicy("core_policy_missing"))?;
    if policy != current_policy {
        return Err(VaultFacadeDenied::ProviderUseStateChanged);
    }
    let (record_version, key_epoch) = match runtime.broker.status().provider {
        VaultSecretStatus::Available {
            record_version,
            key_epoch,
        } => (record_version, key_epoch),
        VaultSecretStatus::Missing | VaultSecretStatus::Forgotten { .. } => {
            return Err(VaultFacadeDenied::Broker(VaultBrokerDenied::SecretMissing));
        }
    };
    Ok((region, policy, record_version, key_epoch))
}

const CONTAINED_WIFI_SEQ: u16 = 0x4001;

struct ContainedWifiCommandValidator {
    bytes: [u8; raios_core::marvell_wifi_supplicant::SUPPLICANT_PMK_SET_MAX_TOTAL_LEN],
}

impl ContainedWifiCommandValidator {
    const fn new() -> Self {
        Self {
            bytes: [0; raios_core::marvell_wifi_supplicant::SUPPLICANT_PMK_SET_MAX_TOTAL_LEN],
        }
    }

    fn consume(&mut self, lease: WifiSecretLease) -> Result<bool, VaultFacadeDenied> {
        let len = lease
            .into_wpa2_psk_ccmp_pmk_set(
                CONTAINED_WIFI_SEQ,
                CONTAINED_WIFI_BSSID,
                CONTAINED_WIFI_SSID,
                &mut self.bytes,
            )
            .map_err(VaultFacadeDenied::Consumer)?;
        let bssid_offset = 20 + CONTAINED_WIFI_SSID.len() + 4;
        Ok(len <= self.bytes.len()
            && len > bssid_offset + CONTAINED_WIFI_BSSID.len()
            && read_le_u16(&self.bytes, 0) as usize == len
            && read_le_u16(&self.bytes, 2) == 1
            && read_le_u16(&self.bytes, 4)
                == raios_core::marvell_wifi_supplicant::SUPPLICANT_PMK_CMD
            && read_le_u16(&self.bytes, 8) == CONTAINED_WIFI_SEQ
            && self.bytes[20..20 + CONTAINED_WIFI_SSID.len()] == *CONTAINED_WIFI_SSID
            && self.bytes[bssid_offset..bssid_offset + CONTAINED_WIFI_BSSID.len()]
                == CONTAINED_WIFI_BSSID)
    }
}

impl Drop for ContainedWifiCommandValidator {
    fn drop(&mut self) {
        keyring::zeroize_recovery_input(&mut self.bytes);
    }
}

fn read_le_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

const OPENAI_AUTHORIZATION_PREFIX: &[u8] = b"Authorization: Bearer ";
const OPENAI_AUTHORIZATION_CAPACITY: usize =
    OPENAI_AUTHORIZATION_PREFIX.len() + MAX_PROVIDER_API_KEY_LEN + 2;

struct ContainedProviderHeaderValidator {
    bytes: [u8; OPENAI_AUTHORIZATION_CAPACITY],
    len: usize,
}

impl ContainedProviderHeaderValidator {
    const fn new() -> Self {
        Self {
            bytes: [0; OPENAI_AUTHORIZATION_CAPACITY],
            len: 0,
        }
    }

    fn accepted(&self) -> bool {
        if self.len <= OPENAI_AUTHORIZATION_PREFIX.len() + 2
            || self.len > OPENAI_AUTHORIZATION_CAPACITY
            || !self.bytes[..self.len].starts_with(OPENAI_AUTHORIZATION_PREFIX)
            || !self.bytes[..self.len].ends_with(b"\r\n")
        {
            return false;
        }
        self.bytes[OPENAI_AUTHORIZATION_PREFIX.len()..self.len - 2]
            .iter()
            .all(u8::is_ascii_graphic)
    }
}

impl embedded_io::ErrorType for ContainedProviderHeaderValidator {
    type Error = embedded_io::ErrorKind;
}

impl embedded_io::Write for ContainedProviderHeaderValidator {
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        let end = self
            .len
            .checked_add(bytes.len())
            .filter(|end| *end <= self.bytes.len())
            .ok_or(embedded_io::ErrorKind::InvalidInput)?;
        self.bytes[self.len..end].copy_from_slice(bytes);
        self.len = end;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Drop for ContainedProviderHeaderValidator {
    fn drop(&mut self) {
        keyring::zeroize_recovery_input(&mut self.bytes);
        self.len = 0;
    }
}

pub(crate) fn recovery_wrapper_generation() -> Option<u64> {
    RUNTIME
        .lock()
        .wrappers
        .as_ref()
        .map(|wrappers| wrappers.current_wrapper().context.wrapper_generation)
}

pub(crate) const fn recovery_wrapper_record_key() -> RecordKey {
    VaultRecordId::RecoveryWrapper.key()
}

pub(crate) const fn provider_record_key() -> RecordKey {
    VaultRecordId::ProviderApiKey.key()
}

pub(crate) const fn wifi_record_key() -> RecordKey {
    VaultRecordId::WifiPassphrase.key()
}

pub(crate) fn is_vault_namespace(namespace: [u8; 16]) -> bool {
    namespace == VAULT_NAMESPACE
}

fn restore_replayed_wrapper(runtime: &mut VaultRuntime) -> Result<(), VaultFacadeDenied> {
    let Some(policy) = runtime.approved_policy else {
        return Ok(());
    };
    let Some(wrapper) = runtime.replayed_wrapper.take() else {
        return Ok(());
    };
    runtime.wrappers = Some(
        RecoveryWrapperSlots::restore_current_from_verified_replay(wrapper, policy)
            .map_err(VaultFacadeDenied::Keyring)?,
    );
    Ok(())
}

fn recovery_wrapper_from_replay(
    replay: &ValidatedReplayWithHistory,
) -> Result<Option<RecoveryVmkWrapperV1>, VaultFacadeDenied> {
    let identity = replay.identity().store;
    let Some(record) = replay
        .state()
        .record(VaultRecordId::RecoveryWrapper.key())
        .map_err(VaultFacadeDenied::StructuredStore)?
    else {
        return Ok(None);
    };
    let decoded = ReplayVerifiedVaultRecord::from_committed(identity, record)
        .map_err(VaultFacadeDenied::Store)?
        .decode()
        .map_err(VaultFacadeDenied::Store)?;
    match decoded {
        DecodedVaultRecord::RecoveryWrapper(wrapper) => Ok(Some(wrapper)),
        DecodedVaultRecord::Tombstone(VaultRecordId::RecoveryWrapper) => Ok(None),
        _ => Err(VaultFacadeDenied::RecoveryWrapperMissing),
    }
}

fn cancel_pending_session() {
    let mut runtime = RUNTIME.lock();
    if matches!(&runtime.provisioning, ProvisioningState::Awaiting(_)) {
        runtime.provisioning = ProvisioningState::Idle;
    }
}

fn reset_pre_io_provisioning() {
    let mut runtime = RUNTIME.lock();
    if matches!(
        &runtime.provisioning,
        ProvisioningState::Reserved | ProvisioningState::Confirming
    ) {
        runtime.provisioning = ProvisioningState::Idle;
    }
}
