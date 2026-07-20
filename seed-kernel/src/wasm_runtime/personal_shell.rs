use super::*;
use super::{envelope::metered_engine, invocation::wasm_execution_busy};

const PERSONAL_SHELL_CONTEXT_BYTES: usize = 32;
const PERSONAL_SHELL_MAX_INPUT_BYTES: usize = 1_040;
const PERSONAL_SHELL_MAX_MEMORY_BYTES: usize = 2 * 1024 * 1024;
const PERSONAL_SHELL_TABLE_COUNT: usize = 1;
const PERSONAL_SHELL_TABLE_ELEMENTS: u32 = 2;

/// A single current-boot personal-shell invocation. The only data crossing into
/// Wasm are the caller-staged immutable packets and the clipped logical viewport.
/// Packet construction and framebuffer presentation stay outside this runtime.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PersonalShellFrameResult {
    Accepted(Vec<u8>),
    Rejected,
}

pub(crate) struct PersonalShellRuntimeResult {
    pub(crate) frame: PersonalShellFrameResult,
    pub(crate) validation_ok: bool,
    pub(crate) import_grant_performed: bool,
    pub(crate) import_grant_reason: &'static str,
    pub(crate) linked_host_import_count: u64,
    pub(crate) module_imports_exact: bool,
    pub(crate) instantiation_error_kind: &'static str,
    pub(crate) run_outcome: &'static str,
    pub(crate) return_value: Option<i32>,
    pub(crate) fuel_used: u64,
}

/// Bounded evidence for one real current-boot proof invocation. Frame bytes and
/// input packets are intentionally discarded before this crosses the runtime
/// boundary; hashes and the clipped-command observation are enough for the
/// diagnostic caller.
pub(crate) struct PersonalShellProofCase {
    pub(crate) accepted: bool,
    pub(crate) instantiation_error_kind: &'static str,
    pub(crate) run_outcome: &'static str,
    pub(crate) return_value: Option<i32>,
    pub(crate) fuel_used: u64,
    pub(crate) frame_sha256: Option<[u8; 32]>,
    pub(crate) clipped_overdraw: bool,
}

/// Current-boot-only proof evidence. This neither installs the proof service
/// nor renders its frame; it is a bounded diagnostic of the real signed path.
pub(crate) struct PersonalShellProofProbe {
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) artifact_identity_descriptor_sha256: [u8; 32],
    pub(crate) artifact_signature_evidence_sha256: [u8; 32],
    pub(crate) load_descriptor_sha256: [u8; 32],
    pub(crate) descriptor_signature_evidence_sha256: [u8; 32],
    pub(crate) authorized_import_list_sha256: [u8; 32],
    pub(crate) artifact_validation_ok: bool,
    pub(crate) authorized_import_count: u64,
    pub(crate) linked_host_import_count: u64,
    pub(crate) normal: PersonalShellProofCase,
    pub(crate) sanitized_input: PersonalShellProofCase,
    pub(crate) malformed_frame: PersonalShellProofCase,
    pub(crate) clipped_overdraw: PersonalShellProofCase,
    pub(crate) guest_trap: PersonalShellProofCase,
    pub(crate) fuel_exhaustion: PersonalShellProofCase,
    pub(crate) frame_changed_after_sanitized_input: bool,
    pub(crate) missing_frame_submit_linker_denial: &'static str,
    pub(crate) broader_import_denial: &'static str,
}

const PERSONAL_SHELL_PROBE_VIEWPORT: Viewport = Viewport {
    width: 640,
    height: 480,
};
const PERSONAL_SHELL_PROBE_MALFORMED_FRAME: u16 = 0x7ff1;
const PERSONAL_SHELL_PROBE_CLIPPED_OVERDRAW: u16 = 0x7ff2;
const PERSONAL_SHELL_PROBE_TRAP: u16 = 0x7ff3;
const PERSONAL_SHELL_PROBE_FUEL_EXHAUSTION: u16 = 0x7ff4;
const PERSONAL_SHELL_MISSING_FRAME_SUBMIT_IMPORTS: &[(&str, &str)] = &[
    ("ui", "viewport"),
    ("ui", "context_len"),
    ("ui", "context_read"),
    ("ui", "input_len"),
    ("ui", "input_read"),
];
const PERSONAL_SHELL_BROADER_IMPORTS: &[(&str, &str)] = &[
    ("ui", "viewport"),
    ("ui", "context_len"),
    ("ui", "context_read"),
    ("ui", "input_len"),
    ("ui", "input_read"),
    ("ui", "frame_submit"),
    ("env", "log"),
];

