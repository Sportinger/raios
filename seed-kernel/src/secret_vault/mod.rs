//! Core-only Secret Vault custody. No service-facing secret accessor lives here.

use alloc::vec::Vec;
use spin::Mutex;

use raios_core::{
    secret_vault::{RecoveryKekContext, RecoveryVmkWrapperV1, VaultMasterKey},
    structured_store::{RecordKey, ReplayTail, StoreDenied},
};

use self::{
    broker::{VaultBroker, VaultBrokerDenied, VaultCompleteReplay, VaultLockStatus},
    keyring::{
        ApprovedCorePolicy, KeyringDenied, RecoveryKeyCandidate, RecoveryKeyText,
        RecoveryWrapperSlots,
    },
    store::{
        encode_recovery_wrapper, DecodedVaultRecord, ReplayVerifiedVaultRecord, VaultRecordId,
        VaultStoreDenied, VAULT_NAMESPACE,
    },
};
use crate::{
    serial,
    structured_store::{ValidatedRegionIdentity, ValidatedReplayWithHistory},
};

mod broker;
mod keyring;
mod store;

pub(crate) use self::{broker::VaultBrokerStatus, store::RECOVERY_WRAPPER_PAYLOAD_LEN};

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
    Broker(VaultBrokerDenied),
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
            Self::Broker(_) => "broker_denied",
        }
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
