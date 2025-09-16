use core::arch::asm;
use core::arch::x86_64::__cpuid;
use core::cmp;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use crate::serial;
use crate::time;
use crate::virtio::rng::VirtioRng;

const POOL_SIZE: usize = 64;
const MIN_READY_BYTES: usize = 32;
const REFRESH_INTERVAL_TSC: u64 = 25_000_000; // ~8 ms at 3 GHz

static ENTROPY_STATE: Mutex<EntropyState> = Mutex::new(EntropyState::new());

static VIRTIO_SOURCE: Mutex<Option<VirtioRng>> = Mutex::new(None);
static LAST_REFRESH_ATTEMPT_TSC: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug)]
pub struct EntropyStats {
    pub ready: bool,
    pub pool_fill: usize,
    pub total_collected: u64,
    pub used_rdrand: bool,
    pub used_virtio: bool,
}

#[derive(Clone, Copy)]
enum EntropySource {
    Rdrand,
    Virtio,
}

#[derive(Clone, Copy)]
struct SourceFlags {
    rdrand: bool,
    virtio: bool,
}

impl SourceFlags {
    const fn new() -> Self {
        Self {
            rdrand: false,
            virtio: false,
        }
    }

    fn mark(&mut self, src: EntropySource) {
        match src {
            EntropySource::Rdrand => self.rdrand = true,
            EntropySource::Virtio => self.virtio = true,
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
            used_virtio: self.sources.virtio,
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
        serial::write_line("Warning: RDRAND unsupported; virtio-rng required for entropy");
    }
}

pub fn attach_virtio_rng(mut device: VirtioRng) {
    serial::write_line("virtio-rng attached; requesting entropy refill");
    let mut scratch = [0u8; POOL_SIZE];
    let produced = device.fill_bytes(&mut scratch);

    if produced == 0 {
        serial::write_line("virtio-rng produced no data during attach");
    } else {
        let (stored, ready_now) = {
            let mut state = ENTROPY_STATE.lock();
            let stored = state.ingest(&scratch[..produced], EntropySource::Virtio);
            (stored, state.ready)
        };
        serial::write_fmt(format_args!(
            "virtio-rng delivered {} bytes (stored {})\r\n",
            produced, stored
        ));
        if ready_now {
            serial::write_line("Entropy pool healthy after virtio-rng refill");
        }
        if stored < produced {
            serial::write_line("Entropy pool at capacity; some virtio-rng output unused");
        }
    }

    *VIRTIO_SOURCE.lock() = Some(device);
}

pub fn maintain() {
    let mut device_guard = VIRTIO_SOURCE.lock();
    let device = match device_guard.as_mut() {
        Some(device) => device,
        None => return,
    };

    let now = time::rdtsc();
    let last = LAST_REFRESH_ATTEMPT_TSC.load(Ordering::Relaxed);
    if now.wrapping_sub(last) < REFRESH_INTERVAL_TSC {
        return;
    }
    if LAST_REFRESH_ATTEMPT_TSC
        .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        return;
    }

    if let Some(report) = refresh_pool(device) {
        if !report.before_ready && report.after_ready {
            serial::write_line("Entropy pool healthy after virtio-rng refresh");
        }
        if report.after_fill == POOL_SIZE && report.before_fill != POOL_SIZE {
            serial::write_line("Entropy pool filled to capacity via virtio-rng");
        }
        if report.stored < report.produced {
            serial::write_line("Entropy pool at capacity; some virtio-rng output unused");
        }
    }
}

struct RefreshReport {
    produced: usize,
    stored: usize,
    before_ready: bool,
    after_ready: bool,
    before_fill: usize,
    after_fill: usize,
}

fn refresh_pool(device: &mut VirtioRng) -> Option<RefreshReport> {
    {
        let state = ENTROPY_STATE.lock();
        if state.fill == POOL_SIZE {
            return None;
        }
    }

    let mut scratch = [0u8; POOL_SIZE];
    let produced = device.fill_bytes(&mut scratch);
    if produced == 0 {
        return None;
    }

    let mut state = ENTROPY_STATE.lock();
    let before_ready = state.ready;
    let before_fill = state.fill;
    let stored = state.ingest(&scratch[..produced], EntropySource::Virtio);
    let after_ready = state.ready;
    let after_fill = state.fill;
    drop(state);

    if stored == 0 {
        return None;
    }

    Some(RefreshReport {
        produced,
        stored,
        before_ready,
        after_ready,
        before_fill,
        after_fill,
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
                serial::write_line("Entropy pool low; requesting virtio-rng refresh");
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
