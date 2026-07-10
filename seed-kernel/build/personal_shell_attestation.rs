//! Build-time evidence binding for the one checked-in personal-shell proof.

use sha2::{Digest, Sha256};
use std::{fs, path::Path};

use crate::{
    read_hex_file, rust_byte_array, rust_string, sha256_hex, text_field, verify_p256_signature,
};

const SERVICE_ID: &str = "svc.user.shell";
const ARTIFACT_ID: &str = "wasm:svc.user.shell";
const ARTIFACT_LOCATOR: &str = "seed-kernel/artifacts/svc.user.shell.wasm";
const IDENTITY_ID: &str = "builtin_artifact_identity.svc.user.shell.wasm.v0";
const LOAD_DESCRIPTOR_ID: &str = "load_descriptor.current_boot.svc.user.shell.v0";
const LOAD_DESCRIPTOR_LOCATOR: &str = "current_boot.service_load_descriptor.svc.user.shell.v0";
const AUTHORIZED_IMPORTS: &str =
    "ui.viewport,ui.context_len,ui.context_read,ui.input_len,ui.input_read,ui.frame_submit";

pub(crate) fn attest(manifest_dir: &Path, out_dir: &Path) {
    let artifact_path = manifest_dir.join("artifacts/svc.user.shell.wasm");
    let identity_path = manifest_dir.join("descriptors/svc.user.shell.wasm_artifact_identity.desc");
    let identity_public_key_path =
        manifest_dir.join("descriptors/svc.user.shell.wasm_artifact_identity.p256.pub.hex");
    let identity_signature_path =
        manifest_dir.join("descriptors/svc.user.shell.wasm_artifact_identity.p256.sig.der.hex");
    let load_path = manifest_dir.join("descriptors/svc.user.shell.current_boot_load.desc");
    let load_public_key_path =
        manifest_dir.join("descriptors/svc.user.shell.current_boot_load.p256.pub.hex");
    let load_signature_path =
        manifest_dir.join("descriptors/svc.user.shell.current_boot_load.p256.sig.der.hex");

    let artifact_bytes = fs::read(artifact_path).unwrap();
    let identity_source = fs::read_to_string(identity_path).unwrap();
    let identity_public_key = read_hex_file(identity_public_key_path);
    let identity_signature = read_hex_file(identity_signature_path);
    let load_source = fs::read_to_string(load_path).unwrap();
    let load_public_key = read_hex_file(load_public_key_path);
    let load_signature = read_hex_file(load_signature_path);
    verify_p256_signature(
        &identity_public_key,
        &identity_signature,
        identity_source.as_bytes(),
    );
    verify_p256_signature(&load_public_key, &load_signature, load_source.as_bytes());

    let artifact_hash = Sha256::digest(&artifact_bytes);
    let artifact_hash_hex = sha256_hex(&artifact_hash);
    let artifact_hash_ref = format!("sha256:{artifact_hash_hex}");
    for (key, expected) in [
        (
            "canonicalization",
            "raios.builtin_artifact_identity.canonical.v0",
        ),
        ("schema", "raios.builtin_artifact_identity.v0"),
        ("id", IDENTITY_ID),
        ("service_id", SERVICE_ID),
        ("artifact_id", ARTIFACT_ID),
        ("artifact_kind", "wasm32_unknown_unknown_service_module"),
        (
            "artifact_reference_schema",
            "raios.builtin_artifact_reference.v0",
        ),
        (
            "artifact_reference_id",
            "builtin_artifact_reference.svc.user.shell.wasm.v0",
        ),
        (
            "artifact_reference_kind",
            "repo_wasm_artifact_bytes_snapshot",
        ),
        ("artifact_reference_locator", ARTIFACT_LOCATOR),
        (
            "artifact_reference_bytes_sha256",
            artifact_hash_ref.as_str(),
        ),
        (
            "artifact_reference_accepts_external_artifact_bytes",
            "false",
        ),
        ("artifact_reference_validates_with_wasmi_module_new", "true"),
        ("artifact_reference_executes_artifact", "false"),
        ("artifact_reference_links_imports", "false"),
        ("artifact_reference_maps_executable_pages", "false"),
        ("scope", "current_boot"),
        ("classification", "local_only"),
        ("persistence", "none"),
        ("trust_tier", "dev_key_not_owner_sealed"),
        ("owner_sealed", "false"),
        ("accepts_external_artifact_bytes", "false"),
        ("validates_artifact_with_wasmi_module_new", "true"),
        ("executes_artifact", "false"),
        ("links_imports", "false"),
        ("maps_executable_pages", "false"),
        ("writes_persistent_state", "false"),
        ("authorizes_external_artifact_load", "false"),
        ("authorizes_persistent_install", "false"),
        ("authorizes_rollback_install", "false"),
    ] {
        assert_eq!(
            text_field(&identity_source, key),
            Some(expected),
            "personal-shell identity descriptor field mismatch: {key}"
        );
    }

    let identity_hash = Sha256::digest(identity_source.as_bytes());
    let identity_hash_hex = sha256_hex(&identity_hash);
    let identity_hash_ref = format!("sha256:{identity_hash_hex}");
    for (key, expected) in [
        (
            "canonicalization",
            "raios.current_boot_load_descriptor.canonical.v0",
        ),
        ("schema", "raios.current_boot_load_descriptor.v0"),
        ("id", LOAD_DESCRIPTOR_ID),
        (
            "source_kind",
            "current_boot_wasm_service_load_descriptor_source",
        ),
        ("source_locator", LOAD_DESCRIPTOR_LOCATOR),
        ("service_id", SERVICE_ID),
        ("artifact_id", ARTIFACT_ID),
        ("artifact_kind", "wasm32_unknown_unknown_service_module"),
        (
            "artifact_identity_schema",
            "raios.builtin_artifact_identity.v0",
        ),
        ("artifact_identity_id", IDENTITY_ID),
        ("artifact_identity_sha256", identity_hash_ref.as_str()),
        ("artifact_reference_locator", ARTIFACT_LOCATOR),
        (
            "artifact_reference_bytes_sha256",
            artifact_hash_ref.as_str(),
        ),
        ("scope", "current_boot"),
        ("classification", "local_only"),
        ("persistence", "none"),
        ("trust_tier", "dev_key_not_owner_sealed"),
        ("owner_sealed", "false"),
        ("service_slot_id", "ram_only:svc.user.shell"),
        (
            "service_capability",
            "cap.service.personal_shell_proof.current_boot",
        ),
        ("execution_model", "wasmi_interpreter_current_boot"),
        ("capability_envelope", "wasmi_linker_import_surface"),
        ("authorized_host_imports", AUTHORIZED_IMPORTS),
        ("authorized_host_import_count", "6"),
        ("entrypoint", "raios_service_main"),
        ("authorizes_current_boot_wasm_execution", "true"),
        ("accepts_external_artifact_bytes", "false"),
        ("validates_artifact_with_wasmi_module_new", "true"),
        ("loads_external_artifact", "false"),
        ("maps_executable_pages", "false"),
        ("writes_persistent_state", "false"),
        ("authorizes_persistent_install", "false"),
        ("authorizes_rollback_install", "false"),
    ] {
        assert_eq!(
            text_field(&load_source, key),
            Some(expected),
            "personal-shell load descriptor field mismatch: {key}"
        );
    }

    let identity_envelope = signature_envelope(
        "raios.builtin_artifact_identity_signature_envelope.v0",
        "artifact_identity_signature.wasm.svc.user.shell.v0",
        "payload_identity_id=builtin_artifact_identity.svc.user.shell.wasm.v0",
        "payload_artifact_id=wasm:svc.user.shell",
        &identity_source,
        &identity_public_key,
        &identity_signature,
        "runtime_before_wasm_artifact_validation",
        "current_boot_wasm_artifact_identity_candidate",
    );
    let identity_envelope_hash = Sha256::digest(identity_envelope.as_bytes());
    let load_envelope = signature_envelope(
        "raios.descriptor_source_signature_envelope.v0",
        "descriptor_source_signature.current_boot_load.svc.user.shell.v0",
        "payload_source_locator=current_boot.service_load_descriptor.svc.user.shell.v0",
        "payload_source_kind=current_boot_wasm_service_load_descriptor_source",
        &load_source,
        &load_public_key,
        &load_signature,
        "build_time_before_current_boot_wasm_service_descriptor_selection",
        "current_boot_wasm_service_load_descriptor_candidate",
    );
    let load_envelope_hash = Sha256::digest(load_envelope.as_bytes());

    fs::write(
        out_dir.join("personal_shell_wasm_artifact.rs"),
        format!(
            r#"pub(crate) const PERSONAL_SHELL_WASM_ARTIFACT_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/artifacts/svc.user.shell.wasm"));
pub(crate) const PERSONAL_SHELL_WASM_ARTIFACT_BYTES_HASH: [u8; 32] = {};
pub(crate) const PERSONAL_SHELL_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE: &str = {};
pub(crate) const PERSONAL_SHELL_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH: [u8; 32] = {};
pub(crate) const PERSONAL_SHELL_WASM_ARTIFACT_IDENTITY_PUBLIC_KEY_SEC1: &[u8] = &{};
pub(crate) const PERSONAL_SHELL_WASM_ARTIFACT_IDENTITY_SIGNATURE_DER: &[u8] = &{};
pub(crate) const PERSONAL_SHELL_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT: &str = {};
pub(crate) const PERSONAL_SHELL_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH: [u8; 32] = {};
"#,
            rust_byte_array(&artifact_hash),
            rust_string(&identity_source),
            rust_byte_array(&identity_hash),
            rust_byte_array(&identity_public_key),
            rust_byte_array(&identity_signature),
            rust_string(&identity_envelope),
            rust_byte_array(&identity_envelope_hash),
        ),
    )
    .unwrap();
    fs::write(
        out_dir.join("personal_shell_current_boot_load_descriptor.rs"),
        format!(
            r#"pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_SOURCE: &str = {};
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_HASH: [u8; 32] = {};
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_PUBLIC_KEY_SEC1: &[u8] = &{};
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_SIGNATURE_DER: &[u8] = &{};
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_TEXT: &str = {};
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_HASH: [u8; 32] = {};
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_SERVICE_ID: &str = "svc.user.shell";
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_ARTIFACT_ID: &str = "wasm:svc.user.shell";
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_ARTIFACT_BYTES_HASH: [u8; 32] = {};
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_SERVICE_SLOT_ID: &str = "ram_only:svc.user.shell";
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_SERVICE_CAPABILITY: &str = "cap.service.personal_shell_proof.current_boot";
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORTS: &str = "{}";
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORT_COUNT: u64 = 6;
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_OWNER_SEALED: bool = false;
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZES_CURRENT_BOOT_WASM_EXECUTION: bool = true;
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_ACCEPTS_EXTERNAL_ARTIFACT_BYTES: bool = false;
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_VALIDATES_WITH_WASMI_MODULE_NEW: bool = true;
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_LOADS_EXTERNAL_ARTIFACT: bool = false;
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_MAPS_EXECUTABLE_PAGES: bool = false;
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_WRITES_PERSISTENT_STATE: bool = false;
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZES_PERSISTENT_INSTALL: bool = false;
pub(crate) const PERSONAL_SHELL_LOAD_DESCRIPTOR_AUTHORIZES_ROLLBACK_INSTALL: bool = false;
"#,
            rust_string(&load_source),
            rust_byte_array(&Sha256::digest(load_source.as_bytes())),
            rust_byte_array(&load_public_key),
            rust_byte_array(&load_signature),
            rust_string(&load_envelope),
            rust_byte_array(&load_envelope_hash),
            rust_byte_array(&artifact_hash),
            AUTHORIZED_IMPORTS,
        ),
    )
    .unwrap();
}

fn signature_envelope(
    schema: &str,
    id: &str,
    subject_a: &str,
    subject_b: &str,
    payload: &str,
    public_key: &[u8],
    signature: &[u8],
    verification_phase: &str,
    trust_scope: &str,
) -> String {
    format!(
        "schema={schema}\nid={id}\nalgorithm=ecdsa_p256_sha256_asn1_der\n{subject_a}\n{subject_b}\npayload_sha256=sha256:{}\npublic_key_sha256=sha256:{}\nsignature_sha256=sha256:{}\nverification_phase={verification_phase}\ntrust_scope={trust_scope}\nclassification=local_only\ntrust_tier=dev_key_not_owner_sealed\nowner_sealed=false\nauthorizes_external_artifact_load=false\nauthorizes_persistent_install=false\nauthorizes_rollback_install=false",
        sha256_hex(&Sha256::digest(payload.as_bytes())),
        sha256_hex(&Sha256::digest(public_key)),
        sha256_hex(&Sha256::digest(signature)),
    )
}
