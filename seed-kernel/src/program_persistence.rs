use alloc::vec::Vec;
use core::str;

use raios_core::{
    boot_control::BootPosture,
    durable_record_frame::{
        parse_reclog_frame, scan_reclog, RECLOG_FRAME_HEADER_LEN,
    },
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
    event_log, pci, personal_shell_service, program_workspace, project_install_store, serial,
};

#[derive(Clone, Copy)]
struct InstalledProgram {
    program_sha256: [u8; 32],
    canonical_program_byte_len: u64,
    activation_approval_sha256: [u8; 32],
    install_envelope_sha256: [u8; 32],
    install_action_sha256: [u8; 32],
    install_authorization_frame_sha256: [u8; 32],
    promotion_transaction_sha256: [u8; 32],
    program_persist_frame_sha256: [u8; 32],
    generation: u64,
    sequence: u64,
}

#[derive(Clone, Copy)]
struct AuthorizationFrame {
    authorization: durable_store::UiProgramInstallAuthorization,
    frame_sha256: [u8; 32],
}

#[derive(Clone, Copy)]
pub(crate) struct ActiveInstalledProgram {
    installed: InstalledProgram,
    authorization: durable_store::UiProgramInstallAuthorization,
    promotion: durable_store::UiProgramPromotionTransactionRecord,
    artifact: artifact_store::UiProgramPersistRecord,
}

#[derive(Clone, Copy)]
pub(crate) enum InstalledProgramResolution {
    None,
    Active(ActiveInstalledProgram),
    RolledBack {
        program_sha256: [u8; 32],
        unpromote_transaction_sha256: [u8; 32],
    },
}

