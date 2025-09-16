use crate::pci::{self, PciAddress};
use crate::serial;

use super::{device_id, VIRTIO_VENDOR_ID};

#[derive(Debug, Clone, Copy)]
pub enum VirtioNetKind {
    Legacy,
    Modern,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioNet {
    pub address: PciAddress,
    pub kind: VirtioNetKind,
}

impl VirtioNet {
    pub fn configure(&self) {
        match self.kind {
            VirtioNetKind::Legacy => serial::write_line("virtio-net legacy transport configuration pending"),
            VirtioNetKind::Modern => serial::write_line("virtio-net modern transport unsupported"),
        }
    }
}

pub fn probe() -> Option<VirtioNet> {
    if let Some(addr) = pci::find_device(VIRTIO_VENDOR_ID, device_id::LEGACY_NET) {
        serial::write_fmt(format_args!("virtio-net (legacy) @ {} detected\r\n", addr));
        return Some(VirtioNet {
            address: addr,
            kind: VirtioNetKind::Legacy,
        });
    }

    if let Some(addr) = pci::find_device(VIRTIO_VENDOR_ID, device_id::MODERN_NET) {
        serial::write_fmt(format_args!(
            "virtio-net (modern) @ {} detected but modern transport not supported yet\r\n",
            addr
        ));
        return Some(VirtioNet {
            address: addr,
            kind: VirtioNetKind::Modern,
        });
    }

    serial::write_line("virtio-net device not present");
    None
}
