//! Physical-input-only Genesis flow for initial Secret Vault recovery setup.

use core::str;

use raios_core::genesis_layout::GenesisLayout;

use crate::{
    framebuffer::{Color, FramebufferSurface},
    input, secret_vault,
    secure_overlay::{SecureOverlay, SecureOverlayAction, SecureOverlayInput, SecureOverlayPrompt},
    serial, text,
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
    Outcome(Outcome),
}

#[derive(Clone, Copy)]
enum Phase {
    Closed,
    Showing,
    Confirming,
    Unlocking,
    Outcome,
}

#[derive(Clone, Copy)]
enum Outcome {
    Provisioned,
    Unlocked,
    Rejected,
    Unavailable,
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
            secret_vault::VaultRecoveryState::Unlocked => Mode::Outcome(Outcome::Unlocked),
            secret_vault::VaultRecoveryState::Unavailable
            | secret_vault::VaultRecoveryState::AwaitingConfirmation
            | secret_vault::VaultRecoveryState::PersistenceOutcomeUncertain => {
                Mode::Outcome(Outcome::Unavailable)
            }
        };
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
            Phase::Confirming | Phase::Unlocking => {
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
        let (primary, cancel) = action_rects(rect_from_layout(layout.trusted_overlay));
        if point_in(x, y, primary) {
            match self.phase() {
                Phase::Showing => self.begin_confirmation(),
                Phase::Confirming | Phase::Unlocking => self.submit(),
                Phase::Outcome => self.close(),
                Phase::Closed => {}
            }
        } else if point_in(x, y, cancel) {
            self.cancel();
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
            secret_vault::VaultRecoveryState::Unlocked => "Secret Vault ready",
            secret_vault::VaultRecoveryState::Unavailable
            | secret_vault::VaultRecoveryState::AwaitingConfirmation
            | secret_vault::VaultRecoveryState::PersistenceOutcomeUncertain => {
                "Secret Vault unavailable"
            }
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

    fn cancel(&mut self) {
        if !self.is_active() {
            return;
        }
        self.close();
        serial::write_line("VAULT_RR1_CANCELLED");
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

fn draw_outcome(surface: &mut FramebufferSurface, rect: LogicalRect, outcome: Outcome) {
    draw_panel(surface, rect, "Secret Vault");
    let (message, color) = match outcome {
        Outcome::Provisioned => ("Recovery key confirmed. Vault is ready.", APP_GREEN),
        Outcome::Unlocked => ("Secret Vault is unlocked.", APP_GREEN),
        Outcome::Rejected => ("Recovery key rejected. No access was granted.", APP_RED),
        Outcome::Unavailable => ("Secret Vault is not available for this boot.", APP_RED),
    };
    text::draw_text(surface, rect.x + 20, rect.y + 64, message, color, None);
    let (primary, _) = action_rects(rect);
    draw_button(surface, primary, "Close", true);
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
