use raios_core::{
    project_install::{
        install_action_signature_payload_sha256, validate_install_action,
        validate_ui_program_install_envelope, ProjectInstallAction, ProjectInstallActionKind,
        ProjectInstallAuthority, UiProgramInstallEnvelope, PROJECT_INSTALL_TRUST_TIER,
    },
    promotion_attestation::{
        verify_promotion_authority_signature, PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
    },
    scoped_wasm_import_grant::PERSONAL_SHELL_SERVICE_ID,
    sha256_bytes,
    ui_program::{Program, PROGRAM_ABI_VERSION},
};
use spin::Mutex;

use crate::{
    agent_protocol::{artifact_store, boot_control, durable_store}, agent_protocol_project_install,
    pci, program_workspace, project_install_store, serial,
};

#[derive(Clone, Copy)]
struct InstalledProgram {
    program_sha256: [u8; 32],
    activation_approval_sha256: [u8; 32],
    install_envelope_sha256: [u8; 32],
    install_action_sha256: [u8; 32],
    install_authorization_frame_sha256: [u8; 32],
    promotion_transaction_sha256: [u8; 32],
    program_persist_frame_sha256: [u8; 32],
    generation: u64,
    sequence: u64,
}

struct State {
    installed: Option<InstalledProgram>,
}

impl State {
    const fn new() -> Self {
        Self { installed: None }
    }
}

static STATE: Mutex<State> = Mutex::new(State::new());

pub(crate) fn install_approved_from_pointer(
    envelope: &UiProgramInstallEnvelope,
    action: &ProjectInstallAction,
) -> Result<(), &'static str> {
    if STATE.lock().installed.is_some() {
        return Err("ui_program_already_installed");
    }
    let approved = validate_install_inputs(envelope, action)?;
    let signature_len = action.authority_signature.len();
    if signature_len == 0 || signature_len > 256 {
        return Err("project_install_signature_invalid");
    }
    let mut signature_der = [0u8; 256];
    signature_der[..signature_len].copy_from_slice(&action.authority_signature);
    let authorization = durable_store::UiProgramInstallAuthorization {
        subject: durable_store::PromotionSubject::UiProgram,
        engine_service_id: PERSONAL_SHELL_SERVICE_ID,
        program_abi_version: PROGRAM_ABI_VERSION,
        canonical_program_sha256: approved.canonical_program_sha256,
        canonical_program_byte_len: approved.canonical_program_byte_len,
        activation_approval_sha256: approved.activation_approval_sha256,
        install_envelope_sha256: envelope.envelope_sha256,
        install_action_sha256: action.action_sha256,
        install_action_message_sha256: install_action_signature_payload_sha256(action)
            .map_err(|error| error.reason())?,
        authority_evidence_sha256: action.authority_evidence_sha256,
        physical_approval_sha256: action
            .physical_approval_sha256
            .ok_or("physical_install_approval_missing")?,
        authority_key_sha256: action
            .authority_key_sha256
            .ok_or("install_action_signature_not_verified")?,
        signature_der,
        signature_len,
        generation: action.generation,
        log_sequence: action.log_sequence,
        previous_commit_sha256: action.previous_commit_sha256,
        trust_tier: PROJECT_INSTALL_TRUST_TIER,
    };
    let appended_authorization = durable_store::append_install_authorization(&authorization);
    if !appended_authorization.performed {
        return Err(appended_authorization.reason);
    }
    let install_authorization_frame_sha256 = appended_authorization
        .frame_sha256
        .ok_or("install_authorization_readback_missing")?;
    let promotion = durable_store::UiProgramPromotionTransactionRecord {
        subject: durable_store::PromotionSubject::UiProgram,
        transaction_kind: durable_store::PromotionTransactionKind::Promote,
        engine_service_id: PERSONAL_SHELL_SERVICE_ID,
        program_abi_version: PROGRAM_ABI_VERSION,
        canonical_program_sha256: approved.canonical_program_sha256,
        canonical_program_byte_len: approved.canonical_program_byte_len,
        activation_approval_sha256: approved.activation_approval_sha256,
        install_authorization_present: true,
        install_envelope_binds_activation: true,
        install_action_signature_verified: true,
        physical_install_approval_consumed: true,
        install_authorization_frame_sha256,
        canonical_verified: true,
        activation_approval_consumed: true,
        generation: action.generation,
        rollback_apply_event_id: None,
    };
    let appended_promotion = durable_store::append_promotion_transaction(&promotion, true);
    if !appended_promotion.performed {
        return Err(appended_promotion.reason);
    }
    let promotion_transaction_sha256 = appended_promotion
        .frame_sha256
        .ok_or("promotion_transaction_readback_missing")?;
    let persisted = artifact_store::persist_ui_program(
        &appended_promotion,
        &approved.canonical_bytes,
        approved.canonical_program_sha256,
        approved.activation_approval_sha256,
        envelope.envelope_sha256,
        install_authorization_frame_sha256,
    );
    if !persisted.performed {
        return Err(persisted.reason);
    }
    let program_persist_frame_sha256 = persisted
        .artifact_persist_frame_sha256
        .ok_or("program_persist_readback_missing")?;
    let installed = InstalledProgram {
        program_sha256: approved.canonical_program_sha256,
        activation_approval_sha256: approved.activation_approval_sha256,
        install_envelope_sha256: envelope.envelope_sha256,
        install_action_sha256: action.action_sha256,
        install_authorization_frame_sha256,
        promotion_transaction_sha256,
        program_persist_frame_sha256,
        generation: action.generation,
        sequence: action.log_sequence,
    };
    program_workspace::restore_persisted_program(
        approved.canonical_bytes,
        program_workspace::DurableSource {
            generation: installed.generation,
            canonical_program_byte_len: envelope.canonical_program_byte_len,
            canonical_program_sha256: installed.program_sha256,
            activation_approval_sha256: installed.activation_approval_sha256,
            install_envelope_sha256: installed.install_envelope_sha256,
            install_action_sha256: installed.install_action_sha256,
            install_authorization_frame_sha256: installed.install_authorization_frame_sha256,
            promotion_transaction_sha256: installed.promotion_transaction_sha256,
            program_persist_frame_sha256: installed.program_persist_frame_sha256,
        },
    )?;
    STATE.lock().installed = Some(installed);
    emit_install_commit_marker(installed);
    Ok(())
}

