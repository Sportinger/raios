use alloc::{boxed::Box, vec::Vec};

const EMPTY_WASM_MODULE: &[u8] = b"\0asm\x01\0\0\0";

#[used]
static WASMI_COMPILE_PROOF: fn() -> bool = validate_empty_module_bytes;

pub(crate) fn validate_empty_module_bytes() -> bool {
    let engine = Box::new(wasmi::Engine::default());
    let wasm = Vec::from(EMPTY_WASM_MODULE).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    wasmi::Module::new(&engine, bytes).is_ok()
}
