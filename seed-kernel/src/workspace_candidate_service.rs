use raios_core::project_runtime::{
    build_workspace_pointer_approval, build_workspace_run_binding, WorkspacePhysicalApproval,
    WorkspaceRunBinding, WORKSPACE_ENTRYPOINT, WORKSPACE_FUEL_BUDGET, WORKSPACE_INSTANCE_LIMIT,
    WORKSPACE_MEMORY_COUNT_LIMIT, WORKSPACE_MEMORY_LIMIT_BYTES, WORKSPACE_SERVICE_ID,
    WORKSPACE_TABLE_LIMIT, WORKSPACE_TRUST_TIER,
};
use spin::Mutex;

use crate::{console, module_candidate_intake, project_build, serial, wasm_runtime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Missing,
    Pending,
    Running,
    Stopped,
    Crashed,
}

impl Phase {
    const fn label(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Pending => "pending_physical_approval",
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Crashed => "crashed",
        }
    }
}

struct State {
    phase: Phase,
    binding: Option<WorkspaceRunBinding>,
    approval: Option<WorkspacePhysicalApproval>,
    generation: u64,
    run_count: u64,
    last_reason: &'static str,
    last_run_outcome: &'static str,
    last_return_value: Option<i32>,
    last_fuel_used: u64,
}

