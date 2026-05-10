use alloc::vec::Vec;
use core::mem::{size_of, MaybeUninit};
use core::ptr::{addr_of, addr_of_mut};
use core::sync::atomic::{fence, AtomicU16, Ordering};

use spin::Mutex;

use crate::memory;
use crate::pci::{self, PciAddress};
use crate::serial;

use super::{device_id, VIRTIO_VENDOR_ID};

const CAP_ID_VENDOR_SPECIFIC: u8 = 0x09;
const CFG_TYPE_COMMON: u8 = 1;
const CFG_TYPE_NOTIFY: u8 = 2;
const CFG_TYPE_ISR: u8 = 3;
const CFG_TYPE_DEVICE: u8 = 4;

const DEVICE_STATUS_ACKNOWLEDGE: u8 = 0x01;
const DEVICE_STATUS_DRIVER: u8 = 0x02;
const DEVICE_STATUS_DRIVER_OK: u8 = 0x04;
const DEVICE_STATUS_FEATURES_OK: u8 = 0x08;

const FEATURE_VERSION_1: u64 = 1u64 << 32;

const EVENT_QUEUE_SIZE: usize = 64;
const EVENT_BUFFER_LEN: usize = core::mem::size_of::<VirtioInputEvent>();
const VIRTQ_DESC_F_WRITE: u16 = 1 << 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtioPciCap {
    cap_vndr: u8,
    cap_next: u8,
    cap_len: u8,
    cfg_type: u8,
    bar: u8,
    id: u8,
    padding: [u8; 2],
    offset: u32,
    length: u32,
}

#[repr(C)]
struct VirtioPciNotifyCap {
    cap: VirtioPciCap,
    notify_off_multiplier: u32,
}

#[repr(C)]
struct VirtioPciCommonCfg {
    device_feature_select: u32,
    device_feature: u32,
    driver_feature_select: u32,
    driver_feature: u32,
    msix_config: u16,
    num_queues: u16,
    device_status: u8,
    config_generation: u8,
    queue_select: u16,
    queue_size: u16,
    queue_msix_vector: u16,
    queue_enable: u16,
    queue_notify_off: u16,
    queue_desc: u64,
    queue_avail: u64,
    queue_used: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; EVENT_QUEUE_SIZE],
    used_event: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; EVENT_QUEUE_SIZE],
    avail_event: u16,
}

