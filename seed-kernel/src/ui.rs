use core::fmt::{self, Write};
use core::str;

use crate::framebuffer::{Color, FramebufferInfo, FramebufferSurface};
use crate::{console, entropy, input, net, serial, text, virtio};

#[derive(Clone, Copy)]
pub struct RuntimeStatus {
    pub virtio_rng_probe_complete: bool,
    pub virtio_net_probe_complete: bool,
    pub input_probe_complete: bool,
}

impl RuntimeStatus {
    pub const fn new() -> Self {
        Self {
            virtio_rng_probe_complete: false,
            virtio_net_probe_complete: false,
            input_probe_complete: false,
        }
    }
}

pub struct StatusUi {
    surface: Option<FramebufferSurface>,
    last_states: Option<SnapshotStates>,
}

impl StatusUi {
    pub fn new(surface: Option<FramebufferSurface>) -> Self {
        Self {
            surface,
            last_states: None,
        }
    }

    pub fn render(&mut self, uptime_ms: u64, runtime: RuntimeStatus) {
        let framebuffer = self.surface.as_ref().map(|surface| surface.info());
        let snapshot = Snapshot::collect(framebuffer, runtime);
        self.log_transitions(&snapshot);

        if let Some(surface) = self.surface.as_mut() {
            draw(surface, uptime_ms, &snapshot);
            surface.present();
        }
    }

    fn log_transitions(&mut self, snapshot: &Snapshot) {
        let states = snapshot.states();
        let previous = self.last_states;

        log_transition(previous.map(|prev| prev.framebuffer), &snapshot.framebuffer);
        log_transition(previous.map(|prev| prev.entropy), &snapshot.entropy);
        log_transition(previous.map(|prev| prev.virtio_rng), &snapshot.virtio_rng);
        log_transition(previous.map(|prev| prev.virtio_net), &snapshot.virtio_net);
        log_transition(previous.map(|prev| prev.input), &snapshot.input);

        self.last_states = Some(states);
    }
}

struct Snapshot {
    framebuffer: StatusLine,
    entropy: StatusLine,
    virtio_rng: StatusLine,
    virtio_net: StatusLine,
    input: StatusLine,
}

impl Snapshot {
    fn collect(framebuffer: Option<FramebufferInfo>, runtime: RuntimeStatus) -> Self {
        let framebuffer = framebuffer_line(framebuffer);
        let entropy = entropy_line();
        let virtio_rng = virtio_rng_line(runtime);
        let virtio_net = virtio_net_line(runtime);
        let input = input_line(runtime);
        Self {
            framebuffer,
            entropy,
            virtio_rng,
            virtio_net,
            input,
        }
    }

    fn states(&self) -> SnapshotStates {
        SnapshotStates {
            framebuffer: self.framebuffer.state,
            entropy: self.entropy.state,
            virtio_rng: self.virtio_rng.state,
            virtio_net: self.virtio_net.state,
            input: self.input.state,
        }
    }
}

#[derive(Clone, Copy)]
struct SnapshotStates {
    framebuffer: RowState,
    entropy: RowState,
    virtio_rng: RowState,
    virtio_net: RowState,
    input: RowState,
}

struct StatusLine {
    label: &'static str,
    state: RowState,
    detail: TextBuf<128>,
}

