use core::sync::atomic::{AtomicBool, Ordering};

use crate::serial;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
const STATUS_OUTPUT_FULL: u8 = 1 << 0;
const STATUS_INPUT_FULL: u8 = 1 << 1;
const MAX_SCANCODES_PER_POLL: usize = 32;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static EXTENDED: AtomicBool = AtomicBool::new(false);

pub fn init_keyboard_polling() {
    let status = unsafe { inb(STATUS_PORT) };
    if status == 0xFF {
        serial::write_line("ps/2 keyboard controller absent or unreadable");
        return;
    }

    drain_pending_bytes();
    ACTIVE.store(true, Ordering::Release);
    serial::write_line("ps/2 keyboard polling enabled");
}

pub fn active() -> bool {
    ACTIVE.load(Ordering::Acquire)
}

pub fn poll<F: FnMut(u16, bool)>(mut f: F) -> usize {
    if !active() {
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

    while unsafe { inb(STATUS_PORT) } & STATUS_INPUT_FULL != 0 {
        core::hint::spin_loop();
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, preserves_flags));
    value
}
