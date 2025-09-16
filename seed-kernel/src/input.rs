use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};

use spin::Once;

use crate::entropy;
use crate::serial;
use crate::time;
use crate::virtio;

const INPUT_RING_CAPACITY: usize = 256;

static RING: InputRing = InputRing::new();
static INIT_ONCE: Once<()> = Once::new();

pub fn init() {
    INIT_ONCE.call_once(|| {
        virtio::input::probe();
    });
}

pub fn poll() {
    if !virtio::input::device_present() {
        return;
    }

    let raw_events = virtio::input::poll();
    if raw_events.is_empty() {
        return;
    }

    let mut entropy_bytes = [0u8; 8];
    entropy::take(&mut entropy_bytes);
    let jitter = u64::from_le_bytes(entropy_bytes) & 0x3;

    let tsc_per_ms = time::tsc_per_ms().max(1);
    let now_tsc = time::rdtsc();
    let ts_ms = now_tsc / tsc_per_ms + jitter;

    let mut inserted = 0usize;

    for raw in raw_events {
        if let Some(kind) = translate_event(&raw) {
            let event = InputEvent { ts_ms, kind };
            RING.push(event);
            inserted += 1;
        }
    }

    if inserted > 0 {
        serial::write_fmt(format_args!(
            "input batch: {} events @ {} ms\r\n",
            inserted, ts_ms
        ));
    }
}

#[allow(dead_code)]
pub fn drain<F: FnMut(InputEvent)>(mut f: F) {
    while let Some(event) = RING.pop() {
        f(event);
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct InputEvent {
    pub ts_ms: u64,
    pub kind: InputEventKind,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum InputEventKind {
    Key { code: u16, pressed: bool },
    Relative(RelativeAxis, i32),
    Absolute { code: u16, value: i32 },
    Raw { type_: u16, code: u16, value: i32 },
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum RelativeAxis {
    X,
    Y,
    Wheel,
    Other(u16),
}

fn translate_event(event: &virtio::input::VirtioInputEvent) -> Option<InputEventKind> {
    match event.type_ {
        0 => None, // EV_SYN
        1 => Some(InputEventKind::Key {
            code: event.code,
            pressed: event.value != 0,
        }),
        2 => Some(InputEventKind::Relative(
            match event.code {
                0 => RelativeAxis::X,
                1 => RelativeAxis::Y,
                8 => RelativeAxis::Wheel,
                other => RelativeAxis::Other(other),
            },
            event.value,
        )),
        3 => Some(InputEventKind::Absolute {
            code: event.code,
            value: event.value,
        }),
        _ => Some(InputEventKind::Raw {
            type_: event.type_,
            code: event.code,
            value: event.value,
        }),
    }
}

struct InputRing {
    head: AtomicUsize,
    tail: AtomicUsize,
    buffer: [UnsafeCell<MaybeUninit<InputEvent>>; INPUT_RING_CAPACITY],
}

unsafe impl Sync for InputRing {}

impl InputRing {
    const fn new() -> Self {
        const SLOT: UnsafeCell<MaybeUninit<InputEvent>> = UnsafeCell::new(MaybeUninit::uninit());
        Self {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            buffer: [SLOT; INPUT_RING_CAPACITY],
        }
    }

    fn push(&self, event: InputEvent) {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) >= INPUT_RING_CAPACITY {
            self.tail.store(tail + 1, Ordering::Release);
        }
        let slot = head % INPUT_RING_CAPACITY;
        unsafe {
            *self.buffer[slot].get() = MaybeUninit::new(event);
        }
        self.head.store(head.wrapping_add(1), Ordering::Release);
    }

    fn pop(&self) -> Option<InputEvent> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        let slot = tail % INPUT_RING_CAPACITY;
        let event = unsafe { (*self.buffer[slot].get()).assume_init_read() };
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(event)
    }
}
