use core::cell::{Cell, UnsafeCell};
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use spin::{Mutex, Once};

use crate::entropy;
use crate::ps2;
use crate::serial;
use crate::time;
use crate::usb;

const INPUT_RING_CAPACITY: usize = 256;
const POINTER_DEFAULT_WIDTH: i32 = 1280;
const POINTER_DEFAULT_HEIGHT: i32 = 800;
const POINTER_ABSOLUTE_MAX: i32 = 0x7FFF;
const MOUSE_LEFT: u8 = 1 << 0;
const MOUSE_RIGHT: u8 = 1 << 1;
const MOUSE_MIDDLE: u8 = 1 << 2;

static RING: InputRing = InputRing::new();
static MOUSE: Mutex<MouseState> = Mutex::new(MouseState::new());
static INIT_ONCE: Once<()> = Once::new();
static SHIFT_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn init() {
    INIT_ONCE.call_once(|| {
        if !usb::keyboard_active() && !usb::mouse_active() {
            ps2::init_keyboard_polling();
        }
    });
}

pub fn device_present() -> bool {
    usb::keyboard_active() || usb::mouse_active() || ps2::active()
}

pub fn device_detail() -> &'static str {
    if usb::keyboard_active() || usb::mouse_active() {
        usb::input_detail()
    } else if ps2::active() {
        "PS/2 KEYBOARD POLLING"
    } else {
        "DEVICE ABSENT OR UNSUPPORTED"
    }
}

pub fn poll() -> bool {
    let mut pointer_changed = false;
    pointer_changed |= poll_usb();
    poll_ps2();
    pointer_changed
}

fn poll_ps2() {
    if !ps2::present() {
        return;
    }

    let ts_ms = timestamp_ms();
    let mut inserted = 0usize;
    ps2::poll(|code, pressed| {
        RING.push(InputEvent {
            ts_ms,
            kind: InputEventKind::Key { code, pressed },
        });
        inserted += 1;
    });

    if inserted > 0 {
        serial::write_fmt(format_args!(
            "ps/2 input batch: {} events @ {} ms\r\n",
            inserted, ts_ms
        ));
    }
}

fn poll_usb() -> bool {
    if !usb::keyboard_active() && !usb::mouse_active() {
        return false;
    }

    let ts_ms = Cell::new(0u64);
    let has_timestamp = Cell::new(false);
    let inserted = Cell::new(0usize);
    let pointer_changed = Cell::new(false);
    usb::poll_input(
        |code, pressed| {
            if !has_timestamp.get() {
                ts_ms.set(timestamp_ms());
                has_timestamp.set(true);
            }
            RING.push(InputEvent {
                ts_ms: ts_ms.get(),
                kind: InputEventKind::Key { code, pressed },
            });
            inserted.set(inserted.get() + 1);
        },
        |report| {
            if !has_timestamp.get() {
                ts_ms.set(timestamp_ms());
                has_timestamp.set(true);
            }
            if report.dx != 0 {
                let kind = InputEventKind::Relative(RelativeAxis::X, report.dx as i32);
                pointer_changed.set(update_mouse(kind) || pointer_changed.get());
                RING.push(InputEvent {
                    ts_ms: ts_ms.get(),
                    kind,
                });
                inserted.set(inserted.get() + 1);
            }
            if report.dy != 0 {
                let kind = InputEventKind::Relative(RelativeAxis::Y, report.dy as i32);
                pointer_changed.set(update_mouse(kind) || pointer_changed.get());
                RING.push(InputEvent {
                    ts_ms: ts_ms.get(),
                    kind,
                });
                inserted.set(inserted.get() + 1);
            }
            if report.wheel != 0 {
                let kind = InputEventKind::Relative(RelativeAxis::Wheel, report.wheel as i32);
                pointer_changed.set(update_mouse(kind) || pointer_changed.get());
                RING.push(InputEvent {
                    ts_ms: ts_ms.get(),
                    kind,
                });
                inserted.set(inserted.get() + 1);
            }
            for (code, mask) in [(272, MOUSE_LEFT), (273, MOUSE_RIGHT), (274, MOUSE_MIDDLE)] {
                let pressed = report.buttons & mask != 0;
                let kind = InputEventKind::Key { code, pressed };
                if update_mouse(kind) {
                    RING.push(InputEvent {
                        ts_ms: ts_ms.get(),
                        kind,
                    });
                    inserted.set(inserted.get() + 1);
                    pointer_changed.set(true);
                }
            }
        },
    );

    if inserted.get() > 0 {
        serial::write_fmt(format_args!(
            "usb input batch: {} events @ {} ms\r\n",
            inserted.get(),
            ts_ms.get()
        ));
    }
    if pointer_changed.get() {
        serial::write_fmt(format_args!(
            "usb pointer: {},{} buttons {}\r\n",
            mouse_snapshot().x,
            mouse_snapshot().y,
            mouse_snapshot().buttons
        ));
    }
    pointer_changed.get()
}

