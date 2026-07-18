use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{anyhow, Result};
use blake3::Hasher;
use ed25519_dalek::{
    Keypair, PublicKey, SecretKey, Signature, Signer, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH,
    SIGNATURE_LENGTH,
};
use hex::{decode, encode};
use p256::ecdsa::{
    signature::{Signer as P256Signer, Verifier as P256Verifier},
    Signature as P256Signature, SigningKey as P256SigningKey, VerifyingKey as P256VerifyingKey,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const ALGORITHM: &str = "ed25519";
/// The known DEV distribution publisher scalar = 2. Public because the DEV key is public.
pub const DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
];
/// The known DEV Wasm descriptor scalar = 1. Public because the DEV key is public.
pub const DEV_WASM_DESCRIPTOR_PRIVATE_SCALAR: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];
/// MUST byte-match `raios_core::distribution_provenance::DISTRIBUTION_PROVENANCE_DOMAIN_TAG`.
pub const DISTRIBUTION_PROVENANCE_DOMAIN_TAG: &[u8] = b"raios.distribution_provenance.v0";

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("invalid key length: {0}")]
    InvalidKeyLength(String),
    #[error("missing secret key material for signing")]
    MissingSecret,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub key_id: String,
    pub algorithm: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
}

impl KeyMaterial {
    pub fn new(key_id: impl Into<String>, keypair: &Keypair, include_secret: bool) -> Self {
        Self {
            key_id: key_id.into(),
            algorithm: ALGORITHM.to_string(),
            public_key: encode(keypair.public.as_bytes()),
            secret_key: include_secret.then(|| encode(keypair.secret.as_bytes())),
        }
    }

    pub fn from_seed(label: &str, context: &str) -> Result<Self> {
        let mut hasher = Hasher::new();
        hasher.update(context.as_bytes());
        hasher.update(label.as_bytes());
        let output = hasher.finalize();
        let mut secret_bytes = [0u8; SECRET_KEY_LENGTH];
        secret_bytes.copy_from_slice(&output.as_bytes()[..SECRET_KEY_LENGTH]);
        let secret = SecretKey::from_bytes(&secret_bytes)?;
        let public = PublicKey::from(&secret);
        let keypair = Keypair { secret, public };
        Ok(Self::new(label, &keypair, true))
    }

    pub fn to_keypair(&self) -> Result<Keypair> {
        let secret_hex = self.secret_key.as_ref().ok_or(KeyError::MissingSecret)?;
        let secret_bytes = decode(secret_hex)?;
        if secret_bytes.len() != SECRET_KEY_LENGTH {
            return Err(KeyError::InvalidKeyLength(format!(
                "secret bytes {} != {}",
                secret_bytes.len(),
                SECRET_KEY_LENGTH
            ))
            .into());
        }
        let mut secret_array = [0u8; SECRET_KEY_LENGTH];
        secret_array.copy_from_slice(&secret_bytes);
        let secret = SecretKey::from_bytes(&secret_array)?;
        let public = PublicKey::from(&secret);
        Ok(Keypair { secret, public })
    }