#[derive(Clone, Copy)]
enum InstalledProgramFold {
    None,
    Incomplete,
    Promote {
        authorization: durable_store::UiProgramInstallAuthorization,
        promotion: durable_store::UiProgramPromotionTransactionRecord,
        promotion_frame_sha256: [u8; 32],
        artifact: Option<artifact_store::UiProgramPersistRecord>,
    },
    RolledBack {
        program_sha256: [u8; 32],
        unpromote_transaction_sha256: [u8; 32],
    },
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
        canonical_program_byte_len: approved.canonical_program_byte_len,
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

pub(crate) fn resolve_installed_program(bytes: &[u8]) -> InstalledProgramResolution {
    let scan = scan_reclog(bytes);
    if !scan.valid_prefix_chain || !scan.full_region_valid {
        return InstalledProgramResolution::None;
    }
    let artifacts = artifact_store::ui_program_persist_records_from_reclog(bytes);
    let mut authorizations = Vec::new();
    let mut fold = InstalledProgramFold::None;
    let mut offset = 0usize;
    let mut seq = 1u64;
    let mut prev = [0u8; 32];
    let mut count = 0u64;
    while count < scan.count && offset < bytes.len() {
        let Ok(frame) = parse_reclog_frame(bytes, offset, seq, prev) else {
            return InstalledProgramResolution::None;
        };
        let payload_start = offset + RECLOG_FRAME_HEADER_LEN;
        let payload_end = payload_start + frame.payload_len as usize;
        let Ok(payload) = str::from_utf8(&bytes[payload_start..payload_end]) else {
            return InstalledProgramResolution::None;
        };
        let ui_program_record = payload.contains("\"subject_kind\": \"ui_program\"")
            || payload.contains(".ui_program.v0\"");
        if ui_program_record && payload.contains("\"schema\": \"raios.install_authorization.v0\"") {
            fold = InstalledProgramFold::Incomplete;
            if let Ok(authorization) =
                durable_store::parse_ui_program_install_authorization_payload(payload)
            {
                authorizations.push(AuthorizationFrame {
                    authorization,
                    frame_sha256: frame.frame_sha256,
                });
            }
        } else if ui_program_record
            && payload.contains("\"schema\": \"raios.promotion_transaction.v0\"")
        {
            fold = match durable_store::parse_ui_program_promotion_transaction_payload(payload) {
                Ok(promotion) => {
                    let authorization = authorizations
                        .iter()
                        .find(|record| {
                            record.frame_sha256 == promotion.install_authorization_frame_sha256
                        })
                        .copied();
                    match (promotion.transaction_kind, authorization, fold) {
                        (
                            durable_store::PromotionTransactionKind::Promote,
                            Some(authorization),
                            _,
                        ) if exact_authorization_link(&authorization, &promotion) => {
                            InstalledProgramFold::Promote {
                                authorization: authorization.authorization,
                                promotion,
                                promotion_frame_sha256: frame.frame_sha256,
                                artifact: None,
                            }
                        }
                        (
                            durable_store::PromotionTransactionKind::Unpromote,
                            Some(authorization),
                            InstalledProgramFold::Promote {
                                authorization: active_authorization,
                                promotion: active_promotion,
                                promotion_frame_sha256,
                                artifact: Some(artifact),
                            },
                        ) if exact_authorization_link(&authorization, &promotion)
                            && exact_unpromote_link(
                                &active_authorization,
                                &active_promotion,
                                &promotion,
                            )
                            && artifact.promotion_transaction_sha256
                                == promotion_frame_sha256 =>
                        {
                            InstalledProgramFold::RolledBack {
                                program_sha256: promotion.canonical_program_sha256,
                                unpromote_transaction_sha256: frame.frame_sha256,
                            }
                        }
                        _ => InstalledProgramFold::Incomplete,
                    }
                }
                Err(_) => InstalledProgramFold::Incomplete,
            };
        } else if ui_program_record
            && payload.contains("\"schema\": \"raios.artifact_persist.v0\"")
        {
            let artifact = artifacts.iter().find(|record| record.seq == frame.seq).copied();
            fold = match (fold, artifact) {
                (
                    InstalledProgramFold::Promote {
                        authorization,
                        promotion,
                        promotion_frame_sha256,
                        artifact: None,
                    },
                    Some(artifact),
                ) if exact_artifact_link(
                    &authorization,
                    &promotion,
                    promotion_frame_sha256,
                    &artifact,
                ) => InstalledProgramFold::Promote {
                    authorization,
                    promotion,
                    promotion_frame_sha256,
                    artifact: Some(artifact),
                },
                _ => InstalledProgramFold::Incomplete,
            };
        }
        offset += frame.frame_len as usize;
        seq = seq.saturating_add(1);
        prev = frame.frame_sha256;
        count += 1;
    }
    match fold {
        InstalledProgramFold::Promote {
            authorization,
            promotion,
            promotion_frame_sha256,
            artifact: Some(artifact),
        } => InstalledProgramResolution::Active(ActiveInstalledProgram {
            installed: installed_from_records(
                &authorization,
                promotion.install_authorization_frame_sha256,
                promotion_frame_sha256,
                artifact.frame_sha256,
            ),
            authorization,
            promotion,
            artifact,
        }),
        InstalledProgramFold::RolledBack {
            program_sha256,
            unpromote_transaction_sha256,
        } => InstalledProgramResolution::RolledBack {
            program_sha256,
            unpromote_transaction_sha256,
        },
        InstalledProgramFold::None
        | InstalledProgramFold::Incomplete
        | InstalledProgramFold::Promote { artifact: None, .. } => {
            InstalledProgramResolution::None
        }
    }
}

fn exact_authorization_link(
    authorization: &AuthorizationFrame,
    promotion: &durable_store::UiProgramPromotionTransactionRecord,
) -> bool {
    authorization.frame_sha256 == promotion.install_authorization_frame_sha256
        && authorization.authorization.canonical_program_sha256
            == promotion.canonical_program_sha256
        && authorization.authorization.canonical_program_byte_len
            == promotion.canonical_program_byte_len
        && authorization.authorization.activation_approval_sha256
            == promotion.activation_approval_sha256
        && authorization.authorization.generation == promotion.generation
}

fn exact_artifact_link(
    authorization: &durable_store::UiProgramInstallAuthorization,
    promotion: &durable_store::UiProgramPromotionTransactionRecord,
    promotion_frame_sha256: [u8; 32],
    artifact: &artifact_store::UiProgramPersistRecord,
) -> bool {
    artifact.promotion_transaction_sha256 == promotion_frame_sha256
        && artifact.install_authorization_frame_sha256
            == promotion.install_authorization_frame_sha256
        && artifact.canonical_program_sha256 == promotion.canonical_program_sha256
        && artifact.canonical_program_byte_len == promotion.canonical_program_byte_len
        && artifact.activation_approval_sha256 == promotion.activation_approval_sha256
        && artifact.install_envelope_sha256 == authorization.install_envelope_sha256
}

fn exact_unpromote_link(
    authorization: &durable_store::UiProgramInstallAuthorization,
    active: &durable_store::UiProgramPromotionTransactionRecord,
    unpromote: &durable_store::UiProgramPromotionTransactionRecord,
) -> bool {
    authorization.canonical_program_sha256 == unpromote.canonical_program_sha256
        && active.canonical_program_sha256 == unpromote.canonical_program_sha256
        && active.canonical_program_byte_len == unpromote.canonical_program_byte_len
        && active.activation_approval_sha256 == unpromote.activation_approval_sha256
        && active.install_authorization_frame_sha256
            == unpromote.install_authorization_frame_sha256
        && active.generation == unpromote.generation
}

fn installed_from_records(
    authorization: &durable_store::UiProgramInstallAuthorization,
    install_authorization_frame_sha256: [u8; 32],
    promotion_transaction_sha256: [u8; 32],
    program_persist_frame_sha256: [u8; 32],
) -> InstalledProgram {
    InstalledProgram {
        program_sha256: authorization.canonical_program_sha256,
        canonical_program_byte_len: authorization.canonical_program_byte_len,
        activation_approval_sha256: authorization.activation_approval_sha256,
        install_envelope_sha256: authorization.install_envelope_sha256,
        install_action_sha256: authorization.install_action_sha256,
        install_authorization_frame_sha256,
        promotion_transaction_sha256,
        program_persist_frame_sha256,
        generation: authorization.generation,
        sequence: authorization.log_sequence,
    }
}

pub(crate) fn run_boot_autoload() {
    let (posture, _, authoritative) = boot_control::current_boot_last_good_view();
    if posture != BootPosture::Normal {
        emit_autoload_marker(
            "denied",
            "denied",
            "program_autoload_posture_not_normal",
            posture,
            None,
            None,
            None,
            false,
            false,
            false,
            false,
        );
        return;
    }
    if !authoritative.is_some_and(|record| record.boot_attempt.success_marked) {
        emit_autoload_marker(
            "denied",
            "denied",
            "program_autoload_requires_boot_success_mark",
            posture,
            None,
            None,
            None,
            false,
            false,
            false,
            false,
        );
        return;
    }
    let Some(controller) = pci::find_mass_storage_controller() else {
        emit_autoload_marker(
            "denied",
            "denied",
            "ahci_controller_not_observed",
            posture,
            None,
            None,
            None,
            false,
            false,
            false,
            false,
        );
        return;
    };
    let scan = artifact_store::current_boot_reclog_scan(controller);
    let Some(bytes) = scan.bytes.as_deref() else {
        emit_autoload_marker(
            "denied",
            "denied",
            scan.reason,
            posture,
            None,
            None,
            None,
            false,
            false,
            false,
            false,
        );
        return;
    };
    let active = match resolve_installed_program(bytes) {
        InstalledProgramResolution::None => {
            emit_autoload_marker(
                "accepted",
                "not_installed",
                "no_w6_authorized_program_install",
                posture,
                None,
                None,
                None,
                false,
                false,
                false,
                false,
            );
            return;
        }
        InstalledProgramResolution::RolledBack {
            program_sha256,
            unpromote_transaction_sha256,
        } => {
            emit_autoload_marker(
                "accepted",
                "rolled_back",
                "ui_program_install_rolled_back",
                posture,
                Some(program_sha256),
                Some(unpromote_transaction_sha256),
                None,
                false,
                false,
                false,
                false,
            );
            return;
        }
        InstalledProgramResolution::Active(active) => active,
    };
    STATE.lock().installed = Some(active.installed);
    let verified = match artifact_store::read_verified_artstor_payload(controller, &active.artifact)
    {
        Ok(verified) => verified,
        Err(reason) => {
            emit_autoload_marker(
                "denied",
                "denied",
                reason,
                posture,
                Some(active.installed.program_sha256),
                Some(active.installed.promotion_transaction_sha256),
                Some(active.installed.program_persist_frame_sha256),
                true,
                false,
                false,
                false,
            );
            return;
        }
    };
    let restored = program_workspace::restore_persisted_program(
        verified.payload,
        program_workspace::DurableSource {
            generation: active.installed.generation,
            canonical_program_byte_len: active.installed.canonical_program_byte_len,
            canonical_program_sha256: active.installed.program_sha256,
            activation_approval_sha256: active.installed.activation_approval_sha256,
            install_envelope_sha256: active.installed.install_envelope_sha256,
            install_action_sha256: active.installed.install_action_sha256,
            install_authorization_frame_sha256: active
                .promotion
                .install_authorization_frame_sha256,
            promotion_transaction_sha256: active.installed.promotion_transaction_sha256,
            program_persist_frame_sha256: active.installed.program_persist_frame_sha256,
        },
    );
    if let Err(reason) = restored {
        emit_autoload_marker(
            "denied",
            "denied",
            reason,
            posture,
            Some(active.installed.program_sha256),
            Some(active.installed.promotion_transaction_sha256),
            Some(active.installed.program_persist_frame_sha256),
            true,
            false,
            false,
            false,
        );
        return;
    }
    emit_autoload_marker(
        "accepted",
        "autoloaded",
        "program_autoloaded",
        posture,
        Some(active.installed.program_sha256),
        Some(active.installed.promotion_transaction_sha256),
        Some(active.installed.program_persist_frame_sha256),
        true,
        true,
        true,
        true,
    );
}

#[allow(clippy::too_many_arguments)]
fn emit_autoload_marker(
    result: &'static str,
    phase: &'static str,
    reason: &'static str,
    posture: BootPosture,
    program_sha256: Option<[u8; 32]>,
    promotion_transaction_sha256: Option<[u8; 32]>,
    program_persist_frame_sha256: Option<[u8; 32]>,
    w6_signature_verified: bool,
    canonical_verified: bool,
    workspace_reloaded: bool,
    cross_reboot_proven: bool,
) {
    serial::write_raw_str("PROGRAM_AUTOLOAD result=");
    serial::write_raw_str(result);
    serial::write_raw_str(" phase=");
    serial::write_raw_str(phase);
    serial::write_raw_str(" reason=");
    serial::write_raw_str(reason);
    serial::write_raw_str(" posture=");
    serial::write_raw_str(posture_str(posture));
    serial::write_raw_str(" program_sha256=");
    write_optional_hash(program_sha256);
    serial::write_raw_str(" promotion_transaction_sha256=");
    write_optional_hash(promotion_transaction_sha256);
    serial::write_raw_str(" program_persist_frame_sha256=");
    write_optional_hash(program_persist_frame_sha256);
    serial::write_raw_str(" w6_signature_verified=");
    write_bool(w6_signature_verified);
    serial::write_raw_str(" canonical_verified=");
    write_bool(canonical_verified);
    serial::write_raw_str(" workspace_reloaded=");
    write_bool(workspace_reloaded);
    serial::write_raw_str(" shell_started=false cross_reboot_proven=");
    write_bool(cross_reboot_proven);
    serial::write_raw_str("\r\n");
}

pub(crate) fn rollback_preview(program_sha256: [u8; 32]) -> Result<(), &'static str> {
    match current_installed_program()? {
        InstalledProgramResolution::Active(active)
            if active.installed.program_sha256 == program_sha256 => Ok(()),
        InstalledProgramResolution::Active(_) => Err("ui_program_rollback_target_mismatch"),
        InstalledProgramResolution::RolledBack { .. } => Err("ui_program_already_rolled_back"),
        InstalledProgramResolution::None => Err("ui_program_not_installed"),
    }
}

