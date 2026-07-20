use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::serial;

const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_DIVISOR: u16 = 0xFFFF;
const PIT_FREQUENCY_HZ: u64 = 1_193_182;
const MAX_PIT_POLLS: usize = 1_000_000;
const CMOS_INDEX: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;
const CMOS_NMI_DISABLE: u8 = 0x80;
const CMOS_REG_SECOND: u8 = 0x00;
const CMOS_REG_MINUTE: u8 = 0x02;
const CMOS_REG_HOUR: u8 = 0x04;
const CMOS_REG_DAY: u8 = 0x07;
const CMOS_REG_MONTH: u8 = 0x08;
const CMOS_REG_YEAR: u8 = 0x09;
const CMOS_REG_STATUS_A: u8 = 0x0A;
const CMOS_REG_STATUS_B: u8 = 0x0B;
const CMOS_REG_CENTURY: u8 = 0x32;
const CMOS_UPDATE_IN_PROGRESS: u8 = 0x80;
const CMOS_BINARY_MODE: u8 = 0x04;
const CMOS_24H_MODE: u8 = 0x02;
const CMOS_PM_FLAG: u8 = 0x80;
const MAX_CMOS_UIP_POLLS: usize = 100_000;
const MAX_CMOS_SNAPSHOT_RETRIES: usize = 8;

static TSC_PER_MS: AtomicU64 = AtomicU64::new(3_000_000);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CmosRtcWallClock {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub data_mode: &'static str,
    pub hour_mode: &'static str,
    pub century_source: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CmosRtcError {
    UpdateNeverSettled,
    ImplausibleField,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct CmosRawSnapshot {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u8,
    century: u8,
    status_b: u8,
}

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
        if elapsed_counts >= PIT_FREQUENCY_HZ / 200 {
            // ~5 ms
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

pub fn read_cmos_rtc_wall_clock() -> Result<CmosRtcWallClock, CmosRtcError> {
    for _ in 0..MAX_CMOS_SNAPSHOT_RETRIES {
        wait_cmos_update_clear()?;
        let first = read_cmos_raw_snapshot();
        wait_cmos_update_clear()?;
        let second = read_cmos_raw_snapshot();
        if first == second {
            return decode_cmos_snapshot(first);
        }
    }
    Err(CmosRtcError::UpdateNeverSettled)
}

fn read_pit_counter() -> Option<u16> {
    unsafe {
        outb(PIT_COMMAND, 0x00); // latch channel 0
        let lo = inb(PIT_CHANNEL0);
        let hi = inb(PIT_CHANNEL0);
        Some(((hi as u16) << 8) | lo as u16)
    }
}

fn wait_cmos_update_clear() -> Result<(), CmosRtcError> {
    for _ in 0..MAX_CMOS_UIP_POLLS {
        if read_cmos_register(CMOS_REG_STATUS_A) & CMOS_UPDATE_IN_PROGRESS == 0 {
            return Ok(());
        }
    }
    Err(CmosRtcError::UpdateNeverSettled)
}

fn read_cmos_raw_snapshot() -> CmosRawSnapshot {
    CmosRawSnapshot {
        second: read_cmos_register(CMOS_REG_SECOND),
        minute: read_cmos_register(CMOS_REG_MINUTE),
        hour: read_cmos_register(CMOS_REG_HOUR),
        day: read_cmos_register(CMOS_REG_DAY),
        month: read_cmos_register(CMOS_REG_MONTH),
        year: read_cmos_register(CMOS_REG_YEAR),
        century: read_cmos_register(CMOS_REG_CENTURY),
        status_b: read_cmos_register(CMOS_REG_STATUS_B),
    }
}

fn decode_cmos_snapshot(raw: CmosRawSnapshot) -> Result<CmosRtcWallClock, CmosRtcError> {
    let binary = raw.status_b & CMOS_BINARY_MODE != 0;
    let is_24h = raw.status_b & CMOS_24H_MODE != 0;
    let data_mode = if binary { "binary" } else { "bcd" };
    let hour_mode = if is_24h { "24h" } else { "12h" };

    let second = decode_cmos_value(raw.second, binary);
    let minute = decode_cmos_value(raw.minute, binary);
    let hour = decode_cmos_hour(raw.hour, binary, is_24h);
    let day = decode_cmos_value(raw.day, binary);
    let month = decode_cmos_value(raw.month, binary);
    let year_low = decode_cmos_value(raw.year, binary) as u16;
    let century_value = decode_cmos_value(raw.century, binary);
    let (century_base, century_source) = if (19..=21).contains(&century_value) {
        ((century_value as u16) * 100, "cmos_register_0x32")
    } else {
        (2000, "assumed_2000s")
    };
    let year = century_base + year_low;

    if year < 2020
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(CmosRtcError::ImplausibleField);
    }

    Ok(CmosRtcWallClock {
        year,
        month,
        day,
        hour,
        minute,
        second,
        data_mode,
        hour_mode,
        century_source,
    })
}

fn decode_cmos_hour(raw_hour: u8, binary: bool, is_24h: bool) -> u8 {
    if is_24h {
        return decode_cmos_value(raw_hour, binary);
    }

    let pm = raw_hour & CMOS_PM_FLAG != 0;
    let mut hour = decode_cmos_value(raw_hour & !CMOS_PM_FLAG, binary);
    if hour == 12 {
        if !pm {
            hour = 0;
        }
    } else if pm {
        hour = hour.saturating_add(12);
    }
    hour
}

fn decode_cmos_value(value: u8, binary: bool) -> u8 {
    if binary {
        value
    } else {
        ((value >> 4) * 10) + (value & 0x0f)
    }
}

fn read_cmos_register(register: u8) -> u8 {
    unsafe {
        outb(CMOS_INDEX, CMOS_NMI_DISABLE | register);
        inb(CMOS_DATA)
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
