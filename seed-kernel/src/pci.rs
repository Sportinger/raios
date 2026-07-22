use alloc::vec::Vec;
use core::fmt;
use spin::Mutex;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

static PCI_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PciCommandWriteResult {
    Written { readback: u16 },
    DeviceUnavailable,
    CommandChanged { observed: u16 },
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
    pub revision: u8,
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

    /// Writes only the PCI Command word after rechecking the function and the
    /// expected command under the config lock. The adjacent Status W1C word is
    /// never read-modify-written.
    pub fn write_command_u16_checked(
        &self,
        expected_vendor_device: u32,
        expected_command: u16,
        value: u16,
    ) -> PciCommandWriteResult {
        pci_config_write_command_u16_checked(
            self.bus,
            self.device,
            self.function,
            expected_vendor_device,
            expected_command,
            value,
        )
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
    let mut backend = AddressConfigBackend { address };
    read_bar_info_with_backend(&mut backend, index)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BarWindow {
    count: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BarClassification {
    Empty,
    Io,
    Memory32,
    Memory64,
    Unsupported,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BarInvalidReason {
    OutsideHeaderWindow,
    TruncatedMemory64,
    Follower,
    UnsupportedEncoding,
    MismatchedSizingEncoding,
    NonCanonicalMask,
    MisalignedBase,
    AddressRangeOverflow,
}

#[derive(Clone, Copy, Debug)]
enum BarDisposition {
    Usable(PciBar),
    Empty,
    Unassigned,
    Unavailable,
    Invalid(BarInvalidReason),
}

struct PciBarProbe {
    consumed_slots: u8,
    disposition: BarDisposition,
}

trait PciConfigBackend {
    fn read_u32(&mut self, offset: u8) -> u32;
    fn read_u16(&mut self, offset: u8) -> u16;
    fn write_u32(&mut self, offset: u8, value: u32);
    fn write_u16(&mut self, offset: u8, value: u16);
}

struct AddressConfigBackend {
    address: PciAddress,
}

impl PciConfigBackend for AddressConfigBackend {
    fn read_u32(&mut self, offset: u8) -> u32 {
        self.address.read_u32(offset)
    }

    fn read_u16(&mut self, offset: u8) -> u16 {
        self.address.read_u16(offset)
    }

    fn write_u32(&mut self, offset: u8, value: u32) {
        self.address.write_u32(offset, value);
    }

    fn write_u16(&mut self, offset: u8, value: u16) {
        self.address.write_u16(offset, value);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PciCaptureMetadata {
    identity: u32,
    command: u16,
    class_revision: u32,
    header_type: u8,
    irq: u16,
}

impl PciCaptureMetadata {
    fn bar_window(self) -> Result<BarWindow, &'static str> {
        let count = match self.header_type & 0x7f {
            0x00 => 6,
            0x01 => 2,
            0x02 => 1,
            _ => return Err("pci_capture_header_invalid"),
        };
        Ok(BarWindow { count })
    }

    fn into_function(self, address: PciAddress, bars: Vec<PciBar>) -> PciFunction {
        PciFunction {
            bus: address.bus,
            device: address.device,
            function: address.function,
            vendor_id: self.identity as u16,
            device_id: (self.identity >> 16) as u16,
            class: (self.class_revision >> 24) as u8,
            subclass: (self.class_revision >> 16) as u8,
            prog_if: (self.class_revision >> 8) as u8,
            revision: self.class_revision as u8,
            interrupt_line: self.irq as u8,
            interrupt_pin: (self.irq >> 8) as u8,
            bars,
        }
    }
}

struct CapturedPciFunction {
    address: PciAddress,
    metadata: PciCaptureMetadata,
    bars: Vec<PciBar>,
}

trait PciCaptureConfigBackend {
    fn read_u32(&mut self, address: PciAddress, offset: u8) -> u32;
    fn read_u16(&mut self, address: PciAddress, offset: u8) -> u16;
    fn write_u32(&mut self, address: PciAddress, offset: u8, value: u32);
    fn write_u16(&mut self, address: PciAddress, offset: u8, value: u16);
}

struct HardwareCaptureConfigBackend;

impl PciCaptureConfigBackend for HardwareCaptureConfigBackend {
    fn read_u32(&mut self, address: PciAddress, offset: u8) -> u32 {
        address.read_u32(offset)
    }

    fn read_u16(&mut self, address: PciAddress, offset: u8) -> u16 {
        address.read_u16(offset)
    }

    fn write_u32(&mut self, address: PciAddress, offset: u8, value: u32) {
        address.write_u32(offset, value);
    }

    fn write_u16(&mut self, address: PciAddress, offset: u8, value: u16) {
        address.write_u16(offset, value);
    }
}

struct CaptureFunctionBackend<'a, B> {
    backend: &'a mut B,
    address: PciAddress,
}

impl<B: PciCaptureConfigBackend> PciConfigBackend for CaptureFunctionBackend<'_, B> {
    fn read_u32(&mut self, offset: u8) -> u32 {
        self.backend.read_u32(self.address, offset)
    }

    fn read_u16(&mut self, offset: u8) -> u16 {
        self.backend.read_u16(self.address, offset)
    }

    fn write_u32(&mut self, offset: u8, value: u32) {
        self.backend.write_u32(self.address, offset, value);
    }

    fn write_u16(&mut self, offset: u8, value: u16) {
        self.backend.write_u16(self.address, offset, value);
    }
}

fn read_bar_info_with_backend(backend: &mut impl PciConfigBackend, index: u8) -> Option<PciBar> {
    match read_bar_probe_with_backend(backend, index).disposition {
        BarDisposition::Usable(bar) => Some(bar),
        _ => None,
    }
}

fn bar_window_with_backend(backend: &mut impl PciConfigBackend) -> Option<(u32, BarWindow)> {
    let identity = backend.read_u32(0x00);
    if identity as u16 == u16::MAX {
        return None;
    }

    let header = backend.read_u32(0x0c);
    if header == u32::MAX {
        return None;
    }
    let count = match ((header >> 16) as u8) & 0x7f {
        0x00 => 6,
        0x01 => 2,
        0x02 => 1,
        _ => return None,
    };
    Some((identity, BarWindow { count }))
}

fn classify_bar_slot(low: u32) -> (BarClassification, u8) {
    if low == u32::MAX {
        (BarClassification::Unavailable, 1)
    } else if low == 0 {
        (BarClassification::Empty, 1)
    } else if low & 0x1 != 0 {
        if low & 0x2 == 0 {
            (BarClassification::Io, 1)
        } else {
            (BarClassification::Unsupported, 1)
        }
    } else {
        match (low >> 1) & 0x3 {
            0x0 => (BarClassification::Memory32, 1),
            0x2 => (BarClassification::Memory64, 2),
            _ => (BarClassification::Unsupported, 1),
        }
    }
}

fn probe_outcome(consumed_slots: u8, disposition: BarDisposition) -> PciBarProbe {
    PciBarProbe {
        consumed_slots,
        disposition,
    }
}

fn validate_bar_range(
    index: u8,
    kind: PciBarKind,
    base: u64,
    mask: u64,
    width_max: u64,
) -> BarDisposition {
    let size = ((!mask) & width_max).wrapping_add(1) & width_max;
    if size == 0 || !size.is_power_of_two() {
        return BarDisposition::Invalid(BarInvalidReason::NonCanonicalMask);
    }
    if base == 0 {
        return BarDisposition::Unassigned;
    }
    let Some(end) = base.checked_add(size - 1) else {
        return BarDisposition::Invalid(BarInvalidReason::AddressRangeOverflow);
    };
    if end > width_max {
        return BarDisposition::Invalid(BarInvalidReason::AddressRangeOverflow);
    }
    if base & (size - 1) != 0 {
        return BarDisposition::Invalid(BarInvalidReason::MisalignedBase);
    }
    BarDisposition::Usable(PciBar {
        index,
        kind,
        base,
        size,
    })
}

fn classify_requested_slot(
    backend: &mut impl PciConfigBackend,
    window: BarWindow,
    index: u8,
) -> Result<(u32, BarClassification, u8), PciBarProbe> {
    if index >= window.count {
        return Err(probe_outcome(
            1,
            BarDisposition::Invalid(BarInvalidReason::OutsideHeaderWindow),
        ));
    }

    let mut cursor = 0;
    while cursor <= index {
        let low = backend.read_u32(0x10 + cursor * 4);
        let (classification, consumed_slots) = classify_bar_slot(low);
        if cursor == index {
            if classification == BarClassification::Memory64 && cursor + 1 >= window.count {
                return Err(probe_outcome(
                    2,
                    BarDisposition::Invalid(BarInvalidReason::TruncatedMemory64),
                ));
            }
            return Ok((low, classification, consumed_slots));
        }

        if classification == BarClassification::Unavailable {
            return Err(probe_outcome(1, BarDisposition::Unavailable));
        }
        if consumed_slots == 2 && cursor + 1 == index {
            return Err(probe_outcome(
                1,
                BarDisposition::Invalid(BarInvalidReason::Follower),
            ));
        }
        cursor = cursor.saturating_add(consumed_slots);
    }
    Err(probe_outcome(
        1,
        BarDisposition::Invalid(BarInvalidReason::Follower),
    ))
}

fn read_bar_probe_in_window(
    backend: &mut impl PciConfigBackend,
    expected_identity: u32,
    expected_window: BarWindow,
    index: u8,
) -> PciBarProbe {
    if bar_window_with_backend(backend) != Some((expected_identity, expected_window)) {
        return probe_outcome(1, BarDisposition::Unavailable);
    }

    let (low, classification, consumed_slots) =
        match classify_requested_slot(backend, expected_window, index) {
            Ok(classification) => classification,
            Err(probe) => return probe,
        };
    match classification {
        BarClassification::Empty => return probe_outcome(consumed_slots, BarDisposition::Empty),
        BarClassification::Unavailable => {
            return probe_outcome(consumed_slots, BarDisposition::Unavailable)
        }
        BarClassification::Unsupported => {
            return probe_outcome(
                consumed_slots,
                BarDisposition::Invalid(BarInvalidReason::UnsupportedEncoding),
            )
        }
        BarClassification::Io | BarClassification::Memory32 | BarClassification::Memory64 => {}
    }

    let offset = 0x10 + index * 4;
    let high = if classification == BarClassification::Memory64 {
        Some(backend.read_u32(offset + 4))
    } else {
        None
    };
    let command = backend.read_u16(0x04);
    if command == u16::MAX {
        return probe_outcome(consumed_slots, BarDisposition::Unavailable);
    }

    backend.write_u16(0x04, command & !0x3);
    backend.write_u32(offset, u32::MAX);
    if high.is_some() {
        backend.write_u32(offset + 4, u32::MAX);
    }
    let sized_low = backend.read_u32(offset);
    let sized_high = high.map(|_| backend.read_u32(offset + 4));
    if let Some(high) = high {
        backend.write_u32(offset + 4, high);
    }
    backend.write_u32(offset, low);
    backend.write_u16(0x04, command);

    if backend.read_u32(0x00) != expected_identity || sized_low == u32::MAX {
        return probe_outcome(consumed_slots, BarDisposition::Unavailable);
    }

    let sizing_encoding_matches = match classification {
        BarClassification::Io => sized_low & 0x3 == low & 0x3,
        BarClassification::Memory32 | BarClassification::Memory64 => sized_low & 0xf == low & 0xf,
        _ => unreachable!(),
    };
    if !sizing_encoding_matches {
        return probe_outcome(
            consumed_slots,
            BarDisposition::Invalid(BarInvalidReason::MismatchedSizingEncoding),
        );
    }

    let disposition = match classification {
        BarClassification::Io => validate_bar_range(
            index,
            PciBarKind::Io,
            (low & !0x3) as u64,
            (sized_low & !0x3) as u64,
            u32::MAX as u64,
        ),
        BarClassification::Memory32 => validate_bar_range(
            index,
            PciBarKind::Memory32,
            (low & !0xf) as u64,
            (sized_low & !0xf) as u64,
            u32::MAX as u64,
        ),
        BarClassification::Memory64 => validate_bar_range(
            index,
            PciBarKind::Memory64,
            ((high.unwrap() as u64) << 32) | (low & !0xf) as u64,
            ((sized_high.unwrap() as u64) << 32) | (sized_low & !0xf) as u64,
            u64::MAX,
        ),
        _ => unreachable!(),
    };
    probe_outcome(consumed_slots, disposition)
}

fn read_bar_probe_with_backend(backend: &mut impl PciConfigBackend, index: u8) -> PciBarProbe {
    let Some((identity, window)) = bar_window_with_backend(backend) else {
        return probe_outcome(1, BarDisposition::Unavailable);
    };
    read_bar_probe_in_window(backend, identity, window, index)
}

fn read_bars_with_backend(backend: &mut impl PciConfigBackend) -> Vec<PciBar> {
    let mut bars = Vec::new();
    let Some((identity, window)) = bar_window_with_backend(backend) else {
        return bars;
    };
    let mut index = 0;
    while index < window.count {
        let probe = read_bar_probe_in_window(backend, identity, window, index);
        index = index.saturating_add(probe.consumed_slots);
        match probe.disposition {
            BarDisposition::Usable(bar) => bars.push(bar),
            BarDisposition::Unavailable => {
                bars.clear();
                return bars;
            }
            _ => {}
        }
    }
    bars
}

fn read_capture_metadata_with_backend(
    backend: &mut impl PciConfigBackend,
) -> Result<PciCaptureMetadata, &'static str> {
    let identity = backend.read_u32(0x00);
    if identity as u16 == u16::MAX || (identity >> 16) as u16 == u16::MAX {
        return Err("pci_capture_identity_unavailable");
    }

    let command = backend.read_u16(0x04);
    let class_revision = backend.read_u32(0x08);
    let header = backend.read_u32(0x0c);
    let irq = backend.read_u16(0x3c);
    if command == u16::MAX || class_revision == u32::MAX || header == u32::MAX || irq == u16::MAX {
        return Err("pci_capture_metadata_unavailable");
    }

    let metadata = PciCaptureMetadata {
        identity,
        command,
        class_revision,
        header_type: (header >> 16) as u8,
        irq,
    };
    metadata.bar_window()?;
    if metadata.irq >> 8 > 4 {
        return Err("pci_capture_irq_invalid");
    }
    Ok(metadata)
}

fn ensure_capture_metadata_unchanged(
    before: PciCaptureMetadata,
    after: PciCaptureMetadata,
) -> Result<(), &'static str> {
    if after.identity != before.identity {
        return Err("pci_capture_identity_changed");
    }
    if after.header_type != before.header_type {
        return Err("pci_capture_header_changed");
    }
    if after != before {
        return Err("pci_capture_metadata_changed");
    }
    Ok(())
}

fn read_bars_for_capture_with_backend(
    backend: &mut impl PciConfigBackend,
    expected_identity: u32,
    window: BarWindow,
) -> Result<Vec<PciBar>, &'static str> {
    let mut bars = Vec::new();
    let mut index = 0;
    while index < window.count {
        let probe = read_bar_probe_in_window(backend, expected_identity, window, index);
        index = index.saturating_add(probe.consumed_slots);
        match probe.disposition {
            BarDisposition::Usable(bar) => bars.push(bar),
            BarDisposition::Empty | BarDisposition::Unassigned => {}
            BarDisposition::Unavailable => return Err("pci_capture_bar_unavailable"),
            BarDisposition::Invalid(_) => return Err("pci_capture_bar_invalid"),
        }
    }
    Ok(bars)
}

