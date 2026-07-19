use alloc::vec::Vec;
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
}

#[derive(Debug)]
pub struct PciFunction {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bars: Vec<PciBar>,
}

#[derive(Clone, Copy, Debug)]
pub struct PciMassStorageController {
    pub address: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub subclass: u8,
    pub prog_if: u8,
}

/// Reads exactly one already-configured AHCI function. This is deliberately
/// not a bus scan: storage authority must never follow the first controller
/// that happens to advertise the mass-storage class.
pub fn exact_ahci_controller(
    address: PciAddress,
) -> Result<PciMassStorageController, &'static str> {
    if address.device >= 32 || address.function >= 8 {
        return Err("pci_exact_ahci_address_invalid");
    }
    let vendor_id = read_vendor(&address);
    if vendor_id == 0xffff {
        return Err("pci_exact_ahci_absent");
    }
    let class = address.read_u8(0x0b);
    let subclass = address.read_u8(0x0a);
    let prog_if = address.read_u8(0x09);
    if class != 0x01 || subclass != 0x06 || prog_if != 0x01 {
        return Err("pci_exact_ahci_class_mismatch");
    }
    Ok(PciMassStorageController {
        address,
        vendor_id,
        device_id: read_device_id(&address),
        subclass,
        prog_if,
    })
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
    pub const fn new(bus: u8, device: u8, function: u8) -> Self {
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

    pub fn write_u32(&self, offset: u8, value: u32) {
        pci_config_write_u32(self.bus, self.device, self.function, offset, value);
    }
}

pub fn enable_bus_master(address: PciAddress) {
    let mut command = (address.read_u32(0x04) & 0xFFFF) as u16;
    command |= 0x1 | 0x2 | 0x4; // I/O space, memory space, bus master
    address.write_u16(0x04, command);
}

pub fn disable_bus_master(address: PciAddress) {
    let mut command = (address.read_u32(0x04) & 0xFFFF) as u16;
    if command == 0xFFFF {
        return;
    }
    command &= !0x4;
    command |= 1 << 10; // interrupt disable
    address.write_u16(0x04, command);
}

pub fn quiesce_function(address: PciAddress) {
    let mut command = (address.read_u32(0x04) & 0xFFFF) as u16;
    if command == 0xFFFF {
        return;
    }
    command &= !(0x1 | 0x2 | 0x4);
    command |= 1 << 10; // interrupt disable
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
            })
        }
    } else {
        let bar_type = (low >> 1) & 0x3;
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
                    })
                }
            }
            _ => None,
        }
    };

    address.write_u16(0x04, command);
    result
}

pub fn read_interrupt_line(address: PciAddress) -> u8 {
    address.read_u8(0x3c)
}

pub fn read_interrupt_pin(address: PciAddress) -> u8 {
    address.read_u8(0x3d)
}

pub fn enumerate_functions() -> Vec<PciFunction> {
    let mut functions = Vec::new();
    for bus in 0..=255 {
        for device in 0..32 {
            let function_zero = PciAddress::new(bus, device, 0);
            if read_vendor(&function_zero) == 0xffff {
                continue;
            }
            let function_count = if has_multiple_functions(&function_zero) {
                8
            } else {
                1
            };
            for function in 0..function_count {
                let address = PciAddress::new(bus, device, function);
                let vendor_id = read_vendor(&address);
                if vendor_id == 0xffff {
                    continue;
                }

                let header_type = address.read_u8(0x0e) & 0x7f;
                let bar_count = match header_type {
                    0x00 => 6,
                    0x01 => 2,
                    0x02 => 1,
                    _ => 0,
                };
                let mut bars = Vec::new();
                let mut index = 0;
                while index < bar_count {
                    if let Some(bar) = read_bar_info(address, index) {
                        let slots = if bar.kind == PciBarKind::Memory64 {
                            2
                        } else {
                            1
                        };
                        bars.push(bar);
                        index += slots;
                    } else {
                        index += 1;
                    }
                }

                functions.push(PciFunction {
                    bus,
                    device,
                    function,
                    vendor_id,
                    device_id: read_device_id(&address),
                    class: address.read_u8(0x0b),
                    subclass: address.read_u8(0x0a),
                    prog_if: address.read_u8(0x09),
                    interrupt_line: read_interrupt_line(address),
                    interrupt_pin: read_interrupt_pin(address),
                    bars,
                });
            }
        }
    }
    functions
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

pub fn find_by_class(class: u8, subclass: u8, prog_if: u8) -> Option<PciAddress> {
    for bus in 0..=255 {
        for dev in 0..32 {
            for func in 0..8 {
                let addr = PciAddress::new(bus, dev, func);
                if read_vendor(&addr) == 0xFFFF {
                    if func == 0 {
                        break;
                    }
                    continue;
                }
                if addr.read_u8(0x0B) == class
                    && addr.read_u8(0x0A) == subclass
                    && addr.read_u8(0x09) == prog_if
                {
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

pub fn find_mass_storage_controller() -> Option<PciMassStorageController> {
    for bus in 0..=255 {
        for dev in 0..32 {
            for func in 0..8 {
                let addr = PciAddress::new(bus, dev, func);
                let vendor_id = read_vendor(&addr);
                if vendor_id == 0xFFFF {
                    if func == 0 {
                        break;
                    }
                    continue;
                }
                if addr.read_u8(0x0B) == 0x01 {
                    return Some(PciMassStorageController {
                        address: addr,
                        vendor_id,
                        device_id: read_device_id(&addr),
                        subclass: addr.read_u8(0x0A),
                        prog_if: addr.read_u8(0x09),
                    });
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