fn validate_install_inputs(
    envelope: &UiProgramInstallEnvelope,
    action: &ProjectInstallAction,
) -> Result<program_workspace::ApprovedProgramInstall, &'static str> {
    validate_ui_program_install_envelope(envelope).map_err(|error| error.reason())?;
    validate_install_action(action).map_err(|error| error.reason())?;
    let approved = program_workspace::approved_program_install()?;
    let program = Program::parse(&approved.canonical_bytes).map_err(|_| "program_malformed")?;
    if program.canonical_bytes() != approved.canonical_bytes
        || program.identity().sha256 != approved.canonical_program_sha256
        || sha256_bytes(&approved.canonical_bytes) != approved.canonical_program_sha256
        || approved.canonical_program_byte_len != approved.canonical_bytes.len() as u64
        || envelope.engine_service_id != PERSONAL_SHELL_SERVICE_ID
        || envelope.program_abi_version != PROGRAM_ABI_VERSION
        || envelope.canonical_program_sha256 != approved.canonical_program_sha256
        || envelope.canonical_program_byte_len != approved.canonical_program_byte_len
        || envelope.activation_approval_sha256 != approved.activation_approval_sha256
        || action.kind != ProjectInstallActionKind::Install
        || action.authority != ProjectInstallAuthority::PhysicalOwner
        || action.service_id != PERSONAL_SHELL_SERVICE_ID
        || action.generation != envelope.generation
        || action.install_envelope_sha256 != Some(envelope.envelope_sha256)
        || action.physical_approval_sha256.is_none()
        || action.authority_key_sha256
            != Some(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256)
    {
        return Err("ui_program_install_preview_stale");
    }
    let signature_message = install_action_signature_payload_sha256(action)
        .map_err(|error| error.reason())?;
    if !verify_promotion_authority_signature(&action.authority_signature, &signature_message) {
        return Err("project_install_physical_signature_invalid");
    }
    let controller = pci::find_mass_storage_controller().ok_or("ahci_controller_not_observed")?;
    let cursor = project_install_store::current_project_install_cursor(
        controller,
        PERSONAL_SHELL_SERVICE_ID,
    )?;
    let physical_approval_sha256 = agent_protocol_project_install::install_pointer_approval_hash(
        action.generation,
        action.log_sequence,
        envelope.envelope_sha256,
        action.previous_commit_sha256,
    );
    if action.generation != cursor.generation
        || action.log_sequence != cursor.next_install_commit_seq
        || action.previous_commit_sha256 != cursor.head_commit_sha256
        || action.physical_approval_sha256 != Some(physical_approval_sha256)
    {
        return Err("ui_program_install_preview_stale");
    }
    Ok(approved)
}

