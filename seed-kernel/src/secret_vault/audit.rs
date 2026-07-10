//! Durable pre-use audit for the exact native OpenAI secret consumer.
//!
//! The record contains metadata only. It never receives secret bytes, their
//! length, or a secret-derived hash. A positive receipt exists only after the
//! shared durable-memory append path proves write/readback/reparse/rescan.

use alloc::{format, vec, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

use raios_core::{
    memory_record::{MemoryRecord, MemoryRecordError, MemoryRecordInput, MemorySource},
    record::{Field, Value},
    scoped_secret_use::{SecretBootScope, SecretUseEvidence},
    secret_vault::OPENAI_API_HOST,
    sha256_bytes,
};

use crate::{agent_protocol::durable_store, provider_trust};

const RECORD_ID_PREFIX: &str = "mem.capability_grant.secret_use.openai_direct";
const WIFI_RECORD_ID_PREFIX: &str = "mem.capability_grant.secret_use.native_wifi_supplicant";
const APPEND_AUTHORITY: &str = "scoped_memory_record_append_authorized";
const CONSUMER: &str = "openai_direct";
const OPERATION: &str = "request_exact_host";
const WIFI_CONSUMER: &str = "native_wifi_supplicant";
const WIFI_OPERATION: &str = "associate_bound_bss";
static NEXT_AUDIT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// One explicit, core-owned Genesis Recovery request to reconnect WiFi in
/// SAFE posture. The parent facade mints it only after its physical-action and
/// owner-policy/last-good checks; moving it into the SAFE append consumes it.
/// Deliberately neither `Clone`, `Copy`, nor `Debug`.
pub(super) struct SafeWifiAuditAuthority {
    _facade_only: (),
}

pub(super) const fn new_safe_wifi_audit_authority() -> SafeWifiAuditAuthority {
    SafeWifiAuditAuthority { _facade_only: () }
}

/// Trust already established for this exact provider request. Constructors
/// reject development bypass and ambiguous/partial trust state.
pub(crate) enum ProviderRequestTrust {
    Pinned {
        mode: &'static str,
        evidence_sha256: [u8; 32],
    },
    ContainedQemuValidator {
        evidence_sha256: [u8; 32],
    },
}

impl ProviderRequestTrust {
    /// Consumes the provider module's opaque proof; raw snapshots cannot mint
    /// production audit trust through this API.
    pub(crate) fn from_verified_openai(
        verified: provider_trust::VerifiedOpenAiProviderTrust,
    ) -> Result<Self, ProviderUseAuditDenied> {
        let mode = match verified.audit_source() {
            "pinned_cert_verified" => "pinned_cert_verified",
            "pinned_spki_verified" => "pinned_spki_verified",
            "webpki_verified" => "webpki_verified",
            _ => return Err(ProviderUseAuditDenied::TrustNotPinned),
        };
        let evidence_sha256 = verified.audit_evidence_sha256();
        if evidence_sha256 == [0; 32] {
            return Err(ProviderUseAuditDenied::TrustEvidenceInvalid);
        }
        Ok(Self::Pinned {
            mode,
            evidence_sha256,
        })
    }

    /// Explicitly non-production trust for the contained QEMU validator. The
    /// caller must bind non-zero evidence from that validator run.
    pub(crate) fn contained_qemu_validator(
        evidence_sha256: [u8; 32],
    ) -> Result<Self, ProviderUseAuditDenied> {
        if evidence_sha256 == [0; 32] {
            return Err(ProviderUseAuditDenied::TrustEvidenceInvalid);
        }
        Ok(Self::ContainedQemuValidator { evidence_sha256 })
    }

    const fn mode(&self) -> &'static str {
        match self {
            Self::Pinned { mode, .. } => mode,
            Self::ContainedQemuValidator { .. } => "contained_qemu_validator_not_production_trust",
        }
    }

    const fn evidence_sha256(&self) -> [u8; 32] {
        match self {
            Self::Pinned {
                evidence_sha256, ..
            }
            | Self::ContainedQemuValidator { evidence_sha256 } => *evidence_sha256,
        }
    }

    const fn network_export_authorized(&self) -> bool {
        matches!(self, Self::Pinned { .. })
    }
}

