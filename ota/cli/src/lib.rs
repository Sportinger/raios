use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{anyhow, Result};
use blake3::Hasher;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer, Signature, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};
use hex::{decode, encode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const ALGORITHM: &str = "ed25519";

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
        let secret_hex = self
            .secret_key
            .as_ref()
            .ok_or(KeyError::MissingSecret)?;
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
    pub fn sign(path: &Path, signer: &KeyMaterial, cert: &SignerCertificate, metadata: Option<Value>) -> Result<Self> {
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
        signer_pk.verify_strict(&signing_envelope_bytes(&self.payload, &self.signer_key_id, self.metadata.as_ref())?, &signature)?;
        Ok(())
    }
}

fn signing_envelope_bytes(payload: &BlobPayload, signer_key_id: &str, metadata: Option<&Value>) -> Result<Vec<u8>> {
    #[derive(Serialize)]
    struct Envelope<'a> {
        payload: &'a BlobPayload,
        signer_key_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<&'a Value>,
    }
    Ok(serde_json::to_vec(&Envelope { payload, signer_key_id, metadata })?)
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