fn timestamp_ms() -> u64 {
    let tsc_per_ms = time::tsc_per_ms().max(1);
    let now_tsc = time::rdtsc();
    let jitter = if entropy::is_ready() {
        let mut entropy_bytes = [0u8; 8];
        entropy::take(&mut entropy_bytes);
        u64::from_le_bytes(entropy_bytes) & 0x3
    } else {
        0
    };

    now_tsc / tsc_per_ms + jitter
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

pub fn set_pointer_bounds(width: usize, height: usize) {
    MOUSE.lock().set_bounds(width, height);
}

pub fn mouse_snapshot() -> MouseSnapshot {
    MOUSE.lock().snapshot()
}

#[derive(Clone, Copy)]
pub struct MouseSnapshot {
    pub x: usize,
    pub y: usize,
    pub buttons: u8,
    pub seen: bool,
    pub sequence: u64,
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

fn update_mouse(kind: InputEventKind) -> bool {
    let mut mouse = MOUSE.lock();
    match kind {
        InputEventKind::Relative(axis, value) => mouse.apply_relative(axis, value),
        InputEventKind::Absolute { code, value } => mouse.apply_absolute(code, value),
        InputEventKind::Key { code, pressed } => match mouse_button_mask(code) {
            Some(mask) => mouse.apply_button(mask, pressed),
            None => false,
        },
        InputEventKind::Raw { .. } => false,
    }
}

fn mouse_button_mask(code: u16) -> Option<u8> {
    match code {
        272 => Some(MOUSE_LEFT),
        273 => Some(MOUSE_RIGHT),
        274 => Some(MOUSE_MIDDLE),
        _ => None,
    }
}

struct MouseState {
    x: i32,
    y: i32,
    max_x: i32,
    max_y: i32,
    buttons: u8,
    seen: bool,
    sequence: u64,
}

impl MouseState {
    const fn new() -> Self {
        Self {
            x: POINTER_DEFAULT_WIDTH / 2,
            y: POINTER_DEFAULT_HEIGHT / 2,
            max_x: POINTER_DEFAULT_WIDTH - 1,
            max_y: POINTER_DEFAULT_HEIGHT - 1,
            buttons: 0,
            seen: false,
            sequence: 0,
        }
    }

    fn set_bounds(&mut self, width: usize, height: usize) {
        self.max_x = usize_to_i32(width.saturating_sub(1));
        self.max_y = usize_to_i32(height.saturating_sub(1));
        if !self.seen {
            self.x = self.max_x / 2;
            self.y = self.max_y / 2;
            return;
        }
        self.x = clamp_i32(self.x, 0, self.max_x);
        self.y = clamp_i32(self.y, 0, self.max_y);
    }

    fn apply_relative(&mut self, axis: RelativeAxis, value: i32) -> bool {
        if value == 0 {
            return false;
        }
        match axis {
            RelativeAxis::X => self.move_by(value, 0),
            RelativeAxis::Y => self.move_by(0, value),
            RelativeAxis::Wheel | RelativeAxis::Other(_) => {
                self.mark_seen();
                false
            }
        }
    }

    fn apply_absolute(&mut self, code: u16, value: i32) -> bool {
        match code {
            0 => {
                let x = scale_absolute(value, self.max_x);
                self.set_position(x, self.y)
            }
            1 => {
                let y = scale_absolute(value, self.max_y);
                self.set_position(self.x, y)
            }
            _ => false,
        }
    }

    fn apply_button(&mut self, mask: u8, pressed: bool) -> bool {
        let previous = self.buttons;
        if pressed {
            self.buttons |= mask;
        } else {
            self.buttons &= !mask;
        }
        self.mark_seen();
        if self.buttons == previous {
            return false;
        }
        self.bump();
        true
    }

    fn move_by(&mut self, dx: i32, dy: i32) -> bool {
        self.set_position(self.x.saturating_add(dx), self.y.saturating_add(dy))
    }

    fn set_position(&mut self, x: i32, y: i32) -> bool {
        let x = clamp_i32(x, 0, self.max_x);
        let y = clamp_i32(y, 0, self.max_y);
        let changed = x != self.x || y != self.y || !self.seen;
        self.x = x;
        self.y = y;
        self.mark_seen();
        if changed {
            self.bump();
        }
        changed
    }

    fn mark_seen(&mut self) {
        self.seen = true;
    }

    fn bump(&mut self) {
        self.sequence = self.sequence.wrapping_add(1);
    }

    fn snapshot(&self) -> MouseSnapshot {
        MouseSnapshot {
            x: self.x as usize,
            y: self.y as usize,
            buttons: self.buttons,
            seen: self.seen,
            sequence: self.sequence,
        }
    }
}

fn scale_absolute(value: i32, max: i32) -> i32 {
    let value = clamp_i32(value, 0, POINTER_ABSOLUTE_MAX) as i64;
    ((value * max as i64) / POINTER_ABSOLUTE_MAX as i64) as i32
}

fn usize_to_i32(value: usize) -> i32 {
    if value > i32::MAX as usize {
        i32::MAX
    } else {
        value as i32
    }
}

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
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