#[repr(align(4096))]
struct QueueStorage {
    desc: [VirtqDesc; EVENT_QUEUE_SIZE],
    avail: VirtqAvail,
    used_padding: [u8; 4096],
    used: VirtqUsed,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtioInputEvent {
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

static mut EVENT_QUEUE_STORAGE: QueueStorage = QueueStorage {
    desc: [VirtqDesc {
        addr: 0,
        len: 0,
        flags: 0,
        next: 0,
    }; EVENT_QUEUE_SIZE],
    avail: VirtqAvail {
        flags: 0,
        idx: 0,
        ring: [0; EVENT_QUEUE_SIZE],
        used_event: 0,
    },
    used_padding: [0; 4096],
    used: VirtqUsed {
        flags: 0,
        idx: 0,
        ring: [VirtqUsedElem { id: 0, len: 0 }; EVENT_QUEUE_SIZE],
        avail_event: 0,
    },
};

static mut EVENT_BUFFERS: [[MaybeUninit<VirtioInputEvent>; 1]; EVENT_QUEUE_SIZE] =
    [[MaybeUninit::uninit(); 1]; EVENT_QUEUE_SIZE];
static EVENT_QUEUE_LAST_USED: AtomicU16 = AtomicU16::new(0);

static DEVICE: Mutex<Option<InputDevice>> = Mutex::new(None);

pub fn probe() {
    let mut guard = DEVICE.lock();
    if guard.is_some() {
        return;
    }

    let Some(address) = pci::find_device(VIRTIO_VENDOR_ID, device_id::MODERN_INPUT) else {
        serial::write_line("virtio-input: device not present");
        return;
    };

    if !mmio_transport_enabled() {
        serial::write_fmt(format_args!(
            "virtio-input: modern device @ {} detected; MMIO transport deferred\r\n",
            address
        ));
        return;
    }

    match unsafe { init_device(address) } {
        Ok(device) => {
            serial::write_fmt(format_args!(
                "virtio-input: modern device @ {} initialised\r\n",
                address
            ));
            device.notify_queue(0);
            *guard = Some(device);
        }
        Err(err) => {
            serial::write_fmt(format_args!("virtio-input init failed: {}\r\n", err));
        }
    }
}

pub fn poll() -> Vec<VirtioInputEvent> {
    let mut guard = DEVICE.lock();
    let Some(device) = guard.as_mut() else {
        return Vec::new();
    };
    unsafe { device.poll_events() }
}

struct InputDevice {
    common_cfg: *mut VirtioPciCommonCfg,
    notify_base: *mut u8,
    notify_off_multiplier: u32,
    isr_status: *mut u8,
    queue_notify_offset: u16,
}

unsafe impl Send for InputDevice {}
unsafe impl Sync for InputDevice {}

unsafe fn init_device(address: PciAddress) -> Result<InputDevice, &'static str> {
    let caps = VirtioCapabilitySet::discover(address)?;
    let common = caps
        .common_cfg
        .ok_or("virtio-input: missing common config")?;
    let notify = caps.notify.ok_or("virtio-input: missing notify config")?;
    let isr = caps.isr.ok_or("virtio-input: missing isr config")?;

    let common_cfg = common.ptr::<VirtioPciCommonCfg>(address)?;
    let notify_base = notify.ptr::<u8>(address)?;
    let isr_status = isr.ptr::<u8>(address)?;
    let notify_off_multiplier = notify.notify_off_multiplier;

    reset_device(common_cfg);
    acknowledge_device(common_cfg);
    driver_ready(common_cfg);

    let features = read_device_features(common_cfg);
    let accepted = features & FEATURE_VERSION_1;
    if accepted & FEATURE_VERSION_1 == 0 {
        return Err("virtio-input: device lacks VERSION_1 support");
    }
    write_driver_features(common_cfg, accepted);

    set_status(
        common_cfg,
        DEVICE_STATUS_ACKNOWLEDGE | DEVICE_STATUS_DRIVER | DEVICE_STATUS_FEATURES_OK,
    );
    if read_status(common_cfg) & DEVICE_STATUS_FEATURES_OK == 0 {
        return Err("virtio-input: device rejected negotiated features");
    }

    let queue_notify_offset = setup_event_queue(common_cfg)?;

    set_status(
        common_cfg,
        DEVICE_STATUS_ACKNOWLEDGE
            | DEVICE_STATUS_DRIVER
            | DEVICE_STATUS_FEATURES_OK
            | DEVICE_STATUS_DRIVER_OK,
    );

    Ok(InputDevice {
        common_cfg,
        notify_base,
        notify_off_multiplier,
        isr_status,
        queue_notify_offset,
    })
}

impl InputDevice {
    #[allow(static_mut_refs)]
    unsafe fn poll_events(&mut self) -> Vec<VirtioInputEvent> {
        let used = &EVENT_QUEUE_STORAGE.used;
        let mut last = EVENT_QUEUE_LAST_USED.load(Ordering::Acquire);
        let current = core::ptr::read_volatile(addr_of!((*used).idx));
        if last == current {
            return Vec::new();
        }

        let mut events = Vec::new();
        while last != current {
            let slot = (last % EVENT_QUEUE_SIZE as u16) as usize;
            let elem = core::ptr::read_volatile(addr_of!((*used).ring[slot]));
            let desc_idx = elem.id as usize;
            let buffer = &EVENT_BUFFERS[desc_idx][0];
            events.push(buffer.assume_init_read());
            recycle_descriptor(desc_idx);
            last = last.wrapping_add(1);
        }
        EVENT_QUEUE_LAST_USED.store(last, Ordering::Release);

        // acknowledge interrupt
        let _ = core::ptr::read_volatile(self.isr_status);

        if !events.is_empty() {
            self.notify_queue(0);
        }

        events
    }

    fn notify_queue(&self, queue: u16) {
        let offset = self.queue_notify_offset as usize * self.notify_off_multiplier as usize;
        let ptr = unsafe { self.notify_base.add(offset) as *mut u16 };
        unsafe {
            core::ptr::write_volatile(ptr, queue);
        }
    }
}

unsafe fn reset_device(common_cfg: *mut VirtioPciCommonCfg) {
    set_status(common_cfg, 0);
}

unsafe fn acknowledge_device(common_cfg: *mut VirtioPciCommonCfg) {
    set_status(common_cfg, DEVICE_STATUS_ACKNOWLEDGE);
}

unsafe fn driver_ready(common_cfg: *mut VirtioPciCommonCfg) {
    set_status(common_cfg, DEVICE_STATUS_ACKNOWLEDGE | DEVICE_STATUS_DRIVER);
}

unsafe fn setup_event_queue(common_cfg: *mut VirtioPciCommonCfg) -> Result<u16, &'static str> {
    select_queue(common_cfg, 0);
    let reported = core::ptr::read_volatile(addr_of!((*common_cfg).queue_size));
    if reported == 0 {
        return Err("virtio-input: queue size reported as zero");
    }
    if reported < EVENT_QUEUE_SIZE as u16 {
        return Err("virtio-input: queue smaller than expected");
    }

