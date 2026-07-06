use alloc::{boxed::Box, vec::Vec};
use raios_core::sha256_bytes;

include!(concat!(env!("OUT_DIR"), "/echo_wasm_artifact.rs"));

const EMPTY_WASM_MODULE: &[u8] = b"\0asm\x01\0\0\0";

#[used]
static WASMI_COMPILE_PROOF: fn() -> bool = validate_empty_module_bytes;
#[used]
static ECHO_WASM_ARTIFACT_PROOF: fn() -> bool = validate_echo_wasm_artifact;

pub(crate) fn validate_empty_module_bytes() -> bool {
    let wasm = Vec::from(EMPTY_WASM_MODULE).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    validate_module_bytes(bytes)
}

pub(crate) fn validate_echo_wasm_artifact() -> bool {
    let wasm = Vec::from(ECHO_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == ECHO_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == ECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == ECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

fn validate_module_bytes(bytes: &[u8]) -> bool {
    let engine = Box::new(wasmi::Engine::default());
    wasmi::Module::new(&engine, bytes).is_ok()
}
