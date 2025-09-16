pub mod rng;
pub mod net;

pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;

pub mod device_id {
    pub const LEGACY_RNG: u16 = 0x1005;
    pub const MODERN_RNG: u16 = 0x1044;
    pub const LEGACY_NET: u16 = 0x1000;
    pub const MODERN_NET: u16 = 0x1041;
}