    initialise_event_storage()?;

    core::ptr::write_volatile(addr_of_mut!((*common_cfg).queue_msix_vector), 0xFFFF);
    core::ptr::write_volatile(
        addr_of_mut!((*common_cfg).queue_size),
        EVENT_QUEUE_SIZE as u16,
    );
    core::ptr::write_volatile(
        addr_of_mut!((*common_cfg).queue_desc),
        dma_addr(
            addr_of!(EVENT_QUEUE_STORAGE.desc),
            "virtio-input desc table",
        )?,
    );
    core::ptr::write_volatile(
        addr_of_mut!((*common_cfg).queue_avail),
        dma_addr(
            addr_of!(EVENT_QUEUE_STORAGE.avail),
            "virtio-input avail ring",
        )?,
    );
    core::ptr::write_volatile(
        addr_of_mut!((*common_cfg).queue_used),
        dma_addr(addr_of!(EVENT_QUEUE_STORAGE.used), "virtio-input used ring")?,
    );
    core::ptr::write_volatile(addr_of_mut!((*common_cfg).queue_enable), 1);

    let notify_off = core::ptr::read_volatile(addr_of!((*common_cfg).queue_notify_off));
    Ok(notify_off)
}

#[allow(static_mut_refs)]
unsafe fn initialise_event_storage() -> Result<(), &'static str> {
    let storage = &mut EVENT_QUEUE_STORAGE;
    for (index, desc) in storage.desc.iter_mut().enumerate().take(EVENT_QUEUE_SIZE) {
        desc.addr = dma_addr(
            addr_of!(EVENT_BUFFERS[index][0]),
            "virtio-input event buffer",
        )?;
        desc.len = EVENT_BUFFER_LEN as u32;
        desc.flags = VIRTQ_DESC_F_WRITE;
        desc.next = 0;
        storage.avail.ring[index] = index as u16;
    }
    storage.avail.idx = EVENT_QUEUE_SIZE as u16;
    EVENT_QUEUE_LAST_USED.store(0, Ordering::Relaxed);
    Ok(())
}

#[allow(static_mut_refs)]
unsafe fn recycle_descriptor(index: usize) {
    let avail = &mut EVENT_QUEUE_STORAGE.avail;
    let slot = (avail.idx % EVENT_QUEUE_SIZE as u16) as usize;
    avail.ring[slot] = index as u16;
    fence(Ordering::Release);
    avail.idx = avail.idx.wrapping_add(1);
}

unsafe fn set_status(common_cfg: *mut VirtioPciCommonCfg, status: u8) {
    core::ptr::write_volatile(addr_of_mut!((*common_cfg).device_status), status);
}

unsafe fn read_status(common_cfg: *mut VirtioPciCommonCfg) -> u8 {
    core::ptr::read_volatile(addr_of!((*common_cfg).device_status))
}

unsafe fn select_queue(common_cfg: *mut VirtioPciCommonCfg, queue: u16) {
    core::ptr::write_volatile(addr_of_mut!((*common_cfg).queue_select), queue);
}

unsafe fn read_device_features(common_cfg: *mut VirtioPciCommonCfg) -> u64 {
    core::ptr::write_volatile(addr_of_mut!((*common_cfg).device_feature_select), 0);
    fence(Ordering::SeqCst);
    let low = core::ptr::read_volatile(addr_of!((*common_cfg).device_feature)) as u64;
    core::ptr::write_volatile(addr_of_mut!((*common_cfg).device_feature_select), 1);
    fence(Ordering::SeqCst);
    let high = core::ptr::read_volatile(addr_of!((*common_cfg).device_feature)) as u64;
    (high << 32) | low
}

