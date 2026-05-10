use core::sync::atomic::{AtomicBool, Ordering};

use crate::serial;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
const STATUS_OUTPUT_FULL: u8 = 1 << 0;
const STATUS_INPUT_FULL: u8 = 1 << 1;
const KEYBOARD_ENABLE_SCANNING: u8 = 0xF4;
const KEYBOARD_ACK: u8 = 0xFA;
const KEYBOARD_RESEND: u8 = 0xFE;
const MAX_SCANCODES_PER_POLL: usize = 32;
const WAIT_ITERS: usize = 100_000;

static PRESENT: AtomicBool = AtomicBool::new(false);
static ACTIVE: AtomicBool = AtomicBool::new(false);
static EXTENDED: AtomicBool = AtomicBool::new(false);

pub fn init_keyboard_polling() {
    let status = unsafe { inb(STATUS_PORT) };
    if status == 0xFF {
        serial::write_line("ps/2 keyboard controller absent or unreadable");
        return;
    }

    PRESENT.store(true, Ordering::Release);
    drain_pending_bytes();
    if send_keyboard_command(KEYBOARD_ENABLE_SCANNING) {
        ACTIVE.store(true, Ordering::Release);
        serial::write_line("ps/2 keyboard polling enabled");
    } else {
        serial::write_line("ps/2 controller present; keyboard did not acknowledge scan enable");
    }
}

pub fn present() -> bool {
    PRESENT.load(Ordering::Acquire)
}

pub fn active() -> bool {
    ACTIVE.load(Ordering::Acquire)
}

pub fn poll<F: FnMut(u16, bool)>(mut f: F) -> usize {
    if !present() {
        return 0;
    }

    let mut handled = 0usize;
    while handled < MAX_SCANCODES_PER_POLL {
        let status = unsafe { inb(STATUS_PORT) };
        if status & STATUS_OUTPUT_FULL == 0 {
            break;
        }

        let scancode = unsafe { inb(DATA_PORT) };
        if scancode == 0 || scancode == 0xFF {
            handled += 1;
            continue;
        }
        if scancode == 0xE0 {
            EXTENDED.store(true, Ordering::Release);
            handled += 1;
            continue;
        }
        if scancode == 0xE1 {
            EXTENDED.store(true, Ordering::Release);
            handled += 1;
            continue;
        }

        let extended = EXTENDED.swap(false, Ordering::AcqRel);
        if extended {
            handled += 1;
            continue;
        }

        let pressed = scancode & 0x80 == 0;
        let code = (scancode & 0x7F) as u16;
        ACTIVE.store(true, Ordering::Release);
        f(code, pressed);
        handled += 1;
    }

    handled
}

fn drain_pending_bytes() {
    let mut count = 0usize;
    while count < MAX_SCANCODES_PER_POLL {
        let status = unsafe { inb(STATUS_PORT) };
        if status & STATUS_OUTPUT_FULL == 0 {
            break;
        }
        let _ = unsafe { inb(DATA_PORT) };
        count += 1;
    }

    let _ = wait_input_clear();
}

fn send_keyboard_command(command: u8) -> bool {
    let mut attempt = 0usize;
    while attempt < 2 {
        if !wait_input_clear() {
            return false;
        }

        unsafe {
            outb(DATA_PORT, command);
        }

        match read_keyboard_response() {
            Some(KEYBOARD_ACK) => return true,
            Some(KEYBOARD_RESEND) => {
                attempt += 1;
                continue;
            }
            _ => return false,
        }
    }

    false
}

fn read_keyboard_response() -> Option<u8> {
    let mut remaining = WAIT_ITERS;
    while remaining > 0 {
        let status = unsafe { inb(STATUS_PORT) };
        if status & STATUS_OUTPUT_FULL != 0 {
            return Some(unsafe { inb(DATA_PORT) });
        }
        core::hint::spin_loop();
        remaining -= 1;
    }

    None
}

fn wait_input_clear() -> bool {
    let mut remaining = WAIT_ITERS;
    while remaining > 0 {
        if unsafe { inb(STATUS_PORT) } & STATUS_INPUT_FULL == 0 {
            return true;
        }
        core::hint::spin_loop();
        remaining -= 1;
    }

    false
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, preserves_flags));
    value
}

unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, preserves_flags));
}
