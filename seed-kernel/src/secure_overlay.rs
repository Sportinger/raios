//! Core-owned masked secret-entry state for trusted Genesis overlays.
//!
//! This is deliberately only an input adapter. It owns neither provider nor
//! WiFi configuration and never returns secret bytes. A successful submit
//! transfers the opaque `SecretPlaintext` to the exact broker join.

use core::sync::atomic::{compiler_fence, Ordering};

use raios_core::secret_vault::{
    SecretKind, SecretPlaintext, SecretVaultError, MAX_PROVIDER_API_KEY_LEN,
    MAX_WIFI_PASSPHRASE_LEN,
};

const MIN_WIFI_PASSPHRASE_LEN: usize = 8;

/// The only trusted secret-entry prompts in Genesis V1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SecureOverlayPrompt {
    ProviderApiKey,
    WifiPassphrase,
}

impl SecureOverlayPrompt {
    pub(crate) const fn secret_kind(self) -> SecretKind {
        match self {
            Self::ProviderApiKey => SecretKind::ProviderApiKey,
            Self::WifiPassphrase => SecretKind::WifiPassphrase,
        }
    }

    const fn capacity(self) -> usize {
        match self {
            Self::ProviderApiKey => MAX_PROVIDER_API_KEY_LEN,
            Self::WifiPassphrase => MAX_WIFI_PASSPHRASE_LEN,
        }
    }

    const fn accepts(self, byte: u8) -> bool {
        match self {
            Self::ProviderApiKey => byte.is_ascii_graphic(),
            Self::WifiPassphrase => byte >= 0x20 && byte <= 0x7e,
        }
    }
}

/// A user input event after the Genesis input adapter has recognized it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SecureOverlayInput {
    TextByte(u8),
    Backspace,
    Submit,
    Cancel,
}

/// Non-secret state safe for trusted overlay rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SecureOverlaySnapshot {
    pub prompt: Option<SecureOverlayPrompt>,
    pub masked_len: usize,
}

/// Explicit reasons a trusted overlay did not advance its secret operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SecureOverlayRejection {
    InvalidByte,
    CapacityReached,
    Empty,
    WifiPassphraseTooShort,
    PlaintextRejected,
}

/// Opaque successful submission for the future exact Secret Broker join.
///
/// There is intentionally no byte accessor. The broker receives the owned
/// `SecretPlaintext`, whose bytes are private to `raios-core::secret_vault`.
pub(crate) struct SecureSecretSubmission {
    prompt: SecureOverlayPrompt,
    secret: SecretPlaintext,
}

impl SecureSecretSubmission {
    pub(crate) const fn prompt(&self) -> SecureOverlayPrompt {
        self.prompt
    }

    pub(crate) const fn secret_kind(&self) -> SecretKind {
        self.secret.kind()
    }

    pub(crate) const fn len(&self) -> usize {
        self.secret.len()
    }

    /// Transfers the opaque plaintext only to the exact core broker adapter.
    pub(crate) fn into_plaintext_for_broker(self) -> SecretPlaintext {
        self.secret
    }
}

/// Result of an explicit trusted-overlay transition.
#[must_use]
pub(crate) enum SecureOverlayAction {
    Opened(SecureOverlaySnapshot),
    Changed(SecureOverlaySnapshot),
    Cancelled(SecureOverlayPrompt),
    Submitted(SecureSecretSubmission),
    Rejected(SecureOverlayRejection),
    Inactive,
}

/// Fixed-capacity core-owned input buffer for one trusted prompt at a time.
pub(crate) struct SecureOverlay {
    prompt: Option<SecureOverlayPrompt>,
    bytes: [u8; MAX_PROVIDER_API_KEY_LEN],
    len: usize,
}

impl SecureOverlay {
    pub(crate) const fn new() -> Self {
        Self {
            prompt: None,
            bytes: [0; MAX_PROVIDER_API_KEY_LEN],
            len: 0,
        }
    }

    pub(crate) fn snapshot(&self) -> SecureOverlaySnapshot {
        SecureOverlaySnapshot {
            prompt: self.prompt,
            masked_len: self.len,
        }
    }

    pub(crate) fn open(&mut self, prompt: SecureOverlayPrompt) -> SecureOverlayAction {
        self.clear();
        self.prompt = Some(prompt);
        SecureOverlayAction::Opened(self.snapshot())
    }

