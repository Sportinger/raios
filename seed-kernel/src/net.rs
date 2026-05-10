extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use core::cmp;
use core::str;

use spin::Mutex;

use smoltcp::iface::{Config as IfaceConfig, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::{dhcpv4, udp};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpCidr, Ipv4Address, Ipv4Cidr};

use crate::entropy;
use crate::serial;
use crate::time;
use crate::virtio;

const DHCP_BUFFER_SIZE: usize = 1024;
const UDP_RX_METADATA: usize = 4;
const UDP_TX_METADATA: usize = 4;
const UDP_BUFFER_LEN: usize = 512;
const UDP_DNS_SOURCE_PORT: u16 = 49_152;
const DNS_QUERY_TIMEOUT_MS: u64 = 4_000;
const DNS_DEFAULT_TTL_SECS: u32 = 300;
const DNS_PORT: u16 = 53;

const DHCP_OPT_IP_LEASE_TIME: u8 = 51;
const DHCP_OPT_RENEWAL_TIME: u8 = 58;
const DHCP_OPT_REBIND_TIME: u8 = 59;

static NET_STATE: Mutex<Option<NetState>> = Mutex::new(None);

pub fn init() {
    let mut state = NET_STATE.lock();
    if state.is_some() {
        serial::write_line("virtio-net already initialised");
        return;
    }

    let Some(device_info) = virtio::net::probe() else {
        serial::write_line("virtio-net probe failed; device absent or unsupported");
        return;
    };

    let mac = EthernetAddress(device_info.mac);
    let mut iface_config = IfaceConfig::new(HardwareAddress::Ethernet(mac));
    let mut seed = [0u8; 8];
    entropy::take(&mut seed);
    iface_config.random_seed = u64::from_le_bytes(seed);

    let mut phy = VirtioPhy;
    let start_instant = Instant::from_millis(0);
    let mut iface = Interface::new(iface_config, &mut phy, start_instant);
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

    *state = Some(NetState {
        iface,
        sockets,
        dhcp_handle,
        dns_udp_handle,
        config: NetConfig::new(device_info.mac),
        dns_cache: None,
        pending_dns: None,
        next_dns_id: 1,
        lease_times: LeaseTimes::default(),
    });

    serial::write_line("virtio-net initialised; awaiting DHCP lease");
}

pub fn poll() {
    let now_ms = now_ms();
    let instant = Instant::from_millis(now_ms.min(i64::MAX as u64) as i64);

    let mut guard = NET_STATE.lock();
    let state = match guard.as_mut() {
        Some(state) => state,
        None => return,
    };

    virtio::net::poll();

    let mut phy = VirtioPhy;
    let _ = state.iface.poll(instant, &mut phy, &mut state.sockets);

    state.handle_dhcp_events(now_ms);
    state.poll_dns(now_ms);
}

#[allow(dead_code)]
pub fn config_snapshot() -> Option<NetConfigSnapshot> {
    let guard = NET_STATE.lock();
    let state = guard.as_ref()?;
    Some(state.config.snapshot())
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

struct NetState {
    iface: Interface,
    sockets: SocketSet<'static>,
    dhcp_handle: SocketHandle,
    dns_udp_handle: SocketHandle,
    config: NetConfig,
    dns_cache: Option<DnsCacheEntry>,
    pending_dns: Option<DnsQueryState>,
    next_dns_id: u16,
    lease_times: LeaseTimes,
}

impl NetState {
    fn handle_dhcp_events(&mut self, now_ms: u64) {
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
                DhcpEventOwned::Configured(cfg) => self.apply_dhcp_config(cfg, now_ms),
                DhcpEventOwned::Deconfigured => self.clear_config(),
            }
        }
    }

    fn apply_dhcp_config(&mut self, cfg: DhcpOwnedConfig, now_ms: u64) {
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
                serial::write_line("virtio-net: failed to install default route");
            }
        }

        self.config.ip = Some(address);
        self.config.gateway = router;
        self.config.dns_servers = cfg.dns_servers;
        self.lease_times = LeaseTimes::from_options(
            cfg.lease_seconds,
            cfg.renew_seconds,
            cfg.rebind_seconds,
            now_ms,
        );
        self.config.lease_expires_ms = cfg.lease_seconds.map(|secs| now_ms + secs as u64 * 1000);

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
        self.lease_times = LeaseTimes::default();
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
                    ttl,
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
            server,
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
    lease_seconds: Option<u32>,
    renew_seconds: Option<u32>,
    rebind_seconds: Option<u32>,
}

