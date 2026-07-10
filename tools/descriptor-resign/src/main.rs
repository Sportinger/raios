//! Explicit local re-signing for checked-in raiOS development descriptors.
//!
//! This tool has no runtime authority. It signs the exact raw descriptor bytes
//! with one fresh development key and writes only that public key and signature.

use std::{env, fs, path::Path, process};

use p256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey, VerifyingKey,
};
use rand::rngs::OsRng;

fn usage() -> ! {
    eprintln!("usage: descriptor-resign <sign|verify> <descriptor.desc> <public.p256.pub.hex> <signature.p256.sig.der.hex>");
    process::exit(2);
}

fn sign_payload(payload: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::random(&mut OsRng);
    let signature: Signature = signing_key.sign(payload);
    let public_key = VerifyingKey::from(&signing_key);
    (
        public_key.to_encoded_point(false).as_bytes().to_vec(),
        signature.to_der().as_bytes().to_vec(),
    )
}

fn verify_payload(public_key: &[u8], signature_der: &[u8], payload: &[u8]) -> Result<(), String> {
    let key = VerifyingKey::from_sec1_bytes(public_key)
        .map_err(|error| format!("invalid SEC1 public key: {error}"))?;
    let signature = Signature::from_der(signature_der)
        .map_err(|error| format!("invalid ASN.1 DER signature: {error}"))?;
    key.verify(payload, &signature)
        .map_err(|error| format!("signature does not verify descriptor bytes: {error}"))
}

fn read_hex(path: &Path) -> Result<Vec<u8>, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    hex::decode(text.trim())
        .map_err(|error| format!("invalid hexadecimal in {}: {error}", path.display()))
}

fn write_hex(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::write(path, format!("{}\n", hex::encode(bytes)))
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn sign(descriptor: &Path, public_key: &Path, signature: &Path) -> Result<(), String> {
    if descriptor == public_key || descriptor == signature || public_key == signature {
        return Err("descriptor, public-key, and signature paths must be distinct".to_owned());
    }
    let payload = fs::read(descriptor)
        .map_err(|error| format!("failed to read {}: {error}", descriptor.display()))?;
    let (public_key_bytes, signature_der) = sign_payload(&payload);
    verify_payload(&public_key_bytes, &signature_der, &payload)?;
    write_hex(public_key, &public_key_bytes)?;
    write_hex(signature, &signature_der)?;
    println!("descriptor-resign: signed {}", descriptor.display());
    Ok(())
}

fn verify(descriptor: &Path, public_key: &Path, signature: &Path) -> Result<(), String> {
    let payload = fs::read(descriptor)
        .map_err(|error| format!("failed to read {}: {error}", descriptor.display()))?;
    let public_key_bytes = read_hex(public_key)?;
    let signature_der = read_hex(signature)?;
    verify_payload(&public_key_bytes, &signature_der, &payload)?;
    println!("descriptor-resign: verified {}", descriptor.display());
    Ok(())
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        usage();
    };
    let (Some(descriptor), Some(public_key), Some(signature)) =
        (args.next(), args.next(), args.next())
    else {
        usage();
    };
    if args.next().is_some() {
        usage();
    }

    let result = match command.as_str() {
        "sign" => sign(
            Path::new(&descriptor),
            Path::new(&public_key),
            Path::new(&signature),
        ),
        "verify" => verify(
            Path::new(&descriptor),
            Path::new(&public_key),
            Path::new(&signature),
        ),
        _ => usage(),
    };
    if let Err(error) = result {
        eprintln!("descriptor-resign: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{sign, verify};

    #[test]
    fn sign_and_verify_rejects_altered_descriptor_bytes() {
        let test_dir = std::env::temp_dir().join(format!(
            "descriptor-resign-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&test_dir).unwrap();
        let descriptor = test_dir.join("descriptor.desc");
        let public_key = test_dir.join("descriptor.p256.pub.hex");
        let signature = test_dir.join("descriptor.p256.sig.der.hex");
        fs::write(
            &descriptor,
            b"schema=raios.test_descriptor.v0\nfield=original\n",
        )
        .unwrap();

        sign(&descriptor, &public_key, &signature).unwrap();
        assert!(verify(&descriptor, &public_key, &signature).is_ok());
        fs::write(
            &descriptor,
            b"schema=raios.test_descriptor.v0\nfield=altered\n",
        )
        .unwrap();
        assert!(verify(&descriptor, &public_key, &signature).is_err());

        fs::remove_dir_all(test_dir).unwrap();
    }
}
