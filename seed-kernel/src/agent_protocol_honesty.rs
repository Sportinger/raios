use alloc::vec;

use crate::{
    agent_protocol_provider::{live_provider_trust_honesty, LiveProviderTrustHonesty},
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha as sha, record_static_str_array, record_str as s,
    },
    agent_protocol_time::{live_time_authority_honesty, LiveTimeAuthorityHonesty},
    entropy, owner_key,
};
use raios_core::{
    record::Value as V,
    scoped_external_acquisition_honesty::{
        evaluate_external_acquisition_honesty, ExternalAcquisitionHonestyInput,
        SCOPED_EXTERNAL_ACQUISITION_HONESTY_DECISION_ID,
        SCOPED_EXTERNAL_ACQUISITION_HONESTY_DECISION_MARKER,
        SCOPED_EXTERNAL_ACQUISITION_HONESTY_DECISION_SCHEMA,
    },
    scoped_provider_trust_honesty::{
        SCOPED_PROVIDER_TRUST_HONESTY_DECISION_ID, SCOPED_PROVIDER_TRUST_HONESTY_DECISION_MARKER,
        SCOPED_PROVIDER_TRUST_HONESTY_DECISION_SCHEMA,
    },
    scoped_time_authority_honesty::{
        EXPECTED_TIME_SOURCE, SCOPED_TIME_AUTHORITY_HONESTY_DECISION_ID,
        SCOPED_TIME_AUTHORITY_HONESTY_DECISION_MARKER,
        SCOPED_TIME_AUTHORITY_HONESTY_DECISION_SCHEMA,
    },
    scoped_wasm_import_grant::KNOWN_HOST_IMPORTS,
};

const TRUST_TIER: &str = "dev_key_not_owner_sealed";
const OWNER_SEALED: bool = false;
const PROVIDER_WRITE_NOT_ATTEMPTED: bool = true;
const TRANSMISSION: bool = false;
const CERT_TIME_VALIDATES_CERT_TIME: bool = false;
const PROVIDER_EXPORT_DISABLED: bool = true;
const PROVIDER_EXPORT_CAN_EXPORT: bool = false;
const PROVIDER_EXPORT_BODY_ATTACHED: bool = false;
const WASM_AUTHORIZES_NEW_IMPORTS: bool = false;
const WASM_AUTHORIZES_BEYOND_ENV: bool = false;
const WASM_REPORT_GRANTS_AUTHORITY: bool = false;
const EXTERNAL_ACQUISITION_ACTIVE: bool = false;
const EXTERNAL_WOULD_BE_CANDIDATE_INTAKE_ONLY: bool = true;
const OWNER_KEY_PROVISIONING_ID: &str = "owner_key.provisioning.posture.current_boot";
const OWNER_KEY_PERSISTENT_INSTALL_POLICY: &str =
    "generate_hardware_bound_owner_key_on_persistent_install";
const OWNER_KEY_RAM_BOOT_POLICY: &str = "ephemeral_current_boot_key_only";
const OWNER_KEY_STATUS: &str = "denied_missing_hardware_bound_owner_key_evidence";
const OWNER_KEY_REASON: &str = "hardware_bound_owner_key_evidence_missing";
const OWNER_KEY_EVIDENCE_INPUT_ID: &str = "owner_key.evidence_input.current_boot";
const OWNER_KEY_HARDWARE_BINDING_SOURCE: &str = "tpm_or_platform_seal";
const OWNER_KEY_RAM_INPUT_READY_STATUS: &str =
    "ram_ephemeral_input_ready_persistent_hardware_binding_missing";
const OWNER_KEY_ENTROPY_NOT_READY_STATUS: &str =
    "denied_entropy_not_ready_and_hardware_binding_missing";
const OWNER_KEY_ENTROPY_NOT_READY_REASON: &str = "entropy_evidence_missing";
const OWNER_KEY_RAM_CANDIDATE_STATUS: &str =
    "ram_ephemeral_candidate_generated_persistent_hardware_binding_missing";

