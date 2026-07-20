//! Fixed TLS 1.3/P-256/AES-128-GCM crypto with opaque key custody.

use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes128Gcm, Nonce, Tag,
};
use alloc::vec::Vec;
use hkdf::{
    hmac::{Hmac, Mac},
    Hkdf,
};
use p256::{
    ecdh::diffie_hellman,
    ecdsa::{
        signature::{Signer, Verifier},
        Signature, SigningKey, VerifyingKey,
    },
    elliptic_curve::sec1::ToEncodedPoint,
    PublicKey, SecretKey,
};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

pub const TLS13_PUBLIC_OUTPUT_LEN: usize = 97;
pub const TLS13_HASH_LEN: usize = 32;
pub const TLS13_RECORD_HEADER_LEN: usize = 5;
pub const TLS13_TAG_LEN: usize = 16;
pub const TLS13_MAX_CIPHERTEXT_LEN: usize = (1 << 14) + 256;
pub const TLS13_MAX_PLAINTEXT_LEN: usize = TLS13_MAX_CIPHERTEXT_LEN - TLS13_TAG_LEN;
pub const TLS13_MAX_HASH_INPUT_LEN: usize = 1 << 14;
pub const TLS13_MAX_SPKI_LEN: usize = 512;
pub const TLS13_MAX_VERIFY_MESSAGE_LEN: usize = 1 << 14;
pub const TLS13_MAX_SIGNATURE_LEN: usize = 80;

const TLS13_HANDLE_SLOT: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tls13Denial {
    InvocationAuthorityInvalid,
    KillGenerationChanged,
    ForeignTcpOwner,
    EntropyNotReady,
    EntropySampleInvalid,
    DuplicateSession,
    WrongPublicOutputSize,
    GuestMemoryFault,
    ForeignSession,
    StaleSession,
    HashInputTooLarge,
    UnsupportedSpkiEncoding,
    UnsupportedSignatureEncoding,
    VerifyInputOutOfBounds,
    SourcePinAbsent,
    WrongPeerKeyEncoding,
    RepeatedOrOutOfOrderTransition,
    ZeroTranscriptHash,
    GuestTrustSelfAssertion,
    CertificateVerifyEvidenceMissing,
    CertificateVerifyMathInvalid,
    SourcePinMismatch,
    ServerFinishedEvidenceMissing,
    ServerFinishedEvidenceInvalid,
    InvalidFinishedMode,
    WrongFinishedProofSize,
    FinishedWrongState,
    InvalidRecordHeader,
    RecordLengthMismatch,
    RecordTooLarge,
    SequenceExhausted,
    OutputTooSmall,
    TagAuthenticationFailed,
    AeadWrongState,
    CryptoPrimitiveFailed,
}

impl Tls13Denial {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvocationAuthorityInvalid => "crypto_invocation_authority_invalid",
            Self::KillGenerationChanged => "crypto_kill_generation_changed",
            Self::ForeignTcpOwner => "crypto_foreign_tcp_owner",
            Self::EntropyNotReady => "crypto_entropy_not_ready",
            Self::EntropySampleInvalid => "crypto_entropy_sample_invalid",
            Self::DuplicateSession => "crypto_duplicate_session",
            Self::WrongPublicOutputSize => "crypto_wrong_public_output_size",
            Self::GuestMemoryFault => "crypto_guest_memory_fault",
            Self::ForeignSession => "crypto_foreign_session",
            Self::StaleSession => "crypto_stale_session",
            Self::HashInputTooLarge => "crypto_hash_input_too_large",
            Self::UnsupportedSpkiEncoding => "crypto_unsupported_spki_encoding",
            Self::UnsupportedSignatureEncoding => "crypto_unsupported_signature_encoding",
            Self::VerifyInputOutOfBounds => "crypto_verify_input_out_of_bounds",
            Self::SourcePinAbsent => "crypto_source_pin_absent",
            Self::WrongPeerKeyEncoding => "crypto_wrong_peer_key_encoding",
            Self::RepeatedOrOutOfOrderTransition => "crypto_repeated_or_out_of_order_transition",
            Self::ZeroTranscriptHash => "crypto_zero_transcript_hash",
            Self::GuestTrustSelfAssertion => "crypto_guest_trust_self_assertion_denied",
            Self::CertificateVerifyEvidenceMissing => "crypto_certificate_verify_evidence_missing",
            Self::CertificateVerifyMathInvalid => "crypto_certificate_verify_math_invalid",
            Self::SourcePinMismatch => "crypto_source_pin_mismatch",
            Self::ServerFinishedEvidenceMissing => "crypto_server_finished_evidence_missing",
            Self::ServerFinishedEvidenceInvalid => "crypto_server_finished_evidence_invalid",
            Self::InvalidFinishedMode => "crypto_invalid_finished_mode",
            Self::WrongFinishedProofSize => "crypto_wrong_finished_proof_size",
            Self::FinishedWrongState => "crypto_finished_wrong_state",
            Self::InvalidRecordHeader => "crypto_invalid_record_header",
            Self::RecordLengthMismatch => "crypto_record_length_mismatch",
            Self::RecordTooLarge => "crypto_record_too_large",
            Self::SequenceExhausted => "crypto_sequence_exhausted",
            Self::OutputTooSmall => "crypto_output_too_small",
            Self::TagAuthenticationFailed => "crypto_tag_authentication_failed",
            Self::AeadWrongState => "crypto_aead_wrong_state",
            Self::CryptoPrimitiveFailed => "crypto_primitive_failed",
        }
    }
}

