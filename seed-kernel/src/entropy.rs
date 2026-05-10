use core::arch::asm;
use core::arch::x86_64::__cpuid;
use core::cmp;
use spin::Mutex;

use crate::serial;

const POOL_SIZE: usize = 64;
const MIN_READY_BYTES: usize = 32;
pub const POOL_CAPACITY: usize = POOL_SIZE;

static ENTROPY_STATE: Mutex<EntropyState> = Mutex::new(EntropyState::new());

#[derive(Clone, Copy, Debug)]
pub struct EntropyStats {
    pub ready: bool,
    pub pool_fill: usize,
    pub total_collected: u64,
    pub used_rdrand: bool,
}

#[derive(Clone, Copy)]
enum EntropySource {
    Rdrand,
}

#[derive(Clone, Copy)]
struct SourceFlags {
    rdrand: bool,
}

impl SourceFlags {
    const fn new() -> Self {
        Self { rdrand: false }
    }

    fn mark(&mut self, src: EntropySource) {
        match src {
            EntropySource::Rdrand => self.rdrand = true,
        }
    }
}

struct EntropyState {
    ready: bool,
    pool: [u8; POOL_SIZE],
    head: usize,
    fill: usize,
    total_collected: u64,
    sources: SourceFlags,
    low_water_reported: bool,
}

impl EntropyState {
    const fn new() -> Self {
        Self {
            ready: false,
            pool: [0u8; POOL_SIZE],
            head: 0,
            fill: 0,
            total_collected: 0,
            sources: SourceFlags::new(),
            low_water_reported: false,
        }
    }

    fn reset(&mut self) {
        self.ready = false;
        self.head = 0;
        self.fill = 0;
        self.total_collected = 0;
        self.sources = SourceFlags::new();
        self.low_water_reported = false;
    }

    fn ingest(&mut self, data: &[u8], source: EntropySource) -> usize {
        let mut stored = 0usize;
        for &byte in data {
            if self.fill == POOL_SIZE {
                break;
            }
            let tail_index = (self.head + self.fill) % POOL_SIZE;
            self.pool[tail_index] = byte;
            self.fill += 1;
            stored += 1;
        }
        if stored > 0 {
            self.total_collected = self.total_collected.saturating_add(stored as u64);
            self.sources.mark(source);
        }
        if self.fill >= MIN_READY_BYTES {
            self.ready = true;
            self.low_water_reported = false;
        }
        stored
    }

    fn consume(&mut self, dest: &mut [u8]) -> ConsumeReport {
        if self.fill == 0 {
            return ConsumeReport {
                taken: 0,
                warn_low_now: false,
            };
        }
        let to_take = cmp::min(dest.len(), self.fill);
        let first_segment = cmp::min(to_take, POOL_SIZE.saturating_sub(self.head));
        dest[..first_segment].copy_from_slice(&self.pool[self.head..self.head + first_segment]);
        if to_take > first_segment {
            let second = to_take - first_segment;
            dest[first_segment..first_segment + second].copy_from_slice(&self.pool[..second]);
            self.head = second;
        } else {
            self.head = (self.head + to_take) % POOL_SIZE;
        }
        self.fill -= to_take;
        let mut warn_low_now = false;
        if self.fill >= MIN_READY_BYTES {
            self.low_water_reported = false;
        } else if self.ready {
            if !self.low_water_reported {
                warn_low_now = true;
                self.low_water_reported = true;
            }
        }
        ConsumeReport {
            taken: to_take,
            warn_low_now,
        }
    }

    fn stats(&self) -> EntropyStats {
        EntropyStats {
            ready: self.ready,
            pool_fill: self.fill,
            total_collected: self.total_collected,
            used_rdrand: self.sources.rdrand,
        }
    }
}

struct ConsumeReport {
    taken: usize,
    warn_low_now: bool,
}

pub fn init() {
    let has_rdrand = cpu_has_rdrand();
    let mut scratch = [0u8; POOL_SIZE];
    let mut collected = 0usize;

    if has_rdrand {
        for chunk in scratch.chunks_exact_mut(8) {
            match rdrand64() {
                Some(value) => {
                    chunk.copy_from_slice(&value.to_le_bytes());
                    collected += 8;
                }
                None => break,
            }
        }
    }

    let ready_after_rdrand = {
        let mut state = ENTROPY_STATE.lock();
        state.reset();
        if collected > 0 {
            let stored = state.ingest(&scratch[..collected], EntropySource::Rdrand);
            if stored < collected {
                serial::write_line("Entropy pool at capacity; some RDRAND output unused");
            }
        }
        state.ready
    };

    if ready_after_rdrand {
        serial::write_line("Entropy seeded via RDRAND");
    } else if has_rdrand {
        serial::write_line("Warning: RDRAND present but insufficient samples gathered");
    } else {
        serial::write_line("Warning: RDRAND unsupported; external entropy source required");
    }
}

pub fn maintain() {
    if let Some(report) = refresh_rdrand_pool() {
        if !report.before_ready && report.after_ready {
            serial::write_line("Entropy pool healthy after RDRAND refresh");
        }
    }
}

struct RefreshReport {
    before_ready: bool,
    after_ready: bool,
}

fn refresh_rdrand_pool() -> Option<RefreshReport> {
    {
        let state = ENTROPY_STATE.lock();
        if state.fill == POOL_SIZE {
            return None;
        }
    }

    if !cpu_has_rdrand() {
        return None;
    }

    let mut scratch = [0u8; POOL_SIZE];
    let mut produced = 0usize;
    for chunk in scratch.chunks_exact_mut(8) {
        let Some(value) = rdrand64() else {
            break;
        };
        chunk.copy_from_slice(&value.to_le_bytes());
        produced += 8;
    }
    if produced == 0 {
        return None;
    }

    let mut state = ENTROPY_STATE.lock();
    let before_ready = state.ready;
    let stored = state.ingest(&scratch[..produced], EntropySource::Rdrand);
    let after_ready = state.ready;
    drop(state);

    if stored == 0 {
        return None;
    }

    Some(RefreshReport {
        before_ready,
        after_ready,
    })
}

pub fn is_ready() -> bool {
    ENTROPY_STATE.lock().ready
}

pub fn take(buffer: &mut [u8]) {
    let mut offset = 0usize;
    while offset < buffer.len() {
        while !is_ready() {
            maintain();
            core::hint::spin_loop();
        }

        let copied = {
            let mut state = ENTROPY_STATE.lock();
            let report = state.consume(&mut buffer[offset..]);
            if report.warn_low_now {
                serial::write_line("Entropy pool low; requesting entropy refresh");
            }
            report.taken
        };

        if copied == 0 {
            maintain();
            core::hint::spin_loop();
            continue;
        }

        offset += copied;
    }
}

pub fn stats() -> EntropyStats {
    ENTROPY_STATE.lock().stats()
}

fn cpu_has_rdrand() -> bool {
    (cpuid_leaf1_ecx() & (1 << 30)) != 0
}

fn cpuid_leaf1_ecx() -> u32 {
    unsafe { __cpuid(1).ecx }
}

fn rdrand64() -> Option<u64> {
    let mut success: u8;
    let mut value: u64;
    unsafe {
        asm!(
            "rdrand {val}\nsetc {succ}",
            val = out(reg) value,
            succ = out(reg_byte) success,
            options(nomem, nostack)
        );
    }
    if success != 0 {
        Some(value)
    } else {
        None
    }
}