impl StatusLine {
    fn new(label: &'static str, state: RowState, detail: TextBuf<128>) -> Self {
        Self {
            label,
            state,
            detail,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RowState {
    Ready,
    Waiting,
    Configured,
    Degraded,
    Missing,
}

impl RowState {
    fn as_str(self) -> &'static str {
        match self {
            RowState::Ready => "READY",
            RowState::Waiting => "WAITING",
            RowState::Configured => "CONFIGURED",
            RowState::Degraded => "DEGRADED",
            RowState::Missing => "MISSING",
        }
    }

    fn badge_color(self) -> Color {
        match self {
            RowState::Ready => Color::new(42, 128, 92),
            RowState::Waiting => Color::new(150, 111, 42),
            RowState::Configured => Color::new(36, 112, 154),
            RowState::Degraded => Color::new(154, 91, 42),
            RowState::Missing => Color::new(142, 62, 62),
        }
    }
}

fn framebuffer_line(info: Option<FramebufferInfo>) -> StatusLine {
    match info {
        Some(info) => StatusLine::new(
            "FRAMEBUFFER",
            RowState::Ready,
            detail(format_args!(
                "{}x{} PITCH {}",
                info.width, info.height, info.pitch
            )),
        ),
        None => StatusLine::new(
            "FRAMEBUFFER",
            RowState::Missing,
            detail(format_args!("LIMINE FRAMEBUFFER UNAVAILABLE")),
        ),
    }
}

fn entropy_line() -> StatusLine {
    let stats = entropy::stats();
    let state = if stats.ready {
        RowState::Ready
    } else {
        RowState::Waiting
    };
    let mut line = detail(format_args!(
        "FILL {}/{} TOTAL {} SRC ",
        stats.pool_fill,
        entropy::POOL_CAPACITY,
        stats.total_collected
    ));
    append_entropy_sources(&mut line, stats);
    StatusLine::new("ENTROPY", state, line)
}

fn virtio_rng_line(runtime: RuntimeStatus) -> StatusLine {
    if entropy::stats().used_virtio {
        return StatusLine::new(
            "VIRTIO-RNG",
            RowState::Ready,
            detail(format_args!("ATTACHED AS ENTROPY SOURCE")),
        );
    }

    if entropy::virtio_source_attached() {
        return StatusLine::new(
            "VIRTIO-RNG",
            RowState::Degraded,
            detail(format_args!("ATTACHED, WAITING FOR DATA")),
        );
    }

    if runtime.virtio_rng_probe_complete {
        StatusLine::new(
            "VIRTIO-RNG",
            RowState::Missing,
            detail(format_args!("DEVICE ABSENT OR UNSUPPORTED")),
        )
    } else {
        StatusLine::new(
            "VIRTIO-RNG",
            RowState::Waiting,
            detail(format_args!("PROBING PCI BUS")),
        )
    }
}

fn virtio_net_line(runtime: RuntimeStatus) -> StatusLine {
    if let Some(config) = net::ui_snapshot() {
        if let Some(ip) = config.ip {
            let gateway = match config.gateway {
                Some(gateway) => detail(format_args!("{}", gateway)),
                None => detail(format_args!("NONE")),
            };
            return StatusLine::new(
                "VIRTIO-NET",
                RowState::Configured,
                detail(format_args!(
                    "IP {}/{} GW {}",
                    ip.address(),
                    ip.prefix_len(),
                    gateway.as_str()
                )),
            );
        }

        return StatusLine::new(
            "VIRTIO-NET",
            RowState::Waiting,
            if net::dhcp_poll_enabled() {
                detail(format_args!("MAC {} AWAITING DHCP", Mac(config.mac)))
            } else {
                detail(format_args!("MAC {} DHCP DEFERRED", Mac(config.mac)))
            },
        );
    }

    if let Some(info) = virtio::net::info() {
        return StatusLine::new(
            "VIRTIO-NET",
            RowState::Waiting,
            detail(format_args!("MAC {} DEVICE READY", Mac(info.mac))),
        );
    }

    if runtime.virtio_net_probe_complete {
        StatusLine::new(
            "VIRTIO-NET",
            RowState::Missing,
            detail(format_args!("DEVICE ABSENT OR UNSUPPORTED")),
        )
    } else if entropy::is_ready() {
        StatusLine::new(
            "VIRTIO-NET",
            RowState::Waiting,
            detail(format_args!("PROBE PENDING")),
        )
    } else {
        StatusLine::new(
            "VIRTIO-NET",
            RowState::Waiting,
            detail(format_args!("WAITING ENTROPY")),
        )
    }
}

fn input_line(runtime: RuntimeStatus) -> StatusLine {
    if input::device_present() {
        return StatusLine::new(
            "INPUT",
            RowState::Ready,
            detail(format_args!("VIRTIO INPUT QUEUE ACTIVE")),
        );
    }

    if runtime.input_probe_complete {
        StatusLine::new(
            "INPUT",
            RowState::Missing,
            detail(format_args!("DEVICE ABSENT OR UNSUPPORTED")),
        )
    } else if entropy::is_ready() {
        StatusLine::new(
            "INPUT",
            RowState::Waiting,
            detail(format_args!("PROBE PENDING")),
        )
    } else {
        StatusLine::new(
            "INPUT",
            RowState::Waiting,
            detail(format_args!("WAITING ENTROPY")),
        )
    }
}

fn append_entropy_sources(buffer: &mut TextBuf<128>, stats: entropy::EntropyStats) {
    let mut wrote = false;
    if stats.used_rdrand {
        buffer.push_str("RDRAND");
        wrote = true;
    }
    if stats.used_virtio {
        if wrote {
            buffer.push_str("+");
        }
        buffer.push_str("VIRTIO-RNG");
        wrote = true;
    }
    if !wrote {
        buffer.push_str("NONE");
    }
}

fn log_transition(previous: Option<RowState>, line: &StatusLine) {
    if previous == Some(line.state) {
        return;
    }

    serial::write_fmt(format_args!(
        "status {}: {} - {}\r\n",
        line.label,
        line.state.as_str(),
        line.detail.as_str()
    ));
    console::record_event(format_args!(
        "STATUS {} {}",
        line.label,
        line.state.as_str()
    ));
}

fn draw(surface: &mut FramebufferSurface, uptime_ms: u64, snapshot: &Snapshot) {
    let width = surface.info().width as usize;
    let height = surface.info().height as usize;

    surface.fill(Color::new(14, 18, 22));
    surface.fill_rect(0, 0, width, 84, Color::new(25, 36, 47));
    surface.fill_rect(0, 84, width, 2, Color::new(66, 92, 112));

    text::draw_text(
        surface,
        24,
        22,
        "SEEDOS STAGE-0",
        Color::new(245, 248, 250),
        None,
    );
    text::draw_text(
        surface,
        24,
        48,
        "AGENT HOST: LIVE STATUS",
        Color::new(178, 205, 224),
        None,
    );

    let uptime = detail64(format_args!("UPTIME {} MS", uptime_ms));
    text::draw_text(
        surface,
        24,
        height.saturating_sub(28),
        uptime.as_str(),
        Color::new(130, 151, 165),
        None,
    );

    let mut y = 116usize;
    draw_row(surface, y, &snapshot.framebuffer);
    y += 46;
    draw_row(surface, y, &snapshot.entropy);
    y += 46;
    draw_row(surface, y, &snapshot.virtio_rng);
    y += 46;
    draw_row(surface, y, &snapshot.virtio_net);
    y += 46;
    draw_row(surface, y, &snapshot.input);

    draw_console(surface, 380);
}

fn draw_row(surface: &mut FramebufferSurface, y: usize, line: &StatusLine) {
    let width = surface.info().width as usize;
    let row_width = width.saturating_sub(48);

    surface.fill_rect(
        24,
        y.saturating_sub(10),
        row_width,
        36,
        Color::new(23, 29, 36),
    );
    surface.fill_rect(196, y.saturating_sub(6), 116, 24, line.state.badge_color());

    text::draw_text(surface, 40, y, line.label, Color::new(225, 232, 238), None);
    text::draw_text(
        surface,
        208,
        y,
        line.state.as_str(),
        Color::new(250, 252, 252),
        None,
    );
    text::draw_text(
        surface,
        336,
        y,
        line.detail.as_str(),
        Color::new(188, 202, 212),
        None,
    );
}

fn draw_console(surface: &mut FramebufferSurface, y: usize) {
    let width = surface.info().width as usize;
    let panel_width = width.saturating_sub(48);
    let snapshot = console::snapshot();

    surface.fill_rect(24, y, panel_width, 188, Color::new(18, 23, 28));
    surface.fill_rect(24, y, panel_width, 2, Color::new(66, 92, 112));

    text::draw_text(
        surface,
        40,
        y + 18,
        "SERIAL CONSOLE",
        Color::new(178, 205, 224),
        None,
    );

    let mut line_y = y + 44;
    let mut idx = 0usize;
    while idx < snapshot.lines.len() {
        let line = snapshot.lines[idx].as_str();
        if !line.is_empty() {
            text::draw_text(surface, 40, line_y, line, Color::new(188, 202, 212), None);
        }
        line_y += 16;
        idx += 1;
    }

    text::draw_text(
        surface,
        40,
        y + 172,
        snapshot.input.as_str(),
        Color::new(245, 248, 250),
        None,
    );
}

fn detail(args: fmt::Arguments<'_>) -> TextBuf<128> {
    let mut buffer = TextBuf::new();
    let _ = buffer.write_fmt(args);
    buffer
}

fn detail64(args: fmt::Arguments<'_>) -> TextBuf<64> {
    let mut buffer = TextBuf::new();
    let _ = buffer.write_fmt(args);
    buffer
}

struct Mac([u8; 6]);

impl fmt::Display for Mac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

struct TextBuf<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> TextBuf<N> {
    const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    fn push_str(&mut self, value: &str) {
        let remaining = N.saturating_sub(self.len);
        let bytes = value.as_bytes();
        let take = usize::min(remaining, bytes.len());
        self.bytes[self.len..self.len + take].copy_from_slice(&bytes[..take]);
        self.len += take;
    }
}

impl<const N: usize> Write for TextBuf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}