/// Facade-built metadata for the current committed provider slot. Its only
/// constructor is visible to the parent Vault composition boundary.
pub(super) struct ProviderUseAuditBinding {
    store_uuid: [u8; 16],
    record_version: u64,
    key_epoch: u64,
    native_core_generation: u64,
    policy_id_sha256: [u8; 32],
    trust: ProviderRequestTrust,
}

impl ProviderUseAuditBinding {
    pub(super) fn from_current_provider_slot(
        store_uuid: [u8; 16],
        record_version: u64,
        key_epoch: u64,
        native_core_generation: u64,
        policy_id_sha256: [u8; 32],
        trust: ProviderRequestTrust,
    ) -> Result<Self, ProviderUseAuditDenied> {
        if store_uuid == [0; 16] {
            return Err(ProviderUseAuditDenied::StoreIdentityInvalid);
        }
        if record_version == 0 {
            return Err(ProviderUseAuditDenied::RecordVersionInvalid);
        }
        if key_epoch == 0 {
            return Err(ProviderUseAuditDenied::KeyEpochInvalid);
        }
        if native_core_generation == 0 {
            return Err(ProviderUseAuditDenied::CoreGenerationInvalid);
        }
        if policy_id_sha256 == [0; 32] {
            return Err(ProviderUseAuditDenied::CorePolicyInvalid);
        }
        Ok(Self {
            store_uuid,
            record_version,
            key_epoch,
            native_core_generation,
            policy_id_sha256,
            trust,
        })
    }
}

/// Metadata-only binding for one exact native WiFi association. The target is
/// already the typed SSID/BSSID/security-profile hash; raw radio identifiers
/// and secret-derived values never enter this module.
pub(super) struct WifiUseAuditBinding {
    store_uuid: [u8; 16],
    target_binding_sha256: [u8; 32],
    record_version: u64,
    key_epoch: u64,
    native_core_generation: u64,
    policy_id_sha256: [u8; 32],
    test_infrastructure: bool,
}

impl WifiUseAuditBinding {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_current_wifi_slot(
        store_uuid: [u8; 16],
        target_binding_sha256: [u8; 32],
        record_version: u64,
        key_epoch: u64,
        native_core_generation: u64,
        policy_id_sha256: [u8; 32],
        test_infrastructure: bool,
    ) -> Result<Self, WifiUseAuditDenied> {
        if store_uuid == [0; 16] {
            return Err(WifiUseAuditDenied::StoreIdentityInvalid);
        }
        if target_binding_sha256 == [0; 32] {
            return Err(WifiUseAuditDenied::TargetBindingInvalid);
        }
        if record_version == 0 {
            return Err(WifiUseAuditDenied::RecordVersionInvalid);
        }
        if key_epoch == 0 {
            return Err(WifiUseAuditDenied::KeyEpochInvalid);
        }
        if native_core_generation == 0 {
            return Err(WifiUseAuditDenied::CoreGenerationInvalid);
        }
        if policy_id_sha256 == [0; 32] {
            return Err(WifiUseAuditDenied::CorePolicyInvalid);
        }
        Ok(Self {
            store_uuid,
            target_binding_sha256,
            record_version,
            key_epoch,
            native_core_generation,
            policy_id_sha256,
            test_infrastructure,
        })
    }
}

/// One durable, one-use precondition for a single exact-host provider lease.
/// Deliberately neither `Clone` nor `Debug`.
pub(crate) struct DurableProviderUseReceipt {
    evidence: SecretUseEvidence,
}

impl DurableProviderUseReceipt {
    pub(super) fn into_secret_use_evidence(self) -> SecretUseEvidence {
        self.evidence
    }
}

/// One durable precondition for one exact bound-BSS formatter invocation.
/// Its distinct type cannot satisfy the provider Broker entrypoint and it is
/// deliberately neither `Clone` nor `Debug`.
pub(crate) struct DurableWifiUseReceipt {
    evidence: SecretUseEvidence,
    target_binding_sha256: [u8; 32],
}

impl DurableWifiUseReceipt {
    pub(super) fn into_secret_use_evidence(self) -> (SecretUseEvidence, [u8; 32]) {
        (self.evidence, self.target_binding_sha256)
    }

