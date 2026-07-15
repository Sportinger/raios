#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::alloc::Layout;
use core::arch::asm;
use core::hint::spin_loop;
use core::panic::PanicInfo;
use core::ptr;
use limine::modules::InternalModule;
use limine::request::{
    ExecutableAddressRequest, ExecutableFileRequest, FramebufferRequest, HhdmRequest,
    ModuleRequest, RequestsEndMarker, RequestsStartMarker, StackSizeRequest,
};
use limine::BaseRevision;
use linked_list_allocator::LockedHeap;

mod agent_protocol;
mod agent_protocol_distribution;
mod agent_protocol_honesty;
mod agent_protocol_memory;
mod agent_protocol_module_approval;
mod agent_protocol_module_attestation;
mod agent_protocol_module_audit;
mod agent_protocol_module_grant;
mod agent_protocol_module_load_gate;
mod agent_protocol_module_load_gate_render;
mod agent_protocol_module_load_gate_selftest;
mod agent_protocol_module_load_gate_selftest_emit;
mod agent_protocol_module_load_gate_selftest_eval;
mod agent_protocol_module_load_gate_selftest_reference_cases;
mod agent_protocol_module_loader_artifact_hash_binding;
mod agent_protocol_module_loader_fact;
mod agent_protocol_module_loader_identity;
mod agent_protocol_module_loader_runtime;
mod agent_protocol_module_reference;
mod agent_protocol_module_service_slot;
mod agent_protocol_module_service_slot_allocator;
mod agent_protocol_module_service_slot_allocator_projection;
mod agent_protocol_module_types;
mod agent_protocol_module_write_boundary;
mod agent_protocol_module_write_boundary_append_contract;
mod agent_protocol_module_write_boundary_append_engine;
mod agent_protocol_module_write_boundary_append_intent;
mod agent_protocol_module_write_boundary_append_payload_hash;
mod agent_protocol_module_write_boundary_availability;
mod agent_protocol_module_write_boundary_boundary;
mod agent_protocol_module_write_boundary_emit;
mod agent_protocol_module_write_boundary_storage_layout;
mod agent_protocol_module_write_boundary_write_policy;
mod agent_protocol_policy;
mod agent_protocol_program;
mod agent_protocol_project;
mod agent_protocol_project_build;
mod agent_protocol_project_dependency;
mod agent_protocol_project_editor;
mod agent_protocol_project_install;
mod agent_protocol_project_query;
mod agent_protocol_project_run;
mod agent_protocol_provider;
mod agent_protocol_support;
mod agent_protocol_system;
mod agent_protocol_time;
mod agent_protocol_ui;
mod agent_protocol_wasm;
mod ahci;
mod console;
mod core_policy_runtime;
mod current_boot_service;
mod descriptor_sources;
mod distribution_candidate;
mod e1000;
mod echo_service;
mod entropy;
mod event_log;
mod event_log_evidence;
mod event_log_hello_selftest;
mod event_log_module_checks;
mod event_log_provider_selftest;
mod event_log_types;
mod framebuffer;
mod granted_candidate_service;
mod hello_service;
mod input;
mod iommu_vtd;
#[allow(dead_code)]
mod marvell_wifi_pcie;
mod memory;
mod memory_store;
mod module_candidate_channel;
mod module_candidate_intake;
mod module_evidence;
mod net;
mod openai;
mod openai_trust;
mod owner_key;
mod pci;
mod personal_shell_service;
mod program_workspace;
mod project_app_autoload;
mod project_build;
mod project_dependency;
mod project_dependency_store;
mod project_editor;
mod project_install_store;
mod project_query;
mod project_store;
mod project_workspace;
mod provider;
mod provider_config;
mod provider_trust;
mod ps2;
mod scheduler;
mod secret_vault;
mod secure_overlay;
mod serial;
mod service_inventory;
mod shell_host;
mod structured_store;
mod structured_store_c1;
mod system_problem_facts;
mod system_status;
mod text;
mod time;
mod tls_io;
mod tpm2_transport;
mod ui;
mod usb;
mod wasm_runtime;
mod wifi;
mod workspace_candidate_service;

