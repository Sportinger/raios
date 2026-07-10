//! Physical-input-only Genesis flow for Secret Vault recovery and secret entry.

use core::str;

use raios_core::genesis_layout::GenesisLayout;

use crate::{
    framebuffer::{Color, FramebufferSurface},
    input, provider_config, secret_vault,
    secure_overlay::{SecureOverlay, SecureOverlayAction, SecureOverlayInput, SecureOverlayPrompt},
    serial, text, wifi,
};

use super::genesis::{
    draw_button, draw_outline, draw_panel, point_in, rect_from_layout, LogicalRect, APP_GREEN,
    APP_RED, HAIRLINE, SURFACE_ALT, TEXT_MAIN, TEXT_MUTED,
};

const RECOVERY_KEY_TEXT_LEN: usize = 80;
const RECOVERY_KEY_ROW_LEN: usize = 40;

enum Mode {
    Closed,
    Showing(secret_vault::RecoveryKeyDisplay),
    Confirming(secret_vault::RecoveryKeyConfirmation),
    Unlocking,
    Managing(ManageAction),
    ConfirmingForget {
        target: ForgetTarget,
        selected: ConfirmAction,
    },
    ProviderEntry,
    WifiTestEntry,
    Outcome(Outcome),
}

#[derive(Clone, Copy)]
enum Phase {
    Closed,
    Showing,
    Confirming,
    Unlocking,
    Managing,
    ConfirmingForget,
    ProviderEntry,
    WifiTestEntry,
    Outcome,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ManageAction {
    ForgetProvider,
    ForgetWifi,
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfirmAction {
    Cancel,
    Forget,
}

#[derive(Clone, Copy)]
enum ForgetTarget {
    Provider,
    Wifi,
}

#[derive(Clone, Copy)]
enum Outcome {
    Provisioned,
    Unlocked,
    Rejected,
    Unavailable,
    ProviderSaved,
    ProviderRejected,
    ProviderUnavailable,
    WifiTestSaved,
    WifiTestRejected,
    WifiTestUnavailable,
    ProviderForgotten,
    ProviderForgetRejected,
    WifiForgotten,
    WifiForgetRejected,
}

/// Core-owned controller. Its only ingress is `ShellHost` physical input.
pub(crate) struct VaultFlow {
    mode: Mode,
    overlay: SecureOverlay,
    display_origin: Option<(usize, usize)>,
    display_logged: bool,
}

impl VaultFlow {
    pub(crate) const fn new() -> Self {
        Self {
            mode: Mode::Closed,
            overlay: SecureOverlay::new(),
            display_origin: None,
            display_logged: false,
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        !matches!(&self.mode, Mode::Closed)
    }

    /// Starts only after a physical Genesis action. There is deliberately no
    /// serial, command, provider, or Wasm adapter for this operation.
    pub(crate) fn begin_explicit(&mut self) -> bool {
        if self.is_active() {
            return true;
        }
        self.overlay.clear();
        self.display_origin = None;
        self.display_logged = false;
        self.mode = match secret_vault::recovery_state() {
            secret_vault::VaultRecoveryState::ReadyToProvision => {
                match secret_vault::begin_initial_recovery() {
                    Ok(display) => Mode::Showing(display),
                    Err(_) => {
                        serial::write_line("VAULT_RR1_START_REJECTED");
                        Mode::Outcome(Outcome::Rejected)
                    }
                }
            }
            secret_vault::VaultRecoveryState::Locked => {
                let _ = self
                    .overlay
                    .open(SecureOverlayPrompt::RecoveryKeyConfirmation);
                serial::write_line("VAULT_RECOVERY_UNLOCK_READY");
                Mode::Unlocking
            }
            secret_vault::VaultRecoveryState::Unlocked => {
                serial::write_line("VAULT_MANAGE_READY");
                Mode::Managing(ManageAction::Close)
            }
            secret_vault::VaultRecoveryState::Unavailable
            | secret_vault::VaultRecoveryState::AwaitingConfirmation
            | secret_vault::VaultRecoveryState::PersistenceOutcomeUncertain => {
                Mode::Outcome(Outcome::Unavailable)
            }
        };
        true
    }

    /// Opens the persistent provider-key path only from a physical Genesis
    /// action and only while the recovery-unlocked Vault is available.
    pub(crate) fn begin_provider_explicit(&mut self) -> bool {
        if self.is_active() {
            return true;
        }
        self.overlay.clear();
        self.display_origin = None;
        self.display_logged = false;
        if secret_vault::recovery_state() == secret_vault::VaultRecoveryState::Unlocked {
            let _ = self.overlay.open(SecureOverlayPrompt::ProviderApiKey);
            self.mode = Mode::ProviderEntry;
            serial::write_line("VAULT_PROVIDER_SECRET_ENTRY_READY");
        } else {
            self.mode = Mode::Outcome(Outcome::ProviderUnavailable);
            serial::write_line("VAULT_PROVIDER_SECRET_REJECTED");
        }
        true
    }

    /// Opens the physical-input-only credential entry used by the exact C1
    /// QEMU proof. The dispatcher exposes it on no other storage identity.
    pub(crate) fn begin_contained_wifi_explicit(&mut self) -> bool {
        if self.is_active() {
            return true;
        }
        self.overlay.clear();
        self.display_origin = None;
        self.display_logged = false;
        if secret_vault::recovery_state() == secret_vault::VaultRecoveryState::Unlocked
            && secret_vault::contained_qemu_wifi_test_available()
        {
            let _ = self.overlay.open(SecureOverlayPrompt::WifiPassphrase);
            self.mode = Mode::WifiTestEntry;
            serial::write_line(
                "VAULT_WIFI_SECRET_ENTRY_READY target=bound_bss test_infrastructure=true",
            );
        } else {
            self.mode = Mode::Outcome(Outcome::WifiTestUnavailable);
            serial::write_line("VAULT_WIFI_SECRET_REJECTED test_infrastructure=true");
        }
        true
    }

    /// Consumes every physical event while active, including releases and
    /// pointer packets that must not fall through to Console or a guest.
    pub(crate) fn handle_physical_input(&mut self, event: input::InputEvent) -> bool {
        if !self.is_active() {
            return false;
        }
        if matches!(event.kind, input::InputEventKind::SecureAttention) {
            self.cancel();
            return true;
        }

        match self.phase() {
            Phase::Showing => {
                // The Enter that opened Vault can leave trailing USB reports.
                // Requiring a distinct physical gesture prevents it from also
                // dismissing the once-only recovery display.
                if is_pressed_key(event, 57) {
                    self.begin_confirmation();
                } else if is_pressed_key(event, 1) {
                    self.cancel();
                }
            }
            Phase::Confirming | Phase::Unlocking | Phase::ProviderEntry | Phase::WifiTestEntry => {
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
                    input::ConsoleInput::Special(input::SpecialKey::Enter) => self.submit(),
                    input::ConsoleInput::Special(input::SpecialKey::Escape) => self.cancel(),
                    input::ConsoleInput::Special(
                        input::SpecialKey::Tab
                        | input::SpecialKey::BackTab
                        | input::SpecialKey::Up
                        | input::SpecialKey::Down
                        | input::SpecialKey::Left
                        | input::SpecialKey::Right,
                    ) => {}
                }
            }
            Phase::Managing => {
                let Some(value) = input::event_to_console_input(event) else {
                    return true;
                };
                match value {
                    input::ConsoleInput::Special(
                        input::SpecialKey::Tab | input::SpecialKey::Right | input::SpecialKey::Down,
                    ) => self.move_manage_selection(true),
                    input::ConsoleInput::Special(
                        input::SpecialKey::BackTab
                        | input::SpecialKey::Left
                        | input::SpecialKey::Up,
                    ) => self.move_manage_selection(false),
                    input::ConsoleInput::Special(input::SpecialKey::Enter) => {
                        self.activate_manage_selection()
                    }
                    input::ConsoleInput::Special(input::SpecialKey::Escape) => self.cancel(),
                    input::ConsoleInput::Byte(_) => {}
                }
            }
            Phase::ConfirmingForget => {
                let Some(value) = input::event_to_console_input(event) else {
                    return true;
                };
                match value {
                    input::ConsoleInput::Special(
                        input::SpecialKey::Tab
                        | input::SpecialKey::Right
                        | input::SpecialKey::Down
                        | input::SpecialKey::BackTab
                        | input::SpecialKey::Left
                        | input::SpecialKey::Up,
                    ) => self.toggle_confirmation_selection(),
                    input::ConsoleInput::Special(input::SpecialKey::Enter) => {
                        self.activate_confirmation_selection()
                    }
                    input::ConsoleInput::Special(input::SpecialKey::Escape) => {
                        self.return_to_manage()
                    }
                    input::ConsoleInput::Byte(_) => {}
                }
            }
            Phase::Outcome => {
                if is_pressed_key(event, 28) || is_pressed_key(event, 1) {
                    self.close();
                }
            }
            Phase::Closed => {}
        }
        true
    }

    pub(crate) fn handle_pointer(&mut self, x: usize, y: usize, layout: GenesisLayout) -> bool {
        if !self.is_active() {
            return false;
        }
        let rect = rect_from_layout(layout.trusted_overlay);
        match self.phase() {
            Phase::Managing => {
                for (index, action) in manage_action_rects(rect).iter().copied().enumerate() {
                    if point_in(x, y, action) {
                        self.mode = Mode::Managing(match index {
                            0 => ManageAction::ForgetProvider,
                            1 => ManageAction::ForgetWifi,
                            _ => ManageAction::Close,
                        });
                        self.activate_manage_selection();
                        break;
                    }
                }
            }
            Phase::ConfirmingForget => {
                let (cancel, forget) = action_rects(rect);
                if point_in(x, y, cancel) {
                    self.return_to_manage();
                } else if point_in(x, y, forget) {
                    if let Mode::ConfirmingForget { selected, .. } = &mut self.mode {
                        *selected = ConfirmAction::Forget;
                    }
                    self.activate_confirmation_selection();
                }
            }
            phase => {
                let (primary, cancel) = action_rects(rect);
                if point_in(x, y, primary) {
                    match phase {
                        Phase::Showing => self.begin_confirmation(),
                        Phase::Confirming
                        | Phase::Unlocking
                        | Phase::ProviderEntry
                        | Phase::WifiTestEntry => self.submit(),
                        Phase::Outcome => self.close(),
                        Phase::Closed | Phase::Managing | Phase::ConfirmingForget => {}
                    }
                } else if point_in(x, y, cancel) {
                    self.cancel();
                }
            }
        }
        true
    }

    pub(crate) fn draw(&mut self, surface: &mut FramebufferSurface, layout: GenesisLayout) {
        if !self.is_active() {
            return;
        }
        let personal = rect_from_layout(layout.personal_surface);
        surface.fill_rect(
            personal.x,
            personal.y,
            personal.w,
            personal.h,
            Color::new(12, 14, 18),
        );
        let rect = rect_from_layout(layout.trusted_overlay);
        match &self.mode {
            Mode::Showing(display) => {
                self.display_origin = draw_display(surface, rect, display);
            }
            Mode::Confirming(_) => self.draw_entry(surface, rect, false),
            Mode::Unlocking => self.draw_entry(surface, rect, true),
            Mode::Managing(selected) => draw_manage(surface, rect, *selected),
            Mode::ConfirmingForget { target, selected } => {
                draw_forget_confirmation(surface, rect, *target, *selected)
            }
            Mode::ProviderEntry => self.draw_provider_entry(surface, rect),
            Mode::WifiTestEntry => self.draw_wifi_test_entry(surface, rect),
            Mode::Outcome(outcome) => draw_outcome(surface, rect, *outcome),
            Mode::Closed => {}
        }
    }

    /// Called immediately after the framebuffer present. This makes the marker
    /// evidence that the complete two-row key is already visible.
    pub(crate) fn note_presented(&mut self) {
        if self.display_logged || !matches!(&self.mode, Mode::Showing(_)) {
            return;
        }
        let Some((x, y)) = self.display_origin else {
            return;
        };
        serial::write_fmt(format_args!(
            "VAULT_RR1_DISPLAY_READY layout=v1 x={} y={} scale=2 rows=2 cols=40\r\n",
            x, y
        ));
        self.display_logged = true;
    }

    pub(crate) fn status_text() -> &'static str {
        match secret_vault::recovery_state() {
            secret_vault::VaultRecoveryState::Unavailable => "Unavailable",
            secret_vault::VaultRecoveryState::ReadyToProvision => "Not configured",
            secret_vault::VaultRecoveryState::AwaitingConfirmation => "Confirming setup",
            secret_vault::VaultRecoveryState::PersistenceOutcomeUncertain => {
                "Recovery check required"
            }
            secret_vault::VaultRecoveryState::Locked => "Locked",
            secret_vault::VaultRecoveryState::Unlocked => "Ready",
        }
    }

    pub(crate) fn action_label(&self) -> &'static str {
        match secret_vault::recovery_state() {
            secret_vault::VaultRecoveryState::ReadyToProvision => "Set up Secret Vault",
            secret_vault::VaultRecoveryState::Locked => "Unlock Secret Vault",
            secret_vault::VaultRecoveryState::Unlocked => "Manage Secret Vault",
            secret_vault::VaultRecoveryState::Unavailable
            | secret_vault::VaultRecoveryState::AwaitingConfirmation
            | secret_vault::VaultRecoveryState::PersistenceOutcomeUncertain => {
                "Secret Vault unavailable"
            }
        }
    }

