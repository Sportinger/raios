pub mod rng;

pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;

pub mod device_id {
    pub const LEGACY_RNG: u16 = 0x1005;
    pub const MODERN_RNG: u16 = 0x1044;
}