/// Exercises the real signed proof artifact through separate fresh Wasm calls.
/// This function intentionally exposes no raw frame or packet bytes.
pub(crate) fn run_personal_shell_proof_probe() -> PersonalShellProofProbe {
    let normal_runtime = personal_shell_probe_run(1, None);
    let artifact_validation_ok = normal_runtime.validation_ok;
    let authorized_import_count = if normal_runtime.import_grant_performed {
        PERSONAL_SHELL_UI_IMPORTS.len() as u64
    } else {
        0
    };
    let linked_host_import_count = normal_runtime.linked_host_import_count;
    let normal = personal_shell_proof_case(normal_runtime, PERSONAL_SHELL_PROBE_VIEWPORT);
    let sanitized_input = personal_shell_proof_case(
        personal_shell_probe_run(2, Some(0)),
        PERSONAL_SHELL_PROBE_VIEWPORT,
    );
    let malformed_frame = personal_shell_proof_case(
        personal_shell_probe_run(3, Some(PERSONAL_SHELL_PROBE_MALFORMED_FRAME)),
        PERSONAL_SHELL_PROBE_VIEWPORT,
    );
    let clipped_overdraw = personal_shell_proof_case(
        personal_shell_probe_run(4, Some(PERSONAL_SHELL_PROBE_CLIPPED_OVERDRAW)),
        PERSONAL_SHELL_PROBE_VIEWPORT,
    );
    let guest_trap = personal_shell_proof_case(
        personal_shell_probe_run(5, Some(PERSONAL_SHELL_PROBE_TRAP)),
        PERSONAL_SHELL_PROBE_VIEWPORT,
    );
    let fuel_exhaustion = personal_shell_proof_case(
        personal_shell_probe_run(6, Some(PERSONAL_SHELL_PROBE_FUEL_EXHAUSTION)),
        PERSONAL_SHELL_PROBE_VIEWPORT,
    );
    let missing_frame_submit_linker_denial =
        evaluate_personal_shell_import_grant(&personal_shell_import_grant_input_for(
            PERSONAL_SHELL_UI_IMPORTS,
            PERSONAL_SHELL_MISSING_FRAME_SUBMIT_IMPORTS,
        ))
        .reason;
    let broader_import_denial =
        evaluate_personal_shell_import_grant(&personal_shell_import_grant_input_for(
            PERSONAL_SHELL_BROADER_IMPORTS,
            PERSONAL_SHELL_BROADER_IMPORTS,
        ))
        .reason;

    PersonalShellProofProbe {
        artifact_sha256: PERSONAL_SHELL_WASM_ARTIFACT_BYTES_HASH,
        artifact_identity_descriptor_sha256: PERSONAL_SHELL_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH,
        artifact_signature_evidence_sha256: PERSONAL_SHELL_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH,
        load_descriptor_sha256: PERSONAL_SHELL_LOAD_DESCRIPTOR_HASH,
        descriptor_signature_evidence_sha256:
            PERSONAL_SHELL_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_HASH,
        authorized_import_list_sha256: authorized_import_list_sha256(
            PERSONAL_SHELL_SERVICE_ID,
            PERSONAL_SHELL_UI_IMPORTS,
        ),
        artifact_validation_ok,
        authorized_import_count,
        linked_host_import_count,
        frame_changed_after_sanitized_input: matches!(
            (normal.frame_sha256, sanitized_input.frame_sha256),
            (Some(normal), Some(input)) if normal != input
        ),
        normal,
        sanitized_input,
        malformed_frame,
        clipped_overdraw,
        guest_trap,
        fuel_exhaustion,
        missing_frame_submit_linker_denial,
        broader_import_denial,
    }
}

fn personal_shell_probe_run(
    invocation_id: u32,
    key_code: Option<u16>,
) -> PersonalShellRuntimeResult {
    let context = PersonalShellContext::new(
        invocation_id,
        PERSONAL_SHELL_PROBE_VIEWPORT.width,
        PERSONAL_SHELL_PROBE_VIEWPORT.height,
        0,
        0,
        0,
        true,
        true,
        0,
    );
    let mut input = PersonalShellInput::new(invocation_id);
    if let Some(code) = key_code {
        let _ = input.push(SanitizedInputEvent::new(
            SanitizedInputKind::Key,
            true,
            false,
            code,
            0,
            0,
            0,
            0,
            0,
        ));
    }
    let context_bytes = context.encode();
    let input_bytes = input.encode();
    run_personal_shell_proof(
        &context_bytes,
        input_bytes.as_bytes(),
        PERSONAL_SHELL_PROBE_VIEWPORT,
    )
}

