//! The existing bounded Marvell guided setup flow, separated from Genesis paint.

use core::fmt::Write;

use raios_core::dot11_scan::Dot11Security;

use crate::framebuffer::FramebufferSurface;
use crate::system_status::TextBuf;
use crate::{console, marvell_wifi_pcie, net, text, wifi};

use super::genesis::{
    draw_button, draw_outline, draw_panel, draw_truncated_text, point_in, LogicalRect, APP_AMBER,
    APP_BLUE, APP_GREEN, APP_RED, FONT_ADVANCE, HAIRLINE, SURFACE_ALT, TEXT_FAINT, TEXT_MAIN,
    TEXT_MUTED,
};

const LIST_LIMIT: usize = 8;
const LIST_ROW_HEIGHT: usize = 28;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Starting {
        scan_started: bool,
    },
    Selecting,
    Password {
        network_index: usize,
        remember_for_boot: bool,
        rejected: bool,
    },
    Associating {
        network_index: usize,
        remember_for_boot: bool,
        started: bool,
    },
    Configured {
        network_index: usize,
        remember_for_boot: bool,
    },
    Failed(&'static str),
}

pub struct GuidedWifi {
    state: State,
}

impl GuidedWifi {
    pub const fn new() -> Self {
        Self { state: State::Idle }
    }

    pub fn is_active(&self) -> bool {
        self.state != State::Idle
    }

    pub fn begin(&mut self) -> bool {
        if wifi::snapshot().state != wifi::WifiState::Detected {
            self.state = State::Failed("wifi_device_not_detected");
            return true;
        }
        let firmware = marvell_wifi_pcie::snapshot();
        if firmware.is_failed() {
            self.state = State::Failed(
                firmware
                    .result
                    .map(|result| result.label())
                    .unwrap_or("firmware_failed"),
            );
            return true;
        }
        self.state = State::Starting {
            scan_started: false,
        };
        if firmware.is_ready() || firmware.running {
            return true;
        }
        let result = marvell_wifi_pcie::start_bring_up_firmware();
        console::write_event(format_args!("WIFI UI START: {}", result.label()));
        match result {
            marvell_wifi_pcie::FirmwareBringupTriggerResult::Started
            | marvell_wifi_pcie::FirmwareBringupTriggerResult::AlreadyRunning => true,
            marvell_wifi_pcie::FirmwareBringupTriggerResult::AlreadyAttempted => {
                self.state = State::Failed("firmware_already_attempted");
                true
            }
            marvell_wifi_pcie::FirmwareBringupTriggerResult::Failed(error) => {
                self.state = State::Failed(error.label());
                true
            }
        }
    }

    pub fn advance(&mut self) -> bool {
        match self.state {
            State::Starting { scan_started } => self.advance_starting(scan_started),
            State::Password {
                network_index,
                remember_for_boot,
                ..
            } => match console::snapshot().wifi_passphrase_entry_result {
                console::WifiPassphraseEntryResult::Set => {
                    self.state = State::Associating {
                        network_index,
                        remember_for_boot,
                        started: false,
                    };
                    true
                }
                console::WifiPassphraseEntryResult::Cancelled => {
                    self.state = State::Selecting;
                    true
                }
                console::WifiPassphraseEntryResult::Rejected => {
                    let _ = console::activate_focus(console::UiFocus::SettingsWifiPassphrase);
                    self.state = State::Password {
                        network_index,
                        remember_for_boot,
                        rejected: true,
                    };
                    true
                }
                console::WifiPassphraseEntryResult::None => false,
            },
            State::Associating {
                network_index,
                remember_for_boot,
                started,
            } => self.advance_association(network_index, remember_for_boot, started),
            State::Idle | State::Selecting | State::Configured { .. } | State::Failed(_) => false,
        }
    }

