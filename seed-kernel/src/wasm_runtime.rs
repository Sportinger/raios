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

include!(concat!(env!("OUT_DIR"), "/echo_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/bufecho_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/certwindow_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/httphead_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/certspki_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/dnsparse_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/personal_shell_wasm_artifact.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/personal_shell_current_boot_load_descriptor.rs"
));

const EMPTY_WASM_MODULE: &[u8] = b"\0asm\x01\0\0\0";
const FORBIDDEN_WRITE_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x02, 0x17,
    0x01, 0x03, 0x65, 0x6e, 0x76, 0x0f, 0x66, 0x6f, 0x72, 0x62, 0x69, 0x64, 0x64, 0x65, 0x6e, 0x5f,
    0x77, 0x72, 0x69, 0x74, 0x65, 0x00, 0x00,
];
const MALFORMED_WASM_MODULE: &[u8] = b"\0bsm\x01\0\0\0";
const OVER_MEMORY_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x02,
];
const FUEL_LOOP_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x03, 0x02,
    0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00,
    0x03, 0x40, 0x0c, 0x00, 0x0b, 0x0b,
];
const UNREACHABLE_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x03, 0x02,
    0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x05, 0x01, 0x03, 0x00,
    0x00, 0x0b,
];
// Labeled NET-2 test infrastructure only. `test.suspend_once` never enters the
// known-import table or the production per-instance linker.
const SUSPEND_ONCE_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x02,
    0x15, 0x01, 0x04, 0x74, 0x65, 0x73, 0x74, 0x0c, 0x73, 0x75, 0x73, 0x70, 0x65, 0x6e, 0x64, 0x5f,
    0x6f, 0x6e, 0x63, 0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
    0x10, 0x02, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
    0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x10, 0x00, 0x0b,
];
// Labeled NET-4 test infrastructure only. The four closures are the real v1
// implementations, but this fixture is their sole linker until owner arming.
// It opens QEMU slirp DNS, sends one TCP DNS query, receives answer bytes, and
// closes twice to prove immediate owner-idempotent cleanup.
const NET_SHIM_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x11, 0x03, 0x60, 0x00, 0x01, 0x7f, 0x60,
    0x03, 0x7f, 0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x01, 0x7f, 0x01, 0x7f, 0x02, 0x3e, 0x04, 0x03, 0x6e,
    0x65, 0x74, 0x08, 0x74, 0x63, 0x70, 0x5f, 0x6f, 0x70, 0x65, 0x6e, 0x00, 0x00, 0x03, 0x6e, 0x65,
    0x74, 0x08, 0x74, 0x63, 0x70, 0x5f, 0x73, 0x65, 0x6e, 0x64, 0x00, 0x01, 0x03, 0x6e, 0x65, 0x74,
    0x08, 0x74, 0x63, 0x70, 0x5f, 0x72, 0x65, 0x63, 0x76, 0x00, 0x01, 0x03, 0x6e, 0x65, 0x74, 0x09,
    0x74, 0x63, 0x70, 0x5f, 0x63, 0x6c, 0x6f, 0x73, 0x65, 0x00, 0x02, 0x03, 0x02, 0x01, 0x00, 0x05,
    0x03, 0x01, 0x00, 0x01, 0x07, 0x10, 0x02, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x04, 0x06, 0x6d, 0x65,
    0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x2c, 0x01, 0x2a, 0x01, 0x01, 0x7f, 0x10, 0x00, 0x22,
    0x00, 0x41, 0x20, 0x41, 0x1f, 0x10, 0x01, 0x1a, 0x20, 0x00, 0x41, 0x80, 0x01, 0x41, 0x80, 0x04,
    0x10, 0x02, 0x1a, 0x20, 0x00, 0x10, 0x03, 0x1a, 0x20, 0x00, 0x10, 0x03, 0x1a, 0x41, 0x80, 0x01,
    0x2f, 0x01, 0x00, 0x0b, 0x0b, 0x25, 0x01, 0x00, 0x41, 0x20, 0x0b, 0x1f, 0x00, 0x1d, 0x12, 0x34,
    0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x65, 0x78, 0x61, 0x6d, 0x70,
    0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
];
const TAIL_SUSPEND_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x02,
    0x15, 0x01, 0x04, 0x74, 0x65, 0x73, 0x74, 0x0c, 0x73, 0x75, 0x73, 0x70, 0x65, 0x6e, 0x64, 0x5f,
    0x6f, 0x6e, 0x63, 0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
    0x10, 0x02, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
    0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x12, 0x00, 0x0b,
];
const NORMAL_RETURN_I32_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04,
    0x00, 0x41, 0x07, 0x0b,
];
const UNREACHABLE_I32_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x05, 0x01, 0x03,
    0x00, 0x00, 0x0b,
];
const FUEL_LOOP_I32_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x0b, 0x01, 0x09,
    0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
];
const MAX_WASM_LOG_BYTES: usize = 256;
// Keep in lockstep with the Phase-B guest buffer size.
const MAX_WASM_INPUT_BYTES: usize = 4096;
const MAX_WASM_OUTPUT_BYTES: usize = 4096;
const BUFFER_SERVICE_MAX_MEMORY_BYTES: usize = 2 * 1024 * 1024;
const WASM_MEMORY_PAGE_BYTES: usize = 64 * 1024;
pub(crate) const ECHO_WASM_FUEL_BUDGET: u64 = 10_000;
/// Deliberately-tiny fuel budget for the labeled fuel-starvation fault injection
/// (`run_echo_fuel_starved`). A real echo invoke under this budget exhausts fuel
/// after a single metered step and traps with a genuine wasmi `OutOfFuel`.
pub(crate) const ECHO_WASM_FUEL_STARVED_BUDGET: u64 = 1;
const FUEL_EXHAUSTION_BUDGET: u64 = 1;
const GUEST_TRAP_FUEL_BUDGET: u64 = 100;
const BEYOND_ENV_FIXTURE_FUEL_BUDGET: u64 = 1_000_000;
const BEYOND_ENV_WALL_BUDGET_MS: u64 = 90_000;
const BEYOND_ENV_PUMP_STEP_BUDGET: u32 = 11_250;
const NET_CALL_MAX_BYTES: usize = 4096;
const NET_TX_QUOTA_BYTES: u64 = 32 * 1024;
const NET_RX_QUOTA_BYTES: u64 = 320 * 1024;
const NET_CONNECT_TIMEOUT_MS: u64 = 5_000;
const NET_IDLE_TIMEOUT_MS: u64 = 15_000;
const NET_TOTAL_TIMEOUT_MS: u64 = 90_000;
const NET_RESPONSIVE_ADDRESS: smoltcp::wire::Ipv4Address =
    smoltcp::wire::Ipv4Address::new(10, 0, 2, 3);
const NET_SILENT_ADDRESS: smoltcp::wire::Ipv4Address =
    smoltcp::wire::Ipv4Address::new(10, 0, 2, 254);
const NET_DNS_PORT: u16 = 53;
pub(crate) const WASM_HARDENING_CASE_COUNT: usize = 4;
pub(crate) const FORBIDDEN_IMPORT_MODULE: &str = "env";
pub(crate) const FORBIDDEN_IMPORT_NAME: &str = "forbidden_write";
pub(crate) const ECHO_SERVICE_ID: &str = "svc.demo.echo";
/// Declared/known only in NET-1. No corresponding linker arm exists.
pub(crate) const WASM_HOST_IMPORT_ABI_V1: &str = HOST_IMPORT_ABI_V1;
pub(crate) const KNOWN_BEYOND_ENV_HOST_IMPORTS_V1: &[HostImportSignature] =
    BEYOND_ENV_HOST_IMPORTS_V1;
pub(crate) const ECHO_AUTHORIZED_IMPORTS: &[(&str, &str)] =
    &[("env", "log"), ("env", "counter_get")];
pub(crate) const BUFECHO_SERVICE_ID: &str = "svc.demo.bufecho";
pub(crate) const BUFECHO_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const CERTWINDOW_SERVICE_ID: &str = "svc.demo.certwindow";
pub(crate) const CERTWINDOW_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const CERTWINDOW_WASM_FUEL_BUDGET: u64 = 1_000_000;
pub(crate) const HTTPHEAD_SERVICE_ID: &str = "svc.demo.httphead";
pub(crate) const HTTPHEAD_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const HTTPHEAD_WASM_FUEL_BUDGET: u64 = 1_000_000;
pub(crate) const CERTSPKI_SERVICE_ID: &str = "svc.demo.certspki";
pub(crate) const CERTSPKI_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const CERTSPKI_WASM_FUEL_BUDGET: u64 = 1_000_000;
pub(crate) const DNSPARSE_SERVICE_ID: &str = "svc.demo.dnsparse";
pub(crate) const DNSPARSE_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const DNSPARSE_WASM_FUEL_BUDGET: u64 = 1_000_000;
pub(crate) const PERSONAL_SHELL_WASM_FUEL_BUDGET: u64 = 250_000;
const PERSONAL_SHELL_CONTEXT_BYTES: usize = 32;
const PERSONAL_SHELL_MAX_INPUT_BYTES: usize = 1_040;
const PERSONAL_SHELL_MAX_MEMORY_BYTES: usize = 2 * 1024 * 1024;
const PERSONAL_SHELL_TABLE_COUNT: usize = 1;
const PERSONAL_SHELL_TABLE_ELEMENTS: u32 = 2;
const ZERO_SHA256: [u8; 32] = [0; 32];

static CURRENT_BOOT_COUNTER: Mutex<u64> = Mutex::new(0);
static BEYOND_ENV_REQUEST: AtomicU8 = AtomicU8::new(0);
static NEXT_BEYOND_ENV_INVOCATION_ID: AtomicU64 = AtomicU64::new(1);
static BEYOND_ENV_PROBE: Mutex<BeyondEnvProbeSnapshot> =
    Mutex::new(BeyondEnvProbeSnapshot::empty());
static BEYOND_ENV_SUITE: Mutex<Option<BeyondEnvLifecycleSuite>> = Mutex::new(None);
static ACTIVE_DROP_TEARDOWN_COUNT: AtomicU64 = AtomicU64::new(0);
static ACTIVE_BEYOND_ENV_INVOCATION: AtomicU8 = AtomicU8::new(0);
static NET_SHIM_PROBE: Mutex<NetShimProbeSnapshot> = Mutex::new(NetShimProbeSnapshot::empty());

#[derive(Clone, Copy, Debug)]
struct HostSuspend {
    invocation_id: u64,
    operation_id: u32,
}

impl fmt::Display for HostSuspend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("raios host operation suspended")
    }
}

impl wasmi::core::HostError for HostSuspend {}

#[derive(Clone, Copy, Debug)]
struct FixtureHostError;

impl fmt::Display for FixtureHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("raios labeled fixture host error")
    }
}

impl wasmi::core::HostError for FixtureHostError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FixtureBehavior {
    Suspend,
    UnrecognizedHostError,
    HostFuelError,
    Net(NetFixtureScenario),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NetFixtureScenario {
    Responsive,
    SilentTimeout,
    SilentKill,
}

impl NetFixtureScenario {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Responsive => "responsive",
            Self::SilentTimeout => "silent_timeout",
            Self::SilentKill => "silent_kill",
        }
    }

    const fn address(self) -> smoltcp::wire::Ipv4Address {
        match self {
            Self::Responsive => NET_RESPONSIVE_ADDRESS,
            Self::SilentTimeout | Self::SilentKill => NET_SILENT_ADDRESS,
        }
    }
}

#[derive(Clone, Copy)]
enum PendingNetworkKind {
    Open,
    Send { len: usize },
    Recv { ptr: usize, cap: usize },
}

struct PendingNetworkOperation {
    owner_invocation_id: u64,
    operation: PendingHostOperation,
    deadline_ms: u64,
    kind: PendingNetworkKind,
    buffer: [u8; NET_CALL_MAX_BYTES],
}

struct NetCompletion {
    result: i32,
    deadline_ms: u64,
    kind: PendingNetworkKind,
    recv_ptr: Option<usize>,
    recv_len: usize,
    buffer: [u8; NET_CALL_MAX_BYTES],
}

struct NetInvocationState {
    scenario: NetFixtureScenario,
    owner: net::TransportOwner,
    lease: Option<net::TransportLeaseToken>,
    pending: Option<PendingNetworkOperation>,
    next_operation_id: u32,
    tx_bytes: u64,
    rx_bytes: u64,
    first_negative_result: Option<i32>,
    connection_opened: bool,
    close_call_count: u32,
    would_block_guest_visible: bool,
}

impl NetInvocationState {
    fn new(scenario: NetFixtureScenario, invocation_id: u64) -> Self {
        Self {
            scenario,
            owner: net::TransportOwner::new(3, invocation_id),
            lease: None,
            pending: None,
            next_operation_id: 1,
            tx_bytes: 0,
            rx_bytes: 0,
            first_negative_result: None,
            connection_opened: false,
            close_call_count: 0,
            would_block_guest_visible: false,
        }
    }