fn capture_function_with_backend(
    backend: &mut impl PciConfigBackend,
    address: PciAddress,
    expected_identity: u32,
) -> Result<CapturedPciFunction, &'static str> {
    let before = read_capture_metadata_with_backend(backend)?;
    if before.identity != expected_identity {
        return Err("pci_capture_identity_changed");
    }
    let bars = read_bars_for_capture_with_backend(backend, before.identity, before.bar_window()?)?;
    let after = read_capture_metadata_with_backend(backend)?;
    ensure_capture_metadata_unchanged(before, after)?;
    Ok(CapturedPciFunction {
        address,
        metadata: before,
        bars,
    })
}

fn enumerate_functions_for_capture_with_backend(
    backend: &mut impl PciCaptureConfigBackend,
) -> Result<Vec<PciFunction>, &'static str> {
    let mut captured = Vec::new();
    let mut probed_identities = Vec::new();

    for bus in 0..=255 {
        for device in 0..32 {
            let function_zero = PciAddress::new(bus, device, 0);
            let identity = backend.read_u32(function_zero, 0x00);
            probed_identities.push((function_zero, identity));
            if identity as u16 == u16::MAX {
                continue;
            }

            let function_zero_capture = {
                let mut function_backend = CaptureFunctionBackend {
                    backend: &mut *backend,
                    address: function_zero,
                };
                capture_function_with_backend(&mut function_backend, function_zero, identity)?
            };
            let function_count = if function_zero_capture.metadata.header_type & 0x80 != 0 {
                8
            } else {
                1
            };
            captured.push(function_zero_capture);

            for function in 1..function_count {
                let address = PciAddress::new(bus, device, function);
                let identity = backend.read_u32(address, 0x00);
                probed_identities.push((address, identity));
                if identity as u16 == u16::MAX {
                    continue;
                }
                let capture = {
                    let mut function_backend = CaptureFunctionBackend {
                        backend: &mut *backend,
                        address,
                    };
                    capture_function_with_backend(&mut function_backend, address, identity)?
                };
                captured.push(capture);
            }
        }
    }

    for (address, identity) in probed_identities {
        if backend.read_u32(address, 0x00) != identity {
            return Err("pci_capture_topology_changed");
        }
    }
    for capture in &captured {
        let current = {
            let mut function_backend = CaptureFunctionBackend {
                backend: &mut *backend,
                address: capture.address,
            };
            read_capture_metadata_with_backend(&mut function_backend)?
        };
        ensure_capture_metadata_unchanged(capture.metadata, current)?;
    }

    let mut functions = Vec::new();
    for capture in captured {
        functions.push(
            capture
                .metadata
                .into_function(capture.address, capture.bars),
        );
    }
    Ok(functions)
}

