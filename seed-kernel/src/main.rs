#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::alloc::Layout;
use core::arch::asm;
use core::panic::PanicInfo;
use core::ptr;

use limine::request::FramebufferRequest;
use limine::BaseRevision;
use linked_list_allocator::LockedHeap;

mod framebuffer;
mod entropy;
mod pci;
mod serial;
mod virtio;
mod text;
mod time;

#[used]
#[link_section = ".limine_reqs"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[link_section = ".limine_reqs"]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 16 * 1024 * 1024;
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("cli", options(nomem, nostack, preserves_flags));
    }
    early_main()
}

fn early_main() -> ! {
    unsafe {
        let heap_start = ptr::addr_of_mut!(HEAP).cast::<u8>();
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
    }
    serial::init();
    serial::write_line("Seed kernel: early init start");

    entropy::init();

    if let Some(rng) = virtio::rng::probe() {
        entropy::attach_virtio_rng(rng);
        entropy::maintain(time::rdtsc());
    }

    if !entropy::is_ready() {
        serial::write_line("Entropy unavailable yet; later subsystems will wait");
    }

    let fb_surface = init_framebuffer();

    if let Some(mut surface) = fb_surface {
        serial::write_line("Framebuffer negotiated via Limine");
        let info = surface.info();
        serial::write_fmt(format_args!(
            "  resolution={}x{} pitch={}",
            info.width, info.height, info.pitch
        ));
        use framebuffer::Color;
        surface.fill(Color::new(20, 24, 28));
        surface.fill_rect(20, 20, 320, 140, Color::new(40, 90, 180));
        surface.fill_rect(30, 40, 300, 36, Color::new(240, 240, 240));
        text::draw_text(&mut surface, 40, 48, "SEEDOS STAGE-0", Color::new(20, 28, 40), Some(Color::new(240, 240, 240)));
        surface.present();
        serial::write_line("Framebuffer hello overlay drawn");
    } else {
        serial::write_line("No framebuffer response from Limine");
    }

    loop {
        let now = time::rdtsc();
        entropy::maintain(now);
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

fn init_framebuffer() -> Option<framebuffer::FramebufferSurface> {
    let response = FRAMEBUFFER_REQUEST.get_response()?;
    let mut iter = response.framebuffers();
    let fb = iter.next()?;
    framebuffer::FramebufferSurface::from_limine(&fb)
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    serial::write_fmt(format_args!("allocation failure: {:?}", layout));
    panic!("allocation failure");
}

#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, value: i32, count: usize) -> *mut u8 {
    core::ptr::write_bytes(dest, value as u8, count);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    core::ptr::copy_nonoverlapping(src, dest, count);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    core::ptr::copy(src, dest, count);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, count: usize) -> i32 {
    for i in 0..count {
        let lhs = *a.add(i);
        let rhs = *b.add(i);
        if lhs != rhs {
            return lhs as i32 - rhs as i32;
        }
    }
    0
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::write_line("*** KERNEL PANIC ***");
    let message = info.message();
    if let Some(s) = message.as_str() {
        serial::write_line(s);
    } else {
        serial::write_fmt(format_args!("{}", message));
        serial::write_line("");
    }
    if let Some(location) = info.location() {
        serial::write_fmt(format_args!(
            "panic at {}:{}",
            location.file(),
            location.line()
        ));
        serial::write_line("");
    }
    loop {
        unsafe { asm!("cli; hlt", options(nomem, nostack, preserves_flags)) }
    }
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    loop {
        unsafe { asm!("cli; hlt", options(nomem, nostack, preserves_flags)) }
    }
}
