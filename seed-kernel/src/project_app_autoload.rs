use raios_core::{
    boot_control::{BootPosture, BootSlotId},
    project_install::{
        validate_install_commit, ProjectInstallActionKind, ProjectInstallCommit,
        ProjectInstallEnvelope,
    },
    project_runtime::WORKSPACE_SERVICE_ID,
    sha256_bytes,
};
use spin::Mutex;

use crate::{
    agent_protocol::boot_control, pci, project_install_store,
    project_install_store::ProjectInstallReplay, serial, workspace_candidate_service,
};

const ATTEMPT_EVIDENCE_DOMAIN: &[u8] = b"raios.project_app_activation_attempt.v1";
const SUCCESS_EVIDENCE_DOMAIN: &[u8] = b"raios.project_app_activation_success.v1";
const ROLLBACK_EVIDENCE_DOMAIN: &[u8] = b"raios.project_app_recovery_failure.v1";

#[derive(Clone, Copy)]
struct State {
    checked: bool,
    phase: &'static str,
    reason: &'static str,
    boot_posture: &'static str,
    installed: bool,
    auto_start: bool,
    generation: u64,
    head_commit_sha256: Option<[u8; 32]>,
    probation_install_commit_sha256: Option<[u8; 32]>,
    active_install_commit_sha256: Option<[u8; 32]>,
    last_good_install_commit_sha256: Option<[u8; 32]>,
    candidate_sha256: Option<[u8; 32]>,
    receipt_sha256: Option<[u8; 32]>,
    activation_attempt_persisted: bool,
    activation_success_persisted: bool,
    rollback_persisted: bool,
    fallback_running: bool,
    install_tombstone_written: bool,
}