    #[cfg(test)]
    pub(super) fn for_test(evidence: SecretUseEvidence, target_binding_sha256: [u8; 32]) -> Self {
        Self {
            evidence,
            target_binding_sha256,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProviderUseAuditDenied {
    StoreIdentityInvalid,
    RecordVersionInvalid,
    KeyEpochInvalid,
    CoreGenerationInvalid,
    CorePolicyInvalid,
    TrustNotPinned,
    TrustEvidenceInvalid,
    AuditSequenceExhausted,
    Record(MemoryRecordError),
    DurableAppend(&'static str),
    DurableEvidenceMismatch,
}

impl ProviderUseAuditDenied {
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::StoreIdentityInvalid => "provider_audit_store_identity_invalid",
            Self::RecordVersionInvalid => "provider_audit_record_version_invalid",
            Self::KeyEpochInvalid => "provider_audit_key_epoch_invalid",
            Self::CoreGenerationInvalid => "provider_audit_core_generation_invalid",
            Self::CorePolicyInvalid => "provider_audit_core_policy_invalid",
            Self::TrustNotPinned => "provider_audit_trust_not_pinned",
            Self::TrustEvidenceInvalid => "provider_audit_trust_evidence_invalid",
            Self::AuditSequenceExhausted => "provider_audit_sequence_exhausted",
            Self::Record(error) => error.reason(),
            Self::DurableAppend(reason) => reason,
            Self::DurableEvidenceMismatch => "provider_audit_durable_evidence_mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WifiUseAuditDenied {
    StoreIdentityInvalid,
    TargetBindingInvalid,
    RecordVersionInvalid,
    KeyEpochInvalid,
    CoreGenerationInvalid,
    CorePolicyInvalid,
    AuditSequenceExhausted,
    Record(MemoryRecordError),
    DurableAppend(&'static str),
    DurableEvidenceMismatch,
}

impl WifiUseAuditDenied {
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::StoreIdentityInvalid => "wifi_audit_store_identity_invalid",
            Self::TargetBindingInvalid => "wifi_audit_target_binding_invalid",
            Self::RecordVersionInvalid => "wifi_audit_record_version_invalid",
            Self::KeyEpochInvalid => "wifi_audit_key_epoch_invalid",
            Self::CoreGenerationInvalid => "wifi_audit_core_generation_invalid",
            Self::CorePolicyInvalid => "wifi_audit_core_policy_invalid",
            Self::AuditSequenceExhausted => "wifi_audit_sequence_exhausted",
            Self::Record(error) => error.reason(),
            Self::DurableAppend(reason) => reason,
            Self::DurableEvidenceMismatch => "wifi_audit_durable_evidence_mismatch",
        }
    }
}

/// Appends one metadata-only `raios.memory_record.v0` capability grant and
/// returns a receipt only after the exact readback is typed-reparsed and the
/// reclog tail/count advance is observed.
pub(super) fn append_provider_use_audit(
    binding: ProviderUseAuditBinding,
) -> Result<DurableProviderUseReceipt, ProviderUseAuditDenied> {
    let sequence = NEXT_AUDIT_SEQUENCE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| ProviderUseAuditDenied::AuditSequenceExhausted)?
        .checked_add(1)
        .ok_or(ProviderUseAuditDenied::AuditSequenceExhausted)?;
    let record_id = format!("{RECORD_ID_PREFIX}.{sequence}.current_boot.v0");
    let record = provider_use_record(&binding, &record_id, sequence)
        .map_err(ProviderUseAuditDenied::Record)?;
    let expected_payload_sha256 = record.record_sha256();
    let append = durable_store::append_memory_record(&record);
    if !append.performed {
        return Err(ProviderUseAuditDenied::DurableAppend(append.reason));
    }

    let seq = append
        .seq
        .ok_or(ProviderUseAuditDenied::DurableEvidenceMismatch)?;
    let count_advanced = append
        .count_before
        .and_then(|count| count.checked_add(1))
        .zip(append.count_after)
        .is_some_and(|(expected, actual)| expected == actual);
    if append.authority != APPEND_AUTHORITY
        || append.payload_sha256 != Some(expected_payload_sha256)
        || append.frame_sha256.is_none()
        || append.frame_sha256 != append.readback_sha256
        || !append.reparse_valid
        || append.tail_seq_after != Some(seq)
        || !count_advanced
    {
        return Err(ProviderUseAuditDenied::DurableEvidenceMismatch);
    }

    Ok(DurableProviderUseReceipt {
        evidence: SecretUseEvidence {
            vault_unlocked: true,
            boot_scope: SecretBootScope::CurrentBoot,
            explicit_recovery_action: false,
            service_generation: binding.native_core_generation,
            current_service_generation: binding.native_core_generation,
            record_version: binding.record_version,
            current_record_version: binding.record_version,
            key_epoch: binding.key_epoch,
            current_key_epoch: binding.key_epoch,
            trust_decision_positive: true,
            committed_store_record_verified: true,
            audit_binding_present: true,
        },
    })
}

/// Appends one metadata-only WiFi use grant. A receipt exists only after the
/// same durable write/readback/reparse/rescan evidence required by provider
/// use; the record contains no plaintext, length, SSID, BSSID, or secret hash.
pub(super) fn append_wifi_use_audit(
    binding: WifiUseAuditBinding,
) -> Result<DurableWifiUseReceipt, WifiUseAuditDenied> {
    append_wifi_use_audit_inner(binding, WifiAuditScope::CurrentBoot)
}

/// SAFE-only counterpart to `append_wifi_use_audit`. Its move-only authority
/// can mint exactly one WiFi receipt, and the durable layer independently
/// rejects every non-WiFi/non-SAFE record shape before media I/O.
pub(super) fn append_safe_wifi_use_audit(
    binding: WifiUseAuditBinding,
    authority: SafeWifiAuditAuthority,
) -> Result<DurableWifiUseReceipt, WifiUseAuditDenied> {
    append_wifi_use_audit_inner(binding, WifiAuditScope::SafeRecovery(authority))
}

enum WifiAuditScope {
    CurrentBoot,
    SafeRecovery(SafeWifiAuditAuthority),
}

fn append_wifi_use_audit_inner(
    binding: WifiUseAuditBinding,
    scope: WifiAuditScope,
) -> Result<DurableWifiUseReceipt, WifiUseAuditDenied> {
    let sequence = NEXT_AUDIT_SEQUENCE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| WifiUseAuditDenied::AuditSequenceExhausted)?
        .checked_add(1)
        .ok_or(WifiUseAuditDenied::AuditSequenceExhausted)?;
    let safe_recovery = matches!(&scope, WifiAuditScope::SafeRecovery(_));
    let scope_name = if safe_recovery {
        "safe_recovery"
    } else {
        "current_boot"
    };
    let record_id = format!("{WIFI_RECORD_ID_PREFIX}.{sequence}.{scope_name}.v0");
    let record = wifi_use_record(&binding, &record_id, sequence, safe_recovery)
        .map_err(WifiUseAuditDenied::Record)?;
    let expected_payload_sha256 = record.record_sha256();
    let append = match scope {
        WifiAuditScope::CurrentBoot => durable_store::append_memory_record(&record),
        WifiAuditScope::SafeRecovery(_authority) => {
            durable_store::append_safe_wifi_audit_record(&record)
        }
    };
    if !append.performed {
        return Err(WifiUseAuditDenied::DurableAppend(append.reason));
    }

    let seq = append
        .seq
        .ok_or(WifiUseAuditDenied::DurableEvidenceMismatch)?;
    let count_advanced = append
        .count_before
        .and_then(|count| count.checked_add(1))
        .zip(append.count_after)
        .is_some_and(|(expected, actual)| expected == actual);
    if append.authority != APPEND_AUTHORITY
        || append.payload_sha256 != Some(expected_payload_sha256)
        || append.frame_sha256.is_none()
        || append.frame_sha256 != append.readback_sha256
        || !append.reparse_valid
        || append.tail_seq_after != Some(seq)
        || !count_advanced
    {
        return Err(WifiUseAuditDenied::DurableEvidenceMismatch);
    }

    Ok(DurableWifiUseReceipt {
        evidence: SecretUseEvidence {
            vault_unlocked: true,
            boot_scope: if safe_recovery {
                SecretBootScope::SafeRecovery
            } else {
                SecretBootScope::CurrentBoot
            },
            explicit_recovery_action: safe_recovery,
            service_generation: binding.native_core_generation,
            current_service_generation: binding.native_core_generation,
            record_version: binding.record_version,
            current_record_version: binding.record_version,
            key_epoch: binding.key_epoch,
            current_key_epoch: binding.key_epoch,
            trust_decision_positive: true,
            committed_store_record_verified: true,
            audit_binding_present: true,
        },
        target_binding_sha256: binding.target_binding_sha256,
    })
}

fn provider_use_record<'a>(
    binding: &ProviderUseAuditBinding,
    record_id: &'a str,
    sequence: u64,
) -> Result<MemoryRecord<'a>, MemoryRecordError> {
    MemoryRecord::new(MemoryRecordInput {
        id: record_id,
        kind: "capability_grant",
        entity: CONSUMER,
        predicate: OPERATION,
        value: Value::Object(vec![
            Field::new("consumer", Value::Str(CONSUMER)),
            Field::new("operation", Value::Str(OPERATION)),
            Field::new("target", Value::Str(OPENAI_API_HOST)),
            Field::new(
                "store_uuid_sha256",
                Value::Sha256(sha256_bytes(&binding.store_uuid)),
            ),
            Field::new("record_version", Value::U64(binding.record_version)),
            Field::new("key_epoch", Value::U64(binding.key_epoch)),
            Field::new(
                "native_consumer_generation",
                Value::U64(binding.native_core_generation),
            ),
            Field::new(
                "core_generation",
                Value::U64(binding.native_core_generation),
            ),
            Field::new(
                "core_policy_sha256",
                Value::Sha256(binding.policy_id_sha256),
            ),
            Field::new("trust_mode", Value::Str(binding.trust.mode())),
            Field::new(
                "trust_evidence_sha256",
                Value::Sha256(binding.trust.evidence_sha256()),
            ),
            Field::new(
                "network_export_authorized",
                Value::Bool(binding.trust.network_export_authorized()),
            ),
            Field::new(
                "test_infrastructure",
                Value::Bool(!binding.trust.network_export_authorized()),
            ),
        ]),
        classification: "local_only",
        authority: "owner_verified_core_policy",
        boot_id: "current_boot",
        sequence,
        source: MemorySource::new("secret_vault.provider_use_audit", "core_policy.current"),
        evidence: Vec::new(),
        tags: vec!["secret_use", "provider", "audit"],
        supersedes: Vec::new(),
        created_at_ticks: 0,
    })
}