    pub(crate) fn handle(&mut self, input: SecureOverlayInput) -> SecureOverlayAction {
        let Some(prompt) = self.prompt else {
            return SecureOverlayAction::Inactive;
        };

        match input {
            SecureOverlayInput::TextByte(byte) => self.push_byte(prompt, byte),
            SecureOverlayInput::Backspace => self.backspace(),
            SecureOverlayInput::Submit => self.submit(prompt),
            SecureOverlayInput::Cancel => {
                self.clear();
                SecureOverlayAction::Cancelled(prompt)
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        erase(&mut self.bytes);
        self.len = 0;
        self.prompt = None;
    }

    fn push_byte(&mut self, prompt: SecureOverlayPrompt, byte: u8) -> SecureOverlayAction {
        if !prompt.accepts(byte) {
            return SecureOverlayAction::Rejected(SecureOverlayRejection::InvalidByte);
        }
        if self.len == prompt.capacity() {
            return SecureOverlayAction::Rejected(SecureOverlayRejection::CapacityReached);
        }

        self.bytes[self.len] = byte;
        self.len += 1;
        SecureOverlayAction::Changed(self.snapshot())
    }

    fn backspace(&mut self) -> SecureOverlayAction {
        if self.len == 0 {
            return SecureOverlayAction::Changed(self.snapshot());
        }

        self.len -= 1;
        erase(&mut self.bytes[self.len..=self.len]);
        SecureOverlayAction::Changed(self.snapshot())
    }

    fn submit(&mut self, prompt: SecureOverlayPrompt) -> SecureOverlayAction {
        if self.len == 0 {
            return SecureOverlayAction::Rejected(SecureOverlayRejection::Empty);
        }
        if prompt == SecureOverlayPrompt::WifiPassphrase && self.len < MIN_WIFI_PASSPHRASE_LEN {
            return SecureOverlayAction::Rejected(SecureOverlayRejection::WifiPassphraseTooShort);
        }

        let len = self.len;
        let mut staged = [0u8; MAX_PROVIDER_API_KEY_LEN];
        staged[..len].copy_from_slice(&self.bytes[..len]);
        self.clear();

        let secret = match SecretPlaintext::take_from(prompt.secret_kind(), &mut staged[..len]) {
            Ok(secret) => secret,
            Err(error) => {
                erase(&mut staged);
                return SecureOverlayAction::Rejected(match error {
                    SecretVaultError::EmptySecret => SecureOverlayRejection::Empty,
                    SecretVaultError::SecretTooLong { .. } => {
                        SecureOverlayRejection::CapacityReached
                    }
                    _ => SecureOverlayRejection::PlaintextRejected,
                });
            }
        };
        erase(&mut staged);
        SecureOverlayAction::Submitted(SecureSecretSubmission { prompt, secret })
    }
}

impl Drop for SecureOverlay {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Uses volatile writes so clearing a secret-bearing buffer is not optimized
/// away by a release kernel build.
fn erase(bytes: &mut [u8]) {
    for byte in bytes {
        // SAFETY: `byte` is a unique mutable reference into the caller-owned buffer.
        unsafe { core::ptr::write_volatile(byte, 0) };
    }
    compiler_fence(Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::{
        SecureOverlay, SecureOverlayAction, SecureOverlayInput, SecureOverlayPrompt,
        SecureOverlayRejection,
    };
    use raios_core::secret_vault::{SecretKind, MAX_PROVIDER_API_KEY_LEN, MAX_WIFI_PASSPHRASE_LEN};

    #[test]
    fn provider_entry_is_masked_and_transfers_only_opaque_plaintext() {
        let mut overlay = SecureOverlay::new();
        assert!(matches!(
            overlay.open(SecureOverlayPrompt::ProviderApiKey),
            SecureOverlayAction::Opened(snapshot)
                if snapshot.prompt == Some(SecureOverlayPrompt::ProviderApiKey)
                    && snapshot.masked_len == 0
        ));
        for byte in b"sk-test" {
            let action = overlay.handle(SecureOverlayInput::TextByte(*byte));
            let SecureOverlayAction::Changed(snapshot) = action else {
                panic!("provider byte should update the masked input");
            };
            assert_eq!(snapshot.masked_len, overlay.snapshot().masked_len);
        }
        assert_eq!(overlay.snapshot().masked_len, 7);

        let action = overlay.handle(SecureOverlayInput::Submit);
        let SecureOverlayAction::Submitted(submission) = action else {
            panic!("provider entry should submit");
        };
        assert_eq!(submission.prompt(), SecureOverlayPrompt::ProviderApiKey);
        assert_eq!(submission.secret_kind(), SecretKind::ProviderApiKey);
        assert_eq!(submission.len(), 7);
        drop(submission.into_plaintext_for_broker());
        assert_eq!(overlay.snapshot().prompt, None);
        assert!(overlay.bytes.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn wifi_entry_requires_the_real_minimum_and_clears_on_cancel() {
        let mut overlay = SecureOverlay::new();
        let _ = overlay.open(SecureOverlayPrompt::WifiPassphrase);
        for byte in b"short" {
            let _ = overlay.handle(SecureOverlayInput::TextByte(*byte));
        }
        let _ = overlay.handle(SecureOverlayInput::Backspace);
        let _ = overlay.handle(SecureOverlayInput::TextByte(b't'));
        assert!(matches!(
            overlay.handle(SecureOverlayInput::Submit),
            SecureOverlayAction::Rejected(SecureOverlayRejection::WifiPassphraseTooShort)
        ));
        assert_eq!(overlay.snapshot().masked_len, 5);

        assert!(matches!(
            overlay.handle(SecureOverlayInput::Cancel),
            SecureOverlayAction::Cancelled(SecureOverlayPrompt::WifiPassphrase)
        ));
        assert_eq!(overlay.snapshot().prompt, None);
        assert!(overlay.bytes.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn prompt_specific_bounds_and_byte_rules_are_fail_closed() {
        let mut overlay = SecureOverlay::new();
        let _ = overlay.open(SecureOverlayPrompt::ProviderApiKey);
        assert!(matches!(
            overlay.handle(SecureOverlayInput::TextByte(b' ')),
            SecureOverlayAction::Rejected(SecureOverlayRejection::InvalidByte)
        ));
        for _ in 0..MAX_PROVIDER_API_KEY_LEN {
            let _ = overlay.handle(SecureOverlayInput::TextByte(b'a'));
        }
        assert!(matches!(
            overlay.handle(SecureOverlayInput::TextByte(b'a')),
            SecureOverlayAction::Rejected(SecureOverlayRejection::CapacityReached)
        ));

        let _ = overlay.open(SecureOverlayPrompt::WifiPassphrase);
        for _ in 0..MAX_WIFI_PASSPHRASE_LEN {
            let _ = overlay.handle(SecureOverlayInput::TextByte(b' '));
        }
        assert_eq!(overlay.snapshot().masked_len, MAX_WIFI_PASSPHRASE_LEN);
        assert!(matches!(
            overlay.handle(SecureOverlayInput::TextByte(b' ')),
            SecureOverlayAction::Rejected(SecureOverlayRejection::CapacityReached)
        ));
    }
}
