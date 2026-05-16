use crate::{provider, system_status};

#[derive(Clone, Copy)]
pub enum HealthSource {
    AlwaysReady,
    Framebuffer,
    Entropy,
    UsbXhci,
    Wifi,
    Network,
    Input,
    OpenAiDirect,
}

pub struct Service {
    pub id: &'static str,
    pub kind: &'static str,
    pub replaceable: bool,
    pub core_owned: bool,
    pub health_source: HealthSource,
    pub capabilities: &'static [&'static str],
}

pub struct ServiceHealth<'a> {
    pub state: &'static str,
    pub last_error: Option<&'a str>,
}

const CAP_BOOT: &[&str] = &["cap.system.boot_log.read"];
const CAP_SERIAL: &[&str] = &["cap.system.boot_log.read", "cap.system.capabilities.read"];
const CAP_SNAPSHOT: &[&str] = &["cap.system.snapshot.read", "cap.service.inventory.read"];
const CAP_UI: &[&str] = &["cap.system.snapshot.read"];
const CAP_CONSOLE: &[&str] = &[
    "cap.system.describe.read",
    "cap.system.snapshot.read",
    "cap.system.boot_log.read",
    "cap.system.capabilities.read",
    "cap.service.inventory.read",
    "cap.problem.list.read",
    "cap.device.graph.read",
];
const CAP_INPUT: &[&str] = &["cap.system.snapshot.read", "cap.device.graph.read"];
const CAP_DEVICE: &[&str] = &["cap.device.graph.read", "cap.system.snapshot.read"];
const CAP_PROVIDER: &[&str] = &["cap.system.snapshot.read"];
const CAP_NONE: &[&str] = &[];

pub const SERVICES: &[Service] = &[
    Service {
        id: "core.boot",
        kind: "core",
        replaceable: false,
        core_owned: true,
        health_source: HealthSource::AlwaysReady,
        capabilities: CAP_BOOT,
    },
    Service {
        id: "core.memory",
        kind: "core",
        replaceable: false,
        core_owned: true,
        health_source: HealthSource::AlwaysReady,
        capabilities: CAP_NONE,
    },
    Service {
        id: "core.serial",
        kind: "core",
        replaceable: false,
        core_owned: true,
        health_source: HealthSource::AlwaysReady,
        capabilities: CAP_SERIAL,
    },
    Service {
        id: "core.scheduler",
        kind: "core",
        replaceable: false,
        core_owned: true,
        health_source: HealthSource::AlwaysReady,
        capabilities: CAP_NONE,
    },
    Service {
        id: "core.entropy",
        kind: "core",
        replaceable: false,
        core_owned: true,
        health_source: HealthSource::Entropy,
        capabilities: CAP_SNAPSHOT,
    },
    Service {
        id: "core.snapshot_root",
        kind: "core",
        replaceable: false,
        core_owned: true,
        health_source: HealthSource::AlwaysReady,
        capabilities: CAP_SNAPSHOT,
    },
    Service {
        id: "svc.ui.framebuffer",
        kind: "service",
        replaceable: true,
        core_owned: false,
        health_source: HealthSource::Framebuffer,
        capabilities: CAP_UI,
    },
    Service {
        id: "svc.console",
        kind: "service",
        replaceable: true,
        core_owned: false,
        health_source: HealthSource::AlwaysReady,
        capabilities: CAP_CONSOLE,
    },
    Service {
        id: "svc.input",
        kind: "service",
        replaceable: true,
        core_owned: false,
        health_source: HealthSource::Input,
        capabilities: CAP_INPUT,
    },
    Service {
        id: "drv.usb.xhci",
        kind: "driver",
        replaceable: true,
        core_owned: false,
        health_source: HealthSource::UsbXhci,
        capabilities: CAP_DEVICE,
    },
    Service {
        id: "drv.net.e1000",
        kind: "driver",
        replaceable: true,
        core_owned: false,
        health_source: HealthSource::Network,
        capabilities: CAP_DEVICE,
    },
    Service {
        id: "svc.net.ipv4",
        kind: "service",
        replaceable: true,
        core_owned: false,
        health_source: HealthSource::Network,
        capabilities: CAP_DEVICE,
    },
    Service {
        id: "drv.wifi.avastar_probe",
        kind: "driver",
        replaceable: true,
        core_owned: false,
        health_source: HealthSource::Wifi,
        capabilities: CAP_DEVICE,
    },
    Service {
        id: "svc.provider.openai_direct",
        kind: "service",
        replaceable: true,
        core_owned: false,
        health_source: HealthSource::OpenAiDirect,
        capabilities: CAP_PROVIDER,
    },
];

pub fn service_health<'a>(
    service: &Service,
    snapshot: &'a system_status::SystemSnapshot,
    provider: &'a provider::Snapshot,
) -> ServiceHealth<'a> {
    match service.health_source {
        HealthSource::AlwaysReady => ServiceHealth {
            state: "healthy",
            last_error: None,
        },
        HealthSource::Framebuffer => status_health(&snapshot.framebuffer),
        HealthSource::Entropy => status_health(&snapshot.entropy),
        HealthSource::UsbXhci => status_health(&snapshot.usb_xhci),
        HealthSource::Wifi => status_health(&snapshot.wifi),
        HealthSource::Network => status_health(&snapshot.network),
        HealthSource::Input => status_health(&snapshot.input),
        HealthSource::OpenAiDirect => {
            if !provider.direct_last_error.as_str().is_empty() {
                ServiceHealth {
                    state: "degraded",
                    last_error: Some(provider.direct_last_error.as_str()),
                }
            } else {
                ServiceHealth {
                    state: "degraded",
                    last_error: Some("TLS certificate verification is bypassed"),
                }
            }
        }
    }
}

fn status_health<'a>(line: &'a system_status::StatusLine) -> ServiceHealth<'a> {
    match line.state {
        system_status::RowState::Ready
        | system_status::RowState::Configured
        | system_status::RowState::Detected => ServiceHealth {
            state: "healthy",
            last_error: None,
        },
        system_status::RowState::Waiting => ServiceHealth {
            state: "starting",
            last_error: None,
        },
        system_status::RowState::Degraded => ServiceHealth {
            state: "degraded",
            last_error: Some(line.detail.as_str()),
        },
        system_status::RowState::Missing => ServiceHealth {
            state: "missing",
            last_error: Some(line.detail.as_str()),
        },
    }
}