pub(crate) fn emit_system_honesty_report(_request: &str) {
    let provider = live_provider_trust_honesty();
    let time = live_time_authority_honesty();
    let external = external_acquisition_projection();
    let entropy_stats = entropy::stats();
    let owner_key_snapshot = owner_key::snapshot();

    let provider_no_overclaim = !provider.descriptor.claims_chain_validated
        && !provider.descriptor.claims_time_validated
        && !provider.decision.chain_validated
        && !provider.decision.time_validated
        && !provider.decision.authorizes_provider_request
        && !provider.decision.authorizes_provider_export
        && PROVIDER_WRITE_NOT_ATTEMPTED
        && !TRANSMISSION;
    let time_no_overclaim = !time.decision.trusted
        && !time.decision.source_verified
        && !time.decision.validates_cert_time
        && !time.decision.timezone_validated
        && !time.decision.authorizes_provider_request
        && !time.decision.authorizes_provider_export
        && !time.decision.durable_write
        && !time.decision.capability_granted;
    let cert_time_no_overclaim = !CERT_TIME_VALIDATES_CERT_TIME;
    let provider_export_no_overclaim = PROVIDER_EXPORT_DISABLED
        && !PROVIDER_EXPORT_CAN_EXPORT
        && !PROVIDER_EXPORT_BODY_ATTACHED
        && PROVIDER_WRITE_NOT_ATTEMPTED;
    let wasm_no_overclaim = !WASM_AUTHORIZES_NEW_IMPORTS
        && !WASM_AUTHORIZES_BEYOND_ENV
        && !WASM_REPORT_GRANTS_AUTHORITY;
    let external_no_overclaim = !EXTERNAL_ACQUISITION_ACTIVE
        && EXTERNAL_WOULD_BE_CANDIDATE_INTAKE_ONLY
        && !external.decision.authorizes_acquisition
        && !external.decision.authorizes_install
        && !external.decision.authorizes_load;
    let owner_key_no_overclaim = !OWNER_SEALED && TRUST_TIER == "dev_key_not_owner_sealed";
    let no_dishonest_overclaim = provider_no_overclaim
        && time_no_overclaim
        && cert_time_no_overclaim
        && provider_export_no_overclaim
        && wasm_no_overclaim
        && external_no_overclaim
        && owner_key_no_overclaim;

    begin_response("system.honesty_report");
    emit_record_fields(
        vec![
            f("schema", s("raios.system_honesty_report.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("method", s("system.honesty_report")),
            f("owner_sealed", b(OWNER_SEALED)),
            f("trust_tier", s(TRUST_TIER)),
            f("read_only", b(true)),
            f("durable_write", b(false)),
            f("provider_write", s("not_attempted")),
            f("transmission", b(false)),
            f("state_change", b(false)),
            f("capability_granted", b(false)),
            f("provider_no_overclaim", b(provider_no_overclaim)),
            f("time_no_overclaim", b(time_no_overclaim)),
            f("cert_time_no_overclaim", b(cert_time_no_overclaim)),
            f(
                "provider_export_no_overclaim",
                b(provider_export_no_overclaim),
            ),
            f("wasm_no_overclaim", b(wasm_no_overclaim)),
            f("external_no_overclaim", b(external_no_overclaim)),
            f("owner_key_no_overclaim", b(owner_key_no_overclaim)),
            f("no_dishonest_overclaim", b(no_dishonest_overclaim)),
            f("provider_trust", provider_trust_record(&provider)),
            f("time_authority", time_authority_record(&time)),
            f("cert_time_validation", cert_time_validation_record()),
            f("provider_export", provider_export_record()),
            f("wasm_import_surface", wasm_import_surface_record()),
            f("external_acquisition", external.record),
            f(
                "owner_key_provisioning",
                owner_key_provisioning_record(entropy_stats, owner_key_snapshot),
            ),
        ],
        6,
    );
    end_response("system.honesty_report");
}

fn provider_trust_record(honesty: &LiveProviderTrustHonesty) -> V<'static> {
    V::Object(vec![
        f(
            "decision_schema",
            s(SCOPED_PROVIDER_TRUST_HONESTY_DECISION_SCHEMA),
        ),
        f("decision_id", s(SCOPED_PROVIDER_TRUST_HONESTY_DECISION_ID)),
        f(
            "decision_marker",
            s(SCOPED_PROVIDER_TRUST_HONESTY_DECISION_MARKER),
        ),
        f("provider_id", s(honesty.descriptor.provider_id)),
        f("descriptor_id", s(honesty.descriptor.id)),
        f("host", s(honesty.descriptor.host)),
        f("descriptor_sha256", sha(honesty.descriptor_sha256)),
        f("trust_state", s(honesty.descriptor.trust_state)),
        f("chain_policy", s(honesty.descriptor.chain_policy)),
        f("time_policy", s(honesty.descriptor.time_policy)),
        f(
            "claims_chain_validated",
            b(honesty.descriptor.claims_chain_validated),
        ),
        f(
            "claims_time_validated",
            b(honesty.descriptor.claims_time_validated),
        ),
        f("performed", b(honesty.decision.performed)),
        f("status", s(honesty.decision.status)),
        f("reason", s(honesty.decision.reason)),
        f("honest", b(honesty.decision.honest)),
        f("chain_validated", b(honesty.decision.chain_validated)),
        f("time_validated", b(honesty.decision.time_validated)),
        f(
            "authorizes_provider_request",
            b(honesty.decision.authorizes_provider_request),
        ),
        f(
            "authorizes_provider_export",
            b(honesty.decision.authorizes_provider_export),
        ),
        f("provider_write", s("not_attempted")),
        f("transmission", b(false)),
    ])
}

fn time_authority_record(honesty: &LiveTimeAuthorityHonesty) -> V<'static> {
    V::Object(vec![
        f(
            "decision_schema",
            s(SCOPED_TIME_AUTHORITY_HONESTY_DECISION_SCHEMA),
        ),
        f("decision_id", s(SCOPED_TIME_AUTHORITY_HONESTY_DECISION_ID)),
        f(
            "decision_marker",
            s(SCOPED_TIME_AUTHORITY_HONESTY_DECISION_MARKER),
        ),
        f("source", s(EXPECTED_TIME_SOURCE)),
        f("read_status", s(honesty.read_status)),
        f("trusted", b(honesty.decision.trusted)),
        f("source_verified", b(honesty.decision.source_verified)),
        f(
            "validates_cert_time",
            b(honesty.decision.validates_cert_time),
        ),
        f("timezone_validated", b(honesty.decision.timezone_validated)),
        f(
            "authorizes_provider_request",
            b(honesty.decision.authorizes_provider_request),
        ),
        f(
            "authorizes_provider_export",
            b(honesty.decision.authorizes_provider_export),
        ),
        f("durable_write", b(honesty.decision.durable_write)),
        f("capability_granted", b(honesty.decision.capability_granted)),
        f("provider_write", s("not_attempted")),
        f("transmission", b(false)),
        f("performed", b(honesty.decision.performed)),
        f("status", s(honesty.decision.status)),
        f("reason", s(honesty.decision.reason)),
    ])
}

