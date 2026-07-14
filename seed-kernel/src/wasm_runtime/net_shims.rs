use super::*;
use super::{envelope::ZERO_SHA256, invocation::*, suspension::HostSuspend};

// Labeled NET-4 test infrastructure only. The four closures are the real v1
// implementations, but this fixture is their sole linker until owner arming.
// It opens QEMU slirp DNS, sends one TCP DNS query, receives answer bytes, and
// closes twice to prove immediate owner-idempotent cleanup.
pub(super) const NET_SHIM_WASM_MODULE: &[u8] = &[
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
const NET_W7_ADDRESS: smoltcp::wire::Ipv4Address = smoltcp::wire::Ipv4Address::new(10, 0, 2, 100);
const NET_W7_PORT: u16 = 8443;

pub(super) static NET_SHIM_PROBE: Mutex<NetShimProbeSnapshot> =
    Mutex::new(NetShimProbeSnapshot::empty());

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NetFixtureScenario {
    Responsive,
    SilentTimeout,
    SilentKill,
    W7Live,
}

impl NetFixtureScenario {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Responsive => "responsive",
            Self::SilentTimeout => "silent_timeout",
            Self::SilentKill => "silent_kill",
            Self::W7Live => "w7_live",
        }
    }

    const fn address(self) -> smoltcp::wire::Ipv4Address {
        match self {
            Self::Responsive => NET_RESPONSIVE_ADDRESS,
            Self::SilentTimeout | Self::SilentKill => NET_SILENT_ADDRESS,
            Self::W7Live => NET_W7_ADDRESS,
        }
    }

    const fn port(self) -> u16 {
        match self {
            Self::W7Live => NET_W7_PORT,
            _ => NET_DNS_PORT,
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

pub(super) struct NetCompletion {
    pub(super) result: i32,
    deadline_ms: u64,
    kind: PendingNetworkKind,
    pub(super) recv_ptr: Option<usize>,
    pub(super) recv_len: usize,
    pub(super) buffer: [u8; NET_CALL_MAX_BYTES],
}

pub(super) struct NetInvocationState {
    scenario: NetFixtureScenario,
    address: smoltcp::wire::Ipv4Address,
    port: u16,
    owner: net::TransportOwner,
    pub(super) lease: Option<net::TransportLeaseToken>,
    pending: Option<PendingNetworkOperation>,
    next_operation_id: u32,
    tx_bytes: u64,
    rx_bytes: u64,
    first_negative_result: Option<i32>,
    connection_opened: bool,
    close_call_count: u32,
    would_block_guest_visible: bool,
    preheld_native_lease: Option<net::TransportLeaseToken>,
}

impl NetInvocationState {
    pub(super) fn new(scenario: NetFixtureScenario, invocation_id: u64) -> Self {
        Self {
            scenario,
            address: scenario.address(),
            port: scenario.port(),
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
            preheld_native_lease: None,
        }
    }

    pub(super) fn new_w7(
        source: super::acquisition_service::W7SourcePolicy,
        invocation_id: u64,
    ) -> Option<Self> {
        if source.id != super::acquisition_service::NET_8_W7_SOURCE_POLICY_ID
            || source.address != [10, 0, 2, 100]
            || source.port != NET_W7_PORT
            || source.sni != super::acquisition_service::NET_8_W7_SNI
        {
            return None;
        }
        let mut state = Self::new(NetFixtureScenario::W7Live, invocation_id);
        state.address = smoltcp::wire::Ipv4Address::new(
            source.address[0],
            source.address[1],
            source.address[2],
            source.address[3],
        );
        state.port = source.port;
        Some(state)
    }

    pub(super) fn new_w7_provider_busy(
        source: super::acquisition_service::W7SourcePolicy,
        invocation_id: u64,
        now_ms: u64,
    ) -> Option<Self> {
        let mut state = Self::new_w7(source, invocation_id)?;
        state.preheld_native_lease = net::tcp_claim(
            net::NATIVE_OPENAI_TRANSPORT_OWNER,
            now_ms,
            NET_TOTAL_TIMEOUT_MS,
        )
        .ok();
        state.preheld_native_lease.map(|_| state)
    }

    fn next_operation(&mut self) -> u32 {
        let operation_id = self.next_operation_id;
        self.next_operation_id = self.next_operation_id.saturating_add(1);
        operation_id
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
    pub(super) const fn empty() -> Self {
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

pub(super) struct NetTerminalState {
    pub(super) tx_bytes: u64,
    pub(super) rx_bytes: u64,
    pub(super) first_negative_result: Option<i32>,
    pub(super) close_call_count: u32,
    pub(super) would_block_guest_visible: bool,
}

pub(super) fn progress_net_operation(
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
            let (scenario, address, port, owner, token) = {
                let net_state = store
                    .data()
                    .net
                    .as_ref()
                    .ok_or(TerminalOutcome::HostError)?;
                (
                    net_state.scenario,
                    net_state.address,
                    net_state.port,
                    net_state.owner,
                    net_state.lease,
                )
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
                        if scenario == NetFixtureScenario::W7Live {
                            super::acquisition_service::note_w7_lease_held();
                        }
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
            match net::tcp_connect_step(token, address, port, now_ms) {
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

pub(super) fn check_net_resume_boundary(
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

pub(super) fn finish_net_resources(
    store: &mut Store<BeyondEnvState>,
    now_ms: u64,
) -> Option<NetTerminalState> {
    let net_state = store.data_mut().net.as_mut()?;
    let token = net_state.lease.take();
    let preheld_native_lease = net_state.preheld_native_lease.take();
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
    if let Some(token) = preheld_native_lease {
        let _ = net::tcp_abort(token, now_ms);
    }
    Some(terminal)
}

pub(crate) fn probe_w7_blocks_provider() -> &'static str {
    let now_ms = runtime_now_ms();
    match net::tcp_claim(net::NATIVE_OPENAI_TRANSPORT_OWNER, now_ms, 1_000) {
        Err(error) => error.as_str(),
        Ok(unexpected) => {
            let _ = net::tcp_abort(unexpected, now_ms);
            "unexpected_native_claim_success"
        }
    }
}

pub(super) fn host_net_tcp_open(mut caller: Caller<'_, BeyondEnvState>) -> Result<i32, Trap> {
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

pub(super) fn host_net_tcp_send(
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

pub(super) fn host_net_tcp_recv(
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

pub(super) fn host_net_tcp_close(
    mut caller: Caller<'_, BeyondEnvState>,
    handle: i32,
) -> Result<i32, Trap> {
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
