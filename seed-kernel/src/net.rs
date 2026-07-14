extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use core::cmp;
use spin::Mutex;

use raios_core::dns_parse::build_dns_query;
use smoltcp::iface::{Config as IfaceConfig, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::{dhcpv4, tcp, udp};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address, Ipv4Cidr};

use crate::e1000;
use crate::entropy;
use crate::marvell_wifi_pcie;
use crate::serial;
use crate::time;

const DHCP_BUFFER_SIZE: usize = 1024;
const UDP_RX_METADATA: usize = 4;
const UDP_TX_METADATA: usize = 4;
const UDP_BUFFER_LEN: usize = 512;
const TCP_RX_BUFFER_LEN: usize = 4096;
const TCP_TX_BUFFER_LEN: usize = 4096;
const UDP_DNS_SOURCE_PORT: u16 = 49_152;
const TCP_SOURCE_PORT_START: u16 = 49_200;
const TCP_SOURCE_PORT_END: u16 = 65_535;
const DNS_QUERY_TIMEOUT_MS: u64 = 4_000;
const DNS_PORT: u16 = 53;
static NET_STATE: Mutex<Option<NetState>> = Mutex::new(None);

pub fn init() {
    let mut state = NET_STATE.lock();
    if state.is_some() {
        serial::write_line("network already initialised");
        return;
    }

    let Some(device_info) = e1000::probe() else {
        serial::write_line("e1000 absent; network stack waiting for an authenticated WiFi link");
        return;
    };

    let mut phy = E1000Phy;
    *state = Some(new_net_state(device_info.mac, NetBackend::E1000, &mut phy));

    serial::write_line("e1000 network initialised; DHCP polling enabled");
}

pub fn attach_wifi(mac: [u8; 6]) {
    if !marvell_wifi_pcie::data_link_ready() || mac == [0; 6] || mac == [0xff; 6] {
        return;
    }

    let mut state = NET_STATE.lock();
    if state
        .as_ref()
        .map(|current| current.backend == NetBackend::Wifi)
        .unwrap_or(false)
    {
        return;
    }
    if state.is_some() {
        serial::write_line("authenticated WiFi link ready; keeping active e1000 network backend");
        return;
    }

    let mut phy = WifiPhy;
    *state = Some(new_net_state(mac, NetBackend::Wifi, &mut phy));
    serial::write_line("authenticated Marvell WiFi link attached; DHCP polling enabled");
}

pub fn detach_wifi() {
    let mut state = NET_STATE.lock();
    if state
        .as_ref()
        .map(|current| current.backend == NetBackend::Wifi)
        .unwrap_or(false)
    {
        *state = None;
        serial::write_line("Marvell WiFi link removed; network state cleared");
    }
}

fn new_net_state<D: Device>(mac_bytes: [u8; 6], backend: NetBackend, phy: &mut D) -> NetState {
    let mac = EthernetAddress(mac_bytes);
    let mut iface_config = IfaceConfig::new(HardwareAddress::Ethernet(mac));
    let mut seed = [0u8; 8];
    entropy::take(&mut seed);
    iface_config.random_seed = u64::from_le_bytes(seed);

    let start_instant = Instant::from_millis(0);
    let mut iface = Interface::new(iface_config, phy, start_instant);
    iface.update_ip_addrs(|addrs| addrs.clear());

    let mut sockets = SocketSet::new(Vec::new());

    let dhcp_buffer = vec![0u8; DHCP_BUFFER_SIZE].into_boxed_slice();
    let mut dhcp_socket = dhcpv4::Socket::new();
    dhcp_socket.set_receive_packet_buffer(Box::leak(dhcp_buffer));
    let dhcp_handle = sockets.add(dhcp_socket);

    let udp_rx = udp::PacketBuffer::new(
        vec![udp::PacketMetadata::EMPTY; UDP_RX_METADATA],
        vec![0u8; UDP_BUFFER_LEN],
    );
    let udp_tx = udp::PacketBuffer::new(
        vec![udp::PacketMetadata::EMPTY; UDP_TX_METADATA],
        vec![0u8; UDP_BUFFER_LEN],
    );
    let mut dns_udp = udp::Socket::new(udp_rx, udp_tx);
    dns_udp
        .bind(UDP_DNS_SOURCE_PORT)
        .expect("bind DNS UDP socket");
    let dns_udp_handle = sockets.add(dns_udp);

    let tcp_rx = tcp::SocketBuffer::new(vec![0u8; TCP_RX_BUFFER_LEN]);
    let tcp_tx = tcp::SocketBuffer::new(vec![0u8; TCP_TX_BUFFER_LEN]);
    let tcp_socket = tcp::Socket::new(tcp_rx, tcp_tx);
    let tcp_handle = sockets.add(tcp_socket);

    NetState {
        backend,
        iface,
        sockets,
        dhcp_handle,
        dns_udp_handle,
        tcp_handle,
        config: NetConfig::new(mac_bytes),
        dns_cache: None,
        pending_dns: None,
        next_dns_id: 1,
        next_tcp_port: TCP_SOURCE_PORT_START,
    }
}