#[used]
#[link_section = ".limine_requests_start"]
static LIMINE_REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[link_section = ".limine_requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[link_section = ".limine_requests"]
static KERNEL_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

static CORE_POLICY_INTERNAL_MODULE: InternalModule =
    InternalModule::new().with_path(c"/raios/core-policy.bin");
static CORE_POLICY_INTERNAL_MODULES: [&InternalModule; 1] = [&CORE_POLICY_INTERNAL_MODULE];

#[used]
#[link_section = ".limine_requests"]
static EXECUTABLE_FILE_REQUEST: ExecutableFileRequest = ExecutableFileRequest::new();

#[used]
#[link_section = ".limine_requests"]
static MODULE_REQUEST: ModuleRequest =
    ModuleRequest::new().with_internal_modules(&CORE_POLICY_INTERNAL_MODULES);

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
    let mut runtime_status = system_status::RuntimeStatus::new();
    runtime_status.framebuffer = framebuffer_info;
    let mut status_ui = shell_host::ShellHost::new(framebuffer_surface);
    if provider_config::init_default_config() {
        serial::write_line("Default provider loaded: OPENAI API key set");
    }
    console::init();
    usb::init();
    let iommu_report = iommu_vtd::probe();
    if iommu_report.present {
        let remapping = if iommu_report.remapping_enabled {
            "ENABLED"
        } else {
            "DISABLED"
        };
        serial::write_fmt(format_args!(
            "iommu-vtd: present=true ver=0x{:08x} drhd={} cap=0x{:016x} ecap=0x{:016x} remapping={} (slice1 detect-only)\r\n",
            iommu_report.version,
            iommu_report.drhd_count,
            iommu_report.cap,
            iommu_report.ecap,
            remapping
        ));
    } else {
        serial::write_fmt(format_args!("iommu-vtd: {}\r\n", iommu_report.reason));
    }
    owner_key::ensure_hardware_binding_probe();
    wifi::probe();
    status_ui.render(0, runtime_status);

    time::calibrate_tsc();
    let tsc_per_ms = time::tsc_per_ms();

    entropy::init();
    core_policy_runtime::init(
        EXECUTABLE_FILE_REQUEST.get_response(),
        MODULE_REQUEST.get_response(),
    );
    structured_store_c1::run_disposable_qemu_boot_probe();
    project_app_autoload::run_boot_autoload();
    agent_protocol::run_provider_autoload();

    input::init();
    runtime_status.input_probe_complete = true;
    status_ui.render(uptime_ms(), runtime_status);

    let entropy_ready = entropy::is_ready();
    if !entropy_ready {
        serial::write_line("Entropy unavailable yet; later subsystems will wait");
    } else {
        owner_key::ensure_current_boot_candidate();
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
    marvell_wifi_fw: scheduler::PeriodicTask,
    usb_rescan: scheduler::PeriodicTask,
    provider: scheduler::PeriodicTask,
    ui: scheduler::PeriodicTask,
    active_beyond_env: Option<wasm_runtime::ActiveBeyondEnvInvocation>,
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
            marvell_wifi_fw: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(1, tsc_per_ms)),
            usb_rescan: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(500, tsc_per_ms)),
            provider: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(50, tsc_per_ms)),
            ui: scheduler::PeriodicTask::new(scheduler::ms_to_tsc(250, tsc_per_ms)),
            active_beyond_env: None,
            entropy_ready,
            net_started: entropy_ready,
        }
    }

    fn run(
        &mut self,
        now_tsc: u64,
        status_ui: &mut shell_host::ShellHost,
        runtime_status: &mut system_status::RuntimeStatus,
    ) {
        self.console.try_run(now_tsc, || {
            if console::poll(*runtime_status, |event| {
                status_ui.handle_input_event(event, *runtime_status)
            }) {
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
        });
        self.entropy.try_run(now_tsc, || entropy::maintain());
        if !self.entropy_ready && entropy::is_ready() {
            serial::write_line("Entropy ready; starting network bring-up");
            owner_key::ensure_current_boot_candidate();
            net::init();
            runtime_status.net_probe_complete = true;
            self.entropy_ready = true;
            self.net_started = true;
            status_ui.render(uptime_ms(), *runtime_status);
            return;
        }
        self.usb_rescan.try_run(now_tsc, || {
            if usb::poll_hotplug() {
                runtime_status.input_probe_complete = true;
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
            if !usb::input_active() && usb::rescan_if_input_missing() {
                runtime_status.input_probe_complete = true;
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
        });
        let mut input_boundary = false;
        self.input.try_run(now_tsc, || {
            input_boundary = true;
            let pointer_changed = input::poll();
            let ui_changed = status_ui.handle_pointer_interaction(*runtime_status);
            if ui_changed {
                status_ui.render_forced(uptime_ms(), *runtime_status);
            } else if pointer_changed {
                status_ui.render_pointer();
            }
        });
        if input_boundary {
            let already_active = self.active_beyond_env.is_some();
            if !already_active {
                if let Some(request) = wasm_runtime::take_beyond_env_fixture_request() {
                    self.active_beyond_env = wasm_runtime::start_beyond_env_fixture(
                        request,
                        uptime_ms(),
                        input::secure_attention_kill_generation(),
                    );
                }
            }
            if already_active {
                let terminal = self
                    .active_beyond_env
                    .as_mut()
                    .map(|active| {
                        active.pump(
                            uptime_ms(),
                            input::secure_attention_kill_generation(),
                            input::secure_attention_kill_boot_ms(),
                        )
                    })
                    .unwrap_or(false);
                if terminal {
                    self.active_beyond_env = None;
                }
            }
        }
        self.marvell_wifi_fw.try_run(now_tsc, || {
            if marvell_wifi_pcie::poll() {
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
            if marvell_wifi_pcie::poll_hw_spec() {
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
            if marvell_wifi_pcie::poll_scan_ext() {
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
            if marvell_wifi_pcie::poll_connection() {
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
            if marvell_wifi_pcie::poll_event_ring() {
                status_ui.render_forced(uptime_ms(), *runtime_status);
            }
        });
        if self.entropy_ready {
            if !self.net_started {
                net::init();
                runtime_status.net_probe_complete = true;
                self.net_started = true;
            }
            self.net.try_run(now_tsc, || net::poll());
            if self.active_beyond_env.is_none() {
                self.provider.try_run(now_tsc, || {
                    if let Some(event) = provider::poll() {
                        let _route = event.route;
                        match event.kind {
                            provider::EventKind::Answer(answer) => match event.target {
                                provider::RequestTarget::Conversation => {
                                    console::write_provider_answer(answer.as_str());
                                }
                                provider::RequestTarget::ProgramWorkspace => {
                                    let outcome =
                                        program_workspace::accept_provider_answer(event.id, answer);
                                    console::write_program_outcome(event.id, outcome);
                                }
                            },
                            provider::EventKind::Error(error) => {
                                if event.target == provider::RequestTarget::ProgramWorkspace {
                                    program_workspace::note_provider_error(event.id);
                                }
                                console::write_event(format_args!("{}", error));
                            }
                        }
                        status_ui.render_forced(uptime_ms(), *runtime_status);
                    }
                });
            }
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

#[no_mangle]
pub unsafe extern "C" fn strlen(value: *const u8) -> usize {
    let mut len = 0usize;
    while ptr::read_volatile(value.add(len)) != 0 {
        len += 1;
    }
    len
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
