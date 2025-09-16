use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::serial;

const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_DIVISOR: u16 = 0xFFFF;
const PIT_FREQUENCY_HZ: u64 = 1_193_182;
const MAX_PIT_POLLS: usize = 1_000_000;

static TSC_PER_MS: AtomicU64 = AtomicU64::new(3_000_000);

#[inline]
pub fn rdtsc() -> u64 {
    let hi: u32;
    let lo: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("edx") hi,
            out("eax") lo,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((hi as u64) << 32) | lo as u64
}

pub fn calibrate_tsc() {
    let mut measured = TSC_PER_MS.load(Ordering::Relaxed);
    unsafe {
        // Program PIT channel 0 in mode 2 (rate generator) with max divisor.
        outb(PIT_COMMAND, 0x34);
        outb(PIT_CHANNEL0, (PIT_DIVISOR & 0xFF) as u8);
        outb(PIT_CHANNEL0, (PIT_DIVISOR >> 8) as u8);
    }

    let start_tsc = rdtsc();
    let mut last_count = PIT_DIVISOR as u16;

    let mut success = false;
    // Wait until at least ~5ms have elapsed according to the PIT.
    for _ in 0..MAX_PIT_POLLS {
        let count = match read_pit_counter() {
            Some(value) => value,
            None => {
                serial::write_line("Warning: PIT read failed during TSC calibration");
                break;
            }
        };
        if count > last_count {
            // Counter wrapped; stop waiting.
            break;
        }
        last_count = count;
        let elapsed_counts = (PIT_DIVISOR as u64).wrapping_sub(count as u64);
        if elapsed_counts >= PIT_FREQUENCY_HZ / 200 { // ~5 ms
            let delta_tsc = rdtsc().wrapping_sub(start_tsc);
            let elapsed_ms = (elapsed_counts * 1_000) / PIT_FREQUENCY_HZ;
            if elapsed_ms > 0 {
                measured = delta_tsc / elapsed_ms.max(1);
            }
            success = true;
            break;
        }
    }

    if !success {
        serial::write_line("Warning: PIT calibration timed out; using fallback TSC estimate");
    }

    TSC_PER_MS.store(measured.max(1), Ordering::Relaxed);
}

pub fn tsc_per_ms() -> u64 {
    TSC_PER_MS.load(Ordering::Relaxed)
}

fn read_pit_counter() -> Option<u16> {
    unsafe {
        outb(PIT_COMMAND, 0x00); // latch channel 0
        let lo = inb(PIT_CHANNEL0);
        let hi = inb(PIT_CHANNEL0);
        Some(((hi as u16) << 8) | lo as u16)
    }
}

unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, preserves_flags));
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, preserves_flags));
    value
}