pub fn poll() {
    let now_ms = now_ms();
    let instant = Instant::from_millis(now_ms.min(i64::MAX as u64) as i64);

    let mut guard = NET_STATE.lock();
    let state = match guard.as_mut() {
        Some(state) => state,
        None => return,
    };

    match state.backend {
        NetBackend::E1000 => {
            let mut phy = E1000Phy;
            let _ = state.iface.poll(instant, &mut phy, &mut state.sockets);
        }
        NetBackend::Wifi => {
            let mut phy = WifiPhy;
            let _ = state.iface.poll(instant, &mut phy, &mut state.sockets);
        }
    }

    state.handle_dhcp_events();
    state.poll_dns(now_ms);
}

pub fn ui_snapshot() -> Option<NetUiSnapshot> {
    let guard = NET_STATE.lock();
    let state = guard.as_ref()?;
    Some(NetUiSnapshot {
        mac: state.config.mac,
        ip: state.config.ip,
        gateway: state.config.gateway,
    })
}

pub fn dhcp_poll_enabled() -> bool {
    true
}

#[derive(Clone, Copy)]
pub struct NetUiSnapshot {
    pub mac: [u8; 6],
    pub ip: Option<Ipv4Cidr>,
    pub gateway: Option<Ipv4Address>,
}

pub fn resolve_hostname(hostname: &str) -> Option<Ipv4Address> {
    let now_ms = now_ms();
    let mut guard = NET_STATE.lock();
    let state = guard.as_mut()?;

    if let Some(cache) = state.dns_cache.as_ref() {
        if cache.hostname == hostname && cache.expires_ms > now_ms {
            return Some(cache.address);
        }
    }

    if state.config.dns_servers.is_empty() {
        return None;
    }

    if state
        .pending_dns
        .as_ref()
        .map(|q| q.hostname == hostname)
        .unwrap_or(false)
    {
        return None;
    }

    state.start_dns_query(hostname, now_ms);
    None
}

pub fn tcp_connect_ipv4(address: Ipv4Address, port: u16) -> TcpConnectResult {
    let mut guard = NET_STATE.lock();
    let Some(state) = guard.as_mut() else {
        return TcpConnectResult::NetworkUnavailable;
    };

    if state.config.ip.is_none() || state.config.gateway.is_none() {
        return TcpConnectResult::NetworkUnconfigured;
    }

    let tcp_handle = state.tcp_handle;
    let socket = state.sockets.get_mut::<tcp::Socket>(tcp_handle);
    if socket.may_send() {
        return TcpConnectResult::Connected;
    }
    if socket.is_active() {
        return TcpConnectResult::Connecting(tcp_state_name(socket.state()));
    }
    if socket.is_open() {
        socket.abort();
    }

    let local_port = state.next_tcp_port;
    state.next_tcp_port = if state.next_tcp_port == TCP_SOURCE_PORT_END {
        TCP_SOURCE_PORT_START
    } else {
        state.next_tcp_port + 1
    };

    let cx = state.iface.context();
    match socket.connect(cx, (IpAddress::Ipv4(address), port), local_port) {
        Ok(()) => TcpConnectResult::Started,
        Err(_) => TcpConnectResult::ConnectError,
    }
}