    fn advance_starting(&mut self, scan_started: bool) -> bool {
        let firmware = marvell_wifi_pcie::snapshot();
        if firmware.is_failed() {
            self.state = State::Failed(
                firmware
                    .result
                    .map(|value| value.label())
                    .unwrap_or("firmware_failed"),
            );
            return true;
        }
        if !firmware.is_ready() {
            return false;
        }
        let hw_spec = marvell_wifi_pcie::hw_spec_snapshot();
        if hw_spec.is_failed() {
            self.state = State::Failed(
                hw_spec
                    .result
                    .map(|value| value.label())
                    .unwrap_or("hw_spec_failed"),
            );
            return true;
        }
        if !hw_spec.is_ready() {
            return false;
        }
        if !scan_started {
            let result = marvell_wifi_pcie::start_scan_ext_24ghz();
            console::write_event(format_args!("WIFI UI SCAN: {}", result.label()));
            return match result {
                marvell_wifi_pcie::ScanCmdTriggerResult::Started
                | marvell_wifi_pcie::ScanCmdTriggerResult::AlreadyRunning => {
                    self.state = State::Starting { scan_started: true };
                    true
                }
                marvell_wifi_pcie::ScanCmdTriggerResult::Failed(error) => {
                    self.state = State::Failed(error.label());
                    true
                }
            };
        }
        let scan = marvell_wifi_pcie::scan_cmd_snapshot();
        if scan.stage == marvell_wifi_pcie::ScanCmdStage::Failed {
            self.state = State::Failed(
                scan.result
                    .map(|value| value.label())
                    .unwrap_or("scan_failed"),
            );
            true
        } else if scan.stage == marvell_wifi_pcie::ScanCmdStage::Done {
            self.state = State::Selecting;
            true
        } else {
            false
        }
    }

    fn advance_association(
        &mut self,
        network_index: usize,
        remember_for_boot: bool,
        started: bool,
    ) -> bool {
        if !started {
            let result = marvell_wifi_pcie::start_association();
            console::write_event(format_args!("WIFI ASSOC: {}", result.label()));
            return match result {
                marvell_wifi_pcie::ConnectionTriggerResult::Started
                | marvell_wifi_pcie::ConnectionTriggerResult::AlreadyRunning => {
                    self.state = State::Associating {
                        network_index,
                        remember_for_boot,
                        started: true,
                    };
                    true
                }
                marvell_wifi_pcie::ConnectionTriggerResult::AlreadyReady => {
                    self.state = State::Configured {
                        network_index,
                        remember_for_boot,
                    };
                    true
                }
                marvell_wifi_pcie::ConnectionTriggerResult::Failed(error) => {
                    self.state = State::Failed(error.label());
                    true
                }
            };
        }
        let connection = marvell_wifi_pcie::connection_snapshot();
        if connection.is_failed() {
            self.state = State::Failed(
                connection
                    .result
                    .map(|value| value.label())
                    .unwrap_or("association_failed"),
            );
            true
        } else if connection.is_ready() {
            self.state = State::Configured {
                network_index,
                remember_for_boot,
            };
            true
        } else {
            false
        }
    }

