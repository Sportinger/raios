use core::arch::asm;
use core::arch::x86_64::__cpuid;
use core::cmp;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use crate::serial;
use crate::virtio::rng::VirtioRng;

const POOL_SIZE: usize = 64;
const MIN_READY_BYTES: usize = 32;
const REFRESH_INTERVAL_TSC: u64 = 25_000_000; // ~8 ms at 3 GHz

static ENTROPY_STATE: Mutex<EntropyState> = Mutex::new(EntropyState {
    ready: false,
    pool: [0u8; POOL_SIZE],
    fill: 0,
});

static VIRTIO_SOURCE: Mutex<Option<VirtioRng>> = Mutex::new(None);
static LAST_REFRESH_ATTEMPT_TSC: AtomicU64 = AtomicU64::new(0);

struct EntropyState {
    ready: bool,
    pool: [u8; POOL_SIZE],
    fill: usize,
}

impl EntropyState {
    fn reset(&mut self) {
        self.ready = false;
        self.fill = 0;
    }

    fn ingest(&mut self, data: &[u8]) -> usize {
        let capacity = POOL_SIZE.saturating_sub(self.fill);
        let to_copy = cmp::min(capacity, data.len());
        if to_copy > 0 {
            let start = self.fill;
            let end = start + to_copy;
            self.pool[start..end].copy_from_slice(&data[..to_copy]);
            self.fill = end;
            if self.fill >= MIN_READY_BYTES {
                self.ready = true;
            }
        }
        to_copy
    }
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
            state.ingest(&scratch[..collected]);
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
            let stored = state.ingest(&scratch[..produced]);
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

pub fn maintain(now_tsc: u64) {
    let mut device_guard = VIRTIO_SOURCE.lock();
    let device = match device_guard.as_mut() {
        Some(device) => device,
        None => return,
    };

    let last = LAST_REFRESH_ATTEMPT_TSC.load(Ordering::Relaxed);
    if now_tsc.wrapping_sub(last) < REFRESH_INTERVAL_TSC {
        return;
    }
    if LAST_REFRESH_ATTEMPT_TSC
        .compare_exchange(last, now_tsc, Ordering::Relaxed, Ordering::Relaxed)
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
    let stored = state.ingest(&scratch[..produced]);
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

#[allow(dead_code)]
pub fn fill_bytes(buffer: &mut [u8]) -> usize {
    let state = ENTROPY_STATE.lock();
    let available = state.fill.min(buffer.len());
    if available > 0 {
        buffer[..available].copy_from_slice(&state.pool[..available]);
    }
    available
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