pub fn tcp_send(data: &[u8]) -> TcpIoResult {
    if data.is_empty() {
        return TcpIoResult::Ready(0);
    }

    let mut guard = NET_STATE.lock();
    let Some(state) = guard.as_mut() else {
        return TcpIoResult::Unavailable;
    };

    let socket = state.sockets.get_mut::<tcp::Socket>(state.tcp_handle);
    if socket.can_send() {
        match socket.send_slice(data) {
            Ok(written) if written > 0 => TcpIoResult::Ready(written),
            Ok(_) => TcpIoResult::WouldBlock,
            Err(_) => TcpIoResult::Closed,
        }
    } else if socket.may_send() {
        TcpIoResult::WouldBlock
    } else {
        TcpIoResult::Closed
    }
}

pub fn tcp_recv(buffer: &mut [u8]) -> TcpIoResult {
    if buffer.is_empty() {
        return TcpIoResult::Ready(0);
    }

    let mut guard = NET_STATE.lock();
    let Some(state) = guard.as_mut() else {
        return TcpIoResult::Unavailable;
    };

    let socket = state.sockets.get_mut::<tcp::Socket>(state.tcp_handle);
    if socket.can_recv() {
        match socket.recv_slice(buffer) {
            Ok(read) if read > 0 => TcpIoResult::Ready(read),
            Ok(_) => TcpIoResult::WouldBlock,
            Err(_) => TcpIoResult::Closed,
        }
    } else if socket.may_recv() {
        TcpIoResult::WouldBlock
    } else {
        TcpIoResult::Closed
    }
}

pub fn tcp_abort() {
    let mut guard = NET_STATE.lock();
    let Some(state) = guard.as_mut() else {
        return;
    };

    let socket = state.sockets.get_mut::<tcp::Socket>(state.tcp_handle);
    if socket.is_open() {
        socket.abort();
    }
}

pub fn tcp_snapshot() -> Option<TcpSnapshot> {
    let mut guard = NET_STATE.lock();
    let state = guard.as_mut()?;
    let socket = state.sockets.get_mut::<tcp::Socket>(state.tcp_handle);
    Some(TcpSnapshot {
        state: tcp_state_name(socket.state()),
        may_send: socket.may_send(),
        may_recv: socket.may_recv(),
    })
}