unsafe fn write_driver_features(common_cfg: *mut VirtioPciCommonCfg, features: u64) {
    core::ptr::write_volatile(addr_of_mut!((*common_cfg).driver_feature_select), 0);
    core::ptr::write_volatile(addr_of_mut!((*common_cfg).driver_feature), features as u32);
    core::ptr::write_volatile(addr_of_mut!((*common_cfg).driver_feature_select), 1);
    core::ptr::write_volatile(
        addr_of_mut!((*common_cfg).driver_feature),
        (features >> 32) as u32,
    );
}

fn dma_addr<T>(ptr: *const T, label: &'static str) -> Result<u64, &'static str> {
    memory::virt_to_phys(ptr).ok_or(label)
}

struct VirtioCapabilitySet {
    common_cfg: Option<VirtioCapability>,
    notify: Option<VirtioNotifyCapability>,
    isr: Option<VirtioCapability>,
    _device_cfg: Option<VirtioCapability>,
}

impl VirtioCapabilitySet {
    fn discover(address: PciAddress) -> Result<Self, &'static str> {
        if address.read_u16(0x06) & (1 << 4) == 0 {
            return Err("virtio-input: PCI capabilities absent");
        }
        let mut ptr = address.read_u8(0x34);
        let mut result = VirtioCapabilitySet {
            common_cfg: None,
            notify: None,
            isr: None,
            _device_cfg: None,
        };
        let mut guard = 0;
        while ptr != 0 {
            guard += 1;
            if guard > 64 {
                return Err("virtio-input: capability loop");
            }
            let header = read_cap_header(address, ptr);
            if header.cap_vndr == CAP_ID_VENDOR_SPECIFIC {
                match header.cfg_type {
                    CFG_TYPE_COMMON => result.common_cfg = Some(VirtioCapability { header }),
                    CFG_TYPE_NOTIFY => {
                        let notify = read_notify_cap(address, ptr);
                        result.notify = Some(notify);
                    }
                    CFG_TYPE_ISR => result.isr = Some(VirtioCapability { header }),
                    CFG_TYPE_DEVICE => result._device_cfg = Some(VirtioCapability { header }),
                    _ => {}
                }
            }
            ptr = header.cap_next;
        }
        Ok(result)
    }
}

#[derive(Clone, Copy)]
struct VirtioCapability {
    header: VirtioPciCap,
}

impl VirtioCapability {
    unsafe fn ptr<T>(&self, address: PciAddress) -> Result<*mut T, &'static str> {
        let base = read_bar(address, self.header.bar)?;
        Ok((base + self.header.offset as u64) as *mut T)
    }
}

struct VirtioNotifyCapability {
    base: VirtioCapability,
    notify_off_multiplier: u32,
}

impl VirtioNotifyCapability {
    unsafe fn ptr<T>(&self, address: PciAddress) -> Result<*mut T, &'static str> {
        self.base.ptr(address)
    }
}

fn read_cap_header(address: PciAddress, offset: u8) -> VirtioPciCap {
    let mut raw = [0u8; size_of::<VirtioPciCap>()];
    for (idx, byte) in raw.iter_mut().enumerate() {
        *byte = address.read_u8(offset.wrapping_add(idx as u8));
    }
    unsafe { core::ptr::read_unaligned(raw.as_ptr() as *const VirtioPciCap) }
}

fn read_notify_cap(address: PciAddress, offset: u8) -> VirtioNotifyCapability {
    let mut raw = [0u8; size_of::<VirtioPciNotifyCap>()];
    for (idx, byte) in raw.iter_mut().enumerate() {
        *byte = address.read_u8(offset.wrapping_add(idx as u8));
    }
    let cap = unsafe { core::ptr::read_unaligned(raw.as_ptr() as *const VirtioPciNotifyCap) };
    VirtioNotifyCapability {
        base: VirtioCapability { header: cap.cap },
        notify_off_multiplier: cap.notify_off_multiplier,
    }
}

unsafe fn read_bar(address: PciAddress, bar_index: u8) -> Result<u64, &'static str> {
    if bar_index >= 6 {
        return Err("virtio-input: invalid BAR index");
    }
    let offset = 0x10 + bar_index * 4;
    let low = address.read_u32(offset);
    if low & 0x1 != 0 {
        return Err("virtio-input: I/O BAR unsupported");
    }
    let ty = (low >> 1) & 0x3;
    let mut base = (low & !0xFu32) as u64;
    if ty == 0x2 {
        let high = address.read_u32(offset + 4) as u64;
        base |= high << 32;
    }
    Ok(base)
}

pub fn device_present() -> bool {
    DEVICE.lock().is_some()
}

fn mmio_transport_enabled() -> bool {
    false
}
