use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::module_grant::{
    compute_capability_grant, ComputeCapabilityGrantRequest, ComputedCapabilityGrantDiagnostic,
};

pub const MODULE_AUDIT_ROLLBACK_DIAGNOSTIC_SCHEMA: &str =
    "raios.module_audit_rollback_diagnostic.v0";
pub const AUDIT_RECORD_SCHEMA: &str = "raios.audit_record.v0";
pub const AUDIT_RECORD_CANONICALIZATION: &str = "raios.audit_record.canonical.v0";
pub const ROLLBACK_PLAN_SCHEMA: &str = "raios.rollback_plan.v0";
pub const ROLLBACK_PLAN_CANONICALIZATION: &str = "raios.rollback_plan.canonical.v0";

#[derive(Clone, Debug)]
pub struct ModuleAuditRollbackDiagnosticRequest {
    pub grant_request: ComputeCapabilityGrantRequest,
    pub computed_capability_grant_hash: String,
    pub denial_event_id: String,
    pub retained_reference_event_id: String,
    pub ram_only_service_slot_id: String,
    pub rollback_service_slot_id: Option<String>,
    pub pre_load_service_inventory_hash: String,
    pub cleanup_actions_hash: String,
    pub expected_rollback_plan_hash: Option<String>,
    pub expected_audit_record_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleAuditRollbackDiagnostic {
    pub schema: String,
    pub status: String,
    pub valid_evidence: bool,
    pub computed_capability_grant: ComputedCapabilityGrantDiagnostic,
    pub rollback_plan: RollbackPlanCandidate,
    pub audit_record: AuditRecordCandidate,
    pub policy_result: AuditRollbackPolicyResult,
    pub denial_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollbackPlanCandidate {
    pub schema: String,
    pub canonicalization: String,
    pub status: String,
    pub rollback_plan_hash: String,
    pub load_mode: String,
    pub scope: String,
    pub artifact_sha256: String,
    pub pre_load_service_inventory_sha256: String,
    pub ram_only_service_slot_id: String,
    pub cleanup_actions_sha256: String,
    pub rollback_actions: Vec<String>,
    pub service_inventory_change: String,
    pub load_attempted: bool,
    pub installed_in_guest: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditRecordCandidate {
    pub schema: String,
    pub canonicalization: String,
    pub status: String,
    pub audit_record_hash: String,
    pub classification: String,
    pub durability: String,
    pub denial_event_id: String,
    pub retained_reference_event_id: String,
    pub requested_capability: String,
    pub load_mode: String,
    pub subject: String,
    pub resource: String,
    pub scope: String,
    pub computed_capability_grant_hash: String,
    pub manifest_sha256: String,
    pub candidate_artifact_sha256: String,
    pub vm_test_report_sha256: String,
    pub local_attestation_sha256: String,
    pub local_approval_sha256: String,
    pub rollback_plan_hash: String,
    pub ram_only_service_slot_id: String,
    pub writes_enabled: bool,
    pub grants_capability: bool,
    pub grants_load_now: bool,
    pub authorizes_guest_load: bool,
    pub can_load_now: bool,
    pub service_inventory_change: String,
    pub load_attempted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditRollbackPolicyResult {
    pub audit_record_present: bool,
    pub rollback_plan_present: bool,
    pub host_diagnostic_only: bool,
    pub durable_audit_written: bool,
    pub rollback_plan_installed: bool,
    pub grants_capability: bool,
    pub grants_load_now: bool,
    pub authorizes_guest_load: bool,
    pub can_load_now: bool,
    pub service_inventory_change: String,
    pub load_attempted: bool,
    pub loader: String,
    pub service_slot: String,
}

pub fn compute_audit_rollback_diagnostic(
    request: &ModuleAuditRollbackDiagnosticRequest,
) -> Result<ModuleAuditRollbackDiagnostic> {
    let computed_grant = compute_capability_grant(&request.grant_request)?;
    let mut denial_reasons = computed_grant.denial_reasons.clone();
    let warnings = computed_grant.warnings.clone();

    let expected_grant_hash = normalize_hash_ref(&request.computed_capability_grant_hash)?;
    let actual_grant_hash = normalize_hash_ref(&computed_grant.computed_capability_grant_hash)?;
    require_equal(
        &mut denial_reasons,
        "computed_grant_hash_mismatch",
        Some(actual_grant_hash.as_str()),
        Some(expected_grant_hash.as_str()),
    );

    if !current_boot_event_id(&request.denial_event_id) {
        denial_reasons.push("denial_event_id_not_current_boot".to_string());
    }
    if !current_boot_event_id(&request.retained_reference_event_id) {
        denial_reasons.push("retained_reference_event_id_not_current_boot".to_string());
    }
    if !request.ram_only_service_slot_id.starts_with("ram_only:") {
        denial_reasons.push("ram_only_service_slot_id_invalid".to_string());
    }
    if let Some(rollback_service_slot_id) = request.rollback_service_slot_id.as_deref() {
        require_equal(
            &mut denial_reasons,
            "rollback_service_slot_mismatch",
            Some(rollback_service_slot_id),
            Some(request.ram_only_service_slot_id.as_str()),
        );
    }

    let pre_load_service_inventory_hash =
        normalize_hash_ref(&request.pre_load_service_inventory_hash)?;
    let cleanup_actions_hash = normalize_hash_ref(&request.cleanup_actions_hash)?;
    let manifest_hash = normalize_hash_ref(&computed_grant.evidence.manifest_sha256)?;
    let artifact_hash = normalize_hash_ref(&computed_grant.evidence.candidate_artifact_sha256)?;
    let vm_report_hash = normalize_hash_ref(&computed_grant.evidence.vm_test_report_sha256)?;
    let attestation_hash = normalize_hash_ref(&computed_grant.evidence.local_attestation_sha256)?;
    let local_approval_hash = sha256_text(&request.grant_request.local_approval);

    let rollback_hash = canonical_rollback_plan_hash(
        &request.grant_request.load_mode,
        &request.grant_request.scope,
        &artifact_hash,
        &pre_load_service_inventory_hash,
        &request.ram_only_service_slot_id,
        &cleanup_actions_hash,
    );
    if let Some(expected) = request.expected_rollback_plan_hash.as_deref() {
        let expected = normalize_hash_ref(expected)?;
        require_equal(
            &mut denial_reasons,
            "rollback_plan_hash_mismatch",
            Some(rollback_hash.as_str()),
            Some(expected.as_str()),
        );
    }

    let audit_hash = canonical_audit_record_hash(CanonicalAuditRecordInput {
        request: &request.grant_request,
        denial_event_id: &request.denial_event_id,
        retained_reference_event_id: &request.retained_reference_event_id,
        computed_grant_hash: &actual_grant_hash,
        manifest_hash: &manifest_hash,
        artifact_hash: &artifact_hash,
        vm_report_hash: &vm_report_hash,
        attestation_hash: &attestation_hash,
        local_approval_hash: &local_approval_hash,
        rollback_hash: &rollback_hash,
        ram_only_service_slot_id: &request.ram_only_service_slot_id,
    });
    if let Some(expected) = request.expected_audit_record_hash.as_deref() {
        let expected = normalize_hash_ref(expected)?;
        require_equal(
            &mut denial_reasons,
            "audit_record_hash_mismatch",
            Some(audit_hash.as_str()),
            Some(expected.as_str()),
        );
    }

    let valid_evidence = denial_reasons.is_empty();
    let status = if valid_evidence {
        "candidate_evidence_valid_load_still_denied"
    } else {
        "candidate_evidence_rejected"
    };

    let rollback_plan = RollbackPlanCandidate {
        schema: ROLLBACK_PLAN_SCHEMA.to_string(),
        canonicalization: ROLLBACK_PLAN_CANONICALIZATION.to_string(),
        status: status.to_string(),
        rollback_plan_hash: format!("sha256:{rollback_hash}"),
        load_mode: request.grant_request.load_mode.clone(),
        scope: request.grant_request.scope.clone(),
        artifact_sha256: format!("sha256:{artifact_hash}"),
        pre_load_service_inventory_sha256: format!("sha256:{pre_load_service_inventory_hash}"),
        ram_only_service_slot_id: request.ram_only_service_slot_id.clone(),
        cleanup_actions_sha256: format!("sha256:{cleanup_actions_hash}"),
        rollback_actions: vec![
            "stop_ram_only_service_slot".to_string(),
            "drop_ram_only_artifact".to_string(),
            "restore_pre_load_service_inventory".to_string(),
        ],
        service_inventory_change: "none".to_string(),
        load_attempted: false,
        installed_in_guest: false,
    };

    let audit_record = AuditRecordCandidate {
        schema: AUDIT_RECORD_SCHEMA.to_string(),
        canonicalization: AUDIT_RECORD_CANONICALIZATION.to_string(),
        status: status.to_string(),
        audit_record_hash: format!("sha256:{audit_hash}"),
        classification: "local_only".to_string(),
        durability: "host_diagnostic_not_durable".to_string(),
        denial_event_id: request.denial_event_id.clone(),
        retained_reference_event_id: request.retained_reference_event_id.clone(),
        requested_capability: request.grant_request.requested_capability.clone(),
        load_mode: request.grant_request.load_mode.clone(),
        subject: request.grant_request.subject.clone(),
        resource: request.grant_request.resource.clone(),
        scope: request.grant_request.scope.clone(),
        computed_capability_grant_hash: format!("sha256:{actual_grant_hash}"),
        manifest_sha256: format!("sha256:{manifest_hash}"),
        candidate_artifact_sha256: format!("sha256:{artifact_hash}"),
        vm_test_report_sha256: format!("sha256:{vm_report_hash}"),
        local_attestation_sha256: format!("sha256:{attestation_hash}"),
        local_approval_sha256: format!("sha256:{local_approval_hash}"),
        rollback_plan_hash: rollback_plan.rollback_plan_hash.clone(),
        ram_only_service_slot_id: request.ram_only_service_slot_id.clone(),
        writes_enabled: false,
        grants_capability: false,
        grants_load_now: false,
        authorizes_guest_load: false,
        can_load_now: false,
        service_inventory_change: "none".to_string(),
        load_attempted: false,
    };

    Ok(ModuleAuditRollbackDiagnostic {
        schema: MODULE_AUDIT_ROLLBACK_DIAGNOSTIC_SCHEMA.to_string(),
        status: status.to_string(),
        valid_evidence,
        computed_capability_grant: computed_grant,
        rollback_plan,
        audit_record,
        policy_result: AuditRollbackPolicyResult {
            audit_record_present: valid_evidence,
            rollback_plan_present: valid_evidence,
            host_diagnostic_only: true,
            durable_audit_written: false,
            rollback_plan_installed: false,
            grants_capability: false,
            grants_load_now: false,
            authorizes_guest_load: false,
            can_load_now: false,
            service_inventory_change: "none".to_string(),
            load_attempted: false,
            loader: "unavailable".to_string(),
            service_slot: "unallocated".to_string(),
        },
        denial_reasons,
        warnings,
    })
}

fn canonical_rollback_plan_hash(
    load_mode: &str,
    scope: &str,
    artifact_hash: &str,
    pre_load_service_inventory_hash: &str,
    ram_only_service_slot_id: &str,
    cleanup_actions_hash: &str,
) -> String {
    let canonical = [
        format!("canonicalization={ROLLBACK_PLAN_CANONICALIZATION}"),
        format!("schema={ROLLBACK_PLAN_SCHEMA}"),
        format!("load_mode={load_mode}"),
        format!("scope={scope}"),
        format!("artifact_sha256={artifact_hash}"),
        format!("pre_load_service_inventory_sha256={pre_load_service_inventory_hash}"),
        format!("ram_only_service_slot_id={ram_only_service_slot_id}"),
        format!("cleanup_actions_sha256={cleanup_actions_hash}"),
        "service_inventory_change=none".to_string(),
        "load_attempted=false".to_string(),
    ]
    .join("\n");
    sha256_text(&canonical)
}

struct CanonicalAuditRecordInput<'a> {
    request: &'a ComputeCapabilityGrantRequest,
    denial_event_id: &'a str,
    retained_reference_event_id: &'a str,
    computed_grant_hash: &'a str,
    manifest_hash: &'a str,
    artifact_hash: &'a str,
    vm_report_hash: &'a str,
    attestation_hash: &'a str,
    local_approval_hash: &'a str,
    rollback_hash: &'a str,
    ram_only_service_slot_id: &'a str,
}

fn canonical_audit_record_hash(input: CanonicalAuditRecordInput<'_>) -> String {
    let canonical = [
        format!("canonicalization={AUDIT_RECORD_CANONICALIZATION}"),
        format!("schema={AUDIT_RECORD_SCHEMA}"),
        format!(
            "requested_capability={}",
            input.request.requested_capability
        ),
        format!("load_mode={}", input.request.load_mode),
        format!("subject={}", input.request.subject),
        format!("resource={}", input.request.resource),
        format!("scope={}", input.request.scope),
        format!("denial_event_id={}", input.denial_event_id),
        format!(
            "retained_reference_event_id={}",
            input.retained_reference_event_id
        ),
        format!(
            "computed_capability_grant_sha256={}",
            input.computed_grant_hash
        ),
        format!("manifest_sha256={}", input.manifest_hash),
        format!("candidate_artifact_sha256={}", input.artifact_hash),
        format!("vm_test_report_sha256={}", input.vm_report_hash),
        format!("local_attestation_sha256={}", input.attestation_hash),
        format!("local_approval_sha256={}", input.local_approval_hash),
        format!("rollback_plan_sha256={}", input.rollback_hash),
        format!(
            "ram_only_service_slot_id={}",
            input.ram_only_service_slot_id
        ),
        "grants_load_now=false".to_string(),
        "authorizes_guest_load=false".to_string(),
        "service_inventory_change=none".to_string(),
        "load_attempted=false".to_string(),
    ]
    .join("\n");
    sha256_text(&canonical)
}

fn current_boot_event_id(value: &str) -> bool {
    let Some(sequence) = value.strip_prefix("event.current_boot.") else {
        return false;
    };
    sequence.len() == 8 && sequence.chars().all(|c| c.is_ascii_digit())
}

fn require_equal(
    denial_reasons: &mut Vec<String>,
    reason: &str,
    actual: Option<&str>,
    expected: Option<&str>,
) {
    if actual != expected {
        denial_reasons.push(reason.to_string());
    }
}

fn normalize_hash_ref(value: &str) -> Result<String> {
    let trimmed = value.trim().to_ascii_lowercase();
    let hash = trimmed.strip_prefix("sha256:").unwrap_or(&trimmed);
    if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(hash.to_string())
    } else {
        Err(anyhow!("invalid sha256 hash reference: {value}"))
    }
}

fn sha256_text(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    let mut out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::{tempdir, TempDir};

    use crate::module_grant::{
        ComputeCapabilityGrantRequest, COMPUTED_CAPABILITY_GRANT_CANONICALIZATION,
        COMPUTED_CAPABILITY_GRANT_SCHEMA, MODULE_MANIFEST_SCHEMA,
    };
    use crate::{LOCAL_ATTESTATION_SCHEMA, VM_TEST_REPORT_SCHEMA};

    const DEFAULT_REQUESTED_CAPABILITY: &str = "cap.module.load_ephemeral";
    const DEFAULT_LOAD_MODE: &str = "ram_only";
    const DEFAULT_SUBJECT: &str = "agent.session.serial";
    const DEFAULT_RESOURCE: &str = "live_service_graph";
    const DEFAULT_SCOPE: &str = "current_boot";

    struct Fixture {
        _temp: TempDir,
        manifest_path: PathBuf,
        artifact_path: PathBuf,
        report_path: PathBuf,
        attestation_path: PathBuf,
        approval: String,
        attestation_hash: String,
        computed_grant_hash: String,
    }

    #[test]
    fn audit_rollback_diagnostic_binds_tuple_but_does_not_authorize_load() -> Result<()> {
        let fixture = valid_fixture()?;
        let diagnostic = compute_audit_rollback_diagnostic(&request_from(&fixture))?;

        assert!(diagnostic.valid_evidence);
        assert_eq!(
            diagnostic.status,
            "candidate_evidence_valid_load_still_denied"
        );
        assert_eq!(diagnostic.rollback_plan.schema, ROLLBACK_PLAN_SCHEMA);
        assert_eq!(diagnostic.audit_record.schema, AUDIT_RECORD_SCHEMA);
        assert!(diagnostic
            .rollback_plan
            .rollback_plan_hash
            .starts_with("sha256:"));
        assert!(diagnostic
            .audit_record
            .audit_record_hash
            .starts_with("sha256:"));
        assert!(diagnostic.policy_result.audit_record_present);
        assert!(diagnostic.policy_result.rollback_plan_present);
        assert!(diagnostic.policy_result.host_diagnostic_only);
        assert!(!diagnostic.policy_result.durable_audit_written);
        assert!(!diagnostic.policy_result.rollback_plan_installed);
        assert!(!diagnostic.policy_result.grants_capability);
        assert!(!diagnostic.policy_result.grants_load_now);
        assert!(!diagnostic.policy_result.can_load_now);
        assert!(!diagnostic.policy_result.load_attempted);
        assert_eq!(diagnostic.policy_result.service_inventory_change, "none");
        assert!(!diagnostic.audit_record.writes_enabled);
        assert!(!diagnostic.audit_record.authorizes_guest_load);
        assert!(!diagnostic.rollback_plan.installed_in_guest);
        Ok(())
    }

    #[test]
    fn audit_rollback_rejects_retained_grant_hash_mismatch() -> Result<()> {
        let fixture = valid_fixture()?;
        let mut request = request_from(&fixture);
        request.computed_capability_grant_hash = wrong_hash();

        assert_rejects(&request, "computed_grant_hash_mismatch")
    }

    #[test]
    fn audit_rollback_rejects_manifest_hash_mismatch() -> Result<()> {
        let fixture = valid_fixture()?;
        mutate_json(&fixture.report_path, |json| {
            json["candidate_manifest"]["sha256"] = json!(wrong_hash());
            json["evidence_binding"]["candidate_manifest_sha256"] = json!(wrong_hash());
        })?;

        assert_rejects(&request_from(&fixture), "vm_report_manifest_hash_mismatch")
    }

    #[test]
    fn audit_rollback_rejects_artifact_hash_mismatch() -> Result<()> {
        let fixture = valid_fixture()?;
        mutate_json(&fixture.report_path, |json| {
            json["candidate_artifact"]["sha256"] = json!(wrong_hash());
            json["evidence_binding"]["candidate_artifact_sha256"] = json!(wrong_hash());
        })?;

        assert_rejects(&request_from(&fixture), "vm_report_artifact_hash_mismatch")
    }

    #[test]
    fn audit_rollback_rejects_report_hash_mismatch() -> Result<()> {
        let fixture = valid_fixture()?;
        mutate_json(&fixture.attestation_path, |json| {
            json["vm_report"]["sha256"] = json!(wrong_hash());
            json["evidence_binding"]["vm_report_sha256"] = json!(wrong_hash());
        })?;

        assert_rejects(
            &request_from(&fixture),
            "local_attestation_report_hash_mismatch",
        )
    }

    #[test]
    fn audit_rollback_rejects_attestation_hash_mismatch() -> Result<()> {
        let fixture = valid_fixture()?;
        let mut request = request_from(&fixture);
        request.grant_request.expected_local_attestation_sha256 = Some(wrong_hash());

        assert_rejects(&request, "local_attestation_hash_mismatch")
    }

    #[test]
    fn audit_rollback_rejects_wrong_approval_phrase() -> Result<()> {
        let fixture = valid_fixture()?;
        let mut request = request_from(&fixture);
        request.grant_request.local_approval = "APPROVE RAM_ONLY wrong".to_string();

        assert_rejects(&request, "approval_phrase_mismatch")
    }

    #[test]
    fn audit_rollback_rejects_rollback_plan_hash_mismatch() -> Result<()> {
        let fixture = valid_fixture()?;
        let mut request = request_from(&fixture);
        request.expected_rollback_plan_hash = Some(wrong_hash());

        assert_rejects(&request, "rollback_plan_hash_mismatch")
    }

    #[test]
    fn audit_rollback_rejects_service_slot_mismatch() -> Result<()> {
        let fixture = valid_fixture()?;
        let mut request = request_from(&fixture);
        request.rollback_service_slot_id = Some("ram_only:other-slot".to_string());

        assert_rejects(&request, "rollback_service_slot_mismatch")
    }

    fn valid_fixture() -> Result<Fixture> {
        let temp = tempdir()?;
        let artifact_path = temp.path().join("candidate.bin");
        fs::write(&artifact_path, b"candidate module bytes")?;
        let artifact_hash = read_file_sha256(&artifact_path)?;
        let base_image_hash = repeat_hash('b');
        let qemu_args_hash = repeat_hash('c');

        let manifest_path = temp.path().join("candidate.manifest.json");
        write_json(
            &manifest_path,
            &json!({
                "schema": MODULE_MANIFEST_SCHEMA,
                "name": "hello-diagnostic",
                "version": "0.1.0",
                "kind": "guest_diagnostic",
                "target": "raios-stage0",
                "abi": "none",
                "built_by": "registry-core-test",
                "provides": ["diagnostic.hello"],
                "requested_caps": [DEFAULT_REQUESTED_CAPABILITY],
                "granted_caps": [],
                "risk": "modify_ram",
                "load_mode": DEFAULT_LOAD_MODE,
                "artifact_hash": format!("sha256:{artifact_hash}"),
                "base_image_hash": format!("sha256:{base_image_hash}"),
                "manifest_hash": null,
                "test_report_hash": null,
                "tests": ["shadow_vm_protocol_smoke"],
                "rollback_id": null
            }),
        )?;
        let manifest_hash = read_file_sha256(&manifest_path)?;

        let report_path = temp.path().join("shadow-report.json");
        write_json(
            &report_path,
            &json!({
                "schema": VM_TEST_REPORT_SCHEMA,
                "result": "passed",
                "run_id": "shadow-test",
                "base_image": {
                    "path": "release/raios-stage0-shadow.img",
                    "sha256": format!("sha256:{base_image_hash}"),
                    "temporary": true
                },
                "candidate_artifact": {
                    "path": artifact_path,
                    "sha256": format!("sha256:{artifact_hash}")
                },
                "candidate_manifest": {
                    "path": manifest_path,
                    "sha256": format!("sha256:{manifest_hash}"),
                    "validation": null
                },
                "qemu": {
                    "args_sha256": format!("sha256:{qemu_args_hash}")
                },
                "evidence_binding": {
                    "base_image_sha256": format!("sha256:{base_image_hash}"),
                    "candidate_artifact_sha256": format!("sha256:{artifact_hash}"),
                    "candidate_manifest_sha256": format!("sha256:{manifest_hash}"),
                    "qemu_args_sha256": format!("sha256:{qemu_args_hash}"),
                    "result": "passed"
                }
            }),
        )?;
        let report_hash = read_file_sha256(&report_path)?;

        let approval_tuple = format!(
            "manifest={manifest_hash};artifact={artifact_hash};report={report_hash};base={base_image_hash};mode={DEFAULT_LOAD_MODE}"
        );
        let approval_tuple_hash = sha256_text(&approval_tuple);
        let approval = format!(
            "APPROVE RAM_ONLY {}",
            approval_tuple_hash
                .get(..16)
                .ok_or_else(|| anyhow!("approval hash too short"))?
        );
        let approval_hash = sha256_text(&approval);

        let attestation_path = temp.path().join("local-attestation.json");
        write_json(
            &attestation_path,
            &json!({
                "schema": LOCAL_ATTESTATION_SCHEMA,
                "attestation_id": "attest-test",
                "result": "evidence_recorded_load_still_denied_in_stage0",
                "load_mode": DEFAULT_LOAD_MODE,
                "manifest": {
                    "path": manifest_path,
                    "sha256": format!("sha256:{manifest_hash}"),
                    "name": "hello-diagnostic",
                    "version": "0.1.0",
                    "kind": "guest_diagnostic",
                    "risk": "modify_ram",
                    "requested_caps": [DEFAULT_REQUESTED_CAPABILITY]
                },
                "artifact": {
                    "path": artifact_path,
                    "sha256": format!("sha256:{artifact_hash}")
                },
                "vm_report": {
                    "path": report_path,
                    "sha256": format!("sha256:{report_hash}"),
                    "run_id": "shadow-test",
                    "base_image_sha256": format!("sha256:{base_image_hash}"),
                    "qemu_args_sha256": format!("sha256:{qemu_args_hash}"),
                    "result": "passed"
                },
                "approval": {
                    "source": "local_user_cli",
                    "phrase_sha256": approval_hash,
                    "expected_phrase": approval,
                    "tuple_sha256": approval_tuple_hash,
                    "tuple_fields": [
                        "manifest_sha256",
                        "artifact_sha256",
                        "vm_report_sha256",
                        "base_image_sha256",
                        "load_mode"
                    ]
                },
                "evidence_binding": {
                    "manifest_sha256": format!("sha256:{manifest_hash}"),
                    "artifact_sha256": format!("sha256:{artifact_hash}"),
                    "vm_report_sha256": format!("sha256:{report_hash}"),
                    "base_image_sha256": format!("sha256:{base_image_hash}"),
                    "qemu_args_sha256": format!("sha256:{qemu_args_hash}")
                },
                "limits": {
                    "grants_load_now": false,
                    "requires_guest_loader": true,
                    "requires_kernel_policy_check": true,
                    "rollback_plan": "drop_on_reboot_or_kill_service"
                }
            }),
        )?;
        let attestation_hash = read_file_sha256(&attestation_path)?;
        let computed_grant_hash = canonical_grant_hash(
            &manifest_hash,
            &artifact_hash,
            &report_hash,
            &attestation_hash,
        );

        Ok(Fixture {
            _temp: temp,
            manifest_path,
            artifact_path,
            report_path,
            attestation_path,
            approval,
            attestation_hash,
            computed_grant_hash,
        })
    }

    fn request_from(fixture: &Fixture) -> ModuleAuditRollbackDiagnosticRequest {
        let mut grant_request = ComputeCapabilityGrantRequest::new(
            fixture.manifest_path.clone(),
            fixture.artifact_path.clone(),
            fixture.report_path.clone(),
            fixture.attestation_path.clone(),
            fixture.approval.clone(),
        );
        grant_request.expected_local_attestation_sha256 = Some(fixture.attestation_hash.clone());
        ModuleAuditRollbackDiagnosticRequest {
            grant_request,
            computed_capability_grant_hash: fixture.computed_grant_hash.clone(),
            denial_event_id: "event.current_boot.00000031".to_string(),
            retained_reference_event_id: "event.current_boot.00000027".to_string(),
            ram_only_service_slot_id: "ram_only:svc.test.0001".to_string(),
            rollback_service_slot_id: None,
            pre_load_service_inventory_hash: repeat_hash('d'),
            cleanup_actions_hash: repeat_hash('e'),
            expected_rollback_plan_hash: None,
            expected_audit_record_hash: None,
        }
    }

    fn canonical_grant_hash(
        manifest_hash: &str,
        artifact_hash: &str,
        report_hash: &str,
        attestation_hash: &str,
    ) -> String {
        let canonical = [
            format!("canonicalization={COMPUTED_CAPABILITY_GRANT_CANONICALIZATION}"),
            format!("schema={COMPUTED_CAPABILITY_GRANT_SCHEMA}"),
            format!("requested_capability={DEFAULT_REQUESTED_CAPABILITY}"),
            format!("load_mode={DEFAULT_LOAD_MODE}"),
            format!("subject={DEFAULT_SUBJECT}"),
            format!("resource={DEFAULT_RESOURCE}"),
            format!("scope={DEFAULT_SCOPE}"),
            format!("manifest_sha256={manifest_hash}"),
            format!("candidate_artifact_sha256={artifact_hash}"),
            format!("vm_test_report_sha256={report_hash}"),
            format!("local_attestation_sha256={attestation_hash}"),
            "grants_load_now=false".to_string(),
            "authorizes_guest_load=false".to_string(),
            "service_inventory_change=none".to_string(),
            "load_attempted=false".to_string(),
        ]
        .join("\n");
        sha256_text(&canonical)
    }

    fn assert_rejects(request: &ModuleAuditRollbackDiagnosticRequest, reason: &str) -> Result<()> {
        let diagnostic = compute_audit_rollback_diagnostic(request)?;
        assert!(!diagnostic.valid_evidence);
        assert!(
            diagnostic.denial_reasons.contains(&reason.to_string()),
            "expected reason {reason}, got {:?}",
            diagnostic.denial_reasons
        );
        assert!(!diagnostic.policy_result.grants_load_now);
        assert!(!diagnostic.policy_result.can_load_now);
        assert!(!diagnostic.policy_result.load_attempted);
        Ok(())
    }

    fn mutate_json(path: &Path, mutate: impl FnOnce(&mut Value)) -> Result<()> {
        let text = fs::read_to_string(path)?;
        let mut json: Value = serde_json::from_str(&text)?;
        mutate(&mut json);
        write_json(path, &json)
    }

    fn write_json(path: &Path, value: &Value) -> Result<()> {
        fs::write(path, serde_json::to_string_pretty(value)?)?;
        Ok(())
    }

    fn read_file_sha256(path: &Path) -> Result<String> {
        let bytes = fs::read(path)?;
        let digest = Sha256::digest(bytes);
        let mut out = String::with_capacity(64);
        for byte in digest {
            use std::fmt::Write;
            let _ = write!(&mut out, "{byte:02x}");
        }
        Ok(out)
    }

    fn repeat_hash(ch: char) -> String {
        ch.to_string().repeat(64)
    }

    fn wrong_hash() -> String {
        repeat_hash('9')
    }
}
