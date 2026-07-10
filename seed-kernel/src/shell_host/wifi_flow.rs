//! The existing bounded Marvell guided setup flow, separated from Genesis paint.

use core::fmt::Write;

use raios_core::{boot_control::BootPosture, dot11_scan::Dot11Security};

use crate::framebuffer::FramebufferSurface;
use crate::system_status::TextBuf;
use crate::{
    console, input, marvell_wifi_pcie, net, secret_vault,
    secure_overlay::{SecureOverlay, SecureOverlayAction, SecureOverlayInput, SecureOverlayPrompt},
    serial, text, wifi,
};

use super::genesis::{
    draw_button, draw_outline, draw_panel, draw_truncated_text, point_in, LogicalRect, APP_AMBER,
    APP_BLUE, APP_GREEN, APP_RED, FONT_ADVANCE, HAIRLINE, SURFACE_ALT, TEXT_FAINT, TEXT_MAIN,
    TEXT_MUTED,
};

const LIST_LIMIT: usize = 8;
const LIST_ROW_HEIGHT: usize = 28;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CredentialStorage {
    Open,
    LegacyRamOnly,
    Vault,
}

#[derive(Clone, Copy)]
enum State {
    Idle,
    Starting {
        scan_started: bool,
    },
    Selecting,
    Password {
        network: wifi::ScannedNetwork,
        storage: CredentialStorage,
        remember_for_boot: bool,
        rejected: bool,
    },
    Associating {
        network: wifi::ScannedNetwork,
        storage: CredentialStorage,
        started: bool,
    },
    Configured {
        network: wifi::ScannedNetwork,
        storage: CredentialStorage,
    },
    Failed(&'static str),
}

pub struct GuidedWifi {
    state: State,
    overlay: SecureOverlay,
    connect_saved_after_scan: bool,
}

impl GuidedWifi {
    pub const fn new() -> Self {
        Self {
            state: State::Idle,
            overlay: SecureOverlay::new(),
            connect_saved_after_scan: false,
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, State::Idle)
    }

    pub fn begin(&mut self) -> bool {
        self.connect_saved_after_scan = false;
        self.begin_scan()
    }

    /// Starts one automatic saved-profile attempt only in an ordinary boot.
    /// SAFE requires a separate physical Genesis action and legacy RAM state
    /// can never enter this path.
    pub fn begin_normal_saved_reconnect(&mut self) -> bool {
        if !matches!(
            crate::agent_protocol::boot_control::current_boot_posture(),
            BootPosture::Normal | BootPosture::Probation
        ) || secret_vault::recovery_state() != secret_vault::VaultRecoveryState::Unlocked
            || !matches!(
                secret_vault::wifi_status(),
                secret_vault::VaultSecretStatus::Available { .. }
            )
        {
            self.state = State::Failed("saved_wifi_reconnect_denied");
            return true;
        }
        self.connect_saved_after_scan = true;
        self.begin_scan()
    }