    pub fn public_key(&self) -> Result<PublicKey> {
        let pk_bytes = decode(&self.public_key)?;
        if pk_bytes.len() != PUBLIC_KEY_LENGTH {
            return Err(KeyError::InvalidKeyLength(format!(
                "public bytes {} != {}",
                pk_bytes.len(),
                PUBLIC_KEY_LENGTH
            ))
            .into());
        }
        let mut pk_array = [0u8; PUBLIC_KEY_LENGTH];
        pk_array.copy_from_slice(&pk_bytes);
        Ok(PublicKey::from_bytes(&pk_array)?)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignerCertificate {
    pub version: u8,
    pub subject: String,
    pub issuer: String,
    pub algorithm: String,
    pub subject_public_key: String,
    pub issued_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub signature: String,
}

impl SignerCertificate {
    pub fn new(
        subject: &KeyMaterial,
        issuer: &KeyMaterial,
        issued_at_unix_ms: u64,
        ttl_ms: u64,
    ) -> Result<Self> {
        let subject_public = subject.public_key()?;
        let issuer_keypair = issuer.to_keypair()?;
        let mut cert = SignerCertificate {
            version: 1,
            subject: subject.key_id.clone(),
            issuer: issuer.key_id.clone(),
            algorithm: ALGORITHM.to_string(),
            subject_public_key: encode(subject_public.as_bytes()),
            issued_at_unix_ms,
            expires_at_unix_ms: issued_at_unix_ms + ttl_ms,
            signature: String::new(),
        };
        cert.signature = encode(sign_certificate_payload(&cert, &issuer_keypair)?);
        Ok(cert)
    }

    pub fn verify(&self, issuer_public: &PublicKey) -> Result<()> {
        let signature_bytes = decode(&self.signature)?;
        if signature_bytes.len() != SIGNATURE_LENGTH {
            return Err(KeyError::InvalidKeyLength(format!(
                "signature bytes {} != {}",
                signature_bytes.len(),
                SIGNATURE_LENGTH
            ))
            .into());
        }
        let mut sig_array = [0u8; SIGNATURE_LENGTH];
        sig_array.copy_from_slice(&signature_bytes);
        let signature = Signature::from_bytes(&sig_array)?;
        issuer_public.verify_strict(&self.payload_bytes()?, &signature)?;
        Ok(())
    }

    fn payload_bytes(&self) -> Result<Vec<u8>> {
        let payload = CertPayloadSer {
            version: self.version,
            subject: &self.subject,
            issuer: &self.issuer,
            algorithm: &self.algorithm,
            subject_public_key: &self.subject_public_key,
            issued_at_unix_ms: self.issued_at_unix_ms,
            expires_at_unix_ms: self.expires_at_unix_ms,
        };
        Ok(serde_json::to_vec(&payload)?)
    }
}

#[derive(Serialize)]
struct CertPayloadSer<'a> {
    version: u8,
    subject: &'a str,
    issuer: &'a str,
    algorithm: &'a str,
    subject_public_key: &'a str,
    issued_at_unix_ms: u64,
    expires_at_unix_ms: u64,
}

fn sign_certificate_payload(cert: &SignerCertificate, issuer: &Keypair) -> Result<Vec<u8>> {
    let payload = cert.payload_bytes()?;
    Ok(issuer.sign(&payload).to_bytes().to_vec())
}

pub fn load_keymaterial<P: AsRef<Path>>(path: P) -> Result<KeyMaterial> {
    let data = std::fs::read_to_string(path)?;
    let key: KeyMaterial = serde_json::from_str(&data)?;
    if key.algorithm != ALGORITHM {
        return Err(anyhow!("unexpected algorithm: {}", key.algorithm));
    }
    Ok(key)
}

pub fn store_certificate<P: AsRef<Path>>(cert: &SignerCertificate, path: P) -> Result<()> {
    let json = serde_json::to_string_pretty(cert)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_certificate<P: AsRef<Path>>(path: P) -> Result<SignerCertificate> {
    let data = std::fs::read_to_string(path)?;
    let cert: SignerCertificate = serde_json::from_str(&data)?;
    if cert.algorithm != ALGORITHM {
        return Err(anyhow!("unexpected algorithm: {}", cert.algorithm));
    }
    Ok(cert)
}

pub fn public_key_from_hex(hex_str: &str) -> Result<PublicKey> {
    let cleaned = hex_str.trim();
    let bytes = decode(cleaned)?;
    if bytes.len() != PUBLIC_KEY_LENGTH {
        return Err(KeyError::InvalidKeyLength(format!(
            "public bytes {} != {}",
            bytes.len(),
            PUBLIC_KEY_LENGTH
        ))
        .into());
    }
    let mut array = [0u8; PUBLIC_KEY_LENGTH];
    array.copy_from_slice(&bytes);
    Ok(PublicKey::from_bytes(&array)?)
}

pub fn load_public_key_hex<P: AsRef<Path>>(path: P) -> Result<PublicKey> {
    let data = std::fs::read_to_string(path)?;
    public_key_from_hex(&data)
}

pub fn public_key_to_hex(public: &PublicKey) -> String {
    encode(public.as_bytes())
}

pub fn decode_hex_bytes(hex_str: &str) -> Result<Vec<u8>> {
    Ok(decode(hex_str.trim())?)
}

pub fn verify_p256_der_signature(
    public_key_sec1: &[u8],
    signature_der: &[u8],
    payload: &[u8],
) -> Result<()> {
    let verifying_key = P256VerifyingKey::from_sec1_bytes(public_key_sec1)?;
    let signature = P256Signature::from_der(signature_der)?;
    verifying_key.verify(payload, &signature)?;
    Ok(())
}

pub fn sign_wasm_descriptor_dev_hex(payload: &[u8]) -> Result<(String, String)> {
    let signing_key = P256SigningKey::from_slice(&DEV_WASM_DESCRIPTOR_PRIVATE_SCALAR)?;
    let signature: P256Signature = signing_key.sign(payload);
    let public_key = P256VerifyingKey::from(&signing_key);
    Ok((
        encode(signature.to_der().as_bytes()),
        encode(public_key.to_encoded_point(false).as_bytes()),
    ))
}

pub fn sign_distribution_provenance_hex(artifact_sha256: &[u8; 32]) -> String {
    let signing_key = P256SigningKey::from_slice(&DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR)
        .expect("dev scalar 2 is a valid key");
    let mut message = [0u8; DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len() + 32];
    message[..DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len()]
        .copy_from_slice(DISTRIBUTION_PROVENANCE_DOMAIN_TAG);
    message[DISTRIBUTION_PROVENANCE_DOMAIN_TAG.len()..].copy_from_slice(artifact_sha256);
    let signature: P256Signature = signing_key.sign(&message);
    encode(signature.to_der().as_bytes())
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlobPayload {
    pub version: u8,
    pub hash_blake3: String,
    pub length_bytes: u64,
}

impl BlobPayload {
    pub fn from_reader(reader: &mut File) -> Result<Self> {
        reader.seek(SeekFrom::Start(0))?;
        let mut hasher = blake3::Hasher::new();
        let length = copy_and_hash(reader, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(Self {
            version: 1,
            hash_blake3: hash.to_hex().to_string(),
            length_bytes: length,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedBlob {
    pub payload: BlobPayload,
    pub signer_key_id: String,
    pub signature: String,
    pub certificate: SignerCertificate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl SignedBlob {
    pub fn sign(
        path: &Path,
        signer: &KeyMaterial,
        cert: &SignerCertificate,
        metadata: Option<Value>,
    ) -> Result<Self> {
        let mut file = File::open(path)?;
        let payload = BlobPayload::from_reader(&mut file)?;
        let signer_key = signer.to_keypair()?;
        let signing_bytes = signing_envelope_bytes(&payload, &signer.key_id, metadata.as_ref())?;
        let sig_bytes = signer_key.sign(&signing_bytes).to_bytes();
        Ok(Self {
            payload,
            signer_key_id: signer.key_id.clone(),
            signature: encode(sig_bytes),
            certificate: cert.clone(),
            metadata,
        })
    }

    pub fn payload_hash(&self) -> &str {
        &self.payload.hash_blake3
    }

    pub fn payload_len(&self) -> u64 {
        self.payload.length_bytes
    }

    pub fn verify(&self, path: &Path, root_key: &PublicKey) -> Result<()> {
        self.certificate.verify(root_key)?;
        let mut file = File::open(path)?;
        let expected_payload = BlobPayload::from_reader(&mut file)?;
        if expected_payload.hash_blake3 != self.payload.hash_blake3
            || expected_payload.length_bytes != self.payload.length_bytes
        {
            return Err(anyhow!("payload mismatch"));
        }
        let signature_bytes = decode(&self.signature)?;
        if signature_bytes.len() != SIGNATURE_LENGTH {
            return Err(KeyError::InvalidKeyLength(format!(
                "signature bytes {} != {}",
                signature_bytes.len(),
                SIGNATURE_LENGTH
            ))
            .into());
        }
        let mut sig_array = [0u8; SIGNATURE_LENGTH];
        sig_array.copy_from_slice(&signature_bytes);
        let signature = Signature::from_bytes(&sig_array)?;
        let signer_public = self.certificate.subject_public_key.clone();
        let signer_public_bytes = decode(&signer_public)?;
        if signer_public_bytes.len() != PUBLIC_KEY_LENGTH {
            return Err(KeyError::InvalidKeyLength(format!(
                "public bytes {} != {}",
                signer_public_bytes.len(),
                PUBLIC_KEY_LENGTH
            ))
            .into());
        }
        let mut signer_array = [0u8; PUBLIC_KEY_LENGTH];
        signer_array.copy_from_slice(&signer_public_bytes);
        let signer_pk = PublicKey::from_bytes(&signer_array)?;
        signer_pk.verify_strict(
            &signing_envelope_bytes(&self.payload, &self.signer_key_id, self.metadata.as_ref())?,
            &signature,
        )?;
        Ok(())
    }
}

fn signing_envelope_bytes(
    payload: &BlobPayload,
    signer_key_id: &str,
    metadata: Option<&Value>,
) -> Result<Vec<u8>> {
    #[derive(Serialize)]
    struct Envelope<'a> {
        payload: &'a BlobPayload,
        signer_key_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<&'a Value>,
    }
    Ok(serde_json::to_vec(&Envelope {
        payload,
        signer_key_id,
        metadata,
    })?)
}

fn copy_and_hash(reader: &mut File, hasher: &mut blake3::Hasher) -> Result<u64> {
    reader.seek(SeekFrom::Start(0))?;
    let mut buf = [0u8; 8192];
    let mut total = 0u64;
    loop {
        let read = reader.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
        total += read as u64;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_hex_bytes, sign_distribution_provenance_hex, sign_wasm_descriptor_dev_hex,
        verify_p256_der_signature, DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR,
        DEV_WASM_DESCRIPTOR_PRIVATE_SCALAR, DISTRIBUTION_PROVENANCE_DOMAIN_TAG,
    };
    use raios_core::distribution_provenance::verify_distribution_provenance_signature;
    use raios_core::promotion_attestation::verify_promotion_authority_signature;

    #[test]
    fn distribution_signer_domain_tag_matches_core() {
        assert_eq!(
            DISTRIBUTION_PROVENANCE_DOMAIN_TAG,
            raios_core::distribution_provenance::DISTRIBUTION_PROVENANCE_DOMAIN_TAG
        );
    }

    #[test]
    fn distribution_signer_output_verifies_as_provenance_not_promotion() {
        let hash: [u8; 32] = [7u8; 32];
        let der = hex::decode(sign_distribution_provenance_hex(&hash)).expect("output is hex");

        assert!(verify_distribution_provenance_signature(&der, &hash));
        assert!(!verify_promotion_authority_signature(&der, &hash));
    }

    #[test]
    fn distribution_signer_output_is_stable() {
        let hash: [u8; 32] = [0x42u8; 32];
        assert_eq!(
            sign_distribution_provenance_hex(&hash),
            sign_distribution_provenance_hex(&hash),
            "RFC6979 signing must be deterministic"
        );
    }

    #[test]
    fn dev_distribution_publisher_scalar_is_two() {
        assert_eq!(DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR[31], 2);
        assert!(DEV_DISTRIBUTION_PUBLISHER_PRIVATE_SCALAR[..31]
            .iter()
            .all(|b| *b == 0));
    }

    #[test]
    fn wasm_descriptor_dev_signature_roundtrip() {
        let payload = b"descriptor bytes";
        let (sig_hex, public_hex) = sign_wasm_descriptor_dev_hex(payload).expect("dev sign");
        let sig = decode_hex_bytes(&sig_hex).expect("signature hex");
        let public = decode_hex_bytes(&public_hex).expect("public key hex");

        verify_p256_der_signature(&public, &sig, payload).expect("signature verifies");
        assert!(verify_p256_der_signature(&public, &sig, b"other descriptor").is_err());
        assert_eq!(DEV_WASM_DESCRIPTOR_PRIVATE_SCALAR[31], 1);
        assert!(DEV_WASM_DESCRIPTOR_PRIVATE_SCALAR[..31]
            .iter()
            .all(|b| *b == 0));
    }
}