impl State {
    const fn new() -> Self {
        Self {
            checked: false,
            phase: "not_checked",
            reason: "project_app_autoload_not_checked",
            boot_posture: "unknown",
            installed: false,
            auto_start: false,
            generation: 0,
            head_commit_sha256: None,
            probation_install_commit_sha256: None,
            active_install_commit_sha256: None,
            last_good_install_commit_sha256: None,
            candidate_sha256: None,
            receipt_sha256: None,
            activation_attempt_persisted: false,
            activation_success_persisted: false,
            rollback_persisted: false,
            fallback_running: false,
            install_tombstone_written: false,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Snapshot {
    pub(crate) checked: bool,
    pub(crate) phase: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) boot_posture: &'static str,
    pub(crate) installed: bool,
    pub(crate) auto_start: bool,
    pub(crate) generation: u64,
    pub(crate) head_commit_sha256: Option<[u8; 32]>,
    pub(crate) probation_install_commit_sha256: Option<[u8; 32]>,
    pub(crate) active_install_commit_sha256: Option<[u8; 32]>,
    pub(crate) last_good_install_commit_sha256: Option<[u8; 32]>,
    pub(crate) candidate_sha256: Option<[u8; 32]>,
    pub(crate) receipt_sha256: Option<[u8; 32]>,
    pub(crate) activation_attempt_persisted: bool,
    pub(crate) activation_success_persisted: bool,
    pub(crate) rollback_persisted: bool,
    pub(crate) fallback_running: bool,
    pub(crate) install_tombstone_written: bool,
}

static STATE: Mutex<State> = Mutex::new(State::new());

pub(crate) fn snapshot() -> Snapshot {
    let state = *STATE.lock();
    Snapshot {
        checked: state.checked,
        phase: state.phase,
        reason: state.reason,
        boot_posture: state.boot_posture,
        installed: state.installed,
        auto_start: state.auto_start,
        generation: state.generation,
        head_commit_sha256: state.head_commit_sha256,
        probation_install_commit_sha256: state.probation_install_commit_sha256,
        active_install_commit_sha256: state.active_install_commit_sha256,
        last_good_install_commit_sha256: state.last_good_install_commit_sha256,
        candidate_sha256: state.candidate_sha256,
        receipt_sha256: state.receipt_sha256,
        activation_attempt_persisted: state.activation_attempt_persisted,
        activation_success_persisted: state.activation_success_persisted,
        rollback_persisted: state.rollback_persisted,
        fallback_running: state.fallback_running,
        install_tombstone_written: state.install_tombstone_written,
    }
}

pub(crate) fn run_boot_autoload() {
    let (posture, _, authoritative) = boot_control::current_boot_last_good_view();
    reset_for_posture(posture);
    if posture != BootPosture::Normal {
        finish("denied", "project_app_autoload_requires_normal_boot");
        return;
    }
    if !authoritative.is_some_and(|record| {
        record.boot_attempt.success_marked && record.pending == BootSlotId::None
    }) {
        finish("denied", "project_app_autoload_requires_boot_success_mark");
        return;
    }
    let Some(controller) = pci::find_mass_storage_controller() else {
        finish("denied", "project_app_autoload_storage_unavailable");
        return;
    };
    let replay =
        match project_install_store::replay_project_install(controller, WORKSPACE_SERVICE_ID) {
            Ok(replay) => replay,
            Err(reason) => {
                finish("denied", reason);
                return;
            }
        };
    project_replay(&replay);
    if replay.resolved.uninstall_tombstoned {
        finish("not_installed", "project_app_uninstall_tombstone_resolved");
        return;
    }
    if !replay.resolved.installed {
        finish("not_installed", "project_app_autoload_no_install");
        return;
    }

    if let Some(probation) = replay.resolved.probation_install_commit_sha256 {
        run_probation(controller, replay, probation);
        return;
    }
    let Some(active) = replay.resolved.active_install_commit_sha256 else {
        finish("denied", "project_app_active_install_missing");
        return;
    };
    run_existing_active(controller, &replay, active, false);
}

fn run_probation(
    controller: pci::PciMassStorageController,
    replay: ProjectInstallReplay,
    probation: [u8; 32],
) {
    let Some(envelope) = envelope_for(&replay, probation) else {
        finish("denied", "project_install_envelope_missing");
        return;
    };
    note_envelope(envelope);
    if !envelope.auto_start {
        finish("disabled", "project_install_autostart_disabled");
        return;
    }
    if replay.resolved.probation_activation_attempted {
        rollback_and_restore(
            controller,
            replay,
            "project_app_probation_previous_boot_incomplete",
        );
        return;
    }
    let attempt_evidence = evidence_hash(
        &replay,
        Some(probation),
        ATTEMPT_EVIDENCE_DOMAIN,
        "activation_attempt",
    );
    if let Err(reason) =
        project_install_store::append_activation_attempt(controller, &replay, attempt_evidence)
    {
        finish("denied", reason);
        return;
    }
    STATE.lock().activation_attempt_persisted = true;
    let replay =
        match project_install_store::replay_project_install(controller, WORKSPACE_SERVICE_ID) {
            Ok(replay) => replay,
            Err(reason) => {
                finish("denied", reason);
                return;
            }
        };
    project_replay(&replay);
    match activate_target(
        controller,
        &replay,
        probation,
        "project_install_probation_autostart",
    ) {
        Ok(()) => {}
        Err(reason) => {
            rollback_and_restore(controller, replay, reason);
            return;
        }
    }
    let success_evidence = evidence_hash(
        &replay,
        Some(probation),
        SUCCESS_EVIDENCE_DOMAIN,
        "activation_success",
    );
    match project_install_store::append_activation_success(controller, &replay, success_evidence) {
        Ok(_) => {
            let replay = match project_install_store::replay_project_install(
                controller,
                WORKSPACE_SERVICE_ID,
            ) {
                Ok(replay) => replay,
                Err(reason) => {
                    controlled_drop("project_app_activation_success_replay_failed");
                    finish("denied", reason);
                    return;
                }
            };
            project_replay(&replay);
            let mut state = STATE.lock();
            state.activation_success_persisted = true;
            state.phase = "running";
            state.reason = "project_app_activation_success_persisted";
            drop(state);
            emit_marker("accepted");
        }
        Err(reason) => {
            controlled_drop("project_app_activation_success_not_persisted");
            finish("denied", reason);
        }
    }
}

fn run_existing_active(
    controller: pci::PciMassStorageController,
    replay: &ProjectInstallReplay,
    target: [u8; 32],
    fallback: bool,
) {
    let Some(envelope) = envelope_for(replay, target) else {
        finish("denied", "project_install_envelope_missing");
        return;
    };
    note_envelope(envelope);
    if !envelope.auto_start {
        finish("disabled", "project_install_autostart_disabled");
        return;
    }
    let run_reason = if fallback {
        "project_install_last_good_autostart"
    } else {
        "project_install_active_autostart"
    };
    match activate_target(controller, replay, target, run_reason) {
        Ok(()) => {
            let mut state = STATE.lock();
            state.phase = if fallback {
                "last_good_running"
            } else {
                "running"
            };
            state.reason = if fallback {
                "project_app_last_good_restored"
            } else {
                "project_app_active_autostarted"
            };
            state.fallback_running = fallback;
            drop(state);
            emit_marker("accepted");
        }
        Err(reason) => finish("denied", reason),
    }
}

fn activate_target(
    controller: pci::PciMassStorageController,
    replay: &ProjectInstallReplay,
    target: [u8; 32],
    success_reason: &'static str,
) -> Result<(), &'static str> {
    let payloads =
        project_install_store::read_project_install_payloads(controller, replay, target)?;
    note_envelope(&payloads.envelope);
    let outcome = workspace_candidate_service::activate_installed(
        &payloads.envelope,
        &payloads.encoded_receipt,
        payloads.candidate,
        success_reason,
    );
    if outcome.accepted {
        Ok(())
    } else {
        Err(outcome.reason)
    }
}

fn rollback_and_restore(
    controller: pci::PciMassStorageController,
    replay: ProjectInstallReplay,
    failure_reason: &'static str,
) {
    emit_probation_failure(failure_reason, &replay);
    let rollback_evidence = evidence_hash(
        &replay,
        replay.resolved.last_good_install_commit_sha256,
        ROLLBACK_EVIDENCE_DOMAIN,
        failure_reason,
    );
    controlled_drop("project_app_probation_failed");
    if let Err(reason) = project_install_store::append_recovery_failure_rollback(
        controller,
        &replay,
        rollback_evidence,
    ) {
        finish("rollback_failed", reason);
        return;
    }
    STATE.lock().rollback_persisted = true;
    let replay =
        match project_install_store::replay_project_install(controller, WORKSPACE_SERVICE_ID) {
            Ok(replay) => replay,
            Err(reason) => {
                finish("rollback_failed", reason);
                return;
            }
        };
    project_replay(&replay);
    let Some(last_good) = replay.resolved.active_install_commit_sha256 else {
        finish("rolled_back_empty", "project_app_no_last_good_available");
        return;
    };
    run_existing_active(controller, &replay, last_good, true);
}

fn emit_probation_failure(reason: &'static str, replay: &ProjectInstallReplay) {
    serial::write_raw_fmt(format_args!(
        "PROJECT_APP_PROBATION_FAILURE reason={} recovery_action=rollback generation={}",
        reason, replay.resolved.generation
    ));
    if let Some(hash) = replay.resolved.probation_install_commit_sha256 {
        serial::write_raw_str(" probation_install_commit_sha256=sha256:");
        write_hash(hash);
    }
    if let Some(hash) = replay.resolved.last_good_install_commit_sha256 {
        serial::write_raw_str(" last_good_install_commit_sha256=sha256:");
        write_hash(hash);
    }
    serial::write_raw_str("\r\n");
}

fn evidence_hash(
    replay: &ProjectInstallReplay,
    target: Option<[u8; 32]>,
    evidence_domain: &[u8],
    evidence_reason: &str,
) -> [u8; 32] {
    let mut evidence = alloc::vec::Vec::from(evidence_domain);
    evidence.extend_from_slice(&replay.resolved.generation.to_le_bytes());
    evidence.extend_from_slice(&replay.resolved.head_commit_sha256.unwrap_or([0; 32]));
    evidence.extend_from_slice(&target.unwrap_or([0; 32]));
    evidence.extend_from_slice(&replay.resolved.candidate_sha256.unwrap_or([0; 32]));
    evidence.extend_from_slice(evidence_reason.as_bytes());
    let runtime = workspace_candidate_service::snapshot();
    evidence.extend_from_slice(
        &runtime
            .binding
            .map(|binding| binding.candidate_sha256)
            .unwrap_or([0; 32]),
    );
    evidence.extend_from_slice(
        &runtime
            .binding
            .map(|binding| binding.receipt_sha256)
            .unwrap_or([0; 32]),
    );
    evidence.extend_from_slice(&runtime.run_count.to_le_bytes());
    evidence.extend_from_slice(&runtime.last_fuel_used.to_le_bytes());
    evidence.extend_from_slice(
        &runtime
            .last_return_value
            .map(|value| value as u32)
            .unwrap_or(u32::MAX)
            .to_le_bytes(),
    );
    evidence.extend_from_slice(runtime.last_run_outcome.as_bytes());
    evidence.extend_from_slice(runtime.activation_authority.as_bytes());
    sha256_bytes(&evidence)
}

fn envelope_for(
    replay: &ProjectInstallReplay,
    install_commit_sha256: [u8; 32],
) -> Option<&ProjectInstallEnvelope> {
    replay
        .commits
        .iter()
        .find(|commit| commit.commit_sha256 == install_commit_sha256)
        .and_then(|commit| commit.envelope.as_ref())
}

fn reset_for_posture(posture: BootPosture) {
    let mut state = STATE.lock();
    *state = State::new();
    state.checked = true;
    state.boot_posture = posture.as_str();
}

fn project_replay(replay: &ProjectInstallReplay) {
    let mut state = STATE.lock();
    state.installed = replay.resolved.installed;
    state.generation = replay.resolved.generation;
    state.head_commit_sha256 = replay.resolved.head_commit_sha256;
    state.probation_install_commit_sha256 = replay.resolved.probation_install_commit_sha256;
    state.active_install_commit_sha256 = replay.resolved.active_install_commit_sha256;
    state.last_good_install_commit_sha256 = replay.resolved.last_good_install_commit_sha256;
    state.candidate_sha256 = replay.resolved.candidate_sha256;
    state.install_tombstone_written = replay.resolved.uninstall_tombstoned;
}

fn note_envelope(envelope: &ProjectInstallEnvelope) {
    let mut state = STATE.lock();
    state.auto_start = envelope.auto_start;
    state.candidate_sha256 = Some(envelope.candidate_sha256);
    state.receipt_sha256 = Some(envelope.build_receipt_sha256);
}

fn controlled_drop(reason: &'static str) {
    if workspace_candidate_service::visible_in_inventory() {
        let _ = workspace_candidate_service::drop_service(reason);
    }
}

pub(crate) fn note_runtime_drop_without_tombstone() {
    let mut state = STATE.lock();
    if !state.installed {
        return;
    }
    state.phase = "stopped_current_boot";
    state.reason = "project_app_runtime_drop_without_install_tombstone";
    state.install_tombstone_written = false;
    drop(state);
    emit_marker("accepted");
}

pub(crate) fn note_physical_uninstall_tombstone(
    commit: &ProjectInstallCommit,
) -> Result<(), &'static str> {
    validate_install_commit(commit).map_err(|error| error.reason())?;
    if commit.action.kind != ProjectInstallActionKind::Uninstall
        || commit.action.service_id != WORKSPACE_SERVICE_ID
    {
        return Err("project_app_uninstall_commit_mismatch");
    }
    let posture = STATE.lock().boot_posture;
    let mut state = State::new();
    state.checked = true;
    state.phase = "not_installed";
    state.reason = "project_app_uninstall_tombstone_persisted";
    state.boot_posture = posture;
    state.generation = commit.action.generation;
    state.head_commit_sha256 = Some(commit.commit_sha256);
    state.install_tombstone_written = true;
    *STATE.lock() = state;
    emit_marker("accepted");
    Ok(())
}