    pub(crate) fn provider_status_text(&self) -> &'static str {
        if secret_vault::recovery_state() != secret_vault::VaultRecoveryState::Unlocked {
            return "API key: unlock Secret Vault before saving";
        }
        match secret_vault::provider_status() {
            secret_vault::VaultSecretStatus::Available { .. } => "API key: saved in Secret Vault",
            secret_vault::VaultSecretStatus::Missing
            | secret_vault::VaultSecretStatus::Forgotten { .. } => {
                "API key: not saved in Secret Vault"
            }
        }
    }

    pub(crate) fn provider_action_label(&self) -> &'static str {
        if secret_vault::recovery_state() != secret_vault::VaultRecoveryState::Unlocked {
            return "Unlock Vault first";
        }
        match secret_vault::provider_status() {
            secret_vault::VaultSecretStatus::Available { .. } => "Replace API key",
            secret_vault::VaultSecretStatus::Missing
            | secret_vault::VaultSecretStatus::Forgotten { .. } => "Save API key",
        }
    }

    fn begin_confirmation(&mut self) {
        let previous = core::mem::replace(&mut self.mode, Mode::Closed);
        match previous {
            Mode::Showing(display) => {
                let confirmation = display.begin_confirmation();
                let _ = self
                    .overlay
                    .open(SecureOverlayPrompt::RecoveryKeyConfirmation);
                self.display_origin = None;
                self.mode = Mode::Confirming(confirmation);
                serial::write_line("VAULT_RR1_CONFIRMATION_READY");
            }
            other => self.mode = other,
        }
    }

    fn submit(&mut self) {
        if matches!(&self.mode, Mode::ProviderEntry) {
            self.submit_provider();
            return;
        }
        if matches!(&self.mode, Mode::WifiTestEntry) {
            self.submit_contained_wifi();
            return;
        }
        let action = self.overlay.handle(SecureOverlayInput::Submit);
        let input = match action {
            SecureOverlayAction::SubmittedRecovery(input) => input,
            SecureOverlayAction::Rejected(_) => {
                serial::write_fmt(format_args!(
                    "VAULT_RR1_INPUT_REJECTED reason=input_shape length={}\r\n",
                    self.overlay.snapshot().masked_len
                ));
                return;
            }
            _ => return,
        };

        let previous = core::mem::replace(&mut self.mode, Mode::Closed);
        let unlocking = previous_is_unlocking(&previous);
        let result = match previous {
            Mode::Confirming(confirmation) => input.confirm_initial(confirmation),
            Mode::Unlocking => input.unlock(),
            other => {
                self.mode = other;
                return;
            }
        };
        self.overlay.clear();
        self.display_origin = None;
        self.display_logged = false;
        match (unlocking, result) {
            (false, Ok(_)) => {
                serial::write_line("VAULT_RR1_CONFIRMED");
                self.mode = Mode::Outcome(Outcome::Provisioned);
            }
            (true, Ok(_)) => {
                if matches!(
                    secret_vault::provider_status(),
                    secret_vault::VaultSecretStatus::Available { .. }
                ) {
                    if let Err(error) = secret_vault::run_contained_qemu_provider_consumer_test() {
                        serial::write_fmt(format_args!(
                            "VAULT_PROVIDER_CONTAINED_REJECTED reason={} test_infrastructure=true\r\n",
                            error.recovery_reason()
                        ));
                        self.mode = Mode::Outcome(Outcome::ProviderRejected);
                        return;
                    }
                }
                if matches!(
                    secret_vault::wifi_status(),
                    secret_vault::VaultSecretStatus::Available { .. }
                ) {
                    if let Err(error) = secret_vault::run_contained_qemu_wifi_consumer_test() {
                        serial::write_fmt(format_args!(
                            "VAULT_WIFI_CONTAINED_REJECTED reason={} test_infrastructure=true\r\n",
                            error.wifi_use_reason()
                        ));
                        self.mode = Mode::Outcome(Outcome::WifiTestRejected);
                        return;
                    }
                }
                if matches!(
                    secret_vault::provider_status(),
                    secret_vault::VaultSecretStatus::Forgotten { .. }
                ) && matches!(
                    secret_vault::wifi_status(),
                    secret_vault::VaultSecretStatus::Forgotten { .. }
                ) && secret_vault::contained_qemu_wifi_test_available()
                {
                    if let Err(error) = secret_vault::run_contained_qemu_forgotten_denial_test() {
                        serial::write_fmt(format_args!(
                            "VAULT_FORGOTTEN_DENIAL_TEST_REJECTED reason={} test_infrastructure=true\r\n",
                            error.recovery_reason()
                        ));
                        self.mode = Mode::Outcome(Outcome::Rejected);
                        return;
                    }
                }
                serial::write_line("VAULT_RR1_UNLOCKED");
                self.mode = Mode::Outcome(Outcome::Unlocked);
            }
            (false, Err(error)) => {
                serial::write_fmt(format_args!(
                    "VAULT_RR1_REJECTED reason={}\r\n",
                    error.recovery_reason()
                ));
                self.mode = Mode::Outcome(Outcome::Rejected);
            }
            (true, Err(error)) => {
                serial::write_fmt(format_args!(
                    "VAULT_RR1_UNLOCK_REJECTED reason={}\r\n",
                    error.recovery_reason()
                ));
                self.mode = Mode::Outcome(Outcome::Rejected);
            }
        }
    }

    fn submit_provider(&mut self) {
        let saved = match self.overlay.handle(SecureOverlayInput::Submit) {
            SecureOverlayAction::Submitted(submission)
                if submission.prompt() == SecureOverlayPrompt::ProviderApiKey =>
            {
                secret_vault::save_or_replace_provider(submission.into_plaintext_for_broker())
                    .is_ok()
            }
            _ => false,
        };
        self.overlay.clear();
        self.display_origin = None;
        self.display_logged = false;
        if saved {
            provider_config::clear_api_key();
            serial::write_line("VAULT_PROVIDER_SECRET_SAVED");
            self.mode = Mode::Outcome(Outcome::ProviderSaved);
        } else {
            serial::write_line("VAULT_PROVIDER_SECRET_REJECTED");
            self.mode = Mode::Outcome(Outcome::ProviderRejected);
        }
    }

    fn submit_contained_wifi(&mut self) {
        let saved = match self.overlay.handle(SecureOverlayInput::Submit) {
            SecureOverlayAction::Submitted(submission)
                if submission.prompt() == SecureOverlayPrompt::WifiPassphrase =>
            {
                secret_vault::save_or_replace_contained_qemu_wifi_for_test(
                    submission.into_plaintext_for_broker(),
                )
                .is_ok()
            }
            _ => false,
        };
        self.overlay.clear();
        self.display_origin = None;
        self.display_logged = false;
        if saved {
            wifi::clear_legacy_passphrase();
            serial::write_line("VAULT_WIFI_SECRET_SAVED target=bound_bss test_infrastructure=true");
            self.mode = Mode::Outcome(Outcome::WifiTestSaved);
        } else {
            serial::write_line("VAULT_WIFI_SECRET_REJECTED test_infrastructure=true");
            self.mode = Mode::Outcome(Outcome::WifiTestRejected);
        }
    }

    fn move_manage_selection(&mut self, forward: bool) {
        if let Mode::Managing(selected) = &mut self.mode {
            *selected = match (*selected, forward) {
                (ManageAction::ForgetProvider, true) | (ManageAction::Close, false) => {
                    ManageAction::ForgetWifi
                }
                (ManageAction::ForgetWifi, true) => ManageAction::Close,
                (ManageAction::ForgetWifi, false) => ManageAction::ForgetProvider,
                (ManageAction::Close, true) => ManageAction::ForgetProvider,
                (ManageAction::ForgetProvider, false) => ManageAction::Close,
            };
        }
    }

    fn activate_manage_selection(&mut self) {
        let selected = match &self.mode {
            Mode::Managing(selected) => *selected,
            _ => return,
        };
        match selected {
            ManageAction::ForgetProvider => {
                serial::write_line("VAULT_PROVIDER_FORGET_CONFIRM_READY");
                self.mode = Mode::ConfirmingForget {
                    target: ForgetTarget::Provider,
                    selected: ConfirmAction::Cancel,
                };
            }
            ManageAction::ForgetWifi => {
                serial::write_line("VAULT_WIFI_FORGET_CONFIRM_READY");
                self.mode = Mode::ConfirmingForget {
                    target: ForgetTarget::Wifi,
                    selected: ConfirmAction::Cancel,
                };
            }
            ManageAction::Close => self.close(),
        }
    }

    fn toggle_confirmation_selection(&mut self) {
        if let Mode::ConfirmingForget { selected, .. } = &mut self.mode {
            *selected = match selected {
                ConfirmAction::Cancel => ConfirmAction::Forget,
                ConfirmAction::Forget => ConfirmAction::Cancel,
            };
        }
    }

    fn activate_confirmation_selection(&mut self) {
        let (target, selected) = match &self.mode {
            Mode::ConfirmingForget { target, selected } => (*target, *selected),
            _ => return,
        };
        if selected == ConfirmAction::Cancel {
            self.return_to_manage();
            return;
        }
        match target {
            ForgetTarget::Provider => match secret_vault::forget_provider() {
                Ok(_) => {
                    provider_config::clear_api_key();
                    self.mode = Mode::Outcome(Outcome::ProviderForgotten);
                }
                Err(error) => {
                    serial::write_fmt(format_args!(
                        "VAULT_PROVIDER_FORGET_REJECTED reason={}\r\n",
                        error.recovery_reason()
                    ));
                    self.mode = Mode::Outcome(Outcome::ProviderForgetRejected);
                }
            },
            ForgetTarget::Wifi => match secret_vault::forget_wifi() {
                Ok(_) => {
                    wifi::clear_legacy_passphrase();
                    self.mode = Mode::Outcome(Outcome::WifiForgotten);
                }
                Err(error) => {
                    serial::write_fmt(format_args!(
                        "VAULT_WIFI_FORGET_REJECTED reason={}\r\n",
                        error.recovery_reason()
                    ));
                    self.mode = Mode::Outcome(Outcome::WifiForgetRejected);
                }
            },
        }
    }

    fn return_to_manage(&mut self) {
        let selected = match &self.mode {
            Mode::ConfirmingForget {
                target: ForgetTarget::Provider,
                ..
            } => ManageAction::ForgetProvider,
            Mode::ConfirmingForget {
                target: ForgetTarget::Wifi,
                ..
            } => ManageAction::ForgetWifi,
            _ => return,
        };
        self.mode = Mode::Managing(selected);
        serial::write_line("VAULT_MANAGE_READY");
    }

    fn draw_entry(&self, surface: &mut FramebufferSurface, rect: LogicalRect, unlock: bool) {
        draw_panel(
            surface,
            rect,
            if unlock {
                "Unlock Secret Vault"
            } else {
                "Confirm recovery key"
            },
        );
        text::draw_text(
            surface,
            rect.x + 20,
            rect.y + 44,
            if unlock {
                "Enter your complete RR1 recovery key."
            } else {
                "Re-enter the complete key to finish setup."
            },
            TEXT_MUTED,
            None,
        );
        let field = LogicalRect::new(rect.x + 20, rect.y + 70, rect.w.saturating_sub(40), 52);
        surface.fill_rect(field.x, field.y, field.w, field.h, SURFACE_ALT);
        draw_outline(surface, field, HAIRLINE);
        draw_masked(
            surface,
            field.x + 10,
            field.y + 12,
            self.overlay.snapshot().masked_len,
        );
        let (primary, cancel) = action_rects(rect);
        draw_button(surface, primary, "Confirm", true);
        draw_button(surface, cancel, "Cancel", false);
    }

    fn draw_provider_entry(&self, surface: &mut FramebufferSurface, rect: LogicalRect) {
        draw_panel(surface, rect, "Save provider API key");
        text::draw_text(
            surface,
            rect.x + 20,
            rect.y + 44,
            "Stored encrypted in the unlocked Secret Vault.",
            TEXT_MUTED,
            None,
        );
        let field = LogicalRect::new(rect.x + 20, rect.y + 70, rect.w.saturating_sub(40), 52);
        surface.fill_rect(field.x, field.y, field.w, field.h, SURFACE_ALT);
        draw_outline(surface, field, HAIRLINE);
        draw_masked(
            surface,
            field.x + 10,
            field.y + 12,
            self.overlay.snapshot().masked_len,
        );
        let (primary, cancel) = action_rects(rect);
        draw_button(surface, primary, "Save", true);
        draw_button(surface, cancel, "Cancel", false);
    }

    fn draw_wifi_test_entry(&self, surface: &mut FramebufferSurface, rect: LogicalRect) {
        draw_panel(surface, rect, "Contained WiFi Vault proof");
        text::draw_text(
            surface,
            rect.x + 20,
            rect.y + 44,
            "C1/QEMU test BSS only. No radio connection is claimed.",
            TEXT_MUTED,
            None,
        );
        let field = LogicalRect::new(rect.x + 20, rect.y + 70, rect.w.saturating_sub(40), 52);
        surface.fill_rect(field.x, field.y, field.w, field.h, SURFACE_ALT);
        draw_outline(surface, field, HAIRLINE);
        draw_masked(
            surface,
            field.x + 10,
            field.y + 12,
            self.overlay.snapshot().masked_len,
        );
        let (primary, cancel) = action_rects(rect);
        draw_button(surface, primary, "Save test credential", true);
        draw_button(surface, cancel, "Cancel", false);
    }

    fn cancel(&mut self) {
        if !self.is_active() {
            return;
        }
        let marker = match &self.mode {
            Mode::ProviderEntry
            | Mode::Outcome(
                Outcome::ProviderSaved | Outcome::ProviderRejected | Outcome::ProviderUnavailable,
            ) => "VAULT_PROVIDER_SECRET_CANCELLED",
            Mode::WifiTestEntry
            | Mode::Outcome(
                Outcome::WifiTestSaved | Outcome::WifiTestRejected | Outcome::WifiTestUnavailable,
            ) => "VAULT_WIFI_SECRET_CANCELLED test_infrastructure=true",
            Mode::Managing(_)
            | Mode::ConfirmingForget { .. }
            | Mode::Outcome(
                Outcome::ProviderForgotten
                | Outcome::ProviderForgetRejected
                | Outcome::WifiForgotten
                | Outcome::WifiForgetRejected,
            ) => "VAULT_MANAGE_CANCELLED",
            _ => "VAULT_RR1_CANCELLED",
        };
        self.close();
        serial::write_line(marker);
    }

    fn close(&mut self) {
        self.overlay.clear();
        self.mode = Mode::Closed;
        self.display_origin = None;
        self.display_logged = false;
    }

    fn phase(&self) -> Phase {
        match &self.mode {
            Mode::Closed => Phase::Closed,
            Mode::Showing(_) => Phase::Showing,
            Mode::Confirming(_) => Phase::Confirming,
            Mode::Unlocking => Phase::Unlocking,
            Mode::Managing(_) => Phase::Managing,
            Mode::ConfirmingForget { .. } => Phase::ConfirmingForget,
            Mode::ProviderEntry => Phase::ProviderEntry,
            Mode::WifiTestEntry => Phase::WifiTestEntry,
            Mode::Outcome(_) => Phase::Outcome,
        }
    }
}