#[derive(Clone, Copy)]
pub enum TcpConnectResult {
    NetworkUnavailable,
    NetworkUnconfigured,
    Started,
    Connecting(&'static str),
    Connected,
    ConnectError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TcpIoResult {
    Ready(usize),
    WouldBlock,
    Closed,
    Unavailable,
}

#[derive(Clone, Copy)]
pub struct TcpSnapshot {
    pub state: &'static str,
    pub may_send: bool,
    pub may_recv: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetBackend {
    E1000,
    Wifi,
}

struct NetState {
    backend: NetBackend,
    iface: Interface,
    sockets: SocketSet<'static>,
    dhcp_handle: SocketHandle,
    dns_udp_handle: SocketHandle,
    tcp_handle: SocketHandle,
    config: NetConfig,
    dns_cache: Option<DnsCacheEntry>,
    pending_dns: Option<DnsQueryState>,
    next_dns_id: u16,
    next_tcp_port: u16,
}

impl NetState {
    fn handle_dhcp_events(&mut self) {
        let event_owned = {
            let socket = self.sockets.get_mut::<dhcpv4::Socket>(self.dhcp_handle);
            match socket.poll() {
                Some(dhcpv4::Event::Configured(cfg)) => {
                    let owned = DhcpOwnedConfig::from_config(&cfg);
                    Some(DhcpEventOwned::Configured(owned))
                }
                Some(dhcpv4::Event::Deconfigured) => Some(DhcpEventOwned::Deconfigured),
                None => None,
            }
        };

        if let Some(event) = event_owned {
            match event {
                DhcpEventOwned::Configured(cfg) => self.apply_dhcp_config(cfg),
                DhcpEventOwned::Deconfigured => self.clear_config(),
            }
        }
    }

    fn apply_dhcp_config(&mut self, cfg: DhcpOwnedConfig) {
        let address = cfg.address;
        let router = cfg.router;
        let dns_servers = cfg.dns_servers.clone();

        self.iface.update_ip_addrs(|addrs| {
            addrs.clear();
            addrs.push(IpCidr::Ipv4(address)).ok();
        });

        let routes = self.iface.routes_mut();
        routes.remove_default_ipv4_route();
        if let Some(gateway) = router {
            if routes.add_default_ipv4_route(gateway).is_err() {
                serial::write_line("network: failed to install default route");
            }
        }

        self.config.ip = Some(address);
        self.config.gateway = router;
        self.config.dns_servers = cfg.dns_servers;

        serial::write_fmt(format_args!(
            "DHCP lease acquired: ip {}/{} gw {} dns {:?}\r\n",
            address.address(),
            address.prefix_len(),
            router
                .map(|r| format!("{}", r))
                .unwrap_or_else(|| "none".into()),
            dns_servers
                .iter()
                .map(|d| format!("{}", d))
                .collect::<Vec<_>>()
        ));
    }

    fn clear_config(&mut self) {
        self.iface.update_ip_addrs(|addrs| addrs.clear());
        self.iface.routes_mut().remove_default_ipv4_route();
        self.config.clear();
        self.dns_cache = None;
        self.pending_dns = None;
        serial::write_line("DHCP lease lost; interface deconfigured");
    }

    fn poll_dns(&mut self, now_ms: u64) {
        let Some(query) = self.pending_dns.as_mut() else {
            return;
        };

        let socket = self.sockets.get_mut::<udp::Socket>(self.dns_udp_handle);
        while let Ok((payload, endpoint)) = socket.recv() {
            if endpoint.endpoint.port != DNS_PORT {
                continue;
            }
            if let Some(result) = parse_dns_response(payload, query.tx_id, &query.hostname) {
                let ttl = result.ttl.max(1);
                let expires = now_ms + ttl as u64 * 1000;
                serial::write_fmt(format_args!(
                    "DNS {} resolved to {} (ttl {}s)\r\n",
                    query.hostname, result.address, ttl
                ));
                self.dns_cache = Some(DnsCacheEntry {
                    hostname: query.hostname.clone(),
                    address: result.address,
                    expires_ms: expires,
                });
                self.pending_dns = None;
                return;
            }
        }

        if now_ms >= query.timeout_ms {
            serial::write_fmt(format_args!(
                "DNS query for {} timed out\r\n",
                query.hostname
            ));
            self.pending_dns = None;
        }
    }

    fn start_dns_query(&mut self, hostname: &str, now_ms: u64) {
        let server = match self.config.dns_servers.first().copied() {
            Some(addr) => addr,
            None => return,
        };

        let tx_id = self.next_dns_id;
        self.next_dns_id = self.next_dns_id.wrapping_add(1).max(1);

        let mut query_buffer = [0u8; UDP_BUFFER_LEN];
        let Some(encoded_len) = build_dns_query(&mut query_buffer, tx_id, hostname) else {
            serial::write_line("DNS query build failed (name too long)");
            return;
        };

        let socket = self.sockets.get_mut::<udp::Socket>(self.dns_udp_handle);
        if socket
            .send_slice(&query_buffer[..encoded_len], (server, DNS_PORT))
            .is_err()
        {
            serial::write_line("DNS UDP socket busy; retry later");
            return;
        }

        serial::write_fmt(format_args!(
            "DNS query sent for {} -> {}\r\n",
            hostname, server
        ));

        self.pending_dns = Some(DnsQueryState {
            hostname: hostname.into(),
            tx_id,
            timeout_ms: now_ms + DNS_QUERY_TIMEOUT_MS,
        });
    }
}

enum DhcpEventOwned {
    Configured(DhcpOwnedConfig),
    Deconfigured,
}

struct DhcpOwnedConfig {
    address: Ipv4Cidr,
    router: Option<Ipv4Address>,
    dns_servers: Vec<Ipv4Address>,
}

impl DhcpOwnedConfig {
    fn from_config(cfg: &dhcpv4::Config<'_>) -> Self {
        let dns_servers = cfg.dns_servers.iter().copied().collect();
        Self {
            address: cfg.address,
            router: cfg.router,
            dns_servers,
        }
    }
}

struct NetConfig {
    mac: [u8; 6],
    ip: Option<Ipv4Cidr>,
    gateway: Option<Ipv4Address>,
    dns_servers: Vec<Ipv4Address>,
}

impl NetConfig {
    fn new(mac: [u8; 6]) -> Self {
        Self {
            mac,
            ip: None,
            gateway: None,
            dns_servers: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.ip = None;
        self.gateway = None;
        self.dns_servers.clear();
    }
}

struct DnsCacheEntry {
    hostname: String,
    address: Ipv4Address,
    expires_ms: u64,
}

struct DnsQueryState {
    hostname: String,
    tx_id: u16,
    timeout_ms: u64,
}

struct E1000Phy;

impl Device for E1000Phy {
    type RxToken<'a>
        = E1000RxToken
    where
        Self: 'a;
    type TxToken<'a>
        = E1000TxToken
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let packet = e1000::receive()?;
        Some((E1000RxToken { packet }, E1000TxToken))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(E1000TxToken)
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = e1000::MAX_FRAME_SIZE;
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps
    }
}

struct E1000RxToken {
    packet: e1000::RxPacket,
}

impl RxToken for E1000RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut packet = self.packet;
        f(packet.as_mut_slice())
    }
}

struct E1000TxToken;

impl TxToken for E1000TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let actual_len = cmp::min(len, e1000::MAX_FRAME_SIZE);
        let mut scratch = vec![0u8; actual_len];
        let result = f(&mut scratch[..]);
        if !e1000::transmit(&scratch[..]) {
            serial::write_line("e1000: submit failed; frame dropped");
        }
        result
    }
}

