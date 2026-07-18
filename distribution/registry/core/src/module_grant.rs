use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{LOCAL_ATTESTATION_SCHEMA, VM_TEST_REPORT_SCHEMA};

pub const MODULE_MANIFEST_SCHEMA: &str = "raios.module_manifest.v0";
pub const COMPUTED_CAPABILITY_GRANT_SCHEMA: &str = "raios.computed_capability_grant.v0";
pub const COMPUTED_CAPABILITY_GRANT_CANONICALIZATION: &str =
    "raios.computed_capability_grant.canonical.v0";

const DEFAULT_REQUESTED_CAPABILITY: &str = "cap.module.load_ephemeral";
const DEFAULT_LOAD_MODE: &str = "ram_only";
const DEFAULT_SUBJECT: &str = "agent.session.serial";
const DEFAULT_RESOURCE: &str = "live_service_graph";
const DEFAULT_SCOPE: &str = "current_boot";

#[derive(Clone, Debug)]
pub struct ComputeCapabilityGrantRequest {
    pub manifest_path: PathBuf,
    pub artifact_path: PathBuf,
    pub vm_report_path: PathBuf,
    pub local_attestation_path: PathBuf,
    pub local_approval: String,
    pub requested_capability: String,
    pub load_mode: String,
    pub subject: String,
    pub resource: String,
    pub scope: String,
    pub expected_local_attestation_sha256: Option<String>,
}