fn wifi_use_record<'a>(
    binding: &WifiUseAuditBinding,
    record_id: &'a str,
    sequence: u64,
    safe_recovery: bool,
) -> Result<MemoryRecord<'a>, MemoryRecordError> {
    let value = vec![
        Field::new("consumer", Value::Str(WIFI_CONSUMER)),
        Field::new("operation", Value::Str(WIFI_OPERATION)),
        Field::new(
            "target_binding_sha256",
            Value::Sha256(binding.target_binding_sha256),
        ),
        Field::new(
            "store_uuid_sha256",
            Value::Sha256(sha256_bytes(&binding.store_uuid)),
        ),
        Field::new("record_version", Value::U64(binding.record_version)),
        Field::new("key_epoch", Value::U64(binding.key_epoch)),
        Field::new(
            "native_consumer_generation",
            Value::U64(binding.native_core_generation),
        ),
        Field::new(
            "core_generation",
            Value::U64(binding.native_core_generation),
        ),
        Field::new(
            "core_policy_sha256",
            Value::Sha256(binding.policy_id_sha256),
        ),
        Field::new("network_export_authorized", Value::Bool(false)),
        Field::new(
            "test_infrastructure",
            Value::Bool(binding.test_infrastructure),
        ),
        Field::new(
            "boot_scope",
            Value::Str(if safe_recovery {
                "safe_recovery"
            } else {
                "current_boot"
            }),
        ),
        Field::new("explicit_recovery_action", Value::Bool(safe_recovery)),
    ];
    MemoryRecord::new(MemoryRecordInput {
        id: record_id,
        kind: "capability_grant",
        entity: WIFI_CONSUMER,
        predicate: WIFI_OPERATION,
        value: Value::Object(value),
        classification: "local_only",
        authority: "owner_verified_core_policy",
        boot_id: "current_boot",
        sequence,
        source: MemorySource::new("secret_vault.wifi_use_audit", "core_policy.current"),
        evidence: Vec::new(),
        tags: vec!["secret_use", "wifi", "audit"],
        supersedes: Vec::new(),
        created_at_ticks: 0,
    })
}
