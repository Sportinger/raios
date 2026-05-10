use core::fmt::{self, Write};
use core::str;

use spin::Mutex;

use crate::{bridge, entropy, input, net, provider, serial, ui, usb, virtio};

const COMMAND_WIDTH: usize = 256;
const OUTPUT_WIDTH: usize = 104;
const OUTPUT_LINES: usize = 8;
const MAX_BYTES_PER_POLL: usize = 64;

static CONSOLE: Mutex<ConsoleState> = Mutex::new(ConsoleState::new());

#[derive(Clone, Copy)]
pub struct ConsoleLine {
    bytes: [u8; OUTPUT_WIDTH],
    len: usize,
}

impl ConsoleLine {
    pub const fn empty() -> Self {
        Self {
            bytes: [0; OUTPUT_WIDTH],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    fn trimmed_bounds(&self) -> (usize, usize) {
        let mut start = 0usize;
        let mut end = self.len;
        while start < end && self.bytes[start].is_ascii_whitespace() {
            start += 1;
        }
        while end > start && self.bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }

        (start, end)
    }

    fn trimmed_str(&self) -> &str {
        let (start, end) = self.trimmed_bounds();
        unsafe { str::from_utf8_unchecked(&self.bytes[start..end]) }
    }

    fn command_word(&self) -> CommandText {
        let (start, end) = self.trimmed_bounds();
        let mut text = CommandText::new();
        let mut idx = start;
        while idx < end && !self.bytes[idx].is_ascii_whitespace() {
            let byte = self.bytes[idx];
            text.push_byte(byte.to_ascii_lowercase());
            idx += 1;
        }
        text
    }

    fn arguments_after_command(&self) -> &str {
        let (mut idx, end) = self.trimmed_bounds();
        while idx < end && !self.bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        while idx < end && self.bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        unsafe { str::from_utf8_unchecked(&self.bytes[idx..end]) }
    }
}

impl Write for ConsoleLine {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining = self.bytes.len().saturating_sub(self.len);
        let bytes = s.as_bytes();
        let take = usize::min(remaining, bytes.len());
        self.bytes[self.len..self.len + take].copy_from_slice(&bytes[..take]);
        self.len += take;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct ConsoleSnapshot {
    pub lines: [ConsoleLine; OUTPUT_LINES],
    pub input: ConsoleLine,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConsoleMode {
    Command,
    SetupMenu,
    ProviderMenu,
    ApiKeyEntry,
}

struct ConsoleState {
    mode: ConsoleMode,
    input: CommandLine,
    lines: [ConsoleLine; OUTPUT_LINES],
    next_line: usize,
    line_count: usize,
}

impl ConsoleState {
    const fn new() -> Self {
        Self {
            mode: ConsoleMode::Command,
            input: CommandLine::new(),
            lines: [ConsoleLine::empty(); OUTPUT_LINES],
            next_line: 0,
            line_count: 0,
        }
    }

    fn push_line(&mut self, line: ConsoleLine) {
        self.lines[self.next_line] = line;
        self.next_line = (self.next_line + 1) % OUTPUT_LINES;
        self.line_count = usize::min(self.line_count + 1, OUTPUT_LINES);
    }

    fn snapshot(&self) -> ConsoleSnapshot {
        let mut lines = [ConsoleLine::empty(); OUTPUT_LINES];
        let oldest = if self.line_count == OUTPUT_LINES {
            self.next_line
        } else {
            0
        };
        let start = OUTPUT_LINES - self.line_count;
        let mut idx = 0usize;
        while idx < self.line_count {
            let source = (oldest + idx) % OUTPUT_LINES;
            lines[start + idx] = self.lines[source];
            idx += 1;
        }

        let mut input = ConsoleLine::empty();
        match self.mode {
            ConsoleMode::Command => {
                let _ = write!(input, "> {}", self.input.as_str());
            }
            ConsoleMode::SetupMenu => {
                let _ = write!(input, "setup> {}", self.input.as_str());
            }
            ConsoleMode::ProviderMenu => {
                let _ = write!(input, "provider> {}", self.input.as_str());
            }
            ConsoleMode::ApiKeyEntry => {
                let _ = write!(input, "api key> ");
                let mut idx = 0usize;
                while idx < self.input.len() {
                    let _ = input.write_str("*");
                    idx += 1;
                }
            }
        }

        ConsoleSnapshot { lines, input }
    }

    fn handle_byte(&mut self, byte: u8) -> ByteAction {
        match self.mode {
            ConsoleMode::Command => self.handle_command_byte(byte),
            ConsoleMode::SetupMenu => self.handle_setup_menu_byte(byte),
            ConsoleMode::ProviderMenu => self.handle_provider_menu_byte(byte),
            ConsoleMode::ApiKeyEntry => self.handle_api_key_byte(byte),
        }
    }

    fn enter_setup_menu(&mut self) {
        self.mode = ConsoleMode::SetupMenu;
        self.input.clear();
    }

    fn handle_command_byte(&mut self, byte: u8) -> ByteAction {
        match byte {
            b'\r' | b'\n' => {
                if self.input.is_empty() {
                    ByteAction::Noop
                } else {
                    let command = self.input.as_line();
                    self.input.clear();
                    ByteAction::Execute(command)
                }
            }
            0x08 | 0x7f => {
                if self.input.pop_byte() {
                    ByteAction::Backspace
                } else {
                    ByteAction::Noop
                }
            }
            b if b.is_ascii_graphic() || b == b' ' => {
                if self.input.push_byte(b) {
                    ByteAction::Echo(b)
                } else {
                    ByteAction::Bell
                }
            }
            _ => ByteAction::Noop,
        }
    }

    fn handle_setup_menu_byte(&mut self, byte: u8) -> ByteAction {
        match byte.to_ascii_lowercase() {
            b'1' => {
                self.mode = ConsoleMode::ProviderMenu;
                self.input.clear();
                ByteAction::ShowProviderMenu
            }
            b'2' => {
                self.mode = ConsoleMode::ApiKeyEntry;
                self.input.clear();
                ByteAction::ShowApiKeyEntry
            }
            b'3' => {
                bridge::clear_api_key();
                ByteAction::ShowSetupMessage(SetupMessage::ApiKeyCleared)
            }
            b'4' => ByteAction::ShowProviderStatus,
            b'q' | 0x1b => {
                self.mode = ConsoleMode::Command;
                self.input.clear();
                ByteAction::SetupClosed
            }
            b'\r' | b'\n' => ByteAction::Noop,
            _ => ByteAction::Bell,
        }
    }

    fn handle_provider_menu_byte(&mut self, byte: u8) -> ByteAction {
        match byte.to_ascii_lowercase() {
            b'1' => {
                bridge::set_provider(bridge::Provider::Echo);
                self.mode = ConsoleMode::SetupMenu;
                self.input.clear();
                ByteAction::ShowSetupMessage(SetupMessage::ProviderSet(bridge::Provider::Echo))
            }
            b'2' => {
                bridge::set_provider(bridge::Provider::OpenAi);
                self.mode = ConsoleMode::SetupMenu;
                self.input.clear();
                ByteAction::ShowSetupMessage(SetupMessage::ProviderSet(bridge::Provider::OpenAi))
            }
            b'q' | 0x1b => {
                self.mode = ConsoleMode::SetupMenu;
                self.input.clear();
                ByteAction::ShowSetupMenu
            }
            b'\r' | b'\n' => ByteAction::Noop,
            _ => ByteAction::Bell,
        }
    }

    fn handle_api_key_byte(&mut self, byte: u8) -> ByteAction {
        match byte {
            b'\r' | b'\n' => {
                let result = bridge::set_api_key(self.input.as_bytes());
                self.input.clear();
                self.mode = ConsoleMode::SetupMenu;
                match result {
                    Ok(()) => ByteAction::ShowSetupMessage(SetupMessage::ApiKeySet),
                    Err(bridge::ApiKeyError::Empty) => {
                        ByteAction::ShowSetupMessage(SetupMessage::ApiKeyEmpty)
                    }
                    Err(bridge::ApiKeyError::TooLong) => {
                        ByteAction::ShowSetupMessage(SetupMessage::ApiKeyTooLong)
                    }
                    Err(bridge::ApiKeyError::InvalidByte) => {
                        ByteAction::ShowSetupMessage(SetupMessage::ApiKeyInvalid)
                    }
                }
            }
            0x1b => {
                self.input.clear();
                self.mode = ConsoleMode::SetupMenu;
                ByteAction::ShowSetupMessage(SetupMessage::ApiKeyCancelled)
            }
            0x08 | 0x7f => {
                if self.input.pop_byte() {
                    ByteAction::Redraw
                } else {
                    ByteAction::Noop
                }
            }
            b if b.is_ascii_graphic() => {
                if self.input.push_byte(b) {
                    ByteAction::Redraw
                } else {
                    ByteAction::Bell
                }
            }
            _ => ByteAction::Noop,
        }
    }
}

enum ByteAction {
    Noop,
    Echo(u8),
    Backspace,
    Bell,
    Execute(ConsoleLine),
    Redraw,
    ShowSetupMenu,
    ShowProviderMenu,
    ShowApiKeyEntry,
    ShowProviderStatus,
    ShowSetupMessage(SetupMessage),
    SetupClosed,
}

enum SetupMessage {
    ProviderSet(bridge::Provider),
    ApiKeySet,
    ApiKeyCleared,
    ApiKeyEmpty,
    ApiKeyTooLong,
    ApiKeyInvalid,
    ApiKeyCancelled,
}

struct CommandLine {
    bytes: [u8; COMMAND_WIDTH],
    len: usize,
}

impl CommandLine {
    const fn new() -> Self {
        Self {
            bytes: [0; COMMAND_WIDTH],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    fn len(&self) -> usize {
        self.len
    }

    fn as_line(&self) -> ConsoleLine {
        let mut line = ConsoleLine::empty();
        let take = usize::min(self.len, OUTPUT_WIDTH);
        line.bytes[..take].copy_from_slice(&self.bytes[..take]);
        line.len = take;
        line
    }

    fn clear(&mut self) {
        self.bytes[..self.len].fill(0);
        self.len = 0;
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn push_byte(&mut self, byte: u8) -> bool {
        if self.len == self.bytes.len() {
            return false;
        }
        self.bytes[self.len] = byte;
        self.len += 1;
        true
    }

    fn pop_byte(&mut self) -> bool {
        if self.len == 0 {
            return false;
        }
        self.len -= 1;
        self.bytes[self.len] = 0;
        true
    }
}

struct CommandText {
    bytes: [u8; COMMAND_WIDTH],
    len: usize,
}

impl CommandText {
    const fn new() -> Self {
        Self {
            bytes: [0; COMMAND_WIDTH],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    fn push_byte(&mut self, byte: u8) {
        if self.len == self.bytes.len() {
            return;
        }
        self.bytes[self.len] = byte;
        self.len += 1;
    }
}

pub fn init() {
    write_output(format_args!("SERIAL CONSOLE READY"));
}

pub fn poll(runtime: ui::RuntimeStatus) -> bool {
    let mut changed = false;
    let mut processed = 0usize;

    while processed < MAX_BYTES_PER_POLL {
        let Some(byte) = serial::try_read_byte() else {
            break;
        };

        changed |= process_serial_byte(byte, runtime);
        processed += 1;
    }

    input::drain_console_bytes(|byte| {
        changed |= process_byte(byte, runtime);
    });

    changed
}

pub fn snapshot() -> ConsoleSnapshot {
    CONSOLE.lock().snapshot()
}

pub fn record_event(args: fmt::Arguments<'_>) {
    let mut line = ConsoleLine::empty();
    let _ = line.write_fmt(args);
    CONSOLE.lock().push_line(line);
}

pub fn write_event(args: fmt::Arguments<'_>) {
    write_output(args);
}

fn process_byte(byte: u8, runtime: ui::RuntimeStatus) -> bool {
    let action = {
        let mut state = CONSOLE.lock();
        state.handle_byte(byte)
    };

    match action {
        ByteAction::Noop => false,
        ByteAction::Echo(byte) => {
            serial::write_byte(byte);
            true
        }
        ByteAction::Backspace => {
            serial::write_fmt(format_args!("\x08 \x08"));
            true
        }
        ByteAction::Bell => {
            serial::write_byte(0x07);
            false
        }
        ByteAction::Execute(command) => {
            serial::write_line("");
            execute(command, runtime);
            true
        }
        ByteAction::Redraw => true,
        ByteAction::ShowSetupMenu => {
            show_setup_menu();
            true
        }
        ByteAction::ShowProviderMenu => {
            show_provider_menu();
            true
        }
        ByteAction::ShowApiKeyEntry => {
            show_api_key_entry();
            true
        }
        ByteAction::ShowProviderStatus => {
            show_provider_status();
            true
        }
        ByteAction::ShowSetupMessage(message) => {
            show_setup_message(message);
            show_setup_menu();
            true
        }
        ByteAction::SetupClosed => {
            write_output(format_args!("SETUP CLOSED"));
            true
        }
    }
}

fn process_serial_byte(byte: u8, runtime: ui::RuntimeStatus) -> bool {
    match bridge::ingest_serial_byte(byte) {
        bridge::SerialIngest::Console(byte) => process_byte(byte, runtime),
        bridge::SerialIngest::Consumed => false,
        bridge::SerialIngest::Response => {
            command_bridge_response();
            true
        }
        bridge::SerialIngest::Error => {
            command_bridge_error();
            true
        }
    }
}

fn execute(command_line: ConsoleLine, runtime: ui::RuntimeStatus) {
    let command = command_line.command_word();
    if command.as_str().is_empty() {
        return;
    }

    write_output(format_args!("> {}", command_line.trimmed_str()));

    match command.as_str() {
        "help" => command_help(),
        "status" => command_status(runtime),
        "devices" => command_devices(runtime),
        "log" => command_log(),
        "bridge" => command_bridge_status(),
        "openai" => command_openai_status(),
        "setup" => command_setup_enter(),
        "ask" => command_ask(command_line.arguments_after_command()),
        _ => write_output(format_args!(
            "UNKNOWN COMMAND: {}",
            command_line.trimmed_str()
        )),
    }
}

fn command_help() {
    write_output(format_args!(
        "COMMANDS: help status devices log bridge openai setup ask <text>"
    ));
}

fn command_status(runtime: ui::RuntimeStatus) {
    let entropy_stats = entropy::stats();
    write_output(format_args!(
        "FRAMEBUFFER: SEE UI    ENTROPY: {} {}/{}",
        if entropy_stats.ready {
            "READY"
        } else {
            "WAITING"
        },
        entropy_stats.pool_fill,
        entropy::POOL_CAPACITY
    ));
    write_output(format_args!(
        "VIRTIO-RNG: {}    USB-XHCI: {}    VIRTIO-NET: {}    INPUT: {}",
        rng_state(runtime),
        usb_state(),
        net_state(runtime),
        input_state(runtime)
    ));
}

fn command_devices(runtime: ui::RuntimeStatus) {
    write_output(format_args!("FRAMEBUFFER: READY"));
    write_output(format_args!("VIRTIO-RNG: {}", rng_state(runtime)));
    write_usb_device_line();

    if let Some(info) = virtio::net::info() {
        write_output(format_args!("VIRTIO-NET: DEVICE MAC {}", Mac(info.mac)));
    } else {
        write_output(format_args!("VIRTIO-NET: {}", net_state(runtime)));
    }

    write_output(format_args!("INPUT: {}", input_state(runtime)));
}

fn write_usb_device_line() {
    let snapshot = usb::snapshot();
    match snapshot.state {
        usb::UsbStatus::NotProbed => write_output(format_args!("USB-XHCI: WAITING")),
        usb::UsbStatus::Missing => write_output(format_args!("USB-XHCI: MISSING")),
        usb::UsbStatus::Error => write_output(format_args!(
            "USB-XHCI: DEGRADED {}",
            snapshot.last_error.unwrap_or("PROBE ERROR")
        )),
        usb::UsbStatus::Ready => {
            let hid = usb_keyboard_status(snapshot.keyboard_status);
            if let Some(address) = snapshot.address {
                write_output(format_args!(
                    "USB-XHCI: READY {} HCI {:04X} PORTS {} CONNECTED {} HID {}",
                    address,
                    snapshot.hci_version,
                    snapshot.max_ports,
                    snapshot.connected_ports,
                    hid
                ));
            } else {
                write_output(format_args!(
                    "USB-XHCI: READY UNKNOWN HCI {:04X} PORTS {} CONNECTED {} HID {}",
                    snapshot.hci_version, snapshot.max_ports, snapshot.connected_ports, hid
                ));
            }

            if let Some(detail) = snapshot.keyboard_detail {
                if snapshot.keyboard_status != usb::UsbKeyboardStatus::Ready {
                    write_output(format_args!("USB-HID: {}", detail));
                }
            }
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

fn command_log() {
    let snapshot = snapshot();
    serial::write_line("RECENT LOG:");
    let mut idx = 0usize;
    while idx < OUTPUT_LINES {
        let line = snapshot.lines[idx];
        if !line.as_str().is_empty() {
            serial::write_line(line.as_str());
        }
        idx += 1;
    }
    record_event(format_args!("RECENT LOG WRITTEN TO SERIAL"));
}

fn command_setup_enter() {
    {
        let mut state = CONSOLE.lock();
        state.enter_setup_menu();
    }

    write_output(format_args!("SETUP"));
    show_setup_menu();
}

fn show_setup_menu() {
    let snapshot = bridge::snapshot();
    write_output(format_args!(
        "1 PROVIDER: {}    2 API KEY: {}",
        snapshot.provider.as_str(),
        api_key_status(snapshot.api_key_set)
    ));
    write_output(format_args!("3 CLEAR API KEY    4 STATUS    Q EXIT"));
}

fn show_provider_menu() {
    write_output(format_args!("PROVIDER"));
    write_output(format_args!("1 ECHO BRIDGE    2 OPENAI DIRECT    Q BACK"));
}

fn show_api_key_entry() {
    write_output(format_args!("API KEY ENTRY"));
    write_output(format_args!("TYPE KEY, ENTER TO SAVE, ESC TO CANCEL"));
}

fn show_provider_status() {
    let snapshot = bridge::snapshot();
    write_output(format_args!(
        "PROVIDER: {}    API KEY: {}",
        snapshot.provider.as_str(),
        api_key_status(snapshot.api_key_set)
    ));
    if snapshot.provider == bridge::Provider::OpenAi && !snapshot.api_key_set {
        write_output(format_args!("OPENAI REQUIRES API KEY"));
    } else if snapshot.provider == bridge::Provider::OpenAi {
        write_output(format_args!(
            "OPENAI DIRECT: {}",
            provider::snapshot().direct_endpoint
        ));
    }
}

fn show_setup_message(message: SetupMessage) {
    match message {
        SetupMessage::ProviderSet(provider) => {
            write_output(format_args!("PROVIDER SET: {}", provider.as_str()))
        }
        SetupMessage::ApiKeySet => write_output(format_args!("API KEY SET (RAM ONLY)")),
        SetupMessage::ApiKeyCleared => write_output(format_args!("API KEY CLEARED")),
        SetupMessage::ApiKeyEmpty => write_output(format_args!("API KEY NOT CHANGED: EMPTY")),
        SetupMessage::ApiKeyTooLong => write_output(format_args!("API KEY NOT CHANGED: TOO LONG")),
        SetupMessage::ApiKeyInvalid => {
            write_output(format_args!("API KEY NOT CHANGED: INVALID BYTE"))
        }
        SetupMessage::ApiKeyCancelled => write_output(format_args!("API KEY ENTRY CANCELLED")),
    }
}

fn command_bridge_status() {
    let snapshot = provider::snapshot();
    write_output(format_args!(
        "PROVIDER: {}    API KEY: {}",
        snapshot.provider_name,
        api_key_status(snapshot.api_key_set)
    ));

    if snapshot.route == provider::Route::OpenAiDirect {
        write_output(format_args!("HOST BRIDGE: BYPASSED FOR OPENAI ASK"));
        command_openai_status();
        return;
    }

    if let Some(id) = snapshot.bridge_pending_id {
        write_output(format_args!("BRIDGE: PENDING REQUEST {}", id));
    } else {
        write_output(format_args!("BRIDGE: READY"));
    }

    if let Some(id) = snapshot.bridge_last_request_id {
        write_output(format_args!(
            "LAST REQUEST {}: {}",
            id,
            snapshot.bridge_last_prompt.as_str()
        ));
    }
    if let Some(id) = snapshot.bridge_last_response_id {
        write_output(format_args!(
            "LAST RESPONSE {}: {}",
            id,
            snapshot.bridge_last_response.as_str()
        ));
    }
    if snapshot.bridge_error_count > 0 {
        write_output(format_args!(
            "BRIDGE ERRORS: {} LAST {}",
            snapshot.bridge_error_count,
            snapshot.bridge_last_error.as_str()
        ));
    }
}

fn command_openai_status() {
    let snapshot = provider::snapshot();
    write_output(format_args!(
        "OPENAI DIRECT: {}    MODEL: {}",
        snapshot.direct_phase, snapshot.direct_model
    ));
    write_output(format_args!("ENDPOINT: {}", snapshot.direct_endpoint));
    if let Some(id) = snapshot.direct_pending_id {
        write_output(format_args!("OPENAI REQUEST {} PENDING", id));
    }
    if let Some(id) = snapshot.direct_last_request_id {
        write_output(format_args!(
            "LAST OPENAI REQUEST {}: {}",
            id,
            snapshot.direct_last_prompt.as_str()
        ));
    }
    if !snapshot.direct_last_event.as_str().is_empty() {
        write_output(format_args!(
            "OPENAI EVENT: {}",
            snapshot.direct_last_event.as_str()
        ));
    }
    if !snapshot.direct_last_error.as_str().is_empty() {
        write_output(format_args!(
            "OPENAI ERROR: {}",
            snapshot.direct_last_error.as_str()
        ));
    }
    if let Some(tcp) = snapshot.tcp {
        write_output(format_args!(
            "TCP: {} SEND {} RECV {}",
            tcp.state,
            yes_no(tcp.may_send),
            yes_no(tcp.may_recv)
        ));
    }
}

fn api_key_status(set: bool) -> &'static str {
    if set {
        "SET"
    } else {
        "MISSING"
    }
}

fn command_ask(prompt: &str) {
    match provider::submit_text(prompt) {
        Ok(submitted) => match submitted.route {
            provider::Route::OpenAiDirect => write_output(format_args!(
                "OPENAI DIRECT REQUEST {} STARTED",
                submitted.id
            )),
            provider::Route::HostBridge => {
                write_output(format_args!("BRIDGE REQUEST {} SENT", submitted.id))
            }
        },
        Err(provider::SubmitError::Empty) => write_output(format_args!("ASK REQUIRES TEXT")),
        Err(provider::SubmitError::MissingApiKey) => {
            write_output(format_args!("OPENAI REQUIRES API KEY"))
        }
        Err(provider::SubmitError::Busy { route, id }) => write_output(format_args!(
            "{} BUSY: REQUEST {} PENDING",
            route.as_str(),
            id
        )),
    }
}

fn command_bridge_response() {
    let snapshot = bridge::snapshot();
    if let Some(id) = snapshot.last_response_id {
        write_output(format_args!(
            "BRIDGE RESPONSE {}: {}",
            id,
            snapshot.last_response.as_str()
        ));
    }
}

fn command_bridge_error() {
    let snapshot = bridge::snapshot();
    write_output(format_args!(
        "BRIDGE ERROR {}: {}",
        snapshot.error_count,
        snapshot.last_error.as_str()
    ));
}

fn rng_state(_runtime: ui::RuntimeStatus) -> &'static str {
    let stats = entropy::stats();
    if stats.used_virtio {
        "READY"
    } else if entropy::virtio_source_attached() {
        "DEGRADED"
    } else {
        "WAITING"
    }
}

fn usb_state() -> &'static str {
    match usb::snapshot().state {
        usb::UsbStatus::NotProbed => "WAITING",
        usb::UsbStatus::Missing => "MISSING",
        usb::UsbStatus::Ready => "READY",
        usb::UsbStatus::Error => "DEGRADED",
    }
}

fn net_state(runtime: ui::RuntimeStatus) -> &'static str {
    if let Some(config) = net::ui_snapshot() {
        if config.ip.is_some() {
            "CONFIGURED"
        } else if net::dhcp_poll_enabled() {
            "DHCP"
        } else {
            "DEVICE"
        }
    } else if virtio::net::info().is_some() {
        "DEVICE"
    } else if runtime.virtio_net_probe_complete {
        "MISSING"
    } else {
        "WAITING"
    }
}

fn input_state(runtime: ui::RuntimeStatus) -> &'static str {
    if input::device_present() {
        "READY"
    } else if runtime.input_probe_complete {
        "MISSING"
    } else {
        "WAITING"
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "YES"
    } else {
        "NO"
    }
}

fn write_output(args: fmt::Arguments<'_>) {
    let mut line = ConsoleLine::empty();
    let _ = line.write_fmt(args);
    serial::write_line(line.as_str());
    CONSOLE.lock().push_line(line);
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
