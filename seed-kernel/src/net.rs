use spin::Mutex;

use crate::serial;
use crate::virtio;

static NET_DEVICE: Mutex<Option<virtio::net::VirtioNet>> = Mutex::new(None);

pub fn init() {
    let mut slot = NET_DEVICE.lock();
    if slot.is_some() {
        serial::write_line("virtio-net already initialised");
        return;
    }

    match virtio::net::probe() {
        Some(device) => {
            serial::write_line("virtio-net init stub (queue wiring pending)");
            device.configure();
            *slot = Some(device);
        }
        None => {
            serial::write_line("virtio-net probe failed; device absent or unsupported");
        }
    }
}

pub fn poll() {
    virtio::net::poll();
}

#[allow(dead_code)]
pub fn status() -> Option<virtio::net::VirtioNet> {
    *NET_DEVICE.lock()
}
