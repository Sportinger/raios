use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::{env, fmt::Write as _, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=RAIOS_DEFAULT_OPENAI_API_KEY");
    println!("cargo:rerun-if-env-changed=RAIOS_OPENAI_CERT_SHA256");
    println!("cargo:rerun-if-env-changed=RAIOS_OPENAI_SPKI_SHA256");
    println!("cargo:rerun-if-env-changed=RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS");
    println!("cargo:rerun-if-changed=descriptors/svc.demo.hello.current_image.desc");
    println!("cargo:rerun-if-changed=descriptors/svc.demo.hello.current_image.p256.pub.hex");
    println!("cargo:rerun-if-changed=descriptors/svc.demo.hello.current_image.p256.sig.der.hex");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let descriptor_path = manifest_dir.join("descriptors/svc.demo.hello.current_image.desc");
    let public_key_path =
        manifest_dir.join("descriptors/svc.demo.hello.current_image.p256.pub.hex");
    let signature_path =
        manifest_dir.join("descriptors/svc.demo.hello.current_image.p256.sig.der.hex");
    let current_source = fs::read_to_string(descriptor_path).unwrap();
    let public_key = read_hex_file(public_key_path);
    let signature_der = read_hex_file(signature_path);
    verify_descriptor_source_signature(&public_key, &signature_der, current_source.as_bytes());

    let current_hash = Sha256::digest(current_source.as_bytes());
    let current_hash_hex = sha256_hex(&current_hash);
    let public_key_hash = Sha256::digest(&public_key);
    let public_key_hash_hex = sha256_hex(&public_key_hash);
    let signature_hash = Sha256::digest(&signature_der);
    let signature_hash_hex = sha256_hex(&signature_hash);
    let envelope_text = format!(
        "schema=raios.descriptor_source_signature_envelope.v0\n\
id=descriptor_source_signature.current_image.svc.demo.hello.v0\n\
algorithm=ecdsa_p256_sha256_asn1_der\n\
payload_source_locator=current_image.descriptor_source.svc.demo.hello.v0\n\
payload_source_kind=current_image_descriptor_source\n\
payload_sha256=sha256:{}\n\
public_key_sha256=sha256:{}\n\
signature_sha256=sha256:{}\n\
verification_phase=runtime_before_descriptor_selection\n\
trust_scope=current_boot_repo_descriptor_source_candidate\n\
classification=local_only\n\
authorizes_external_artifact_load=false\n\
authorizes_persistent_install=false",
        current_hash_hex, public_key_hash_hex, signature_hash_hex
    );
    let envelope_hash = Sha256::digest(envelope_text.as_bytes());
    let host_source = format!(
        "canonicalization=raios.current_boot_load_descriptor.canonical.v0\n\
schema=raios.current_boot_load_descriptor.v0\n\
id=load_descriptor.current_boot.svc.demo.hello.v0\n\
source_kind=host_bound_descriptor_source\n\
source_locator=host_build.descriptor_source.svc.demo.hello.v0\n\
binds_source_locator=current_image.descriptor_source.svc.demo.hello.v0\n\
binds_source_kind=current_image_descriptor_source\n\
binds_source_hash=sha256:{}\n\
service_id=svc.demo.hello\n\
artifact_id=builtin:svc.demo.hello\n\
artifact_kind=builtin_stage0_test_service\n\
scope=current_boot\n\
classification=local_only\n\
persistence=none\n\
accepts_external_artifact_bytes=false\n\
loads_external_artifact=false\n\
writes_persistent_state=false",
        current_hash_hex
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(
        out_dir.join("hello_host_bound_descriptor_source.rs"),
        format!(
            "pub(crate) const HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_LOCATOR: &str = \"host_build.descriptor_source.svc.demo.hello.v0\";\n\
pub(crate) const HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_KIND: &str = \"host_bound_descriptor_source\";\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_SCHEMA: &str = \"raios.descriptor_source_signature_envelope.v0\";\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_ID: &str = \"descriptor_source_signature.current_image.svc.demo.hello.v0\";\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_SIGNATURE_ALGORITHM: &str = \"ecdsa_p256_sha256_asn1_der\";\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_VERIFICATION_PHASE: &str = \"runtime_before_descriptor_selection\";\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_TRUST_SCOPE: &str = \"current_boot_repo_descriptor_source_candidate\";\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_TEXT: &str = {};\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_PUBLIC_KEY_SEC1: &[u8] = &{};\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_SIGNATURE_DER: &[u8] = &{};\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_PUBLIC_KEY_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_SIGNATURE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_HOST_BOUND_CURRENT_IMAGE_SOURCE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_HOST_BOUND_DESCRIPTOR_SOURCE: &str = {};\n",
            rust_string(&envelope_text),
            rust_byte_array(&envelope_hash),
            rust_byte_array(&public_key),
            rust_byte_array(&signature_der),
            rust_byte_array(&public_key_hash),
            rust_byte_array(&signature_hash),
            rust_byte_array(&current_hash),
            rust_string(&host_source)
        ),
    )
    .unwrap();
}

fn read_hex_file(path: PathBuf) -> Vec<u8> {
    parse_hex(fs::read_to_string(path).unwrap().trim()).unwrap()
}

fn verify_descriptor_source_signature(public_key: &[u8], signature_der: &[u8], payload: &[u8]) {
    let verifying_key = VerifyingKey::from_sec1_bytes(public_key).unwrap();
    let signature = Signature::from_der(signature_der).unwrap();
    verifying_key.verify(payload, &signature).unwrap();
}

fn parse_hex(value: &str) -> Result<Vec<u8>, String> {
    let bytes = value.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err("hex value has odd length".to_string());
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut idx = 0usize;
    while idx < bytes.len() {
        let high = hex_nibble(bytes[idx])?;
        let low = hex_nibble(bytes[idx + 1])?;
        out.push((high << 4) | low);
        idx += 2;
    }
    Ok(out)
}

fn hex_nibble(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("hex value contains non-hex byte".to_string()),
    }
}

fn sha256_hex(hash: &[u8]) -> String {
    let mut out = String::new();
    for byte in hash {
        write!(&mut out, "{:02x}", byte).unwrap();
    }
    out
}

fn rust_byte_array(bytes: &[u8]) -> String {
    let mut out = String::from("[");
    for (idx, byte) in bytes.iter().enumerate() {
        if idx != 0 {
            out.push_str(", ");
        }
        write!(&mut out, "0x{:02x}", byte).unwrap();
    }
    out.push(']');
    out
}

fn rust_string(value: &str) -> String {
    format!("{:?}", value)
}
