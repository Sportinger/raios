use alloc::vec;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields_trailing_comma, end_response, raw_line,
        record_bool as b, record_field as f, record_sha, record_sha_or_null, record_str as s,
    },
    personal_shell_service::{self, PersonalShellProofMode},
    wasm_runtime::{self, PersonalShellProofCase},
};
use raios_core::{
    record::Value as V,
    scoped_wasm_import_grant::{PERSONAL_SHELL_SERVICE_ID, PERSONAL_SHELL_UI_IMPORTS},
};

/// Emits bounded current-boot evidence for the checked-in personal-shell proof.
pub(crate) fn emit_personal_shell_proof(method: &str) {
    let activation = request_personal_shell_activation(method);
    let proof = wasm_runtime::run_personal_shell_proof_probe();

    begin_response("ui.personal_shell_proof");
    emit_record_fields_trailing_comma(
        vec![
            f("method", s("ui.personal_shell_proof")),
            f("schema", s("raios.personal_shell_proof.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("non_default", b(true)),
            f("activation_mode", s(activation.mode)),
            f("activation_requested", b(activation.requested)),
            f("activation_request_reason", s(activation.reason)),
            f("service_id", s(PERSONAL_SHELL_SERVICE_ID)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
            f("owner_sealed", b(false)),
            f("artifact_sha256", record_sha(proof.artifact_sha256)),
            f(
                "artifact_identity_descriptor_sha256",
                record_sha(proof.artifact_identity_descriptor_sha256),
            ),
            f(
                "artifact_signature_evidence_sha256",
                record_sha(proof.artifact_signature_evidence_sha256),
            ),
            f(
                "load_descriptor_sha256",
                record_sha(proof.load_descriptor_sha256),
            ),
            f(
                "descriptor_signature_evidence_sha256",
                record_sha(proof.descriptor_signature_evidence_sha256),
            ),
            f(
                "authorized_import_list_sha256",
                record_sha(proof.authorized_import_list_sha256),
            ),
            f("authorized_imports", personal_shell_imports()),
            f("artifact_validation_ok", b(proof.artifact_validation_ok)),
            f(
                "authorized_import_count",
                V::U64(proof.authorized_import_count),
            ),
            f(
                "linked_host_import_count",
                V::U64(proof.linked_host_import_count),
            ),
            f(
                "fuel_budget",
                V::U64(wasm_runtime::PERSONAL_SHELL_WASM_FUEL_BUDGET),
            ),
            f("normal", proof_case(&proof.normal)),
            f("sanitized_input", proof_case(&proof.sanitized_input)),
            f(
                "frame_changed_after_sanitized_input",
                b(proof.frame_changed_after_sanitized_input),
            ),
            f("malformed_frame", proof_case(&proof.malformed_frame)),
            f(
                "malformed_frame_rejected_atomically",
                b(!proof.malformed_frame.accepted && proof.malformed_frame.frame_sha256.is_none()),
            ),
            f("clipped_overdraw", proof_case(&proof.clipped_overdraw)),
            f(
                "clipped_overdraw_proved",
                b(proof.clipped_overdraw.accepted && proof.clipped_overdraw.clipped_overdraw),
            ),
            f("guest_trap", proof_case(&proof.guest_trap)),
            f(
                "guest_trap_rejected",
                b(!proof.guest_trap.accepted && proof.guest_trap.run_outcome == "trap"),
            ),
            f("fuel_exhaustion", proof_case(&proof.fuel_exhaustion)),
            f(
                "fuel_exhaustion_rejected",
                b(!proof.fuel_exhaustion.accepted
                    && proof.fuel_exhaustion.run_outcome == "fuel_exhausted"),
            ),
            f(
                "missing_frame_submit_linker_denial",
                s(proof.missing_frame_submit_linker_denial),
            ),
            f("broader_import_denial", s(proof.broader_import_denial)),
            f("generic_loader_used", b(false)),
            f("accepts_external_artifact_bytes", b(false)),
            f("authorizes_external_artifact_intake", b(false)),
            f("authorizes_arbitrary_shell_artifacts", b(false)),
            f("authorizes_persistent_install", b(false)),
            f("writes_persistent_state", b(false)),
            f("authorizes_provider_access", b(false)),
            f("provider_auto_load", b(false)),
            f("authorizes_provider_export", b(false)),
            f("authorizes_secret_access", b(false)),
            f("authorizes_secret_plaintext", b(false)),
            f("authorizes_network_access", b(false)),
            f("authorizes_recovery_access", b(false)),
            f("authorizes_capability_decision", b(false)),
            f("authorizes_raw_framebuffer_access", b(false)),
            f("authorizes_broader_mutation", b(false)),
            f("persistent_service_install", b(false)),
            f(
                "service_inventory_change",
                s(if activation.requested {
                    "pending_current_boot_activation"
                } else {
                    "none"
                }),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("ui.personal_shell_proof");
}

struct PersonalShellActivation {
    mode: &'static str,
    requested: bool,
    reason: &'static str,
}

fn request_personal_shell_activation(method: &str) -> PersonalShellActivation {
    let suffix = method
        .strip_prefix("ui.personal_shell_proof")
        .unwrap_or_default()
        .trim();
    let (mode, label) = match suffix {
        "" => (Some(PersonalShellProofMode::Normal), "normal"),
        "trap" => (Some(PersonalShellProofMode::Trap), "trap"),
        "fuel" => (Some(PersonalShellProofMode::FuelExhaustion), "fuel"),
        _ => (None, "invalid"),
    };
    let Some(mode) = mode else {
        return PersonalShellActivation {
            mode: label,
            requested: false,
            reason: "invalid_bounded_test_mode",
        };
    };
    let requested = personal_shell_service::request_current_boot_proof_start(mode);
    PersonalShellActivation {
        mode: label,
        requested,
        reason: if requested {
            "queued_for_core_owned_shell_host"
        } else {
            "current_boot_start_already_pending"
        },
    }
}

fn personal_shell_imports() -> V<'static> {
    V::Array(
        PERSONAL_SHELL_UI_IMPORTS
            .iter()
            .map(|(module, name)| V::InlineObject(vec![f("module", s(module)), f("name", s(name))]))
            .collect(),
    )
}

fn proof_case(case: &PersonalShellProofCase) -> V<'static> {
    V::InlineObject(vec![
        f("accepted", b(case.accepted)),
        f("instantiation_error_kind", s(case.instantiation_error_kind)),
        f("run_outcome", s(case.run_outcome)),
        f("fuel_used", V::U64(case.fuel_used)),
        f("frame_sha256", record_sha_or_null(case.frame_sha256)),
        f("clipped_overdraw", b(case.clipped_overdraw)),
    ])
}
