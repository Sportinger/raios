use core::fmt;
use spin::Mutex;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

static PCI_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl fmt::Display for PciAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}:{:02x}.{}", self.bus, self.device, self.function)
    }
}

impl PciAddress {
    pub fn new(bus: u8, device: u8, function: u8) -> Self {
        Self { bus, device, function }
    }

    pub fn read_u32(&self, offset: u8) -> u32 {
        pci_config_read_u32(self.bus, self.device, self.function, offset)
    }

    pub fn write_u16(&self, offset: u8, value: u16) {
        pci_config_write_u16(self.bus, self.device, self.function, offset, value);
    }

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