impl Drop for VaultFlow {
    fn drop(&mut self) {
        self.close();
    }
}

fn draw_masked(surface: &mut FramebufferSurface, x: usize, y: usize, len: usize) {
    let mut first = [b' '; RECOVERY_KEY_ROW_LEN];
    let mut second = [b' '; RECOVERY_KEY_ROW_LEN];
    first[..len.min(RECOVERY_KEY_ROW_LEN)].fill(b'*');
    if len > RECOVERY_KEY_ROW_LEN {
        second[..(len - RECOVERY_KEY_ROW_LEN).min(RECOVERY_KEY_ROW_LEN)].fill(b'*');
    }
    // Both arrays are fixed ASCII by construction.
    if let (Ok(first), Ok(second)) = (str::from_utf8(&first), str::from_utf8(&second)) {
        text::draw_text(surface, x, y, first, TEXT_MAIN, None);
        text::draw_text(surface, x, y + 16, second, TEXT_MAIN, None);
    }
}

fn draw_display(
    surface: &mut FramebufferSurface,
    rect: LogicalRect,
    display: &secret_vault::RecoveryKeyDisplay,
) -> Option<(usize, usize)> {
    draw_panel(surface, rect, "Secret Vault recovery key");
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 44,
        "Write this key down. It is shown only once.",
        TEXT_MUTED,
        None,
    );
    let x = rect.x + 20;
    let y = rect.y + 76;
    let bytes = display.as_bytes();
    let origin = if let (Ok(first), Ok(second)) = (
        str::from_utf8(&bytes[..RECOVERY_KEY_ROW_LEN]),
        str::from_utf8(&bytes[RECOVERY_KEY_ROW_LEN..RECOVERY_KEY_TEXT_LEN]),
    ) {
        text::draw_text(surface, x, y, first, TEXT_MAIN, None);
        text::draw_text(surface, x, y + 16, second, TEXT_MAIN, None);
        Some((x, y))
    } else {
        None
    };
    let (primary, cancel) = action_rects(rect);
    draw_button(surface, primary, "Space: I saved it", true);
    draw_button(surface, cancel, "Cancel", false);
    origin
}

