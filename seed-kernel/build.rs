use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::{env, fmt::Write as _, fs, path::PathBuf};

// Ordered repo-relative artifact source-set manifest; root first, then modules
// in declaration order from seed-kernel/src/hello_service.rs.
const HELLO_ARTIFACT_SOURCE_SET: &[&str] = &[
    "seed-kernel/src/hello_service.rs",
    "seed-kernel/src/hello_service/command_targets.rs",
    "seed-kernel/src/hello_service/constants.rs",
    "seed-kernel/src/hello_service/descriptor_identity.rs",
    "seed-kernel/src/hello_service/emitters.rs",
    "seed-kernel/src/hello_service/hash_support.rs",
    "seed-kernel/src/hello_service/lifecycle_binding.rs",
    "seed-kernel/src/hello_service/preflight.rs",
    "seed-kernel/src/hello_service/records.rs",
    "seed-kernel/src/hello_service/rollback_authority_gates.rs",
    "seed-kernel/src/hello_service/rollback_bindings.rs",
    "seed-kernel/src/hello_service/rollback_hashes_a.rs",
    "seed-kernel/src/hello_service/rollback_hashes_b.rs",
    "seed-kernel/src/hello_service/rollback_writer_bindings.rs",
    "seed-kernel/src/hello_service/rollback_writer_gate.rs",
    "seed-kernel/src/hello_service/runtime.rs",
    "seed-kernel/src/hello_service/state_machine.rs",
    "seed-kernel/src/hello_service/state_records.rs",
    "seed-kernel/src/hello_service/storage_authority_gate.rs",
    "seed-kernel/src/hello_service/storage_gate_hash.rs",
];