struct WifiPhy;

impl Device for WifiPhy {
    type RxToken<'a>
        = WifiRxToken
    where
        Self: 'a;
    type TxToken<'a>
        = WifiTxToken
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let packet = marvell_wifi_pcie::receive_ethernet()?;
        Some((WifiRxToken { packet }, WifiTxToken))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        marvell_wifi_pcie::data_link_ready().then_some(WifiTxToken)
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = marvell_wifi_pcie::MAX_ETHERNET_FRAME_SIZE;
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps
    }
}

struct WifiRxToken {
    packet: marvell_wifi_pcie::RxPacket,
}

impl RxToken for WifiRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut packet = self.packet;
        f(packet.as_mut_slice())
    }
}

struct WifiTxToken;

impl TxToken for WifiTxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let actual_len = cmp::min(len, marvell_wifi_pcie::MAX_ETHERNET_FRAME_SIZE);
        let mut scratch = vec![0u8; actual_len];
        let result = f(&mut scratch[..]);
        if !marvell_wifi_pcie::transmit_ethernet(&scratch) {
            serial::write_line("marvell wifi: TX ring unavailable; frame dropped");
        }
        result
    }
}

struct DnsParseResult {
    address: Ipv4Address,
    ttl: u32,
}

fn parse_dns_response(payload: &[u8], expected_id: u16, hostname: &str) -> Option<DnsParseResult> {
    let result = raios_core::dns_parse::parse_dns_response(payload, expected_id, hostname)?;
    Some(DnsParseResult {
        address: Ipv4Address::from_bytes(&result.address),
        ttl: result.ttl,
    })
}

fn now_ms() -> u64 {
    let per_ms = time::tsc_per_ms().max(1);
    time::rdtsc() / per_ms
}

fn tcp_state_name(state: tcp::State) -> &'static str {
    match state {
        tcp::State::Closed => "CLOSED",
        tcp::State::Listen => "LISTEN",
        tcp::State::SynSent => "SYN-SENT",
        tcp::State::SynReceived => "SYN-RECEIVED",
        tcp::State::Established => "ESTABLISHED",
        tcp::State::FinWait1 => "FIN-WAIT-1",
        tcp::State::FinWait2 => "FIN-WAIT-2",
        tcp::State::CloseWait => "CLOSE-WAIT",
        tcp::State::Closing => "CLOSING",
        tcp::State::LastAck => "LAST-ACK",
        tcp::State::TimeWait => "TIME-WAIT",
    }
}