pub const ALL_TLS13_DENIALS: [Tls13Denial; 35] = [
    Tls13Denial::InvocationAuthorityInvalid,
    Tls13Denial::KillGenerationChanged,
    Tls13Denial::ForeignTcpOwner,
    Tls13Denial::EntropyNotReady,
    Tls13Denial::EntropySampleInvalid,
    Tls13Denial::DuplicateSession,
    Tls13Denial::WrongPublicOutputSize,
    Tls13Denial::GuestMemoryFault,
    Tls13Denial::ForeignSession,
    Tls13Denial::StaleSession,
    Tls13Denial::HashInputTooLarge,
    Tls13Denial::UnsupportedSpkiEncoding,
    Tls13Denial::UnsupportedSignatureEncoding,
    Tls13Denial::VerifyInputOutOfBounds,
    Tls13Denial::SourcePinAbsent,
    Tls13Denial::WrongPeerKeyEncoding,
    Tls13Denial::RepeatedOrOutOfOrderTransition,
    Tls13Denial::ZeroTranscriptHash,
    Tls13Denial::GuestTrustSelfAssertion,
    Tls13Denial::CertificateVerifyEvidenceMissing,
    Tls13Denial::CertificateVerifyMathInvalid,
    Tls13Denial::SourcePinMismatch,
    Tls13Denial::ServerFinishedEvidenceMissing,
    Tls13Denial::ServerFinishedEvidenceInvalid,
    Tls13Denial::InvalidFinishedMode,
    Tls13Denial::WrongFinishedProofSize,
    Tls13Denial::FinishedWrongState,
    Tls13Denial::InvalidRecordHeader,
    Tls13Denial::RecordLengthMismatch,
    Tls13Denial::RecordTooLarge,
    Tls13Denial::SequenceExhausted,
    Tls13Denial::OutputTooSmall,
    Tls13Denial::TagAuthenticationFailed,
    Tls13Denial::AeadWrongState,
    Tls13Denial::CryptoPrimitiveFailed,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tls13SessionState {
    Opened,
    HandshakeKeys,
    ApplicationKeysDerived,
    ClientFinishedReady,
    ApplicationTraffic,
    Closed,
}

impl Tls13SessionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Opened => "opened",
            Self::HandshakeKeys => "handshake_keys",
            Self::ApplicationKeysDerived => "application_keys_derived",
            Self::ClientFinishedReady => "client_finished_ready",
            Self::ApplicationTraffic => "application_traffic",
            Self::Closed => "closed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tls13Evidence {
    pub public_output_sha256: [u8; 32],
    pub peer_public_sha256: [u8; 32],
    pub hello_transcript_sha256: [u8; 32],
    pub application_transcript_sha256: [u8; 32],
    pub spki_sha256: [u8; 32],
    pub verify_message_sha256: [u8; 32],
    pub signature_sha256: [u8; 32],
    pub certificate_verify_observed: bool,
    pub math_valid: bool,
    pub pin_match: bool,
    pub server_finished_observed: bool,
    pub server_finished_valid: bool,
    pub trust_label_granted_by_guest: bool,
    pub transition_count: u32,
    pub last_transition: &'static str,
}

impl Tls13Evidence {
    const fn empty() -> Self {
        Self {
            public_output_sha256: [0; 32],
            peer_public_sha256: [0; 32],
            hello_transcript_sha256: [0; 32],
            application_transcript_sha256: [0; 32],
            spki_sha256: [0; 32],
            verify_message_sha256: [0; 32],
            signature_sha256: [0; 32],
            certificate_verify_observed: false,
            math_valid: false,
            pin_match: false,
            server_finished_observed: false,
            server_finished_valid: false,
            trust_label_granted_by_guest: false,
            transition_count: 0,
            last_transition: "not_started",
        }
    }
}

struct TrafficKeys {
    client_key: [u8; 16],
    server_key: [u8; 16],
    client_iv: [u8; 12],
    server_iv: [u8; 12],
    client_finished_key: [u8; 32],
    server_finished_key: [u8; 32],
}

impl Drop for TrafficKeys {
    fn drop(&mut self) {
        self.client_key.zeroize();
        self.server_key.zeroize();
        self.client_iv.zeroize();
        self.server_iv.zeroize();
        self.client_finished_key.zeroize();
        self.server_finished_key.zeroize();
    }
}

pub enum Tls13FinishedOutput {
    ClientProof([u8; 32]),
    ServerVerified(bool),
}

pub struct OpaqueSession {
    private_key: Option<SecretKey>,
    expected_spki_pin: Option<[u8; 32]>,
    handshake_secret: [u8; 32],
    handshake_keys: Option<TrafficKeys>,
    application_keys: Option<TrafficKeys>,
    send_sequence: u64,
    receive_sequence: u64,
    state: Tls13SessionState,
    evidence: Tls13Evidence,
}

impl OpaqueSession {
    pub fn open(
        entropy: &mut [u8; 64],
        expected_spki_pin: Option<[u8; 32]>,
    ) -> Result<(Self, [u8; TLS13_PUBLIC_OUTPUT_LEN]), Tls13Denial> {
        let private_key = match SecretKey::from_slice(&entropy[32..]) {
            Ok(private_key) => private_key,
            Err(_) => {
                entropy.zeroize();
                return Err(Tls13Denial::EntropySampleInvalid);
            }
        };
        let public_key = private_key.public_key().to_encoded_point(false);
        let public_bytes = public_key.as_bytes();
        if public_bytes.len() != 65 {
            return Err(Tls13Denial::CryptoPrimitiveFailed);
        }
        let mut public_output = [0u8; TLS13_PUBLIC_OUTPUT_LEN];
        public_output[..32].copy_from_slice(&entropy[..32]);
        public_output[32..].copy_from_slice(public_bytes);
        entropy.zeroize();
        let mut evidence = Tls13Evidence::empty();
        evidence.public_output_sha256 = digest(&public_output);
        evidence.transition_count = 1;
        evidence.last_transition = "session_opened";
        Ok((
            Self {
                private_key: Some(private_key),
                expected_spki_pin,
                handshake_secret: [0; 32],
                handshake_keys: None,
                application_keys: None,
                send_sequence: 0,
                receive_sequence: 0,
                state: Tls13SessionState::Opened,
                evidence,
            },
            public_output,
        ))
    }

    pub fn sha256(&self, input: &[u8]) -> Result<[u8; 32], Tls13Denial> {
        if input.len() > TLS13_MAX_HASH_INPUT_LEN {
            return Err(Tls13Denial::HashInputTooLarge);
        }
        Ok(digest(input))
    }

    pub fn p256_verify(
        &mut self,
        spki_der: &[u8],
        message: &[u8],
        signature_der: &[u8],
    ) -> Result<bool, Tls13Denial> {
        if self.state != Tls13SessionState::HandshakeKeys {
            return Err(Tls13Denial::RepeatedOrOutOfOrderTransition);
        }
        if spki_der.len() > TLS13_MAX_SPKI_LEN
            || message.len() > TLS13_MAX_VERIFY_MESSAGE_LEN
            || signature_der.len() > TLS13_MAX_SIGNATURE_LEN
        {
            return Err(Tls13Denial::VerifyInputOutOfBounds);
        }
        self.evidence.spki_sha256 = digest(spki_der);
        self.evidence.verify_message_sha256 = digest(message);
        self.evidence.signature_sha256 = digest(signature_der);
        let expected_pin = self.expected_spki_pin.ok_or(Tls13Denial::SourcePinAbsent)?;
        self.evidence.pin_match = self.evidence.spki_sha256 == expected_pin;
        // Parse the SPKI with raiOS's own no-dep parser (the M11-8 relocation),
        // not p256's pkcs8 path: that feature drags der/base64ct + PEM machinery
        // into the permanent core to do what this crate already does, host-tested.
        let spki = raios_x509_spki::parse_p256_spki(spki_der)
            .ok_or(Tls13Denial::UnsupportedSpkiEncoding)?;
        let verifying_key = VerifyingKey::from_sec1_bytes(spki.public_key)
            .map_err(|_| Tls13Denial::UnsupportedSpkiEncoding)?;
        let signature = Signature::from_der(signature_der)
            .map_err(|_| Tls13Denial::UnsupportedSignatureEncoding)?;
        let math_valid = verifying_key.verify(message, &signature).is_ok();
        self.evidence.certificate_verify_observed = true;
        self.evidence.math_valid = math_valid;
        Ok(math_valid)
    }

