use core::fmt::{self, Write};
use core::str;

use crate::framebuffer::{Color, FramebufferInfo, FramebufferSurface};
use crate::{console, entropy, input, net, serial, text, usb};

#[derive(Clone, Copy)]
pub struct RuntimeStatus {
    pub net_probe_complete: bool,
    pub input_probe_complete: bool,
}

impl RuntimeStatus {
    pub const fn new() -> Self {
        Self {
            net_probe_complete: false,
            input_probe_complete: false,
        }
    }
}

pub struct StatusUi {
    surface: Option<FramebufferSurface>,
    last_states: Option<SnapshotStates>,
    last_draw_states: Option<SnapshotStates>,
}

impl StatusUi {
    pub fn new(surface: Option<FramebufferSurface>) -> Self {
        if let Some(surface) = surface.as_ref() {
            let info = surface.info();
            input::set_pointer_bounds(info.width as usize, info.height as usize);
        }
        Self {
            surface,
            last_states: None,
            last_draw_states: None,
        }
    }

    pub fn render(&mut self, uptime_ms: u64, runtime: RuntimeStatus) {
        self.render_inner(uptime_ms, runtime, false);
    }

    pub fn render_forced(&mut self, uptime_ms: u64, runtime: RuntimeStatus) {
        self.render_inner(uptime_ms, runtime, true);
    }

    fn render_inner(&mut self, uptime_ms: u64, runtime: RuntimeStatus, force_draw: bool) {
        let framebuffer = self.surface.as_ref().map(|surface| surface.info());
        let snapshot = Snapshot::collect(framebuffer, runtime);
        self.log_transitions(&snapshot);

        let states = snapshot.states();
        let should_draw = force_draw || self.last_draw_states != Some(states);

        if should_draw {
            if let Some(surface) = self.surface.as_mut() {
                draw(surface, uptime_ms, &snapshot);
                surface.present();
                self.last_draw_states = Some(states);
            }
        }
    }

    fn log_transitions(&mut self, snapshot: &Snapshot) {
        let states = snapshot.states();
        let previous = self.last_states;

        log_transition(previous.map(|prev| prev.framebuffer), &snapshot.framebuffer);
        log_transition(previous.map(|prev| prev.entropy), &snapshot.entropy);
        log_transition(previous.map(|prev| prev.usb_xhci), &snapshot.usb_xhci);
        log_transition(previous.map(|prev| prev.network), &snapshot.network);
        log_transition(previous.map(|prev| prev.input), &snapshot.input);

        self.last_states = Some(states);
    }
}

struct Snapshot {
    framebuffer: StatusLine,
    entropy: StatusLine,
    usb_xhci: StatusLine,
    network: StatusLine,
    input: StatusLine,
    mouse: input::MouseSnapshot,
}

impl Snapshot {
    fn collect(framebuffer: Option<FramebufferInfo>, runtime: RuntimeStatus) -> Self {
        let framebuffer = framebuffer_line(framebuffer);
        let entropy = entropy_line();
        let usb_xhci = usb_xhci_line();
        let network = network_line(runtime);
        let input = input_line(runtime);
        let mouse = input::mouse_snapshot();
        Self {
            framebuffer,
            entropy,
            usb_xhci,
            network,
            input,
            mouse,
        }
    }