fn draw_manage(surface: &mut FramebufferSurface, rect: LogicalRect, selected: ManageAction) {
    draw_panel(surface, rect, "Manage Secret Vault");
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 44,
        match secret_vault::provider_status() {
            secret_vault::VaultSecretStatus::Available { .. } => "Provider credential: saved",
            secret_vault::VaultSecretStatus::Missing => "Provider credential: not saved",
            secret_vault::VaultSecretStatus::Forgotten { .. } => "Provider credential: forgotten",
        },
        TEXT_MAIN,
        None,
    );
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 62,
        match secret_vault::wifi_status() {
            secret_vault::VaultSecretStatus::Available { .. } => "WiFi credential: saved",
            secret_vault::VaultSecretStatus::Missing => "WiFi credential: not saved",
            secret_vault::VaultSecretStatus::Forgotten { .. } => "WiFi credential: forgotten",
        },
        TEXT_MAIN,
        None,
    );
    let actions = manage_action_rects(rect);
    draw_button(
        surface,
        actions[0],
        "Forget provider credential",
        selected == ManageAction::ForgetProvider,
    );
    draw_button(
        surface,
        actions[1],
        "Forget WiFi credential",
        selected == ManageAction::ForgetWifi,
    );
    draw_button(
        surface,
        actions[2],
        "Close",
        selected == ManageAction::Close,
    );
}

