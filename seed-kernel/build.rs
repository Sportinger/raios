use sha2::{Digest, Sha256};
use std::{env, fmt::Write as _, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=RAIOS_DEFAULT_OPENAI_API_KEY");
    println!("cargo:rerun-if-env-changed=RAIOS_OPENAI_CERT_SHA256");
    println!("cargo:rerun-if-env-changed=RAIOS_OPENAI_SPKI_SHA256");
    println!("cargo:rerun-if-env-changed=RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS");
    println!("cargo:rerun-if-changed=descriptors/svc.demo.hello.current_image.desc");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let descriptor_path = manifest_dir.join("descriptors/svc.demo.hello.current_image.desc");
    let current_source = fs::read_to_string(descriptor_path).unwrap();
    let current_hash = Sha256::digest(current_source.as_bytes());
    let current_hash_hex = sha256_hex(&current_hash);
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
pub(crate) const HELLO_HOST_BOUND_CURRENT_IMAGE_SOURCE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_HOST_BOUND_DESCRIPTOR_SOURCE: &str = {};\n",
            rust_byte_array(&current_hash),
            rust_string(&host_source)
        ),
    )
    .unwrap();
}

fn sha256_hex(hash: &[u8]) -> String {
    let mut out = String::new();
    for byte in hash {
        write!(&mut out, "{:02x}", byte).unwrap();
    }
    out
}

fn rust_byte_array(hash: &[u8]) -> String {
    let mut out = String::from("[");
    for (idx, byte) in hash.iter().enumerate() {
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
