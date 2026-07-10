//! Secret Vault VMK custody and recovery-wrapper generation state.
//!
//! The keyring owns opaque in-RAM VMK/recovery-key values and encrypted
//! recovery wrappers.  It neither parses store frames nor authorizes secret
//! consumers.  TPM ACPI discovery alone never changes its `not_proven` TPM
//! status; a future real seal/readback/unseal path must provide its own proof.

use core::sync::atomic::{compiler_fence, Ordering};

use sha2::{Digest, Sha256};

use raios_core::core_policy::VerifiedCorePolicy;
use raios_core::secret_vault::{
    unwrap_vmk_from_recovery, wrap_vmk_for_recovery, FreshWrapperNonce, RecoveryKekContext,
    RecoveryKey, RecoveryVmkWrapperV1, SecretVaultError, VaultMasterKey, RECOVERY_WRAPPER_VERSION,
};

use crate::entropy;

pub(crate) const RECOVERY_KEY_LEN: usize = 32;
pub(crate) const RECOVERY_KEY_TEXT_LEN: usize = 80;

const RECOVERY_KEY_PREFIX: &[u8; 4] = b"RR1-";
const RECOVERY_KEY_CHECKSUM_DOMAIN: &[u8] = b"raios-recovery-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TpmAutoUnlockStatus {
    /// No real create/load/unseal/readback evidence has been accepted.
    NotProven,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KeyringDenied {
    EntropyNotReady,
    InvalidRecoveryKeyFormat,
    InvalidRecoveryKeyChecksum,
    RecoveryKeyConfirmationMismatch,
    InvalidWrapperContext,
    WrapperGenerationNotMonotonic,
    DuplicateWrapperGeneration,
    NextWrapperAlreadyStaged,
    ReadbackNotVerified,
    ReadbackMismatch,
    NoStagedWrapper,
    NoMatchingRecoveryWrapper,
    RecoveryWrapper(SecretVaultError),
}

/// A one-time secure-overlay value.  It zeroizes itself after the overlay
/// dismisses it and exposes no heap allocation or string conversion.
pub(crate) struct RecoveryKeyText {
    bytes: [u8; RECOVERY_KEY_TEXT_LEN],
}

impl RecoveryKeyText {
    pub(crate) fn as_bytes(&self) -> &[u8; RECOVERY_KEY_TEXT_LEN] {
        &self.bytes
    }
}

impl Drop for RecoveryKeyText {
    fn drop(&mut self) {
        zeroize_secret(&mut self.bytes);
    }
}

/// An uncommitted first-provisioning recovery key.  Genesis displays its text,
/// receives a full re-entry, and only then converts it to the opaque C2 key.
pub(crate) struct RecoveryKeyCandidate {
    bytes: [u8; RECOVERY_KEY_LEN],
}

impl RecoveryKeyCandidate {
    pub(crate) fn confirm(mut self, reentered: &mut [u8]) -> Result<RecoveryKey, KeyringDenied> {
        let mut parsed = parse_recovery_key(reentered)?;
        if !constant_time_eq(&self.bytes, &parsed) {
            zeroize_secret(&mut parsed);
            return Err(KeyringDenied::RecoveryKeyConfirmationMismatch);
        }
        zeroize_secret(&mut self.bytes);
        Ok(RecoveryKey::take_from(&mut parsed))
    }
}

impl Drop for RecoveryKeyCandidate {
    fn drop(&mut self) {
        zeroize_secret(&mut self.bytes);
    }
}

/// Exact core policy that a recovery wrapper may unlock.  The facade supplies
/// this only from the current, next, or last-good core policy records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ApprovedCorePolicy {
    core_generation: u64,
    policy_id_sha256: [u8; 32],
}

