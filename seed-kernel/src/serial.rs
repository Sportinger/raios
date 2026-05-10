use core::fmt::{self, Write};
use spin::Mutex;
use uart_16550::SerialPort;

static COM1: Mutex<Option<SerialPort>> = Mutex::new(None);
const COM1_PORT: u16 = 0x3F8;
const COM1_LINE_STATUS: u16 = COM1_PORT + 5;
const LINE_STATUS_DATA_READY: u8 = 1 << 0;

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

pub fn write_byte(byte: u8) {
    if let Some(port) = COM1.lock().as_mut() {
        port.send(byte);
    }
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
        }
        Ok(())
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, preserves_flags));
    value
}