    pub fn handle_pointer(&mut self, x: usize, y: usize, width: usize, height: usize) -> bool {
        match self.state {
            State::Idle | State::Starting { .. } | State::Associating { .. } => false,
            State::Selecting => self.select_pointer(x, y, width, height),
            State::Password {
                network_index,
                remember_for_boot,
                rejected,
            } => {
                let rect = password_rect(width, height);
                if point_in(
                    x,
                    y,
                    LogicalRect::new(rect.x + 24, rect.y + 116, rect.w - 48, 20),
                ) {
                    self.state = State::Password {
                        network_index,
                        remember_for_boot: !remember_for_boot,
                        rejected,
                    };
                    wifi::set_remember_passphrase_for_boot(!remember_for_boot);
                    return true;
                }
                let [back, submit] = action_rects(rect);
                if point_in(x, y, back) {
                    let _ = console::cancel_wifi_passphrase_entry();
                    self.state = State::Selecting;
                    return true;
                }
                if point_in(x, y, submit) {
                    wifi::set_remember_passphrase_for_boot(remember_for_boot);
                    return console::submit_wifi_passphrase_entry();
                }
                false
            }
            State::Configured { .. } => {
                let [_, done] = action_rects(result_rect(width, height));
                if point_in(x, y, done) {
                    self.state = State::Idle;
                    true
                } else {
                    false
                }
            }
            State::Failed(_) => {
                let [retry, close] = action_rects(result_rect(width, height));
                if point_in(x, y, retry) {
                    self.begin()
                } else if point_in(x, y, close) {
                    self.state = State::Idle;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn select_pointer(&mut self, x: usize, y: usize, width: usize, height: usize) -> bool {
        let scan = wifi::scan_results();
        let rect = selection_rect(width, height, scan.count);
        for index in 0..usize::min(scan.count, LIST_LIMIT) {
            let row = LogicalRect::new(
                rect.x + 18,
                rect.y + 48 + index * LIST_ROW_HEIGHT,
                rect.w - 36,
                22,
            );
            if !point_in(x, y, row) {
                continue;
            }
            let network = scan.networks[index];
            if network.hidden_ssid || network.ssid.is_empty() {
                self.state = State::Failed("hidden_ssid_requires_manual_entry");
                return true;
            }
            wifi::clear_config();
            if wifi::select_scan_result(index).is_err() {
                self.state = State::Failed("bss_evidence_unavailable");
                return true;
            }
            if network.security == Dot11Security::Open {
                self.state = State::Associating {
                    network_index: index,
                    remember_for_boot: false,
                    started: false,
                };
                return true;
            }
            wifi::set_remember_passphrase_for_boot(true);
            let _ = console::activate_focus(console::UiFocus::SettingsWifiPassphrase);
            self.state = State::Password {
                network_index: index,
                remember_for_boot: true,
                rejected: false,
            };
            return true;
        }
        let [scan_again, close] = action_rects(rect);
        if point_in(x, y, scan_again) {
            self.state = State::Starting {
                scan_started: false,
            };
            true
        } else if point_in(x, y, close) {
            self.state = State::Idle;
            true
        } else {
            false
        }
    }

    pub fn draw(&self, surface: &mut FramebufferSurface, width: usize, height: usize) {
        match self.state {
            State::Idle => {}
            State::Starting { .. } => draw_progress(surface, width, height, false),
            State::Selecting => draw_selection(surface, width, height),
            State::Password {
                network_index,
                remember_for_boot,
                rejected,
            } => draw_password(
                surface,
                width,
                height,
                network_index,
                remember_for_boot,
                rejected,
            ),
            State::Associating { .. } => draw_progress(surface, width, height, true),
            State::Configured {
                network_index,
                remember_for_boot,
            } => draw_configured(surface, width, height, network_index, remember_for_boot),
            State::Failed(reason) => draw_failed(surface, width, height, reason),
        }
    }
}

fn draw_progress(surface: &mut FramebufferSurface, width: usize, height: usize, associating: bool) {
    let rect = progress_rect(width, height);
    draw_panel(
        surface,
        rect,
        if associating {
            "Connecting WiFi"
        } else {
            "Starting WiFi"
        },
    );
    let (percent, label) = if associating {
        connection_progress()
    } else {
        startup_progress()
    };
    draw_truncated_text(
        surface,
        rect.x + 20,
        rect.y + 52,
        label,
        (rect.w - 40) / FONT_ADVANCE,
        TEXT_MAIN,
    );
    surface.fill_rect(rect.x + 20, rect.y + 78, rect.w - 40, 12, SURFACE_ALT);
    surface.fill_rect(
        rect.x + 20,
        rect.y + 78,
        (rect.w - 40) * percent / 100,
        12,
        APP_BLUE,
    );
    draw_outline(
        surface,
        LogicalRect::new(rect.x + 20, rect.y + 78, rect.w - 40, 12),
        HAIRLINE,
    );
}

fn draw_selection(surface: &mut FramebufferSurface, width: usize, height: usize) {
    let scan = wifi::scan_results();
    let rect = selection_rect(width, height, scan.count);
    draw_panel(surface, rect, "WiFi networks");
    if scan.count == 0 {
        text::draw_text(
            surface,
            rect.x + 20,
            rect.y + 56,
            "No networks found",
            TEXT_MUTED,
            None,
        );
    }
    for index in 0..usize::min(scan.count, LIST_LIMIT) {
        let network = scan.networks[index];
        let row = LogicalRect::new(
            rect.x + 18,
            rect.y + 48 + index * LIST_ROW_HEIGHT,
            rect.w - 36,
            22,
        );
        surface.fill_rect(row.x, row.y, row.w, row.h, SURFACE_ALT);
        draw_outline(surface, row, HAIRLINE);
        let line = scan_line(network);
        draw_truncated_text(
            surface,
            row.x + 8,
            row.y + 7,
            line.as_str(),
            (row.w - 16) / FONT_ADVANCE,
            if network.hidden_ssid {
                TEXT_FAINT
            } else {
                TEXT_MAIN
            },
        );
    }
    let [scan_again, close] = action_rects(rect);
    draw_button(surface, scan_again, "Scan again", false);
    draw_button(surface, close, "Close", false);
}

fn draw_password(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    network_index: usize,
    remember_for_boot: bool,
    rejected: bool,
) {
    let rect = password_rect(width, height);
    draw_panel(surface, rect, "WiFi password");
    let scan = wifi::scan_results();
    let ssid = scan
        .networks
        .get(network_index)
        .map(|item| item.ssid.as_str())
        .unwrap_or("Unknown network");
    draw_truncated_text(
        surface,
        rect.x + 20,
        rect.y + 48,
        ssid,
        (rect.w - 40) / FONT_ADVANCE,
        TEXT_MAIN,
    );
    let input = console::snapshot().input;
    surface.fill_rect(rect.x + 20, rect.y + 66, rect.w - 40, 28, SURFACE_ALT);
    draw_outline(
        surface,
        LogicalRect::new(rect.x + 20, rect.y + 66, rect.w - 40, 28),
        APP_BLUE,
    );
    draw_truncated_text(
        surface,
        rect.x + 28,
        rect.y + 76,
        input.as_str(),
        (rect.w - 56) / FONT_ADVANCE,
        TEXT_MAIN,
    );
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 106,
        if rejected {
            "Password must contain 8-63 printable characters"
        } else {
            "8-63 printable characters"
        },
        if rejected { APP_RED } else { TEXT_FAINT },
        None,
    );
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 120,
        if remember_for_boot {
            "[x] Remember for this boot (RAM only)"
        } else {
            "[ ] Remember for this boot (RAM only)"
        },
        TEXT_MUTED,
        None,
    );
    let [back, submit] = action_rects(rect);
    draw_button(surface, back, "Back", false);
    draw_button(surface, submit, "Set credentials", true);
}

fn draw_configured(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    network_index: usize,
    remember_for_boot: bool,
) {
    let rect = result_rect(width, height);
    draw_panel(surface, rect, "WiFi setup");
    let scan = wifi::scan_results();
    let ssid = scan
        .networks
        .get(network_index)
        .map(|item| item.ssid.as_str())
        .unwrap_or("Selected network");
    draw_truncated_text(
        surface,
        rect.x + 20,
        rect.y + 50,
        ssid,
        (rect.w - 40) / FONT_ADVANCE,
        APP_GREEN,
    );
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 70,
        if remember_for_boot {
            "Credentials ready in RAM for this boot"
        } else {
            "Open network selected for this boot"
        },
        TEXT_MAIN,
        None,
    );
    let link = if net::ui_snapshot()
        .and_then(|snapshot| snapshot.ip)
        .is_some()
    {
        "Connected"
    } else {
        "Link ready - requesting network address"
    };
    text::draw_text(surface, rect.x + 20, rect.y + 86, link, APP_AMBER, None);
    let [_, done] = action_rects(rect);
    draw_button(surface, done, "Done", true);
}

fn draw_failed(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    reason: &'static str,
) {
    let rect = result_rect(width, height);
    draw_panel(surface, rect, "WiFi unavailable");
    draw_truncated_text(
        surface,
        rect.x + 20,
        rect.y + 54,
        reason,
        (rect.w - 40) / FONT_ADVANCE,
        APP_RED,
    );
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 76,
        "No network state was granted",
        TEXT_MUTED,
        None,
    );
    let [retry, close] = action_rects(rect);
    draw_button(surface, retry, "Retry", false);
    draw_button(surface, close, "Close", false);
}