    fn states(&self) -> SnapshotStates {
        SnapshotStates {
            framebuffer: self.framebuffer.state,
            entropy: self.entropy.state,
            usb_xhci: self.usb_xhci.state,
            network: self.network.state,
            input: self.input.state,
            mouse_sequence: if self.mouse.seen {
                self.mouse.sequence
            } else {
                0
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct SnapshotStates {
    framebuffer: RowState,
    entropy: RowState,
    usb_xhci: RowState,
    network: RowState,
    input: RowState,
    mouse_sequence: u64,
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

fn usb_xhci_line() -> StatusLine {
    let snapshot = usb::snapshot();
    match snapshot.state {
        usb::UsbStatus::NotProbed => StatusLine::new(
            "USB-XHCI",
            RowState::Waiting,
            detail(format_args!("PROBE PENDING")),
        ),
        usb::UsbStatus::Missing => StatusLine::new(
            "USB-XHCI",
            RowState::Missing,
            detail(format_args!("CONTROLLER ABSENT")),
        ),
        usb::UsbStatus::Error => StatusLine::new(
            "USB-XHCI",
            RowState::Degraded,
            detail(format_args!(
                "{}",
                snapshot.last_error.unwrap_or("PROBE ERROR")
            )),
        ),
        usb::UsbStatus::Ready => {
            let address = match snapshot.address {
                Some(address) => detail(format_args!("{}", address)),
                None => detail(format_args!("UNKNOWN")),
            };
            let keyboard = usb_keyboard_status(snapshot.keyboard_status);
            let mouse = usb_mouse_status(snapshot.mouse_status);
            StatusLine::new(
                "USB-XHCI",
                RowState::Ready,
                detail(format_args!(
                    "{} HCI {:04X} PORTS {} CONNECTED {} KBD {} MOUSE {}",
                    address.as_str(),
                    snapshot.hci_version,
                    snapshot.max_ports,
                    snapshot.connected_ports,
                    keyboard,
                    mouse
                )),
            )
        }
    }
}

fn usb_keyboard_status(status: usb::UsbKeyboardStatus) -> &'static str {
    match status {
        usb::UsbKeyboardStatus::NotProbed => "PENDING",
        usb::UsbKeyboardStatus::Ready => "READY",
        usb::UsbKeyboardStatus::NotFound => "NONE",
        usb::UsbKeyboardStatus::Error => "ERROR",
    }
}

fn usb_mouse_status(status: usb::UsbMouseStatus) -> &'static str {
    match status {
        usb::UsbMouseStatus::NotProbed => "PENDING",
        usb::UsbMouseStatus::Ready => "READY",
        usb::UsbMouseStatus::NotFound => "NONE",
        usb::UsbMouseStatus::Error => "ERROR",
    }
}

fn network_line(runtime: RuntimeStatus) -> StatusLine {
    if let Some(config) = net::ui_snapshot() {
        if let Some(ip) = config.ip {
            let gateway = match config.gateway {
                Some(gateway) => detail(format_args!("{}", gateway)),
                None => detail(format_args!("NONE")),
            };
            return StatusLine::new(
                "NETWORK",
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
            "NETWORK",
            RowState::Waiting,
            if net::dhcp_poll_enabled() {
                detail(format_args!("MAC {} AWAITING DHCP", Mac(config.mac)))
            } else {
                detail(format_args!("MAC {} DHCP DEFERRED", Mac(config.mac)))
            },
        );
    }

    if runtime.net_probe_complete {
        StatusLine::new(
            "NETWORK",
            RowState::Missing,
            detail(format_args!("E1000 DEVICE ABSENT OR UNSUPPORTED")),
        )
    } else if entropy::is_ready() {
        StatusLine::new(
            "NETWORK",
            RowState::Waiting,
            detail(format_args!("PROBE PENDING")),
        )
    } else {
        StatusLine::new(
            "NETWORK",
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
            detail(format_args!("{}", input::device_detail())),
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
    draw_row(surface, y, &snapshot.usb_xhci);
    y += 46;
    draw_row(surface, y, &snapshot.network);
    y += 46;
    draw_row(surface, y, &snapshot.input);

    draw_console(surface, 380);
    draw_mouse_cursor(surface, snapshot.mouse);
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

fn draw_mouse_cursor(surface: &mut FramebufferSurface, mouse: input::MouseSnapshot) {
    if !mouse.seen {
        return;
    }

    let fill = if mouse.buttons & 1 != 0 {
        Color::new(92, 204, 255)
    } else {
        Color::new(245, 248, 250)
    };
    let outline = Color::new(4, 8, 12);
    let x = mouse.x;
    let y = mouse.y;
    let shape = [
        "X",
        "XX",
        "XOX",
        "XOOX",
        "XOOOX",
        "XOOOOX",
        "XOOOOOX",
        "XOOOOOOX",
        "XOOOOOOOX",
        "XOOOOX",
        "XOOXOX",
        "XOXXOX",
        "XX  XOX",
        "X    XOX",
        "     XOX",
        "      X",
    ];

    for (row, pattern) in shape.iter().enumerate() {
        for (col, byte) in pattern.as_bytes().iter().copied().enumerate() {
            match byte {
                b'X' => surface.set_pixel(x + col, y + row, outline),
                b'O' => surface.set_pixel(x + col, y + row, fill),
                _ => {}
            }
        }
    }
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