    fn begin_scan(&mut self) -> bool {
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
                network,
                storage,
                remember_for_boot,
                ..
            } if storage == CredentialStorage::LegacyRamOnly => {
                match console::snapshot().wifi_passphrase_entry_result {
                    console::WifiPassphraseEntryResult::Set => {
                        self.state = State::Associating {
                            network,
                            storage,
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
                            network,
                            storage,
                            remember_for_boot,
                            rejected: true,
                        };
                        true
                    }
                    console::WifiPassphraseEntryResult::None => false,
                }
            }
            State::Password { .. } => false,
            State::Associating {
                network,
                storage,
                started,
            } => self.advance_association(network, storage, started),
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
            if self.connect_saved_after_scan {
                self.connect_saved_after_scan = false;
                self.state = match select_saved_scan_target() {
                    Some(network) => State::Associating {
                        network,
                        storage: CredentialStorage::Vault,
                        started: false,
                    },
                    None => State::Failed("saved_wifi_not_visible"),
                };
            } else {
                self.state = State::Selecting;
            }
            true
        } else {
            false
        }
    }

    fn advance_association(
        &mut self,
        network: wifi::ScannedNetwork,
        storage: CredentialStorage,
        started: bool,
    ) -> bool {
        if !started {
            let result = if storage == CredentialStorage::Vault {
                marvell_wifi_pcie::start_association_from_physical_genesis()
            } else {
                marvell_wifi_pcie::start_association()
            };
            console::write_event(format_args!("WIFI ASSOC: {}", result.label()));
            return match result {
                marvell_wifi_pcie::ConnectionTriggerResult::Started
                | marvell_wifi_pcie::ConnectionTriggerResult::AlreadyRunning => {
                    self.state = State::Associating {
                        network,
                        storage,
                        started: true,
                    };
                    true
                }
                marvell_wifi_pcie::ConnectionTriggerResult::AlreadyReady => {
                    self.state = State::Configured { network, storage };
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
            self.state = State::Configured { network, storage };
            true
        } else {
            false
        }
    }

    /// Consumes physical input only for the trusted Vault prompt. Serial input
    /// never reaches this path and remains the explicitly legacy RAM-only path.
    pub fn handle_physical_input(&mut self, event: input::InputEvent) -> bool {
        let State::Password {
            network,
            storage: CredentialStorage::Vault,
            remember_for_boot,
            ..
        } = self.state
        else {
            return false;
        };
        if matches!(event.kind, input::InputEventKind::SecureAttention) {
            self.cancel_vault_password();
            return true;
        }
        let Some(value) = input::event_to_console_input(event) else {
            return true;
        };
        match value {
            input::ConsoleInput::Byte(0x08) | input::ConsoleInput::Byte(0x7f) => {
                let _ = self.overlay.handle(SecureOverlayInput::Backspace);
            }
            input::ConsoleInput::Byte(byte) => {
                let _ = self.overlay.handle(SecureOverlayInput::TextByte(byte));
            }
            input::ConsoleInput::Special(input::SpecialKey::Enter) => {
                self.submit_vault_password(network, remember_for_boot)
            }
            input::ConsoleInput::Special(input::SpecialKey::Escape) => self.cancel_vault_password(),
            input::ConsoleInput::Special(
                input::SpecialKey::Tab
                | input::SpecialKey::BackTab
                | input::SpecialKey::Up
                | input::SpecialKey::Down
                | input::SpecialKey::Left
                | input::SpecialKey::Right,
            ) => {}
        }
        true
    }

    pub fn handle_pointer(&mut self, x: usize, y: usize, width: usize, height: usize) -> bool {
        match self.state {
            State::Idle | State::Starting { .. } | State::Associating { .. } => false,
            State::Selecting => self.select_pointer(x, y, width, height),
            State::Password {
                network,
                storage,
                remember_for_boot,
                rejected,
            } => {
                let rect = password_rect(width, height);
                if storage == CredentialStorage::LegacyRamOnly
                    && point_in(
                        x,
                        y,
                        LogicalRect::new(rect.x + 24, rect.y + 116, rect.w - 48, 20),
                    )
                {
                    self.state = State::Password {
                        network,
                        storage,
                        remember_for_boot: !remember_for_boot,
                        rejected,
                    };
                    wifi::set_remember_passphrase_for_boot(!remember_for_boot);
                    return true;
                }
                let [back, submit] = action_rects(rect);
                if point_in(x, y, back) {
                    if storage == CredentialStorage::Vault {
                        self.cancel_vault_password();
                    } else {
                        let _ = console::cancel_wifi_passphrase_entry();
                        self.state = State::Selecting;
                    }
                    return true;
                }
                if point_in(x, y, submit) {
                    if storage == CredentialStorage::Vault {
                        self.submit_vault_password(network, remember_for_boot);
                        return true;
                    }
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

    fn submit_vault_password(&mut self, network: wifi::ScannedNetwork, remember_for_boot: bool) {
        let selected = wifi::association_target();
        let target_matches = selected.is_some_and(|selected| {
            selected.association_ready()
                && selected.supports_wpa2_psk_ccmp()
                && selected.bssid == network.bssid
                && selected.ssid.as_bytes() == network.ssid.as_bytes()
        });
        if !target_matches {
            self.overlay.clear();
            self.state = State::Failed("selected_bss_changed");
            return;
        }
        let saved = match self.overlay.handle(SecureOverlayInput::Submit) {
            SecureOverlayAction::Submitted(submission)
                if submission.prompt() == SecureOverlayPrompt::WifiPassphrase =>
            {
                secret_vault::save_or_replace_wifi(
                    network.ssid.as_bytes(),
                    network.bssid,
                    submission.into_plaintext_for_broker(),
                )
                .is_ok()
            }
            SecureOverlayAction::Rejected(_) => {
                self.state = State::Password {
                    network,
                    storage: CredentialStorage::Vault,
                    remember_for_boot,
                    rejected: true,
                };
                return;
            }
            _ => false,
        };
        self.overlay.clear();
        if saved {
            wifi::clear_legacy_passphrase();
            serial::write_line("VAULT_WIFI_SECRET_SAVED target=bound_bss");
            self.state = State::Associating {
                network,
                storage: CredentialStorage::Vault,
                started: false,
            };
        } else {
            self.state = State::Failed("vault_wifi_save_rejected");
        }
    }

    fn cancel_vault_password(&mut self) {
        let _ = self.overlay.handle(SecureOverlayInput::Cancel);
        self.overlay.clear();
        self.state = State::Selecting;
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
                    network,
                    storage: CredentialStorage::Open,
                    started: false,
                };
                return true;
            }
            if !network.supports_wpa2_psk_ccmp() {
                self.state = State::Failed("unsupported_wifi_security");
                return true;
            }
            let storage = if secret_vault::recovery_state()
                == secret_vault::VaultRecoveryState::Unlocked
            {
                if matches!(
                    secret_vault::wifi_status(),
                    secret_vault::VaultSecretStatus::Available { .. }
                ) && secret_vault::wifi_target_matches(network.ssid.as_bytes(), network.bssid)
                {
                    self.state = State::Associating {
                        network,
                        storage: CredentialStorage::Vault,
                        started: false,
                    };
                    return true;
                }
                let _ = self.overlay.open(SecureOverlayPrompt::WifiPassphrase);
                CredentialStorage::Vault
            } else {
                match secret_vault::wifi_status() {
                    secret_vault::VaultSecretStatus::Missing => {
                        wifi::set_remember_passphrase_for_boot(true);
                        let _ = console::activate_focus(console::UiFocus::SettingsWifiPassphrase);
                        CredentialStorage::LegacyRamOnly
                    }
                    secret_vault::VaultSecretStatus::Available { .. } => {
                        self.state = State::Failed("unlock_secret_vault_before_wifi");
                        return true;
                    }
                    secret_vault::VaultSecretStatus::Forgotten { .. } => {
                        self.state = State::Failed("wifi_secret_forgotten");
                        return true;
                    }
                }
            };
            self.state = State::Password {
                network,
                storage,
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
                network,
                storage,
                remember_for_boot,
                rejected,
            } => {
                let masked_len = if storage == CredentialStorage::Vault {
                    self.overlay.snapshot().masked_len
                } else {
                    console::snapshot().input.as_str().chars().count()
                };
                draw_password(
                    surface,
                    width,
                    height,
                    network,
                    storage,
                    remember_for_boot,
                    rejected,
                    masked_len,
                )
            }
            State::Associating { .. } => draw_progress(surface, width, height, true),
            State::Configured { network, storage } => {
                draw_configured(surface, width, height, network, storage)
            }
            State::Failed(reason) => draw_failed(surface, width, height, reason),
        }
    }
}

fn select_saved_scan_target() -> Option<wifi::ScannedNetwork> {
    let scan = wifi::scan_results();
    for index in 0..scan.count {
        let network = scan.networks[index];
        if network.hidden_ssid
            || !network.association_ready()
            || !network.supports_wpa2_psk_ccmp()
            || !secret_vault::wifi_target_matches(network.ssid.as_bytes(), network.bssid)
        {
            continue;
        }
        wifi::clear_config();
        if wifi::select_scan_result(index).is_ok() {
            return Some(network);
        }
        return None;
    }
    None
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
    network: wifi::ScannedNetwork,
    storage: CredentialStorage,
    remember_for_boot: bool,
    rejected: bool,
    masked_len: usize,
) {
    let rect = password_rect(width, height);
    draw_panel(surface, rect, "WiFi password");
    draw_truncated_text(
        surface,
        rect.x + 20,
        rect.y + 48,
        network.ssid.as_str(),
        (rect.w - 40) / FONT_ADVANCE,
        TEXT_MAIN,
    );
    surface.fill_rect(rect.x + 20, rect.y + 66, rect.w - 40, 28, SURFACE_ALT);
    draw_outline(
        surface,
        LogicalRect::new(rect.x + 20, rect.y + 66, rect.w - 40, 28),
        APP_BLUE,
    );
    let visible_len = masked_len.min((rect.w - 56) / FONT_ADVANCE);
    for index in 0..visible_len {
        text::draw_text(
            surface,
            rect.x + 28 + index * FONT_ADVANCE,
            rect.y + 76,
            "*",
            TEXT_MAIN,
            None,
        );
    }
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
        if storage == CredentialStorage::Vault {
            "Will be encrypted for this exact access point"
        } else if remember_for_boot {
            "[x] Remember for this boot (LEGACY RAM-ONLY)"
        } else {
            "[ ] Remember for this boot (LEGACY RAM-ONLY)"
        },
        TEXT_MUTED,
        None,
    );
    let [back, submit] = action_rects(rect);
    draw_button(surface, back, "Back", false);
    draw_button(
        surface,
        submit,
        if storage == CredentialStorage::Vault {
            "Save and connect"
        } else {
            "Set credentials"
        },
        true,
    );
}

fn draw_configured(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    network: wifi::ScannedNetwork,
    storage: CredentialStorage,
) {
    let rect = result_rect(width, height);
    draw_panel(surface, rect, "WiFi setup");
    draw_truncated_text(
        surface,
        rect.x + 20,
        rect.y + 50,
        network.ssid.as_str(),
        (rect.w - 40) / FONT_ADVANCE,
        APP_GREEN,
    );
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 70,
        match storage {
            CredentialStorage::Vault => "Credential saved in Secret Vault",
            CredentialStorage::LegacyRamOnly => "Credential ready (LEGACY RAM-ONLY)",
            CredentialStorage::Open => "Open network selected for this boot",
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
        marvell_wifi_pcie::ConnectionStage::SupplicantPmk => (62, "Loading connection credential"),
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
