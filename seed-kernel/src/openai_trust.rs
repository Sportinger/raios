use embedded_tls::blocking::{Aes128GcmSha256, Certificate, TlsCipherSuite, TlsVerifier};
use embedded_tls::{CertificateRef, CertificateVerify, SignatureScheme, TlsError};
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::provider_trust;

const EXPECTED_HOST: &str = "api.openai.com";
const TLS13_CERT_VERIFY_CONTEXT: &[u8] = b"TLS 1.3, server CertificateVerify\x00";
const P256_UNCOMPRESSED_POINT_LEN: usize = 65;
const OID_EC_PUBLIC_KEY: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01];
const OID_PRIME256V1: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07];

pub struct OpenAiPinnedCertVerifier {
    host_ok: bool,
    transcript: Option<<Aes128GcmSha256 as TlsCipherSuite>::Hash>,
    public_key: [u8; P256_UNCOMPRESSED_POINT_LEN],
    public_key_len: usize,
    pin_matched: bool,
}

impl OpenAiPinnedCertVerifier {
    fn reject(state: provider_trust::TrustState, error: TlsError) -> Result<(), TlsError> {
        match state {
            provider_trust::TrustState::PinVerifierUnavailable => {
                provider_trust::mark_pin_verifier_unavailable()
            }
            _ => provider_trust::mark_pin_mismatch(),
        }
        Err(error)
    }
}

impl<'a> TlsVerifier<'a, Aes128GcmSha256> for OpenAiPinnedCertVerifier {
    fn new(host: Option<&'a str>) -> Self {
        Self {
            host_ok: host == Some(EXPECTED_HOST),
            transcript: None,
            public_key: [0; P256_UNCOMPRESSED_POINT_LEN],
            public_key_len: 0,
            pin_matched: false,
        }
    }

    fn verify_certificate(
        &mut self,
        transcript: &<Aes128GcmSha256 as TlsCipherSuite>::Hash,
        _ca: &Option<Certificate>,
        cert: CertificateRef,
    ) -> Result<(), TlsError> {
        if !self.host_ok {
            return Self::reject(
                provider_trust::TrustState::PinMismatch,
                TlsError::InvalidCertificate,
            );
        }

        let expected_pin = match provider_trust::openai_cert_pin_bytes() {
            Ok(pin) => pin,
            Err(state) => return Self::reject(state, TlsError::InvalidCertificate),
        };
        let leaf = cert.leaf_x509_der().ok_or(TlsError::InvalidCertificate)?;
        let actual_pin = Sha256::digest(leaf);
        if actual_pin.as_slice() != expected_pin {
            return Self::reject(
                provider_trust::TrustState::PinMismatch,
                TlsError::InvalidCertificate,
            );
        }

        let public_key = extract_p256_public_key(leaf).ok_or_else(|| {
            provider_trust::mark_pin_verifier_unavailable();
            TlsError::InvalidCertificate
        })?;

        self.public_key.copy_from_slice(public_key);
        self.public_key_len = public_key.len();
        self.pin_matched = true;
        self.transcript.replace(transcript.clone());
        Ok(())
    }

    fn verify_signature(&mut self, verify: CertificateVerify) -> Result<(), TlsError> {
        if !self.pin_matched {
            return Self::reject(
                provider_trust::TrustState::PinMismatch,
                TlsError::InvalidSignature,
            );
        }
        if verify.signature_scheme() != SignatureScheme::EcdsaSecp256r1Sha256 {
            return Self::reject(
                provider_trust::TrustState::PinVerifierUnavailable,
                TlsError::InvalidSignatureScheme,
            );
        }

        let Some(transcript) = self.transcript.take() else {
            return Self::reject(
                provider_trust::TrustState::PinMismatch,
                TlsError::InvalidHandshake,
            );
        };
        let transcript_hash = transcript.finalize();
        let mut message = [0u8; 64 + TLS13_CERT_VERIFY_CONTEXT.len() + 32];
        message[..64].fill(0x20);
        message[64..64 + TLS13_CERT_VERIFY_CONTEXT.len()]
            .copy_from_slice(TLS13_CERT_VERIFY_CONTEXT);
        message[64 + TLS13_CERT_VERIFY_CONTEXT.len()..].copy_from_slice(transcript_hash.as_slice());

        let verifying_key = VerifyingKey::from_sec1_bytes(&self.public_key[..self.public_key_len])
            .map_err(|_| {
                provider_trust::mark_pin_verifier_unavailable();
                TlsError::DecodeError
            })?;
        let signature = Signature::from_der(verify.signature()).map_err(|_| {
            provider_trust::mark_pin_verifier_unavailable();
            TlsError::DecodeError
        })?;
        if verifying_key.verify(&message, &signature).is_err() {
            provider_trust::mark_pin_mismatch();
            return Err(TlsError::InvalidSignature);
        }

        provider_trust::mark_pinned_cert_verified();
        Ok(())
    }
}