fn cert_time_validation_record() -> V<'static> {
    V::Object(vec![
        f("basis", s("unverified")),
        f("performed", b(false)),
        f("validates_cert_time", b(CERT_TIME_VALIDATES_CERT_TIME)),
        f("live_certificate", s("not_read")),
        f("provider_write", s("not_attempted")),
        f("transmission", b(TRANSMISSION)),
    ])
}

fn provider_export_record() -> V<'static> {
    V::Object(vec![
        f("state", s("disabled")),
        f("can_export", b(PROVIDER_EXPORT_CAN_EXPORT)),
        f(
            "context_attached_to_provider_body",
            b(PROVIDER_EXPORT_BODY_ATTACHED),
        ),
        f("provider_write", s("not_attempted")),
    ])
}

fn wasm_import_surface_record() -> V<'static> {
    V::Object(vec![
        f("capability_envelope", s("wasmi_linker_import_surface")),
        f("per_instance_linker_enforced", b(true)),
        f(
            "known_host_imports",
            record_static_str_array(&["env.log", "env.counter_get"]),
        ),
        f(
            "known_host_import_count",
            V::U64(KNOWN_HOST_IMPORTS.len() as u64),
        ),
        f("authorizes_new_imports", b(WASM_AUTHORIZES_NEW_IMPORTS)),
        f("authorizes_beyond_env", b(WASM_AUTHORIZES_BEYOND_ENV)),
        f("report_grants_authority", b(WASM_REPORT_GRANTS_AUTHORITY)),
    ])
}

fn owner_key_evidence_input_status(stats: entropy::EntropyStats) -> &'static str {
    if stats.ready {
        OWNER_KEY_RAM_INPUT_READY_STATUS
    } else {
        OWNER_KEY_ENTROPY_NOT_READY_STATUS
    }
}