impl State {
    const fn new() -> Self {
        Self {
            phase: Phase::Missing,
            binding: None,
            approval: None,
            generation: 0,
            run_count: 0,
            last_reason: "not_prepared",
            last_run_outcome: "not_run",
            last_return_value: None,
            last_fuel_used: 0,
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            phase: self.phase.label(),
            binding: self.binding,
            approval: self.approval,
            generation: self.generation,
            run_count: self.run_count,
            last_reason: self.last_reason,
            last_run_outcome: self.last_run_outcome,
            last_return_value: self.last_return_value,
            last_fuel_used: self.last_fuel_used,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Snapshot {
    pub(crate) phase: &'static str,
    pub(crate) binding: Option<WorkspaceRunBinding>,
    pub(crate) approval: Option<WorkspacePhysicalApproval>,
    pub(crate) generation: u64,
    pub(crate) run_count: u64,
    pub(crate) last_reason: &'static str,
    pub(crate) last_run_outcome: &'static str,
    pub(crate) last_return_value: Option<i32>,
    pub(crate) last_fuel_used: u64,
}

pub(crate) struct Outcome {
    pub(crate) accepted: bool,
    pub(crate) reason: &'static str,
    pub(crate) snapshot: Snapshot,
}

static STATE: Mutex<State> = Mutex::new(State::new());

pub(crate) fn snapshot() -> Snapshot {
    STATE.lock().snapshot()
}

pub(crate) fn pending_approval() -> bool {
    STATE.lock().phase == Phase::Pending
}

pub(crate) fn visible_in_inventory() -> bool {
    STATE.lock().phase != Phase::Missing
}

pub(crate) fn health_state(snapshot: Snapshot) -> &'static str {
    match snapshot.phase {
        "pending_physical_approval" => "starting",
        "running" => "healthy",
        "stopped" => "stopped",
        "crashed" => "degraded",
        _ => "missing",
    }
}

pub(crate) fn prepare(
    project_id: [u8; 16],
    revision_sha256: [u8; 32],
    receipt_sha256: [u8; 32],
    candidate_sha256: [u8; 32],
) -> Outcome {
    {
        let state = STATE.lock();
        if state.phase != Phase::Missing {
            return denied(&state, "workspace_service_slot_occupied");
        }
    }
    let binding = match validate_exact(
        project_id,
        revision_sha256,
        receipt_sha256,
        candidate_sha256,
    ) {
        Ok(binding) => binding,
        Err(reason) => return denied(&STATE.lock(), reason),
    };
    let mut state = STATE.lock();
    state.phase = Phase::Pending;
    state.binding = Some(binding);
    state.approval = None;
    state.last_reason = "workspace_run_pending_physical_approval";
    state.last_run_outcome = "not_run";
    state.last_return_value = None;
    state.last_fuel_used = 0;
    let outcome = Outcome {
        accepted: true,
        reason: state.last_reason,
        snapshot: state.snapshot(),
    };
    drop(state);
    console::write_event(format_args!(
        "Approve workspace {:02x}{:02x}{:02x}{:02x} candidate {:02x}{:02x}{:02x}{:02x} receipt {:02x}{:02x}{:02x}{:02x}: imports=0, memory=4 MiB, current_boot only",
        project_id[0],
        project_id[1],
        project_id[2],
        project_id[3],
        candidate_sha256[0],
        candidate_sha256[1],
        candidate_sha256[2],
        candidate_sha256[3],
        receipt_sha256[0],
        receipt_sha256[1],
        receipt_sha256[2],
        receipt_sha256[3],
    ));
    serial::write_raw_str(
        "WORKSPACE_CURRENT_BOOT_PENDING result=accepted candidate_sha256=sha256:",
    );
    write_hash(candidate_sha256);
    serial::write_raw_str(" receipt_sha256=sha256:");
    write_hash(receipt_sha256);
    serial::write_raw_str(
        " imports=0 memory_limit=4194304 scope=current_boot approval=genesis_pointer_required\r\n",
    );
    outcome
}

pub(crate) fn cancel() -> Outcome {
    let mut state = STATE.lock();
    if state.phase != Phase::Pending {
        return denied(&state, "workspace_no_pending_approval");
    }
    *state = State::new();
    Outcome {
        accepted: true,
        reason: "workspace_pending_run_cancelled",
        snapshot: state.snapshot(),
    }
}

/// Only Genesis calls this after observing a fresh pointer press inside the
/// core-owned approval button. No agent/serial protocol route reaches it.
pub(crate) fn approve_and_run_from_pointer() -> bool {
    let binding = {
        let state = STATE.lock();
        if state.phase != Phase::Pending {
            return false;
        }
        let Some(binding) = state.binding else {
            return false;
        };
        binding
    };
    if let Err(reason) = validate_binding(binding) {
        let mut state = STATE.lock();
        state.phase = Phase::Crashed;
        state.last_reason = reason;
        emit_activation_marker(state.snapshot(), false);
        return true;
    }
    let approval = build_workspace_pointer_approval(&binding);
    {
        let mut state = STATE.lock();
        state.approval = Some(approval);
        state.generation = state.generation.saturating_add(1);
    }
    run_approved("workspace_physical_pointer_approved");
    true
}

pub(crate) fn start() -> Outcome {
    let phase = STATE.lock().phase;
    match phase {
        Phase::Running => return accepted("workspace_service_already_running"),
        Phase::Stopped => {}
        Phase::Pending => return denied(&STATE.lock(), "workspace_physical_approval_required"),
        Phase::Crashed => return denied(&STATE.lock(), "workspace_crash_requires_new_approval"),
        Phase::Missing => return denied(&STATE.lock(), "workspace_service_not_loaded"),
    }
    let binding = STATE.lock().binding;
    let Some(binding) = binding else {
        return denied(&STATE.lock(), "workspace_service_binding_missing");
    };
    if let Err(reason) = validate_binding(binding) {
        return denied(&STATE.lock(), reason);
    }
    run_approved("workspace_service_restarted")
}

pub(crate) fn stop() -> Outcome {
    let mut state = STATE.lock();
    match state.phase {
        Phase::Running => {
            state.phase = Phase::Stopped;
            state.last_reason = "workspace_service_stopped";
            Outcome {
                accepted: true,
                reason: state.last_reason,
                snapshot: state.snapshot(),
            }
        }
        Phase::Stopped => accepted_locked(&state, "workspace_service_already_stopped"),
        Phase::Pending => denied(&state, "workspace_physical_approval_required"),
        Phase::Crashed => denied(&state, "workspace_service_crashed"),
        Phase::Missing => denied(&state, "workspace_service_not_loaded"),
    }
}

pub(crate) fn drop_service(reason: &'static str) -> Outcome {
    let mut state = STATE.lock();
    if state.phase == Phase::Missing {
        return denied(&state, "workspace_service_not_loaded");
    }
    let owned_hash = state.binding.map(|binding| binding.candidate_sha256);
    if module_candidate_intake::retained()
        .as_ref()
        .is_some_and(|candidate| Some(candidate.sha256) == owned_hash)
    {
        module_candidate_intake::clear();
    }
    *state = State::new();
    state.last_reason = reason;
    Outcome {
        accepted: true,
        reason,
        snapshot: state.snapshot(),
    }
}

pub(crate) fn secure_attention_drop() -> bool {
    if STATE.lock().phase == Phase::Missing {
        return false;
    }
    let _ = drop_service("workspace_f12_drop_to_genesis");
    serial::write_line(
        "WORKSPACE_CURRENT_BOOT_RECOVERY secure_attention=F12 action=drop_to_genesis result=accepted",
    );
    true
}

fn run_approved(success_reason: &'static str) -> Outcome {
    let candidate = match module_candidate_intake::retained() {
        Some(candidate) => candidate,
        None => return denied(&STATE.lock(), "workspace_candidate_missing"),
    };
    let run = wasm_runtime::execute_workspace_no_import_candidate(&candidate.bytes);
    // The ABI deliberately exposes the i32 as the application's result, not as
    // a process-exit convention. A completed typed call is success regardless
    // of its value; individual fixtures may assert a particular result.
    let success = run.run_outcome == "success" && run.return_value.is_some();
    let mut state = STATE.lock();
    state.run_count = state.run_count.saturating_add(1);
    state.last_run_outcome = run.run_outcome;
    state.last_return_value = run.return_value;
    state.last_fuel_used = run.fuel_used;
    state.phase = if success {
        Phase::Running
    } else {
        Phase::Crashed
    };
    state.last_reason = if success {
        success_reason
    } else {
        state.last_run_outcome
    };
    let outcome = Outcome {
        accepted: success,
        reason: state.last_reason,
        snapshot: state.snapshot(),
    };
    emit_activation_marker(outcome.snapshot, success);
    if success {
        console::write_event(format_args!(
            "Workspace app running current_boot result={}",
            run.return_value.unwrap_or(i32::MIN)
        ));
    } else {
        console::write_event(format_args!(
            "Workspace app fallback to Genesis: {}",
            outcome.reason
        ));
    }
    outcome
}

fn validate_exact(
    project_id: [u8; 16],
    revision_sha256: [u8; 32],
    receipt_sha256: [u8; 32],
    candidate_sha256: [u8; 32],
) -> Result<WorkspaceRunBinding, &'static str> {
    let receipt =
        project_build::verified_receipt_exact(project_id, revision_sha256, receipt_sha256)?
            .ok_or("workspace_receipt_not_found")?;
    let candidate = module_candidate_intake::retained().ok_or("workspace_candidate_missing")?;
    if !candidate.wasm_valid || candidate.sha256 != candidate_sha256 {
        return Err("workspace_candidate_mismatch");
    }
    let inspection = wasm_runtime::inspect_workspace_imports(&candidate.bytes);
    if !inspection.validation_ok {
        return Err(inspection.reason);
    }
    build_workspace_run_binding(
        &receipt,
        candidate.sha256,
        candidate.bytes.len(),
        inspection.import_list_observed,
        inspection.import_count,
    )
    .map_err(|error| error.reason())
}

fn validate_binding(binding: WorkspaceRunBinding) -> Result<(), &'static str> {
    let current = validate_exact(
        binding.project_id,
        binding.project_revision_sha256,
        binding.receipt_sha256,
        binding.candidate_sha256,
    )?;
    if current != binding {
        return Err("workspace_approval_binding_stale");
    }
    Ok(())
}

