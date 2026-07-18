//! Pure Secret Vault V1 use authorization.
//!
//! A positive result is metadata only. This module never receives, decrypts,
//! returns, formats, or logs plaintext.

use crate::secret_vault::{
    SecretConsumer, SecretKind, SecretOperation, SecretTargetBinding, SecretTargetKind,
    OPENAI_API_HOST,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretBootScope {
    CurrentBoot,
    SafeRecovery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecretUseEvidence {
    pub vault_unlocked: bool,
    pub boot_scope: SecretBootScope,
    pub explicit_recovery_action: bool,
    pub service_generation: u64,
    pub current_service_generation: u64,
    pub record_version: u64,
    pub current_record_version: u64,
    pub key_epoch: u64,
    pub current_key_epoch: u64,
    pub trust_decision_positive: bool,
    pub committed_store_record_verified: bool,
    pub audit_binding_present: bool,
}

#[derive(Clone, Copy)]
pub struct SecretUseRequest<'a> {
    pub kind: SecretKind,
    pub consumer: SecretConsumer,
    pub operation: SecretOperation,
    pub bound_target: SecretTargetBinding,
    pub requested_target: SecretTargetBinding,
    pub provider_host: Option<&'a str>,
    pub evidence: SecretUseEvidence,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SecretUseAuthorization {
    kind: SecretKind,
    consumer: SecretConsumer,
    operation: SecretOperation,
    target: SecretTargetBinding,
    boot_scope: SecretBootScope,
    service_generation: u64,
    record_version: u64,
    key_epoch: u64,
}

impl SecretUseAuthorization {
    pub const fn kind(&self) -> SecretKind {
        self.kind
    }

    pub const fn consumer(&self) -> SecretConsumer {
        self.consumer
    }

    pub const fn operation(&self) -> SecretOperation {
        self.operation
    }

    pub const fn target(&self) -> SecretTargetBinding {
        self.target
    }

    pub const fn boot_scope(&self) -> SecretBootScope {
        self.boot_scope
    }

    pub const fn service_generation(&self) -> u64 {
        self.service_generation
    }

    pub const fn record_version(&self) -> u64 {
        self.record_version
    }

    pub const fn key_epoch(&self) -> u64 {
        self.key_epoch
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretUseDenial {
    SecretKindMismatch,
    ConsumerMismatch,
    OperationMismatch,
    TargetKindMismatch,
    TargetMismatch,
    ProviderHostMissing,
    ProviderHostMismatch,
    UnexpectedProviderHost,
    VaultLocked,
    SafeRecoveryActionMissing,
    InvalidServiceGeneration,
    ServiceGenerationMismatch,
    InvalidRecordVersion,
    RecordVersionMismatch,
    InvalidKeyEpoch,
    KeyEpochMismatch,
    TrustDecisionMissing,
    CommittedStoreEvidenceMissing,
    AuditBindingMissing,
}

pub type SecretUseDecision = Result<SecretUseAuthorization, SecretUseDenial>;

/// Authorizes only one bound WPA2 supplicant association or one OpenAI request.
pub fn authorize_secret_use(request: &SecretUseRequest<'_>) -> SecretUseDecision {
    match request.bound_target.kind() {
        SecretTargetKind::WifiBoundBss => authorize_wifi_shape(request)?,
        SecretTargetKind::ProviderExactHost => authorize_provider_shape(request)?,
    }
    authorize_evidence(&request.evidence)?;

    Ok(SecretUseAuthorization {
        kind: request.kind,
        consumer: request.consumer,
        operation: request.operation,
        target: request.requested_target,
        boot_scope: request.evidence.boot_scope,
        service_generation: request.evidence.service_generation,
        record_version: request.evidence.record_version,
        key_epoch: request.evidence.key_epoch,
    })
}

fn authorize_wifi_shape(request: &SecretUseRequest<'_>) -> Result<(), SecretUseDenial> {
    if request.kind != SecretKind::WifiPassphrase {
        return Err(SecretUseDenial::SecretKindMismatch);
    }
    if request.consumer != SecretConsumer::NativeWifiSupplicant {
        return Err(SecretUseDenial::ConsumerMismatch);
    }
    if request.operation != SecretOperation::AssociateBoundBss {
        return Err(SecretUseDenial::OperationMismatch);
    }
    if request.requested_target.kind() != SecretTargetKind::WifiBoundBss {
        return Err(SecretUseDenial::TargetKindMismatch);
    }
    if request.requested_target != request.bound_target {
        return Err(SecretUseDenial::TargetMismatch);
    }
    if request.provider_host.is_some() {
        return Err(SecretUseDenial::UnexpectedProviderHost);
    }
    Ok(())
}

fn authorize_provider_shape(request: &SecretUseRequest<'_>) -> Result<(), SecretUseDenial> {
    if request.kind != SecretKind::ProviderApiKey {
        return Err(SecretUseDenial::SecretKindMismatch);
    }
    if request.consumer != SecretConsumer::OpenAiDirect {
        return Err(SecretUseDenial::ConsumerMismatch);
    }
    if request.operation != SecretOperation::RequestExactHost {
        return Err(SecretUseDenial::OperationMismatch);
    }
    if request.requested_target.kind() != SecretTargetKind::ProviderExactHost {
        return Err(SecretUseDenial::TargetKindMismatch);
    }
    if request.requested_target != request.bound_target {
        return Err(SecretUseDenial::TargetMismatch);
    }
    match request.provider_host {
        None => return Err(SecretUseDenial::ProviderHostMissing),
        Some(host) if host != OPENAI_API_HOST => return Err(SecretUseDenial::ProviderHostMismatch),
        Some(_) => {}
    }
    let exact_openai_target = SecretTargetBinding::for_openai_host(OPENAI_API_HOST)
        .map_err(|_| SecretUseDenial::ProviderHostMismatch)?;
    if request.bound_target != exact_openai_target {
        return Err(SecretUseDenial::TargetMismatch);
    }
    Ok(())
}

fn authorize_evidence(evidence: &SecretUseEvidence) -> Result<(), SecretUseDenial> {
    if !evidence.vault_unlocked {
        return Err(SecretUseDenial::VaultLocked);
    }
    if evidence.boot_scope == SecretBootScope::SafeRecovery && !evidence.explicit_recovery_action {
        return Err(SecretUseDenial::SafeRecoveryActionMissing);
    }
    if evidence.service_generation == 0 || evidence.current_service_generation == 0 {
        return Err(SecretUseDenial::InvalidServiceGeneration);
    }
    if evidence.service_generation != evidence.current_service_generation {
        return Err(SecretUseDenial::ServiceGenerationMismatch);
    }
    if evidence.record_version == 0 || evidence.current_record_version == 0 {
        return Err(SecretUseDenial::InvalidRecordVersion);
    }
    if evidence.record_version != evidence.current_record_version {
        return Err(SecretUseDenial::RecordVersionMismatch);
    }
    if evidence.key_epoch == 0 || evidence.current_key_epoch == 0 {
        return Err(SecretUseDenial::InvalidKeyEpoch);
    }
    if evidence.key_epoch != evidence.current_key_epoch {
        return Err(SecretUseDenial::KeyEpochMismatch);
    }
    if !evidence.trust_decision_positive {
        return Err(SecretUseDenial::TrustDecisionMissing);
    }
    if !evidence.committed_store_record_verified {
        return Err(SecretUseDenial::CommittedStoreEvidenceMissing);
    }
    if !evidence.audit_binding_present {
        return Err(SecretUseDenial::AuditBindingMissing);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wifi_target(ssid: &[u8]) -> SecretTargetBinding {
        SecretTargetBinding::for_wifi_wpa2_psk_ccmp(ssid, [0x02, 0x11, 0x22, 0x33, 0x44, 0x55])
            .unwrap()
    }

    fn evidence() -> SecretUseEvidence {
        SecretUseEvidence {
            vault_unlocked: true,
            boot_scope: SecretBootScope::CurrentBoot,
            explicit_recovery_action: false,
            service_generation: 3,
            current_service_generation: 3,
            record_version: 11,
            current_record_version: 11,
            key_epoch: 7,
            current_key_epoch: 7,
            trust_decision_positive: true,
            committed_store_record_verified: true,
            audit_binding_present: true,
        }
    }

    fn wifi_request() -> SecretUseRequest<'static> {
        let target = wifi_target(b"TestNetwork");
        SecretUseRequest {
            kind: SecretKind::WifiPassphrase,
            consumer: SecretConsumer::NativeWifiSupplicant,
            operation: SecretOperation::AssociateBoundBss,
            bound_target: target,
            requested_target: target,
            provider_host: None,
            evidence: evidence(),
        }
    }

    fn provider_request() -> SecretUseRequest<'static> {
        let target = SecretTargetBinding::for_openai_host(OPENAI_API_HOST).unwrap();
        SecretUseRequest {
            kind: SecretKind::ProviderApiKey,
            consumer: SecretConsumer::OpenAiDirect,
            operation: SecretOperation::RequestExactHost,
            bound_target: target,
            requested_target: target,
            provider_host: Some(OPENAI_API_HOST),
            evidence: evidence(),
        }
    }

    #[test]
    fn authorizes_only_exact_wifi_and_openai_uses() {
        let wifi = authorize_secret_use(&wifi_request()).unwrap();
        assert_eq!(wifi.kind(), SecretKind::WifiPassphrase);
        assert_eq!(wifi.consumer(), SecretConsumer::NativeWifiSupplicant);
        assert_eq!(wifi.operation(), SecretOperation::AssociateBoundBss);

        let provider = authorize_secret_use(&provider_request()).unwrap();
        assert_eq!(provider.kind(), SecretKind::ProviderApiKey);
        assert_eq!(provider.consumer(), SecretConsumer::OpenAiDirect);
        assert_eq!(provider.operation(), SecretOperation::RequestExactHost);
    }

    #[test]
    fn wifi_pairwise_shape_denials_are_stable() {
        let cases: &[(fn(&mut SecretUseRequest<'static>), SecretUseDenial)] = &[
            (
                |r| r.kind = SecretKind::ProviderApiKey,
                SecretUseDenial::SecretKindMismatch,
            ),
            (
                |r| r.consumer = SecretConsumer::OpenAiDirect,
                SecretUseDenial::ConsumerMismatch,
            ),
            (
                |r| r.operation = SecretOperation::RequestExactHost,
                SecretUseDenial::OperationMismatch,
            ),
            (
                |r| {
                    r.requested_target =
                        SecretTargetBinding::for_openai_host(OPENAI_API_HOST).unwrap()
                },
                SecretUseDenial::TargetKindMismatch,
            ),
            (
                |r| r.requested_target = wifi_target(b"OtherNetwork"),
                SecretUseDenial::TargetMismatch,
            ),
            (
                |r| r.provider_host = Some(OPENAI_API_HOST),
                SecretUseDenial::UnexpectedProviderHost,
            ),
        ];
        for (mutate, expected) in cases {
            let mut request = wifi_request();
            mutate(&mut request);
            assert_eq!(authorize_secret_use(&request).unwrap_err(), *expected);
        }
    }

    #[test]
    fn provider_pairwise_shape_denials_are_stable() {
        let cases: &[(fn(&mut SecretUseRequest<'static>), SecretUseDenial)] = &[
            (
                |r| r.kind = SecretKind::WifiPassphrase,
                SecretUseDenial::SecretKindMismatch,
            ),
            (
                |r| r.consumer = SecretConsumer::NativeWifiSupplicant,
                SecretUseDenial::ConsumerMismatch,
            ),
            (
                |r| r.operation = SecretOperation::AssociateBoundBss,
                SecretUseDenial::OperationMismatch,
            ),
            (
                |r| r.requested_target = wifi_target(b"TestNetwork"),
                SecretUseDenial::TargetKindMismatch,
            ),
            (
                |r| {
                    r.requested_target = SecretTargetBinding::from_stored(
                        SecretTargetKind::ProviderExactHost,
                        [0x55; 32],
                    )
                },
                SecretUseDenial::TargetMismatch,
            ),
            (
                |r| r.provider_host = None,
                SecretUseDenial::ProviderHostMissing,
            ),
            (
                |r| r.provider_host = Some("example.invalid"),
                SecretUseDenial::ProviderHostMismatch,
            ),
        ];
        for (mutate, expected) in cases {
            let mut request = provider_request();
            mutate(&mut request);
            assert_eq!(authorize_secret_use(&request).unwrap_err(), *expected);
        }
    }

    #[test]
    fn every_evidence_failure_has_a_typed_denial() {
        let cases: &[(fn(&mut SecretUseEvidence), SecretUseDenial)] = &[
            (|e| e.vault_unlocked = false, SecretUseDenial::VaultLocked),
            (
                |e| {
                    e.boot_scope = SecretBootScope::SafeRecovery;
                    e.explicit_recovery_action = false;
                },
                SecretUseDenial::SafeRecoveryActionMissing,
            ),
            (
                |e| e.service_generation = 0,
                SecretUseDenial::InvalidServiceGeneration,
            ),
            (
                |e| e.service_generation += 1,
                SecretUseDenial::ServiceGenerationMismatch,
            ),
            (
                |e| e.record_version = 0,
                SecretUseDenial::InvalidRecordVersion,
            ),
            (
                |e| e.record_version += 1,
                SecretUseDenial::RecordVersionMismatch,
            ),
            (|e| e.key_epoch = 0, SecretUseDenial::InvalidKeyEpoch),
            (|e| e.key_epoch += 1, SecretUseDenial::KeyEpochMismatch),
            (
                |e| e.trust_decision_positive = false,
                SecretUseDenial::TrustDecisionMissing,
            ),
            (
                |e| e.committed_store_record_verified = false,
                SecretUseDenial::CommittedStoreEvidenceMissing,
            ),
            (
                |e| e.audit_binding_present = false,
                SecretUseDenial::AuditBindingMissing,
            ),
        ];

        for (mutate, expected) in cases {
            let mut request = wifi_request();
            mutate(&mut request.evidence);
            assert_eq!(authorize_secret_use(&request).unwrap_err(), *expected);
        }
    }

    #[test]
    fn safe_recovery_requires_and_accepts_one_explicit_action() {
        let mut request = wifi_request();
        request.evidence.boot_scope = SecretBootScope::SafeRecovery;
        assert_eq!(
            authorize_secret_use(&request).unwrap_err(),
            SecretUseDenial::SafeRecoveryActionMissing
        );
        request.evidence.explicit_recovery_action = true;
        assert!(authorize_secret_use(&request).is_ok());
    }
}
