use alloc::vec::Vec;

use raios_core::{
    project_install::{
        validate_install_envelope, ProjectInstallEnvelope, PROJECT_INSTALL_TRUST_TIER,
    },
    project_runtime::{
        build_workspace_pointer_approval, build_workspace_pointer_approval_from_challenge,
        build_workspace_run_binding, workspace_import_list_sha256, WorkspacePhysicalApproval,
        WorkspaceRunBinding, WORKSPACE_ENTRYPOINT, WORKSPACE_FUEL_BUDGET, WORKSPACE_INSTANCE_LIMIT,
        WORKSPACE_MEMORY_COUNT_LIMIT, WORKSPACE_MEMORY_LIMIT_BYTES, WORKSPACE_SERVICE_ID,
        WORKSPACE_TABLE_LIMIT, WORKSPACE_TRUST_TIER,
    },
    project_workspace::{Classification, ProjectId},
    sha256_bytes,
};
use spin::Mutex;
use wasmi::{core::ValueType, Engine, Module};

use crate::{
    agent_build_loop, console, module_candidate_intake, project_build, project_store, serial,
    wasm_runtime,
};

const ASSEMBLED_CHALLENGE_DOMAIN: &[u8] = b"raios.assembled_workspace_run_challenge.v1";
const ASSEMBLED_TRUST_TIER: &str =
    "on_device_signed_assembler_double_build_plus_physical_current_boot";

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
    assembled_binding: Option<AssembledWorkspaceRunBinding>,
    approval: Option<WorkspacePhysicalApproval>,
    generation: u64,
    run_count: u64,
    last_reason: &'static str,
    last_run_outcome: &'static str,
    last_return_value: Option<i32>,
    last_fuel_used: u64,
    activation_authority: &'static str,
}