fn owner_key_evidence_input_reason(stats: entropy::EntropyStats) -> &'static str {
    if stats.ready {
        OWNER_KEY_REASON
    } else {
        OWNER_KEY_ENTROPY_NOT_READY_REASON
    }
}

fn owner_key_candidate_status(snapshot: owner_key::OwnerKeySnapshot) -> &'static str {
    if snapshot.generated {
        OWNER_KEY_RAM_CANDIDATE_STATUS
    } else {
        OWNER_KEY_STATUS
    }
}

fn optional_str(value: Option<&'static str>) -> V<'static> {
    match value {
        Some(value) => s(value),
        None => V::Null,
    }
}

fn optional_sha(value: Option<[u8; 32]>) -> V<'static> {
    match value {
        Some(value) => sha(value),
        None => V::Null,
    }
}

fn owner_key_evidence_input_record(
    stats: entropy::EntropyStats,
    snapshot: owner_key::OwnerKeySnapshot,
) -> V<'static> {
    V::Object(vec![
        f("id", s(OWNER_KEY_EVIDENCE_INPUT_ID)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("consumes_entropy_source", s("core.entropy")),
        f("entropy_evidence_present", b(stats.ready)),
        f(
            "entropy_status",
            s(if stats.ready { "ready" } else { "waiting" }),
        ),
        f("entropy_source_rdrand_observed", b(stats.used_rdrand)),
        f("entropy_pool_fill", V::U64(stats.pool_fill as u64)),
        f(
            "entropy_pool_capacity",
            V::U64(entropy::POOL_CAPACITY as u64),
        ),
        f("entropy_total_collected", V::U64(stats.total_collected)),
        f(
            "hardware_binding_source",
            s(OWNER_KEY_HARDWARE_BINDING_SOURCE),
        ),
        f(
            "hardware_binding_probe_source",
            s(owner_key::HARDWARE_BINDING_PROBE_SOURCE),
        ),
        f(
            "hardware_binding_probe_performed",
            b(snapshot.hardware_binding.probe_performed),
        ),
        f(
            "hardware_binding_probe_status",
            s(snapshot.hardware_binding.status),
        ),
        f(
            "hardware_binding_probe_reason",
            s(snapshot.hardware_binding.reason),
        ),
        f(
            "acpi_rsdp_present",
            b(snapshot.hardware_binding.acpi_rsdp_present),
        ),
        f(
            "acpi_root_table_valid",
            b(snapshot.hardware_binding.acpi_root_table_valid),
        ),
        f(
            "tpm2_acpi_table_present",
            b(snapshot.hardware_binding.tpm2_acpi_table_present),
        ),
        f(
            "tpm2_acpi_table_phys",
            V::U64(snapshot.hardware_binding.tpm2_acpi_table_phys),
        ),
        f(
            "tpm2_acpi_table_length",
            V::U64(snapshot.hardware_binding.tpm2_acpi_table_length as u64),
        ),
        f(
            "tpm2_acpi_table_revision",
            V::U64(snapshot.hardware_binding.tpm2_acpi_table_revision as u64),
        ),
        f(
            "tpm2_table_details_valid",
            b(snapshot.hardware_binding.tpm2_table_details_valid),
        ),
        f(
            "tpm2_platform_class",
            V::U64(snapshot.hardware_binding.tpm2_platform_class as u64),
        ),
        f(
            "tpm2_control_area",
            V::U64(snapshot.hardware_binding.tpm2_control_area),
        ),
        f(
            "tpm2_start_method",
            V::U64(snapshot.hardware_binding.tpm2_start_method as u64),
        ),
        f(
            "tpm2_interface_kind",
            s(snapshot.hardware_binding.tpm2_interface_kind),
        ),
        f(
            "tpm2_interface_status_probe_performed",
            b(snapshot
                .hardware_binding
                .tpm2_interface_status_probe_performed),
        ),
        f(
            "tpm2_interface_status",
            s(snapshot.hardware_binding.tpm2_interface_status),
        ),
        f(
            "tpm2_interface_status_reason",
            s(snapshot.hardware_binding.tpm2_interface_status_reason),
        ),
        f(
            "tpm2_status_read_plan_available",
            b(snapshot.hardware_binding.tpm2_status_read_plan_available),
        ),
        f(
            "tpm2_status_register_kind",
            s(snapshot.hardware_binding.tpm2_status_register_kind),
        ),
        f(
            "tpm2_status_register_phys",
            V::U64(snapshot.hardware_binding.tpm2_status_register_phys),
        ),
        f(
            "tpm2_status_register_width_bytes",
            V::U64(snapshot.hardware_binding.tpm2_status_register_width_bytes as u64),
        ),
        f(
            "tpm2_status_read_plan_reason",
            s(snapshot.hardware_binding.tpm2_status_read_plan_reason),
        ),
        f("hardware_binding_evidence_present", b(false)),
        f("tpm_binding_state", s(snapshot.hardware_binding.status)),
        f("ram_boot_ephemeral_input_ready", b(stats.ready)),
        f("persistent_install_input_ready", b(false)),
        f("status", s(owner_key_evidence_input_status(stats))),
        f("reason", s(owner_key_evidence_input_reason(stats))),
        f("ram_candidate_generated", b(snapshot.generated)),
        f("ram_candidate_id", s(owner_key::RAM_CANDIDATE_ID)),
        f("ram_candidate_handle", optional_str(snapshot.handle)),
        f(
            "ram_candidate_fingerprint",
            optional_sha(snapshot.fingerprint),
        ),
        f("authorizes_key_generation", b(false)),
        f("authorizes_owner_seal", b(false)),
        f("authorizes_persistent_install", b(false)),
        f("authorizes_load", b(false)),
        f("durable_write", b(false)),
    ])
}