fn main() {
    println!("cargo:rerun-if-env-changed=RAIOS_DEFAULT_OPENAI_API_KEY");
    println!("cargo:rerun-if-env-changed=RAIOS_OPENAI_CERT_SHA256");
    println!("cargo:rerun-if-env-changed=RAIOS_OPENAI_SPKI_SHA256");
    println!("cargo:rerun-if-env-changed=RAIOS_OPENAI_SPKI_SHA256_NEXT");
    println!("cargo:rerun-if-env-changed=RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS");
    println!("cargo:rerun-if-changed=descriptors/svc.demo.hello.current_image.desc");
    println!("cargo:rerun-if-changed=descriptors/svc.demo.hello.current_image.p256.pub.hex");
    println!("cargo:rerun-if-changed=descriptors/svc.demo.hello.current_image.p256.sig.der.hex");
    println!("cargo:rerun-if-changed=descriptors/svc.demo.hello.builtin_artifact_identity.desc");
    println!(
        "cargo:rerun-if-changed=descriptors/svc.demo.hello.builtin_artifact_identity.p256.pub.hex"
    );
    println!(
        "cargo:rerun-if-changed=descriptors/svc.demo.hello.builtin_artifact_identity.p256.sig.der.hex"
    );
    println!("cargo:rerun-if-changed=descriptors/svc.demo.hello.builtin_artifact_identity.v2.desc");
    println!(
        "cargo:rerun-if-changed=descriptors/svc.demo.hello.builtin_artifact_identity.v2.p256.pub.hex"
    );
    println!(
        "cargo:rerun-if-changed=descriptors/svc.demo.hello.builtin_artifact_identity.v2.p256.sig.der.hex"
    );
    println!("cargo:rerun-if-changed=artifacts/svc.demo.hello.builtin.artifact");
    println!("cargo:rerun-if-changed=artifacts/svc.demo.echo.wasm");
    println!("cargo:rerun-if-changed=descriptors/svc.demo.echo.wasm_artifact_identity.desc");
    println!(
        "cargo:rerun-if-changed=descriptors/svc.demo.echo.wasm_artifact_identity.p256.pub.hex"
    );
    println!(
        "cargo:rerun-if-changed=descriptors/svc.demo.echo.wasm_artifact_identity.p256.sig.der.hex"
    );

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.parent().unwrap();
    for source_path in HELLO_ARTIFACT_SOURCE_SET {
        println!(
            "cargo:rerun-if-changed={}",
            repo_root.join(source_path).display()
        );
    }
    let descriptor_path = manifest_dir.join("descriptors/svc.demo.hello.current_image.desc");
    let artifact_bytes_path = manifest_dir.join("artifacts/svc.demo.hello.builtin.artifact");
    let public_key_path =
        manifest_dir.join("descriptors/svc.demo.hello.current_image.p256.pub.hex");
    let signature_path =
        manifest_dir.join("descriptors/svc.demo.hello.current_image.p256.sig.der.hex");
    let artifact_identity_path =
        manifest_dir.join("descriptors/svc.demo.hello.builtin_artifact_identity.desc");
    let artifact_identity_public_key_path =
        manifest_dir.join("descriptors/svc.demo.hello.builtin_artifact_identity.p256.pub.hex");
    let artifact_identity_signature_path =
        manifest_dir.join("descriptors/svc.demo.hello.builtin_artifact_identity.p256.sig.der.hex");
    let artifact_identity_v2_path =
        manifest_dir.join("descriptors/svc.demo.hello.builtin_artifact_identity.v2.desc");
    let artifact_identity_v2_public_key_path =
        manifest_dir.join("descriptors/svc.demo.hello.builtin_artifact_identity.v2.p256.pub.hex");
    let artifact_identity_v2_signature_path = manifest_dir
        .join("descriptors/svc.demo.hello.builtin_artifact_identity.v2.p256.sig.der.hex");
    let current_source = fs::read_to_string(descriptor_path).unwrap();
    let artifact_source_snapshot = read_ordered_source_set(repo_root, HELLO_ARTIFACT_SOURCE_SET);
    let artifact_bytes = fs::read(artifact_bytes_path).unwrap();
    let artifact_identity_source = fs::read_to_string(artifact_identity_path).unwrap();
    let artifact_identity_v2_source = fs::read_to_string(artifact_identity_v2_path).unwrap();
    let public_key = read_hex_file(public_key_path);
    let signature_der = read_hex_file(signature_path);
    let artifact_identity_public_key = read_hex_file(artifact_identity_public_key_path);
    let artifact_identity_signature_der = read_hex_file(artifact_identity_signature_path);
    let artifact_identity_v2_public_key = read_hex_file(artifact_identity_v2_public_key_path);
    let artifact_identity_v2_signature_der = read_hex_file(artifact_identity_v2_signature_path);
    verify_p256_signature(&public_key, &signature_der, current_source.as_bytes());
    verify_p256_signature(
        &artifact_identity_public_key,
        &artifact_identity_signature_der,
        artifact_identity_source.as_bytes(),
    );
    verify_p256_signature(
        &artifact_identity_v2_public_key,
        &artifact_identity_v2_signature_der,
        artifact_identity_v2_source.as_bytes(),
    );

    let current_hash = Sha256::digest(current_source.as_bytes());
    let current_hash_hex = sha256_hex(&current_hash);
    let public_key_hash = Sha256::digest(&public_key);
    let public_key_hash_hex = sha256_hex(&public_key_hash);
    let signature_hash = Sha256::digest(&signature_der);
    let signature_hash_hex = sha256_hex(&signature_hash);
    let artifact_identity_hash = Sha256::digest(artifact_identity_source.as_bytes());
    let artifact_identity_hash_hex = sha256_hex(&artifact_identity_hash);
    let artifact_identity_v2_hash = Sha256::digest(artifact_identity_v2_source.as_bytes());
    let artifact_identity_v2_hash_hex = sha256_hex(&artifact_identity_v2_hash);
    let artifact_content_source_hash = Sha256::digest(&artifact_source_snapshot);
    let artifact_content_source_hash_hex = sha256_hex(&artifact_content_source_hash);
    let artifact_content_binding_text = format!(
        "schema=raios.builtin_artifact_content_binding.v0\n\
id=builtin_artifact_content.svc.demo.hello.v0\n\
artifact_id=builtin:svc.demo.hello\n\
content_kind=repo_source_snapshot\n\
source_locator=seed-kernel/src/hello_service.rs\n\
source_sha256=sha256:{}\n\
accepts_external_artifact_bytes=false\n\
loads_external_artifact=false\n\
maps_executable_pages=false\n\
writes_persistent_state=false",
        artifact_content_source_hash_hex
    );
    let artifact_content_binding_hash = Sha256::digest(artifact_content_binding_text.as_bytes());
    let artifact_content_binding_hash_hex = sha256_hex(&artifact_content_binding_hash);
    let artifact_bytes_hash = Sha256::digest(&artifact_bytes);
    let artifact_bytes_hash_hex = sha256_hex(&artifact_bytes_hash);
    let artifact_reference_text = format!(
        "schema=raios.builtin_artifact_reference.v0\n\
id=builtin_artifact_reference.svc.demo.hello.v0\n\
artifact_id=builtin:svc.demo.hello\n\
service_id=svc.demo.hello\n\
reference_kind=repo_artifact_bytes_snapshot\n\
artifact_locator=seed-kernel/artifacts/svc.demo.hello.builtin.artifact\n\
artifact_bytes_sha256=sha256:{}\n\
content_binding_sha256=sha256:{}\n\
accepts_external_artifact_bytes=false\n\
loads_artifact_as_code=false\n\
maps_executable_pages=false\n\
writes_persistent_state=false",
        artifact_bytes_hash_hex, artifact_content_binding_hash_hex
    );
    let artifact_reference_hash = Sha256::digest(artifact_reference_text.as_bytes());
    let artifact_reference_hash_hex = sha256_hex(&artifact_reference_hash);
    let expected_artifact_content_source_hash =
        format!("sha256:{}", artifact_content_source_hash_hex);
    let expected_artifact_content_binding_hash =
        format!("sha256:{}", artifact_content_binding_hash_hex);
    let expected_artifact_reference_hash = format!("sha256:{}", artifact_reference_hash_hex);
    let expected_artifact_bytes_hash = format!("sha256:{}", artifact_bytes_hash_hex);
    assert_eq!(
        text_field(&artifact_identity_source, "artifact_content_source_sha256"),
        Some(expected_artifact_content_source_hash.as_str())
    );
    assert_eq!(
        text_field(&artifact_identity_source, "artifact_content_binding_sha256"),
        Some(expected_artifact_content_binding_hash.as_str())
    );
    assert_eq!(
        text_field(&artifact_identity_source, "artifact_reference_sha256"),
        Some(expected_artifact_reference_hash.as_str())
    );
    assert_eq!(
        text_field(&artifact_identity_source, "artifact_reference_bytes_sha256"),
        Some(expected_artifact_bytes_hash.as_str())
    );
    assert_eq!(
        text_field(
            &artifact_identity_source,
            "artifact_reference_content_binding_sha256"
        ),
        Some(expected_artifact_content_binding_hash.as_str())
    );
    assert_eq!(
        text_field(&artifact_identity_v2_source, "id"),
        Some("builtin_artifact_identity.svc.demo.hello.v2")
    );
    assert_eq!(
        text_field(
            &artifact_identity_v2_source,
            "artifact_content_source_sha256"
        ),
        Some(expected_artifact_content_source_hash.as_str())
    );
    assert_eq!(
        text_field(
            &artifact_identity_v2_source,
            "artifact_content_binding_sha256"
        ),
        Some(expected_artifact_content_binding_hash.as_str())
    );
    assert_eq!(
        text_field(&artifact_identity_v2_source, "artifact_reference_sha256"),
        Some(expected_artifact_reference_hash.as_str())
    );
    assert_eq!(
        text_field(
            &artifact_identity_v2_source,
            "artifact_reference_bytes_sha256"
        ),
        Some(expected_artifact_bytes_hash.as_str())
    );
    assert_eq!(
        text_field(
            &artifact_identity_v2_source,
            "artifact_reference_content_binding_sha256"
        ),
        Some(expected_artifact_content_binding_hash.as_str())
    );
    let artifact_identity_public_key_hash = Sha256::digest(&artifact_identity_public_key);
    let artifact_identity_public_key_hash_hex = sha256_hex(&artifact_identity_public_key_hash);
    let artifact_identity_signature_hash = Sha256::digest(&artifact_identity_signature_der);
    let artifact_identity_signature_hash_hex = sha256_hex(&artifact_identity_signature_hash);
    let artifact_identity_v2_public_key_hash = Sha256::digest(&artifact_identity_v2_public_key);
    let artifact_identity_v2_public_key_hash_hex =
        sha256_hex(&artifact_identity_v2_public_key_hash);
    let artifact_identity_v2_signature_hash = Sha256::digest(&artifact_identity_v2_signature_der);
    let artifact_identity_v2_signature_hash_hex = sha256_hex(&artifact_identity_v2_signature_hash);
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
    let artifact_identity_envelope_text = format!(
        "schema=raios.builtin_artifact_identity_signature_envelope.v0\n\
id=artifact_identity_signature.builtin.svc.demo.hello.v0\n\
algorithm=ecdsa_p256_sha256_asn1_der\n\
payload_identity_id=builtin_artifact_identity.svc.demo.hello.v0\n\
payload_artifact_id=builtin:svc.demo.hello\n\
payload_sha256=sha256:{}\n\
public_key_sha256=sha256:{}\n\
signature_sha256=sha256:{}\n\
verification_phase=runtime_before_builtin_artifact_selection\n\
trust_scope=current_boot_builtin_artifact_identity_candidate\n\
classification=local_only\n\
authorizes_external_artifact_load=false\n\
authorizes_persistent_install=false\n\
authorizes_rollback_install=false",
        artifact_identity_hash_hex,
        artifact_identity_public_key_hash_hex,
        artifact_identity_signature_hash_hex
    );
    let artifact_identity_envelope_hash =
        Sha256::digest(artifact_identity_envelope_text.as_bytes());
    let artifact_identity_v2_envelope_text = format!(
        "schema=raios.builtin_artifact_identity_signature_envelope.v0\n\
id=artifact_identity_signature.builtin.svc.demo.hello.v2\n\
algorithm=ecdsa_p256_sha256_asn1_der\n\
payload_identity_id=builtin_artifact_identity.svc.demo.hello.v2\n\
payload_artifact_id=builtin:svc.demo.hello\n\
payload_sha256=sha256:{}\n\
public_key_sha256=sha256:{}\n\
signature_sha256=sha256:{}\n\
verification_phase=runtime_before_builtin_artifact_selection\n\
trust_scope=current_boot_builtin_artifact_identity_candidate\n\
classification=local_only\n\
authorizes_external_artifact_load=false\n\
authorizes_persistent_install=false\n\
authorizes_rollback_install=false",
        artifact_identity_v2_hash_hex,
        artifact_identity_v2_public_key_hash_hex,
        artifact_identity_v2_signature_hash_hex
    );
    let artifact_identity_v2_envelope_hash =
        Sha256::digest(artifact_identity_v2_envelope_text.as_bytes());
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
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_SCHEMA: &str = \"raios.builtin_artifact_identity_signature_envelope.v0\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_ID: &str = \"artifact_identity_signature.builtin.svc.demo.hello.v0\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_SIGNATURE_ALGORITHM: &str = \"ecdsa_p256_sha256_asn1_der\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_VERIFICATION_PHASE: &str = \"runtime_before_builtin_artifact_selection\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_TRUST_SCOPE: &str = \"current_boot_builtin_artifact_identity_candidate\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_TEXT: &str = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_PUBLIC_KEY_SEC1: &[u8] = &{};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_SIGNATURE_DER: &[u8] = &{};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_PUBLIC_KEY_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_SIGNATURE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_SOURCE: &str = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_ENVELOPE_ID: &str = \"artifact_identity_signature.builtin.svc.demo.hello.v2\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_ENVELOPE_TEXT: &str = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_ENVELOPE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_PUBLIC_KEY_SEC1: &[u8] = &{};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_SIGNATURE_DER: &[u8] = &{};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_PUBLIC_KEY_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_SIGNATURE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_SCHEMA: &str = \"raios.builtin_artifact_content_binding.v0\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_ID: &str = \"builtin_artifact_content.svc.demo.hello.v0\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_CONTENT_KIND: &str = \"repo_source_snapshot\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_CONTENT_SOURCE_LOCATOR: &str = \"seed-kernel/src/hello_service.rs\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_CONTENT_SOURCE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_TEXT: &str = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_REFERENCE_SCHEMA: &str = \"raios.builtin_artifact_reference.v0\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_REFERENCE_ID: &str = \"builtin_artifact_reference.svc.demo.hello.v0\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_REFERENCE_KIND: &str = \"repo_artifact_bytes_snapshot\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_REFERENCE_LOCATOR: &str = \"seed-kernel/artifacts/svc.demo.hello.builtin.artifact\";\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_BYTES_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_REFERENCE_TEXT: &str = {};\n\
pub(crate) const HELLO_BUILTIN_ARTIFACT_REFERENCE_HASH: [u8; 32] = {};\n\
pub(crate) const HELLO_HOST_BOUND_DESCRIPTOR_SOURCE: &str = {};\n",
            rust_string(&envelope_text),
            rust_byte_array(&envelope_hash),
            rust_byte_array(&public_key),
            rust_byte_array(&signature_der),
            rust_byte_array(&public_key_hash),
            rust_byte_array(&signature_hash),
            rust_byte_array(&current_hash),
            rust_byte_array(&artifact_identity_hash),
            rust_string(&artifact_identity_envelope_text),
            rust_byte_array(&artifact_identity_envelope_hash),
            rust_byte_array(&artifact_identity_public_key),
            rust_byte_array(&artifact_identity_signature_der),
            rust_byte_array(&artifact_identity_public_key_hash),
            rust_byte_array(&artifact_identity_signature_hash),
            rust_string(&artifact_identity_v2_source),
            rust_byte_array(&artifact_identity_v2_hash),
            rust_string(&artifact_identity_v2_envelope_text),
            rust_byte_array(&artifact_identity_v2_envelope_hash),
            rust_byte_array(&artifact_identity_v2_public_key),
            rust_byte_array(&artifact_identity_v2_signature_der),
            rust_byte_array(&artifact_identity_v2_public_key_hash),
            rust_byte_array(&artifact_identity_v2_signature_hash),
            rust_byte_array(&artifact_content_source_hash),
            rust_string(&artifact_content_binding_text),
            rust_byte_array(&artifact_content_binding_hash),
            rust_byte_array(&artifact_bytes_hash),
            rust_string(&artifact_reference_text),
            rust_byte_array(&artifact_reference_hash),
            rust_string(&host_source)
        ),
    )
    .unwrap();
    attest_echo_wasm_artifact(&manifest_dir, &out_dir);
}