fn connection_progress() -> (usize, &'static str) {
    match marvell_wifi_pcie::connection_snapshot().stage {
        marvell_wifi_pcie::ConnectionStage::Idle => (4, "Preparing connection"),
        marvell_wifi_pcie::ConnectionStage::RegisterRings => (18, "Registering data rings"),
        marvell_wifi_pcie::ConnectionStage::MacControl => (32, "Enabling radio data path"),
        marvell_wifi_pcie::ConnectionStage::SupplicantProfile => (48, "Configuring WPA2 profile"),
        marvell_wifi_pcie::ConnectionStage::SupplicantPmk => (62, "Loading boot-only credential"),
        marvell_wifi_pcie::ConnectionStage::Associate => (76, "Associating with access point"),
        marvell_wifi_pcie::ConnectionStage::WaitPortRelease => (90, "Completing WPA2 key exchange"),
        marvell_wifi_pcie::ConnectionStage::LinkReady => (100, "Link ready; requesting address"),
        marvell_wifi_pcie::ConnectionStage::Failed => (100, "Connection failed"),
    }
}

fn startup_progress() -> (usize, &'static str) {
    let firmware = marvell_wifi_pcie::snapshot();
    if !firmware.is_ready() {
        let percent = if firmware.total == 0 {
            4
        } else {
            4 + firmware.downloaded.saturating_mul(76) / firmware.total
        };
        return (usize::min(percent, 80), "Loading radio firmware");
    }
    if !marvell_wifi_pcie::hw_spec_snapshot().is_ready() {
        return (88, "Reading radio identity");
    }
    if marvell_wifi_pcie::scan_cmd_snapshot().stage == marvell_wifi_pcie::ScanCmdStage::Done {
        (100, "Networks ready")
    } else {
        (94, "Scanning networks")
    }
}