pub(crate) fn emit_install_ready_marker(approved: &program_workspace::ApprovedProgramApproval) {
    serial::write_raw_str(
        "PROGRAM_INSTALL_READY result=accepted physical_approval=genesis_pointer program_sha256=sha256:",
    );
    write_hash(approved.canonical_program_sha256);
    serial::write_raw_str(" activation_approval_sha256=sha256:");
    write_hash(approved.activation_approval_sha256);
    serial::write_raw_str(
        " engine=svc.user.shell persistence_authority=false reason=program_current_boot_approved\r\n",
    );
}

fn emit_install_commit_marker(installed: InstalledProgram) {
    serial::write_raw_str(
        "PROGRAM_INSTALL_COMMIT result=accepted physical_approval=genesis_pointer subject_kind=ui_program program_sha256=sha256:",
    );
    write_hash(installed.program_sha256);
    serial::write_raw_str(" activation_approval_sha256=sha256:");
    write_hash(installed.activation_approval_sha256);
    serial::write_raw_str(" install_envelope_sha256=sha256:");
    write_hash(installed.install_envelope_sha256);
    serial::write_raw_str(" install_action_sha256=sha256:");
    write_hash(installed.install_action_sha256);
    serial::write_raw_str(" promotion_transaction_sha256=sha256:");
    write_hash(installed.promotion_transaction_sha256);
    serial::write_raw_str(" program_persist_frame_sha256=sha256:");
    write_hash(installed.program_persist_frame_sha256);
    serial::write_raw_fmt(format_args!(
        " generation={} sequence={} engine=svc.user.shell guest_installed=false durable_writes=true reason=program_installed\r\n",
        installed.generation, installed.sequence,
    ));
}

// B1.3 later packet: fold exact RECLOG links and restore inert RUIP data.
pub(crate) fn resolve_installed_program() -> Result<(), &'static str> {
    Err("program_persistence_not_implemented")
}

// B1.3 later packet: called beside the existing autoloaders before input init.
pub(crate) fn run_boot_autoload() {
    let _ = resolve_installed_program();
    emit_autoload_marker();
}

fn emit_autoload_marker() {
    serial::write_raw_str("PROGRAM_AUTOLOAD result=denied phase=denied reason=program_persistence_not_implemented posture=");
    serial::write_raw_str(boot_control::current_boot_posture().as_str());
    serial::write_raw_str(" program_sha256=none promotion_transaction_sha256=none program_persist_frame_sha256=none w6_signature_verified=false canonical_verified=false workspace_reloaded=false shell_started=false cross_reboot_proven=false\r\n");
}

// B1.3 later packet: validate the exact active installed hash without writing.
pub(crate) fn rollback_preview(_program_sha256: [u8; 32]) -> Result<(), &'static str> {
    Err("program_rollback_not_implemented")
}

// B1.3 later packet: append the linked unpromote before removing durable workspace data.
pub(crate) fn rollback_apply(_program_sha256: [u8; 32]) -> Result<(), &'static str> {
    Err("program_rollback_not_implemented")
}

#[allow(dead_code)]
fn emit_rollback_commit_marker(
    program_sha256: [u8; 32],
    promotion_transaction_sha256: [u8; 32],
    unpromote_transaction_sha256: [u8; 32],
    workspace_removed: bool,
) {
    serial::write_raw_str("PROGRAM_ROLLBACK_COMMIT result=accepted program_sha256=sha256:");
    write_hash(program_sha256);
    serial::write_raw_str(" promotion_transaction_sha256=sha256:");
    write_hash(promotion_transaction_sha256);
    serial::write_raw_str(" unpromote_transaction_sha256=sha256:");
    write_hash(unpromote_transaction_sha256);
    serial::write_raw_str(" workspace_removed=");
    serial::write_raw_str(if workspace_removed { "true" } else { "false" });
    serial::write_raw_str(" durable_writes=true reason=program_unpromoted\r\n");
}

fn write_hash(hash: [u8; 32]) {
    for byte in hash {
        serial::write_raw_fmt(format_args!("{byte:02x}"));
    }
}
