use core::fmt;
use spin::Mutex;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

static PCI_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy, Debug)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PciBarKind {
    Io,
    Memory32,
    Memory64,
}

#[derive(Clone, Copy, Debug)]
pub struct PciBar {
    pub index: u8,
    pub kind: PciBarKind,
    pub base: u64,
    pub size: u64,
    pub prefetchable: bool,
}

impl PciBar {
    pub fn is_memory(&self) -> bool {
        self.kind != PciBarKind::Io
    }
}

impl fmt::Display for PciAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}:{:02x}.{}", self.bus, self.device, self.function)
    }
}

impl PciAddress {
    pub fn new(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus,
            device,
            function,
        }
    }

    pub fn read_u32(&self, offset: u8) -> u32 {
        pci_config_read_u32(self.bus, self.device, self.function, offset)
    }

    pub fn read_u16(&self, offset: u8) -> u16 {
        let value = self.read_u32(offset & !0x3);
        let shift = ((offset & 0x2) as u32) * 8;
        ((value >> shift) & 0xFFFF) as u16
    }

    pub fn read_u8(&self, offset: u8) -> u8 {
        let value = self.read_u32(offset & !0x3);
        let shift = ((offset & 0x3) as u32) * 8;
        ((value >> shift) & 0xFF) as u8
    }

    pub fn write_u16(&self, offset: u8, value: u16) {
        pci_config_write_u16(self.bus, self.device, self.function, offset, value);
    }

    pub fn write_u8(&self, offset: u8, value: u8) {
        let aligned = offset & !0x3;
        let shift = ((offset & 0x3) as u32) * 8;
        let mask = !(0xFFu32 << shift);

        let _guard = PCI_LOCK.lock();
        let address = config_address(self.bus, self.device, self.function, aligned);
        unsafe {
            outl(CONFIG_ADDRESS, address);
            let mut current = inl(CONFIG_DATA);
            current = (current & mask) | ((value as u32) << shift);
            outl(CONFIG_ADDRESS, address);
            outl(CONFIG_DATA, current);
        }
    }

    pub fn write_u32(&self, offset: u8, value: u32) {
        pci_config_write_u32(self.bus, self.device, self.function, offset, value);
    }
}

pub fn enable_bus_master(address: PciAddress) {
    let mut command = (address.read_u32(0x04) & 0xFFFF) as u16;
    command |= 0x1 | 0x2 | 0x4; // I/O space, memory space, bus master
    address.write_u16(0x04, command);
}

pub fn read_bar_info(address: PciAddress, index: u8) -> Option<PciBar> {
    if index >= 6 {
        return None;
    }

    let offset = 0x10 + index * 4;
    let low = address.read_u32(offset);
    if low == 0 {
        return None;
    }

    let command = address.read_u16(0x04);
    address.write_u16(0x04, command & !0x3);

    let result = if low & 0x1 != 0 {
        address.write_u32(offset, u32::MAX);
        let mask = address.read_u32(offset) & !0x3;
        address.write_u32(offset, low);

        let size = (!mask).wrapping_add(1) as u64;
        let base = (low & !0x3) as u64;
        if base == 0 || size == 0 {
            None
        } else {
            Some(PciBar {
                index,
                kind: PciBarKind::Io,
                base,
                size,
                prefetchable: false,
            })
        }
    } else {
        let bar_type = (low >> 1) & 0x3;
        let prefetchable = low & (1 << 3) != 0;
        match bar_type {
            0x0 => {
                address.write_u32(offset, u32::MAX);
                let mask = address.read_u32(offset) & !0xF;
                address.write_u32(offset, low);

                let size = (!mask).wrapping_add(1) as u64;
                let base = (low & !0xF) as u64;
                if base == 0 || size == 0 {
                    None
                } else {
                    Some(PciBar {
                        index,
                        kind: PciBarKind::Memory32,
                        base,
                        size,
                        prefetchable,
                    })
                }
            }
            0x2 if index < 5 => {
                let high_offset = offset + 4;
                let high = address.read_u32(high_offset);
                address.write_u32(offset, u32::MAX);
                address.write_u32(high_offset, u32::MAX);
                let sized_low = address.read_u32(offset);
                let sized_high = address.read_u32(high_offset);
                address.write_u32(high_offset, high);
                address.write_u32(offset, low);

                let mask = ((sized_high as u64) << 32) | ((sized_low & !0xF) as u64);
                let size = (!mask).wrapping_add(1);
                let base = ((high as u64) << 32) | ((low & !0xF) as u64);
                if base == 0 || size == 0 {
                    None
                } else {
                    Some(PciBar {
                        index,
                        kind: PciBarKind::Memory64,
                        base,
                        size,
                        prefetchable,
                    })
                }
            }
            _ => None,
        }
    };

    address.write_u16(0x04, command);
    result
}

pub fn find_device(vendor: u16, device: u16) -> Option<PciAddress> {
    for bus in 0..=255 {
        for dev in 0..32 {
            for func in 0..8 {
                let addr = PciAddress::new(bus, dev, func);
                if read_vendor(&addr) != vendor {
                    continue;
                }
                if read_device_id(&addr) == device {
                    return Some(addr);
                }
                if func == 0 && !has_multiple_functions(&addr) {
                    break;
                }
            }
        }
    }
    None
}

fn read_vendor(addr: &PciAddress) -> u16 {
    (addr.read_u32(0) & 0xFFFF) as u16
}

fn read_device_id(addr: &PciAddress) -> u16 {
    ((addr.read_u32(0) >> 16) & 0xFFFF) as u16
}

fn has_multiple_functions(addr: &PciAddress) -> bool {
    (addr.read_u32(0x0C) & (1 << 23)) != 0
}

fn pci_config_read_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let _guard = PCI_LOCK.lock();
    let address = config_address(bus, device, function, offset);
    unsafe {
        outl(CONFIG_ADDRESS, address);
        inl(CONFIG_DATA)
    }
}

fn pci_config_write_u16(bus: u8, device: u8, function: u8, offset: u8, value: u16) {
    let aligned = offset & !0x3;
    let shift = ((offset & 0x2) as u32) * 8;
    let mask = !(0xFFFFu32 << shift);

    let _guard = PCI_LOCK.lock();
    let address = config_address(bus, device, function, aligned);
    unsafe {
        outl(CONFIG_ADDRESS, address);
        let mut current = inl(CONFIG_DATA);
        current = (current & mask) | ((value as u32) << shift);
        outl(CONFIG_ADDRESS, address);
        outl(CONFIG_DATA, current);
    }
}

fn pci_config_write_u32(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let _guard = PCI_LOCK.lock();
    let address = config_address(bus, device, function, offset);
    unsafe {
        outl(CONFIG_ADDRESS, address);
        outl(CONFIG_DATA, value);
    }
}

fn config_address(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    1u32 << 31
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | (offset as u32 & 0xFC)
}

unsafe fn outl(port: u16, value: u32) {
    core::arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, preserves_flags));
}

unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    core::arch::asm!("in eax, dx", in("dx") port, out("eax") value, options(nomem, preserves_flags));
    value
}