fn centered_rect(
    width: usize,
    height: usize,
    wanted_width: usize,
    wanted_height: usize,
) -> LogicalRect {
    let w = wanted_width.min(width.saturating_sub(24));
    let h = wanted_height.min(height.saturating_sub(44));
    LogicalRect::new(
        width.saturating_sub(w) / 2,
        height.saturating_sub(h) / 2,
        w,
        h,
    )
}

fn selection_rect(width: usize, height: usize, count: usize) -> LogicalRect {
    centered_rect(
        width,
        height,
        360,
        112 + usize::min(count, LIST_LIMIT) * LIST_ROW_HEIGHT,
    )
}

fn password_rect(width: usize, height: usize) -> LogicalRect {
    centered_rect(width, height, 360, 190)
}
fn result_rect(width: usize, height: usize) -> LogicalRect {
    centered_rect(width, height, 340, 152)
}
fn progress_rect(width: usize, height: usize) -> LogicalRect {
    centered_rect(width, height, 340, 126)
}

fn action_rects(rect: LogicalRect) -> [LogicalRect; 2] {
    let width = rect.w.saturating_sub(50) / 2;
    let y = rect.y + rect.h.saturating_sub(32);
    [
        LogicalRect::new(rect.x + 16, y, width, 20),
        LogicalRect::new(rect.x + 34 + width, y, width, 20),
    ]
}

fn scan_line(network: wifi::ScannedNetwork) -> TextBuf<160> {
    let mut line = TextBuf::new();
    let ssid = if network.hidden_ssid || network.ssid.is_empty() {
        "(hidden)"
    } else {
        network.ssid.as_str()
    };
    if network.channel == 0 {
        let _ = write!(
            line,
            "{}  CH?  {}  {}",
            ssid,
            wifi::scan_security_label(network.security),
            network.source.tag()
        );
    } else {
        let _ = write!(
            line,
            "{}  CH{}  {}  {}",
            ssid,
            network.channel,
            wifi::scan_security_label(network.security),
            network.source.tag()
        );
    }
    line
}