fn owner_key_provisioning_record(
    stats: entropy::EntropyStats,
    snapshot: owner_key::OwnerKeySnapshot,
) -> V<'static> {
    V::Object(vec![
        f("id", s(OWNER_KEY_PROVISIONING_ID)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("automatic_generation_intended", b(true)),
        f("automatic_generation_performed", b(snapshot.generated)),
        f(
            "persistent_install_policy",
            s(OWNER_KEY_PERSISTENT_INSTALL_POLICY),
        ),
        f("ram_boot_policy", s(OWNER_KEY_RAM_BOOT_POLICY)),
        f("persistent_install_hardware_binding_required", b(true)),
        f("entropy_evidence_present", b(stats.ready)),
        f(
            "hardware_binding_probe_source",
            s(owner_key::HARDWARE_BINDING_PROBE_SOURCE),
        ),
        f(
            "hardware_binding_probe_performed",
            b(snapshot.hardware_binding.probe_performed),
        ),
        f(
            "hardware_binding_probe_status",
            s(snapshot.hardware_binding.status),
        ),
        f(
            "hardware_binding_probe_reason",
            s(snapshot.hardware_binding.reason),
        ),
        f(
            "tpm2_acpi_table_present",
            b(snapshot.hardware_binding.tpm2_acpi_table_present),
        ),
        f(
            "tpm2_acpi_table_phys",
            V::U64(snapshot.hardware_binding.tpm2_acpi_table_phys),
        ),
        f(
            "tpm2_acpi_table_length",
            V::U64(snapshot.hardware_binding.tpm2_acpi_table_length as u64),
        ),
        f(
            "tpm2_acpi_table_revision",
            V::U64(snapshot.hardware_binding.tpm2_acpi_table_revision as u64),
        ),
        f(
            "tpm2_table_details_valid",
            b(snapshot.hardware_binding.tpm2_table_details_valid),
        ),
        f(
            "tpm2_platform_class",
            V::U64(snapshot.hardware_binding.tpm2_platform_class as u64),
        ),
        f(
            "tpm2_control_area",
            V::U64(snapshot.hardware_binding.tpm2_control_area),
        ),
        f(
            "tpm2_start_method",
            V::U64(snapshot.hardware_binding.tpm2_start_method as u64),
        ),
        f(
            "tpm2_interface_kind",
            s(snapshot.hardware_binding.tpm2_interface_kind),
        ),
        f(
            "tpm2_interface_status_probe_performed",
            b(snapshot
                .hardware_binding
                .tpm2_interface_status_probe_performed),
        ),
        f(
            "tpm2_interface_status",
            s(snapshot.hardware_binding.tpm2_interface_status),
        ),
        f(
            "tpm2_interface_status_reason",
            s(snapshot.hardware_binding.tpm2_interface_status_reason),
        ),
        f(
            "tpm2_status_read_plan_available",
            b(snapshot.hardware_binding.tpm2_status_read_plan_available),
        ),
        f(
            "tpm2_status_register_kind",
            s(snapshot.hardware_binding.tpm2_status_register_kind),
        ),
        f(
            "tpm2_status_register_phys",
            V::U64(snapshot.hardware_binding.tpm2_status_register_phys),
        ),
        f(
            "tpm2_status_register_width_bytes",
            V::U64(snapshot.hardware_binding.tpm2_status_register_width_bytes as u64),
        ),
        f(
            "tpm2_status_read_plan_reason",
            s(snapshot.hardware_binding.tpm2_status_read_plan_reason),
        ),
        f("hardware_binding_evidence_present", b(false)),
        f("ram_boot_ephemeral_input_ready", b(stats.ready)),
        f("persistent_install_input_ready", b(false)),
        f("persistent_owner_key_generated", b(false)),
        f("ram_boot_ephemeral_key_allowed", b(true)),
        f("ram_boot_entropy_required", b(true)),
        f("ram_boot_ephemeral_key_generated", b(snapshot.generated)),
        f("ram_boot_ephemeral_key_id", s(owner_key::RAM_CANDIDATE_ID)),
        f(
            "ram_boot_ephemeral_key_handle",
            optional_str(snapshot.handle),
        ),
        f(
            "ram_boot_ephemeral_key_algorithm",
            s(owner_key::RAM_CANDIDATE_ALGORITHM),
        ),
        f(
            "ram_boot_ephemeral_key_secret_len",
            V::U64(snapshot.secret_len as u64),
        ),
        f(
            "ram_boot_ephemeral_key_material_classification",
            s("secret"),
        ),
        f(
            "ram_boot_ephemeral_key_fingerprint",
            optional_sha(snapshot.fingerprint),
        ),
        f("owner_key_material_exported", b(false)),
        f("status", s(owner_key_candidate_status(snapshot))),
        f("reason", s(OWNER_KEY_REASON)),
        f("owner_sealed", b(OWNER_SEALED)),
        f("trust_tier", s(TRUST_TIER)),
        f("authorizes_owner_seal", b(false)),
        f("authorizes_persistent_install", b(false)),
        f("authorizes_load", b(false)),
        f("durable_write", b(false)),
        f(
            "owner_key_evidence_input",
            owner_key_evidence_input_record(stats, snapshot),
        ),
    ])
}