fn finish(phase: &'static str, reason: &'static str) {
    let mut state = STATE.lock();
    state.phase = phase;
    state.reason = reason;
    drop(state);
    emit_marker(if phase == "denied" || phase == "rollback_failed" {
        "denied"
    } else {
        "accepted"
    });
}

fn emit_marker(result: &'static str) {
    let state = *STATE.lock();
    serial::write_raw_fmt(format_args!(
        "PROJECT_APP_AUTOLOAD result={} phase={} reason={} posture={} generation={} installed={} auto_start={} activation_attempt_persisted={} activation_success_persisted={} rollback_persisted={} fallback_running={} install_tombstone_written={}",
        result,
        state.phase,
        state.reason,
        state.boot_posture,
        state.generation,
        state.installed,
        state.auto_start,
        state.activation_attempt_persisted,
        state.activation_success_persisted,
        state.rollback_persisted,
        state.fallback_running,
        state.install_tombstone_written,
    ));
    if let Some(hash) = state.candidate_sha256 {
        serial::write_raw_str(" candidate_sha256=sha256:");
        write_hash(hash);
    }
    if let Some(hash) = state.head_commit_sha256 {
        serial::write_raw_str(" head_commit_sha256=sha256:");
        write_hash(hash);
    }
    serial::write_raw_str("\r\n");
}

fn write_hash(hash: [u8; 32]) {
    for byte in hash {
        serial::write_raw_fmt(format_args!("{byte:02x}"));
    }
}
