use core::fmt::{self, Write};
use spin::Mutex;
use uart_16550::SerialPort;

static COM1: Mutex<Option<SerialPort>> = Mutex::new(None);
const COM1_PORT: u16 = 0x3F8;

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