fn attest_echo_wasm_artifact(manifest_dir: &std::path::Path, out_dir: &std::path::Path) {
    let artifact_path = manifest_dir.join("artifacts/svc.demo.echo.wasm");
    let descriptor_path =
        manifest_dir.join("descriptors/svc.demo.echo.wasm_artifact_identity.desc");
    let public_key_path =
        manifest_dir.join("descriptors/svc.demo.echo.wasm_artifact_identity.p256.pub.hex");
    let signature_path =
        manifest_dir.join("descriptors/svc.demo.echo.wasm_artifact_identity.p256.sig.der.hex");

    let artifact_bytes = fs::read(artifact_path).unwrap();
    let descriptor_source = fs::read_to_string(descriptor_path).unwrap();
    let public_key = read_hex_file(public_key_path);
    let signature_der = read_hex_file(signature_path);
    verify_p256_signature(&public_key, &signature_der, descriptor_source.as_bytes());

    let artifact_hash = Sha256::digest(&artifact_bytes);
    let artifact_hash_hex = sha256_hex(&artifact_hash);
    let expected_artifact_hash = format!("sha256:{}", artifact_hash_hex);
    assert_eq!(
        text_field(&descriptor_source, "artifact_reference_bytes_sha256"),
        Some(expected_artifact_hash.as_str())
    );
    assert_eq!(
        text_field(&descriptor_source, "id"),
        Some("builtin_artifact_identity.svc.demo.echo.wasm.v0")
    );
    assert_eq!(
        text_field(&descriptor_source, "service_id"),
        Some("svc.demo.echo")
    );
    assert_eq!(
        text_field(&descriptor_source, "artifact_id"),
        Some("wasm:svc.demo.echo")
    );
    assert_eq!(
        text_field(&descriptor_source, "artifact_reference_locator"),
        Some("seed-kernel/artifacts/svc.demo.echo.wasm")
    );
    assert_eq!(
        text_field(&descriptor_source, "accepts_external_artifact_bytes"),
        Some("false")
    );
    assert_eq!(
        text_field(
            &descriptor_source,
            "validates_artifact_with_wasmi_module_new"
        ),
        Some("true")
    );
    assert_eq!(
        text_field(&descriptor_source, "executes_artifact"),
        Some("false")
    );
    assert_eq!(
        text_field(&descriptor_source, "links_imports"),
        Some("false")
    );
    assert_eq!(
        text_field(&descriptor_source, "maps_executable_pages"),
        Some("false")
    );

    let descriptor_hash = Sha256::digest(descriptor_source.as_bytes());
    let descriptor_hash_hex = sha256_hex(&descriptor_hash);
    let public_key_hash = Sha256::digest(&public_key);
    let public_key_hash_hex = sha256_hex(&public_key_hash);
    let signature_hash = Sha256::digest(&signature_der);
    let signature_hash_hex = sha256_hex(&signature_hash);
    let envelope_text = format!(
        "schema=raios.builtin_artifact_identity_signature_envelope.v0\n\
id=artifact_identity_signature.wasm.svc.demo.echo.v0\n\
algorithm=ecdsa_p256_sha256_asn1_der\n\
payload_identity_id=builtin_artifact_identity.svc.demo.echo.wasm.v0\n\
payload_artifact_id=wasm:svc.demo.echo\n\
payload_sha256=sha256:{}\n\
public_key_sha256=sha256:{}\n\
signature_sha256=sha256:{}\n\
verification_phase=runtime_before_wasm_artifact_validation\n\
trust_scope=current_boot_wasm_artifact_identity_candidate\n\
classification=local_only\n\
authorizes_external_artifact_load=false\n\
authorizes_persistent_install=false\n\
authorizes_rollback_install=false",
        descriptor_hash_hex, public_key_hash_hex, signature_hash_hex
    );
    let envelope_hash = Sha256::digest(envelope_text.as_bytes());

    fs::write(
        out_dir.join("echo_wasm_artifact.rs"),
        format!(
            "pub(crate) const ECHO_WASM_ARTIFACT_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/artifacts/svc.demo.echo.wasm\"));\n\
pub(crate) const ECHO_WASM_ARTIFACT_BYTES_HASH: [u8; 32] = {};\n\
pub(crate) const ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE: &str = {};\n\
pub(crate) const ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH: [u8; 32] = {};\n\
pub(crate) const ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT: &str = {};\n\
pub(crate) const ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH: [u8; 32] = {};\n",
            rust_byte_array(&artifact_hash),
            rust_string(&descriptor_source),
            rust_byte_array(&descriptor_hash),
            rust_string(&envelope_text),
            rust_byte_array(&envelope_hash)
        ),
    )
    .unwrap();
}