fn accepted(reason: &'static str) -> Outcome {
    accepted_locked(&STATE.lock(), reason)
}

fn accepted_locked(state: &State, reason: &'static str) -> Outcome {
    Outcome {
        accepted: true,
        reason,
        snapshot: state.snapshot(),
    }
}

fn denied(state: &State, reason: &'static str) -> Outcome {
    Outcome {
        accepted: false,
        reason,
        snapshot: state.snapshot(),
    }
}

fn emit_activation_marker(snapshot: Snapshot, accepted: bool) {
    let Some(binding) = snapshot.binding else {
        return;
    };
    serial::write_raw_str("WORKSPACE_CURRENT_BOOT_ACTIVATION physical_approval=pointer result=");
    serial::write_raw_str(if accepted { "accepted" } else { "denied" });
    serial::write_raw_str(" candidate_sha256=sha256:");
    write_hash(binding.candidate_sha256);
    serial::write_raw_str(" receipt_sha256=sha256:");
    write_hash(binding.receipt_sha256);
    serial::write_raw_fmt(format_args!(
        " imports=0 entrypoint={} fuel_budget={} memory_limit={} run_outcome={} return_value={}\r\n",
        WORKSPACE_ENTRYPOINT,
        WORKSPACE_FUEL_BUDGET,
        WORKSPACE_MEMORY_LIMIT_BYTES,
        snapshot.last_run_outcome,
        snapshot.last_return_value.unwrap_or(i32::MIN),
    ));
}

fn write_hash(hash: [u8; 32]) {
    for byte in hash {
        serial::write_raw_fmt(format_args!("{byte:02x}"));
    }
}

pub(crate) const fn instance_limit() -> usize {
    WORKSPACE_INSTANCE_LIMIT
}

pub(crate) const fn memory_count_limit() -> usize {
    WORKSPACE_MEMORY_COUNT_LIMIT
}

pub(crate) const fn table_limit() -> usize {
    WORKSPACE_TABLE_LIMIT
}

pub(crate) const fn service_id() -> &'static str {
    WORKSPACE_SERVICE_ID
}

pub(crate) const fn trust_tier() -> &'static str {
    WORKSPACE_TRUST_TIER
}

fn target_matches(input: &str, method: &str) -> bool {
    let mut parts = input.split_ascii_whitespace();
    parts.next() == Some(method)
        && parts.next() == Some(WORKSPACE_SERVICE_ID)
        && parts.next().is_none()
}

pub(crate) fn is_health_method(input: &str) -> bool {
    target_matches(input, "service.health")
}

pub(crate) fn is_start_method(input: &str) -> bool {
    target_matches(input, "service.start")
}

pub(crate) fn is_stop_method(input: &str) -> bool {
    target_matches(input, "service.stop")
}

pub(crate) fn is_drop_method(input: &str) -> bool {
    target_matches(input, "service.drop")
}