fn extract_p256_public_key(cert_der: &[u8]) -> Option<&[u8]> {
    let cert = read_single_tlv(cert_der, 0x30)?;
    let mut cert_reader = DerReader::new(cert.value);
    let tbs = cert_reader.read_tlv(0x30)?;
    let mut tbs_reader = DerReader::new(tbs.value);

    if tbs_reader.peek_tag()? == 0xa0 {
        tbs_reader.read_any()?;
    }
    tbs_reader.read_any()?;
    tbs_reader.read_tlv(0x30)?;
    tbs_reader.read_tlv(0x30)?;
    tbs_reader.read_tlv(0x30)?;
    tbs_reader.read_tlv(0x30)?;

    let spki = tbs_reader.read_tlv(0x30)?;
    let mut spki_reader = DerReader::new(spki.value);
    let algorithm = spki_reader.read_tlv(0x30)?;
    if !algorithm_is_p256_ec(algorithm.value) {
        return None;
    }
    let public_key = spki_reader.read_tlv(0x03)?;
    let (&unused_bits, key) = public_key.value.split_first()?;
    if unused_bits != 0
        || key.len() != P256_UNCOMPRESSED_POINT_LEN
        || key.first().copied() != Some(0x04)
    {
        return None;
    }
    Some(key)
}

fn algorithm_is_p256_ec(algorithm_der: &[u8]) -> bool {
    let mut reader = DerReader::new(algorithm_der);
    let Some(ec_public_key) = reader.read_tlv(0x06) else {
        return false;
    };
    let Some(prime256v1) = reader.read_tlv(0x06) else {
        return false;
    };
    ec_public_key.value == OID_EC_PUBLIC_KEY && prime256v1.value == OID_PRIME256V1
}

fn read_single_tlv(input: &[u8], expected_tag: u8) -> Option<DerTlv<'_>> {
    let mut reader = DerReader::new(input);
    let value = reader.read_tlv(expected_tag)?;
    if reader.is_empty() {
        Some(value)
    } else {
        None
    }
}

struct DerReader<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> DerReader<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, offset: 0 }
    }

    fn is_empty(&self) -> bool {
        self.offset == self.input.len()
    }

    fn peek_tag(&self) -> Option<u8> {
        self.input.get(self.offset).copied()
    }

    fn read_tlv(&mut self, expected_tag: u8) -> Option<DerTlv<'a>> {
        let tlv = self.read_any()?;
        if tlv.tag == expected_tag {
            Some(tlv)
        } else {
            None
        }
    }

    fn read_any(&mut self) -> Option<DerTlv<'a>> {
        let start = self.offset;
        let tag = *self.input.get(self.offset)?;
        self.offset += 1;
        let len = self.read_len()?;
        let value_start = self.offset;
        let value_end = value_start.checked_add(len)?;
        if value_end > self.input.len() {
            return None;
        }
        self.offset = value_end;
        Some(DerTlv {
            tag,
            value: &self.input[value_start..value_end],
            full: &self.input[start..value_end],
        })
    }

    fn read_len(&mut self) -> Option<usize> {
        let first = *self.input.get(self.offset)?;
        self.offset += 1;
        if first & 0x80 == 0 {
            return Some(first as usize);
        }

        let octets = (first & 0x7f) as usize;
        if octets == 0 || octets > 4 {
            return None;
        }
        let mut len = 0usize;
        let mut idx = 0usize;
        while idx < octets {
            len = len.checked_mul(256)?;
            len = len.checked_add(*self.input.get(self.offset)? as usize)?;
            self.offset += 1;
            idx += 1;
        }
        Some(len)
    }
}

struct DerTlv<'a> {
    tag: u8,
    value: &'a [u8],
    #[allow(dead_code)]
    full: &'a [u8],
}