impl ComputeCapabilityGrantRequest {
    pub fn new(
        manifest_path: PathBuf,
        artifact_path: PathBuf,
        vm_report_path: PathBuf,
        local_attestation_path: PathBuf,
        local_approval: String,
    ) -> Self {
        Self {
            manifest_path,
            artifact_path,
            vm_report_path,
            local_attestation_path,
            local_approval,
            requested_capability: DEFAULT_REQUESTED_CAPABILITY.to_string(),
            load_mode: DEFAULT_LOAD_MODE.to_string(),
            subject: DEFAULT_SUBJECT.to_string(),
            resource: DEFAULT_RESOURCE.to_string(),
            scope: DEFAULT_SCOPE.to_string(),
            expected_local_attestation_sha256: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputedCapabilityGrantDiagnostic {
    pub schema: String,
    pub canonicalization: String,
    pub status: String,
    pub valid_evidence: bool,
    pub computed_capability_grant_hash: String,
    pub request: GrantRequestRecord,
    pub evidence: GrantEvidenceRecord,
    pub manifest: ManifestSummary,
    pub bindings: GrantBindingState,
    pub policy_result: GrantPolicyResult,
    pub missing_guest_gates: Vec<String>,
    pub denial_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantRequestRecord {
    pub requested_capability: String,
    pub load_mode: String,
    pub subject: String,
    pub resource: String,
    pub scope: String,
    pub risk: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantEvidenceRecord {
    pub manifest_sha256: String,
    pub candidate_artifact_sha256: String,
    pub vm_test_report_sha256: String,
    pub local_attestation_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_local_attestation_sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestSummary {
    pub schema: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub kind: Option<String>,
    pub target: Option<String>,
    pub risk: Option<String>,
    pub load_mode: Option<String>,
    pub requested_caps: Vec<String>,
    pub granted_caps: Vec<String>,
    pub artifact_hash: Option<String>,
    pub base_image_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantBindingState {
    pub manifest_artifact_bound: bool,
    pub vm_report_manifest_bound: bool,
    pub vm_report_artifact_bound: bool,
    pub vm_report_result_passed: bool,
    pub local_attestation_manifest_bound: bool,
    pub local_attestation_artifact_bound: bool,
    pub local_attestation_report_bound: bool,
    pub local_attestation_grants_load_now: bool,
    pub local_approval_bound: bool,
    pub current_boot_scope: bool,
    pub recovery_artifact_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantPolicyResult {
    pub grant_source: String,
    pub computed_candidate_present: bool,
    pub grants_capability: bool,
    pub grants_load_now: bool,
    pub authorizes_guest_load: bool,
    pub can_load_now: bool,
    pub service_inventory_change: String,
    pub load_attempted: bool,
    pub loader: String,
    pub service_slot: String,
    pub durable_audit_record: String,
    pub rollback_plan: String,
}

pub fn compute_capability_grant(
    request: &ComputeCapabilityGrantRequest,
) -> Result<ComputedCapabilityGrantDiagnostic> {
    let (manifest, manifest_hash) = read_json_with_sha256(&request.manifest_path)?;
    let artifact_hash = read_file_sha256(&request.artifact_path)?;
    let (report, report_hash) = read_json_with_sha256(&request.vm_report_path)?;
    let (attestation, attestation_hash) = read_json_with_sha256(&request.local_attestation_path)?;

    let expected_attestation_hash = request
        .expected_local_attestation_sha256
        .as_deref()
        .map(normalize_hash_ref)
        .transpose()?;

    let manifest_summary = ManifestSummary {
        schema: str_at(&manifest, "/schema").map(ToOwned::to_owned),
        name: str_at(&manifest, "/name").map(ToOwned::to_owned),
        version: str_at(&manifest, "/version").map(ToOwned::to_owned),
        kind: str_at(&manifest, "/kind").map(ToOwned::to_owned),
        target: str_at(&manifest, "/target").map(ToOwned::to_owned),
        risk: str_at(&manifest, "/risk").map(ToOwned::to_owned),
        load_mode: str_at(&manifest, "/load_mode").map(ToOwned::to_owned),
        requested_caps: string_array_at(&manifest, "/requested_caps"),
        granted_caps: string_array_at(&manifest, "/granted_caps"),
        artifact_hash: hash_at(&manifest, "/artifact_hash"),
        base_image_hash: hash_at(&manifest, "/base_image_hash"),
    };

    let report_manifest_hash = hash_at(&report, "/candidate_manifest/sha256");
    let report_artifact_hash = hash_at(&report, "/candidate_artifact/sha256");
    let report_base_hash = hash_at(&report, "/base_image/sha256");
    let report_qemu_args_hash = hash_at(&report, "/qemu/args_sha256");
    let report_binding_manifest_hash =
        hash_at(&report, "/evidence_binding/candidate_manifest_sha256");
    let report_binding_artifact_hash =
        hash_at(&report, "/evidence_binding/candidate_artifact_sha256");
    let report_binding_base_hash = hash_at(&report, "/evidence_binding/base_image_sha256");
    let report_binding_qemu_args_hash = hash_at(&report, "/evidence_binding/qemu_args_sha256");

    let attestation_manifest_hash = hash_at(&attestation, "/manifest/sha256");
    let attestation_artifact_hash = hash_at(&attestation, "/artifact/sha256");
    let attestation_report_hash = hash_at(&attestation, "/vm_report/sha256");
    let attestation_report_base_hash = hash_at(&attestation, "/vm_report/base_image_sha256");
    let attestation_report_qemu_args_hash = hash_at(&attestation, "/vm_report/qemu_args_sha256");
    let attestation_binding_manifest_hash =
        hash_at(&attestation, "/evidence_binding/manifest_sha256");
    let attestation_binding_artifact_hash =
        hash_at(&attestation, "/evidence_binding/artifact_sha256");
    let attestation_binding_report_hash =
        hash_at(&attestation, "/evidence_binding/vm_report_sha256");
    let attestation_binding_base_hash =
        hash_at(&attestation, "/evidence_binding/base_image_sha256");
    let attestation_binding_qemu_args_hash =
        hash_at(&attestation, "/evidence_binding/qemu_args_sha256");

    let approval_tuple = format!(
        "manifest={};artifact={};report={};base={};mode={}",
        manifest_hash,
        artifact_hash,
        report_hash,
        report_base_hash.as_deref().unwrap_or(""),
        request.load_mode
    );
    let approval_tuple_hash = sha256_text(&approval_tuple);
    let approval_prefix = approval_tuple_hash
        .get(..16)
        .ok_or_else(|| anyhow!("internal approval tuple hash too short"))?;
    let expected_approval = format!(
        "APPROVE {} {}",
        request.load_mode.to_ascii_uppercase(),
        approval_prefix
    );
    let approval_phrase_hash = sha256_text(&request.local_approval);

    let mut denial_reasons = Vec::new();
    let mut warnings = Vec::new();

    require_equal(
        &mut denial_reasons,
        "requested_capability_must_be_cap_module_load_ephemeral",
        Some(request.requested_capability.as_str()),
        Some(DEFAULT_REQUESTED_CAPABILITY),
    );
    require_equal(
        &mut denial_reasons,
        "scope_must_be_current_boot",
        Some(request.scope.as_str()),
        Some(DEFAULT_SCOPE),
    );
    require_equal(
        &mut denial_reasons,
        "resource_must_be_live_service_graph",
        Some(request.resource.as_str()),
        Some(DEFAULT_RESOURCE),
    );

    require_equal(
        &mut denial_reasons,
        "manifest_schema_mismatch",
        manifest_summary.schema.as_deref(),
        Some(MODULE_MANIFEST_SCHEMA),
    );
    require_equal(
        &mut denial_reasons,
        "manifest_artifact_hash_mismatch",
        manifest_summary.artifact_hash.as_deref(),
        Some(artifact_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "manifest_load_mode_mismatch",
        manifest_summary.load_mode.as_deref(),
        Some(request.load_mode.as_str()),
    );
    if !manifest_summary.granted_caps.is_empty() {
        denial_reasons.push("manifest_granted_caps_non_empty".to_string());
    }
    if !manifest_summary
        .requested_caps
        .iter()
        .any(|cap| cap == DEFAULT_REQUESTED_CAPABILITY)
    {
        warnings
            .push("manifest_requested_caps_does_not_include_loader_capability_request".to_string());
    }

    require_equal(
        &mut denial_reasons,
        "vm_report_schema_mismatch",
        str_at(&report, "/schema"),
        Some(VM_TEST_REPORT_SCHEMA),
    );
    require_equal(
        &mut denial_reasons,
        "vm_report_result_not_passed",
        str_at(&report, "/result"),
        Some("passed"),
    );
    require_equal(
        &mut denial_reasons,
        "vm_report_manifest_hash_mismatch",
        report_manifest_hash.as_deref(),
        Some(manifest_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "vm_report_artifact_hash_mismatch",
        report_artifact_hash.as_deref(),
        Some(artifact_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "vm_report_evidence_manifest_hash_mismatch",
        report_binding_manifest_hash.as_deref(),
        report_manifest_hash.as_deref(),
    );
    require_equal(
        &mut denial_reasons,
        "vm_report_evidence_artifact_hash_mismatch",
        report_binding_artifact_hash.as_deref(),
        report_artifact_hash.as_deref(),
    );
    require_equal(
        &mut denial_reasons,
        "vm_report_evidence_base_image_hash_mismatch",
        report_binding_base_hash.as_deref(),
        report_base_hash.as_deref(),
    );
    require_equal(
        &mut denial_reasons,
        "vm_report_evidence_qemu_args_hash_mismatch",
        report_binding_qemu_args_hash.as_deref(),
        report_qemu_args_hash.as_deref(),
    );

    require_equal(
        &mut denial_reasons,
        "local_attestation_schema_mismatch",
        str_at(&attestation, "/schema"),
        Some(LOCAL_ATTESTATION_SCHEMA),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_load_mode_mismatch",
        str_at(&attestation, "/load_mode"),
        Some(request.load_mode.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_manifest_hash_mismatch",
        attestation_manifest_hash.as_deref(),
        Some(manifest_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_artifact_hash_mismatch",
        attestation_artifact_hash.as_deref(),
        Some(artifact_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_report_hash_mismatch",
        attestation_report_hash.as_deref(),
        Some(report_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_report_base_image_hash_mismatch",
        attestation_report_base_hash.as_deref(),
        report_base_hash.as_deref(),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_report_qemu_args_hash_mismatch",
        attestation_report_qemu_args_hash.as_deref(),
        report_qemu_args_hash.as_deref(),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_evidence_manifest_hash_mismatch",
        attestation_binding_manifest_hash.as_deref(),
        Some(manifest_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_evidence_artifact_hash_mismatch",
        attestation_binding_artifact_hash.as_deref(),
        Some(artifact_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_evidence_report_hash_mismatch",
        attestation_binding_report_hash.as_deref(),
        Some(report_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_evidence_base_image_hash_mismatch",
        attestation_binding_base_hash.as_deref(),
        report_base_hash.as_deref(),
    );
    require_equal(
        &mut denial_reasons,
        "local_attestation_evidence_qemu_args_hash_mismatch",
        attestation_binding_qemu_args_hash.as_deref(),
        report_qemu_args_hash.as_deref(),
    );
    if bool_at(&attestation, "/limits/grants_load_now").unwrap_or(true) {
        denial_reasons.push("local_attestation_grants_load_now_true".to_string());
    }
    if bool_at(&attestation, "/limits/requires_guest_loader") != Some(true) {
        denial_reasons.push("local_attestation_guest_loader_requirement_missing".to_string());
    }
    if bool_at(&attestation, "/limits/requires_kernel_policy_check") != Some(true) {
        denial_reasons.push("local_attestation_kernel_policy_requirement_missing".to_string());
    }

    require_equal(
        &mut denial_reasons,
        "approval_phrase_mismatch",
        Some(request.local_approval.as_str()),
        Some(expected_approval.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "approval_phrase_hash_mismatch",
        str_at(&attestation, "/approval/phrase_sha256"),
        Some(approval_phrase_hash.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "approval_expected_phrase_mismatch",
        str_at(&attestation, "/approval/expected_phrase"),
        Some(expected_approval.as_str()),
    );
    require_equal(
        &mut denial_reasons,
        "approval_tuple_hash_mismatch",
        str_at(&attestation, "/approval/tuple_sha256"),
        Some(approval_tuple_hash.as_str()),
    );
    if let Some(expected_attestation_hash) = expected_attestation_hash.as_deref() {
        require_equal(
            &mut denial_reasons,
            "local_attestation_hash_mismatch",
            Some(attestation_hash.as_str()),
            Some(expected_attestation_hash),
        );
    }

    let canonical_hash = canonical_grant_hash(
        request,
        &manifest_hash,
        &artifact_hash,
        &report_hash,
        &attestation_hash,
    );
    let valid_evidence = denial_reasons.is_empty();

    Ok(ComputedCapabilityGrantDiagnostic {
        schema: COMPUTED_CAPABILITY_GRANT_SCHEMA.to_string(),
        canonicalization: COMPUTED_CAPABILITY_GRANT_CANONICALIZATION.to_string(),
        status: if valid_evidence {
            "candidate_evidence_valid_load_still_denied".to_string()
        } else {
            "candidate_evidence_rejected".to_string()
        },
        valid_evidence,
        computed_capability_grant_hash: format!("sha256:{canonical_hash}"),
        request: GrantRequestRecord {
            requested_capability: request.requested_capability.clone(),
            load_mode: request.load_mode.clone(),
            subject: request.subject.clone(),
            resource: request.resource.clone(),
            scope: request.scope.clone(),
            risk: "modify_ram".to_string(),
        },
        evidence: GrantEvidenceRecord {
            manifest_sha256: format!("sha256:{manifest_hash}"),
            candidate_artifact_sha256: format!("sha256:{artifact_hash}"),
            vm_test_report_sha256: format!("sha256:{report_hash}"),
            local_attestation_sha256: format!("sha256:{attestation_hash}"),
            expected_local_attestation_sha256: expected_attestation_hash
                .map(|hash| format!("sha256:{hash}")),
        },
        manifest: manifest_summary,
        bindings: GrantBindingState {
            manifest_artifact_bound: manifest_summary_hash_equals(
                &manifest,
                "/artifact_hash",
                &artifact_hash,
            ),
            vm_report_manifest_bound: report_manifest_hash.as_deref()
                == Some(manifest_hash.as_str()),
            vm_report_artifact_bound: report_artifact_hash.as_deref()
                == Some(artifact_hash.as_str()),
            vm_report_result_passed: str_at(&report, "/result") == Some("passed"),
            local_attestation_manifest_bound: attestation_manifest_hash.as_deref()
                == Some(manifest_hash.as_str()),
            local_attestation_artifact_bound: attestation_artifact_hash.as_deref()
                == Some(artifact_hash.as_str()),
            local_attestation_report_bound: attestation_report_hash.as_deref()
                == Some(report_hash.as_str()),
            local_attestation_grants_load_now: bool_at(&attestation, "/limits/grants_load_now")
                .unwrap_or(true),
            local_approval_bound: request.local_approval == expected_approval,
            current_boot_scope: request.scope == DEFAULT_SCOPE,
            recovery_artifact_path: "separate_recovery_v0_gate_not_applicable".to_string(),
        },
        policy_result: GrantPolicyResult {
            grant_source: "host_local_policy_diagnostic".to_string(),
            computed_candidate_present: valid_evidence,
            grants_capability: false,
            grants_load_now: false,
            authorizes_guest_load: false,
            can_load_now: false,
            service_inventory_change: "none".to_string(),
            load_attempted: false,
            loader: "unavailable".to_string(),
            service_slot: "unallocated".to_string(),
            durable_audit_record: "missing".to_string(),
            rollback_plan: "missing_in_guest_policy".to_string(),
        },
        missing_guest_gates: vec![
            "raios.audit_record.v0".to_string(),
            "rollback_plan".to_string(),
            "module_loader".to_string(),
            "ram_only_service_slot".to_string(),
            "in_guest_policy_grant_writer".to_string(),
        ],
        denial_reasons,
        warnings,
    })
}

fn read_json_with_sha256(path: &Path) -> Result<(Value, String)> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let hash = sha256_bytes(&bytes);
    let text = String::from_utf8(bytes)
        .with_context(|| format!("{} is not valid UTF-8", path.display()))?;
    let json_text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let json = serde_json::from_str(json_text)
        .with_context(|| format!("parsing JSON {}", path.display()))?;
    Ok((json, hash))
}

fn read_file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn sha256_text(text: &str) -> String {
    sha256_bytes(text.as_bytes())
}

fn canonical_grant_hash(
    request: &ComputeCapabilityGrantRequest,
    manifest_hash: &str,
    artifact_hash: &str,
    report_hash: &str,
    attestation_hash: &str,
) -> String {
    let canonical = [
        format!("canonicalization={COMPUTED_CAPABILITY_GRANT_CANONICALIZATION}"),
        format!("schema={COMPUTED_CAPABILITY_GRANT_SCHEMA}"),
        format!("requested_capability={}", request.requested_capability),
        format!("load_mode={}", request.load_mode),
        format!("subject={}", request.subject),
        format!("resource={}", request.resource),
        format!("scope={}", request.scope),
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

fn str_at<'a>(json: &'a Value, pointer: &str) -> Option<&'a str> {
    json.pointer(pointer).and_then(Value::as_str)
}

fn bool_at(json: &Value, pointer: &str) -> Option<bool> {
    json.pointer(pointer).and_then(Value::as_bool)
}

fn string_array_at(json: &Value, pointer: &str) -> Vec<String> {
    json.pointer(pointer)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn hash_at(json: &Value, pointer: &str) -> Option<String> {
    str_at(json, pointer).and_then(|value| normalize_hash_ref(value).ok())
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

fn manifest_summary_hash_equals(manifest: &Value, pointer: &str, expected_hash: &str) -> bool {
    hash_at(manifest, pointer).as_deref() == Some(expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::{tempdir, TempDir};

    struct Fixture {
        _temp: TempDir,
        manifest_path: PathBuf,
        artifact_path: PathBuf,
        report_path: PathBuf,
        attestation_path: PathBuf,
        approval: String,
        attestation_hash: String,
    }

    #[test]
    fn computed_grant_validates_tuple_but_does_not_authorize_load() -> Result<()> {
        let fixture = valid_fixture()?;
        let diagnostic = compute_capability_grant(&request_from(&fixture))?;

        assert!(diagnostic.valid_evidence);
        assert_eq!(
            diagnostic.status,
            "candidate_evidence_valid_load_still_denied"
        );
        assert!(diagnostic.policy_result.computed_candidate_present);
        assert!(!diagnostic.policy_result.grants_capability);
        assert!(!diagnostic.policy_result.grants_load_now);
        assert!(!diagnostic.policy_result.authorizes_guest_load);
        assert!(!diagnostic.policy_result.can_load_now);
        assert_eq!(diagnostic.policy_result.loader, "unavailable");
        assert_eq!(diagnostic.policy_result.service_slot, "unallocated");
        assert_eq!(diagnostic.policy_result.service_inventory_change, "none");
        assert!(!diagnostic.policy_result.load_attempted);
        assert!(diagnostic
            .computed_capability_grant_hash
            .starts_with("sha256:"));
        assert!(diagnostic
            .missing_guest_gates
            .contains(&"raios.audit_record.v0".to_string()));
        Ok(())
    }

    #[test]
    fn computed_grant_rejects_mismatched_manifest_hash() -> Result<()> {
        let fixture = valid_fixture()?;
        mutate_json(&fixture.report_path, |json| {
            json["candidate_manifest"]["sha256"] = json!(wrong_hash());
            json["evidence_binding"]["candidate_manifest_sha256"] = json!(wrong_hash());
        })?;

        assert_rejects(&fixture, "vm_report_manifest_hash_mismatch")?;
        Ok(())
    }

    #[test]
    fn computed_grant_rejects_mismatched_artifact_hash() -> Result<()> {
        let fixture = valid_fixture()?;
        mutate_json(&fixture.report_path, |json| {
            json["candidate_artifact"]["sha256"] = json!(wrong_hash());
            json["evidence_binding"]["candidate_artifact_sha256"] = json!(wrong_hash());
        })?;

        assert_rejects(&fixture, "vm_report_artifact_hash_mismatch")?;
        Ok(())
    }

    #[test]
    fn computed_grant_rejects_mismatched_report_hash() -> Result<()> {
        let fixture = valid_fixture()?;
        mutate_json(&fixture.attestation_path, |json| {
            json["vm_report"]["sha256"] = json!(wrong_hash());
            json["evidence_binding"]["vm_report_sha256"] = json!(wrong_hash());
        })?;

        assert_rejects(&fixture, "local_attestation_report_hash_mismatch")?;
        Ok(())
    }

    #[test]
    fn computed_grant_rejects_mismatched_attestation_hash() -> Result<()> {
        let fixture = valid_fixture()?;
        let mut request = request_from(&fixture);
        request.expected_local_attestation_sha256 = Some(wrong_hash());

        let diagnostic = compute_capability_grant(&request)?;
        assert!(!diagnostic.valid_evidence);
        assert!(diagnostic
            .denial_reasons
            .contains(&"local_attestation_hash_mismatch".to_string()));
        Ok(())
    }

    #[test]
    fn computed_grant_rejects_manifest_granted_caps() -> Result<()> {
        let fixture = valid_fixture()?;
        mutate_json(&fixture.manifest_path, |json| {
            json["granted_caps"] = json!(["cap.system.snapshot.read"]);
        })?;

        assert_rejects(&fixture, "manifest_granted_caps_non_empty")?;
        Ok(())
    }

    #[test]
    fn computed_grant_rejects_wrong_approval_phrase() -> Result<()> {
        let fixture = valid_fixture()?;
        let mut request = request_from(&fixture);
        request.local_approval = "APPROVE RAM_ONLY wrong".to_string();

        let diagnostic = compute_capability_grant(&request)?;
        assert!(!diagnostic.valid_evidence);
        assert!(diagnostic
            .denial_reasons
            .contains(&"approval_phrase_mismatch".to_string()));
        Ok(())
    }

    #[test]
    fn computed_grant_rejects_attestation_that_grants_load_now() -> Result<()> {
        let fixture = valid_fixture()?;
        mutate_json(&fixture.attestation_path, |json| {
            json["limits"]["grants_load_now"] = json!(true);
        })?;

        assert_rejects(&fixture, "local_attestation_grants_load_now_true")?;
        Ok(())
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

        Ok(Fixture {
            _temp: temp,
            manifest_path,
            artifact_path,
            report_path,
            attestation_path,
            approval,
            attestation_hash,
        })
    }

    fn request_from(fixture: &Fixture) -> ComputeCapabilityGrantRequest {
        let mut request = ComputeCapabilityGrantRequest::new(
            fixture.manifest_path.clone(),
            fixture.artifact_path.clone(),
            fixture.report_path.clone(),
            fixture.attestation_path.clone(),
            fixture.approval.clone(),
        );
        request.expected_local_attestation_sha256 = Some(fixture.attestation_hash.clone());
        request
    }

    fn assert_rejects(fixture: &Fixture, reason: &str) -> Result<()> {
        let diagnostic = compute_capability_grant(&request_from(fixture))?;
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

    fn repeat_hash(ch: char) -> String {
        ch.to_string().repeat(64)
    }

    fn wrong_hash() -> String {
        repeat_hash('9')
    }
}
