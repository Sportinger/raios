use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use spin::Once;

use crate::entropy;
use crate::serial;
use crate::time;
use crate::virtio;

const INPUT_RING_CAPACITY: usize = 256;

static RING: InputRing = InputRing::new();
static INIT_ONCE: Once<()> = Once::new();
static SHIFT_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn init() {
    INIT_ONCE.call_once(|| {
        virtio::input::probe();
    });
}

pub fn device_present() -> bool {
    virtio::input::device_present()
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

pub fn drain_console_bytes<F: FnMut(u8)>(mut f: F) {
    drain(|event| {
        if let Some(byte) = event_to_console_byte(event) {
            f(byte);
        }
    });
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

fn event_to_console_byte(event: InputEvent) -> Option<u8> {
    let InputEventKind::Key { code, pressed } = event.kind else {
        return None;
    };

    match code {
        42 | 54 => {
            SHIFT_ACTIVE.store(pressed, Ordering::Relaxed);
            return None;
        }
        _ if !pressed => return None,
        14 => return Some(0x08),
        28 => return Some(b'\r'),
        57 => return Some(b' '),
        _ => {}
    }

    let shifted = SHIFT_ACTIVE.load(Ordering::Relaxed);
    keycode_to_ascii(code, shifted)
}

fn keycode_to_ascii(code: u16, shifted: bool) -> Option<u8> {
    let byte = match code {
        2 => digit_ascii(b'1', b'!', shifted),
        3 => digit_ascii(b'2', b'@', shifted),
        4 => digit_ascii(b'3', b'#', shifted),
        5 => digit_ascii(b'4', b'$', shifted),
        6 => digit_ascii(b'5', b'%', shifted),
        7 => digit_ascii(b'6', b'^', shifted),
        8 => digit_ascii(b'7', b'&', shifted),
        9 => digit_ascii(b'8', b'*', shifted),
        10 => digit_ascii(b'9', b'(', shifted),
        11 => digit_ascii(b'0', b')', shifted),
        12 => digit_ascii(b'-', b'_', shifted),
        13 => digit_ascii(b'=', b'+', shifted),
        16 => letter_ascii(b'q', shifted),
        17 => letter_ascii(b'w', shifted),
        18 => letter_ascii(b'e', shifted),
        19 => letter_ascii(b'r', shifted),
        20 => letter_ascii(b't', shifted),
        21 => letter_ascii(b'y', shifted),
        22 => letter_ascii(b'u', shifted),
        23 => letter_ascii(b'i', shifted),
        24 => letter_ascii(b'o', shifted),
        25 => letter_ascii(b'p', shifted),
        26 => digit_ascii(b'[', b'{', shifted),
        27 => digit_ascii(b']', b'}', shifted),
        30 => letter_ascii(b'a', shifted),
        31 => letter_ascii(b's', shifted),
        32 => letter_ascii(b'd', shifted),
        33 => letter_ascii(b'f', shifted),
        34 => letter_ascii(b'g', shifted),
        35 => letter_ascii(b'h', shifted),
        36 => letter_ascii(b'j', shifted),
        37 => letter_ascii(b'k', shifted),
        38 => letter_ascii(b'l', shifted),
        39 => digit_ascii(b';', b':', shifted),
        40 => digit_ascii(b'\'', b'"', shifted),
        41 => digit_ascii(b'`', b'~', shifted),
        43 => digit_ascii(b'\\', b'|', shifted),
        44 => letter_ascii(b'z', shifted),
        45 => letter_ascii(b'x', shifted),
        46 => letter_ascii(b'c', shifted),
        47 => letter_ascii(b'v', shifted),
        48 => letter_ascii(b'b', shifted),
        49 => letter_ascii(b'n', shifted),
        50 => letter_ascii(b'm', shifted),
        51 => digit_ascii(b',', b'<', shifted),
        52 => digit_ascii(b'.', b'>', shifted),
        53 => digit_ascii(b'/', b'?', shifted),
        _ => return None,
    };
    Some(byte)
}

fn letter_ascii(lower: u8, shifted: bool) -> u8 {
    if shifted {
        lower.to_ascii_uppercase()
    } else {
        lower
    }
}

fn digit_ascii(normal: u8, shifted: u8, is_shifted: bool) -> u8 {
    if is_shifted {
        shifted
    } else {
        normal
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