impl ApprovedCorePolicy {
    /// The only runtime constructor: callers cannot turn generation/hash
    /// claims into Vault authority without the opaque Core Policy proof.
    pub(super) const fn from_verified(policy: &VerifiedCorePolicy) -> Self {
        Self {
            core_generation: policy.core_generation(),
            policy_id_sha256: policy.policy_id_sha256(),
        }
    }

    pub(super) const fn core_generation(self) -> u64 {
        self.core_generation
    }

    pub(super) const fn policy_id_sha256(self) -> [u8; 32] {
        self.policy_id_sha256
    }

    #[cfg(test)]
    pub(super) const fn for_test(core_generation: u64, policy_id_sha256: [u8; 32]) -> Self {
        Self {
            core_generation,
            policy_id_sha256,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryWrapperSlot {
    Current,
    LastGood,
}

struct WrapperSlot {
    wrapper: RecoveryVmkWrapperV1,
    readback_verified: bool,
}

/// Current/next/last-good recovery-wrapper slots.  A `next` wrapper is never
/// selected until a decrypt/AEAD verification succeeds under the exact next
/// policy; a stale or corrupt wrapper cannot silently replace `current`.
pub(crate) struct RecoveryWrapperSlots {
    store_uuid: [u8; 16],
    key_epoch: u64,
    current: WrapperSlot,
    next: Option<WrapperSlot>,
    last_good: Option<WrapperSlot>,
    tpm_auto_unlock: TpmAutoUnlockStatus,
}

impl RecoveryWrapperSlots {
    /// Reconstructs the current recovery slot only from the exact wrapper that
    /// the structured-store replay path has read back.  This establishes no
    /// TPM, store-I/O, or secret-consumer authority; it only carries the
    /// readback proof into the in-RAM keyring.
    pub(super) fn restore_current_from_verified_replay(
        replayed: RecoveryVmkWrapperV1,
        expected: ApprovedCorePolicy,
    ) -> Result<Self, KeyringDenied> {
        validate_context(replayed.context)?;
        if replayed.version != RECOVERY_WRAPPER_VERSION
            || replayed.context.core_generation != expected.core_generation
            || replayed.context.policy_id_sha256 != expected.policy_id_sha256
        {
            return Err(KeyringDenied::InvalidWrapperContext);
        }
        Ok(Self {
            store_uuid: replayed.context.store_uuid,
            key_epoch: replayed.context.key_epoch,
            current: WrapperSlot {
                wrapper: replayed,
                readback_verified: true,
            },
            next: None,
            last_good: None,
            tpm_auto_unlock: TpmAutoUnlockStatus::NotProven,
        })
    }

    /// Creates the initial recovery wrapper from ready core entropy.  This
    /// function makes no TPM claim and performs no store write.
    pub(crate) fn provision_initial(
        vmk: &VaultMasterKey,
        recovery_key: &RecoveryKey,
        context: RecoveryKekContext,
    ) -> Result<Self, KeyringDenied> {
        let nonce = fresh_random_nonce()?;
        Self::provision_initial_with_nonce(vmk, recovery_key, context, nonce)
    }

    /// Stages a strictly newer wrapper.  The caller must persist and read back
    /// `next_wrapper()` before it invokes `promote_verified_next`.
    pub(crate) fn stage_next(
        &mut self,
        vmk: &VaultMasterKey,
        recovery_key: &RecoveryKey,
        context: RecoveryKekContext,
    ) -> Result<(), KeyringDenied> {
        let nonce = fresh_random_nonce()?;
        self.stage_next_with_nonce(vmk, recovery_key, context, nonce)
    }

    /// AEAD-verifies the staged wrapper before selecting it.  This proves only
    /// the recovery wrapper; it is deliberately not TPM auto-unlock evidence.
    pub(crate) fn promote_verified_next(
        &mut self,
        recovery_key: &RecoveryKey,
        expected: ApprovedCorePolicy,
    ) -> Result<(), KeyringDenied> {
        let staged = self.next.as_ref().ok_or(KeyringDenied::NoStagedWrapper)?;
        if !staged.readback_verified {
            return Err(KeyringDenied::ReadbackNotVerified);
        }
        let expected_context = self.expected_context(&staged.wrapper, expected)?;
        let verified = unwrap_vmk_from_recovery(recovery_key, &staged.wrapper, &expected_context)
            .map_err(KeyringDenied::RecoveryWrapper)?;
        drop(verified);

        let mut next = self.next.take().ok_or(KeyringDenied::NoStagedWrapper)?;
        next.readback_verified = true;
        self.last_good = Some(core::mem::replace(&mut self.current, next));
        Ok(())
    }

    /// Unlocks only a wrapper matching the approved current or last-good core
    /// policy.  It never tries an unverified `next` wrapper.
    pub(crate) fn unlock_with_recovery(
        &self,
        recovery_key: &RecoveryKey,
        expected: ApprovedCorePolicy,
    ) -> Result<(VaultMasterKey, RecoveryWrapperSlot), KeyringDenied> {
        if self.current.readback_verified {
            if let Some(context) = self.context_if_matches(&self.current.wrapper, expected) {
                match unwrap_vmk_from_recovery(recovery_key, &self.current.wrapper, &context) {
                    Ok(vmk) => return Ok((vmk, RecoveryWrapperSlot::Current)),
                    Err(_) => {}
                }
            }
        }
        if let Some(last_good) = self.last_good.as_ref() {
            if last_good.readback_verified {
                if let Some(context) = self.context_if_matches(&last_good.wrapper, expected) {
                    let vmk = unwrap_vmk_from_recovery(recovery_key, &last_good.wrapper, &context)
                        .map_err(KeyringDenied::RecoveryWrapper)?;
                    return Ok((vmk, RecoveryWrapperSlot::LastGood));
                }
            }
        }
        Err(KeyringDenied::NoMatchingRecoveryWrapper)
    }

    /// Once the selected current core has booted successfully, the older
    /// fallback is no longer retained as an active recovery wrapper slot.
    pub(crate) fn confirm_current_boot_success(&mut self) {
        self.last_good = None;
    }

    /// Accepts only byte-for-byte equivalent wrapper bytes after the store has
    /// committed, flushed, read back, and reparsed the record.
    pub(crate) fn confirm_current_readback(
        &mut self,
        persisted: &RecoveryVmkWrapperV1,
    ) -> Result<(), KeyringDenied> {
        if !same_wrapper(&self.current.wrapper, persisted) {
            return Err(KeyringDenied::ReadbackMismatch);
        }
        self.current.readback_verified = true;
        Ok(())
    }

    /// The next wrapper remains unusable until its exact persisted bytes are
    /// read back by the structured store path.
    pub(crate) fn confirm_next_readback(
        &mut self,
        persisted: &RecoveryVmkWrapperV1,
    ) -> Result<(), KeyringDenied> {
        let next = self.next.as_mut().ok_or(KeyringDenied::NoStagedWrapper)?;
        if !same_wrapper(&next.wrapper, persisted) {
            return Err(KeyringDenied::ReadbackMismatch);
        }
        next.readback_verified = true;
        Ok(())
    }

    pub(crate) fn current_wrapper(&self) -> &RecoveryVmkWrapperV1 {
        &self.current.wrapper
    }

    pub(crate) fn next_wrapper(&self) -> Option<&RecoveryVmkWrapperV1> {
        self.next.as_ref().map(|slot| &slot.wrapper)
    }

    pub(crate) fn last_good_wrapper(&self) -> Option<&RecoveryVmkWrapperV1> {
        self.last_good.as_ref().map(|slot| &slot.wrapper)
    }

    pub(crate) const fn tpm_auto_unlock_status(&self) -> TpmAutoUnlockStatus {
        self.tpm_auto_unlock
    }

    fn provision_initial_with_nonce(
        vmk: &VaultMasterKey,
        recovery_key: &RecoveryKey,
        context: RecoveryKekContext,
        nonce: [u8; 12],
    ) -> Result<Self, KeyringDenied> {
        validate_context(context)?;
        let wrapper = wrap_vmk_for_recovery(
            recovery_key,
            vmk,
            context,
            FreshWrapperNonce {
                nonce,
                retained_kek_nonces: &[],
                retained_set_complete: true,
            },
        )
        .map_err(KeyringDenied::RecoveryWrapper)?;
        Ok(Self {
            store_uuid: context.store_uuid,
            key_epoch: context.key_epoch,
            current: WrapperSlot {
                wrapper,
                readback_verified: false,
            },
            next: None,
            last_good: None,
            tpm_auto_unlock: TpmAutoUnlockStatus::NotProven,
        })
    }

    fn stage_next_with_nonce(
        &mut self,
        vmk: &VaultMasterKey,
        recovery_key: &RecoveryKey,
        context: RecoveryKekContext,
        nonce: [u8; 12],
    ) -> Result<(), KeyringDenied> {
        validate_context(context)?;
        if context.store_uuid != self.store_uuid || context.key_epoch != self.key_epoch {
            return Err(KeyringDenied::InvalidWrapperContext);
        }
        if self.next.is_some() {
            return Err(KeyringDenied::NextWrapperAlreadyStaged);
        }
        let generation = context.wrapper_generation;
        if generation <= self.current.wrapper.context.wrapper_generation
            || self
                .last_good
                .as_ref()
                .is_some_and(|slot| generation <= slot.wrapper.context.wrapper_generation)
        {
            return Err(KeyringDenied::WrapperGenerationNotMonotonic);
        }
        let wrapper = wrap_vmk_for_recovery(
            recovery_key,
            vmk,
            context,
            FreshWrapperNonce {
                nonce,
                retained_kek_nonces: &[],
                retained_set_complete: true,
            },
        )
        .map_err(KeyringDenied::RecoveryWrapper)?;
        self.next = Some(WrapperSlot {
            wrapper,
            readback_verified: false,
        });
        Ok(())
    }

    fn expected_context(
        &self,
        wrapper: &RecoveryVmkWrapperV1,
        expected: ApprovedCorePolicy,
    ) -> Result<RecoveryKekContext, KeyringDenied> {
        self.context_if_matches(wrapper, expected)
            .ok_or(KeyringDenied::NoMatchingRecoveryWrapper)
    }

    fn context_if_matches(
        &self,
        wrapper: &RecoveryVmkWrapperV1,
        expected: ApprovedCorePolicy,
    ) -> Option<RecoveryKekContext> {
        let context = wrapper.context;
        if wrapper.version != RECOVERY_WRAPPER_VERSION
            || context.store_uuid != self.store_uuid
            || context.key_epoch != self.key_epoch
            || context.core_generation != expected.core_generation
            || context.policy_id_sha256 != expected.policy_id_sha256
        {
            return None;
        }
        Some(context)
    }
}

fn same_wrapper(left: &RecoveryVmkWrapperV1, right: &RecoveryVmkWrapperV1) -> bool {
    left.version == right.version
        && left.context == right.context
        && left.plaintext_len == right.plaintext_len
        && constant_time_eq(&left.nonce, &right.nonce)
        && constant_time_eq(&left.ciphertext, &right.ciphertext)
        && constant_time_eq(&left.tag, &right.tag)
}

/// Generates a new nonpersistent 32-byte VMK from ready core entropy.
pub(crate) fn generate_vault_master_key() -> Result<VaultMasterKey, KeyringDenied> {
    if !entropy::is_ready() {
        return Err(KeyringDenied::EntropyNotReady);
    }
    let mut bytes = [0u8; RECOVERY_KEY_LEN];
    entropy::take(&mut bytes);
    Ok(VaultMasterKey::take_from(&mut bytes))
}

/// Generates the high-entropy recovery key and its exact RR1 presentation.
/// The candidate must be re-entered and confirmed before wrapper persistence.
pub(crate) fn generate_recovery_key_candidate(
) -> Result<(RecoveryKeyCandidate, RecoveryKeyText), KeyringDenied> {
    if !entropy::is_ready() {
        return Err(KeyringDenied::EntropyNotReady);
    }
    let mut bytes = [0u8; RECOVERY_KEY_LEN];
    entropy::take(&mut bytes);
    let text = format_recovery_key(&bytes);
    let candidate = RecoveryKeyCandidate { bytes };
    zeroize_secret(&mut bytes);
    Ok((candidate, text))
}

fn validate_context(context: RecoveryKekContext) -> Result<(), KeyringDenied> {
    if context.store_uuid == [0; 16]
        || context.key_epoch == 0
        || context.wrapper_generation == 0
        || context.core_generation == 0
        || context.policy_id_sha256 == [0; 32]
    {
        return Err(KeyringDenied::InvalidWrapperContext);
    }
    Ok(())
}

fn fresh_random_nonce() -> Result<[u8; 12], KeyringDenied> {
    if !entropy::is_ready() {
        return Err(KeyringDenied::EntropyNotReady);
    }
    let mut nonce = [0u8; 12];
    entropy::take(&mut nonce);
    Ok(nonce)
}

fn format_recovery_key(key: &[u8; RECOVERY_KEY_LEN]) -> RecoveryKeyText {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut bytes = [0u8; RECOVERY_KEY_TEXT_LEN];
    bytes[..RECOVERY_KEY_PREFIX.len()].copy_from_slice(RECOVERY_KEY_PREFIX);
    let mut out = RECOVERY_KEY_PREFIX.len();
    for (index, byte) in key.iter().copied().enumerate() {
        bytes[out] = HEX[(byte >> 4) as usize];
        bytes[out + 1] = HEX[(byte & 0x0f) as usize];
        out += 2;
        if index % 4 == 3 {
            bytes[out] = b'-';
            out += 1;
        }
    }
    let checksum = recovery_checksum(key);
    bytes[out] = HEX[(checksum[0] >> 4) as usize];
    bytes[out + 1] = HEX[(checksum[0] & 0x0f) as usize];
    bytes[out + 2] = HEX[(checksum[1] >> 4) as usize];
    bytes[out + 3] = HEX[(checksum[1] & 0x0f) as usize];
    RecoveryKeyText { bytes }
}

fn parse_recovery_key(input: &mut [u8]) -> Result<[u8; RECOVERY_KEY_LEN], KeyringDenied> {
    let result = parse_recovery_key_inner(input);
    zeroize_secret(input);
    result
}

pub(super) fn parse_recovery_key_input(input: &mut [u8]) -> Result<RecoveryKey, KeyringDenied> {
    let mut parsed = parse_recovery_key(input)?;
    Ok(RecoveryKey::take_from(&mut parsed))
}

pub(super) fn zeroize_recovery_input(input: &mut [u8]) {
    zeroize_secret(input);
}

fn parse_recovery_key_inner(input: &[u8]) -> Result<[u8; RECOVERY_KEY_LEN], KeyringDenied> {
    if input.len() != RECOVERY_KEY_TEXT_LEN || input.get(..4) != Some(RECOVERY_KEY_PREFIX) {
        return Err(KeyringDenied::InvalidRecoveryKeyFormat);
    }
    let mut key = [0u8; RECOVERY_KEY_LEN];
    let mut input_index = RECOVERY_KEY_PREFIX.len();
    for output_index in 0..RECOVERY_KEY_LEN {
        let high = hex_value(
            *input
                .get(input_index)
                .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?,
        )
        .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?;
        let low = hex_value(
            *input
                .get(input_index + 1)
                .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?,
        )
        .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?;
        key[output_index] = (high << 4) | low;
        input_index += 2;
        if output_index % 4 == 3 {
            if input.get(input_index) != Some(&b'-') {
                key.fill(0);
                return Err(KeyringDenied::InvalidRecoveryKeyFormat);
            }
            input_index += 1;
        }
    }
    let actual = [
        (hex_value(
            *input
                .get(input_index)
                .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?,
        )
        .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?
            << 4)
            | hex_value(
                *input
                    .get(input_index + 1)
                    .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?,
            )
            .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?,
        (hex_value(
            *input
                .get(input_index + 2)
                .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?,
        )
        .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?
            << 4)
            | hex_value(
                *input
                    .get(input_index + 3)
                    .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?,
            )
            .ok_or(KeyringDenied::InvalidRecoveryKeyFormat)?,
    ];
    if input_index + 4 != input.len() || !constant_time_eq(&actual, &recovery_checksum(&key)) {
        key.fill(0);
        return Err(KeyringDenied::InvalidRecoveryKeyChecksum);
    }
    Ok(key)
}

fn recovery_checksum(key: &[u8; RECOVERY_KEY_LEN]) -> [u8; 2] {
    let mut hash = Sha256::new();
    hash.update(RECOVERY_KEY_CHECKSUM_DOMAIN);
    hash.update(key);
    let digest = hash.finalize();
    [digest[0], digest[1]]
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (left, right) in left.iter().zip(right.iter()) {
        difference |= left ^ right;
    }
    difference == 0
}

fn zeroize_secret(bytes: &mut [u8]) {
    for byte in bytes {
        // SAFETY: `byte` is a unique mutable reference into the caller-owned buffer.
        unsafe { core::ptr::write_volatile(byte, 0) };
    }
    compiler_fence(Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vmk(byte: u8) -> VaultMasterKey {
        VaultMasterKey::take_from(&mut [byte; RECOVERY_KEY_LEN])
    }

    fn recovery_key(byte: u8) -> RecoveryKey {
        RecoveryKey::take_from(&mut [byte; RECOVERY_KEY_LEN])
    }

    fn context(generation: u64, core_generation: u64, policy: u8) -> RecoveryKekContext {
        RecoveryKekContext {
            store_uuid: [0x11; 16],
            key_epoch: 7,
            wrapper_generation: generation,
            core_generation,
            policy_id_sha256: [policy; 32],
        }
    }

    #[test]
    fn rr1_round_trip_accepts_case_and_zeroizes_input() {
        let key = [0x5a; RECOVERY_KEY_LEN];
        let text = format_recovery_key(&key);
        assert_eq!(text.as_bytes().len(), RECOVERY_KEY_TEXT_LEN);
        let mut input = *text.as_bytes();
        input[4] = b'A';
        let parsed = parse_recovery_key(&mut input).unwrap();
        assert_eq!(parsed[0] >> 4, 0x0a);
        assert_eq!(input, [0; RECOVERY_KEY_TEXT_LEN]);

        let mut wrong = *text.as_bytes();
        wrong[RECOVERY_KEY_TEXT_LEN - 1] = b'0';
        assert_eq!(
            parse_recovery_key(&mut wrong),
            Err(KeyringDenied::InvalidRecoveryKeyChecksum)
        );
        assert_eq!(wrong, [0; RECOVERY_KEY_TEXT_LEN]);
    }

    #[test]
    fn staged_wrapper_must_verify_before_selection_and_last_good_unlocks() {
        let key = recovery_key(0x42);
        let master = vmk(0x17);
        let mut slots = RecoveryWrapperSlots::provision_initial_with_nonce(
            &master,
            &key,
            context(1, 10, 0x33),
            [1; 12],
        )
        .unwrap();
        let mut mismatched = duplicate_wrapper(slots.current_wrapper());
        mismatched.tag[0] ^= 1;
        assert_eq!(
            slots.confirm_current_readback(&mismatched),
            Err(KeyringDenied::ReadbackMismatch)
        );
        let readback = duplicate_wrapper(slots.current_wrapper());
        slots.confirm_current_readback(&readback).unwrap();
        slots
            .stage_next_with_nonce(&master, &key, context(2, 11, 0x44), [2; 12])
            .unwrap();
        let next_readback = duplicate_wrapper(slots.next_wrapper().unwrap());
        slots.confirm_next_readback(&next_readback).unwrap();
        assert!(slots.next_wrapper().is_some());
        slots
            .promote_verified_next(&key, ApprovedCorePolicy::for_test(11, [0x44; 32]))
            .unwrap();
        let (_, selected) = slots
            .unlock_with_recovery(&key, ApprovedCorePolicy::for_test(10, [0x33; 32]))
            .unwrap();
        assert_eq!(selected, RecoveryWrapperSlot::LastGood);
        assert_eq!(
            slots.tpm_auto_unlock_status(),
            TpmAutoUnlockStatus::NotProven
        );
    }

    #[test]
    fn replayed_readback_restores_only_the_approved_current_wrapper() {
        let key = recovery_key(0x42);
        let master = vmk(0x17);
        let slots = RecoveryWrapperSlots::provision_initial_with_nonce(
            &master,
            &key,
            context(3, 12, 0x55),
            [3; 12],
        )
        .unwrap();
        let approved = ApprovedCorePolicy::for_test(12, [0x55; 32]);
        let replayed = duplicate_wrapper(slots.current_wrapper());
        let restored =
            RecoveryWrapperSlots::restore_current_from_verified_replay(replayed, approved).unwrap();
        assert_eq!(
            restored.unlock_with_recovery(&key, approved).unwrap().1,
            RecoveryWrapperSlot::Current
        );

        let replayed = duplicate_wrapper(slots.current_wrapper());
        assert!(matches!(
            RecoveryWrapperSlots::restore_current_from_verified_replay(
                replayed,
                ApprovedCorePolicy::for_test(13, [0x66; 32]),
            ),
            Err(KeyringDenied::InvalidWrapperContext)
        ));
    }

    #[test]
    fn unverified_current_cannot_be_selected_before_readback() {
        let key = recovery_key(0x42);
        let master = vmk(0x17);
        let slots = RecoveryWrapperSlots::provision_initial_with_nonce(
            &master,
            &key,
            context(3, 12, 0x55),
            [3; 12],
        )
        .unwrap();
        assert!(matches!(
            slots.unlock_with_recovery(&key, ApprovedCorePolicy::for_test(12, [0x55; 32]),),
            Err(KeyringDenied::NoMatchingRecoveryWrapper)
        ));
    }

    #[test]
    fn stale_generation_and_corrupt_wrapper_fail_closed() {
        let key = recovery_key(0x42);
        let master = vmk(0x17);
        let mut slots = RecoveryWrapperSlots::provision_initial_with_nonce(
            &master,
            &key,
            context(3, 12, 0x55),
            [3; 12],
        )
        .unwrap();
        assert_eq!(
            slots.stage_next_with_nonce(&master, &key, context(3, 13, 0x66), [4; 12]),
            Err(KeyringDenied::WrapperGenerationNotMonotonic)
        );
        slots.current.wrapper.tag[0] ^= 1;
        assert!(matches!(
            slots.unlock_with_recovery(&key, ApprovedCorePolicy::for_test(12, [0x55; 32]),),
            Err(KeyringDenied::NoMatchingRecoveryWrapper)
        ));
    }

    fn duplicate_wrapper(value: &RecoveryVmkWrapperV1) -> RecoveryVmkWrapperV1 {
        RecoveryVmkWrapperV1 {
            version: value.version,
            context: value.context,
            plaintext_len: value.plaintext_len,
            nonce: value.nonce,
            ciphertext: value.ciphertext,
            tag: value.tag,
        }
    }
}