fn draw_forget_confirmation(
    surface: &mut FramebufferSurface,
    rect: LogicalRect,
    target: ForgetTarget,
    selected: ConfirmAction,
) {
    let (title, message, action) = match target {
        ForgetTarget::Provider => (
            "Forget provider credential?",
            "Future provider use will be denied until a new key is saved.",
            "Forget provider",
        ),
        ForgetTarget::Wifi => (
            "Forget WiFi credential?",
            "Future WiFi use will be denied until a new credential is saved.",
            "Forget WiFi",
        ),
    };
    draw_panel(surface, rect, title);
    text::draw_text(surface, rect.x + 20, rect.y + 54, message, APP_RED, None);
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 76,
        "This action requires this second explicit confirmation.",
        TEXT_MUTED,
        None,
    );
    let (cancel, forget) = action_rects(rect);
    draw_button(surface, cancel, "Cancel", selected == ConfirmAction::Cancel);
    draw_button(surface, forget, action, selected == ConfirmAction::Forget);
}

fn draw_outcome(surface: &mut FramebufferSurface, rect: LogicalRect, outcome: Outcome) {
    draw_panel(surface, rect, "Secret Vault");
    let (message, color) = match outcome {
        Outcome::Provisioned => ("Recovery key confirmed. Vault is ready.", APP_GREEN),
        Outcome::Unlocked => ("Secret Vault is unlocked.", APP_GREEN),
        Outcome::Rejected => ("Recovery key rejected. No access was granted.", APP_RED),
        Outcome::Unavailable => ("Secret Vault is not available for this boot.", APP_RED),
        Outcome::ProviderSaved => ("Provider API key saved in Secret Vault.", APP_GREEN),
        Outcome::ProviderRejected => ("Provider API key was not saved.", APP_RED),
        Outcome::ProviderUnavailable => ("Unlock Secret Vault before saving an API key.", APP_RED),
        Outcome::WifiTestSaved => (
            "Contained WiFi credential saved in Secret Vault.",
            APP_GREEN,
        ),
        Outcome::WifiTestRejected => ("Contained WiFi Vault proof was denied.", APP_RED),
        Outcome::WifiTestUnavailable => ("Contained WiFi proof is unavailable here.", APP_RED),
        Outcome::ProviderForgotten => ("Provider credential forgotten.", APP_GREEN),
        Outcome::ProviderForgetRejected => ("Provider credential was not forgotten.", APP_RED),
        Outcome::WifiForgotten => ("WiFi credential forgotten.", APP_GREEN),
        Outcome::WifiForgetRejected => ("WiFi credential was not forgotten.", APP_RED),
    };
    text::draw_text(surface, rect.x + 20, rect.y + 64, message, color, None);
    let (primary, _) = action_rects(rect);
    draw_button(surface, primary, "Close", true);
}

fn manage_action_rects(rect: LogicalRect) -> [LogicalRect; 3] {
    let x = rect.x + 20;
    let y = rect.y + 92;
    let width = rect.w.saturating_sub(40);
    [
        LogicalRect::new(x, y, width, 24),
        LogicalRect::new(x, y + 32, width, 24),
        LogicalRect::new(x, y + 64, width, 24),
    ]
}

fn action_rects(rect: LogicalRect) -> (LogicalRect, LogicalRect) {
    let width = rect.w.saturating_sub(52) / 2;
    let y = rect.y + rect.h.saturating_sub(44);
    (
        LogicalRect::new(rect.x + 20, y, width, 24),
        LogicalRect::new(rect.x + 32 + width, y, width, 24),
    )
}

fn is_pressed_key(event: input::InputEvent, code: u16) -> bool {
    matches!(
        event.kind,
        input::InputEventKind::Key {
            code: event_code,
            pressed: true
        } if event_code == code
    )
}

fn previous_is_unlocking(mode: &Mode) -> bool {
    matches!(mode, Mode::Unlocking)
}