pub fn enumerate_functions_for_capture() -> Result<Vec<PciFunction>, &'static str> {
    let mut backend = HardwareCaptureConfigBackend;
    enumerate_functions_for_capture_with_backend(&mut backend)
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

                let mut backend = AddressConfigBackend { address };
                let bars = read_bars_with_backend(&mut backend);

                functions.push(PciFunction {
                    bus,
                    device,
                    function,
                    vendor_id,
                    device_id: read_device_id(&address),
                    class: address.read_u8(0x0b),
                    subclass: address.read_u8(0x0a),
                    prog_if: address.read_u8(0x09),
                    revision: address.read_u8(0x08),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PciConfigWriteU16Plan {
    address: u32,
    data_port: u16,
    value: u16,
}

fn pci_config_write_u16_plan(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
    value: u16,
) -> Option<PciConfigWriteU16Plan> {
    if offset & 0x1 != 0 {
        return None;
    }

    Some(PciConfigWriteU16Plan {
        address: config_address(bus, device, function, offset),
        data_port: CONFIG_DATA + (offset & 0x2) as u16,
        value,
    })
}

trait PciConfigWriteU16Io {
    unsafe fn write_u32(&mut self, port: u16, value: u32);
    unsafe fn write_u16(&mut self, port: u16, value: u16);
}

struct X86PciConfigIo;

impl PciConfigWriteU16Io for X86PciConfigIo {
    #[inline(always)]
    unsafe fn write_u32(&mut self, port: u16, value: u32) {
        outl(port, value);
    }

    #[inline(always)]
    unsafe fn write_u16(&mut self, port: u16, value: u16) {
        outw(port, value);
    }
}

#[inline(always)]
fn pci_config_write_u16_with_io(
    io: &mut impl PciConfigWriteU16Io,
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
    value: u16,
) -> Option<()> {
    let plan = pci_config_write_u16_plan(bus, device, function, offset, value)?;
    unsafe {
        io.write_u32(CONFIG_ADDRESS, plan.address);
        io.write_u16(plan.data_port, plan.value);
    }
    Some(())
}

#[cfg(pci_bar_transport_proof)]
#[no_mangle]
#[inline(never)]
pub extern "C" fn retained_pci_config_write_u16_transport(value: u16) -> bool {
    let lower = pci_config_write_u16_with_io(&mut X86PciConfigIo, 0, 0, 0, 0x04, value);
    let upper = pci_config_write_u16_with_io(&mut X86PciConfigIo, 0, 0, 0, 0x06, value);
    lower.is_some() && upper.is_some()
}

fn pci_config_write_u16(bus: u8, device: u8, function: u8, offset: u8, value: u16) {
    let _guard = PCI_LOCK.lock();
    pci_config_write_u16_with_io(&mut X86PciConfigIo, bus, device, function, offset, value)
        .expect("PCI u16 configuration writes require an even offset");
}

fn pci_config_write_command_u16_checked(
    bus: u8,
    device: u8,
    function: u8,
    expected_vendor_device: u32,
    expected_command: u16,
    value: u16,
) -> PciCommandWriteResult {
    let _guard = PCI_LOCK.lock();
    unsafe {
        // SAFETY: PCI_LOCK serializes mechanism-one CF8/CFC transactions. All
        // reads, the presence/expected-value checks, the command-only word
        // write, and its readback remain inside this critical section.
        outl(CONFIG_ADDRESS, config_address(bus, device, function, 0x00));
        let vendor_device = inl(CONFIG_DATA);
        if vendor_device == u32::MAX || vendor_device != expected_vendor_device {
            return PciCommandWriteResult::DeviceUnavailable;
        }

        outl(CONFIG_ADDRESS, config_address(bus, device, function, 0x04));
        let command = (inl(CONFIG_DATA) & 0xffff) as u16;
        if command == u16::MAX {
            return PciCommandWriteResult::DeviceUnavailable;
        }
        if command != expected_command {
            return PciCommandWriteResult::CommandChanged { observed: command };
        }

        outw(CONFIG_DATA, value);
        outl(CONFIG_ADDRESS, config_address(bus, device, function, 0x04));
        let readback = (inl(CONFIG_DATA) & 0xffff) as u16;
        if readback == u16::MAX {
            PciCommandWriteResult::DeviceUnavailable
        } else {
            PciCommandWriteResult::Written { readback }
        }
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

unsafe fn outw(port: u16, value: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, preserves_flags));
}

unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    core::arch::asm!("in eax, dx", in("dx") port, out("eax") value, options(nomem, preserves_flags));
    value
}

#[cfg(test)]
mod bar_sizing_tests {
    use super::*;
    use alloc::vec;

    const STATUS_W1C_MASK: u16 = 0xf900;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Write {
        U16(u8, u16),
        U32(u8, u32),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum PortWrite {
        U16(u16, u16),
        U32(u16, u32),
    }

    struct FakePortIo {
        writes: Vec<PortWrite>,
    }

    impl FakePortIo {
        fn new() -> Self {
            Self { writes: Vec::new() }
        }
    }

    impl PciConfigWriteU16Io for FakePortIo {
        unsafe fn write_u32(&mut self, port: u16, value: u32) {
            self.writes.push(PortWrite::U32(port, value));
        }

        unsafe fn write_u16(&mut self, port: u16, value: u16) {
            self.writes.push(PortWrite::U16(port, value));
        }
    }

    struct FakeConfig {
        registers: [u32; 16],
        sizing_masks: [u32; 16],
        sizing_active: [bool; 16],
        read_counts: [usize; 16],
        writes: Vec<Write>,
        identity_after_mutation: Option<u32>,
        identity_after_reads: Option<(usize, u32)>,
        header_after_reads: Option<(usize, u32)>,
        register_after_reads: [Option<(usize, u32)>; 16],
        mutated: bool,
    }

    impl FakeConfig {
        fn new(command_status: u32) -> Self {
            let mut registers = [0; 16];
            registers[0] = 0x5678_1234;
            registers[1] = command_status;
            registers[3] = 0;
            Self {
                registers,
                sizing_masks: [0; 16],
                sizing_active: [false; 16],
                read_counts: [0; 16],
                writes: Vec::new(),
                identity_after_mutation: None,
                identity_after_reads: None,
                header_after_reads: None,
                register_after_reads: [None; 16],
                mutated: false,
            }
        }

        fn set_header_type(&mut self, header_type: u8) {
            self.registers[3] = (header_type as u32) << 16;
        }

        fn set_bar(&mut self, index: u8, value: u32, sizing_mask: u32) {
            let slot = (0x10 / 4 + index) as usize;
            self.registers[slot] = value;
            self.sizing_masks[slot] = sizing_mask;
        }

        fn set_register_after_reads(&mut self, offset: u8, unchanged_reads: usize, value: u32) {
            self.register_after_reads[(offset / 4) as usize] = Some((unchanged_reads, value));
        }

        fn snapshot(&self) -> ([u32; 16], [bool; 16]) {
            (self.registers, self.sizing_active)
        }
    }

    impl PciConfigBackend for FakeConfig {
        fn read_u32(&mut self, offset: u8) -> u32 {
            let slot = (offset / 4) as usize;
            self.read_counts[slot] += 1;
            if offset == 0 {
                if let Some((unchanged_reads, identity)) = self.identity_after_reads {
                    if self.read_counts[slot] > unchanged_reads {
                        return identity;
                    }
                }
            }
            if offset == 0 && self.mutated {
                if let Some(identity) = self.identity_after_mutation {
                    return identity;
                }
            }
            if offset == 0x0c {
                if let Some((unchanged_reads, header)) = self.header_after_reads {
                    if self.read_counts[slot] > unchanged_reads {
                        return header;
                    }
                }
            }
            if let Some((unchanged_reads, value)) = self.register_after_reads[slot] {
                if self.read_counts[slot] > unchanged_reads {
                    return value;
                }
            }
            if offset >= 0x10 && self.sizing_active[slot] {
                self.sizing_masks[slot]
            } else {
                self.registers[slot]
            }
        }

        fn read_u16(&mut self, offset: u8) -> u16 {
            let value = self.read_u32(offset & !0x3);
            ((value >> (((offset & 0x2) as u32) * 8)) & 0xffff) as u16
        }

        fn write_u32(&mut self, offset: u8, value: u32) {
            self.writes.push(Write::U32(offset, value));
            self.mutated = true;
            let slot = (offset / 4) as usize;
            if offset == 0x04 {
                let status = (self.registers[slot] >> 16) as u16;
                let replayed_status = (value >> 16) as u16;
                let status = status & !(replayed_status & STATUS_W1C_MASK);
                self.registers[slot] = ((status as u32) << 16) | (value & 0xffff);
            } else {
                self.registers[slot] = value;
                if offset >= 0x10 {
                    self.sizing_active[slot] = if value == u32::MAX {
                        !self.sizing_active[slot]
                    } else {
                        false
                    };
                }
            }
        }

        fn write_u16(&mut self, offset: u8, value: u16) {
            self.writes.push(Write::U16(offset, value));
            self.mutated = true;
            let slot = (offset / 4) as usize;
            let shift = ((offset & 0x2) as u32) * 8;
            let mask = !(0xffff << shift);
            self.registers[slot] = (self.registers[slot] & mask) | ((value as u32) << shift);
        }
    }

    struct FakeCaptureConfig {
        functions: Vec<(PciAddress, FakeConfig)>,
    }

    impl FakeCaptureConfig {
        fn new(functions: Vec<(PciAddress, FakeConfig)>) -> Self {
            Self { functions }
        }

        fn function_mut(&mut self, address: PciAddress) -> Option<&mut FakeConfig> {
            self.functions
                .iter_mut()
                .find(|(candidate, _)| *candidate == address)
                .map(|(_, config)| config)
        }
    }

    impl PciCaptureConfigBackend for FakeCaptureConfig {
        fn read_u32(&mut self, address: PciAddress, offset: u8) -> u32 {
            self.function_mut(address)
                .map(|c| c.read_u32(offset))
                .unwrap_or(u32::MAX)
        }
        fn read_u16(&mut self, address: PciAddress, offset: u8) -> u16 {
            self.function_mut(address)
                .map(|c| c.read_u16(offset))
                .unwrap_or(u16::MAX)
        }
        fn write_u32(&mut self, address: PciAddress, offset: u8, value: u32) {
            self.function_mut(address).unwrap().write_u32(offset, value);
        }
        fn write_u16(&mut self, address: PciAddress, offset: u8, value: u16) {
            self.function_mut(address).unwrap().write_u16(offset, value);
        }
    }

    fn capture_config(identity: u32, class_revision: u32, header_type: u8, irq: u16) -> FakeConfig {
        let mut config = FakeConfig::new(0xa55a_0003);
        config.registers[0] = identity;
        config.registers[2] = class_revision;
        config.registers[3] = (header_type as u32) << 16;
        config.registers[15] = irq as u32;
        config
    }

    fn capture_single(config: FakeConfig) -> Result<Vec<PciFunction>, &'static str> {
        let mut backend = FakeCaptureConfig::new(vec![(PciAddress::new(0, 0, 0), config)]);
        enumerate_functions_for_capture_with_backend(&mut backend)
    }

    fn assert_bar(bar: Option<PciBar>, kind: PciBarKind, base: u64, size: u64) {
        let bar = bar.expect("BAR must be implemented");
        assert_eq!(bar.kind, kind);
        assert_eq!(bar.base, base);
        assert_eq!(bar.size, size);
    }

    #[test]
    fn capture_enumeration_returns_canonical_complete_functions_and_bars() {
        let no_bars = capture_config(0x1000_1111, 0x0102_0304, 0x00, 0x0105);

        let mut memory32 = capture_config(0x2000_2222, 0x0203_0405, 0x00, 0x0206);
        memory32.set_bar(0, 0xd000_0000, 0xffff_f000);

        let mut memory64 = capture_config(0x3000_3333, 0x0304_0506, 0x00, 0x0307);
        memory64.set_bar(0, 0x3440_0004, 0xffe0_0004);
        memory64.set_bar(1, 0x0000_0012, u32::MAX);
        memory64.set_bar(2, 0xe000_0000, 0xffff_f000);

        let mut unassigned = capture_config(0x4000_4444, 0x0405_0607, 0x00, 0x0408);
        unassigned.set_bar(0, 0x0000_0004, 0xffff_f004);
        unassigned.set_bar(1, 0, u32::MAX);

        let mut backend = FakeCaptureConfig::new(vec![
            (PciAddress::new(0, 2, 0), memory64),
            (PciAddress::new(0, 0, 0), no_bars),
            (PciAddress::new(0, 3, 0), unassigned),
            (PciAddress::new(0, 1, 0), memory32),
        ]);
        let functions = enumerate_functions_for_capture_with_backend(&mut backend).unwrap();

        assert_eq!(
            functions
                .iter()
                .map(|function| (function.bus, function.device, function.function))
                .collect::<Vec<_>>(),
            vec![(0, 0, 0), (0, 1, 0), (0, 2, 0), (0, 3, 0)]
        );
        assert_eq!(functions[0].revision, 0x04);
        assert!(functions[0].bars.is_empty());
        assert_eq!(functions[1].bars.len(), 1);
        assert_eq!(functions[1].bars[0].kind, PciBarKind::Memory32);
        assert_eq!(functions[1].bars[0].base, 0xd000_0000);
        assert_eq!(functions[2].bars.len(), 2);
        assert_eq!(functions[2].bars[0].index, 0);
        assert_eq!(functions[2].bars[0].kind, PciBarKind::Memory64);
        assert_eq!(functions[2].bars[1].index, 2);
        assert_eq!(functions[2].bars[1].kind, PciBarKind::Memory32);
        assert!(functions[3].bars.is_empty());
    }

    #[test]
    fn capture_enumeration_distinguishes_empty_headers_from_probe_rejection() {
        for header_type in [0x00, 0x01, 0x02] {
            let functions = capture_single(capture_config(
                0x1000_1111,
                0x0102_0304,
                header_type,
                0x0105,
            ))
            .unwrap();
            assert_eq!(functions.len(), 1);
            assert!(functions[0].bars.is_empty());
        }

        let mut unavailable = capture_config(0x2000_2222, 0x0203_0405, 0x00, 0x0206);
        unavailable.set_bar(0, u32::MAX, u32::MAX);
        assert_eq!(
            capture_single(unavailable).unwrap_err(),
            "pci_capture_bar_unavailable"
        );

        let mut all_ones_mask = capture_config(0x3000_3333, 0x0304_0506, 0x00, 0x0307);
        all_ones_mask.set_bar(0, 0xd000_0000, u32::MAX);
        assert_eq!(
            capture_single(all_ones_mask).unwrap_err(),
            "pci_capture_bar_unavailable"
        );

        let mut invalid = capture_config(0x4000_4444, 0x0405_0607, 0x00, 0x0408);
        invalid.set_bar(0, 0xd000_0000, 0xffff_f0f0);
        assert_eq!(
            capture_single(invalid).unwrap_err(),
            "pci_capture_bar_invalid"
        );
    }

    #[test]
    fn capture_enumeration_rejects_identity_class_and_header_changes() {
        let mut identity = capture_config(0x1000_1111, 0x0102_0304, 0x00, 0x0105);
        identity.set_register_after_reads(0x00, 8, 0x9999_8888);
        assert_eq!(
            capture_single(identity).unwrap_err(),
            "pci_capture_identity_changed"
        );

        let mut class = capture_config(0x2000_2222, 0x0203_0405, 0x00, 0x0206);
        class.set_register_after_reads(0x08, 1, 0x0908_0706);
        assert_eq!(
            capture_single(class).unwrap_err(),
            "pci_capture_metadata_changed"
        );

        let mut header = capture_config(0x3000_3333, 0x0304_0506, 0x00, 0x0307);
        header.set_register_after_reads(0x0c, 7, 0x0080_0000);
        assert_eq!(
            capture_single(header).unwrap_err(),
            "pci_capture_header_changed"
        );
    }

    #[test]
    fn capture_enumeration_rejects_command_irq_and_mixed_metadata_changes() {
        let mut command = capture_config(0x1000_1111, 0x0102_0304, 0x00, 0x0105);
        command.set_register_after_reads(0x04, 1, 0xa55a_0007);
        assert_eq!(
            capture_single(command).unwrap_err(),
            "pci_capture_metadata_changed"
        );

        let mut irq = capture_config(0x2000_2222, 0x0203_0405, 0x00, 0x0206);
        irq.set_register_after_reads(0x3c, 1, 0x0000_0309);
        assert_eq!(
            capture_single(irq).unwrap_err(),
            "pci_capture_metadata_changed"
        );

        let mut mixed = capture_config(0x3000_3333, 0x0304_0506, 0x00, 0x0307);
        mixed.set_register_after_reads(0x08, 1, 0x0908_0706);
        mixed.set_register_after_reads(0x3c, 1, 0x0000_010a);
        assert_eq!(
            capture_single(mixed).unwrap_err(),
            "pci_capture_metadata_changed"
        );
    }

    #[test]
    fn capture_enumeration_rejects_all_ones_active_metadata() {
        let mut command = capture_config(0x1000_1111, 0x0102_0304, 0x00, 0x0105);
        command.registers[1] = 0xa55a_ffff;

        let mut class = capture_config(0x2000_2222, 0x0203_0405, 0x00, 0x0206);
        class.registers[2] = u32::MAX;

        let mut header = capture_config(0x3000_3333, 0x0304_0506, 0x00, 0x0307);
        header.registers[3] = u32::MAX;

        let mut irq = capture_config(0x4000_4444, 0x0405_0607, 0x00, 0x0408);
        irq.registers[15] = u16::MAX as u32;

        let identity = capture_config(0xffff_5555, 0x0506_0708, 0x00, 0x0109);

        for config in [command, class, header, irq, identity] {
            assert!(capture_single(config).is_err());
        }
    }

    #[test]
    fn production_u16_transport_writes_one_cf8_dword_and_one_data_word() {
        let mut lower = FakePortIo::new();
        assert_eq!(
            pci_config_write_u16_with_io(&mut lower, 0x12, 0x1a, 0x05, 0x04, 0xa55a),
            Some(())
        );
        assert_eq!(
            lower.writes,
            vec![
                PortWrite::U32(0x0cf8, 0x8012_d504),
                PortWrite::U16(0x0cfc, 0xa55a),
            ]
        );

        let mut upper = FakePortIo::new();
        assert_eq!(
            pci_config_write_u16_with_io(&mut upper, 0x12, 0x1a, 0x05, 0x06, 0x5aa5),
            Some(())
        );
        assert_eq!(
            upper.writes,
            vec![
                PortWrite::U32(0x0cf8, 0x8012_d504),
                PortWrite::U16(0x0cfe, 0x5aa5),
            ]
        );

        let mut odd = FakePortIo::new();
        assert_eq!(
            pci_config_write_u16_with_io(&mut odd, 0x12, 0x1a, 0x05, 0x05, 0xffff),
            None
        );
        assert!(odd.writes.is_empty());
    }

    #[test]
    fn command_word_write_preserves_status_that_dword_replay_would_clear() {
        let original = 0x5a25_a5a7;
        let mut word_write = FakeConfig::new(original);
        word_write.write_u16(0x04, 0xa5a4);
        assert_eq!(word_write.registers[1], 0x5a25_a5a4);

        let mut dword_replay = FakeConfig::new(original);
        dword_replay.write_u32(0x04, 0x5a25_a5a4);
        assert_eq!(dword_replay.registers[1], 0x0225_a5a4);
    }

    #[test]
    fn io_bar_sizes_and_restores_full_command_and_bar() {
        let mut config = FakeConfig::new(0x5aa5_a5a7);
        config.set_bar(0, 0x0000_c001, 0xffff_ff01);
        let before = config.snapshot();

        let bar = read_bar_info_with_backend(&mut config, 0);

        assert_bar(bar, PciBarKind::Io, 0xc000, 0x100);
        assert_eq!(config.snapshot(), before);
        assert_eq!(
            config.writes,
            vec![
                Write::U16(0x04, 0xa5a4),
                Write::U32(0x10, u32::MAX),
                Write::U32(0x10, 0x0000_c001),
                Write::U16(0x04, 0xa5a7),
            ]
        );
    }

    #[test]
    fn memory32_bar_sizes_and_restores_exact_state() {
        let mut config = FakeConfig::new(0xc33c_0703);
        config.set_bar(2, 0xd000_0008, 0xffff_e008);
        let before = config.snapshot();

        let bar = read_bar_info_with_backend(&mut config, 2);

        assert_bar(bar, PciBarKind::Memory32, 0xd000_0000, 0x2000);
        assert_eq!(config.snapshot(), before);
        assert_eq!(
            config.writes,
            vec![
                Write::U16(0x04, 0x0700),
                Write::U32(0x18, u32::MAX),
                Write::U32(0x18, 0xd000_0008),
                Write::U16(0x04, 0x0703),
            ]
        );
    }

    #[test]
    fn memory64_restores_high_before_low_and_iteration_skips_follower() {
        let mut config = FakeConfig::new(0xf00d_0003);
        // The 2 MiB sizing mask requires a 2 MiB-aligned base.
        config.set_bar(0, 0x3440_0004, 0xffe0_0004);
        config.set_bar(1, 0x0000_0012, 0xffff_ffff);
        config.set_bar(2, 0xe000_0000, 0xffff_f000);
        let before = config.snapshot();

        let bars = read_bars_with_backend(&mut config);

        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].index, 0);
        assert_eq!(bars[0].kind, PciBarKind::Memory64);
        assert_eq!(bars[0].base, 0x0000_0012_3440_0000);
        assert_eq!(bars[0].size, 0x20_0000);
        assert_eq!(bars[1].index, 2);
        assert_eq!(bars[1].kind, PciBarKind::Memory32);
        assert_eq!(config.snapshot(), before);
        assert_eq!(
            config.writes,
            vec![
                Write::U16(0x04, 0x0000),
                Write::U32(0x10, u32::MAX),
                Write::U32(0x14, u32::MAX),
                Write::U32(0x14, 0x0000_0012),
                Write::U32(0x10, 0x3440_0004),
                Write::U16(0x04, 0x0003),
                Write::U16(0x04, 0x0000),
                Write::U32(0x18, u32::MAX),
                Write::U32(0x18, 0xe000_0000),
                Write::U16(0x04, 0x0003),
            ]
        );
    }

    #[test]
    fn unassigned_memory64_restores_state_and_iteration_skips_follower() {
        let mut config = FakeConfig::new(0xa55a_0003);
        config.set_bar(0, 0x0000_0004, 0xffff_f004);
        config.set_bar(1, 0x0000_0000, 0xffff_ffff);
        config.set_bar(2, 0xe000_0000, 0xffff_f000);
        let before = config.snapshot();

        let bars = read_bars_with_backend(&mut config);

        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].index, 2);
        assert_eq!(bars[0].kind, PciBarKind::Memory32);
        assert_eq!(config.read_counts[5], 2);
        assert_eq!(config.snapshot(), before);
    }

    #[test]
    fn enumeration_discards_collected_bars_before_stale_identity_or_header_slot_access() {
        let mut identity_change = FakeConfig::new(0xa55a_0003);
        identity_change.set_bar(0, 0xd000_0000, 0xffff_f000);
        identity_change.identity_after_reads = Some((3, 0x9999_8888));
        let identity_before = identity_change.snapshot();

        assert!(read_bars_with_backend(&mut identity_change).is_empty());
        assert_eq!(identity_change.snapshot(), identity_before);
        assert_eq!(identity_change.read_counts[5], 0);
        assert_eq!(identity_change.writes.len(), 4);

        for replacement_header_type in [0x01, 0x02] {
            let mut header_change = FakeConfig::new(0x5aa5_0003);
            header_change.set_header_type(0x00);
            header_change.set_bar(0, 0xd000_0000, 0xffff_f000);
            header_change.set_bar(1, 0x8000_0002, 0xffff_f002);
            header_change.header_after_reads = Some((2, (replacement_header_type as u32) << 16));
            let header_before = header_change.snapshot();

            assert!(read_bars_with_backend(&mut header_change).is_empty());
            assert_eq!(header_change.snapshot(), header_before);
            assert_eq!(header_change.read_counts[5], 0);
            assert_eq!(header_change.writes.len(), 4);
        }
    }

    #[test]
    fn noncontiguous_io_memory32_and_memory64_masks_are_rejected_and_restored() {
        let mut io = FakeConfig::new(0x1111_0003);
        io.set_bar(0, 0x0000_c001, 0xffff_ff0d);
        let io_before = io.snapshot();
        assert!(read_bar_info_with_backend(&mut io, 0).is_none());
        assert_eq!(io.snapshot(), io_before);

        let mut memory32 = FakeConfig::new(0x2222_0003);
        memory32.set_bar(0, 0xd000_0000, 0xffff_f0f0);
        let memory32_before = memory32.snapshot();
        assert!(read_bar_info_with_backend(&mut memory32, 0).is_none());
        assert_eq!(memory32.snapshot(), memory32_before);

        let mut memory64 = FakeConfig::new(0x3333_0003);
        memory64.set_bar(0, 0x3450_0004, 0xffff_f0f4);
        memory64.set_bar(1, 0x0000_0012, 0xffff_ffff);
        let memory64_before = memory64.snapshot();
        assert!(read_bar_info_with_backend(&mut memory64, 0).is_none());
        assert_eq!(memory64.snapshot(), memory64_before);
    }

    #[test]
    fn repeated_xhci_like_sizing_is_deterministic_and_non_cumulative() {
        let mut config = FakeConfig::new(0x1234_0407);
        config.set_bar(0, 0xf720_0000, 0xffff_0000);
        let before = config.snapshot();

        let first = read_bar_info_with_backend(&mut config, 0).unwrap();
        let after_first = config.snapshot();
        let second = read_bar_info_with_backend(&mut config, 0).unwrap();

        assert_eq!(first.kind, PciBarKind::Memory32);
        assert_eq!(first.base, 0xf720_0000);
        assert_eq!(first.size, 0x1_0000);
        assert_eq!(first.index, second.index);
        assert_eq!(first.kind, second.kind);
        assert_eq!(first.base, second.base);
        assert_eq!(first.size, second.size);
        assert_eq!(after_first, before);
        assert_eq!(config.snapshot(), before);
    }

    #[test]
    fn invalid_and_null_bars_do_not_write_configuration() {
        let mut config = FakeConfig::new(0xbeef_0003);
        let before = config.snapshot();

        assert!(read_bar_info_with_backend(&mut config, 6).is_none());
        assert!(read_bar_info_with_backend(&mut config, 0).is_none());

        assert_eq!(config.snapshot(), before);
        assert!(config.writes.is_empty());
    }

    #[test]
    fn unimplemented_mask_and_unsupported_type_leave_no_residual_change() {
        let mut unimplemented = FakeConfig::new(0xface_0043);
        unimplemented.set_bar(0, 0x9000_0000, 0x0000_0000);
        let unimplemented_before = unimplemented.snapshot();
        assert!(read_bar_info_with_backend(&mut unimplemented, 0).is_none());
        assert_eq!(unimplemented.snapshot(), unimplemented_before);

        let mut unsupported = FakeConfig::new(0xabcd_00c3);
        unsupported.set_bar(0, 0x8000_0002, 0xffff_f002);
        let unsupported_before = unsupported.snapshot();
        assert!(read_bar_info_with_backend(&mut unsupported, 0).is_none());
        assert_eq!(unsupported.snapshot(), unsupported_before);
        assert!(unsupported.writes.is_empty());
    }

    #[test]
    fn header_type_zero_one_and_two_bound_usable_bars() {
        let mut type_zero = FakeConfig::new(0x1000_0003);
        type_zero.set_header_type(0x00);
        type_zero.set_bar(5, 0x0000_c001, 0xffff_ff01);
        assert_bar(
            read_bar_info_with_backend(&mut type_zero, 5),
            PciBarKind::Io,
            0xc000,
            0x100,
        );

        let mut type_one = FakeConfig::new(0x2000_0003);
        type_one.set_header_type(0x01);
        type_one.set_bar(1, 0xd000_0000, 0xffff_f000);
        assert_bar(
            read_bar_info_with_backend(&mut type_one, 1),
            PciBarKind::Memory32,
            0xd000_0000,
            0x1000,
        );

        let mut type_two = FakeConfig::new(0x3000_0003);
        type_two.set_header_type(0x02);
        type_two.set_bar(0, 0x0000_d001, 0xffff_ff01);
        assert_bar(
            read_bar_info_with_backend(&mut type_two, 0),
            PciBarKind::Io,
            0xd000,
            0x100,
        );
    }

    #[test]
    fn final_slot_memory64_is_truncated_without_follower_access_or_mutation() {
        for (header_type, final_index) in [(0x00, 5), (0x01, 1), (0x02, 0)] {
            let mut config = FakeConfig::new(0x5a5a_0003);
            config.set_header_type(header_type);
            config.set_bar(final_index, 0x8000_0004, 0xffff_f004);
            let follower_slot = (0x10 / 4 + final_index + 1) as usize;

            let probe = read_bar_probe_with_backend(&mut config, final_index);

            assert_eq!(probe.consumed_slots, 2);
            assert!(matches!(
                probe.disposition,
                BarDisposition::Invalid(BarInvalidReason::TruncatedMemory64)
            ));
            assert_eq!(config.read_counts[follower_slot], 0);
            assert!(config.writes.is_empty());
        }
    }

    #[test]
    fn direct_memory64_follower_query_never_reads_or_sizes_follower() {
        let mut config = FakeConfig::new(0x1234_0003);
        config.set_bar(0, 0x8000_0004, 0xffff_f004);
        config.set_bar(1, 0x0000_0001, 0xffff_ffff);

        let probe = read_bar_probe_with_backend(&mut config, 1);

        assert_eq!(probe.consumed_slots, 1);
        assert!(matches!(
            probe.disposition,
            BarDisposition::Invalid(BarInvalidReason::Follower)
        ));
        assert_eq!(config.read_counts[5], 0);
        assert!(config.writes.is_empty());
    }

    #[test]
    fn unavailable_identity_header_leader_and_command_never_mutate() {
        for unavailable_identity in [u32::MAX, 0x1234_ffff] {
            let mut identity = FakeConfig::new(0x1000_0003);
            identity.registers[0] = unavailable_identity;
            assert!(read_bar_info_with_backend(&mut identity, 0).is_none());
            assert_eq!(identity.read_counts[3], 0);
            assert!(identity.writes.is_empty());
        }

        let mut header = FakeConfig::new(0x2000_0003);
        header.registers[3] = u32::MAX;
        assert!(read_bar_info_with_backend(&mut header, 0).is_none());
        assert!(header.writes.is_empty());

        let mut leader = FakeConfig::new(0x3000_0003);
        leader.set_bar(0, u32::MAX, 0);
        assert!(read_bar_info_with_backend(&mut leader, 0).is_none());
        assert!(leader.writes.is_empty());

        let mut command = FakeConfig::new(0x4000_ffff);
        command.set_bar(0, 0xd000_0000, 0xffff_f000);
        assert!(read_bar_info_with_backend(&mut command, 0).is_none());
        assert!(command.writes.is_empty());
    }

    #[test]
    fn disappearance_or_changed_identity_after_mutation_is_unusable_and_restored() {
        for final_identity in [u32::MAX, 0x9999_8888] {
            let mut config = FakeConfig::new(0x5a25_0003);
            config.set_bar(0, 0xd000_0000, 0xffff_f000);
            config.identity_after_mutation = Some(final_identity);
            let before = config.snapshot();

            let probe = read_bar_probe_with_backend(&mut config, 0);

            assert!(matches!(probe.disposition, BarDisposition::Unavailable));
            assert_eq!(config.snapshot(), before);
            assert_eq!(config.writes.last(), Some(&Write::U16(0x04, 0x0003)));
        }
    }

    #[test]
    fn all_ones_sizing_low_is_unavailable_and_restored() {
        let mut config = FakeConfig::new(0x7b7b_0003);
        config.set_bar(0, 0xd000_0000, u32::MAX);
        let before = config.snapshot();

        let probe = read_bar_probe_with_backend(&mut config, 0);

        assert!(matches!(probe.disposition, BarDisposition::Unavailable));
        assert_eq!(config.snapshot(), before);
        assert_eq!(config.writes.last(), Some(&Write::U16(0x04, 0x0003)));
    }

    #[test]
    fn mismatched_sizing_encodings_for_each_bar_class_are_rejected_after_exact_restore() {
        let mut io = FakeConfig::new(0x1111_0003);
        io.set_bar(0, 0x0000_c001, 0xffff_f000);
        let io_before = io.snapshot();
        let io_probe = read_bar_probe_with_backend(&mut io, 0);
        assert!(matches!(
            io_probe.disposition,
            BarDisposition::Invalid(BarInvalidReason::MismatchedSizingEncoding)
        ));
        assert_eq!(io.snapshot(), io_before);
        assert_eq!(io.writes.last(), Some(&Write::U16(0x04, 0x0003)));

        let mut memory32 = FakeConfig::new(0x2222_0003);
        memory32.set_bar(0, 0xd000_0000, 0xffff_f001);
        let memory32_before = memory32.snapshot();
        let memory32_probe = read_bar_probe_with_backend(&mut memory32, 0);
        assert!(matches!(
            memory32_probe.disposition,
            BarDisposition::Invalid(BarInvalidReason::MismatchedSizingEncoding)
        ));
        assert_eq!(memory32.snapshot(), memory32_before);
        assert_eq!(memory32.writes.last(), Some(&Write::U16(0x04, 0x0003)));

        let mut memory64 = FakeConfig::new(0x3333_0003);
        memory64.set_bar(0, 0x3440_0004, 0xffff_f000);
        memory64.set_bar(1, 0x0000_0012, u32::MAX);
        let memory64_before = memory64.snapshot();
        let memory64_probe = read_bar_probe_with_backend(&mut memory64, 0);
        assert!(matches!(
            memory64_probe.disposition,
            BarDisposition::Invalid(BarInvalidReason::MismatchedSizingEncoding)
        ));
        assert_eq!(memory64.snapshot(), memory64_before);
        assert_eq!(memory64.writes.last(), Some(&Write::U16(0x04, 0x0003)));
    }

    #[test]
    fn unassigned_and_invalid_memory64_always_consume_two_slots() {
        let mut unassigned = FakeConfig::new(0x1111_0003);
        unassigned.set_bar(0, 0x0000_0004, 0xffff_f004);
        unassigned.set_bar(1, 0, u32::MAX);
        let unassigned_probe = read_bar_probe_with_backend(&mut unassigned, 0);
        assert_eq!(unassigned_probe.consumed_slots, 2);
        assert!(matches!(
            unassigned_probe.disposition,
            BarDisposition::Unassigned
        ));

        let mut invalid = FakeConfig::new(0x2222_0003);
        invalid.set_bar(0, 0x8000_0004, 0xffff_f0f4);
        invalid.set_bar(1, 0, u32::MAX);
        let invalid_probe = read_bar_probe_with_backend(&mut invalid, 0);
        assert_eq!(invalid_probe.consumed_slots, 2);
        assert!(matches!(
            invalid_probe.disposition,
            BarDisposition::Invalid(BarInvalidReason::NonCanonicalMask)
        ));
    }

    #[test]
    fn misaligned_bases_for_each_width_are_rejected_after_exact_restore() {
        let mut io = FakeConfig::new(0x1111_0003);
        io.set_bar(0, 0x0000_c081, 0xffff_ff01);
        let io_before = io.snapshot();
        assert!(read_bar_info_with_backend(&mut io, 0).is_none());
        assert_eq!(io.snapshot(), io_before);

        let mut memory32 = FakeConfig::new(0x2222_0003);
        memory32.set_bar(0, 0xd000_1000, 0xffff_e000);
        let memory32_before = memory32.snapshot();
        assert!(read_bar_info_with_backend(&mut memory32, 0).is_none());
        assert_eq!(memory32.snapshot(), memory32_before);

        let mut memory64 = FakeConfig::new(0x3333_0003);
        memory64.set_bar(0, 0x3450_1004, 0xffff_e004);
        memory64.set_bar(1, 0x0000_0012, u32::MAX);
        let memory64_before = memory64.snapshot();
        assert!(read_bar_info_with_backend(&mut memory64, 0).is_none());
        assert_eq!(memory64.snapshot(), memory64_before);
    }

    #[test]
    fn range_overflow_for_each_width_is_rejected_after_probe_and_exact_restore() {
        let mut io = FakeConfig::new(0x1111_0003);
        io.set_bar(0, 0xffff_f001, 0xffff_e001);
        let io_before = io.snapshot();
        let io_probe = read_bar_probe_with_backend(&mut io, 0);
        assert!(matches!(
            io_probe.disposition,
            BarDisposition::Invalid(BarInvalidReason::AddressRangeOverflow)
        ));
        assert_eq!(io.snapshot(), io_before);
        assert_eq!(io.writes.last(), Some(&Write::U16(0x04, 0x0003)));

        let mut memory32 = FakeConfig::new(0x2222_0003);
        memory32.set_bar(0, 0xffff_f000, 0xffff_e000);
        let memory32_before = memory32.snapshot();
        let memory32_probe = read_bar_probe_with_backend(&mut memory32, 0);
        assert!(matches!(
            memory32_probe.disposition,
            BarDisposition::Invalid(BarInvalidReason::AddressRangeOverflow)
        ));
        assert_eq!(memory32.snapshot(), memory32_before);
        assert_eq!(memory32.writes.last(), Some(&Write::U16(0x04, 0x0003)));

        let mut memory64 = FakeConfig::new(0x3333_0003);
        memory64.set_bar(0, 0xffff_f004, 0xffff_e004);
        memory64.set_bar(1, u32::MAX, u32::MAX);
        let memory64_before = memory64.snapshot();
        let memory64_probe = read_bar_probe_with_backend(&mut memory64, 0);
        assert!(matches!(
            memory64_probe.disposition,
            BarDisposition::Invalid(BarInvalidReason::AddressRangeOverflow)
        ));
        assert_eq!(memory64.snapshot(), memory64_before);
        assert_eq!(memory64.writes.last(), Some(&Write::U16(0x04, 0x0003)));
    }
}
