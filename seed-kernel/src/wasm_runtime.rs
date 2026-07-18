use alloc::{boxed::Box, string::String, vec::Vec};
use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicU64, AtomicU8, Ordering},
};
use raios_core::{
    beyond_env_invocation::{
        BoundaryState, InvocationAuthority, InvocationLifecycle, PendingHostOperation,
        TerminalOutcome,
    },
    host_import_abi_v1::{
        host_import_abi_ordered_list_sha256, HostImportSignature, BEYOND_ENV_HOST_IMPORTS_V1,
        HOST_IMPORT_ABI_V1, HOST_IMPORT_ERROR_CLOSED, HOST_IMPORT_ERROR_INVALID_ARGUMENT,
        HOST_IMPORT_ERROR_INVALID_STATE, HOST_IMPORT_ERROR_LIMIT_EXCEEDED,
        HOST_IMPORT_ERROR_RESOURCE_BUSY, HOST_IMPORT_ERROR_TIMED_OUT, HOST_IMPORT_ERROR_TRANSPORT,
        NET_HOST_IMPORTS_V1,
    },
    personal_shell_abi::{
        PersonalShellContext, PersonalShellInput, SanitizedInputEvent, SanitizedInputKind,
        MAX_PROGRAM_CONTEXT_LEN,
    },
    project_runtime::{
        WORKSPACE_ENTRYPOINT, WORKSPACE_FUEL_BUDGET, WORKSPACE_MEMORY_LIMIT_BYTES,
        WORKSPACE_SERVICE_ID,
    },
    scoped_wasm_import_grant::{
        authorized_import_list_sha256, evaluate_evidence_bound_wasm_import_grant,
        evaluate_observed_wasm_import_grant, evaluate_personal_shell_import_grant,
        evaluate_wasm_import_grant, EvidenceBoundWasmImportGrantInput, ObservedWasmImports,
        PersonalShellImportGrantDecision, PersonalShellImportGrantInput, VerifiedImportEvidence,
        WasmImportGrantDecision, WasmImportGrantInput, PERSONAL_SHELL_SERVICE_ID,
        PERSONAL_SHELL_UI_IMPORTS,
    },
    sha256_bytes,
    ui_frame::{self, Viewport},
};
use spin::Mutex;
use wasmi::{
    core::{Trap, TrapCode},
    errors::{InstantiationError, LinkerError, MemoryError, TableError},
    Caller, Config, Engine, Extern, Instance, Linker, Memory, Module, ResumableCall,
    ResumableInvocation, Store, StoreLimits, StoreLimitsBuilder, Value,
};

use crate::{net, serial};

mod acquire_shims;
mod acquisition_service;
mod artifacts;
mod crypto_shims;
mod envelope;
mod invocation;
mod net_shims;
mod personal_shell;
mod probes;
mod suspension;
mod thread_job;
mod wasi_build_job;
mod wasi_build_storage;
mod wasi_preview1;

pub(crate) use acquire_shims::{
    acquire_shim_grant_probe, AcquireFixtureCase, AcquireFixtureProbeSnapshot,
    AcquireShimGrantProbe,
};
pub(crate) use acquisition_service::{
    acquisition_service_probe, observe_w7_blocks_provider, w7_acquisition_snapshot,
    AcquisitionServiceProbe, W7AcquisitionSnapshot, NET_ACQUIRE_W7_AUTHORIZED_IMPORTS,
    NET_ACQUIRE_W7_SERVICE_ID,
};
pub(crate) use artifacts::*;
pub(crate) use crypto_shims::{
    crypto_fixture_probe_snapshot, crypto_shim_grant_probe, crypto_shim_probe_snapshot,
    CryptoFixtureProbeSnapshot, CryptoShimCallEvidence, CryptoShimGrantProbe,
};
pub(crate) use envelope::{
    execute_module_bytes, execute_workspace_no_import_candidate, inspect_workspace_imports,
    EchoRunEvidence, NegativeRun, WorkspaceImportInspection,
};
pub(crate) use invocation::{
    beyond_env_probe_snapshot, request_beyond_env_fixture, request_w7_acquisition,
    request_w7_provider_busy_probe, start_beyond_env_fixture, take_beyond_env_fixture_request,
    ActiveBeyondEnvInvocation, BeyondEnvFixtureRequest, BeyondEnvLifecycleCase,
    BeyondEnvLifecycleSuite, BeyondEnvProbeSnapshot,
};
pub(crate) use net_shims::{
    net_shim_grant_probe, net_shim_probe_snapshot, NetShimGrantProbe, NetShimProbeSnapshot,
};
pub(crate) use personal_shell::*;
pub(crate) use probes::{
    forbidden_import_link_failure_evidence, run_acquire_fixture_probe,
    run_beyond_env_lifecycle_suite, run_bufecho_roundtrip, run_bufecho_unauthorized_probe,
    run_certspki_roundtrip, run_certspki_unauthorized_probe, run_certwindow_roundtrip,
    run_certwindow_unauthorized_probe, run_dnsparse_roundtrip, run_dnsparse_unauthorized_probe,
    run_echo_fuel_starved, run_echo_probe, run_echo_service, run_httphead_roundtrip,
    run_httphead_unauthorized_probe, BufechoRoundtripEvidence, EchoFuelStarvedEvidence, EchoProbe,
    WasmHardeningCase,
};
pub(crate) use thread_job::emit_threads_selftest;
pub(crate) use wasi_build_job::{emit_wasi_mem_selftest, emit_wasi_selftest};
