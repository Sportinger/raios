use core::sync::atomic::{AtomicU64, Ordering};

pub struct PeriodicTask {
    interval_tsc: u64,
    last_run: AtomicU64,
}

impl PeriodicTask {
    pub const fn new(interval_tsc: u64) -> Self {
        Self {
            interval_tsc,
            last_run: AtomicU64::new(0),
        }
    }

    pub fn try_run<F>(&self, now_tsc: u64, f: F)
    where
        F: FnOnce(),
    {
        let last = self.last_run.load(Ordering::Relaxed);
        if now_tsc.wrapping_sub(last) < self.interval_tsc {
            return;
        }
        if self
            .last_run
            .compare_exchange(last, now_tsc, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        f();
    }
}

pub const fn ms_to_tsc(ms: u64, tsc_per_ms: u64) -> u64 {
    ms.saturating_mul(tsc_per_ms)
}
