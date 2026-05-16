use core::fmt::{self, Write};
use core::str;
use spin::Mutex;
use uart_16550::SerialPort;

static COM1: Mutex<Option<SerialPort>> = Mutex::new(None);
static LOG: Mutex<SerialLog> = Mutex::new(SerialLog::new());
const COM1_PORT: u16 = 0x3F8;
const COM1_LINE_STATUS: u16 = COM1_PORT + 5;
const LINE_STATUS_DATA_READY: u8 = 1 << 0;
pub const LOG_LINES: usize = 32;
pub const LOG_LINE_WIDTH: usize = 160;

#[derive(Clone, Copy)]
pub struct LogLine {
    bytes: [u8; LOG_LINE_WIDTH],
    len: usize,
}

impl LogLine {
    pub const fn empty() -> Self {
        Self {
            bytes: [0; LOG_LINE_WIDTH],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    fn clear(&mut self) {
        self.bytes[..self.len].fill(0);
        self.len = 0;
    }

    fn push_byte(&mut self, byte: u8) {
        if self.len == self.bytes.len() {
            return;
        }
        self.bytes[self.len] = byte;
        self.len += 1;
    }
}

#[derive(Clone, Copy)]
pub struct LogSnapshot {
    pub lines: [LogLine; LOG_LINES],
}

struct SerialLog {
    lines: [LogLine; LOG_LINES],
    current: LogLine,
    next_line: usize,
    line_count: usize,
}

impl SerialLog {
    const fn new() -> Self {
        Self {
            lines: [LogLine::empty(); LOG_LINES],
            current: LogLine::empty(),
            next_line: 0,
            line_count: 0,
        }
    }

    fn record_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.commit_current(),
            b'\r' => {}
            0x20..=0x7e => self.current.push_byte(byte),
            b'\t' => self.current.push_byte(b' '),
            _ => {}
        }
    }

    fn commit_current(&mut self) {
        if self.current.len == 0 {
            return;
        }
        self.lines[self.next_line] = self.current;
        self.next_line = (self.next_line + 1) % LOG_LINES;
        self.line_count = usize::min(self.line_count + 1, LOG_LINES);
        self.current.clear();
    }

    fn snapshot(&self) -> LogSnapshot {
        let mut lines = [LogLine::empty(); LOG_LINES];
        let oldest = if self.line_count == LOG_LINES {
            self.next_line
        } else {
            0
        };
        let start = LOG_LINES - self.line_count;
        let mut idx = 0usize;
        while idx < self.line_count {
            let source = (oldest + idx) % LOG_LINES;
            lines[start + idx] = self.lines[source];
            idx += 1;
        }

        if self.current.len != 0 {
            let insert = usize::min(LOG_LINES - 1, start + self.line_count);
            lines[insert] = self.current;
        }

        LogSnapshot { lines }
    }
}

pub fn init() {
    let mut port = unsafe { SerialPort::new(COM1_PORT) };
    port.init();
    COM1.lock().replace(port);
}

pub fn write_fmt(args: fmt::Arguments<'_>) {
    if let Some(port) = COM1.lock().as_mut() {
        let _ = SerialWriter { port }.write_fmt(args);
    }
}

pub fn write_line(line: &str) {
    write_fmt(format_args!("{}\r\n", line));
}

pub fn write_raw_fmt(args: fmt::Arguments<'_>) {
    if let Some(port) = COM1.lock().as_mut() {
        let _ = SerialRawWriter { port }.write_fmt(args);
    }
}

pub fn write_raw_line(line: &str) {
    write_raw_fmt(format_args!("{}\r\n", line));
}

pub fn write_raw_str(value: &str) {
    if let Some(port) = COM1.lock().as_mut() {
        for byte in value.bytes() {
            port.send(byte);
        }
    }
}

pub fn write_byte(byte: u8) {
    if let Some(port) = COM1.lock().as_mut() {
        port.send(byte);
    }
}

pub fn log_snapshot() -> LogSnapshot {
    LOG.lock().snapshot()
}

pub fn try_read_byte() -> Option<u8> {
    let status = unsafe { inb(COM1_LINE_STATUS) };
    if status & LINE_STATUS_DATA_READY == 0 {
        return None;
    }
    Some(unsafe { inb(COM1_PORT) })
}

struct SerialWriter<'a> {
    port: &'a mut SerialPort,
}

impl<'a> Write for SerialWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.port.send(byte);
            LOG.lock().record_byte(byte);
        }
        Ok(())
    }
}

struct SerialRawWriter<'a> {
    port: &'a mut SerialPort,
}

impl<'a> Write for SerialRawWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.port.send(byte);
        }
        Ok(())
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, preserves_flags));
    value
}