struct ExternalAcquisitionProjection {
    decision: raios_core::scoped_external_acquisition_honesty::ExternalAcquisitionHonestyDecision,
    record: V<'static>,
}

fn external_acquisition_projection() -> ExternalAcquisitionProjection {
    let input = ExternalAcquisitionHonestyInput {
        source_kind: Some("standing_policy_projection_not_live_acquisition"),
        artifact_sha256_present: true,
        intake_as_candidate: true,
        claims_installable: false,
        claims_load_authorized_by_distribution: false,
        requires_m6_reverify_for_load: true,
        claims_owner_sealed: false,
    };
    let decision = evaluate_external_acquisition_honesty(&input);
    let record = V::Object(vec![
        f(
            "decision_schema",
            s(SCOPED_EXTERNAL_ACQUISITION_HONESTY_DECISION_SCHEMA),
        ),
        f(
            "decision_id",
            s(SCOPED_EXTERNAL_ACQUISITION_HONESTY_DECISION_ID),
        ),
        f(
            "decision_marker",
            s(SCOPED_EXTERNAL_ACQUISITION_HONESTY_DECISION_MARKER),
        ),
        f("projection", s("standing_policy_not_live_acquisition")),
        f("acquisition_active", b(EXTERNAL_ACQUISITION_ACTIVE)),
        f("external_distribution_active", b(false)),
        f(
            "would_be_candidate_intake_only",
            b(EXTERNAL_WOULD_BE_CANDIDATE_INTAKE_ONLY),
        ),
        f("live_acquisition", s("not_attempted")),
        f("performed", b(decision.performed)),
        f("status", s(decision.status)),
        f("reason", s(decision.reason)),
        f("honest", b(decision.honest)),
        f("authorizes_acquisition", b(decision.authorizes_acquisition)),
        f("authorizes_install", b(decision.authorizes_install)),
        f("authorizes_load", b(decision.authorizes_load)),
    ]);

    ExternalAcquisitionProjection { decision, record }
}
