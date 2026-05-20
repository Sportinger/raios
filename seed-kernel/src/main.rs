#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::alloc::Layout;
use core::arch::asm;
use core::hint::spin_loop;
use core::panic::PanicInfo;
use core::ptr;
use limine::request::{
    ExecutableAddressRequest, FramebufferRequest, HhdmRequest, RequestsEndMarker,
    RequestsStartMarker, StackSizeRequest,
};
use limine::BaseRevision;
use linked_list_allocator::LockedHeap;

mod agent_protocol;
mod agent_protocol_module_types;
mod agent_protocol_support;
mod console;
mod e1000;
mod entropy;
mod event_log;
mod framebuffer;
mod input;
mod memory;
mod module_evidence;
mod net;
mod openai;
mod openai_trust;
mod pci;
mod provider;
mod provider_config;
mod provider_trust;
mod ps2;
mod scheduler;
mod serial;
mod service_inventory;
mod system_status;
mod text;
mod time;
mod tls_io;
mod ui;
mod usb;
mod wifi;

#[used]
#[link_section = ".limine_requests_start"]
static LIMINE_REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[link_section = ".limine_requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[link_section = ".limine_requests"]
static KERNEL_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[link_section = ".limine_requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[link_section = ".limine_requests"]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(1024 * 1024);

#[used]
#[link_section = ".limine_requests"]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".limine_requests_end"]
static LIMINE_REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 64 * 1024 * 1024;

#[repr(align(4096))]
struct KernelHeap([u8; HEAP_SIZE]);

static mut HEAP: KernelHeap = KernelHeap([0; HEAP_SIZE]);

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("cli", options(nomem, nostack, preserves_flags));
        asm!(
            "mov rax, cr0",
            "btr rax, 2",
            "bts rax, 1",
            "mov cr0, rax",
            "mov rax, cr4",
            "bts rax, 9",
            "bts rax, 10",
            "mov cr4, rax",
            out("rax") _,
            options(nostack)
        );
    }
    early_main()
}

fn early_main() -> ! {
    unsafe {
        let heap_start = ptr::addr_of_mut!(HEAP.0).cast::<u8>();
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
    }
    serial::init();
    serial::write_line("Seed kernel: early init start");
    if let Some(revision) = BASE_REVISION.loaded_revision() {
        serial::write_fmt(format_args!("Limine loaded base revision: {}", revision));
        serial::write_line("");
    } else {
        serial::write_line("Limine loaded base revision not reported");
    }
    if !BASE_REVISION.is_supported() {
        serial::write_line("Limine base revision request was not satisfied");
    }
    memory::init(
        KERNEL_ADDRESS_REQUEST.get_response(),
        HHDM_REQUEST.get_response(),
    );

    let framebuffer_surface = init_framebuffer();
    if let Some(surface) = framebuffer_surface.as_ref() {
        serial::write_line("Framebuffer negotiated via Limine");
        let info = surface.info();
        serial::write_fmt(format_args!(
            "  resolution={}x{} pitch={}\r\n",
            info.width, info.height, info.pitch
        ));
    } else {
        serial::write_line("No framebuffer response from Limine");
    }

    let framebuffer_info = framebuffer_surface.as_ref().map(|surface| surface.info());
    let mut runtime_status = ui::RuntimeStatus::new();
    runtime_status.framebuffer = framebuffer_info;
    let mut status_ui = ui::StatusUi::new(framebuffer_surface);
    if provider_config::init_default_config() {
        serial::write_line("Default provider loaded: OPENAI API key set");
    }
    console::init();
    usb::init();
    wifi::probe();
    status_ui.render(0, runtime_status);

    time::calibrate_tsc();
    let tsc_per_ms = time::tsc_per_ms();

    entropy::init();

    input::init();
    runtime_status.input_probe_complete = true;
    status_ui.render(uptime_ms(), runtime_status);

    let entropy_ready = entropy::is_ready();
    if !entropy_ready {
        serial::write_line("Entropy unavailable yet; later subsystems will wait");
    }

    if entropy_ready {
        net::init();
        runtime_status.net_probe_complete = true;
    } else {
        serial::write_line("network initialization deferred; waiting for entropy");
    }
    status_ui.render(uptime_ms(), runtime_status);

    let mut periodic = PeriodicTasks::new(tsc_per_ms, entropy_ready);

    loop {
        let tsc_now = now();
        periodic.run(tsc_now, &mut status_ui, &mut runtime_status);
        for _ in 0..100_000 {
            spin_loop();
        }
    }
}

