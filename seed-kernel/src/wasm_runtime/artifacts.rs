use super::*;

include!(concat!(env!("OUT_DIR"), "/echo_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/bufecho_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/certwindow_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/httphead_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/certspki_wasm_artifact.rs"));
include!(concat!(env!("OUT_DIR"), "/dnsparse_wasm_artifact.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/build_assembler_wasm_artifact.rs"
));
include!(concat!(env!("OUT_DIR"), "/personal_shell_wasm_artifact.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/personal_shell_current_boot_load_descriptor.rs"
));

const EMPTY_WASM_MODULE: &[u8] = b"\0asm\x01\0\0\0";

pub(crate) const ECHO_WASM_FUEL_BUDGET: u64 = 10_000;
/// Deliberately-tiny fuel budget for the labeled fuel-starvation fault injection
/// (`run_echo_fuel_starved`). A real echo invoke under this budget exhausts fuel
/// after a single metered step and traps with a genuine wasmi `OutOfFuel`.
pub(crate) const ECHO_WASM_FUEL_STARVED_BUDGET: u64 = 1;

pub(crate) const WASM_HARDENING_CASE_COUNT: usize = 4;
pub(crate) const FORBIDDEN_IMPORT_MODULE: &str = "env";
pub(crate) const FORBIDDEN_IMPORT_NAME: &str = "forbidden_write";
pub(crate) const ECHO_SERVICE_ID: &str = "svc.demo.echo";
/// Declared/known only in NET-1. No corresponding linker arm exists.
pub(crate) const WASM_HOST_IMPORT_ABI_V1: &str = HOST_IMPORT_ABI_V1;
pub(crate) const KNOWN_BEYOND_ENV_HOST_IMPORTS_V1: &[HostImportSignature] =
    BEYOND_ENV_HOST_IMPORTS_V1;
pub(crate) const ECHO_AUTHORIZED_IMPORTS: &[(&str, &str)] =
    &[("env", "log"), ("env", "counter_get")];
pub(crate) const BUFECHO_SERVICE_ID: &str = "svc.demo.bufecho";
pub(crate) const BUFECHO_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const CERTWINDOW_SERVICE_ID: &str = "svc.demo.certwindow";
pub(crate) const CERTWINDOW_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const CERTWINDOW_WASM_FUEL_BUDGET: u64 = 1_000_000;
pub(crate) const HTTPHEAD_SERVICE_ID: &str = "svc.demo.httphead";
pub(crate) const HTTPHEAD_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const HTTPHEAD_WASM_FUEL_BUDGET: u64 = 1_000_000;
pub(crate) const CERTSPKI_SERVICE_ID: &str = "svc.demo.certspki";
pub(crate) const CERTSPKI_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const CERTSPKI_WASM_FUEL_BUDGET: u64 = 1_000_000;
pub(crate) const DNSPARSE_SERVICE_ID: &str = "svc.demo.dnsparse";
pub(crate) const DNSPARSE_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const DNSPARSE_WASM_FUEL_BUDGET: u64 = 1_000_000;
pub(crate) const BUILD_ASSEMBLER_SERVICE_ID: &str = "svc.build.assembler";
pub(crate) const BUILD_ASSEMBLER_AUTHORIZED_IMPORTS: &[(&str, &str)] = &[
    ("env", "input_len"),
    ("env", "input_read"),
    ("env", "output_write"),
];
pub(crate) const PERSONAL_SHELL_WASM_FUEL_BUDGET: u64 = 250_000;

#[used]
static WASMI_COMPILE_PROOF: fn() -> bool = validate_empty_module_bytes;
#[used]
static ECHO_WASM_ARTIFACT_PROOF: fn() -> bool = validate_echo_wasm_artifact;
#[used]
static BUFECHO_WASM_ARTIFACT_PROOF: fn() -> bool = validate_bufecho_wasm_artifact;
#[used]
static CERTWINDOW_WASM_ARTIFACT_PROOF: fn() -> bool = validate_certwindow_wasm_artifact;
#[used]
static HTTPHEAD_WASM_ARTIFACT_PROOF: fn() -> bool = validate_httphead_wasm_artifact;
#[used]
static CERTSPKI_WASM_ARTIFACT_PROOF: fn() -> bool = validate_certspki_wasm_artifact;
#[used]
static DNSPARSE_WASM_ARTIFACT_PROOF: fn() -> bool = validate_dnsparse_wasm_artifact;
#[used]
static BUILD_ASSEMBLER_WASM_ARTIFACT_PROOF: fn() -> bool = validate_build_assembler_wasm_artifact;

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

pub(crate) fn validate_bufecho_wasm_artifact() -> bool {
    let wasm = Vec::from(BUFECHO_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == BUFECHO_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(BUFECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == BUFECHO_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(BUFECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == BUFECHO_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_certwindow_wasm_artifact() -> bool {
    let wasm = Vec::from(CERTWINDOW_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == CERTWINDOW_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(CERTWINDOW_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == CERTWINDOW_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(CERTWINDOW_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == CERTWINDOW_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_httphead_wasm_artifact() -> bool {
    let wasm = Vec::from(HTTPHEAD_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == HTTPHEAD_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(HTTPHEAD_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == HTTPHEAD_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(HTTPHEAD_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == HTTPHEAD_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_certspki_wasm_artifact() -> bool {
    let wasm = Vec::from(CERTSPKI_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == CERTSPKI_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(CERTSPKI_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == CERTSPKI_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(CERTSPKI_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == CERTSPKI_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_dnsparse_wasm_artifact() -> bool {
    let wasm = Vec::from(DNSPARSE_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == DNSPARSE_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(DNSPARSE_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == DNSPARSE_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(DNSPARSE_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == DNSPARSE_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn validate_build_assembler_wasm_artifact() -> bool {
    let wasm = Vec::from(BUILD_ASSEMBLER_WASM_ARTIFACT_BYTES).into_boxed_slice();
    let bytes: &[u8] = &wasm;

    sha256_bytes(bytes) == BUILD_ASSEMBLER_WASM_ARTIFACT_BYTES_HASH
        && sha256_bytes(BUILD_ASSEMBLER_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_SOURCE.as_bytes())
            == BUILD_ASSEMBLER_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH
        && sha256_bytes(BUILD_ASSEMBLER_WASM_ARTIFACT_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == BUILD_ASSEMBLER_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH
        && validate_module_bytes(bytes)
}

pub(crate) fn run_build_assembler_roundtrip(input: &[u8], fuel_budget: u64) -> EchoRunEvidence {
    super::envelope::execute_validated_module_bytes(
        BUILD_ASSEMBLER_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        BUILD_ASSEMBLER_SERVICE_ID,
        true,
        BUILD_ASSEMBLER_AUTHORIZED_IMPORTS,
        validate_build_assembler_wasm_artifact(),
        input,
        fuel_budget,
    )
}

pub(crate) fn loader_available() -> bool {
    true
}

pub(crate) fn validate_module_bytes(bytes: &[u8]) -> bool {
    let engine = Box::new(wasmi::Engine::default());
    wasmi::Module::new(&engine, bytes).is_ok()
}