fn read_ordered_source_set(repo_root: &std::path::Path, source_set: &[&str]) -> Vec<u8> {
    let mut out = Vec::new();
    for repo_relative_path in source_set {
        let file_bytes = fs::read(repo_root.join(repo_relative_path)).unwrap();
        frame_source_file(&mut out, repo_relative_path.as_bytes(), &file_bytes);
    }
    out
}

fn frame_source_file(out: &mut Vec<u8>, path: &[u8], file_bytes: &[u8]) {
    // Source-set framing is, in manifest order, decimal path byte length + "\n"
    // + path bytes + "\n" + decimal file byte length + "\n" + file bytes + "\n".
    out.extend_from_slice(path.len().to_string().as_bytes());
    out.push(b'\n');
    out.extend_from_slice(path);
    out.push(b'\n');
    out.extend_from_slice(file_bytes.len().to_string().as_bytes());
    out.push(b'\n');
    out.extend_from_slice(file_bytes);
    out.push(b'\n');
}

fn read_hex_file(path: PathBuf) -> Vec<u8> {
    parse_hex(fs::read_to_string(path).unwrap().trim()).unwrap()
}

fn verify_p256_signature(public_key: &[u8], signature_der: &[u8], payload: &[u8]) {
    let verifying_key = VerifyingKey::from_sec1_bytes(public_key).unwrap();
    let signature = Signature::from_der(signature_der).unwrap();
    verifying_key.verify(payload, &signature).unwrap();
}

fn text_field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let mut found = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let eq = line.find('=')?;
        if line[..eq].trim() == key {
            if found.is_some() {
                return None;
            }
            found = Some(line[eq + 1..].trim());
        }
    }
    found
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