fn personal_shell_proof_case(
    runtime: PersonalShellRuntimeResult,
    viewport: Viewport,
) -> PersonalShellProofCase {
    let (accepted, frame_sha256, clipped_overdraw) = match runtime.frame {
        PersonalShellFrameResult::Accepted(frame) => (
            runtime.run_outcome == "success",
            Some(sha256_bytes(&frame)),
            personal_shell_frame_has_clipped_overdraw(&frame, viewport),
        ),
        PersonalShellFrameResult::Rejected => (false, None, false),
    };
    PersonalShellProofCase {
        accepted,
        instantiation_error_kind: runtime.instantiation_error_kind,
        run_outcome: runtime.run_outcome,
        return_value: runtime.return_value,
        fuel_used: runtime.fuel_used,
        frame_sha256,
        clipped_overdraw,
    }
}

fn personal_shell_frame_has_clipped_overdraw(frame: &[u8], viewport: Viewport) -> bool {
    let Ok(frame) = ui_frame::validate_frame(frame, viewport) else {
        return false;
    };
    frame.commands().iter().any(|command| {
        matches!(
            command,
            ui_frame::Command::FillRect { rect, .. }
                if rect.x == viewport.width - 1
                    && rect.y == viewport.height - 1
                    && rect.width == 1
                    && rect.height == 1
        )
    })
}