    fn next_operation(&mut self) -> u32 {
        let operation_id = self.next_operation_id;
        self.next_operation_id = self.next_operation_id.saturating_add(1);
        operation_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BeyondEnvFixtureRequest {
    HoldForKill,
    ResumeToFinish,
    NetResponsive,
    NetSilentTimeout,
    NetSilentKill,
}

impl BeyondEnvFixtureRequest {
    const fn encode(self) -> u8 {
        match self {
            Self::HoldForKill => 1,
            Self::ResumeToFinish => 2,
            Self::NetResponsive => 3,
            Self::NetSilentTimeout => 4,
            Self::NetSilentKill => 5,
        }
    }

    const fn decode(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::HoldForKill),
            2 => Some(Self::ResumeToFinish),
            3 => Some(Self::NetResponsive),
            4 => Some(Self::NetSilentTimeout),
            5 => Some(Self::NetSilentKill),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NetShimProbeSnapshot {
    pub(crate) request_status: &'static str,
    pub(crate) active: bool,
    pub(crate) scenario: &'static str,
    pub(crate) invocation_id: u64,
    pub(crate) suspended: bool,
    pub(crate) outcome: &'static str,
    pub(crate) suspension_count: u32,
    pub(crate) resume_count: u32,
    pub(crate) teardown_count: u32,
    pub(crate) teardown_complete: bool,
    pub(crate) no_resume_after_kill: bool,
    pub(crate) lease_held: bool,
    pub(crate) open_operation_id: u32,
    pub(crate) send_operation_id: u32,
    pub(crate) recv_operation_id: u32,
    pub(crate) first_close_operation_id: u32,
    pub(crate) second_close_operation_id: u32,
    pub(crate) close_call_count: u32,
    pub(crate) tx_bytes: u64,
    pub(crate) rx_bytes: u64,
    pub(crate) rx_sha256: [u8; 32],
    pub(crate) received_prefix_le: u64,
    pub(crate) guest_return_value: u64,
    pub(crate) dns_tcp_length_prefix_present: bool,
    pub(crate) would_block_guest_visible: bool,
    pub(crate) responsive_run_count: u64,
    pub(crate) timeout_run_count: u64,
    pub(crate) killed_run_count: u64,
    pub(crate) terminal_boot_ms: u64,
    pub(crate) kill_observed_boot_ms: u64,
}

impl NetShimProbeSnapshot {
    const fn empty() -> Self {
        Self {
            request_status: "idle",
            active: false,
            scenario: "not_run",
            invocation_id: 0,
            suspended: false,
            outcome: "not_run",
            suspension_count: 0,
            resume_count: 0,
            teardown_count: 0,
            teardown_complete: false,
            no_resume_after_kill: false,
            lease_held: false,
            open_operation_id: 0,
            send_operation_id: 0,
            recv_operation_id: 0,
            first_close_operation_id: 0,
            second_close_operation_id: 0,
            close_call_count: 0,
            tx_bytes: 0,
            rx_bytes: 0,
            rx_sha256: ZERO_SHA256,
            received_prefix_le: 0,
            guest_return_value: 0,
            dns_tcp_length_prefix_present: false,
            would_block_guest_visible: false,
            responsive_run_count: 0,
            timeout_run_count: 0,
            killed_run_count: 0,
            terminal_boot_ms: 0,
            kill_observed_boot_ms: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NetShimGrantProbe {
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) import_list_sha256: [u8; 32],
    pub(crate) policy_denial_reason: &'static str,
    pub(crate) exact_list_drift_reason: &'static str,
    pub(crate) linker_drift_reason: &'static str,
    pub(crate) requested_import_count: u64,
    pub(crate) denied_before_instantiation: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct BeyondEnvProbeSnapshot {
    pub(crate) request_status: &'static str,
    pub(crate) active: bool,
    pub(crate) invocation_id: u64,
    pub(crate) run_count: u64,
    pub(crate) suspended: bool,
    pub(crate) suspended_boot_ms: u64,
    pub(crate) outcome: &'static str,
    pub(crate) terminal_boot_ms: u64,
    pub(crate) kill_observed_boot_ms: u64,
    pub(crate) suspension_count: u32,
    pub(crate) resume_count: u32,
    pub(crate) teardown_count: u32,
    pub(crate) fuel_used: u64,
    pub(crate) no_resume_after_kill: bool,
    pub(crate) teardown_complete: bool,
    pub(crate) handles_invalid: bool,
    pub(crate) lease_held: bool,
    pub(crate) pending_acquisition_present: bool,
    pub(crate) prior_candidate_unchanged: bool,
    pub(crate) killed_run_count: u64,
    pub(crate) finished_run_count: u64,
}

impl BeyondEnvProbeSnapshot {
    const fn empty() -> Self {
        Self {
            request_status: "idle",
            active: false,
            invocation_id: 0,
            run_count: 0,
            suspended: false,
            suspended_boot_ms: 0,
            outcome: "not_run",
            terminal_boot_ms: 0,
            kill_observed_boot_ms: 0,
            suspension_count: 0,
            resume_count: 0,
            teardown_count: 0,
            fuel_used: 0,
            no_resume_after_kill: false,
            teardown_complete: false,
            handles_invalid: true,
            lease_held: false,
            pending_acquisition_present: false,
            prior_candidate_unchanged: true,
            killed_run_count: 0,
            finished_run_count: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BeyondEnvLifecycleCase {
    pub(crate) name: &'static str,
    pub(crate) outcome: &'static str,
    pub(crate) suspended: bool,
    pub(crate) suspension_count: u32,
    pub(crate) resume_count: u32,
    pub(crate) teardown_count: u32,
    pub(crate) teardown_complete: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct BeyondEnvLifecycleSuite {
    pub(crate) cases: [BeyondEnvLifecycleCase; 7],
    pub(crate) tail_call_terminal: bool,
    pub(crate) busy_loop_out_of_fuel: bool,
    pub(crate) busy_loop_wall_ms: u64,
    pub(crate) fuel_budget: u64,
    pub(crate) wall_budget_ms: u64,
    pub(crate) pump_step_budget: u32,
}

struct BeyondEnvState {
    lifecycle: InvocationLifecycle,
    behavior: FixtureBehavior,
    net: Option<NetInvocationState>,
    limits: StoreLimits,
}

pub(crate) struct ActiveBeyondEnvInvocation {
    store: Option<Store<BeyondEnvState>>,
    instance: Option<Instance>,
    memory: Option<Memory>,
    continuation: Option<ResumableInvocation>,
    outputs: [Value; 1],
    auto_resume: bool,
    closed: bool,
}

pub(crate) fn request_beyond_env_fixture(request: BeyondEnvFixtureRequest) -> &'static str {
    if wasm_execution_busy() {
        return "resource_busy_active_invocation";
    }
    match BEYOND_ENV_REQUEST.compare_exchange(
        0,
        request.encode(),
        Ordering::AcqRel,
        Ordering::Acquire,
    ) {
        Ok(_) => {
            if matches!(
                request,
                BeyondEnvFixtureRequest::NetResponsive
                    | BeyondEnvFixtureRequest::NetSilentTimeout
                    | BeyondEnvFixtureRequest::NetSilentKill
            ) {
                NET_SHIM_PROBE.lock().request_status = "accepted_pending";
            } else {
                BEYOND_ENV_PROBE.lock().request_status = "accepted_pending";
            }
            "accepted_pending"
        }
        Err(_) => "resource_busy_active_invocation",
    }
}

pub(crate) fn net_shim_probe_snapshot() -> NetShimProbeSnapshot {
    *NET_SHIM_PROBE.lock()
}

pub(crate) fn net_shim_grant_probe() -> NetShimGrantProbe {
    const SERVICE_ID: &str = "test.fixture.net_shims.signed";
    const DESCRIPTOR_EVIDENCE_SHA256: [u8; 32] = [0x45; 32];
    const ARTIFACT_EVIDENCE_SHA256: [u8; 32] = [0x46; 32];
    const GRANT_EVIDENCE_SHA256: [u8; 32] = [0x47; 32];
    const NET_IMPORTS: &[(&str, &str)] = &[
        ("net", "tcp_open"),
        ("net", "tcp_send"),
        ("net", "tcp_recv"),
        ("net", "tcp_close"),
    ];
    const NET_IMPORTS_DRIFTED: &[(&str, &str)] = &[
        ("net", "tcp_open"),
        ("net", "tcp_send"),
        ("net", "tcp_close"),
    ];

    let artifact_sha256 = sha256_bytes(NET_SHIM_WASM_MODULE);
    let import_list_sha256 = host_import_abi_ordered_list_sha256(HOST_IMPORT_ABI_V1, NET_IMPORTS);
    let evidence = |evidence_sha256| VerifiedImportEvidence {
        evidence_sha256,
        artifact_sha256,
        import_list_sha256,
    };
    let observed = ObservedWasmImports {
        artifact_sha256,
        import_list_sha256,
        imports: NET_IMPORTS,
    };
    let input = |observed_imports: ObservedWasmImports<'static>,
                 linker_implementations: &'static [(&'static str, &'static str)],
                 policy_allows_beyond_env|
     -> EvidenceBoundWasmImportGrantInput<'static> {
        EvidenceBoundWasmImportGrantInput {
            service_id: Some(SERVICE_ID),
            artifact_sha256: Some(artifact_sha256),
            host_import_abi: Some(HOST_IMPORT_ABI_V1),
            declared_import_list_sha256: Some(import_list_sha256),
            requested_imports: NET_IMPORTS,
            descriptor_source_signature_evidence: Some(evidence(DESCRIPTOR_EVIDENCE_SHA256)),
            artifact_signature_attestation_evidence: Some(evidence(ARTIFACT_EVIDENCE_SHA256)),
            computed_grant_evidence: Some(evidence(GRANT_EVIDENCE_SHA256)),
            observed_imports: Some(observed_imports),
            linker_implementations,
            policy_allows_beyond_env,
        }
    };
    let policy = evaluate_evidence_bound_wasm_import_grant(&input(observed, NET_IMPORTS, false));
    let exact_list = evaluate_evidence_bound_wasm_import_grant(&input(
        ObservedWasmImports {
            imports: NET_IMPORTS_DRIFTED,
            ..observed
        },
        NET_IMPORTS,
        false,
    ));
    // Test-only evaluator reachability for the NET-1 linker-drift negative.
    // No Store, instance, linker, or production policy input is created here.
    let linker =
        evaluate_evidence_bound_wasm_import_grant(&input(observed, NET_IMPORTS_DRIFTED, true));
    NetShimGrantProbe {
        artifact_sha256,
        import_list_sha256,
        policy_denial_reason: policy.reason,
        exact_list_drift_reason: exact_list.reason,
        linker_drift_reason: linker.reason,
        requested_import_count: NET_HOST_IMPORTS_V1.len() as u64,
        denied_before_instantiation: !policy.performed
            && !exact_list.performed
            && !linker.performed,
    }
}

fn wasm_execution_busy() -> bool {
    ACTIVE_BEYOND_ENV_INVOCATION.load(Ordering::Acquire) != 0
}

pub(crate) fn take_beyond_env_fixture_request() -> Option<BeyondEnvFixtureRequest> {
    BeyondEnvFixtureRequest::decode(BEYOND_ENV_REQUEST.swap(0, Ordering::AcqRel))
}

pub(crate) fn beyond_env_probe_snapshot() -> BeyondEnvProbeSnapshot {
    *BEYOND_ENV_PROBE.lock()
}

pub(crate) fn start_beyond_env_fixture(
    request: BeyondEnvFixtureRequest,
    now_ms: u64,
    kill_generation: u64,
) -> Option<ActiveBeyondEnvInvocation> {
    let active = try_start_beyond_env_fixture(request, now_ms, kill_generation);
    if active.is_none() {
        if matches!(
            request,
            BeyondEnvFixtureRequest::NetResponsive
                | BeyondEnvFixtureRequest::NetSilentTimeout
                | BeyondEnvFixtureRequest::NetSilentKill
        ) {
            let mut probe = NET_SHIM_PROBE.lock();
            probe.request_status = "fixture_setup_failed";
            probe.active = false;
            probe.outcome = "fixture_setup_failed";
            serial::write_fmt(format_args!(
                "RAIOS_NET_SHIM outcome=fixture_setup_failed teardown_complete=true boot_ms={}\r\n",
                now_ms
            ));
        } else {
            let mut probe = BEYOND_ENV_PROBE.lock();
            probe.request_status = "fixture_setup_failed";
            probe.active = false;
            probe.outcome = "fixture_setup_failed";
            serial::write_fmt(format_args!(
                "RAIOS_BEYOND_ENV_LIFECYCLE outcome=fixture_setup_failed teardown_complete=true boot_ms={}\r\n",
                now_ms
            ));
        }
    }
    active
}

fn try_start_beyond_env_fixture(
    request: BeyondEnvFixtureRequest,
    now_ms: u64,
    kill_generation: u64,
) -> Option<ActiveBeyondEnvInvocation> {
    let invocation_id = NEXT_BEYOND_ENV_INVOCATION_ID.fetch_add(1, Ordering::Relaxed);
    let (service_id, behavior, module_bytes) = match request {
        BeyondEnvFixtureRequest::HoldForKill | BeyondEnvFixtureRequest::ResumeToFinish => (
            "test.fixture.beyond_env_lifecycle",
            FixtureBehavior::Suspend,
            SUSPEND_ONCE_WASM_MODULE,
        ),
        BeyondEnvFixtureRequest::NetResponsive => (
            "test.fixture.net_shims",
            FixtureBehavior::Net(NetFixtureScenario::Responsive),
            NET_SHIM_WASM_MODULE,
        ),
        BeyondEnvFixtureRequest::NetSilentTimeout => (
            "test.fixture.net_shims",
            FixtureBehavior::Net(NetFixtureScenario::SilentTimeout),
            NET_SHIM_WASM_MODULE,
        ),
        BeyondEnvFixtureRequest::NetSilentKill => (
            "test.fixture.net_shims",
            FixtureBehavior::Net(NetFixtureScenario::SilentKill),
            NET_SHIM_WASM_MODULE,
        ),
    };
    let authority = InvocationAuthority {
        service_id,
        invocation_id,
        service_generation: 1,
        instance_generation: 1,
        captured_kill_generation: kill_generation,
        policy_allows_beyond_env: false,
    };
    let engine = beyond_env_engine();
    let module = Module::new(&engine, module_bytes).ok()?;
    let net_state = match behavior {
        FixtureBehavior::Net(scenario) => Some(NetInvocationState::new(scenario, invocation_id)),
        _ => None,
    };
    let mut store = Store::new(
        &engine,
        BeyondEnvState {
            lifecycle: InvocationLifecycle::new(
                authority,
                now_ms,
                BEYOND_ENV_WALL_BUDGET_MS,
                BEYOND_ENV_PUMP_STEP_BUDGET,
                invocation_id as u32,
            ),
            behavior,
            net: net_state,
            limits: beyond_env_limits(),
        },
    );
    store.limiter(|state| &mut state.limits);
    store.add_fuel(BEYOND_ENV_FIXTURE_FUEL_BUDGET).ok()?;
    let mut linker = Linker::<BeyondEnvState>::new(&engine);
    match behavior {
        FixtureBehavior::Net(_) => {
            linker
                .func_wrap("net", "tcp_open", host_net_tcp_open)
                .ok()?;
            linker
                .func_wrap("net", "tcp_send", host_net_tcp_send)
                .ok()?;
            linker
                .func_wrap("net", "tcp_recv", host_net_tcp_recv)
                .ok()?;
            linker
                .func_wrap("net", "tcp_close", host_net_tcp_close)
                .ok()?;
        }
        _ => {
            linker
                .func_wrap("test", "suspend_once", host_test_suspend_once)
                .ok()?;
        }
    }
    let instance = linker
        .instantiate(&mut store, &module)
        .ok()?
        .start(&mut store)
        .ok()?;
    let memory = instance
        .get_export(&store, "memory")
        .and_then(Extern::into_memory)?;
    let function = instance.get_export(&store, "run")?.into_func()?;
    let mut outputs = [Value::I32(0)];
    let continuation = match function.call_resumable(&mut store, &[], &mut outputs) {
        Ok(ResumableCall::Resumable(invocation))
            if classify_resumable(&invocation, &store).is_ok() =>
        {
            invocation
        }
        Ok(ResumableCall::Resumable(_)) => {
            finish_store(&mut store, TerminalOutcome::HostError);
            return None;
        }
        Ok(ResumableCall::Finished) => {
            finish_store(&mut store, TerminalOutcome::Finished);
            return None;
        }
        Err(error) => {
            finish_store(&mut store, terminal_from_error(&error));
            return None;
        }
    };
    let suspended_boot_ms = crate::time::rdtsc() / crate::time::tsc_per_ms().max(1);
    if let FixtureBehavior::Net(scenario) = behavior {
        let mut probe = NET_SHIM_PROBE.lock();
        let counts = (
            probe.responsive_run_count,
            probe.timeout_run_count,
            probe.killed_run_count,
        );
        *probe = NetShimProbeSnapshot::empty();
        probe.responsive_run_count = counts.0;
        probe.timeout_run_count = counts.1;
        probe.killed_run_count = counts.2;
        probe.request_status = "running";
        probe.active = true;
        probe.scenario = scenario.as_str();
        probe.invocation_id = invocation_id;
        probe.suspended = true;
        probe.outcome = "pending";
        probe.suspension_count = store.data().lifecycle.suspension_count();
        probe.open_operation_id = store
            .data()
            .lifecycle
            .pending()
            .map_or(0, |pending| pending.operation_id);
        probe.lease_held = store
            .data()
            .net
            .as_ref()
            .is_some_and(|net| net.lease.is_some());
        serial::write_fmt(format_args!(
            "RAIOS_NET_SHIM suspended=true scenario={} operation=tcp_open invocation_id={} boot_ms={}\r\n",
            scenario.as_str(), invocation_id, suspended_boot_ms
        ));
    } else {
        let mut probe = BEYOND_ENV_PROBE.lock();
        probe.request_status = "running";
        probe.active = true;
        probe.invocation_id = invocation_id;
        probe.run_count = probe.run_count.saturating_add(1);
        probe.suspended = true;
        probe.suspended_boot_ms = suspended_boot_ms;
        probe.outcome = "pending";
        probe.terminal_boot_ms = 0;
        probe.kill_observed_boot_ms = 0;
        probe.suspension_count = store.data().lifecycle.suspension_count();
        probe.resume_count = 0;
        probe.teardown_count = 0;
        probe.teardown_complete = false;
        probe.no_resume_after_kill = false;
        serial::write_fmt(format_args!(
            "RAIOS_BEYOND_ENV_LIFECYCLE suspended=true invocation_id={} boot_ms={}\r\n",
            invocation_id, suspended_boot_ms
        ));
    }
    ACTIVE_BEYOND_ENV_INVOCATION.store(1, Ordering::Release);
    Some(ActiveBeyondEnvInvocation {
        store: Some(store),
        instance: Some(instance),
        memory: Some(memory),
        continuation: Some(continuation),
        outputs,
        auto_resume: request == BeyondEnvFixtureRequest::ResumeToFinish,
        closed: false,
    })
}

impl ActiveBeyondEnvInvocation {
    pub(crate) fn pump(&mut self, now_ms: u64, kill_generation: u64, kill_boot_ms: u64) -> bool {
        if self.closed {
            return true;
        }
        let boundary = BoundaryState {
            now_ms,
            kill_generation,
            service_generation: 1,
            instance_generation: 1,
            posture_allows_execution: true,
        };
        let boundary_result = self
            .store
            .as_mut()
            .expect("active beyond-env store missing")
            .data_mut()
            .lifecycle
            .check_boundary(boundary);
        if let Err(outcome) = boundary_result {
            self.finish(outcome, now_ms, kill_boot_ms);
            return true;
        }
        if matches!(
            self.store
                .as_ref()
                .expect("active beyond-env store missing")
                .data()
                .behavior,
            FixtureBehavior::Net(_)
        ) {
            return self.pump_net(now_ms, kill_boot_ms, boundary);
        }
        if !self.auto_resume {
            return false;
        }

        let store = self
            .store
            .as_mut()
            .expect("active beyond-env store missing");
        if store.data_mut().lifecycle.take_pending().is_none() {
            self.finish(TerminalOutcome::HostError, now_ms, 0);
            return true;
        }
        if let Err(outcome) = store.data_mut().lifecycle.check_boundary(boundary) {
            self.finish(outcome, now_ms, kill_boot_ms);
            return true;
        }
        store.data_mut().lifecycle.note_resume();
        let continuation = self
            .continuation
            .take()
            .expect("active beyond-env continuation missing");
        match continuation.resume(&mut *store, &[Value::I32(7)], &mut self.outputs) {
            Ok(ResumableCall::Finished) => self.finish(TerminalOutcome::Finished, now_ms, 0),
            Ok(ResumableCall::Resumable(next)) => match classify_resumable(&next, store) {
                Ok(()) => {
                    self.continuation = Some(next);
                    return false;
                }
                Err(outcome) => self.finish(outcome, now_ms, 0),
            },
            Err(error) => self.finish(terminal_from_error(&error), now_ms, 0),
        }
        true
    }

    fn pump_net(&mut self, now_ms: u64, kill_boot_ms: u64, boundary: BoundaryState) -> bool {
        let completion = {
            let store = self
                .store
                .as_mut()
                .expect("active beyond-env store missing");
            match progress_net_operation(store, now_ms) {
                Ok(Some(completion)) => completion,
                Ok(None) => return false,
                Err(outcome) => {
                    self.finish(outcome, now_ms, kill_boot_ms);
                    return true;
                }
            }
        };
        let store = self
            .store
            .as_mut()
            .expect("active beyond-env store missing");
        if let Err(outcome) = check_net_resume_boundary(store, &completion, now_ms) {
            self.finish(outcome, now_ms, kill_boot_ms);
            return true;
        }
        let Some(pending) = store.data_mut().lifecycle.take_pending() else {
            self.finish(TerminalOutcome::HostError, now_ms, 0);
            return true;
        };
        if pending.invocation_id != store.data().lifecycle.authority().invocation_id {
            self.finish(TerminalOutcome::HostError, now_ms, 0);
            return true;
        }
        if let Err(outcome) = store.data_mut().lifecycle.check_boundary(boundary) {
            self.finish(outcome, now_ms, kill_boot_ms);
            return true;
        }
        if let Some(ptr) = completion.recv_ptr {
            let memory = self.memory.expect("active beyond-env memory missing");
            if memory
                .write(&mut *store, ptr, &completion.buffer[..completion.recv_len])
                .is_err()
            {
                self.finish(TerminalOutcome::HostError, now_ms, 0);
                return true;
            }
        }
        store.data_mut().lifecycle.note_resume();
        let continuation = self
            .continuation
            .take()
            .expect("active beyond-env continuation missing");
        match continuation.resume(
            &mut *store,
            &[Value::I32(completion.result)],
            &mut self.outputs,
        ) {
            Ok(ResumableCall::Finished) => self.finish(TerminalOutcome::Finished, now_ms, 0),
            Ok(ResumableCall::Resumable(next)) => match classify_resumable(&next, store) {
                Ok(()) => {
                    self.continuation = Some(next);
                    let mut probe = NET_SHIM_PROBE.lock();
                    probe.suspended = true;
                    probe.suspension_count = store.data().lifecycle.suspension_count();
                    probe.resume_count = store.data().lifecycle.resume_count();
                    probe.lease_held = store
                        .data()
                        .net
                        .as_ref()
                        .is_some_and(|net| net.lease.is_some());
                    return false;
                }
                Err(outcome) => self.finish(outcome, now_ms, 0),
            },
            Err(error) => self.finish(terminal_from_error(&error), now_ms, 0),
        }
        true
    }

    fn finish(&mut self, outcome: TerminalOutcome, now_ms: u64, kill_boot_ms: u64) {
        let store = self
            .store
            .as_mut()
            .expect("active beyond-env store missing");
        let net_terminal = finish_net_resources(store, now_ms);
        finish_store(store, outcome);
        let lifecycle = &store.data().lifecycle;
        let receipt = lifecycle.teardown_receipt();
        let suspension_count = lifecycle.suspension_count();
        let resume_count = lifecycle.resume_count();
        let fuel_used = store.fuel_consumed().unwrap_or(0);
        let guest_return_value = self.outputs[0].i32().unwrap_or(0).max(0) as u64;

        // Teardown step 9: no raiOS lock is held while wasmi state is dropped.
        drop(self.continuation.take());
        drop(self.memory.take());
        drop(self.instance.take());
        drop(self.store.take());
        self.closed = true;

        if let Some(net_terminal) = net_terminal {
            let mut probe = NET_SHIM_PROBE.lock();
            probe.request_status = "complete";
            probe.active = false;
            probe.suspended = false;
            probe.outcome = if outcome == TerminalOutcome::Killed {
                "killed"
            } else {
                match net_terminal.first_negative_result {
                    Some(HOST_IMPORT_ERROR_TIMED_OUT) => "timed_out",
                    Some(_) => "transport_error",
                    None => "finished",
                }
            };
            probe.terminal_boot_ms = now_ms;
            probe.kill_observed_boot_ms = kill_boot_ms;
            probe.suspension_count = suspension_count;
            probe.resume_count = resume_count;
            probe.teardown_count = receipt.teardown_count;
            probe.teardown_complete = receipt.terminal_outcome_recorded;
            probe.no_resume_after_kill = outcome == TerminalOutcome::Killed && resume_count == 0;
            probe.lease_held = false;
            probe.close_call_count = net_terminal.close_call_count;
            probe.tx_bytes = net_terminal.tx_bytes;
            probe.rx_bytes = net_terminal.rx_bytes;
            probe.guest_return_value = guest_return_value;
            probe.would_block_guest_visible = net_terminal.would_block_guest_visible;
            match probe.outcome {
                "finished" => {
                    probe.responsive_run_count = probe.responsive_run_count.saturating_add(1)
                }
                "timed_out" => probe.timeout_run_count = probe.timeout_run_count.saturating_add(1),
                "killed" => probe.killed_run_count = probe.killed_run_count.saturating_add(1),
                _ => {}
            }
            serial::write_fmt(format_args!(
                "RAIOS_NET_SHIM outcome={} scenario={} teardown_complete=true resume_count={} boot_ms={}\r\n",
                probe.outcome, probe.scenario, resume_count, now_ms
            ));
        } else {
            let mut probe = BEYOND_ENV_PROBE.lock();
            probe.request_status = "complete";
            probe.active = false;
            probe.suspended = false;
            probe.outcome = outcome.as_str();
            probe.terminal_boot_ms = now_ms;
            probe.kill_observed_boot_ms = kill_boot_ms;
            probe.suspension_count = suspension_count;
            probe.resume_count = resume_count;
            probe.teardown_count = receipt.teardown_count;
            probe.fuel_used = fuel_used;
            if outcome == TerminalOutcome::Killed {
                probe.no_resume_after_kill = resume_count == 0;
            }
            probe.teardown_complete = receipt.terminal_outcome_recorded;
            probe.handles_invalid = receipt.handles_invalid;
            probe.lease_held = false;
            probe.pending_acquisition_present = false;
            probe.prior_candidate_unchanged = receipt.prior_candidate_unchanged;
            if outcome == TerminalOutcome::Killed {
                probe.killed_run_count = probe.killed_run_count.saturating_add(1);
            }
            if outcome == TerminalOutcome::Finished {
                probe.finished_run_count = probe.finished_run_count.saturating_add(1);
            }
            serial::write_fmt(format_args!(
                "RAIOS_BEYOND_ENV_LIFECYCLE outcome={} teardown_complete=true handles_invalid=true resume_count={} boot_ms={}\r\n",
                outcome.as_str(), resume_count, now_ms
            ));
        }
        ACTIVE_BEYOND_ENV_INVOCATION.store(0, Ordering::Release);
    }
}

struct NetTerminalState {
    tx_bytes: u64,
    rx_bytes: u64,
    first_negative_result: Option<i32>,
    close_call_count: u32,
    would_block_guest_visible: bool,
}

fn progress_net_operation(
    store: &mut Store<BeyondEnvState>,
    now_ms: u64,
) -> Result<Option<NetCompletion>, TerminalOutcome> {
    let mut pending = store
        .data_mut()
        .net
        .as_mut()
        .and_then(|net| net.pending.take())
        .ok_or(TerminalOutcome::HostError)?;
    if pending.owner_invocation_id != store.data().lifecycle.authority().invocation_id
        || store.data().lifecycle.pending() != Some(pending.operation)
    {
        return Err(TerminalOutcome::HostError);
    }
    if now_ms >= pending.deadline_ms {
        let token = store.data_mut().net.as_mut().and_then(|net| {
            net.connection_opened = false;
            net.lease.take()
        });
        if let Some(token) = token {
            let _ = net::tcp_abort(token, now_ms);
        }
        return Ok(Some(complete_net_operation(
            store,
            pending,
            HOST_IMPORT_ERROR_TIMED_OUT,
            None,
            0,
        )));
    }

    match pending.kind {
        PendingNetworkKind::Open => {
            let (scenario, owner, token) = {
                let net_state = store
                    .data()
                    .net
                    .as_ref()
                    .ok_or(TerminalOutcome::HostError)?;
                (net_state.scenario, net_state.owner, net_state.lease)
            };
            let token = match token {
                Some(token) => token,
                None => match net::tcp_claim(owner, now_ms, NET_TOTAL_TIMEOUT_MS) {
                    Ok(token) => {
                        store
                            .data_mut()
                            .net
                            .as_mut()
                            .expect("net state missing")
                            .lease = Some(token);
                        NET_SHIM_PROBE.lock().lease_held = true;
                        token
                    }
                    Err(error) => {
                        return Ok(Some(complete_net_operation(
                            store,
                            pending,
                            transport_error_result(error),
                            None,
                            0,
                        )))
                    }
                },
            };
            match net::tcp_connect_step(token, scenario.address(), NET_DNS_PORT, now_ms) {
                Ok(net::TcpConnectResult::Started | net::TcpConnectResult::Connecting(_)) => {
                    store
                        .data_mut()
                        .net
                        .as_mut()
                        .expect("net state missing")
                        .pending = Some(pending);
                    Ok(None)
                }
                Ok(net::TcpConnectResult::Connected) => {
                    store
                        .data_mut()
                        .net
                        .as_mut()
                        .expect("net state missing")
                        .connection_opened = true;
                    let handle = store.data().lifecycle.handle();
                    Ok(Some(complete_net_operation(
                        store, pending, handle, None, 0,
                    )))
                }
                Ok(
                    net::TcpConnectResult::NetworkUnavailable
                    | net::TcpConnectResult::NetworkUnconfigured
                    | net::TcpConnectResult::ConnectError,
                ) => Ok(Some(complete_net_operation(
                    store,
                    pending,
                    HOST_IMPORT_ERROR_TRANSPORT,
                    None,
                    0,
                ))),
                Err(_) => Err(TerminalOutcome::GenerationInvalidated),
            }
        }
        PendingNetworkKind::Send { len } => {
            if store
                .data()
                .net
                .as_ref()
                .is_none_or(|net| net.tx_bytes.saturating_add(len as u64) > NET_TX_QUOTA_BYTES)
            {
                return Ok(Some(complete_net_operation(
                    store,
                    pending,
                    HOST_IMPORT_ERROR_LIMIT_EXCEEDED,
                    None,
                    0,
                )));
            }
            let token = store
                .data()
                .net
                .as_ref()
                .and_then(|net| net.lease)
                .ok_or(TerminalOutcome::GenerationInvalidated)?;
            match net::tcp_send_step(token, &pending.buffer[..len], now_ms) {
                Ok(net::TcpIoResult::WouldBlock) => {
                    store
                        .data_mut()
                        .net
                        .as_mut()
                        .expect("net state missing")
                        .pending = Some(pending);
                    Ok(None)
                }
                Ok(net::TcpIoResult::Ready(written)) => Ok(Some(complete_net_operation(
                    store,
                    pending,
                    written as i32,
                    None,
                    0,
                ))),
                Ok(net::TcpIoResult::Closed) => Ok(Some(complete_net_operation(
                    store,
                    pending,
                    HOST_IMPORT_ERROR_CLOSED,
                    None,
                    0,
                ))),
                Ok(net::TcpIoResult::Unavailable) => Ok(Some(complete_net_operation(
                    store,
                    pending,
                    HOST_IMPORT_ERROR_TRANSPORT,
                    None,
                    0,
                ))),
                Err(_) => Err(TerminalOutcome::GenerationInvalidated),
            }
        }
        PendingNetworkKind::Recv { ptr, cap } => {
            if store
                .data()
                .net
                .as_ref()
                .is_none_or(|net| net.rx_bytes.saturating_add(cap as u64) > NET_RX_QUOTA_BYTES)
            {
                return Ok(Some(complete_net_operation(
                    store,
                    pending,
                    HOST_IMPORT_ERROR_LIMIT_EXCEEDED,
                    None,
                    0,
                )));
            }
            let token = store
                .data()
                .net
                .as_ref()
                .and_then(|net| net.lease)
                .ok_or(TerminalOutcome::GenerationInvalidated)?;
            match net::tcp_receive_inspect_step(token, now_ms) {
                Ok(net::TcpReceiveInspection::WouldBlock) => {
                    store
                        .data_mut()
                        .net
                        .as_mut()
                        .expect("net state missing")
                        .pending = Some(pending);
                    Ok(None)
                }
                Ok(net::TcpReceiveInspection::Available(available)) if available < 2 => {
                    store
                        .data_mut()
                        .net
                        .as_mut()
                        .expect("net state missing")
                        .pending = Some(pending);
                    Ok(None)
                }
                Ok(net::TcpReceiveInspection::Available(_)) => {
                    match net::tcp_recv_step(token, &mut pending.buffer[..cap], now_ms) {
                        Ok(net::TcpIoResult::Ready(read)) => Ok(Some(complete_net_operation(
                            store,
                            pending,
                            read as i32,
                            Some(ptr),
                            read,
                        ))),
                        Ok(net::TcpIoResult::WouldBlock) => {
                            store
                                .data_mut()
                                .net
                                .as_mut()
                                .expect("net state missing")
                                .pending = Some(pending);
                            Ok(None)
                        }
                        Ok(net::TcpIoResult::Closed) => Ok(Some(complete_net_operation(
                            store,
                            pending,
                            0,
                            Some(ptr),
                            0,
                        ))),
                        Ok(net::TcpIoResult::Unavailable) => Ok(Some(complete_net_operation(
                            store,
                            pending,
                            HOST_IMPORT_ERROR_TRANSPORT,
                            None,
                            0,
                        ))),
                        Err(_) => Err(TerminalOutcome::GenerationInvalidated),
                    }
                }
                Ok(net::TcpReceiveInspection::Closed) => Ok(Some(complete_net_operation(
                    store,
                    pending,
                    0,
                    Some(ptr),
                    0,
                ))),
                Ok(net::TcpReceiveInspection::Unavailable) => Ok(Some(complete_net_operation(
                    store,
                    pending,
                    HOST_IMPORT_ERROR_TRANSPORT,
                    None,
                    0,
                ))),
                Err(_) => Err(TerminalOutcome::GenerationInvalidated),
            }
        }
    }
}

fn complete_net_operation(
    store: &mut Store<BeyondEnvState>,
    pending: PendingNetworkOperation,
    result: i32,
    recv_ptr: Option<usize>,
    recv_len: usize,
) -> NetCompletion {
    let operation_id = pending.operation.operation_id;
    let resume_count = store.data().lifecycle.resume_count().saturating_add(1);
    let (tx_bytes, rx_bytes) = {
        let net_state = store.data_mut().net.as_mut().expect("net state missing");
        match pending.kind {
            PendingNetworkKind::Send { .. } if result > 0 => {
                net_state.tx_bytes = net_state.tx_bytes.saturating_add(result as u64)
            }
            PendingNetworkKind::Recv { .. } if result > 0 => {
                net_state.rx_bytes = net_state.rx_bytes.saturating_add(result as u64)
            }
            _ => {}
        }
        if result < 0 && net_state.first_negative_result.is_none() {
            net_state.first_negative_result = Some(result);
        }
        (net_state.tx_bytes, net_state.rx_bytes)
    };
    let mut probe = NET_SHIM_PROBE.lock();
    probe.resume_count = resume_count;
    probe.tx_bytes = tx_bytes;
    probe.rx_bytes = rx_bytes;
    if recv_len > 0 {
        probe.rx_sha256 = sha256_bytes(&pending.buffer[..recv_len]);
    }
    if recv_len >= 2 {
        probe.received_prefix_le =
            u16::from_le_bytes([pending.buffer[0], pending.buffer[1]]) as u64;
        probe.dns_tcp_length_prefix_present =
            u16::from_be_bytes([pending.buffer[0], pending.buffer[1]]) > 0;
    }
    serial::write_fmt(format_args!(
        "RAIOS_NET_SHIM operation_complete operation_id={} result={}\r\n",
        operation_id, result
    ));
    NetCompletion {
        result,
        deadline_ms: pending.deadline_ms,
        kind: pending.kind,
        recv_ptr,
        recv_len,
        buffer: pending.buffer,
    }
}

fn check_net_resume_boundary(
    store: &mut Store<BeyondEnvState>,
    completion: &NetCompletion,
    now_ms: u64,
) -> Result<(), TerminalOutcome> {
    if now_ms >= completion.deadline_ms && completion.result != HOST_IMPORT_ERROR_TIMED_OUT {
        return Err(TerminalOutcome::WallBudgetExceeded);
    }
    let net_state = store
        .data()
        .net
        .as_ref()
        .ok_or(TerminalOutcome::HostError)?;
    if net_state.tx_bytes > NET_TX_QUOTA_BYTES || net_state.rx_bytes > NET_RX_QUOTA_BYTES {
        return Err(TerminalOutcome::HostError);
    }
    let lease_required = completion.result >= 0
        && matches!(
            completion.kind,
            PendingNetworkKind::Open
                | PendingNetworkKind::Send { .. }
                | PendingNetworkKind::Recv { .. }
        );
    match net_state.lease {
        Some(token) => {
            net::tcp_validate(token, now_ms).map_err(|_| TerminalOutcome::GenerationInvalidated)
        }
        None if lease_required => Err(TerminalOutcome::GenerationInvalidated),
        None => Ok(()),
    }
}

fn finish_net_resources(
    store: &mut Store<BeyondEnvState>,
    now_ms: u64,
) -> Option<NetTerminalState> {
    let net_state = store.data_mut().net.as_mut()?;
    let token = net_state.lease.take();
    net_state.pending = None;
    net_state.connection_opened = false;
    let terminal = NetTerminalState {
        tx_bytes: net_state.tx_bytes,
        rx_bytes: net_state.rx_bytes,
        first_negative_result: net_state.first_negative_result,
        close_call_count: net_state.close_call_count,
        would_block_guest_visible: net_state.would_block_guest_visible,
    };
    if let Some(token) = token {
        let _ = net::tcp_abort(token, now_ms);
    }
    Some(terminal)
}

impl Drop for ActiveBeyondEnvInvocation {
    fn drop(&mut self) {
        if let Some(store) = self.store.as_mut() {
            let _ = finish_net_resources(store, runtime_now_ms());
            if store
                .data_mut()
                .lifecycle
                .teardown(TerminalOutcome::Abandoned)
            {
                ACTIVE_DROP_TEARDOWN_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }
        ACTIVE_BEYOND_ENV_INVOCATION.store(0, Ordering::Release);
    }
}

fn finish_store(store: &mut Store<BeyondEnvState>, outcome: TerminalOutcome) {
    store.data_mut().lifecycle.teardown(outcome);
}

fn host_test_suspend_once(mut caller: Caller<'_, BeyondEnvState>) -> Result<i32, Trap> {
    match caller.data().behavior {
        FixtureBehavior::Suspend => {
            let authority = caller.data().lifecycle.authority();
            let pending = caller
                .data_mut()
                .lifecycle
                .record_pending(1)
                .ok_or_else(|| Trap::new("test suspend pending operation already exists"))?;
            Err(HostSuspend {
                invocation_id: authority.invocation_id,
                operation_id: pending.operation_id,
            }
            .into())
        }
        FixtureBehavior::UnrecognizedHostError => Err(FixtureHostError.into()),
        FixtureBehavior::HostFuelError => {
            caller
                .consume_fuel(u64::MAX)
                .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
            Ok(0)
        }
        FixtureBehavior::Net(_) => Err(Trap::new("test.suspend_once unavailable to net fixture")),
    }
}

fn host_net_tcp_open(mut caller: Caller<'_, BeyondEnvState>) -> Result<i32, Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if caller
        .data()
        .net
        .as_ref()
        .is_none_or(|net| net.pending.is_some() || net.lease.is_some() || net.connection_opened)
    {
        return Ok(HOST_IMPORT_ERROR_INVALID_STATE);
    }
    record_net_pending(
        &mut caller,
        PendingNetworkKind::Open,
        [0; NET_CALL_MAX_BYTES],
        NET_CONNECT_TIMEOUT_MS,
    )
}

fn host_net_tcp_send(
    mut caller: Caller<'_, BeyondEnvState>,
    handle: i32,
    ptr: i32,
    len: i32,
) -> Result<i32, Trap> {
    let Some((ptr, len)) = checked_net_memory_range(ptr, len) else {
        return Ok(HOST_IMPORT_ERROR_INVALID_ARGUMENT);
    };
    if len > NET_CALL_MAX_BYTES {
        return Ok(HOST_IMPORT_ERROR_LIMIT_EXCEEDED);
    }
    let valid = caller.data().lifecycle.accepts_handle(handle)
        && caller.data().net.as_ref().is_some_and(|net| {
            net.connection_opened
                && net.pending.is_none()
                && net.tx_bytes.saturating_add(len as u64) <= NET_TX_QUOTA_BYTES
        });
    if !valid {
        return Ok(HOST_IMPORT_ERROR_INVALID_STATE);
    }
    caller
        .consume_fuel(25 + len as u64)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("net.tcp_send memory export missing"))?;
    let mut buffer = [0u8; NET_CALL_MAX_BYTES];
    memory
        .read(&caller, ptr, &mut buffer[..len])
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    record_net_pending(
        &mut caller,
        PendingNetworkKind::Send { len },
        buffer,
        NET_IDLE_TIMEOUT_MS,
    )
}

fn host_net_tcp_recv(
    mut caller: Caller<'_, BeyondEnvState>,
    handle: i32,
    ptr: i32,
    cap: i32,
) -> Result<i32, Trap> {
    let Some((ptr, cap)) = checked_net_memory_range(ptr, cap) else {
        return Ok(HOST_IMPORT_ERROR_INVALID_ARGUMENT);
    };
    if cap > NET_CALL_MAX_BYTES {
        return Ok(HOST_IMPORT_ERROR_LIMIT_EXCEEDED);
    }
    let valid = caller.data().lifecycle.accepts_handle(handle)
        && caller.data().net.as_ref().is_some_and(|net| {
            net.connection_opened
                && net.pending.is_none()
                && net.rx_bytes.saturating_add(cap as u64) <= NET_RX_QUOTA_BYTES
        });
    if !valid {
        return Ok(HOST_IMPORT_ERROR_INVALID_STATE);
    }
    caller
        .consume_fuel(25 + cap as u64)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("net.tcp_recv memory export missing"))?;
    memory
        .read(&caller, ptr, &mut [])
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    if ptr
        .checked_add(cap)
        .is_none_or(|end| end > memory.data(&caller).len())
    {
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    }
    record_net_pending(
        &mut caller,
        PendingNetworkKind::Recv { ptr, cap },
        [0; NET_CALL_MAX_BYTES],
        NET_IDLE_TIMEOUT_MS,
    )
}

fn host_net_tcp_close(mut caller: Caller<'_, BeyondEnvState>, handle: i32) -> Result<i32, Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if !caller.data().lifecycle.accepts_handle(handle) {
        return Ok(HOST_IMPORT_ERROR_INVALID_STATE);
    }
    let now_ms = runtime_now_ms();
    let (token, operation_id, close_call_count) = {
        let net = caller
            .data_mut()
            .net
            .as_mut()
            .ok_or_else(|| Trap::new("net.tcp_close outside net fixture"))?;
        if net.pending.is_some() || !net.connection_opened {
            return Ok(HOST_IMPORT_ERROR_INVALID_STATE);
        }
        let operation_id = net.next_operation();
        net.close_call_count = net.close_call_count.saturating_add(1);
        (net.lease.take(), operation_id, net.close_call_count)
    };
    let result = match token {
        Some(token) => match net::tcp_close(token, now_ms) {
            Ok(_) => 0,
            Err(error) => transport_error_result(error),
        },
        None => 0,
    };
    let mut probe = NET_SHIM_PROBE.lock();
    probe.close_call_count = close_call_count;
    if close_call_count == 1 {
        probe.first_close_operation_id = operation_id;
    } else if close_call_count == 2 {
        probe.second_close_operation_id = operation_id;
    }
    probe.lease_held = false;
    Ok(result)
}

fn record_net_pending(
    caller: &mut Caller<'_, BeyondEnvState>,
    kind: PendingNetworkKind,
    buffer: [u8; NET_CALL_MAX_BYTES],
    timeout_ms: u64,
) -> Result<i32, Trap> {
    let operation_id = caller
        .data_mut()
        .net
        .as_mut()
        .ok_or_else(|| Trap::new("net host import outside net fixture"))?
        .next_operation();
    let pending = caller
        .data_mut()
        .lifecycle
        .record_pending(operation_id)
        .ok_or_else(|| Trap::new("net pending operation already exists"))?;
    let operation_name = match kind {
        PendingNetworkKind::Open => "tcp_open",
        PendingNetworkKind::Send { .. } => "tcp_send",
        PendingNetworkKind::Recv { .. } => "tcp_recv",
    };
    let scenario = caller
        .data()
        .net
        .as_ref()
        .map_or("invalid", |net| net.scenario.as_str());
    caller
        .data_mut()
        .net
        .as_mut()
        .expect("net state disappeared")
        .pending = Some(PendingNetworkOperation {
        owner_invocation_id: pending.invocation_id,
        operation: pending,
        deadline_ms: runtime_now_ms().saturating_add(timeout_ms),
        kind,
        buffer,
    });
    {
        let mut probe = NET_SHIM_PROBE.lock();
        probe.suspended = true;
        probe.suspension_count = caller.data().lifecycle.suspension_count();
        match operation_name {
            "tcp_open" => probe.open_operation_id = operation_id,
            "tcp_send" => probe.send_operation_id = operation_id,
            "tcp_recv" => probe.recv_operation_id = operation_id,
            _ => {}
        }
    }
    serial::write_fmt(format_args!(
        "RAIOS_NET_SHIM suspended=true scenario={} operation={} operation_id={}\r\n",
        scenario, operation_name, operation_id
    ));
    Err(HostSuspend {
        invocation_id: pending.invocation_id,
        operation_id,
    }
    .into())
}

fn checked_net_memory_range(ptr: i32, len: i32) -> Option<(usize, usize)> {
    if ptr < 0 || len < 0 {
        return None;
    }
    let ptr = ptr as usize;
    let len = len as usize;
    ptr.checked_add(len)?;
    Some((ptr, len))
}

fn transport_error_result(error: net::TransportLeaseError) -> i32 {
    match error {
        net::TransportLeaseError::NetworkTransportBusy => HOST_IMPORT_ERROR_RESOURCE_BUSY,
        net::TransportLeaseError::LeaseTimedOut => HOST_IMPORT_ERROR_TIMED_OUT,
        net::TransportLeaseError::ForeignOwner
        | net::TransportLeaseError::StaleGeneration
        | net::TransportLeaseError::LeaseRevoked
        | net::TransportLeaseError::GenerationExhausted => HOST_IMPORT_ERROR_TRANSPORT,
    }
}

fn runtime_now_ms() -> u64 {
    crate::time::rdtsc() / crate::time::tsc_per_ms().max(1)
}

fn classify_resumable(
    invocation: &ResumableInvocation,
    store: &Store<BeyondEnvState>,
) -> Result<(), TerminalOutcome> {
    if matches!(
        invocation.host_error().trap_code(),
        Some(TrapCode::OutOfFuel)
    ) {
        return Err(TerminalOutcome::OutOfFuel);
    }
    let marker = invocation
        .host_error()
        .downcast_ref::<HostSuspend>()
        .ok_or(TerminalOutcome::HostError)?;
    if !store
        .data()
        .lifecycle
        .suspend_marker_matches(marker.invocation_id, marker.operation_id)
    {
        return Err(TerminalOutcome::HostError);
    }
    Ok(())
}

fn terminal_from_error(error: &wasmi::Error) -> TerminalOutcome {
    match error {
        wasmi::Error::Trap(trap) if matches!(trap.trap_code(), Some(TrapCode::OutOfFuel)) => {
            TerminalOutcome::OutOfFuel
        }
        wasmi::Error::Trap(trap)
            if matches!(trap.trap_code(), Some(TrapCode::UnreachableCodeReached)) =>
        {
            TerminalOutcome::GuestTrap
        }
        wasmi::Error::Trap(_) => TerminalOutcome::HostError,
        _ => TerminalOutcome::HostError,
    }
}

fn beyond_env_engine() -> Engine {
    let mut config = Config::default();
    config.consume_fuel(true).wasm_tail_call(true);
    Engine::new(&config)
}

fn beyond_env_limits() -> StoreLimits {
    StoreLimitsBuilder::new()
        .memory_size(BUFFER_SERVICE_MAX_MEMORY_BYTES)
        .instances(1)
        .memories(1)
        .tables(1)
        .table_elements(64)
        .build()
}

pub(crate) fn run_beyond_env_lifecycle_suite() -> BeyondEnvLifecycleSuite {
    if let Some(suite) = *BEYOND_ENV_SUITE.lock() {
        return suite;
    }
    if wasm_execution_busy() {
        return busy_beyond_env_lifecycle_suite();
    }
    let normal = run_beyond_env_case(
        "normal_return",
        NORMAL_RETURN_I32_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::None,
    );
    let guest_trap = run_beyond_env_case(
        "guest_trap",
        UNREACHABLE_I32_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::None,
    );
    let unrecognized = run_beyond_env_case(
        "unrecognized_host_error",
        SUSPEND_ONCE_WASM_MODULE,
        FixtureBehavior::UnrecognizedHostError,
        100,
        CaseAction::None,
    );
    let host_fuel = run_beyond_env_case(
        "host_fuel_error",
        SUSPEND_ONCE_WASM_MODULE,
        FixtureBehavior::HostFuelError,
        100,
        CaseAction::None,
    );
    let busy_started = crate::time::rdtsc() / crate::time::tsc_per_ms().max(1);
    let wasm_fuel = run_beyond_env_case(
        "wasm_out_of_fuel",
        FUEL_LOOP_I32_WASM_MODULE,
        FixtureBehavior::Suspend,
        BEYOND_ENV_FIXTURE_FUEL_BUDGET,
        CaseAction::None,
    );
    let busy_loop_wall_ms =
        (crate::time::rdtsc() / crate::time::tsc_per_ms().max(1)).saturating_sub(busy_started);
    let marker_mismatch = run_beyond_env_case(
        "suspend_marker_mismatch",
        SUSPEND_ONCE_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::Mismatch,
    );
    let abandoned = run_beyond_env_case(
        "abandoned_drop_guard",
        SUSPEND_ONCE_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::Abandon,
    );
    let tail = run_beyond_env_case(
        "tail_call_terminal",
        TAIL_SUSPEND_WASM_MODULE,
        FixtureBehavior::Suspend,
        100,
        CaseAction::None,
    );
    let suite = BeyondEnvLifecycleSuite {
        cases: [
            normal,
            guest_trap,
            unrecognized,
            host_fuel,
            wasm_fuel,
            marker_mismatch,
            abandoned,
        ],
        tail_call_terminal: tail.outcome == "host_error" && !tail.suspended,
        busy_loop_out_of_fuel: wasm_fuel.outcome == "out_of_fuel",
        busy_loop_wall_ms,
        fuel_budget: BEYOND_ENV_FIXTURE_FUEL_BUDGET,
        wall_budget_ms: BEYOND_ENV_WALL_BUDGET_MS,
        pump_step_budget: BEYOND_ENV_PUMP_STEP_BUDGET,
    };
    *BEYOND_ENV_SUITE.lock() = Some(suite);
    suite
}

#[derive(Clone, Copy)]
enum CaseAction {
    None,
    Mismatch,
    Abandon,
}

fn run_beyond_env_case(
    name: &'static str,
    bytes: &[u8],
    behavior: FixtureBehavior,
    fuel_budget: u64,
    action: CaseAction,
) -> BeyondEnvLifecycleCase {
    let invocation_id = NEXT_BEYOND_ENV_INVOCATION_ID.fetch_add(1, Ordering::Relaxed);
    let authority = InvocationAuthority {
        service_id: "test.fixture.beyond_env_lifecycle",
        invocation_id,
        service_generation: 1,
        instance_generation: 1,
        captured_kill_generation: 0,
        policy_allows_beyond_env: false,
    };
    let engine = beyond_env_engine();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => return failed_beyond_env_case(name),
    };
    let mut store = Store::new(
        &engine,
        BeyondEnvState {
            lifecycle: InvocationLifecycle::new(
                authority,
                0,
                BEYOND_ENV_WALL_BUDGET_MS,
                BEYOND_ENV_PUMP_STEP_BUDGET,
                invocation_id as u32,
            ),
            behavior,
            net: None,
            limits: beyond_env_limits(),
        },
    );
    store.limiter(|state| &mut state.limits);
    if store.add_fuel(fuel_budget).is_err() {
        return failed_beyond_env_case(name);
    }
    let mut linker = Linker::<BeyondEnvState>::new(&engine);
    if linker
        .func_wrap("test", "suspend_once", host_test_suspend_once)
        .is_err()
    {
        return failed_beyond_env_case(name);
    }
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(pre) => match pre.start(&mut store) {
            Ok(instance) => instance,
            Err(error) => {
                let outcome = terminal_from_error(&error);
                return teardown_case(name, &mut store, outcome, false);
            }
        },
        Err(error) => {
            let outcome = terminal_from_error(&error);
            return teardown_case(name, &mut store, outcome, false);
        }
    };
    let Some(function) = instance
        .get_export(&store, "run")
        .and_then(Extern::into_func)
    else {
        return teardown_case(name, &mut store, TerminalOutcome::HostError, false);
    };
    let mut outputs = [Value::I32(0)];
    match function.call_resumable(&mut store, &[], &mut outputs) {
        Ok(ResumableCall::Finished) => {
            teardown_case(name, &mut store, TerminalOutcome::Finished, false)
        }
        Ok(ResumableCall::Resumable(invocation)) => {
            if matches!(action, CaseAction::Mismatch) {
                store.data_mut().lifecycle.take_pending();
                store.data_mut().lifecycle.record_pending(2);
            }
            let classification = classify_resumable(&invocation, &store);
            if matches!(action, CaseAction::Abandon) {
                let suspension_count = store.data().lifecycle.suspension_count();
                let memory = instance
                    .get_export(&store, "memory")
                    .and_then(Extern::into_memory);
                let before = ACTIVE_DROP_TEARDOWN_COUNT.load(Ordering::Relaxed);
                drop(ActiveBeyondEnvInvocation {
                    store: Some(store),
                    instance: Some(instance),
                    memory,
                    continuation: Some(invocation),
                    outputs,
                    auto_resume: false,
                    closed: false,
                });
                let teardown_count = ACTIVE_DROP_TEARDOWN_COUNT
                    .load(Ordering::Relaxed)
                    .saturating_sub(before) as u32;
                return BeyondEnvLifecycleCase {
                    name,
                    outcome: TerminalOutcome::Abandoned.as_str(),
                    suspended: true,
                    suspension_count,
                    resume_count: 0,
                    teardown_count,
                    teardown_complete: teardown_count == 1,
                };
            }
            if let Err(outcome) = classification {
                return teardown_case(name, &mut store, outcome, false);
            }
            store.data_mut().lifecycle.take_pending();
            store.data_mut().lifecycle.note_resume();
            match invocation.resume(&mut store, &[Value::I32(7)], &mut outputs) {
                Ok(ResumableCall::Finished) => {
                    teardown_case(name, &mut store, TerminalOutcome::Finished, true)
                }
                Ok(ResumableCall::Resumable(next)) => {
                    let outcome = classify_resumable(&next, &store)
                        .err()
                        .unwrap_or(TerminalOutcome::HostError);
                    teardown_case(name, &mut store, outcome, true)
                }
                Err(error) => {
                    let outcome = terminal_from_error(&error);
                    teardown_case(name, &mut store, outcome, true)
                }
            }
        }
        Err(error) => {
            let outcome = terminal_from_error(&error);
            teardown_case(name, &mut store, outcome, false)
        }
    }
}

fn teardown_case(
    name: &'static str,
    store: &mut Store<BeyondEnvState>,
    outcome: TerminalOutcome,
    suspended: bool,
) -> BeyondEnvLifecycleCase {
    finish_store(store, outcome);
    case_from_store(name, store, suspended)
}

fn case_from_store(
    name: &'static str,
    store: &Store<BeyondEnvState>,
    suspended: bool,
) -> BeyondEnvLifecycleCase {
    let lifecycle = &store.data().lifecycle;
    let receipt = lifecycle.teardown_receipt();
    BeyondEnvLifecycleCase {
        name,
        outcome: lifecycle
            .terminal()
            .map(TerminalOutcome::as_str)
            .unwrap_or("not_terminal"),
        suspended,
        suspension_count: lifecycle.suspension_count(),
        resume_count: lifecycle.resume_count(),
        teardown_count: receipt.teardown_count,
        teardown_complete: receipt.terminal_outcome_recorded,
    }
}

fn failed_beyond_env_case(name: &'static str) -> BeyondEnvLifecycleCase {
    BeyondEnvLifecycleCase {
        name,
        outcome: "fixture_setup_failed",
        suspended: false,
        suspension_count: 0,
        resume_count: 0,
        teardown_count: 0,
        teardown_complete: false,
    }
}

fn busy_beyond_env_lifecycle_suite() -> BeyondEnvLifecycleSuite {
    let case = BeyondEnvLifecycleCase {
        name: "wasm_execution_busy",
        outcome: "wasm_execution_busy",
        suspended: false,
        suspension_count: 0,
        resume_count: 0,
        teardown_count: 0,
        teardown_complete: false,
    };
    BeyondEnvLifecycleSuite {
        cases: [case; 7],
        tail_call_terminal: false,
        busy_loop_out_of_fuel: false,
        busy_loop_wall_ms: 0,
        fuel_budget: BEYOND_ENV_FIXTURE_FUEL_BUDGET,
        wall_budget_ms: BEYOND_ENV_WALL_BUDGET_MS,
        pump_step_budget: BEYOND_ENV_PUMP_STEP_BUDGET,
    }
}

#[used]
static WASMI_COMPILE_PROOF: fn() -> bool = validate_empty_module_bytes;
#[used]
static ECHO_WASM_ARTIFACT_PROOF: fn() -> bool = validate_echo_wasm_artifact;
#[used]
static BUFECHO_WASM_ARTIFACT_PROOF: fn() -> bool = validate_bufecho_wasm_artifact;
#[used]
static CERTWINDOW_WASM_ARTIFACT_PROOF: fn() -> bool = validate_certwindow_wasm_artifact;
#[used]
static HTTPHEAD_WASM_ARTIFACT_PROOF: fn() -> bool = validate_httphead_wasm_artifact;
#[used]
static CERTSPKI_WASM_ARTIFACT_PROOF: fn() -> bool = validate_certspki_wasm_artifact;
#[used]
static DNSPARSE_WASM_ARTIFACT_PROOF: fn() -> bool = validate_dnsparse_wasm_artifact;

pub(crate) fn validate_empty_module_bytes() -> bool {
    let wasm = Vec::from(EMPTY_WASM_MODULE).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    validate_module_bytes(bytes)
}

pub(crate) fn validate_echo_wasm_artifact() -> bool {
    let wasm = Vec::from(ECHO_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == ECHO_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_bufecho_wasm_artifact() -> bool {
    let wasm = Vec::from(BUFECHO_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == BUFECHO_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(BUFECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == BUFECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(BUFECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == BUFECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_certwindow_wasm_artifact() -> bool {
    let wasm = Vec::from(CERTWINDOW_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == CERTWINDOW_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(CERTWINDOW_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == CERTWINDOW_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(CERTWINDOW_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == CERTWINDOW_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_httphead_wasm_artifact() -> bool {
    let wasm = Vec::from(HTTPHEAD_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == HTTPHEAD_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(HTTPHEAD_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == HTTPHEAD_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(HTTPHEAD_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == HTTPHEAD_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_certspki_wasm_artifact() -> bool {
    let wasm = Vec::from(CERTSPKI_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == CERTSPKI_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(CERTSPKI_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == CERTSPKI_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(CERTSPKI_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == CERTSPKI_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_dnsparse_wasm_artifact() -> bool {
    let wasm = Vec::from(DNSPARSE_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == DNSPARSE_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(DNSPARSE_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == DNSPARSE_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(DNSPARSE_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == DNSPARSE_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

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

pub(crate) struct EchoProbe {
    pub(crate) artifact_hash: [u8; 32],
    pub(crate) descriptor_hash: [u8; 32],
    pub(crate) signature_envelope_hash: [u8; 32],
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) run_outcome: &'static str,
    pub(crate) return_value: Option<i32>,
    pub(crate) fuel_budget: u64,
    pub(crate) fuel_used: u64,
    pub(crate) log_line: Option<String>,
    pub(crate) forbidden_validation_ok: bool,
    pub(crate) forbidden_instantiation_ok: bool,
    pub(crate) forbidden_link_error_kind: &'static str,
    pub(crate) forbidden_missing_import_module: Option<&'static str>,
    pub(crate) forbidden_missing_import_name: Option<&'static str>,
    pub(crate) forbidden_boundary_held: bool,
    pub(crate) hardening_cases: [WasmHardeningCase; WASM_HARDENING_CASE_COUNT],
}

pub(crate) struct EchoRunEvidence {
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) run_outcome: &'static str,
    pub(crate) return_value: Option<i32>,
    pub(crate) fuel_budget: u64,
    pub(crate) fuel_used: u64,
    pub(crate) log_line: Option<String>,
    pub(crate) import_grant_performed: bool,
    pub(crate) import_grant_status: &'static str,
    pub(crate) import_grant_reason: &'static str,
    pub(crate) authorized_import_count: u64,
    pub(crate) authorized_import_list_sha256: [u8; 32],
    pub(crate) captured_output_len: u64,
    pub(crate) captured_output_sha256: [u8; 32],
    pub(crate) raw_captured_output: Vec<u8>,
    pub(crate) linked_host_import_count: u64,
    pub(crate) module_imports_within_authorized_list: bool,
    pub(crate) missing_import_module: Option<String>,
    pub(crate) missing_import_name: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) struct WorkspaceImportInspection {
    pub(crate) validation_ok: bool,
    pub(crate) import_list_observed: bool,
    pub(crate) import_count: usize,
    pub(crate) reason: &'static str,
}

pub(crate) struct BufechoRoundtripEvidence {
    pub(crate) run: EchoRunEvidence,
    pub(crate) input_len: u64,
    pub(crate) input_sha256: [u8; 32],
}

/// Evidence from a labeled fuel-starvation fault injection against the real echo
/// module. `out_of_fuel` is TRUE only when the caught trap was specifically a
/// wasmi `OutOfFuel`; any other trap kind is reported honestly in `run_outcome`
/// with `out_of_fuel=false` so a caller never mislabels a different fault as a
/// fuel wedge.
pub(crate) struct EchoFuelStarvedEvidence {
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) run_outcome: &'static str,
    pub(crate) out_of_fuel: bool,
    pub(crate) fuel_budget: u64,
    pub(crate) fuel_used: u64,
    pub(crate) log_line: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) struct WasmHardeningCase {
    pub(crate) name: &'static str,
    pub(crate) mechanism: &'static str,
    pub(crate) expected_outcome: &'static str,
    pub(crate) actual_outcome: &'static str,
    pub(crate) passed: bool,
}

fn busy_echo_probe() -> EchoProbe {
    let positive = wasm_busy_run(ECHO_SERVICE_ID, ECHO_WASM_FUEL_BUDGET);
    let busy_case = WasmHardeningCase {
        name: "wasm_execution_busy",
        mechanism: "central_wasm_execution_gate",
        expected_outcome: "wasm_execution_busy",
        actual_outcome: "wasm_execution_busy",
        passed: true,
    };
    EchoProbe {
        artifact_hash: ECHO_WASM_ARTIFACT_BYTES_HASH,
        descriptor_hash: ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH,
        signature_envelope_hash: ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH,
        validation_ok: false,
        instantiation_ok: false,
        run_outcome: positive.run_outcome,
        return_value: None,
        fuel_budget: positive.fuel_budget,
        fuel_used: 0,
        log_line: None,
        forbidden_validation_ok: false,
        forbidden_instantiation_ok: false,
        forbidden_link_error_kind: "wasm_execution_busy",
        forbidden_missing_import_module: None,
        forbidden_missing_import_name: None,
        forbidden_boundary_held: true,
        hardening_cases: [busy_case; WASM_HARDENING_CASE_COUNT],
    }
}

pub(crate) fn run_echo_probe() -> EchoProbe {
    if wasm_execution_busy() {
        return busy_echo_probe();
    }
    let positive = run_echo_service();
    let negative = instantiate_forbidden_import_module();
    let hardening_cases = run_hardening_cases();

    EchoProbe {
        artifact_hash: ECHO_WASM_ARTIFACT_BYTES_HASH,
        descriptor_hash: ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH,
        signature_envelope_hash: ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH,
        validation_ok: positive.validation_ok,
        instantiation_ok: positive.instantiation_ok,
        run_outcome: positive.run_outcome,
        return_value: positive.return_value,
        fuel_budget: positive.fuel_budget,
        fuel_used: positive.fuel_used,
        log_line: positive.log_line,
        forbidden_validation_ok: negative.validation_ok,
        forbidden_instantiation_ok: negative.instantiation_ok,
        forbidden_link_error_kind: negative.link_error_kind,
        forbidden_missing_import_module: negative.missing_import_module,
        forbidden_missing_import_name: negative.missing_import_name,
        forbidden_boundary_held: negative.boundary_held,
        hardening_cases,
    }
}

pub(crate) fn run_echo_service() -> EchoRunEvidence {
    execute_echo_module(validate_echo_wasm_artifact())
}

pub(crate) fn run_bufecho_roundtrip(input: &[u8]) -> BufechoRoundtripEvidence {
    let capped_len = input.len().min(MAX_WASM_INPUT_BYTES);
    let capped = &input[..capped_len];
    BufechoRoundtripEvidence {
        run: execute_validated_module_bytes(
            BUFECHO_WASM_ARTIFACT_BYTES,
            "raios_service_main",
            BUFECHO_SERVICE_ID,
            true,
            BUFECHO_AUTHORIZED_IMPORTS,
            validate_bufecho_wasm_artifact(),
            capped,
            ECHO_WASM_FUEL_BUDGET,
        ),
        input_len: capped_len as u64,
        input_sha256: sha256_bytes(capped),
    }
}

pub(crate) fn run_bufecho_unauthorized_probe() -> EchoRunEvidence {
    execute_validated_module_bytes(
        BUFECHO_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        BUFECHO_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_bufecho_wasm_artifact(),
        b"raios-m11-bufecho-unauthorized-nonce",
        ECHO_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_certwindow_roundtrip(cert_der: &[u8]) -> EchoRunEvidence {
    let capped = &cert_der[..cert_der.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        CERTWINDOW_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        CERTWINDOW_SERVICE_ID,
        true,
        CERTWINDOW_AUTHORIZED_IMPORTS,
        validate_certwindow_wasm_artifact(),
        capped,
        CERTWINDOW_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_certwindow_unauthorized_probe() -> EchoRunEvidence {
    execute_validated_module_bytes(
        CERTWINDOW_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        CERTWINDOW_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_certwindow_wasm_artifact(),
        b"raios-m11-certwindow-unauthorized-nonce",
        CERTWINDOW_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_httphead_roundtrip(response: &[u8]) -> EchoRunEvidence {
    let capped = &response[..response.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        HTTPHEAD_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        HTTPHEAD_SERVICE_ID,
        true,
        HTTPHEAD_AUTHORIZED_IMPORTS,
        validate_httphead_wasm_artifact(),
        capped,
        HTTPHEAD_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_httphead_unauthorized_probe() -> EchoRunEvidence {
    execute_validated_module_bytes(
        HTTPHEAD_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        HTTPHEAD_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_httphead_wasm_artifact(),
        b"raios-m11-httphead-unauthorized-nonce",
        HTTPHEAD_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_certspki_roundtrip(cert_der: &[u8]) -> EchoRunEvidence {
    let capped = &cert_der[..cert_der.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        CERTSPKI_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        CERTSPKI_SERVICE_ID,
        true,
        CERTSPKI_AUTHORIZED_IMPORTS,
        validate_certspki_wasm_artifact(),
        capped,
        CERTSPKI_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_certspki_unauthorized_probe() -> EchoRunEvidence {
    execute_validated_module_bytes(
        CERTSPKI_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        CERTSPKI_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_certspki_wasm_artifact(),
        b"raios-m11-certspki-unauthorized-nonce",
        CERTSPKI_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_dnsparse_roundtrip(input_record: &[u8]) -> EchoRunEvidence {
    let capped = &input_record[..input_record.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        DNSPARSE_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        DNSPARSE_SERVICE_ID,
        true,
        DNSPARSE_AUTHORIZED_IMPORTS,
        validate_dnsparse_wasm_artifact(),
        capped,
        DNSPARSE_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn run_dnsparse_unauthorized_probe(input_record: &[u8]) -> EchoRunEvidence {
    let capped = &input_record[..input_record.len().min(MAX_WASM_INPUT_BYTES)];
    execute_validated_module_bytes(
        DNSPARSE_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        DNSPARSE_SERVICE_ID,
        true,
        &[("env", "input_len")],
        validate_dnsparse_wasm_artifact(),
        capped,
        DNSPARSE_WASM_FUEL_BUDGET,
    )
}

/// Labeled fault injection: run the REAL echo artifact (`raios_service_main`)
/// through a metered store carrying only `ECHO_WASM_FUEL_STARVED_BUDGET` fuel, so
/// the invoke genuinely traps with wasmi `OutOfFuel` (never simulated). The trap
/// is an `Err` value that is CAUGHT here — never unwrapped/panicked — and
/// classified via `classify_trap_error`; the cooperative kernel loop is unharmed.
/// Reuses `metered_engine`/`default_state`/`define_granted_imports` exactly
/// like the healthy path so the ONLY difference is the fuel budget.
pub(crate) fn run_echo_fuel_starved() -> EchoFuelStarvedEvidence {
    if wasm_execution_busy() {
        return fuel_starved_evidence(false, false, "wasm_execution_busy", false, 0, None);
    }
    if !validate_echo_wasm_artifact() {
        return fuel_starved_evidence(false, false, "validation_failed", false, 0, None);
    }
    let authorized = match authorize_wasm_imports(ECHO_SERVICE_ID, true, ECHO_AUTHORIZED_IMPORTS) {
        Ok(authorized) => authorized,
        Err(_) => return fuel_starved_evidence(true, false, "import_grant_denied", false, 0, None),
    };

    let wasm = Vec::from(ECHO_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return fuel_starved_evidence(true, false, "module_compile_failed", false, 0, None)
        }
    };
    let mut store = Box::new(Store::new(&engine, default_state()));
    if store.add_fuel(ECHO_WASM_FUEL_STARVED_BUDGET).is_err() {
        return fuel_starved_evidence(true, false, "fuel_metering_unavailable", false, 0, None);
    }
    let mut linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    if let Err(reason) = define_granted_imports(&mut linker, &authorized) {
        return fuel_starved_evidence(true, false, reason, false, 0, None);
    }
    if first_unauthorized_module_import(&module, &authorized).is_some() {
        return fuel_starved_evidence(true, false, "module_import_not_authorized", false, 0, None);
    }

    let instance = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(instance) => instance,
            Err(error) => {
                let outcome = classify_trap_error(error, ExpectedTrap::OutOfFuel);
                return fuel_starved_evidence(
                    true,
                    false,
                    outcome,
                    outcome == "fuel_exhausted",
                    store.fuel_consumed().unwrap_or(0),
                    store.data().log_line.clone(),
                );
            }
        },
        Err(_) => {
            return fuel_starved_evidence(
                true,
                false,
                "instantiation_failed",
                false,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
            )
        }
    };

    let Some(func) = instance
        .get_export(&*store, "raios_service_main")
        .and_then(Extern::into_func)
    else {
        return fuel_starved_evidence(
            true,
            true,
            "entrypoint_missing",
            false,
            store.fuel_consumed().unwrap_or(0),
            store.data().log_line.clone(),
        );
    };

    let mut outputs = Vec::from([Value::I32(0)]).into_boxed_slice();
    match func.call(&mut *store, &[], &mut outputs) {
        Ok(()) => fuel_starved_evidence(
            true,
            true,
            "run_success_unexpected",
            false,
            store.fuel_consumed().unwrap_or(0),
            store.data().log_line.clone(),
        ),
        Err(error) => {
            let outcome = classify_trap_error(error, ExpectedTrap::OutOfFuel);
            fuel_starved_evidence(
                true,
                true,
                outcome,
                outcome == "fuel_exhausted",
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
            )
        }
    }
}

fn fuel_starved_evidence(
    validation_ok: bool,
    instantiation_ok: bool,
    run_outcome: &'static str,
    out_of_fuel: bool,
    fuel_used: u64,
    log_line: Option<String>,
) -> EchoFuelStarvedEvidence {
    EchoFuelStarvedEvidence {
        validation_ok,
        instantiation_ok,
        run_outcome,
        out_of_fuel,
        fuel_budget: ECHO_WASM_FUEL_STARVED_BUDGET,
        fuel_used,
        log_line,
    }
}

pub(crate) fn loader_available() -> bool {
    true
}

pub(crate) fn validate_module_bytes(bytes: &[u8]) -> bool {
    let engine = Box::new(wasmi::Engine::default());
    wasmi::Module::new(&engine, bytes).is_ok()
}

struct EnvelopeState {
    log_line: Option<String>,
    staged_input: Vec<u8>,
    captured_output: Vec<u8>,
    limits: StoreLimits,
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

struct AuthorizedWasmImports<'a> {
    imports: &'a [(&'a str, &'a str)],
    decision: WasmImportGrantDecision,
    import_list_sha256: [u8; 32],
}

struct ImportGrantEvidence {
    performed: bool,
    status: &'static str,
    reason: &'static str,
    authorized_import_count: u64,
    authorized_import_list_sha256: [u8; 32],
    linked_host_import_count: u64,
    module_imports_within_authorized_list: bool,
    missing_import_module: Option<String>,
    missing_import_name: Option<String>,
}

pub(crate) struct NegativeRun {
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) link_error_kind: &'static str,
    pub(crate) missing_import_module: Option<&'static str>,
    pub(crate) missing_import_name: Option<&'static str>,
    pub(crate) boundary_held: bool,
}

fn execute_echo_module(validation_ok: bool) -> EchoRunEvidence {
    execute_validated_module_bytes(
        ECHO_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        ECHO_SERVICE_ID,
        true,
        ECHO_AUTHORIZED_IMPORTS,
        validation_ok,
        &[],
        ECHO_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn execute_module_bytes(
    bytes: &[u8],
    entrypoint: &str,
    service_id: &str,
    artifact_sha256_present: bool,
    requested_imports: &[(&str, &str)],
) -> EchoRunEvidence {
    execute_validated_module_bytes(
        bytes,
        entrypoint,
        service_id,
        artifact_sha256_present,
        requested_imports,
        validate_module_bytes(bytes),
        &[],
        ECHO_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn inspect_workspace_imports(bytes: &[u8]) -> WorkspaceImportInspection {
    let engine = metered_engine();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => {
            return WorkspaceImportInspection {
                validation_ok: false,
                import_list_observed: false,
                import_count: 0,
                reason: "workspace_module_invalid",
            }
        }
    };
    let import_count = module.imports().count();
    WorkspaceImportInspection {
        validation_ok: true,
        import_list_observed: true,
        import_count,
        reason: if import_count == 0 {
            "workspace_import_surface_observed_empty"
        } else {
            "workspace_import_surface_not_empty"
        },
    }
}

pub(crate) fn execute_workspace_no_import_candidate(bytes: &[u8]) -> EchoRunEvidence {
    if wasm_execution_busy() {
        return wasm_busy_run(WORKSPACE_SERVICE_ID, WORKSPACE_FUEL_BUDGET);
    }
    let inspection = inspect_workspace_imports(bytes);
    let grant = evaluate_observed_wasm_import_grant(
        &WasmImportGrantInput {
            service_id: Some(WORKSPACE_SERVICE_ID),
            artifact_sha256_present: true,
            requested_imports: &[],
            policy_allows_beyond_env: false,
        },
        inspection.import_list_observed,
    );
    let import_evidence = ImportGrantEvidence {
        performed: grant.performed,
        status: grant.status,
        reason: grant.reason,
        authorized_import_count: grant.authorized_import_count as u64,
        authorized_import_list_sha256: authorized_import_list_sha256(WORKSPACE_SERVICE_ID, &[]),
        linked_host_import_count: 0,
        module_imports_within_authorized_list: inspection.import_count == 0,
        missing_import_module: None,
        missing_import_name: None,
    };
    if !inspection.validation_ok || inspection.import_count != 0 || !grant.performed {
        return positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            inspection.validation_ok,
            false,
            inspection.reason,
            None,
            0,
            None,
            import_evidence,
        );
    }
    let engine = metered_engine();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => {
            return positive_run(
                WORKSPACE_SERVICE_ID,
                WORKSPACE_FUEL_BUDGET,
                false,
                false,
                "workspace_module_invalid",
                None,
                0,
                None,
                import_evidence,
            )
        }
    };
    let mut store = Store::new(&engine, limited_state(WORKSPACE_MEMORY_LIMIT_BYTES));
    store.limiter(|state| &mut state.limits);
    if store.add_fuel(WORKSPACE_FUEL_BUDGET).is_err() {
        return positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            true,
            false,
            "fuel_metering_unavailable",
            None,
            0,
            None,
            import_evidence,
        );
    }
    let linker = Linker::<EnvelopeState>::new(&engine);
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(pre) => match pre.start(&mut store) {
            Ok(instance) => instance,
            Err(error) => {
                return positive_run(
                    WORKSPACE_SERVICE_ID,
                    WORKSPACE_FUEL_BUDGET,
                    true,
                    false,
                    classify_workspace_error(error),
                    None,
                    store.fuel_consumed().unwrap_or(0),
                    None,
                    import_evidence,
                )
            }
        },
        Err(error) => {
            return positive_run(
                WORKSPACE_SERVICE_ID,
                WORKSPACE_FUEL_BUDGET,
                true,
                false,
                classify_workspace_error(error),
                None,
                store.fuel_consumed().unwrap_or(0),
                None,
                import_evidence,
            )
        }
    };
    let Some(function) = instance
        .get_export(&store, WORKSPACE_ENTRYPOINT)
        .and_then(Extern::into_func)
    else {
        return positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            true,
            true,
            "entrypoint_missing",
            None,
            store.fuel_consumed().unwrap_or(0),
            None,
            import_evidence,
        );
    };
    let mut output = [Value::I32(0)];
    match function.call(&mut store, &[], &mut output) {
        Ok(()) => positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            true,
            true,
            if output[0].i32().is_some() {
                "success"
            } else {
                "entrypoint_type_mismatch"
            },
            output[0].i32(),
            store.fuel_consumed().unwrap_or(0),
            None,
            import_evidence,
        ),
        Err(error) => positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            true,
            true,
            classify_workspace_error(error),
            None,
            store.fuel_consumed().unwrap_or(0),
            None,
            import_evidence,
        ),
    }
}

fn classify_workspace_error(error: wasmi::Error) -> &'static str {
    match error {
        wasmi::Error::Trap(trap) => match trap.trap_code() {
            Some(TrapCode::OutOfFuel) => "fuel_exhausted",
            Some(TrapCode::UnreachableCodeReached) => "guest_trap",
            Some(_) => "guest_trap",
            None => "guest_trap",
        },
        wasmi::Error::Memory(_) => "memory_limit_exceeded",
        wasmi::Error::Instantiation(_) => "instantiation_failed",
        _ => "workspace_run_failed",
    }
}

fn execute_validated_module_bytes(
    bytes: &[u8],
    entrypoint: &str,
    service_id: &str,
    artifact_sha256_present: bool,
    requested_imports: &[(&str, &str)],
    validation_ok: bool,
    staged_input: &[u8],
    fuel_budget: u64,
) -> EchoRunEvidence {
    if wasm_execution_busy() {
        return wasm_busy_run(service_id, fuel_budget);
    }
    let authorized =
        match authorize_wasm_imports(service_id, artifact_sha256_present, requested_imports) {
            Ok(authorized) => authorized,
            Err(decision) => {
                return positive_run(
                    service_id,
                    fuel_budget,
                    false,
                    false,
                    "import_grant_denied",
                    None,
                    0,
                    None,
                    import_grant_denied_evidence(decision),
                )
            }
        };
    if !validation_ok {
        return positive_run(
            service_id,
            fuel_budget,
            false,
            false,
            "validation_failed",
            None,
            0,
            None,
            import_grant_evidence(&authorized, 0, false, None),
        );
    }

    let wasm = Vec::from(bytes).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return positive_run(
                service_id,
                fuel_budget,
                true,
                false,
                "module_compile_failed",
                None,
                0,
                None,
                import_grant_evidence(&authorized, 0, false, None),
            )
        }
    };
    let mut store = Box::new(Store::new(&engine, buffer_state(staged_input)));
    store.limiter(|state| &mut state.limits);
    if store.add_fuel(fuel_budget).is_err() {
        return positive_run(
            service_id,
            fuel_budget,
            true,
            false,
            "fuel_metering_unavailable",
            None,
            0,
            None,
            import_grant_evidence(&authorized, 0, false, None),
        );
    }
    let mut linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    let linked_host_import_count = match define_granted_imports(&mut linker, &authorized) {
        Ok(count) => count,
        Err(reason) => {
            return positive_run(
                service_id,
                fuel_budget,
                true,
                false,
                reason,
                None,
                0,
                None,
                import_grant_evidence(&authorized, 0, false, None),
            )
        }
    };
    if let Some(missing) = first_unauthorized_module_import(&module, &authorized) {
        return positive_run(
            service_id,
            fuel_budget,
            true,
            false,
            "module_import_not_authorized",
            None,
            0,
            None,
            import_grant_evidence(&authorized, linked_host_import_count, false, Some(missing)),
        );
    }

    let instance = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(instance) => instance,
            Err(_) => {
                return positive_run(
                    service_id,
                    fuel_budget,
                    true,
                    false,
                    "instantiation_start_trap",
                    None,
                    store.fuel_consumed().unwrap_or(0),
                    store.data().log_line.clone(),
                    import_grant_evidence(&authorized, linked_host_import_count, true, None),
                )
            }
        },
        Err(_) => {
            return positive_run(
                service_id,
                fuel_budget,
                true,
                false,
                "instantiation_failed",
                None,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
                import_grant_evidence(&authorized, linked_host_import_count, true, None),
            )
        }
    };

    let Some(func) = instance
        .get_export(&*store, entrypoint)
        .and_then(Extern::into_func)
    else {
        return positive_run(
            service_id,
            fuel_budget,
            true,
            true,
            "entrypoint_missing",
            None,
            store.fuel_consumed().unwrap_or(0),
            store.data().log_line.clone(),
            import_grant_evidence(&authorized, linked_host_import_count, true, None),
        );
    };

    let mut outputs = Vec::from([Value::I32(0)]).into_boxed_slice();
    match func.call(&mut *store, &[], &mut outputs) {
        Ok(()) => {
            let return_value = outputs[0].i32();
            let outcome = if return_value.is_some() {
                "success"
            } else {
                "bad_return_type"
            };
            let mut ev = positive_run(
                service_id,
                fuel_budget,
                true,
                true,
                outcome,
                return_value,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
                import_grant_evidence(&authorized, linked_host_import_count, true, None),
            );
            let out = &store.data().captured_output;
            if out.is_empty() {
                ev.captured_output_len = 0;
                ev.captured_output_sha256 = ZERO_SHA256;
            } else {
                ev.captured_output_len = out.len() as u64;
                ev.captured_output_sha256 = sha256_bytes(out);
            }
            ev.raw_captured_output = out.clone();
            ev
        }
        Err(_) => {
            let mut ev = positive_run(
                service_id,
                fuel_budget,
                true,
                true,
                "trap",
                None,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
                import_grant_evidence(&authorized, linked_host_import_count, true, None),
            );
            let out = &store.data().captured_output;
            if out.is_empty() {
                ev.captured_output_len = 0;
                ev.captured_output_sha256 = ZERO_SHA256;
            } else {
                ev.captured_output_len = out.len() as u64;
                ev.captured_output_sha256 = sha256_bytes(out);
            }
            ev.raw_captured_output = out.clone();
            ev
        }
    }
}

fn instantiate_forbidden_import_module() -> NegativeRun {
    let wasm = Vec::from(FORBIDDEN_WRITE_WASM_MODULE).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return NegativeRun {
                validation_ok: false,
                instantiation_ok: false,
                link_error_kind: "module_compile_failed",
                missing_import_module: None,
                missing_import_name: None,
                boundary_held: false,
            }
        }
    };
    let mut store = Box::new(Store::new(&engine, default_state()));
    let mut linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    let authorized = match authorize_wasm_imports(ECHO_SERVICE_ID, true, ECHO_AUTHORIZED_IMPORTS) {
        Ok(authorized) => authorized,
        Err(_) => {
            return NegativeRun {
                validation_ok: true,
                instantiation_ok: false,
                link_error_kind: "import_grant_denied",
                missing_import_module: None,
                missing_import_name: None,
                boundary_held: false,
            }
        }
    };
    if define_granted_imports(&mut linker, &authorized).is_err() {
        return NegativeRun {
            validation_ok: true,
            instantiation_ok: false,
            link_error_kind: "missing_host_import_implementation",
            missing_import_module: None,
            missing_import_name: None,
            boundary_held: false,
        };
    }

    match linker.instantiate(&mut *store, &module) {
        Ok(_) => NegativeRun {
            validation_ok: true,
            instantiation_ok: true,
            link_error_kind: "none",
            missing_import_module: None,
            missing_import_name: None,
            boundary_held: false,
        },
        Err(wasmi::Error::Linker(LinkerError::MissingDefinition { name, .. })) => {
            let module_ok = name.module() == FORBIDDEN_IMPORT_MODULE;
            let name_ok = name.name() == FORBIDDEN_IMPORT_NAME;
            NegativeRun {
                validation_ok: true,
                instantiation_ok: false,
                link_error_kind: "missing_definition",
                missing_import_module: module_ok.then_some(FORBIDDEN_IMPORT_MODULE),
                missing_import_name: name_ok.then_some(FORBIDDEN_IMPORT_NAME),
                boundary_held: module_ok && name_ok,
            }
        }
        Err(wasmi::Error::Linker(_)) => NegativeRun {
            validation_ok: true,
            instantiation_ok: false,
            link_error_kind: "other_link_error",
            missing_import_module: None,
            missing_import_name: None,
            boundary_held: false,
        },
        Err(_) => NegativeRun {
            validation_ok: true,
            instantiation_ok: false,
            link_error_kind: "non_link_error",
            missing_import_module: None,
            missing_import_name: None,
            boundary_held: false,
        },
    }
}

pub(crate) fn forbidden_import_link_failure_evidence() -> NegativeRun {
    if wasm_execution_busy() {
        return busy_negative_run();
    }
    instantiate_forbidden_import_module()
}

fn run_hardening_cases() -> [WasmHardeningCase; WASM_HARDENING_CASE_COUNT] {
    [
        malformed_bytes_case(),
        over_memory_case(),
        fuel_exhaustion_case(),
        guest_trap_case(),
    ]
}

fn malformed_bytes_case() -> WasmHardeningCase {
    let wasm = Vec::from(MALFORMED_WASM_MODULE).into_boxed_slice();
    let engine = metered_engine();
    let actual = if Module::new(&engine, &*wasm).is_err() {
        "module_new_error"
    } else {
        "module_new_ok"
    };

    hardening_case(
        "malformed_bytes",
        "wasmi::Module::new",
        "module_new_error",
        actual,
    )
}

fn over_memory_case() -> WasmHardeningCase {
    let wasm = Vec::from(OVER_MEMORY_WASM_MODULE).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return hardening_case(
                "over_memory",
                "wasmi::StoreLimitsBuilder::memory_size+Store::limiter",
                "limiter_instantiation_error",
                "module_new_error",
            )
        }
    };
    let mut store = Box::new(Store::new(&engine, limited_state(WASM_MEMORY_PAGE_BYTES)));
    store.limiter(|state| &mut state.limits);
    let linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    let actual = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(_) => "instantiation_ok",
            Err(_) => "start_trap",
        },
        Err(wasmi::Error::Instantiation(_)) | Err(wasmi::Error::Memory(_)) => {
            "limiter_instantiation_error"
        }
        Err(_) => "other_instantiation_error",
    };

    hardening_case(
        "over_memory",
        "wasmi::StoreLimitsBuilder::memory_size+Store::limiter",
        "limiter_instantiation_error",
        actual,
    )
}

fn fuel_exhaustion_case() -> WasmHardeningCase {
    hardening_case(
        "fuel_exhaustion",
        "wasmi::Config::consume_fuel+Store::add_fuel",
        "fuel_exhausted",
        run_trap_module(
            FUEL_LOOP_WASM_MODULE,
            FUEL_EXHAUSTION_BUDGET,
            ExpectedTrap::OutOfFuel,
        ),
    )
}

fn guest_trap_case() -> WasmHardeningCase {
    hardening_case(
        "guest_trap",
        "wasm_unreachable_trap",
        "guest_trap",
        run_trap_module(
            UNREACHABLE_WASM_MODULE,
            GUEST_TRAP_FUEL_BUDGET,
            ExpectedTrap::Unreachable,
        ),
    )
}

fn run_trap_module(bytes: &[u8], fuel_budget: u64, expected: ExpectedTrap) -> &'static str {
    let wasm = Vec::from(bytes).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => return "module_new_error",
    };
    let mut store = Box::new(Store::new(&engine, default_state()));
    if store.add_fuel(fuel_budget).is_err() {
        return "fuel_metering_unavailable";
    }
    let linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    let instance = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(instance) => instance,
            Err(error) => return classify_trap_error(error, expected),
        },
        Err(_) => return "instantiation_error",
    };
    let Some(func) = instance
        .get_export(&*store, "run")
        .and_then(Extern::into_func)
    else {
        return "entrypoint_missing";
    };
    let mut outputs = Vec::<Value>::new().into_boxed_slice();
    match func.call(&mut *store, &[], &mut outputs) {
        Ok(()) => "run_success",
        Err(error) => classify_trap_error(error, expected),
    }
}

#[derive(Clone, Copy)]
enum ExpectedTrap {
    OutOfFuel,
    Unreachable,
}

fn classify_trap_error(error: wasmi::Error, expected: ExpectedTrap) -> &'static str {
    let wasmi::Error::Trap(trap) = error else {
        return "run_error";
    };
    match (expected, trap.trap_code()) {
        (ExpectedTrap::OutOfFuel, Some(TrapCode::OutOfFuel)) => "fuel_exhausted",
        (ExpectedTrap::Unreachable, Some(TrapCode::UnreachableCodeReached)) => "guest_trap",
        (_, Some(TrapCode::OutOfFuel)) => "fuel_exhausted",
        (_, Some(TrapCode::UnreachableCodeReached)) => "guest_trap",
        (_, Some(_)) => "other_trap",
        (_, None) => "trap_without_code",
    }
}

fn hardening_case(
    name: &'static str,
    mechanism: &'static str,
    expected_outcome: &'static str,
    actual_outcome: &'static str,
) -> WasmHardeningCase {
    WasmHardeningCase {
        name,
        mechanism,
        expected_outcome,
        actual_outcome,
        passed: actual_outcome == expected_outcome,
    }
}

fn positive_run(
    service_id: &str,
    fuel_budget: u64,
    validation_ok: bool,
    instantiation_ok: bool,
    run_outcome: &'static str,
    return_value: Option<i32>,
    fuel_used: u64,
    log_line: Option<String>,
    import_grant: ImportGrantEvidence,
) -> EchoRunEvidence {
    let evidence = EchoRunEvidence {
        validation_ok,
        instantiation_ok,
        run_outcome,
        return_value,
        fuel_budget,
        fuel_used,
        log_line,
        import_grant_performed: import_grant.performed,
        import_grant_status: import_grant.status,
        import_grant_reason: import_grant.reason,
        authorized_import_count: import_grant.authorized_import_count,
        authorized_import_list_sha256: import_grant.authorized_import_list_sha256,
        captured_output_len: 0,
        captured_output_sha256: ZERO_SHA256,
        raw_captured_output: Vec::new(),
        linked_host_import_count: import_grant.linked_host_import_count,
        module_imports_within_authorized_list: import_grant.module_imports_within_authorized_list,
        missing_import_module: import_grant.missing_import_module,
        missing_import_name: import_grant.missing_import_name,
    };
    let _ = (
        evidence.captured_output_len,
        evidence.captured_output_sha256,
    );
    emit_import_grant_marker(service_id, &evidence);
    evidence
}

fn wasm_busy_run(service_id: &str, fuel_budget: u64) -> EchoRunEvidence {
    positive_run(
        service_id,
        fuel_budget,
        false,
        false,
        "wasm_execution_busy",
        None,
        0,
        None,
        ImportGrantEvidence {
            performed: false,
            status: "import_grant_not_evaluated",
            reason: "wasm_execution_busy",
            authorized_import_count: 0,
            authorized_import_list_sha256: ZERO_SHA256,
            linked_host_import_count: 0,
            module_imports_within_authorized_list: false,
            missing_import_module: None,
            missing_import_name: None,
        },
    )
}

fn busy_negative_run() -> NegativeRun {
    NegativeRun {
        validation_ok: false,
        instantiation_ok: false,
        link_error_kind: "wasm_execution_busy",
        missing_import_module: None,
        missing_import_name: None,
        boundary_held: true,
    }
}

fn metered_engine() -> Box<Engine> {
    let mut config = Config::default();
    config.consume_fuel(true);
    Box::new(Engine::new(&config))
}

fn default_state() -> EnvelopeState {
    EnvelopeState {
        log_line: None,
        staged_input: Vec::new(),
        captured_output: Vec::new(),
        limits: StoreLimitsBuilder::new().build(),
    }
}

fn limited_state(memory_size: usize) -> EnvelopeState {
    EnvelopeState {
        log_line: None,
        staged_input: Vec::new(),
        captured_output: Vec::new(),
        limits: StoreLimitsBuilder::new()
            .memory_size(memory_size)
            .instances(1)
            .memories(1)
            .tables(0)
            .build(),
    }
}

fn buffer_state(staged_input: &[u8]) -> EnvelopeState {
    EnvelopeState {
        log_line: None,
        staged_input: staged_input.to_vec(),
        captured_output: Vec::new(),
        limits: StoreLimitsBuilder::new()
            .memory_size(BUFFER_SERVICE_MAX_MEMORY_BYTES)
            .instances(1)
            .memories(1)
            // Measured 2026-07-14: httphead/certspki instantiate one funcref
            // table (3 and 2 elements), bufecho/certwindow none — `.tables(0)`
            // would deny those signed guests before instantiation. Bound is the
            // measured shape plus an element cap against table.grow exhaustion.
            .tables(1)
            .table_elements(64)
            .build(),
    }
}

fn authorize_wasm_imports<'a>(
    service_id: &'a str,
    artifact_sha256_present: bool,
    requested_imports: &'a [(&'a str, &'a str)],
) -> Result<AuthorizedWasmImports<'a>, WasmImportGrantDecision> {
    let input = WasmImportGrantInput {
        service_id: Some(service_id),
        artifact_sha256_present,
        requested_imports,
        policy_allows_beyond_env: false,
    };
    let decision = evaluate_wasm_import_grant(&input);
    if !decision.performed {
        return Err(decision);
    }
    Ok(AuthorizedWasmImports {
        imports: requested_imports,
        decision,
        import_list_sha256: authorized_import_list_sha256(service_id, requested_imports),
    })
}

fn define_granted_imports(
    linker: &mut Linker<EnvelopeState>,
    authorized: &AuthorizedWasmImports<'_>,
) -> Result<u64, &'static str> {
    let mut linked = 0u64;
    let mut idx = 0usize;
    while idx < authorized.imports.len() {
        match authorized.imports[idx] {
            ("env", "log") => {
                linker
                    .func_wrap("env", "log", host_log)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("env", "counter_get") => {
                linker
                    .func_wrap("env", "counter_get", host_counter_get)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("env", "input_len") => {
                linker
                    .func_wrap("env", "input_len", host_input_len)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("env", "input_read") => {
                linker
                    .func_wrap("env", "input_read", host_input_read)
                    .map_err(|_| "host_import_link_failed")?;
            }
            ("env", "output_write") => {
                linker
                    .func_wrap("env", "output_write", host_output_write)
                    .map_err(|_| "host_import_link_failed")?;
            }
            _ => return Err("missing_host_import_implementation"),
        }
        linked += 1;
        idx += 1;
    }
    Ok(linked)
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

fn first_unauthorized_module_import(
    module: &Module,
    authorized: &AuthorizedWasmImports<'_>,
) -> Option<(String, String)> {
    for import in module.imports() {
        if !authorized_contains(authorized, import.module(), import.name()) {
            return Some((String::from(import.module()), String::from(import.name())));
        }
    }
    None
}

fn authorized_contains(authorized: &AuthorizedWasmImports<'_>, module: &str, name: &str) -> bool {
    authorized
        .imports
        .iter()
        .any(|(authorized_module, authorized_name)| {
            *authorized_module == module && *authorized_name == name
        })
}

fn import_grant_denied_evidence(decision: WasmImportGrantDecision) -> ImportGrantEvidence {
    ImportGrantEvidence {
        performed: decision.performed,
        status: decision.status,
        reason: decision.reason,
        authorized_import_count: decision.authorized_import_count as u64,
        authorized_import_list_sha256: ZERO_SHA256,
        linked_host_import_count: 0,
        module_imports_within_authorized_list: false,
        missing_import_module: None,
        missing_import_name: None,
    }
}

fn import_grant_evidence(
    authorized: &AuthorizedWasmImports<'_>,
    linked_host_import_count: u64,
    module_imports_within_authorized_list: bool,
    missing_import: Option<(String, String)>,
) -> ImportGrantEvidence {
    let (missing_import_module, missing_import_name) = match missing_import {
        Some((module, name)) => (Some(module), Some(name)),
        None => (None, None),
    };
    ImportGrantEvidence {
        performed: authorized.decision.performed,
        status: authorized.decision.status,
        reason: authorized.decision.reason,
        authorized_import_count: authorized.decision.authorized_import_count as u64,
        authorized_import_list_sha256: authorized.import_list_sha256,
        linked_host_import_count,
        module_imports_within_authorized_list,
        missing_import_module,
        missing_import_name,
    }
}

fn emit_import_grant_marker(service_id: &str, evidence: &EchoRunEvidence) {
    let mut line = String::new();
    let _ = write!(
        &mut line,
        "WASM_IMPORT_GRANT {{\"service_id\":\"{}\",\"import_grant_performed\":{},\"import_grant_status\":\"{}\",\"import_grant_reason\":\"{}\",\"authorized_import_count\":{},\"authorized_import_list_sha256\":\"sha256:",
        service_id,
        if evidence.import_grant_performed { "true" } else { "false" },
        evidence.import_grant_status,
        evidence.import_grant_reason,
        evidence.authorized_import_count
    );
    push_sha256_hex(&mut line, evidence.authorized_import_list_sha256);
    let _ = write!(
        &mut line,
        "\",\"linked_host_import_count\":{},\"module_imports_within_authorized_list\":{},\"run_outcome\":\"{}\",\"missing_import_module\":",
        evidence.linked_host_import_count,
        if evidence.module_imports_within_authorized_list {
            "true"
        } else {
            "false"
        },
        evidence.run_outcome
    );
    push_json_string_or_null(&mut line, evidence.missing_import_module.as_deref());
    line.push_str(",\"missing_import_name\":");
    push_json_string_or_null(&mut line, evidence.missing_import_name.as_deref());
    line.push('}');
    serial::write_raw_line(&line);
}

fn push_json_string_or_null(out: &mut String, value: Option<&str>) {
    let Some(value) = value else {
        out.push_str("null");
        return;
    };
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

fn push_sha256_hex(out: &mut String, value: [u8; 32]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut idx = 0usize;
    while idx < value.len() {
        let byte = value[idx];
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
        idx += 1;
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

fn host_log(mut caller: Caller<'_, EnvelopeState>, ptr: i32, len: i32) -> Result<(), Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if ptr < 0 || len < 0 {
        return Err(Trap::new("env.log negative pointer or length"));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > MAX_WASM_LOG_BYTES {
        return Err(Trap::new("env.log length exceeds 256 bytes"));
    }
    ptr.checked_add(len)
        .ok_or_else(|| Trap::new("env.log pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("env.log memory export missing"))?;
    let mut bytes = Vec::new();
    bytes.resize(len, 0);
    memory
        .read(&caller, ptr, &mut bytes)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;

    let mut line = String::new();
    let mut idx = 0usize;
    while idx < bytes.len() {
        let byte = bytes[idx];
        line.push(if (0x20..=0x7e).contains(&byte) {
            byte as char
        } else {
            ' '
        });
        idx += 1;
    }

    serial::write_raw_str("WASM_GUEST_LOG ");
    serial::write_raw_line(&line);
    caller.data_mut().log_line = Some(line);
    Ok(())
}

fn host_input_len(mut caller: Caller<'_, EnvelopeState>) -> Result<i32, Trap> {
    caller
        .consume_fuel(5)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let len = caller.data().staged_input.len();
    if len > MAX_WASM_INPUT_BYTES {
        return Err(Trap::new("env.input_len staged input exceeds max"));
    }
    Ok(len as i32)
}

fn host_input_read(mut caller: Caller<'_, EnvelopeState>, ptr: i32, len: i32) -> Result<i32, Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if ptr < 0 || len < 0 {
        return Err(Trap::new("env.input_read negative pointer or length"));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > MAX_WASM_INPUT_BYTES {
        return Err(Trap::new("env.input_read length exceeds 4096 bytes"));
    }
    let count = len.min(caller.data().staged_input.len());
    ptr.checked_add(count)
        .ok_or_else(|| Trap::new("env.input_read pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("env.input_read memory export missing"))?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&caller.data().staged_input[..count]);
    memory
        .write(&mut caller, ptr, &bytes)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    Ok(count as i32)
}

// Raw guest bytes: surfaced only as len+sha256 later. This env-module output
// channel is bounded + per-service-declared + subset-checked, and by design does
// NOT arm the policy_allows_beyond_env owner gate; move output_write to its own
// module if the owner later wants policy-gated output.
fn host_output_write(
    mut caller: Caller<'_, EnvelopeState>,
    ptr: i32,
    len: i32,
) -> Result<i32, Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if ptr < 0 || len < 0 {
        return Err(Trap::new("env.output_write negative pointer or length"));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > MAX_WASM_OUTPUT_BYTES {
        return Err(Trap::new("env.output_write length exceeds 4096 bytes"));
    }
    ptr.checked_add(len)
        .ok_or_else(|| Trap::new("env.output_write pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("env.output_write memory export missing"))?;
    let mut bytes = Vec::new();
    bytes.resize(len, 0);
    memory
        .read(&caller, ptr, &mut bytes)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    caller
        .data()
        .captured_output
        .len()
        .checked_add(len)
        .filter(|total| *total <= MAX_WASM_OUTPUT_BYTES)
        .ok_or_else(|| Trap::new("env.output_write captured output exceeds 4096 bytes"))?;
    caller.data_mut().captured_output.extend_from_slice(&bytes);
    Ok(len as i32)
}

fn host_counter_get(mut caller: Caller<'_, EnvelopeState>) -> Result<i64, Trap> {
    caller
        .consume_fuel(5)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let mut counter = CURRENT_BOOT_COUNTER.lock();
    *counter = counter.saturating_add(1);
    Ok((*counter).min(i64::MAX as u64) as i64)
}