    pub fn handshake_keys(
        &mut self,
        peer_public_key: &[u8],
        hello_transcript_hash: &[u8; 32],
    ) -> Result<(), Tls13Denial> {
        if self.state != Tls13SessionState::Opened {
            return Err(Tls13Denial::RepeatedOrOutOfOrderTransition);
        }
        if hello_transcript_hash.iter().all(|byte| *byte == 0) {
            return Err(Tls13Denial::ZeroTranscriptHash);
        }
        if peer_public_key.len() != 65 {
            return Err(Tls13Denial::WrongPeerKeyEncoding);
        }
        let peer = PublicKey::from_sec1_bytes(peer_public_key)
            .map_err(|_| Tls13Denial::WrongPeerKeyEncoding)?;
        let private_key = self
            .private_key
            .take()
            .ok_or(Tls13Denial::RepeatedOrOutOfOrderTransition)?;
        let shared = diffie_hellman(private_key.to_nonzero_scalar(), peer.as_affine());
        let mut schedule =
            derive_handshake_schedule(shared.raw_secret_bytes(), hello_transcript_hash)?;
        self.handshake_secret = schedule.handshake_secret;
        schedule.handshake_secret.zeroize();
        self.handshake_keys = Some(schedule.keys);
        self.evidence.peer_public_sha256 = digest(peer_public_key);
        self.evidence.hello_transcript_sha256 = *hello_transcript_hash;
        self.transition(Tls13SessionState::HandshakeKeys, "handshake_keys_derived");
        Ok(())
    }

    pub fn application_keys(
        &mut self,
        handshake_transcript_hash: &[u8; 32],
    ) -> Result<(), Tls13Denial> {
        if self.state != Tls13SessionState::HandshakeKeys {
            return Err(Tls13Denial::RepeatedOrOutOfOrderTransition);
        }
        if handshake_transcript_hash.iter().all(|byte| *byte == 0) {
            return Err(Tls13Denial::ZeroTranscriptHash);
        }
        if !self.evidence.certificate_verify_observed {
            return Err(Tls13Denial::CertificateVerifyEvidenceMissing);
        }
        if !self.evidence.math_valid {
            return Err(Tls13Denial::CertificateVerifyMathInvalid);
        }
        if !self.evidence.pin_match {
            return Err(Tls13Denial::SourcePinMismatch);
        }
        if !self.evidence.server_finished_observed {
            return Err(Tls13Denial::ServerFinishedEvidenceMissing);
        }
        if !self.evidence.server_finished_valid {
            return Err(Tls13Denial::ServerFinishedEvidenceInvalid);
        }
        self.application_keys = Some(derive_application_keys(
            &self.handshake_secret,
            handshake_transcript_hash,
        )?);
        self.evidence.application_transcript_sha256 = *handshake_transcript_hash;
        self.receive_sequence = 0;
        self.transition(
            Tls13SessionState::ApplicationKeysDerived,
            "application_keys_derived_after_core_evidence",
        );
        Ok(())
    }

    pub fn finished(
        &mut self,
        mode: i32,
        transcript_hash: &[u8; 32],
        proof: &[u8],
    ) -> Result<Tls13FinishedOutput, Tls13Denial> {
        if transcript_hash.iter().all(|byte| *byte == 0) {
            return Err(Tls13Denial::ZeroTranscriptHash);
        }
        match mode {
            0 => {
                if self.state != Tls13SessionState::ApplicationKeysDerived {
                    return Err(Tls13Denial::FinishedWrongState);
                }
                if proof.len() != 32 {
                    return Err(Tls13Denial::WrongFinishedProofSize);
                }
                let key = &self
                    .handshake_keys
                    .as_ref()
                    .ok_or(Tls13Denial::FinishedWrongState)?
                    .client_finished_key;
                let output = finished_proof(key, transcript_hash)?;
                self.transition(
                    Tls13SessionState::ClientFinishedReady,
                    "client_finished_proof_emitted",
                );
                Ok(Tls13FinishedOutput::ClientProof(output))
            }
            1 => {
                if self.state != Tls13SessionState::HandshakeKeys {
                    return Err(Tls13Denial::FinishedWrongState);
                }
                if proof.len() != 32 {
                    return Err(Tls13Denial::WrongFinishedProofSize);
                }
                let key = &self
                    .handshake_keys
                    .as_ref()
                    .ok_or(Tls13Denial::FinishedWrongState)?
                    .server_finished_key;
                let valid = verify_finished(key, transcript_hash, proof)?;
                self.evidence.server_finished_observed = true;
                self.evidence.server_finished_valid = valid;
                Ok(Tls13FinishedOutput::ServerVerified(valid))
            }
            _ => Err(Tls13Denial::InvalidFinishedMode),
        }
    }

