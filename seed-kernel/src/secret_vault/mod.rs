//! Core-only Secret Vault custody. No service-facing secret accessor lives here.

use alloc::vec::Vec;
use spin::Mutex;

use self::{
    broker::{VaultBroker, VaultBrokerDenied, VaultCompleteReplay},
    store::VAULT_NAMESPACE,
};
use crate::structured_store::ValidatedReplayWithHistory;

mod broker;
mod keyring;
mod store;

pub(crate) use self::broker::VaultBrokerStatus;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VaultFacadeDenied {
    HistoryLocked,
    Broker(VaultBrokerDenied),
}

static BROKER: Mutex<VaultBroker> = Mutex::new(VaultBroker::new());

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

    let identity = replay.identity().store;
    if replay.state().identity != identity {
        return Err(VaultFacadeDenied::Broker(
            VaultBrokerDenied::StoreIdentityMismatch,
        ));
    }
    let complete = VaultCompleteReplay::from_complete_history(identity, &vault_records)
        .map_err(VaultFacadeDenied::Broker)?;
    let mut broker = BROKER.lock();
    broker
        .load_complete_replay(complete)
        .map_err(VaultFacadeDenied::Broker)?;
    Ok(broker.status())
}

pub(crate) fn status() -> VaultBrokerStatus {
    BROKER.lock().status()
}