impl State {
    const fn new() -> Self {
        Self {
            phase: Phase::Missing,
            binding: None,
            assembled_binding: None,
            approval: None,
            generation: 0,
            run_count: 0,
            last_reason: "not_prepared",
            last_run_outcome: "not_run",
            last_return_value: None,
            last_fuel_used: 0,
            activation_authority: "none",
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            phase: self.phase.label(),
            binding: self.binding,
            assembled_binding: self.assembled_binding,
            approval: self.approval,
            generation: self.generation,
            run_count: self.run_count,
            last_reason: self.last_reason,
            last_run_outcome: self.last_run_outcome,
            last_return_value: self.last_return_value,
            last_fuel_used: self.last_fuel_used,
            activation_authority: self.activation_authority,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Snapshot {
    pub(crate) phase: &'static str,
    pub(crate) binding: Option<WorkspaceRunBinding>,
    pub(crate) assembled_binding: Option<AssembledWorkspaceRunBinding>,
    pub(crate) approval: Option<WorkspacePhysicalApproval>,
    pub(crate) generation: u64,
    pub(crate) run_count: u64,
    pub(crate) last_reason: &'static str,
    pub(crate) last_run_outcome: &'static str,
    pub(crate) last_return_value: Option<i32>,
    pub(crate) last_fuel_used: u64,
    pub(crate) activation_authority: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AssembledWorkspaceRunBinding {
    pub(crate) project_id: [u8; 16],
    pub(crate) project_revision_sha256: [u8; 32],
    pub(crate) source_tree_sha256: [u8; 32],
    pub(crate) input_sha256: [u8; 32],
    pub(crate) input_byte_len: usize,
    pub(crate) candidate_sha256: [u8; 32],
    pub(crate) candidate_byte_len: usize,
    pub(crate) import_list_sha256: [u8; 32],
    pub(crate) import_count: usize,
    pub(crate) fuel_budget: u64,
    pub(crate) memory_limit_bytes: usize,
    pub(crate) instance_limit: usize,
    pub(crate) memory_count_limit: usize,
    pub(crate) table_limit: usize,
    pub(crate) approval_challenge_sha256: [u8; 32],
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
    state.assembled_binding = None;
    state.approval = None;
    state.last_reason = "workspace_run_pending_physical_approval";
    state.last_run_outcome = "not_run";
    state.last_return_value = None;
    state.last_fuel_used = 0;
    state.activation_authority = "genesis_pointer_pending";
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

pub(crate) fn prepare_assembled(
    project_id: [u8; 16],
    revision_sha256: [u8; 32],
    tree_sha256: [u8; 32],
    input_sha256: [u8; 32],
    input_byte_len: usize,
    output_sha256: [u8; 32],
    candidate_bytes: Vec<u8>,
) -> Outcome {
    {
        let state = STATE.lock();
        if state.phase != Phase::Missing {
            return denied(&state, "workspace_service_slot_occupied");
        }
    }
    let binding = match build_assembled_binding(
        project_id,
        revision_sha256,
        tree_sha256,
        input_sha256,
        input_byte_len,
        output_sha256,
        &candidate_bytes,
    ) {
        Ok(binding) => binding,
        Err(reason) => return denied(&STATE.lock(), reason),
    };
    if let Err(reason) = validate_assembled_source_current(binding) {
        return denied(&STATE.lock(), reason);
    }
    module_candidate_intake::retain(candidate_bytes, output_sha256, true);
    let mut state = STATE.lock();
    state.phase = Phase::Pending;
    state.binding = None;
    state.assembled_binding = Some(binding);
    state.approval = None;
    state.last_reason = "build_run_pending_physical_approval";
    state.last_run_outcome = "not_run";
    state.last_return_value = None;
    state.last_fuel_used = 0;
    state.activation_authority = "genesis_pointer_pending";
    let outcome = Outcome {
        accepted: true,
        reason: state.last_reason,
        snapshot: state.snapshot(),
    };
    drop(state);
    console::write_event(format_args!(
        "Approve assembled workspace candidate {:02x}{:02x}{:02x}{:02x}: imports=0, memory=4 MiB, current_boot only",
        output_sha256[0], output_sha256[1], output_sha256[2], output_sha256[3],
    ));
    serial::write_raw_str("BUILD_RUN_PENDING result=accepted revision_sha256=sha256:");
    write_hash(revision_sha256);
    serial::write_raw_str(" tree_sha256=sha256:");
    write_hash(tree_sha256);
    serial::write_raw_str(" input_sha256=sha256:");
    write_hash(input_sha256);
    serial::write_raw_str(" output_sha256=sha256:");
    write_hash(output_sha256);
    serial::write_raw_fmt(format_args!(
        " imports=0 entrypoint={} fuel_budget={} memory_limit={} scope=current_boot approval=genesis_pointer_required\r\n",
        WORKSPACE_ENTRYPOINT, WORKSPACE_FUEL_BUDGET, WORKSPACE_MEMORY_LIMIT_BYTES,
    ));
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
    let (binding, assembled_binding) = {
        let state = STATE.lock();
        if state.phase != Phase::Pending {
            return false;
        }
        if state.binding.is_none() == state.assembled_binding.is_none() {
            return false;
        }
        (state.binding, state.assembled_binding)
    };
    let validation = match (binding, assembled_binding) {
        (Some(binding), None) => validate_binding(binding),
        (None, Some(binding)) => validate_assembled_binding(binding),
        _ => Err("workspace_approval_binding_invalid"),
    };
    if let Err(reason) = validation {
        let mut state = STATE.lock();
        state.phase = Phase::Crashed;
        state.last_reason = reason;
        emit_activation_marker(state.snapshot(), false);
        return true;
    }
    let approval = match (binding, assembled_binding) {
        (Some(binding), None) => build_workspace_pointer_approval(&binding),
        (None, Some(binding)) => {
            build_workspace_pointer_approval_from_challenge(binding.approval_challenge_sha256)
        }
        _ => return false,
    };
    {
        let mut state = STATE.lock();
        state.approval = Some(approval);
        state.activation_authority = "genesis_pointer";
        state.generation = state.generation.saturating_add(1);
    }
    run_approved("workspace_physical_pointer_approved");
    true
}

/// Restores and executes one physically installed, durable app without inventing
/// a fresh Genesis approval. The signed install action is the activation
/// authority; every runtime field is rederived from the receipt and candidate
/// and compared to the sealed envelope before either byte is retained.
pub(crate) fn activate_installed(
    envelope: &ProjectInstallEnvelope,
    receipt_bytes: &[u8],
    candidate_bytes: Vec<u8>,
    reason: &'static str,
) -> Outcome {
    {
        let state = STATE.lock();
        if state.phase != Phase::Missing {
            return denied(&state, "project_install_workspace_slot_occupied");
        }
    }
    let binding = match validate_installed(envelope, receipt_bytes, &candidate_bytes) {
        Ok(binding) => binding,
        Err(reason) => return denied(&STATE.lock(), reason),
    };
    module_candidate_intake::retain(candidate_bytes, binding.candidate_sha256, true);
    {
        let mut state = STATE.lock();
        *state = State::new();
        state.phase = Phase::Stopped;
        state.binding = Some(binding);
        state.generation = envelope.generation;
        state.last_reason = "project_install_restored_exact";
        state.activation_authority = "durable_project_install";
    }
    run_approved(reason)
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
    if STATE.lock().assembled_binding.is_some() {
        return denied(&STATE.lock(), "build_run_fresh_prepare_required");
    }
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
    let owned_hash = state
        .binding
        .map(|binding| binding.candidate_sha256)
        .or_else(|| {
            state
                .assembled_binding
                .map(|binding| binding.candidate_sha256)
        });
    if module_candidate_intake::retained()
        .as_ref()
        .is_some_and(|candidate| Some(candidate.sha256) == owned_hash)
    {
        module_candidate_intake::clear();
    }
    *state = State::new();
    state.last_reason = reason;
    let outcome = Outcome {
        accepted: true,
        reason,
        snapshot: state.snapshot(),
    };
    drop(state);
    crate::project_app_autoload::note_runtime_drop_without_tombstone();
    outcome
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

fn build_assembled_binding(
    project_id: [u8; 16],
    revision_sha256: [u8; 32],
    tree_sha256: [u8; 32],
    input_sha256: [u8; 32],
    input_byte_len: usize,
    output_sha256: [u8; 32],
    candidate_bytes: &[u8],
) -> Result<AssembledWorkspaceRunBinding, &'static str> {
    if project_id == [0; 16]
        || revision_sha256 == [0; 32]
        || tree_sha256 == [0; 32]
        || input_sha256 == [0; 32]
        || input_byte_len == 0
        || output_sha256 == [0; 32]
        || candidate_bytes.is_empty()
        || sha256_bytes(candidate_bytes) != output_sha256
    {
        return Err("build_run_binding_invalid");
    }
    let inspection = wasm_runtime::inspect_workspace_imports(candidate_bytes);
    if !inspection.validation_ok {
        return Err(inspection.reason);
    }
    if inspection.import_count != 0 {
        return Err("workspace_import_surface_not_empty");
    }
    let engine = Engine::default();
    let module = Module::new(&engine, candidate_bytes).map_err(|_| "produced_module_invalid")?;
    let mut exports = module.exports();
    let exact_entrypoint = exports.next().is_some_and(|export| {
        export.name() == WORKSPACE_ENTRYPOINT
            && export
                .ty()
                .func()
                .is_some_and(|ty| ty.params().is_empty() && ty.results() == [ValueType::I32])
    }) && exports.next().is_none();
    if !exact_entrypoint {
        return Err("workspace_entrypoint_invalid");
    }
    let mut binding = AssembledWorkspaceRunBinding {
        project_id,
        project_revision_sha256: revision_sha256,
        source_tree_sha256: tree_sha256,
        input_sha256,
        input_byte_len,
        candidate_sha256: output_sha256,
        candidate_byte_len: candidate_bytes.len(),
        import_list_sha256: workspace_import_list_sha256(),
        import_count: inspection.import_count,
        fuel_budget: WORKSPACE_FUEL_BUDGET,
        memory_limit_bytes: WORKSPACE_MEMORY_LIMIT_BYTES,
        instance_limit: WORKSPACE_INSTANCE_LIMIT,
        memory_count_limit: WORKSPACE_MEMORY_COUNT_LIMIT,
        table_limit: WORKSPACE_TABLE_LIMIT,
        approval_challenge_sha256: [0; 32],
    };
    binding.approval_challenge_sha256 = assembled_challenge_hash(&binding);
    Ok(binding)
}

fn validate_assembled_binding(binding: AssembledWorkspaceRunBinding) -> Result<(), &'static str> {
    validate_assembled_source_current(binding)?;
    let candidate = module_candidate_intake::retained().ok_or("workspace_candidate_missing")?;
    let current = build_assembled_binding(
        binding.project_id,
        binding.project_revision_sha256,
        binding.source_tree_sha256,
        binding.input_sha256,
        binding.input_byte_len,
        binding.candidate_sha256,
        &candidate.bytes,
    )?;
    if !candidate.wasm_valid || current != binding {
        return Err("workspace_approval_binding_stale");
    }
    Ok(())
}

fn validate_assembled_source_current(
    binding: AssembledWorkspaceRunBinding,
) -> Result<(), &'static str> {
    let snapshot = agent_build_loop::snapshot();
    let current = snapshot
        .latest_revision
        .as_ref()
        .filter(|revision| snapshot.project_id == Some(binding.project_id))
        .ok_or("build_assemble_revision_stale")?;
    if current.project_id.bytes() != binding.project_id
        || current.revision_sha256 != binding.project_revision_sha256
        || current.tree_sha256 != binding.source_tree_sha256
    {
        return Err("build_assemble_revision_stale");
    }
    let loaded = project_store::load_revision(
        ProjectId::new(binding.project_id),
        binding.project_revision_sha256,
    )
    .map_err(|error| error.reason())?
    .ok_or("build_assemble_revision_not_readable")?;
    if loaded.revision != *current {
        return Err("build_assemble_revision_stale");
    }
    let mut matching = loaded
        .files
        .into_iter()
        .filter(|file| file.entry.path == "main.rwir");
    let file = matching.next().ok_or("build_assemble_main_rwir_missing")?;
    if matching.next().is_some()
        || file.entry.media_type != "text/raios-wasm-ir"
        || file.entry.classification != Classification::LocalOnly
        || file.entry.byte_len != binding.input_byte_len
        || file.entry.blob_sha256 != binding.input_sha256
        || file.bytes.len() != binding.input_byte_len
        || sha256_bytes(&file.bytes) != binding.input_sha256
    {
        return Err("workspace_approval_binding_stale");
    }
    Ok(())
}

fn assembled_challenge_hash(binding: &AssembledWorkspaceRunBinding) -> [u8; 32] {
    let mut bytes = Vec::from(ASSEMBLED_CHALLENGE_DOMAIN);
    bytes.extend_from_slice(&binding.project_id);
    bytes.extend_from_slice(&binding.project_revision_sha256);
    bytes.extend_from_slice(&binding.source_tree_sha256);
    bytes.extend_from_slice(&binding.input_sha256);
    bytes.extend_from_slice(&(binding.input_byte_len as u64).to_le_bytes());
    bytes.extend_from_slice(&binding.candidate_sha256);
    bytes.extend_from_slice(&(binding.candidate_byte_len as u64).to_le_bytes());
    bytes.extend_from_slice(&binding.import_list_sha256);
    bytes.extend_from_slice(&(binding.import_count as u64).to_le_bytes());
    bytes.extend_from_slice(b"main.rwir");
    bytes.extend_from_slice(b"text/raios-wasm-ir");
    bytes.extend_from_slice(WORKSPACE_SERVICE_ID.as_bytes());
    bytes.extend_from_slice(WORKSPACE_ENTRYPOINT.as_bytes());
    bytes.extend_from_slice(&binding.fuel_budget.to_le_bytes());
    bytes.extend_from_slice(&(binding.memory_limit_bytes as u64).to_le_bytes());
    bytes.extend_from_slice(&(binding.instance_limit as u64).to_le_bytes());
    bytes.extend_from_slice(&(binding.memory_count_limit as u64).to_le_bytes());
    bytes.extend_from_slice(&(binding.table_limit as u64).to_le_bytes());
    sha256_bytes(&bytes)
}

fn validate_installed(
    envelope: &ProjectInstallEnvelope,
    receipt_bytes: &[u8],
    candidate_bytes: &[u8],
) -> Result<WorkspaceRunBinding, &'static str> {
    validate_install_envelope(envelope).map_err(|error| error.reason())?;
    if envelope.service_id != WORKSPACE_SERVICE_ID {
        return Err("project_install_service_id_mismatch");
    }
    if !envelope.auto_start {
        return Err("project_install_autostart_disabled");
    }
    if sha256_bytes(receipt_bytes) != envelope.encoded_receipt_sha256 {
        return Err("project_install_receipt_blob_hash_mismatch");
    }
    let receipt = raios_core::project_build::decode_receipt(receipt_bytes)
        .map_err(|_| "project_install_receipt_invalid")?;
    if sha256_bytes(candidate_bytes) != envelope.candidate_sha256
        || candidate_bytes.len() as u64 != envelope.candidate_byte_len
    {
        return Err("project_install_candidate_blob_hash_mismatch");
    }
    let inspection = wasm_runtime::inspect_workspace_imports(candidate_bytes);
    if !inspection.validation_ok {
        return Err(inspection.reason);
    }
    let binding = build_workspace_run_binding(
        &receipt,
        envelope.candidate_sha256,
        candidate_bytes.len(),
        inspection.import_list_observed,
        inspection.import_count,
    )
    .map_err(|error| error.reason())?;
    let exact = envelope.project_id == binding.project_id
        && envelope.project_revision_sha256 == binding.project_revision_sha256
        && envelope.input_manifest_sha256 == binding.input_manifest_sha256
        && envelope.build_receipt_sha256 == binding.receipt_sha256
        && envelope.workspace_run_binding_sha256 == binding.approval_challenge_sha256
        && envelope.candidate_sha256 == binding.candidate_sha256
        && envelope.candidate_byte_len == binding.candidate_byte_len as u64
        && envelope.import_list_sha256 == binding.import_list_sha256
        && envelope.import_count == binding.import_count as u32
        && envelope.entrypoint == WORKSPACE_ENTRYPOINT
        && envelope.fuel_budget == binding.fuel_budget
        && envelope.memory_limit_bytes == binding.memory_limit_bytes as u64
        && envelope.instance_limit == binding.instance_limit as u32
        && envelope.memory_count_limit == binding.memory_count_limit as u32
        && envelope.table_limit == binding.table_limit as u32
        && envelope.trust_tier == PROJECT_INSTALL_TRUST_TIER;
    if !exact {
        return Err("project_install_runtime_binding_mismatch");
    }
    Ok(binding)
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
    if let Some(binding) = snapshot.assembled_binding {
        serial::write_raw_str("BUILD_RUN_ACTIVATION physical_approval=pointer result=");
        serial::write_raw_str(if accepted { "accepted" } else { "denied" });
        serial::write_raw_str(" revision_sha256=sha256:");
        write_hash(binding.project_revision_sha256);
        serial::write_raw_str(" tree_sha256=sha256:");
        write_hash(binding.source_tree_sha256);
        serial::write_raw_str(" input_sha256=sha256:");
        write_hash(binding.input_sha256);
        serial::write_raw_str(" output_sha256=sha256:");
        write_hash(binding.candidate_sha256);
        serial::write_raw_fmt(format_args!(
            " imports=0 entrypoint={} fuel_budget={} memory_limit={} run_outcome={} return_value={} reason={} install=false persistence=false\r\n",
            WORKSPACE_ENTRYPOINT,
            WORKSPACE_FUEL_BUDGET,
            WORKSPACE_MEMORY_LIMIT_BYTES,
            snapshot.last_run_outcome,
            snapshot.last_return_value.unwrap_or(i32::MIN),
            snapshot.last_reason,
        ));
        return;
    }
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

pub(crate) fn trust_tier() -> &'static str {
    if STATE.lock().assembled_binding.is_some() {
        ASSEMBLED_TRUST_TIER
    } else {
        WORKSPACE_TRUST_TIER
    }
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