/// Runs only the signed `svc.user.shell` proof artifact in a new metered Wasm
/// store. This is deliberately not a generic artifact loader.
pub(crate) fn run_personal_shell_proof(
    context: &[u8],
    input: &[u8],
    viewport: Viewport,
) -> PersonalShellRuntimeResult {
    if wasm_execution_busy() {
        return personal_shell_result(
            false,
            None,
            0,
            false,
            "wasm_execution_busy",
            None,
            0,
            PersonalShellFrameResult::Rejected,
        );
    }
    let validation_ok = validate_personal_shell_proof_artifact();
    if context.len() < PERSONAL_SHELL_CONTEXT_BYTES
        || context.len() > MAX_PROGRAM_CONTEXT_LEN
        || input.len() > PERSONAL_SHELL_MAX_INPUT_BYTES
        || viewport.width == 0
        || viewport.height == 0
    {
        return personal_shell_result(
            false,
            None,
            0,
            false,
            "invocation_packet_out_of_bounds",
            None,
            0,
            PersonalShellFrameResult::Rejected,
        );
    }
    if !validation_ok {
        return personal_shell_result(
            false,
            None,
            0,
            false,
            "artifact_validation_failed",
            None,
            0,
            PersonalShellFrameResult::Rejected,
        );
    }

    let decision = evaluate_personal_shell_import_grant(&personal_shell_import_grant_input());
    if !decision.performed {
        return personal_shell_result(
            true,
            Some(decision),
            0,
            false,
            "import_grant_denied",
            None,
            0,
            PersonalShellFrameResult::Rejected,
        );
    }

    let wasm = Vec::from(PERSONAL_SHELL_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return personal_shell_result(
                true,
                Some(decision),
                0,
                false,
                "module_compile_failed",
                None,
                0,
                PersonalShellFrameResult::Rejected,
            )
        }
    };
    if !personal_shell_module_imports_exact(&module, decision.authorized_imports) {
        return personal_shell_result(
            true,
            Some(decision),
            0,
            false,
            "module_import_surface_mismatch",
            None,
            0,
            PersonalShellFrameResult::Rejected,
        );
    }

    let mut store = Box::new(Store::new(
        &engine,
        PersonalShellInvocationState::new(context, input, viewport),
    ));
    store.limiter(|state| &mut state.limits);
    if store.add_fuel(PERSONAL_SHELL_WASM_FUEL_BUDGET).is_err() {
        return personal_shell_result(
            true,
            Some(decision),
            0,
            true,
            "fuel_metering_unavailable",
            None,
            0,
            PersonalShellFrameResult::Rejected,
        );
    }

    let mut linker = Box::new(Linker::<PersonalShellInvocationState>::new(&engine));
    let linked_host_import_count = match define_personal_shell_imports(&mut linker, &decision) {
        Ok(count) => count,
        Err(reason) => {
            return personal_shell_result(
                true,
                Some(decision),
                0,
                true,
                reason,
                None,
                store.fuel_consumed().unwrap_or(0),
                PersonalShellFrameResult::Rejected,
            )
        }
    };

    let instance = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(instance) => instance,
            Err(error) => {
                return personal_shell_result(
                    true,
                    Some(decision),
                    linked_host_import_count,
                    true,
                    classify_personal_shell_error(error),
                    None,
                    store.fuel_consumed().unwrap_or(0),
                    PersonalShellFrameResult::Rejected,
                )
            }
        },
        Err(error) => {
            return personal_shell_instantiation_failure(
                Some(decision),
                linked_host_import_count,
                store.fuel_consumed().unwrap_or(0),
                error,
            )
        }
    };
    let Some(entrypoint) = instance
        .get_export(&*store, "raios_service_main")
        .and_then(Extern::into_func)
    else {
        return personal_shell_result(
            true,
            Some(decision),
            linked_host_import_count,
            true,
            "entrypoint_missing",
            None,
            store.fuel_consumed().unwrap_or(0),
            PersonalShellFrameResult::Rejected,
        );
    };

    let mut outputs = Vec::from([Value::I32(0)]).into_boxed_slice();
    match entrypoint.call(&mut *store, &[], &mut outputs) {
        Err(error) => personal_shell_result(
            true,
            Some(decision),
            linked_host_import_count,
            true,
            classify_personal_shell_error(error),
            None,
            store.fuel_consumed().unwrap_or(0),
            PersonalShellFrameResult::Rejected,
        ),
        Ok(()) => {
            let return_value = outputs[0].i32();
            let fuel_used = store.fuel_consumed().unwrap_or(0);
            let rejected = store.data().rejected;
            let frame = store.data_mut().pending_frame.take();
            let (run_outcome, frame) = match (return_value, rejected, frame) {
                (Some(0), false, Some(frame)) => {
                    ("success", PersonalShellFrameResult::Accepted(frame))
                }
                (Some(0), false, None) => ("frame_missing", PersonalShellFrameResult::Rejected),
                (Some(_), _, _) => ("frame_rejected", PersonalShellFrameResult::Rejected),
                (None, _, _) => ("bad_return_type", PersonalShellFrameResult::Rejected),
            };
            personal_shell_result(
                true,
                Some(decision),
                linked_host_import_count,
                true,
                run_outcome,
                return_value,
                fuel_used,
                frame,
            )
        }
    }
}

