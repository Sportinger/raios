use core::fmt::{self, Write};
use core::str;

use spin::Mutex;

use crate::{entropy, input, net, serial, ui, virtio};

const COMMAND_WIDTH: usize = 80;
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

    fn trimmed_command(&self) -> CommandText {
        let mut start = 0usize;
        let mut end = self.len;
        while start < end && self.bytes[start].is_ascii_whitespace() {
            start += 1;
        }
        while end > start && self.bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }

        let mut text = CommandText::new();
        for &byte in &self.bytes[start..end] {
            text.push_byte(byte.to_ascii_lowercase());
        }
        text
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

struct ConsoleState {
    input: CommandLine,
    lines: [ConsoleLine; OUTPUT_LINES],
    next_line: usize,
    line_count: usize,
}

impl ConsoleState {
    const fn new() -> Self {
        Self {
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
        let _ = write!(input, "> {}", self.input.as_str());

        ConsoleSnapshot { lines, input }
    }

    fn handle_byte(&mut self, byte: u8) -> ByteAction {
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
}

enum ByteAction {
    Noop,
    Echo(u8),
    Backspace,
    Bell,
    Execute(ConsoleLine),
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

    fn as_line(&self) -> ConsoleLine {
        let mut line = ConsoleLine::empty();
        let take = usize::min(self.len, OUTPUT_WIDTH);
        line.bytes[..take].copy_from_slice(&self.bytes[..take]);
        line.len = take;
        line
    }

    fn clear(&mut self) {
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

        changed |= process_byte(byte, runtime);
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
    }
}

fn execute(command_line: ConsoleLine, runtime: ui::RuntimeStatus) {
    let command = command_line.trimmed_command();
    if command.as_str().is_empty() {
        return;
    }

    write_output(format_args!("> {}", command_line.as_str()));

    match command.as_str() {
        "help" => command_help(),
        "status" => command_status(runtime),
        "devices" => command_devices(runtime),
        "log" => command_log(),
        _ => write_output(format_args!("UNKNOWN COMMAND: {}", command_line.as_str())),
    }
}

fn command_help() {
    write_output(format_args!("COMMANDS: help status devices log"));
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
        "VIRTIO-RNG: {}    VIRTIO-NET: {}    INPUT: {}",
        rng_state(runtime),
        net_state(runtime),
        input_state(runtime)
    ));
}

fn command_devices(runtime: ui::RuntimeStatus) {
    write_output(format_args!("FRAMEBUFFER: READY"));
    write_output(format_args!("VIRTIO-RNG: {}", rng_state(runtime)));

    if let Some(info) = virtio::net::info() {
        write_output(format_args!("VIRTIO-NET: DEVICE MAC {}", Mac(info.mac)));
    } else {
        write_output(format_args!("VIRTIO-NET: {}", net_state(runtime)));
    }

    write_output(format_args!("INPUT: {}", input_state(runtime)));
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
