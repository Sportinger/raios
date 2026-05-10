use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use limine::response::ExecutableAddressResponse;

use crate::serial;

static KERNEL_PHYSICAL_BASE: AtomicU64 = AtomicU64::new(0);
static KERNEL_VIRTUAL_BASE: AtomicU64 = AtomicU64::new(0);
static KERNEL_ADDRESS_READY: AtomicBool = AtomicBool::new(false);

pub fn init(response: Option<&ExecutableAddressResponse>) {
    let Some(response) = response else {
        serial::write_line("Kernel address response unavailable; DMA translation disabled");
        KERNEL_ADDRESS_READY.store(false, Ordering::Release);
        return;
    };

    let physical_base = response.physical_base();
    let virtual_base = response.virtual_base();
    KERNEL_PHYSICAL_BASE.store(physical_base, Ordering::Relaxed);
    KERNEL_VIRTUAL_BASE.store(virtual_base, Ordering::Relaxed);
    KERNEL_ADDRESS_READY.store(true, Ordering::Release);

    serial::write_fmt(format_args!(
        "Kernel address map: physical=0x{:x} virtual=0x{:x}\r\n",
        physical_base, virtual_base
    ));
}

pub fn virt_to_phys<T>(ptr: *const T) -> Option<u64> {
    let virt = ptr as u64;
    if !KERNEL_ADDRESS_READY.load(Ordering::Acquire) {
        return identity_phys(virt);
    }

    let virtual_base = KERNEL_VIRTUAL_BASE.load(Ordering::Relaxed);
    let physical_base = KERNEL_PHYSICAL_BASE.load(Ordering::Relaxed);
    if virt >= virtual_base {
        return virt.checked_sub(virtual_base)?.checked_add(physical_base);
    }

    identity_phys(virt)
}

fn identity_phys(virt: u64) -> Option<u64> {
    if virt < 0x0000_8000_0000_0000 {
        Some(virt)
    } else {
        None
    }
}