pub(crate) fn validate_personal_shell_proof_artifact() -> bool {
    let wasm = Vec::from(PERSONAL_SHELL_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == PERSONAL_SHELL_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(PERSONAL_SHELL_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == PERSONAL_SHELL_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(PERSONAL_SHELL_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == PERSONAL_SHELL_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && sha256_bytes(PERSONAL_SHELL_LOAD_DESCRIPTOR_SOURCE.as_bytes())
            == PERSONAL_SHELL_LOAD_DESCRIPTOR_HASH
        && sha256_bytes(PERSONAL_SHELL_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == PERSONAL_SHELL_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_HASH
        && PERSONAL_SHELL_LOAD_DESCRIPTOR_SERVICE_ID == PERSONAL_SHELL_SERVICE_ID
        && PERSONAL_SHELL_LOAD_DESCRIPTOR_ARTIFACT_BYTES_HASH
            == PERSONAL_SHELL_WASM_ARTIFACT_BYTES_HASH
        && PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORTS
            == "ui.viewport,ui.context_len,ui.context_read,ui.input_len,ui.input_read,ui.frame_submit"
        && PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORT_COUNT
            == PERSONAL_SHELL_UI_IMPORTS.len() as u64
        && PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZES_CURRENT_BOOT_WASM_EXECUTION
        && PERSONAL_SHELL_LOAD_DESCRIPTOR_VALIDATES_WITH_WASMI_MODULE_NEW
        && !PERSONAL_SHELL_LOAD_DESCRIPTOR_ACCEPTS_EXTERNAL_ARTIFACT_BYTES
        && !PERSONAL_SHELL_LOAD_DESCRIPTOR_LOADS_EXTERNAL_ARTIFACT
        && !PERSONAL_SHELL_LOAD_DESCRIPTOR_MAPS_EXECUTABLE_PAGES
        && !PERSONAL_SHELL_LOAD_DESCRIPTOR_WRITES_PERSISTENT_STATE
        && !PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZES_PERSISTENT_INSTALL
        && !PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZES_ROLLBACK_INSTALL
        && validate_module_bytes(bytes)
}

fn personal_shell_import_grant_input() -> PersonalShellImportGrantInput<'static> {
    personal_shell_import_grant_input_for(PERSONAL_SHELL_UI_IMPORTS, PERSONAL_SHELL_UI_IMPORTS)
}

fn personal_shell_import_grant_input_for(
    requested_imports: &'static [(&'static str, &'static str)],
    linker_implementations: &'static [(&'static str, &'static str)],
) -> PersonalShellImportGrantInput<'static> {
    let import_list_sha256 =
        authorized_import_list_sha256(PERSONAL_SHELL_SERVICE_ID, requested_imports);
    PersonalShellImportGrantInput {
        service_id: Some(PERSONAL_SHELL_SERVICE_ID),
        artifact_sha256: Some(PERSONAL_SHELL_WASM_ARTIFACT_BYTES_HASH),
        descriptor_source_signature_evidence: Some(VerifiedImportEvidence {
            evidence_sha256: PERSONAL_SHELL_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_HASH,
            artifact_sha256: PERSONAL_SHELL_WASM_ARTIFACT_BYTES_HASH,
            import_list_sha256,
        }),
        artifact_signature_attestation_evidence: Some(VerifiedImportEvidence {
            evidence_sha256: PERSONAL_SHELL_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH,
            artifact_sha256: PERSONAL_SHELL_WASM_ARTIFACT_BYTES_HASH,
            import_list_sha256,
        }),
        computed_grant_evidence: None,
        declared_import_list_sha256: Some(import_list_sha256),
        requested_imports,
        linker_implementations,
    }
}

fn personal_shell_result(
    validation_ok: bool,
    decision: Option<PersonalShellImportGrantDecision>,
    linked_host_import_count: u64,
    module_imports_exact: bool,
    run_outcome: &'static str,
    return_value: Option<i32>,
    fuel_used: u64,
    frame: PersonalShellFrameResult,
) -> PersonalShellRuntimeResult {
    PersonalShellRuntimeResult {
        frame,
        validation_ok,
        import_grant_performed: decision.is_some_and(|decision| decision.performed),
        import_grant_reason: decision.map_or("not_evaluated", |decision| decision.reason),
        linked_host_import_count,
        module_imports_exact,
        instantiation_error_kind: "none",
        run_outcome,
        return_value,
        fuel_used,
    }
}

fn personal_shell_instantiation_failure(
    decision: Option<PersonalShellImportGrantDecision>,
    linked_host_import_count: u64,
    fuel_used: u64,
    error: wasmi::Error,
) -> PersonalShellRuntimeResult {
    let mut result = personal_shell_result(
        true,
        decision,
        linked_host_import_count,
        true,
        "instantiation_failed",
        None,
        fuel_used,
        PersonalShellFrameResult::Rejected,
    );
    result.instantiation_error_kind = classify_personal_shell_instantiation_error(&error);
    result
}

fn classify_personal_shell_instantiation_error(error: &wasmi::Error) -> &'static str {
    match error {
        wasmi::Error::Table(TableError::TooManyTables)
        | wasmi::Error::Instantiation(InstantiationError::Table(TableError::TooManyTables)) => {
            "table_count_limit_exceeded"
        }
        wasmi::Error::Table(TableError::GrowOutOfBounds { .. })
        | wasmi::Error::Instantiation(InstantiationError::Table(TableError::GrowOutOfBounds {
            ..
        })) => "table_element_limit_exceeded",
        wasmi::Error::Memory(MemoryError::TooManyMemories)
        | wasmi::Error::Instantiation(InstantiationError::Memory(MemoryError::TooManyMemories)) => {
            "memory_count_limit_exceeded"
        }
        wasmi::Error::Memory(MemoryError::OutOfBoundsAllocation)
        | wasmi::Error::Instantiation(InstantiationError::Memory(
            MemoryError::OutOfBoundsAllocation,
        )) => "memory_allocation_denied",
        wasmi::Error::Instantiation(InstantiationError::TooManyInstances) => {
            "instance_count_limit_exceeded"
        }
        wasmi::Error::Instantiation(InstantiationError::SignatureMismatch { .. }) => {
            "import_signature_mismatch"
        }
        wasmi::Error::Instantiation(InstantiationError::ImportsExternalsLenMismatch) => {
            "import_count_mismatch"
        }
        wasmi::Error::Linker(LinkerError::MissingDefinition { .. }) => "missing_linker_definition",
        wasmi::Error::Linker(_) => "linker_error",
        _ => "other_instantiation_error",
    }
}

const PERSONAL_CALL_VIEWPORT: u8 = 1 << 0;
const PERSONAL_CALL_CONTEXT_LEN: u8 = 1 << 1;
const PERSONAL_CALL_CONTEXT_READ: u8 = 1 << 2;
const PERSONAL_CALL_INPUT_LEN: u8 = 1 << 3;
const PERSONAL_CALL_INPUT_READ: u8 = 1 << 4;
const PERSONAL_CALL_FRAME_SUBMIT: u8 = 1 << 5;

struct PersonalShellInvocationState {
    context: Vec<u8>,
    input: Vec<u8>,
    viewport: Viewport,
    calls: u8,
    pending_frame: Option<Vec<u8>>,
    rejected: bool,
    limits: StoreLimits,
}

impl PersonalShellInvocationState {
    fn new(context: &[u8], input: &[u8], viewport: Viewport) -> Self {
        Self {
            context: context.to_vec(),
            input: input.to_vec(),
            viewport,
            calls: 0,
            pending_frame: None,
            rejected: false,
            limits: StoreLimitsBuilder::new()
                .memory_size(PERSONAL_SHELL_MAX_MEMORY_BYTES)
                .instances(1)
                .memories(1)
                // The signed proof declares one funcref table with min=max=2.
                // Keep that exact bounded table instead of denying all tables.
                .tables(PERSONAL_SHELL_TABLE_COUNT)
                .table_elements(PERSONAL_SHELL_TABLE_ELEMENTS)
                .build(),
        }
    }
}

fn define_personal_shell_imports(
    linker: &mut Linker<PersonalShellInvocationState>,
    decision: &PersonalShellImportGrantDecision,
) -> Result<u64, &'static str> {
    let mut linked = 0u64;
    let mut index = 0usize;
    while index < decision.authorized_imports.len() {
        match decision.authorized_imports[index] {
            ("ui", "viewport") => {
                linker
                    .func_wrap("ui", "viewport", host_personal_viewport)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("ui", "context_len") => {
                linker
                    .func_wrap("ui", "context_len", host_personal_context_len)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("ui", "context_read") => {
                linker
                    .func_wrap("ui", "context_read", host_personal_context_read)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("ui", "input_len") => {
                linker
                    .func_wrap("ui", "input_len", host_personal_input_len)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("ui", "input_read") => {
                linker
                    .func_wrap("ui", "input_read", host_personal_input_read)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("ui", "frame_submit") => {
                linker
                    .func_wrap("ui", "frame_submit", host_personal_frame_submit)
                    .map_err(|_| "host_import_link_failed")?;
            }
            _ => return Err("missing_host_import_implementation"),
        }
        linked += 1;
        index += 1;
    }
    if linked != PERSONAL_SHELL_UI_IMPORTS.len() as u64 {
        return Err("personal_shell_linker_surface_mismatch");
    }
    Ok(linked)
}

fn personal_shell_module_imports_exact(module: &Module, authorized: &[(&str, &str)]) -> bool {
    let mut index = 0usize;
    for import in module.imports() {
        if authorized.get(index) != Some(&(import.module(), import.name())) {
            return false;
        }
        index += 1;
    }
    index == authorized.len()
}

fn classify_personal_shell_error(error: wasmi::Error) -> &'static str {
    match error {
        wasmi::Error::Trap(trap) if matches!(trap.trap_code(), Some(TrapCode::OutOfFuel)) => {
            "fuel_exhausted"
        }
        wasmi::Error::Trap(_) => "trap",
        _ => "wasm_error",
    }
}

fn charge_personal_call(
    caller: &mut Caller<'_, PersonalShellInvocationState>,
    call: u8,
) -> Result<(), Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let state = caller.data_mut();
    if state.calls & call != 0 {
        state.rejected = true;
        state.pending_frame = None;
        return Err(Trap::new("personal shell import called more than once"));
    }
    state.calls |= call;
    Ok(())
}

fn host_personal_viewport(
    mut caller: Caller<'_, PersonalShellInvocationState>,
) -> Result<i64, Trap> {
    charge_personal_call(&mut caller, PERSONAL_CALL_VIEWPORT)?;
    let viewport = caller.data().viewport;
    Ok(((i64::from(viewport.width)) << 32) | i64::from(viewport.height))
}

fn host_personal_context_len(
    mut caller: Caller<'_, PersonalShellInvocationState>,
) -> Result<i32, Trap> {
    charge_personal_call(&mut caller, PERSONAL_CALL_CONTEXT_LEN)?;
    let len = caller.data().context.len();
    if len < PERSONAL_SHELL_CONTEXT_BYTES || len > MAX_PROGRAM_CONTEXT_LEN {
        return Err(Trap::new("personal shell context length is invalid"));
    }
    Ok(len as i32)
}

fn host_personal_context_read(
    mut caller: Caller<'_, PersonalShellInvocationState>,
    ptr: i32,
    cap: i32,
) -> Result<i32, Trap> {
    charge_personal_call(&mut caller, PERSONAL_CALL_CONTEXT_READ)?;
    personal_packet_read(&mut caller, ptr, cap, true)
}

fn host_personal_input_len(
    mut caller: Caller<'_, PersonalShellInvocationState>,
) -> Result<i32, Trap> {
    charge_personal_call(&mut caller, PERSONAL_CALL_INPUT_LEN)?;
    let len = caller.data().input.len();
    if len > PERSONAL_SHELL_MAX_INPUT_BYTES {
        return Err(Trap::new("personal shell input length exceeds maximum"));
    }
    Ok(len as i32)
}

fn host_personal_input_read(
    mut caller: Caller<'_, PersonalShellInvocationState>,
    ptr: i32,
    cap: i32,
) -> Result<i32, Trap> {
    charge_personal_call(&mut caller, PERSONAL_CALL_INPUT_READ)?;
    personal_packet_read(&mut caller, ptr, cap, false)
}

fn personal_packet_read(
    caller: &mut Caller<'_, PersonalShellInvocationState>,
    ptr: i32,
    cap: i32,
    context: bool,
) -> Result<i32, Trap> {
    if ptr < 0 || cap < 0 {
        return Err(Trap::new(
            "personal shell packet read has negative pointer or capacity",
        ));
    }
    let ptr = ptr as usize;
    let cap = cap as usize;
    ptr.checked_add(cap)
        .ok_or_else(|| Trap::new("personal shell packet read pointer overflow"))?;

    let packet = if context {
        caller.data().context.clone()
    } else {
        caller.data().input.clone()
    };
    if cap < packet.len() {
        return Err(Trap::new(
            "personal shell packet read capacity is insufficient",
        ));
    }
    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("personal shell memory export missing"))?;
    memory
        .write(caller, ptr, &packet)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    Ok(packet.len() as i32)
}

fn host_personal_frame_submit(
    mut caller: Caller<'_, PersonalShellInvocationState>,
    ptr: i32,
    len: i32,
) -> Result<i32, Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if caller.data().calls & PERSONAL_CALL_FRAME_SUBMIT != 0 {
        let state = caller.data_mut();
        state.rejected = true;
        state.pending_frame = None;
        return Ok(ui_frame::FRAME_RESULT_SECOND_SUBMIT);
    }
    caller.data_mut().calls |= PERSONAL_CALL_FRAME_SUBMIT;

    if ptr < 0 || len < 0 {
        return Err(Trap::new(
            "personal shell frame submit has negative pointer or length",
        ));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > ui_frame::MAX_FRAME_BYTES {
        return Ok(ui_frame::FRAME_RESULT_LIMIT_EXCEEDED);
    }
    ptr.checked_add(len)
        .ok_or_else(|| Trap::new("personal shell frame submit pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("personal shell memory export missing"))?;
    let mut scratch = Vec::new();
    scratch.resize(len, 0);
    memory
        .read(&caller, ptr, &mut scratch)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;

    let viewport = caller.data().viewport;
    match ui_frame::validate_frame(&scratch, viewport) {
        Ok(_) => {
            caller.data_mut().pending_frame = Some(scratch);
            Ok(ui_frame::FRAME_RESULT_ACCEPTED)
        }
        Err(error) => Ok(error.result_code()),
    }
}