    pub fn aead_seal(&mut self, header: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, Tls13Denial> {
        let cipher_len = plaintext
            .len()
            .checked_add(TLS13_TAG_LEN)
            .ok_or(Tls13Denial::RecordTooLarge)?;
        validate_record_header(header, cipher_len)?;
        if plaintext.len() > TLS13_MAX_PLAINTEXT_LEN {
            return Err(Tls13Denial::RecordTooLarge);
        }
        if self.send_sequence == u64::MAX {
            return Err(Tls13Denial::SequenceExhausted);
        }
        let (key, iv, activates_application) = match self.state {
            Tls13SessionState::HandshakeKeys => {
                let keys = self
                    .handshake_keys
                    .as_ref()
                    .ok_or(Tls13Denial::AeadWrongState)?;
                (&keys.client_key, &keys.client_iv, false)
            }
            Tls13SessionState::ClientFinishedReady => {
                let keys = self
                    .handshake_keys
                    .as_ref()
                    .ok_or(Tls13Denial::AeadWrongState)?;
                (&keys.client_key, &keys.client_iv, true)
            }
            Tls13SessionState::ApplicationTraffic => {
                let keys = self
                    .application_keys
                    .as_ref()
                    .ok_or(Tls13Denial::AeadWrongState)?;
                (&keys.client_key, &keys.client_iv, false)
            }
            _ => return Err(Tls13Denial::AeadWrongState),
        };
        let mut output = plaintext.to_vec();
        let nonce = sequence_nonce(iv, self.send_sequence);
        let tag = Aes128Gcm::new_from_slice(key)
            .map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?
            .encrypt_in_place_detached(Nonce::from_slice(&nonce), header, &mut output)
            .map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?;
        output.extend_from_slice(tag.as_slice());
        self.send_sequence += 1;
        if activates_application {
            self.send_sequence = 0;
            self.transition(
                Tls13SessionState::ApplicationTraffic,
                "client_finished_sealed_application_traffic_active",
            );
        }
        Ok(output)
    }

    pub fn aead_open(&mut self, header: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, Tls13Denial> {
        validate_record_header(header, ciphertext.len())?;
        if ciphertext.len() < TLS13_TAG_LEN {
            return Err(Tls13Denial::RecordLengthMismatch);
        }
        if self.receive_sequence == u64::MAX {
            return Err(Tls13Denial::SequenceExhausted);
        }
        let keys = match self.state {
            Tls13SessionState::HandshakeKeys => self.handshake_keys.as_ref(),
            Tls13SessionState::ApplicationKeysDerived
            | Tls13SessionState::ClientFinishedReady
            | Tls13SessionState::ApplicationTraffic => self.application_keys.as_ref(),
            _ => return Err(Tls13Denial::AeadWrongState),
        }
        .ok_or(Tls13Denial::AeadWrongState)?;
        let split = ciphertext.len() - TLS13_TAG_LEN;
        let mut output = ciphertext[..split].to_vec();
        let nonce = sequence_nonce(&keys.server_iv, self.receive_sequence);
        Aes128Gcm::new_from_slice(&keys.server_key)
            .map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?
            .decrypt_in_place_detached(
                Nonce::from_slice(&nonce),
                header,
                &mut output,
                Tag::from_slice(&ciphertext[split..]),
            )
            .map_err(|_| Tls13Denial::TagAuthenticationFailed)?;
        self.receive_sequence += 1;
        Ok(output)
    }

    pub const fn state(&self) -> Tls13SessionState {
        self.state
    }

    pub const fn evidence(&self) -> Tls13Evidence {
        self.evidence
    }

    pub const fn sequences(&self) -> (u64, u64) {
        (self.send_sequence, self.receive_sequence)
    }

    pub fn close(&mut self) {
        self.private_key = None;
        self.handshake_keys = None;
        self.application_keys = None;
        self.handshake_secret.zeroize();
        if let Some(pin) = self.expected_spki_pin.as_mut() {
            pin.zeroize();
        }
        self.expected_spki_pin = None;
        self.send_sequence = 0;
        self.receive_sequence = 0;
        self.state = Tls13SessionState::Closed;
    }

    fn transition(&mut self, state: Tls13SessionState, name: &'static str) {
        self.state = state;
        self.evidence.transition_count = self.evidence.transition_count.saturating_add(1);
        self.evidence.last_transition = name;
    }
}

impl Drop for OpaqueSession {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct OpaqueSessionSlot {
    owner_invocation_id: u64,
    owner_connection_handle: i32,
    generation: u32,
    session: Option<OpaqueSession>,
}

impl OpaqueSessionSlot {
    pub const fn new() -> Self {
        Self {
            owner_invocation_id: 0,
            owner_connection_handle: 0,
            generation: 1,
            session: None,
        }
    }

    pub fn open(
        &mut self,
        owner_invocation_id: u64,
        owner_connection_handle: i32,
        entropy: &mut [u8; 64],
        expected_spki_pin: Option<[u8; 32]>,
    ) -> Result<(i32, [u8; TLS13_PUBLIC_OUTPUT_LEN]), Tls13Denial> {
        if self.session.is_some() {
            return Err(Tls13Denial::DuplicateSession);
        }
        let (session, public_output) = OpaqueSession::open(entropy, expected_spki_pin)?;
        self.owner_invocation_id = owner_invocation_id;
        self.owner_connection_handle = owner_connection_handle;
        self.session = Some(session);
        Ok((self.handle(), public_output))
    }

    pub fn get_mut(
        &mut self,
        owner_invocation_id: u64,
        handle: i32,
    ) -> Result<&mut OpaqueSession, Tls13Denial> {
        if owner_invocation_id != self.owner_invocation_id {
            return Err(Tls13Denial::ForeignSession);
        }
        if handle <= 0 || handle != self.handle() || self.session.is_none() {
            return Err(Tls13Denial::StaleSession);
        }
        self.session.as_mut().ok_or(Tls13Denial::StaleSession)
    }

    pub fn close_owner(&mut self, owner_invocation_id: u64) -> bool {
        if self.session.is_none() || owner_invocation_id != self.owner_invocation_id {
            return false;
        }
        if let Some(mut session) = self.session.take() {
            session.close();
        }
        self.owner_connection_handle = 0;
        self.generation = self.generation.wrapping_add(1).max(1);
        true
    }

    pub const fn is_open(&self) -> bool {
        self.session.is_some()
    }

    pub const fn owner_connection_handle(&self) -> i32 {
        self.owner_connection_handle
    }

    fn handle(&self) -> i32 {
        (((self.generation & 0x007f_ffff) << 8) | TLS13_HANDLE_SLOT) as i32
    }
}

impl Default for OpaqueSessionSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OpaqueSessionSlot {
    fn drop(&mut self) {
        let owner = self.owner_invocation_id;
        let _ = self.close_owner(owner);
    }
}

pub fn validate_record_header(header: &[u8], payload_len: usize) -> Result<(), Tls13Denial> {
    if header.len() != TLS13_RECORD_HEADER_LEN
        || header[0] != 0x17
        || header[1] != 0x03
        || header[2] != 0x03
    {
        return Err(Tls13Denial::InvalidRecordHeader);
    }
    if payload_len > TLS13_MAX_CIPHERTEXT_LEN {
        return Err(Tls13Denial::RecordTooLarge);
    }
    if usize::from(u16::from_be_bytes([header[3], header[4]])) != payload_len {
        return Err(Tls13Denial::RecordLengthMismatch);
    }
    Ok(())
}

struct HandshakeSchedule {
    handshake_secret: [u8; 32],
    keys: TrafficKeys,
}

fn derive_handshake_schedule(
    shared_secret: &[u8],
    hello_transcript_hash: &[u8; 32],
) -> Result<HandshakeSchedule, Tls13Denial> {
    let zeros = [0u8; 32];
    let early_secret = Zeroizing::new(hkdf_extract(&zeros, &zeros));
    let empty_hash = digest(&[]);
    let derived = Zeroizing::new(derive_secret(&early_secret, b"derived", &empty_hash)?);
    let handshake_secret = hkdf_extract(&derived[..], shared_secret);
    let client_traffic = Zeroizing::new(derive_secret(
        &handshake_secret,
        b"c hs traffic",
        hello_transcript_hash,
    )?);
    let server_traffic = Zeroizing::new(derive_secret(
        &handshake_secret,
        b"s hs traffic",
        hello_transcript_hash,
    )?);
    Ok(HandshakeSchedule {
        handshake_secret,
        keys: derive_traffic_keys(&client_traffic, &server_traffic)?,
    })
}

fn derive_application_keys(
    handshake_secret: &[u8; 32],
    transcript_hash: &[u8; 32],
) -> Result<TrafficKeys, Tls13Denial> {
    let empty_hash = digest(&[]);
    let derived = Zeroizing::new(derive_secret(handshake_secret, b"derived", &empty_hash)?);
    let master_secret = Zeroizing::new(hkdf_extract(&derived[..], &[0u8; 32]));
    let client_traffic = Zeroizing::new(derive_secret(
        &master_secret,
        b"c ap traffic",
        transcript_hash,
    )?);
    let server_traffic = Zeroizing::new(derive_secret(
        &master_secret,
        b"s ap traffic",
        transcript_hash,
    )?);
    derive_traffic_keys(&client_traffic, &server_traffic)
}

fn derive_traffic_keys(
    client_traffic: &[u8; 32],
    server_traffic: &[u8; 32],
) -> Result<TrafficKeys, Tls13Denial> {
    let mut keys = TrafficKeys {
        client_key: [0; 16],
        server_key: [0; 16],
        client_iv: [0; 12],
        server_iv: [0; 12],
        client_finished_key: [0; 32],
        server_finished_key: [0; 32],
    };
    hkdf_expand_label(client_traffic, b"key", &[], &mut keys.client_key)?;
    hkdf_expand_label(server_traffic, b"key", &[], &mut keys.server_key)?;
    hkdf_expand_label(client_traffic, b"iv", &[], &mut keys.client_iv)?;
    hkdf_expand_label(server_traffic, b"iv", &[], &mut keys.server_iv)?;
    hkdf_expand_label(
        client_traffic,
        b"finished",
        &[],
        &mut keys.client_finished_key,
    )?;
    hkdf_expand_label(
        server_traffic,
        b"finished",
        &[],
        &mut keys.server_finished_key,
    )?;
    Ok(keys)
}

fn hkdf_extract(salt: &[u8], input: &[u8]) -> [u8; 32] {
    let (prk, _) = Hkdf::<Sha256>::extract(Some(salt), input);
    let mut output = [0u8; 32];
    output.copy_from_slice(prk.as_slice());
    output
}

fn derive_secret(
    secret: &[u8; 32],
    label: &[u8],
    transcript_hash: &[u8; 32],
) -> Result<[u8; 32], Tls13Denial> {
    let mut output = [0u8; 32];
    hkdf_expand_label(secret, label, transcript_hash, &mut output)?;
    Ok(output)
}

fn hkdf_expand_label(
    secret: &[u8],
    label: &[u8],
    context: &[u8],
    output: &mut [u8],
) -> Result<(), Tls13Denial> {
    let full_label_len = 6usize
        .checked_add(label.len())
        .ok_or(Tls13Denial::CryptoPrimitiveFailed)?;
    if full_label_len > u8::MAX as usize
        || context.len() > u8::MAX as usize
        || output.len() > u16::MAX as usize
    {
        return Err(Tls13Denial::CryptoPrimitiveFailed);
    }
    let mut info = Vec::with_capacity(4 + full_label_len + context.len());
    info.extend_from_slice(&(output.len() as u16).to_be_bytes());
    info.push(full_label_len as u8);
    info.extend_from_slice(b"tls13 ");
    info.extend_from_slice(label);
    info.push(context.len() as u8);
    info.extend_from_slice(context);
    Hkdf::<Sha256>::from_prk(secret)
        .map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?
        .expand(&info, output)
        .map_err(|_| Tls13Denial::CryptoPrimitiveFailed)
}

fn finished_proof(
    finished_key: &[u8; 32],
    transcript_hash: &[u8; 32],
) -> Result<[u8; 32], Tls13Denial> {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(finished_key)
        .map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?;
    mac.update(transcript_hash);
    let mut output = [0u8; 32];
    output.copy_from_slice(mac.finalize().into_bytes().as_slice());
    Ok(output)
}

fn verify_finished(
    finished_key: &[u8; 32],
    transcript_hash: &[u8; 32],
    proof: &[u8],
) -> Result<bool, Tls13Denial> {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(finished_key)
        .map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?;
    mac.update(transcript_hash);
    Ok(mac.verify_slice(proof).is_ok())
}

fn sequence_nonce(iv: &[u8; 12], sequence: u64) -> [u8; 12] {
    let mut nonce = *iv;
    for (target, source) in nonce[4..].iter_mut().zip(sequence.to_be_bytes()) {
        *target ^= source;
    }
    nonce
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut output = [0u8; 32];
    output.copy_from_slice(&digest);
    output
}

#[derive(Clone, Copy)]
pub struct Tls13HostVectorReport {
    pub import_outcomes: [&'static str; 8],
    pub all_eight_positive: bool,
    pub rfc8448_key_schedule_valid: bool,
    pub record_framing_valid: bool,
    pub state_machine_valid: bool,
    pub sequence_rules_valid: bool,
    pub foreign_stale_state_size_negatives: bool,
    pub core_negative_vectors_positive: bool,
    pub tag_failure_preserved_sequence: bool,
    pub session_zeroized: bool,
    pub public_output_sha256: [u8; 32],
    pub hello_transcript_sha256: [u8; 32],
    pub application_transcript_sha256: [u8; 32],
    pub spki_sha256: [u8; 32],
    pub message_sha256: [u8; 32],
    pub signature_sha256: [u8; 32],
    pub math_valid: bool,
    pub pin_match: bool,
    pub server_finished_valid: bool,
    pub trust_label_granted_by_guest: bool,
    pub transition_count: u32,
    pub final_state: &'static str,
    pub send_sequence: u64,
    pub receive_sequence: u64,
    pub typed_denials: [Tls13Denial; 35],
}

impl Tls13HostVectorReport {
    const fn failed() -> Self {
        Self {
            import_outcomes: ["not_observed"; 8],
            all_eight_positive: false,
            rfc8448_key_schedule_valid: false,
            record_framing_valid: false,
            state_machine_valid: false,
            sequence_rules_valid: false,
            foreign_stale_state_size_negatives: false,
            core_negative_vectors_positive: false,
            tag_failure_preserved_sequence: false,
            session_zeroized: false,
            public_output_sha256: [0; 32],
            hello_transcript_sha256: [0; 32],
            application_transcript_sha256: [0; 32],
            spki_sha256: [0; 32],
            message_sha256: [0; 32],
            signature_sha256: [0; 32],
            math_valid: false,
            pin_match: false,
            server_finished_valid: false,
            trust_label_granted_by_guest: false,
            transition_count: 0,
            final_state: "failed",
            send_sequence: 0,
            receive_sequence: 0,
            typed_denials: ALL_TLS13_DENIALS,
        }
    }
}

/// Builds the SubjectPublicKeyInfo DER for a P-256 verifying key: the fixed
/// 26-byte RFC 5480 header (SEQUENCE, AlgorithmIdentifier{id-ecPublicKey,
/// prime256v1}, BIT STRING) followed by the uncompressed SEC1 point.
///
/// This is DER FRAMING for the vector, not a crypto primitive, and it is
/// self-checking: the SPKI is handed straight to `p256_verify`, whose real
/// `DecodePublicKey` parses it — a wrong byte fails the vector loudly instead
/// of passing falsely. The alternative, `to_public_key_der()`, is gated behind
/// the ecdsa crate's `pem` feature, which would pull PEM/base64 encoding into
/// the permanent core to serve one test vector.
fn p256_spki_der(key: &VerifyingKey) -> Vec<u8> {
    const P256_SPKI_HEADER: [u8; 26] = [
        0x30, 0x59, // SEQUENCE, 89 bytes
        0x30, 0x13, // SEQUENCE, 19 bytes (AlgorithmIdentifier)
        0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, // OID id-ecPublicKey
        0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, // OID prime256v1
        0x03, 0x42, 0x00, // BIT STRING, 66 bytes, 0 unused bits
    ];
    let point = key.to_encoded_point(false);
    let mut der = Vec::with_capacity(P256_SPKI_HEADER.len() + point.as_bytes().len());
    der.extend_from_slice(&P256_SPKI_HEADER);
    der.extend_from_slice(point.as_bytes());
    der
}

/// Runs deterministic, core-owned vectors for the read-only in-guest probe.
/// Test private material never crosses this function boundary.
pub fn run_tls13_host_vectors() -> Tls13HostVectorReport {
    run_tls13_host_vectors_inner().unwrap_or_else(|_| Tls13HostVectorReport::failed())
}

fn run_tls13_host_vectors_inner() -> Result<Tls13HostVectorReport, Tls13Denial> {
    let hello_hash = digest(b"raios tls13 hello transcript");
    let server_finished_hash = digest(b"raios tls13 server finished transcript");
    let application_hash = digest(b"raios tls13 application transcript");
    let client_finished_hash = digest(b"raios tls13 client finished transcript");
    let verify_message = b"TLS 1.3, server CertificateVerify\0raios";

    let signing_secret =
        SecretKey::from_slice(&scalar_bytes(3)).map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?;
    let signing_key = SigningKey::from(signing_secret);
    let spki = p256_spki_der(signing_key.verifying_key());
    let signature: Signature = signing_key.sign(verify_message);
    let signature_der = signature.to_der();
    let spki_pin = digest(&spki);

    let mut entropy = [0x5au8; 64];
    entropy[32..].copy_from_slice(&scalar_bytes(1));
    let mut slot = OpaqueSessionSlot::new();
    let (handle, public_output) = slot.open(7, 0x701, &mut entropy, Some(spki_pin))?;
    let mut duplicate_entropy = [0x33; 64];
    let duplicate_denial = slot
        .open(7, 0x701, &mut duplicate_entropy, Some(spki_pin))
        .unwrap_err();
    duplicate_entropy.zeroize();

    let peer_secret =
        SecretKey::from_slice(&scalar_bytes(2)).map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?;
    let peer_public = peer_secret.public_key();
    let peer_encoded = peer_public.to_encoded_point(false);
    let client_secret =
        SecretKey::from_slice(&scalar_bytes(1)).map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?;
    let shared = diffie_hellman(client_secret.to_nonzero_scalar(), peer_public.as_affine());
    let mut schedule = derive_handshake_schedule(shared.raw_secret_bytes(), &hello_hash)?;

    let session = slot.get_mut(7, handle)?;
    let sha_ok = session.sha256(b"public transcript")? == digest(b"public transcript");
    session.handshake_keys(peer_encoded.as_bytes(), &hello_hash)?;
    let math_valid = session.p256_verify(&spki, verify_message, signature_der.as_bytes())?;

    let server_finished =
        finished_proof(&schedule.keys.server_finished_key, &server_finished_hash)?;
    let server_finished_header = record_header(server_finished.len() + TLS13_TAG_LEN)?;
    let server_finished_ciphertext = seal_raw(
        &schedule.keys.server_key,
        &schedule.keys.server_iv,
        0,
        &server_finished_header,
        &server_finished,
    )?;
    let opened_finished =
        session.aead_open(&server_finished_header, &server_finished_ciphertext)?;
    let server_finished_valid =
        match session.finished(1, &server_finished_hash, &opened_finished)? {
            Tls13FinishedOutput::ServerVerified(valid) => valid,
            Tls13FinishedOutput::ClientProof(_) => false,
        };
    let application_schedule =
        derive_application_keys(&schedule.handshake_secret, &application_hash)?;
    schedule.handshake_secret.zeroize();
    session.application_keys(&application_hash)?;
    let client_finished = match session.finished(0, &client_finished_hash, &[0; 32])? {
        Tls13FinishedOutput::ClientProof(proof) => proof,
        Tls13FinishedOutput::ServerVerified(_) => return Err(Tls13Denial::CryptoPrimitiveFailed),
    };
    let client_finished_header = record_header(client_finished.len() + TLS13_TAG_LEN)?;
    let sealed_finished = session.aead_seal(&client_finished_header, &client_finished)?;

    let application_plaintext = b"bounded application record";
    let application_header = record_header(application_plaintext.len() + TLS13_TAG_LEN)?;
    let sealed_application = session.aead_seal(&application_header, application_plaintext)?;
    let server_application_ciphertext = seal_raw(
        &application_schedule.server_key,
        &application_schedule.server_iv,
        0,
        &application_header,
        application_plaintext,
    )?;
    let opened_application =
        session.aead_open(&application_header, &server_application_ciphertext)?;
    let sequences_before_bad_tag = session.sequences();
    let mut bad_tag = server_application_ciphertext.clone();
    if let Some(last) = bad_tag.last_mut() {
        *last ^= 1;
    }
    let tag_denial = session
        .aead_open(&application_header, &bad_tag)
        .unwrap_err();
    let tag_failure_preserved_sequence = session.sequences() == sequences_before_bad_tag;
    let evidence = session.evidence();
    let (send_sequence, receive_sequence) = session.sequences();
    let state = session.state();

    let foreign_denial = slot
        .get_mut(8, handle)
        .err()
        .ok_or(Tls13Denial::CryptoPrimitiveFailed)?;
    let session_zeroized = slot.close_owner(7);
    let stale_denial = slot
        .get_mut(7, handle)
        .err()
        .ok_or(Tls13Denial::CryptoPrimitiveFailed)?;
    let mut unopened_entropy = [0x44; 64];
    unopened_entropy[32..].copy_from_slice(&scalar_bytes(4));
    let (mut unopened, _) = OpaqueSession::open(&mut unopened_entropy, Some(spki_pin))?;
    let state_denial = unopened.application_keys(&application_hash).unwrap_err();
    let size_denial = unopened
        .sha256(&[0; TLS13_MAX_HASH_INPUT_LEN + 1])
        .unwrap_err();
    let peer_denial = unopened.handshake_keys(&[0; 64], &hello_hash).unwrap_err();
    let zero_hash_denial = unopened
        .handshake_keys(peer_encoded.as_bytes(), &[0; 32])
        .unwrap_err();
    let header_denial = validate_record_header(&[0x16, 3, 3, 0, 16], 16).unwrap_err();
    let length_denial = validate_record_header(&[0x17, 3, 3, 0, 15], 16).unwrap_err();
    let oversized_denial =
        validate_record_header(&[0x17, 3, 3, 0xff, 0xff], TLS13_MAX_CIPHERTEXT_LEN + 1)
            .unwrap_err();

    let mut rfc = derive_handshake_schedule(
        &[
            0x8b, 0xd4, 0x05, 0x4f, 0xb5, 0x5b, 0x9d, 0x63, 0xfd, 0xfb, 0xac, 0xf9, 0xf0, 0x4b,
            0x9f, 0x0d, 0x35, 0xe6, 0xd6, 0x3f, 0x53, 0x75, 0x63, 0xef, 0xd4, 0x62, 0x72, 0x90,
            0x0f, 0x89, 0x49, 0x2d,
        ],
        &[
            0x86, 0x0c, 0x06, 0xed, 0xc0, 0x78, 0x58, 0xee, 0x8e, 0x78, 0xf0, 0xe7, 0x42, 0x8c,
            0x58, 0xed, 0xd6, 0xb4, 0x3f, 0x2c, 0xa3, 0xe6, 0xe9, 0x5f, 0x02, 0xed, 0x06, 0x3c,
            0xf0, 0xe1, 0xca, 0xd8,
        ],
    )?;
    let rfc8448_key_schedule_valid = rfc.keys.client_key
        == [
            0xdb, 0xfa, 0xa6, 0x93, 0xd1, 0x76, 0x2c, 0x5b, 0x66, 0x6a, 0xf5, 0xd9, 0x50, 0x25,
            0x8d, 0x01,
        ];
    rfc.handshake_secret.zeroize();

    Ok(Tls13HostVectorReport {
        import_outcomes: [
            "session_handle_and_public_bytes",
            if sha_ok { "digest32" } else { "failed" },
            if math_valid { "math_valid" } else { "invalid" },
            "handshake_keys_opaque",
            "application_keys_opaque",
            if server_finished_valid {
                "server_verified_client_proof"
            } else {
                "failed"
            },
            if !sealed_finished.is_empty() && !sealed_application.is_empty() {
                "ciphertext"
            } else {
                "failed"
            },
            if opened_application == application_plaintext {
                "plaintext"
            } else {
                "failed"
            },
        ],
        all_eight_positive: sha_ok
            && math_valid
            && server_finished_valid
            && opened_application == application_plaintext,
        rfc8448_key_schedule_valid,
        record_framing_valid: sealed_application.len() == application_plaintext.len() + 16,
        state_machine_valid: state == Tls13SessionState::ApplicationTraffic,
        sequence_rules_valid: send_sequence == 1 && receive_sequence == 1,
        foreign_stale_state_size_negatives: duplicate_denial == Tls13Denial::DuplicateSession
            && foreign_denial == Tls13Denial::ForeignSession
            && stale_denial == Tls13Denial::StaleSession
            && state_denial == Tls13Denial::RepeatedOrOutOfOrderTransition
            && size_denial == Tls13Denial::HashInputTooLarge,
        core_negative_vectors_positive: peer_denial == Tls13Denial::WrongPeerKeyEncoding
            && zero_hash_denial == Tls13Denial::ZeroTranscriptHash
            && header_denial == Tls13Denial::InvalidRecordHeader
            && length_denial == Tls13Denial::RecordLengthMismatch
            && oversized_denial == Tls13Denial::RecordTooLarge
            && tag_denial == Tls13Denial::TagAuthenticationFailed,
        tag_failure_preserved_sequence,
        session_zeroized,
        public_output_sha256: digest(&public_output),
        hello_transcript_sha256: hello_hash,
        application_transcript_sha256: application_hash,
        spki_sha256: evidence.spki_sha256,
        message_sha256: evidence.verify_message_sha256,
        signature_sha256: evidence.signature_sha256,
        math_valid: evidence.math_valid,
        pin_match: evidence.pin_match,
        server_finished_valid: evidence.server_finished_valid,
        trust_label_granted_by_guest: evidence.trust_label_granted_by_guest,
        transition_count: evidence.transition_count,
        final_state: state.as_str(),
        send_sequence,
        receive_sequence,
        typed_denials: ALL_TLS13_DENIALS,
    })
}

fn scalar_bytes(value: u8) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[31] = value;
    bytes
}

fn record_header(payload_len: usize) -> Result<[u8; 5], Tls13Denial> {
    let length = u16::try_from(payload_len).map_err(|_| Tls13Denial::RecordTooLarge)?;
    let [hi, lo] = length.to_be_bytes();
    Ok([0x17, 0x03, 0x03, hi, lo])
}

fn seal_raw(
    key: &[u8; 16],
    iv: &[u8; 12],
    sequence: u64,
    header: &[u8; 5],
    plaintext: &[u8],
) -> Result<Vec<u8>, Tls13Denial> {
    let mut output = plaintext.to_vec();
    let nonce = sequence_nonce(iv, sequence);
    let tag = Aes128Gcm::new_from_slice(key)
        .map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?
        .encrypt_in_place_detached(Nonce::from_slice(&nonce), header, &mut output)
        .map_err(|_| Tls13Denial::CryptoPrimitiveFailed)?;
    output.extend_from_slice(tag.as_slice());
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC8448_SHARED_SECRET: [u8; 32] = [
        0x8b, 0xd4, 0x05, 0x4f, 0xb5, 0x5b, 0x9d, 0x63, 0xfd, 0xfb, 0xac, 0xf9, 0xf0, 0x4b, 0x9f,
        0x0d, 0x35, 0xe6, 0xd6, 0x3f, 0x53, 0x75, 0x63, 0xef, 0xd4, 0x62, 0x72, 0x90, 0x0f, 0x89,
        0x49, 0x2d,
    ];
    const RFC8448_HELLO_HASH: [u8; 32] = [
        0x86, 0x0c, 0x06, 0xed, 0xc0, 0x78, 0x58, 0xee, 0x8e, 0x78, 0xf0, 0xe7, 0x42, 0x8c, 0x58,
        0xed, 0xd6, 0xb4, 0x3f, 0x2c, 0xa3, 0xe6, 0xe9, 0x5f, 0x02, 0xed, 0x06, 0x3c, 0xf0, 0xe1,
        0xca, 0xd8,
    ];

    fn session() -> (OpaqueSession, [u8; 97]) {
        let mut entropy = [0xabu8; 64];
        entropy[32..].fill(0);
        entropy[63] = 1;
        OpaqueSession::open(&mut entropy, Some([0x44; 32])).unwrap()
    }

    #[test]
    fn rfc8448_handshake_schedule_and_record_header_vectors_match() {
        let schedule = derive_handshake_schedule(&RFC8448_SHARED_SECRET, &RFC8448_HELLO_HASH)
            .expect("RFC schedule");
        assert_eq!(
            schedule.handshake_secret,
            [
                0x1d, 0xc8, 0x26, 0xe9, 0x36, 0x06, 0xaa, 0x6f, 0xdc, 0x0a, 0xad, 0xc1, 0x2f, 0x74,
                0x1b, 0x01, 0x04, 0x6a, 0xa6, 0xb9, 0x9f, 0x69, 0x1e, 0xd2, 0x21, 0xa9, 0xf0, 0xca,
                0x04, 0x3f, 0xbe, 0xac,
            ]
        );
        assert_eq!(
            schedule.keys.client_key,
            [
                0xdb, 0xfa, 0xa6, 0x93, 0xd1, 0x76, 0x2c, 0x5b, 0x66, 0x6a, 0xf5, 0xd9, 0x50, 0x25,
                0x8d, 0x01,
            ]
        );
        assert_eq!(
            schedule.keys.server_key,
            [
                0x3f, 0xce, 0x51, 0x60, 0x09, 0xc2, 0x17, 0x27, 0xd0, 0xf2, 0xe4, 0xe8, 0x6e, 0xe4,
                0x03, 0xbc,
            ]
        );
        assert_eq!(
            validate_record_header(&[0x17, 0x03, 0x03, 0x00, 0x35], 53),
            Ok(())
        );
        assert_eq!(
            validate_record_header(&[0x16, 0x03, 0x03, 0x00, 0x35], 53),
            Err(Tls13Denial::InvalidRecordHeader)
        );
        assert_eq!(
            validate_record_header(&[0x17, 0x03, 0x03, 0x00, 0x34], 53),
            Err(Tls13Denial::RecordLengthMismatch)
        );
    }

    #[test]
    fn opaque_slot_owns_keys_and_rejects_duplicate_foreign_and_stale_handles() {
        let mut slot = OpaqueSessionSlot::new();
        let mut entropy = [0x11; 64];
        entropy[32..].fill(0);
        entropy[63] = 1;
        let (handle, public) = slot.open(7, 99, &mut entropy, Some([3; 32])).unwrap();
        assert_eq!(public.len(), TLS13_PUBLIC_OUTPUT_LEN);
        let mut second = [0x22; 64];
        assert_eq!(
            slot.open(7, 99, &mut second, Some([3; 32])).unwrap_err(),
            Tls13Denial::DuplicateSession
        );
        assert!(matches!(
            slot.get_mut(8, handle),
            Err(Tls13Denial::ForeignSession)
        ));
        assert!(slot.close_owner(7));
        assert!(matches!(
            slot.get_mut(7, handle),
            Err(Tls13Denial::StaleSession)
        ));
    }

    #[test]
    fn state_size_and_sequence_negatives_fail_closed() {
        let (mut session, _) = session();
        assert_eq!(
            session.application_keys(&[1; 32]),
            Err(Tls13Denial::RepeatedOrOutOfOrderTransition)
        );
        assert_eq!(
            session.handshake_keys(&[4; 64], &[1; 32]),
            Err(Tls13Denial::WrongPeerKeyEncoding)
        );
        assert_eq!(
            session.sha256(&[0; TLS13_MAX_HASH_INPUT_LEN + 1]),
            Err(Tls13Denial::HashInputTooLarge)
        );
        assert_eq!(
            session.aead_seal(&[0x17, 0x03, 0x03, 0, 16], &[]),
            Err(Tls13Denial::AeadWrongState)
        );
        session.send_sequence = u64::MAX;
        session.receive_sequence = u64::MAX;
        session.state = Tls13SessionState::HandshakeKeys;
        assert_eq!(
            session.aead_seal(&[0x17, 0x03, 0x03, 0, 16], &[]),
            Err(Tls13Denial::SequenceExhausted)
        );
        assert_eq!(session.send_sequence, u64::MAX);
        assert_eq!(
            session.aead_open(&[0x17, 0x03, 0x03, 0, 16], &[0; 16]),
            Err(Tls13Denial::SequenceExhausted)
        );
        assert_eq!(session.receive_sequence, u64::MAX);
    }

    #[test]
    fn application_evidence_and_finished_refusals_are_distinct() {
        let (mut session, _) = session();
        session.state = Tls13SessionState::HandshakeKeys;
        assert_eq!(
            session.application_keys(&[1; 32]),
            Err(Tls13Denial::CertificateVerifyEvidenceMissing)
        );
        session.evidence.certificate_verify_observed = true;
        assert_eq!(
            session.application_keys(&[1; 32]),
            Err(Tls13Denial::CertificateVerifyMathInvalid)
        );
        session.evidence.math_valid = true;
        assert_eq!(
            session.application_keys(&[1; 32]),
            Err(Tls13Denial::SourcePinMismatch)
        );
        session.evidence.pin_match = true;
        assert_eq!(
            session.application_keys(&[1; 32]),
            Err(Tls13Denial::ServerFinishedEvidenceMissing)
        );
        session.evidence.server_finished_observed = true;
        assert_eq!(
            session.application_keys(&[1; 32]),
            Err(Tls13Denial::ServerFinishedEvidenceInvalid)
        );
        assert!(matches!(
            session.finished(2, &[1; 32], &[0; 32]),
            Err(Tls13Denial::InvalidFinishedMode)
        ));
        assert!(matches!(
            session.finished(1, &[1; 32], &[0; 31]),
            Err(Tls13Denial::WrongFinishedProofSize)
        ));
        session.state = Tls13SessionState::Opened;
        assert!(matches!(
            session.finished(1, &[1; 32], &[0; 32]),
            Err(Tls13Denial::FinishedWrongState)
        ));
    }

    #[test]
    fn close_removes_every_retained_secret() {
        let (mut session, _) = session();
        session.handshake_secret.fill(0x77);
        session.close();
        assert!(session.private_key.is_none());
        assert!(session.handshake_secret.iter().all(|byte| *byte == 0));
        assert!(session.handshake_keys.is_none() && session.application_keys.is_none());
        assert_eq!(session.sequences(), (0, 0));
        assert_eq!(session.state(), Tls13SessionState::Closed);
    }

    #[test]
    fn denial_strings_are_pairwise_distinct() {
        for (index, denial) in ALL_TLS13_DENIALS.iter().enumerate() {
            assert!(ALL_TLS13_DENIALS[index + 1..]
                .iter()
                .all(|other| denial.as_str() != other.as_str()));
        }
    }

    #[test]
    fn host_vectors_cover_all_imports_and_fail_closed_edges() {
        let report = run_tls13_host_vectors();
        assert!(report.all_eight_positive);
        assert!(report.rfc8448_key_schedule_valid);
        assert!(report.record_framing_valid && report.state_machine_valid);
        assert!(report.sequence_rules_valid && report.tag_failure_preserved_sequence);
        assert!(report.session_zeroized);
        assert!(report.foreign_stale_state_size_negatives);
        assert!(report.core_negative_vectors_positive);
        assert!(report.math_valid && report.pin_match && report.server_finished_valid);
        assert!(!report.trust_label_granted_by_guest);
        assert_eq!(report.final_state, "application_traffic");
        assert_eq!((report.send_sequence, report.receive_sequence), (1, 1));
        for (index, denial) in report.typed_denials.iter().enumerate() {
            assert!(report.typed_denials[index + 1..]
                .iter()
                .all(|other| denial.as_str() != other.as_str()));
        }
    }
}