pub(crate) fn rollback_apply(
    program_sha256: [u8; 32],
    event_id: event_log::EventId,
) -> Result<(), &'static str> {
    let active = match current_installed_program()? {
        InstalledProgramResolution::Active(active)
            if active.installed.program_sha256 == program_sha256 => active,
        InstalledProgramResolution::Active(_) => {
            return Err("ui_program_rollback_target_mismatch")
        }
        InstalledProgramResolution::RolledBack { .. } => {
            return Err("ui_program_already_rolled_back")
        }
        InstalledProgramResolution::None => return Err("ui_program_not_installed"),
    };
    if personal_shell_service::lifecycle_snapshot().running {
        return Err("ui_program_rollback_requires_shell_stopped");
    }
    durable_store::validate_ui_program_install_authorization(&active.authorization)?;
    let unpromote = durable_store::UiProgramPromotionTransactionRecord {
        subject: durable_store::PromotionSubject::UiProgram,
        transaction_kind: durable_store::PromotionTransactionKind::Unpromote,
        engine_service_id: PERSONAL_SHELL_SERVICE_ID,
        program_abi_version: PROGRAM_ABI_VERSION,
        canonical_program_sha256: active.promotion.canonical_program_sha256,
        canonical_program_byte_len: active.promotion.canonical_program_byte_len,
        activation_approval_sha256: active.promotion.activation_approval_sha256,
        install_authorization_present: true,
        install_envelope_binds_activation: true,
        install_action_signature_verified: true,
        physical_install_approval_consumed: true,
        install_authorization_frame_sha256: active
            .promotion
            .install_authorization_frame_sha256,
        canonical_verified: true,
        activation_approval_consumed: true,
        generation: active.promotion.generation,
        rollback_apply_event_id: Some(event_id),
    };
    let appended = durable_store::append_promotion_transaction(&unpromote, true);
    if !appended.performed {
        return Err(appended.reason);
    }
    let unpromote_transaction_sha256 = appended
        .frame_sha256
        .ok_or("promotion_transaction_readback_missing")?;
    let workspace_removed = program_workspace::remove_restored_program(program_sha256);
    STATE.lock().installed = None;
    emit_rollback_commit_marker(
        program_sha256,
        active.installed.promotion_transaction_sha256,
        unpromote_transaction_sha256,
        workspace_removed,
    );
    Ok(())
}

fn current_installed_program() -> Result<InstalledProgramResolution, &'static str> {
    let controller = pci::find_mass_storage_controller().ok_or("ahci_controller_not_observed")?;
    let scan = artifact_store::current_boot_reclog_scan(controller);
    let bytes = scan.bytes.as_deref().ok_or(scan.reason)?;
    Ok(resolve_installed_program(bytes))
}

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

fn posture_str(posture: BootPosture) -> &'static str {
    match posture {
        BootPosture::Normal => "Normal",
        BootPosture::Probation => "Probation",
        BootPosture::Safe => "Safe",
        BootPosture::PersistenceUnavailable => "PersistenceUnavailable",
    }
}

fn write_optional_hash(hash: Option<[u8; 32]>) {
    if let Some(hash) = hash {
        serial::write_raw_str("sha256:");
        write_hash(hash);
    } else {
        serial::write_raw_str("none");
    }
}

fn write_bool(value: bool) {
    serial::write_raw_str(if value { "true" } else { "false" });
}

fn write_hash(hash: [u8; 32]) {
    for byte in hash {
        serial::write_raw_fmt(format_args!("{byte:02x}"));
    }
}