impl DhcpOwnedConfig {
    fn from_config(cfg: &dhcpv4::Config<'_>) -> Self {
        let dns_servers = cfg.dns_servers.iter().copied().collect();
        let mut lease = None;
        let mut renew = None;
        let mut rebind = None;
        if let Some(packet) = cfg.packet {
            for opt in packet.options() {
                match (opt.kind, opt.data.len()) {
                    (DHCP_OPT_IP_LEASE_TIME, 4) => {
                        lease = Some(u32::from_be_bytes([
                            opt.data[0],
                            opt.data[1],
                            opt.data[2],
                            opt.data[3],
                        ]));
                    }
                    (DHCP_OPT_RENEWAL_TIME, 4) => {
                        renew = Some(u32::from_be_bytes([
                            opt.data[0],
                            opt.data[1],
                            opt.data[2],
                            opt.data[3],
                        ]));
                    }
                    (DHCP_OPT_REBIND_TIME, 4) => {
                        rebind = Some(u32::from_be_bytes([
                            opt.data[0],
                            opt.data[1],
                            opt.data[2],
                            opt.data[3],
                        ]));
                    }
                    _ => {}
                }
            }
        }
        Self {
            address: cfg.address,
            router: cfg.router,
            dns_servers,
            lease_seconds: lease,
            renew_seconds: renew,
            rebind_seconds: rebind,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct NetConfigSnapshot {
    pub mac: [u8; 6],
    pub ip: Option<Ipv4Cidr>,
    pub gateway: Option<Ipv4Address>,
    pub dns_servers: Vec<Ipv4Address>,
    pub lease_expires_ms: Option<u64>,
}

struct NetConfig {
    mac: [u8; 6],
    ip: Option<Ipv4Cidr>,
    gateway: Option<Ipv4Address>,
    dns_servers: Vec<Ipv4Address>,
    lease_expires_ms: Option<u64>,
}

impl NetConfig {
    fn new(mac: [u8; 6]) -> Self {
        Self {
            mac,
            ip: None,
            gateway: None,
            dns_servers: Vec::new(),
            lease_expires_ms: None,
        }
    }

    fn clear(&mut self) {
        self.ip = None;
        self.gateway = None;
        self.dns_servers.clear();
        self.lease_expires_ms = None;
    }

    #[allow(dead_code)]
    fn snapshot(&self) -> NetConfigSnapshot {
        NetConfigSnapshot {
            mac: self.mac,
            ip: self.ip,
            gateway: self.gateway,
            dns_servers: self.dns_servers.clone(),
            lease_expires_ms: self.lease_expires_ms,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct LeaseTimes {
    renew_at_ms: Option<u64>,
    rebind_at_ms: Option<u64>,
    expires_at_ms: Option<u64>,
}

impl LeaseTimes {
    fn from_options(
        lease: Option<u32>,
        renew: Option<u32>,
        rebind: Option<u32>,
        now_ms: u64,
    ) -> Self {
        let lease_ms = lease.map(|secs| now_ms + secs as u64 * 1000);
        let renew_ms = renew.map(|secs| now_ms + secs as u64 * 1000);
        let rebind_ms = rebind.map(|secs| now_ms + secs as u64 * 1000);
        Self {
            renew_at_ms: renew_ms,
            rebind_at_ms: rebind_ms,
            expires_at_ms: lease_ms,
        }
    }
}

struct DnsCacheEntry {
    hostname: String,
    address: Ipv4Address,
    ttl: u32,
    expires_ms: u64,
}

struct DnsQueryState {
    hostname: String,
    server: Ipv4Address,
    tx_id: u16,
    timeout_ms: u64,
}

struct VirtioPhy;

impl Device for VirtioPhy {
    type RxToken<'a>
        = VirtioRxToken
    where
        Self: 'a;
    type TxToken<'a>
        = VirtioTxToken
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        virtio::net::poll();
        let packet = virtio::net::pop_rx_packet()?;
        Some((VirtioRxToken { packet }, VirtioTxToken::new()))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        virtio::net::poll();
        Some(VirtioTxToken::new())
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = virtio::net::MAX_FRAME_SIZE;
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps
    }
}

struct VirtioRxToken {
    packet: virtio::net::RxPacket,
}

impl RxToken for VirtioRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let packet = self.packet;
        let result = {
            let buffer = virtio::net::rx_packet_buffer(&packet);
            f(buffer)
        };
        virtio::net::recycle_rx_packet(packet);
        result
    }
}

struct VirtioTxToken {
    handle: Option<virtio::net::TxPacket>,
    buffer: Option<&'static mut [u8]>,
}

impl VirtioTxToken {
    fn new() -> Self {
        match virtio::net::alloc_tx_packet() {
            Some((handle, buffer)) => Self {
                handle: Some(handle),
                buffer: Some(buffer),
            },
            None => Self {
                handle: None,
                buffer: None,
            },
        }
    }
}

impl TxToken for VirtioTxToken {
    fn consume<R, F>(mut self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        if let (Some(handle), Some(buffer)) = (self.handle.take(), self.buffer.take()) {
            let actual_len = cmp::min(len, buffer.len());
            let result = f(&mut buffer[..actual_len]);
            if !virtio::net::submit_tx_packet(handle, actual_len) {
                virtio::net::release_tx_packet(handle);
                serial::write_line("virtio-net: submit failed; frame dropped");
            }
            result
        } else {
            let mut scratch = vec![0u8; len];
            let result = f(&mut scratch[..]);
            serial::write_line("virtio-net: TX buffers exhausted; dropping frame");
            result
        }
    }
}

struct DnsParseResult {
    address: Ipv4Address,
    ttl: u32,
}

fn build_dns_query(buffer: &mut [u8], tx_id: u16, hostname: &str) -> Option<usize> {
    if buffer.len() < 12 {
        return None;
    }

    buffer[0] = (tx_id >> 8) as u8;
    buffer[1] = tx_id as u8;
    buffer[2] = 0x01;
    buffer[3] = 0x00;
    buffer[4] = 0x00;
    buffer[5] = 0x01;
    buffer[6..12].fill(0);

    let mut offset = 12;
    for label in hostname.split('.') {
        if label.is_empty() || label.len() > 63 {
            return None;
        }
        if offset + 1 + label.len() > buffer.len() {
            return None;
        }
        buffer[offset] = label.len() as u8;
        offset += 1;
        buffer[offset..offset + label.len()].copy_from_slice(label.as_bytes());
        offset += label.len();
    }

    if offset >= buffer.len() {
        return None;
    }
    buffer[offset] = 0;
    offset += 1;

    if offset + 4 > buffer.len() {
        return None;
    }
    buffer[offset] = 0;
    buffer[offset + 1] = 1; // A record
    buffer[offset + 2] = 0;
    buffer[offset + 3] = 1; // IN class
    Some(offset + 4)
}

fn parse_dns_response<'a>(
    payload: &'a [u8],
    expected_id: u16,
    hostname: &str,
) -> Option<DnsParseResult> {
    if payload.len() < 12 {
        return None;
    }
    let id = u16::from_be_bytes([payload[0], payload[1]]);
    if id != expected_id {
        return None;
    }
    let flags = u16::from_be_bytes([payload[2], payload[3]]);
    if flags & 0x8000 == 0 || flags & 0x000f != 0 {
        return None;
    }
    let qdcount = u16::from_be_bytes([payload[4], payload[5]]) as usize;
    let ancount = u16::from_be_bytes([payload[6], payload[7]]) as usize;

    let mut offset = 12;
    for _ in 0..qdcount {
        let (name, next) = read_dns_name(payload, offset)?;
        offset = next + 4; // skip type + class
        if name != hostname {
            return None;
        }
    }

    for _ in 0..ancount {
        let (name, next) = read_dns_name(payload, offset)?;
        if next + 10 > payload.len() {
            return None;
        }
        let rtype = u16::from_be_bytes([payload[next], payload[next + 1]]);
        let class = u16::from_be_bytes([payload[next + 2], payload[next + 3]]);
        let ttl = u32::from_be_bytes([
            payload[next + 4],
            payload[next + 5],
            payload[next + 6],
            payload[next + 7],
        ]);
        let rdlength = u16::from_be_bytes([payload[next + 8], payload[next + 9]]) as usize;
        let data_offset = next + 10;
        if data_offset + rdlength > payload.len() {
            return None;
        }
        if rtype == 1 && class == 1 && rdlength == 4 && name == hostname {
            let addr = Ipv4Address::from_bytes(&payload[data_offset..data_offset + 4]);
            let ttl_effective = if ttl == 0 { DNS_DEFAULT_TTL_SECS } else { ttl };
            return Some(DnsParseResult {
                address: addr,
                ttl: ttl_effective,
            });
        }
        offset = data_offset + rdlength;
    }

    None
}

fn read_dns_name(payload: &[u8], start: usize) -> Option<(String, usize)> {
    let mut labels = Vec::new();
    let mut offset = start;
    let mut jumped = false;
    let mut jump_return = 0;
    let mut steps = 0;

    loop {
        if offset >= payload.len() {
            return None;
        }
        let len = payload[offset];
        if len & 0xC0 == 0xC0 {
            if offset + 1 >= payload.len() {
                return None;
            }
            let ptr = (((len & 0x3F) as usize) << 8) | payload[offset + 1] as usize;
            if !jumped {
                jump_return = offset + 2;
                jumped = true;
            }
            offset = ptr;
            steps += 1;
            if steps > 16 {
                return None;
            }
            continue;
        } else if len == 0 {
            offset += 1;
            break;
        } else {
            offset += 1;
            let end = offset + len as usize;
            if end > payload.len() {
                return None;
            }
            let label_bytes = &payload[offset..end];
            let label = str::from_utf8(label_bytes).ok()?;
            labels.push(label);
            offset = end;
        }
    }

    let mut name = String::new();
    for (i, part) in labels.iter().enumerate() {
        if i > 0 {
            name.push('.');
        }
        name.push_str(part);
    }

    let next_offset = if jumped { jump_return } else { offset };
    Some((name, next_offset))
}

fn now_ms() -> u64 {
    let per_ms = time::tsc_per_ms().max(1);
    time::rdtsc() / per_ms
}