fn init_framebuffer() -> Option<framebuffer::FramebufferSurface> {
    serial::write_line("Framebuffer request: checking response");
    let response = FRAMEBUFFER_REQUEST.get_response()?;
    serial::write_fmt(format_args!(
        "Framebuffer response revision: {}",
        response.revision()
    ));
    serial::write_line("");
    let mut iter = response.framebuffers();
    serial::write_line("Framebuffer response: iterating framebuffers");
    let fb = iter.next()?;
    serial::write_line("Framebuffer response: first framebuffer found");
    framebuffer::FramebufferSurface::from_limine(&fb)
}

fn now() -> u64 {
    time::rdtsc()
}

fn uptime_ms() -> u64 {
    now() / time::tsc_per_ms().max(1)
}

struct PeriodicTasks {
    console: scheduler::PeriodicTask,
    entropy: scheduler::PeriodicTask,
    net: scheduler::PeriodicTask,
    input: scheduler::PeriodicTask,
    usb_rescan: scheduler::PeriodicTask,
    provider: scheduler::PeriodicTask,
    ui: scheduler::PeriodicTask,
    entropy_ready: bool,
    net_started: bool,
}

impl PeriodicTasks {
    fn new(tsc_per_ms: u64, entropy_ready: bool) -> Self {
        Self {
            console: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(8, tsc_per_ms)),
            entropy: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(8, tsc_per_ms)),
            net: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(50, tsc_per_ms)),
            input: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(8, tsc_per_ms)),
            usb_rescan: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(1000, tsc_per_ms)),
            provider: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(50, tsc_per_ms)),
            ui: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(250, tsc_per_ms)),
            entropy_ready,
            net_started: entropy_ready,
        }
    }

    fn run(
        &mut self,
        now_tsc: u64,
        status_ui: &mut ui::StatusUi,
        runtime_status: &mut ui::RuntimeStatus,
    ) {
        self.console.try_run(now_tsc, || {
            if console::poll(*runtime_status) {
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
        });
        self.entropy.try_run(now_tsc, || entropy::maintain());
        if !self.entropy_ready && entropy::is_ready() {
            serial::write_line("Entropy ready; starting network bring-up");
            net::init();
            runtime_status.net_probe_complete = true;
            self.entropy_ready = true;
            self.net_started = true;
            status_ui.render(uptime_ms(), *runtime_status);
            return;
        }
        self.usb_rescan.try_run(now_tsc, || {
            if !usb::input_active() && usb::rescan_if_input_missing() {
                runtime_status.input_probe_complete = true;
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
        });
        self.input.try_run(now_tsc, || {
            let pointer_changed = input::poll();
            let ui_changed = status_ui.handle_pointer_interaction();
            if ui_changed {
                status_ui.render_forced(uptime_ms(), *runtime_status);
            } else if pointer_changed {
                status_ui.render_pointer();
            }
        });
        if self.entropy_ready {
            if !self.net_started {
                net::init();
                runtime_status.net_probe_complete = true;
                self.net_started = true;
            }
            self.net.try_run(now_tsc, || net::poll());
            self.provider.try_run(now_tsc, || {
                if let Some(event) = provider::poll() {
                    let _route = event.route;
                    console::write_event(format_args!("{}", event.line.as_str()));
                    status_ui.render_forced(uptime_ms(), *runtime_status);
                }
            });
        }
        self.ui.try_run(now_tsc, || {
            status_ui.render(uptime_ms(), *runtime_status);
        });
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    serial::write_fmt(format_args!("allocation failure: {:?}", layout));
    panic!("allocation failure");
}

#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, value: i32, count: usize) -> *mut u8 {
    let mut index = 0usize;
    while index < count {
        ptr::write_volatile(dest.add(index), value as u8);
        index += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    let mut index = 0usize;
    while index < count {
        let value = ptr::read_volatile(src.add(index));
        ptr::write_volatile(dest.add(index), value);
        index += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    if (dest as usize) <= (src as usize) {
        let mut index = 0usize;
        while index < count {
            let value = ptr::read_volatile(src.add(index));
            ptr::write_volatile(dest.add(index), value);
            index += 1;
        }
    } else {
        let mut index = count;
        while index > 0 {
            index -= 1;
            let value = ptr::read_volatile(src.add(index));
            ptr::write_volatile(dest.add(index), value);
        }
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, count: usize) -> i32 {
    for i in 0..count {
        let lhs = ptr::read_volatile(a.add(i));
        let rhs = ptr::read_volatile(b.add(i));
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
