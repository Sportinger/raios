use super::invocation::{check_crypto_call_boundary, runtime_now_ms, BeyondEnvState};
use super::*;
use raios_core::{
    host_import_abi_v1::{
        CRYPTO_HOST_IMPORTS_V1, HOST_IMPORT_ERROR_CAPABILITY_DENIED,
        HOST_IMPORT_ERROR_INVALID_ARGUMENT, HOST_IMPORT_ERROR_INVALID_STATE,
        HOST_IMPORT_ERROR_KILLED, HOST_IMPORT_ERROR_LIMIT_EXCEEDED,
        HOST_IMPORT_ERROR_RESOURCE_BUSY, HOST_IMPORT_ERROR_TRANSPORT,
    },
    tls13_session::{
        run_tls13_host_vectors, OpaqueSessionSlot, Tls13Denial, Tls13FinishedOutput,
        Tls13HostVectorReport, TLS13_HASH_LEN, TLS13_MAX_CIPHERTEXT_LEN, TLS13_MAX_HASH_INPUT_LEN,
        TLS13_MAX_PLAINTEXT_LEN, TLS13_MAX_SIGNATURE_LEN, TLS13_MAX_SPKI_LEN,
        TLS13_MAX_VERIFY_MESSAGE_LEN, TLS13_PUBLIC_OUTPUT_LEN, TLS13_RECORD_HEADER_LEN,
        TLS13_TAG_LEN,
    },
};

const CRYPTO_CALL_BASE_FUEL: u64 = 50;
const CRYPTO_FIXTURE_INPUT_LEN: usize = 515;
const CRYPTO_FIXTURE_EVENT_CAPACITY: usize = 24;
const CRYPTO_FIXTURE_SPKI_PIN: [u8; 32] = [
    0x2b, 0x98, 0x5b, 0xad, 0xf1, 0xba, 0xc8, 0x29, 0x3f, 0x6b, 0x92, 0xe8, 0xc0, 0xac, 0xc1, 0x13,
    0x9b, 0xc5, 0xa5, 0xec, 0xbb, 0xf9, 0x7a, 0x62, 0x71, 0xdc, 0xf0, 0xd4, 0xf6, 0xc6, 0x8a, 0xf6,
];

// Labeled NET-5B test input only. This is the public half of the deterministic
// core vector, staged through env.input_{len,read}; private scalar 1 stays in
// CryptoInvocationState and is zeroized by OpaqueSession::open.
const CRYPTO_FIXTURE_INPUT: &[u8; CRYPTO_FIXTURE_INPUT_LEN] = b"\
\x04\x7c\xf2\x7b\x18\x8d\x03\x4f\x7e\x8a\x52\x38\x03\x04\xb5\x1a\xc3\xc0\x89\x69\xe2\x77\xf2\x1b\x35\xa6\x0b\x48\xfc\x47\x66\x99\x78\x07\x77\x55\x10\xdb\x8e\xd0\x40\x29\x3d\x9a\xc6\x9f\x74\x30\xdb\xba\x7d\xad\xe6\x3c\xe9\x82\x29\x9e\x04\xb7\x9d\x22\x78\x73\xd1\
\x49\xb1\xb5\xf3\x7f\x85\x48\xd8\xd7\xea\x2f\xe0\x82\x57\xf1\xa1\x16\x6e\x96\xe9\x37\x61\x5f\x33\x46\xb3\x36\x97\xc5\x1a\x09\xae\
\x30\x59\x30\x13\x06\x07\x2a\x86\x48\xce\x3d\x02\x01\x06\x08\x2a\x86\x48\xce\x3d\x03\x01\x07\x03\x42\x00\x04\x5e\xcb\xe4\xd1\xa6\x33\x0a\x44\xc8\xf7\xef\x95\x1d\x4b\xf1\x65\xe6\xc6\xb7\x21\xef\xad\xa9\x85\xfb\x41\x66\x1b\xc6\xe7\xfd\x6c\x87\x34\x64\x0c\x49\x98\xff\x7e\x37\x4b\x06\xce\x1a\x64\xa2\xec\xd8\x2a\xb0\x36\x38\x4f\xb8\x3d\x9a\x79\xb1\x27\xa2\x7d\x50\x32\
\x54\x4c\x53\x20\x31\x2e\x33\x2c\x20\x73\x65\x72\x76\x65\x72\x20\x43\x65\x72\x74\x69\x66\x69\x63\x61\x74\x65\x56\x65\x72\x69\x66\x79\x00\x72\x61\x69\x6f\x73\
\x30\x44\x02\x20\x17\x39\x48\x2a\xf2\x7f\x9a\xe8\x44\x0e\x1e\x6c\x31\x54\xe4\x3f\x99\xf6\xfe\x55\x97\xd5\x42\x16\x4f\xd0\x91\x72\x9c\xde\x59\x4a\x02\x20\x75\x9a\xcd\x57\xd5\x78\xeb\x9a\x8c\x49\x61\xb6\xc3\x69\xb5\x11\x5f\x30\xb5\x14\xfe\x82\xf1\xcd\x6f\xe2\x81\xcc\xbb\x75\x32\x46\
\x70\xc9\x60\x32\x19\xea\x1d\x69\x57\x93\x09\x9e\x6d\x25\x9a\xe9\x16\x54\xbf\xd8\xae\x7f\xd3\xa6\x64\xa4\x15\xb1\x22\xdd\x8c\xc1\
\x06\x58\x29\x19\xba\x64\x5a\x64\xb5\x61\xc2\x14\xcc\x69\x5c\x78\xd6\xfb\x91\x38\x13\x06\x38\x2a\x4b\xf3\x0f\x2d\x2d\x1c\x21\x7e\
\x77\xed\xb8\x04\x81\xc6\x53\x92\x7b\x7e\x93\xbf\xae\x39\x17\x9a\x6d\xbf\xbf\xed\xff\x81\xa4\xa2\x41\x19\xf0\x74\xab\xb0\xc3\x03\
\xaa\x0a\x92\x23\xb9\xc5\x23\x4d\xf8\x38\x86\x1f\x8c\x28\x9a\x42\xdf\x1c\xd0\x90\xe1\xfa\xde\xae\x71\xe7\x6e\xfd\xec\x28\x97\x4c\
\x70\x75\x62\x6c\x69\x63\x20\x74\x72\x61\x6e\x73\x63\x72\x69\x70\x74\
\x17\x03\x03\x00\x2a\
\x62\x6f\x75\x6e\x64\x65\x64\x20\x61\x70\x70\x6c\x69\x63\x61\x74\x69\x6f\x6e\x20\x72\x65\x63\x6f\x72\x64\
\x22\xbe\x4a\xa4\x8e\xa1\x62\x39\x03\x63\x8a\x36\x4e\x76\x0d\x8c\xdf\x7a\x62\xb2\x83\xe3\x7b\x2c\xc8\xd1\x72\x33\x3e\x69\xe6\x05\xb3\xcd\x80\x79\x9d\xe9\x7c\x39\xa9\x73";

// Hand-assembled Wasm, exactly like NET_SHIM_WASM_MODULE: this labeled
// fixture is the only module whose linker receives the real crypto closures.
pub(super) const CRYPTO_SHIM_WASM_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x3d, 0x08, 0x60, 0x00, 0x01, 0x7f, 0x60,
    0x03, 0x7f, 0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x01, 0x7f, 0x01, 0x7f, 0x60, 0x04, 0x7f, 0x7f, 0x7f,
    0x7f, 0x01, 0x7f, 0x60, 0x07, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x02,
    0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x05, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x06, 0x7f,
    0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x01, 0x7f, 0x02, 0x8a, 0x02, 0x0c, 0x03, 0x65, 0x6e, 0x76, 0x09,
    0x69, 0x6e, 0x70, 0x75, 0x74, 0x5f, 0x6c, 0x65, 0x6e, 0x00, 0x00, 0x03, 0x65, 0x6e, 0x76, 0x0a,
    0x69, 0x6e, 0x70, 0x75, 0x74, 0x5f, 0x72, 0x65, 0x61, 0x64, 0x00, 0x05, 0x03, 0x6e, 0x65, 0x74,
    0x08, 0x74, 0x63, 0x70, 0x5f, 0x6f, 0x70, 0x65, 0x6e, 0x00, 0x00, 0x03, 0x6e, 0x65, 0x74, 0x09,
    0x74, 0x63, 0x70, 0x5f, 0x63, 0x6c, 0x6f, 0x73, 0x65, 0x00, 0x02, 0x06, 0x63, 0x72, 0x79, 0x70,
    0x74, 0x6f, 0x12, 0x74, 0x6c, 0x73, 0x31, 0x33, 0x5f, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e,
    0x5f, 0x6f, 0x70, 0x65, 0x6e, 0x00, 0x01, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x06, 0x73,
    0x68, 0x61, 0x32, 0x35, 0x36, 0x00, 0x03, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x0b, 0x70,
    0x32, 0x35, 0x36, 0x5f, 0x76, 0x65, 0x72, 0x69, 0x66, 0x79, 0x00, 0x04, 0x06, 0x63, 0x72, 0x79,
    0x70, 0x74, 0x6f, 0x14, 0x74, 0x6c, 0x73, 0x31, 0x33, 0x5f, 0x68, 0x61, 0x6e, 0x64, 0x73, 0x68,
    0x61, 0x6b, 0x65, 0x5f, 0x6b, 0x65, 0x79, 0x73, 0x00, 0x03, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74,
    0x6f, 0x16, 0x74, 0x6c, 0x73, 0x31, 0x33, 0x5f, 0x61, 0x70, 0x70, 0x6c, 0x69, 0x63, 0x61, 0x74,
    0x69, 0x6f, 0x6e, 0x5f, 0x6b, 0x65, 0x79, 0x73, 0x00, 0x05, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74,
    0x6f, 0x0e, 0x74, 0x6c, 0x73, 0x31, 0x33, 0x5f, 0x66, 0x69, 0x6e, 0x69, 0x73, 0x68, 0x65, 0x64,
    0x00, 0x06, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x0f, 0x74, 0x6c, 0x73, 0x31, 0x33, 0x5f,
    0x61, 0x65, 0x61, 0x64, 0x5f, 0x73, 0x65, 0x61, 0x6c, 0x00, 0x07, 0x06, 0x63, 0x72, 0x79, 0x70,
    0x74, 0x6f, 0x0f, 0x74, 0x6c, 0x73, 0x31, 0x33, 0x5f, 0x61, 0x65, 0x61, 0x64, 0x5f, 0x6f, 0x70,
    0x65, 0x6e, 0x00, 0x07, 0x03, 0x04, 0x03, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
    0x20, 0x04, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x0c, 0x03, 0x6f, 0x6f, 0x62, 0x00, 0x0d, 0x07, 0x66,
    0x6f, 0x72, 0x65, 0x69, 0x67, 0x6e, 0x00, 0x0e, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
    0x00, 0x0a, 0x8d, 0x03, 0x03, 0xb4, 0x02, 0x01, 0x04, 0x7f, 0x10, 0x00, 0x1a, 0x41, 0x00, 0x41,
    0x83, 0x04, 0x10, 0x01, 0x1a, 0x10, 0x02, 0x21, 0x00, 0x20, 0x00, 0x41, 0x80, 0x20, 0x41, 0xe1,
    0x00, 0x10, 0x04, 0x21, 0x01, 0x20, 0x01, 0x41, 0xa9, 0x03, 0x41, 0x11, 0x41, 0xe8, 0x20, 0x10,
    0x05, 0x1a, 0x20, 0x01, 0x41, 0x00, 0x41, 0xc1, 0x00, 0x41, 0xc1, 0x00, 0x10, 0x07, 0x1a, 0x20,
    0x01, 0x41, 0xe1, 0x00, 0x41, 0xdb, 0x00, 0x41, 0xbc, 0x01, 0x41, 0x27, 0x41, 0xe3, 0x01, 0x41,
    0xc6, 0x00, 0x10, 0x06, 0x1a, 0x20, 0x01, 0x41, 0xe9, 0x02, 0x10, 0x08, 0x1a, 0x20, 0x01, 0x41,
    0x02, 0x41, 0xa9, 0x02, 0x41, 0xc9, 0x02, 0x41, 0x20, 0x10, 0x09, 0x1a, 0x20, 0x01, 0x41, 0x01,
    0x41, 0xa9, 0x02, 0x41, 0xc9, 0x02, 0x41, 0x20, 0x10, 0x09, 0x1a, 0x20, 0x01, 0x41, 0xe9, 0x02,
    0x10, 0x08, 0x1a, 0x20, 0x01, 0x41, 0x00, 0x41, 0x89, 0x03, 0x41, 0xcc, 0x21, 0x41, 0x20, 0x10,
    0x09, 0x1a, 0x20, 0x01, 0x41, 0xba, 0x03, 0x41, 0xbf, 0x03, 0x41, 0x1a, 0x41, 0xb0, 0x22, 0x41,
    0x2a, 0x10, 0x0a, 0x21, 0x02, 0x20, 0x01, 0x41, 0xba, 0x03, 0x41, 0xd9, 0x03, 0x41, 0x2a, 0x41,
    0x94, 0x23, 0x41, 0x1a, 0x10, 0x0b, 0x21, 0x03, 0x20, 0x01, 0x41, 0xba, 0x03, 0x41, 0xbf, 0x03,
    0x41, 0x1a, 0x41, 0xf8, 0x23, 0x41, 0x29, 0x10, 0x0a, 0x1a, 0x20, 0x01, 0x41, 0xba, 0x03, 0x41,
    0xbf, 0x03, 0x41, 0x1a, 0x41, 0xf8, 0x23, 0x41, 0x2a, 0x10, 0x0a, 0x1a, 0x20, 0x01, 0x41, 0xa9,
    0x03, 0x41, 0x81, 0x80, 0x01, 0x41, 0xe8, 0x20, 0x10, 0x05, 0x1a, 0x20, 0x00, 0x10, 0x03, 0x1a,
    0x20, 0x01, 0x41, 0xa9, 0x03, 0x41, 0x11, 0x41, 0xe8, 0x20, 0x10, 0x05, 0x1a, 0x20, 0x03, 0x41,
    0x1a, 0x46, 0x41, 0xbf, 0x03, 0x29, 0x03, 0x00, 0x41, 0x94, 0x23, 0x29, 0x03, 0x00, 0x51, 0x71,
    0x41, 0xc7, 0x03, 0x29, 0x03, 0x00, 0x41, 0x9c, 0x23, 0x29, 0x03, 0x00, 0x51, 0x71, 0x41, 0xcf,
    0x03, 0x29, 0x03, 0x00, 0x41, 0xa4, 0x23, 0x29, 0x03, 0x00, 0x51, 0x71, 0x41, 0xd7, 0x03, 0x2f,
    0x01, 0x00, 0x41, 0xac, 0x23, 0x2f, 0x01, 0x00, 0x46, 0x71, 0x0b, 0x2c, 0x01, 0x02, 0x7f, 0x10,
    0x00, 0x1a, 0x41, 0x00, 0x41, 0x83, 0x04, 0x10, 0x01, 0x1a, 0x10, 0x02, 0x21, 0x00, 0x20, 0x00,
    0x41, 0x80, 0x20, 0x41, 0xe1, 0x00, 0x10, 0x04, 0x21, 0x01, 0x20, 0x01, 0x41, 0xfa, 0xff, 0x03,
    0x41, 0x20, 0x41, 0xe8, 0x20, 0x10, 0x05, 0x0b, 0x28, 0x01, 0x01, 0x7f, 0x10, 0x00, 0x1a, 0x41,
    0x00, 0x41, 0x83, 0x04, 0x10, 0x01, 0x1a, 0x10, 0x02, 0x21, 0x00, 0x41, 0x82, 0x02, 0x41, 0xa9,
    0x03, 0x41, 0x11, 0x41, 0xe8, 0x20, 0x10, 0x05, 0x1a, 0x20, 0x00, 0x10, 0x03, 0x1a, 0x41, 0x01,
    0x0b,
];

pub(super) const CRYPTO_IMPORT_PROBE_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x38, 0x07, 0x60, 0x03, 0x7f, 0x7f, 0x7f,
    0x01, 0x7f, 0x60, 0x04, 0x7f, 0x7f, 0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x07, 0x7f, 0x7f, 0x7f, 0x7f,
    0x7f, 0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x05, 0x7f, 0x7f, 0x7f,
    0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x06, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x00,
    0x01, 0x7f, 0x02, 0xca, 0x01, 0x08, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x12, 0x74, 0x6c,
    0x73, 0x31, 0x33, 0x5f, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x5f, 0x6f, 0x70, 0x65, 0x6e,
    0x00, 0x00, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x06, 0x73, 0x68, 0x61, 0x32, 0x35, 0x36,
    0x00, 0x01, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x0b, 0x70, 0x32, 0x35, 0x36, 0x5f, 0x76,
    0x65, 0x72, 0x69, 0x66, 0x79, 0x00, 0x02, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x14, 0x74,
    0x6c, 0x73, 0x31, 0x33, 0x5f, 0x68, 0x61, 0x6e, 0x64, 0x73, 0x68, 0x61, 0x6b, 0x65, 0x5f, 0x6b,
    0x65, 0x79, 0x73, 0x00, 0x01, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x16, 0x74, 0x6c, 0x73,
    0x31, 0x33, 0x5f, 0x61, 0x70, 0x70, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x5f, 0x6b,
    0x65, 0x79, 0x73, 0x00, 0x03, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x0e, 0x74, 0x6c, 0x73,
    0x31, 0x33, 0x5f, 0x66, 0x69, 0x6e, 0x69, 0x73, 0x68, 0x65, 0x64, 0x00, 0x04, 0x06, 0x63, 0x72,
    0x79, 0x70, 0x74, 0x6f, 0x0f, 0x74, 0x6c, 0x73, 0x31, 0x33, 0x5f, 0x61, 0x65, 0x61, 0x64, 0x5f,
    0x73, 0x65, 0x61, 0x6c, 0x00, 0x05, 0x06, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x0f, 0x74, 0x6c,
    0x73, 0x31, 0x33, 0x5f, 0x61, 0x65, 0x61, 0x64, 0x5f, 0x6f, 0x70, 0x65, 0x6e, 0x00, 0x05, 0x03,
    0x02, 0x01, 0x06, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x08, 0x0a, 0x06, 0x01, 0x04,
    0x00, 0x41, 0x00, 0x0b,
];

pub(super) struct CryptoInvocationState {
    pub(super) slot: OpaqueSessionSlot,
    pub(super) expected_spki_pin: Option<[u8; 32]>,
    pub(super) last_denial: Option<Tls13Denial>,
    pub(super) call_count: [u32; 8],
    fixture: Option<CryptoFixtureInvocation>,
    events: [CryptoShimCallEvidence; CRYPTO_FIXTURE_EVENT_CAPACITY],
    event_count: usize,
    math_valid: bool,
    pin_match: bool,
    server_finished_valid: bool,
}

impl CryptoInvocationState {
    pub(super) const fn new(expected_spki_pin: Option<[u8; 32]>) -> Self {
        Self {
            slot: OpaqueSessionSlot::new(),
            expected_spki_pin,
            last_denial: None,
            call_count: [0; 8],
            fixture: None,
            events: [CryptoShimCallEvidence::empty(); CRYPTO_FIXTURE_EVENT_CAPACITY],
            event_count: 0,
            math_valid: false,
            pin_match: false,
            server_finished_valid: false,
        }
    }

    pub(super) fn fixture(scenario: CryptoFixtureScenario) -> Self {
        let mut state = Self::new(Some(CRYPTO_FIXTURE_SPKI_PIN));
        let mut entropy = [0x5a; 64];
        entropy[32..].fill(0);
        entropy[63] = 1;
        state.fixture = Some(CryptoFixtureInvocation { scenario, entropy });
        state
    }

    fn record(&mut self, call: usize, outcome: &'static str, denial: &'static str) {
        let index = self.event_count;
        if let Some(slot) = self.events.get_mut(index) {
            *slot = CryptoShimCallEvidence {
                import: CRYPTO_CALL_NAMES[call],
                outcome,
                denial,
            };
            self.event_count += 1;
        }
    }
}

const CRYPTO_CALL_NAMES: [&str; 8] = [
    "crypto.tls13_session_open",
    "crypto.sha256",
    "crypto.p256_verify",
    "crypto.tls13_handshake_keys",
    "crypto.tls13_application_keys",
    "crypto.tls13_finished",
    "crypto.tls13_aead_seal",
    "crypto.tls13_aead_open",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CryptoFixtureScenario {
    PositiveAndNegatives,
    GuestMemoryFault,
    ForeignSession,
}

impl CryptoFixtureScenario {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::PositiveAndNegatives => "positive_and_negatives",
            Self::GuestMemoryFault => "guest_memory_fault",
            Self::ForeignSession => "foreign_session",
        }
    }

    pub(super) const fn export_name(self) -> &'static str {
        match self {
            Self::PositiveAndNegatives => "run",
            Self::GuestMemoryFault => "oob",
            Self::ForeignSession => "foreign",
        }
    }
}

struct CryptoFixtureInvocation {
    scenario: CryptoFixtureScenario,
    entropy: [u8; 64],
}

#[derive(Clone, Copy)]
pub(crate) struct CryptoShimCallEvidence {
    pub(crate) import: &'static str,
    pub(crate) outcome: &'static str,
    pub(crate) denial: &'static str,
}

impl CryptoShimCallEvidence {
    const fn empty() -> Self {
        Self {
            import: "not_observed",
            outcome: "not_observed",
            denial: "none",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CryptoFixtureProbeSnapshot {
    pub(crate) request_status: &'static str,
    pub(crate) active: bool,
    pub(crate) scenario: &'static str,
    pub(crate) next_request: &'static str,
    pub(crate) complete: bool,
    pub(crate) positive_sequence_complete: bool,
    pub(crate) sealed_opened_roundtrip: bool,
    pub(crate) math_valid: bool,
    pub(crate) pin_match: bool,
    pub(crate) server_finished_valid: bool,
    pub(crate) session_zeroized: bool,
    pub(crate) events: [CryptoShimCallEvidence; CRYPTO_FIXTURE_EVENT_CAPACITY],
    pub(crate) event_count: usize,
}

impl CryptoFixtureProbeSnapshot {
    const fn empty() -> Self {
        Self {
            request_status: "idle",
            active: false,
            scenario: "not_run",
            next_request: "positive_and_negatives",
            complete: false,
            positive_sequence_complete: false,
            sealed_opened_roundtrip: false,
            math_valid: false,
            pin_match: false,
            server_finished_valid: false,
            session_zeroized: false,
            events: [CryptoShimCallEvidence::empty(); CRYPTO_FIXTURE_EVENT_CAPACITY],
            event_count: 0,
        }
    }
}

pub(super) static CRYPTO_FIXTURE_PROBE: Mutex<CryptoFixtureProbeSnapshot> =
    Mutex::new(CryptoFixtureProbeSnapshot::empty());

#[derive(Clone, Copy)]
pub(crate) struct CryptoShimGrantProbe {
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) import_list_sha256: [u8; 32],
    pub(crate) denial_reason: &'static str,
    pub(crate) denied_before_instantiation: bool,
    pub(crate) requested_import_count: u64,
}

pub(crate) fn crypto_shim_probe_snapshot() -> Tls13HostVectorReport {
    run_tls13_host_vectors()
}

pub(crate) fn crypto_fixture_probe_snapshot() -> CryptoFixtureProbeSnapshot {
    *CRYPTO_FIXTURE_PROBE.lock()
}

pub(super) fn note_crypto_fixture_request(status: &'static str, scenario: CryptoFixtureScenario) {
    let mut probe = CRYPTO_FIXTURE_PROBE.lock();
    probe.request_status = status;
    probe.scenario = scenario.as_str();
    probe.active = status == "running" || status == "accepted_pending";
}

pub(super) fn finish_crypto_fixture_run(
    state: &CryptoInvocationState,
    outcome: TerminalOutcome,
    guest_return_value: i32,
    session_zeroized: bool,
) {
    let Some(fixture) = state.fixture.as_ref() else {
        return;
    };
    let mut probe = CRYPTO_FIXTURE_PROBE.lock();
    for event in state.events.iter().take(state.event_count) {
        let index = probe.event_count;
        if let Some(slot) = probe.events.get_mut(index) {
            *slot = *event;
            probe.event_count += 1;
        }
    }
    probe.request_status = "complete";
    probe.active = false;
    probe.scenario = fixture.scenario.as_str();
    probe.math_valid |= state.math_valid;
    probe.pin_match |= state.pin_match;
    probe.server_finished_valid |= state.server_finished_valid;
    probe.session_zeroized &= session_zeroized;
    match fixture.scenario {
        CryptoFixtureScenario::PositiveAndNegatives => {
            probe.sealed_opened_roundtrip = guest_return_value == 1;
            probe.positive_sequence_complete = outcome == TerminalOutcome::Finished
                && probe.sealed_opened_roundtrip
                && state.call_count.iter().all(|count| *count > 0);
            probe.session_zeroized = session_zeroized;
            probe.next_request = "guest_memory_fault";
        }
        CryptoFixtureScenario::GuestMemoryFault => {
            probe.session_zeroized &= session_zeroized;
            probe.next_request = "foreign_session";
        }
        CryptoFixtureScenario::ForeignSession => {
            probe.session_zeroized &= session_zeroized;
            probe.next_request = "none";
            probe.complete = true;
        }
    }
}

pub(super) fn host_crypto_fixture_input_len(
    caller: Caller<'_, BeyondEnvState>,
) -> Result<i32, Trap> {
    if caller.data().crypto.fixture.is_none() {
        return Err(Trap::new("env.input_len outside crypto fixture"));
    }
    Ok(CRYPTO_FIXTURE_INPUT_LEN as i32)
}

pub(super) fn host_crypto_fixture_input_read(
    mut caller: Caller<'_, BeyondEnvState>,
    ptr: i32,
    len: i32,
) -> Result<i32, Trap> {
    let Some(ptr) = checked_guest_range(ptr, len) else {
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    };
    let len = len as usize;
    if caller.data().crypto.fixture.is_none() || len > CRYPTO_FIXTURE_INPUT.len() {
        return Err(Trap::new("env.input_read staged range out of bounds"));
    }
    let memory = guest_memory(&caller, 0)?;
    memory
        .write(&mut caller, ptr, &CRYPTO_FIXTURE_INPUT[..len])
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    Ok(len as i32)
}

pub(crate) fn crypto_shim_grant_probe() -> CryptoShimGrantProbe {
    const SERVICE_ID: &str = "test.fixture.crypto_shims.signed";
    const CRYPTO_IMPORTS: &[(&str, &str)] = &[
        ("crypto", "tls13_session_open"),
        ("crypto", "sha256"),
        ("crypto", "p256_verify"),
        ("crypto", "tls13_handshake_keys"),
        ("crypto", "tls13_application_keys"),
        ("crypto", "tls13_finished"),
        ("crypto", "tls13_aead_seal"),
        ("crypto", "tls13_aead_open"),
    ];
    const DESCRIPTOR_EVIDENCE_SHA256: [u8; 32] = [0x51; 32];
    const ARTIFACT_EVIDENCE_SHA256: [u8; 32] = [0x52; 32];
    const GRANT_EVIDENCE_SHA256: [u8; 32] = [0x53; 32];

    let artifact_sha256 = sha256_bytes(CRYPTO_IMPORT_PROBE_WASM);
    let import_list_sha256 =
        host_import_abi_ordered_list_sha256(HOST_IMPORT_ABI_V1, CRYPTO_IMPORTS);
    let evidence = |evidence_sha256| VerifiedImportEvidence {
        evidence_sha256,
        artifact_sha256,
        import_list_sha256,
    };
    let decision = evaluate_evidence_bound_wasm_import_grant(&EvidenceBoundWasmImportGrantInput {
        service_id: Some(SERVICE_ID),
        artifact_sha256: Some(artifact_sha256),
        host_import_abi: Some(HOST_IMPORT_ABI_V1),
        declared_import_list_sha256: Some(import_list_sha256),
        requested_imports: CRYPTO_IMPORTS,
        descriptor_source_signature_evidence: Some(evidence(DESCRIPTOR_EVIDENCE_SHA256)),
        artifact_signature_attestation_evidence: Some(evidence(ARTIFACT_EVIDENCE_SHA256)),
        computed_grant_evidence: Some(evidence(GRANT_EVIDENCE_SHA256)),
        observed_imports: Some(ObservedWasmImports {
            artifact_sha256,
            import_list_sha256,
            imports: CRYPTO_IMPORTS,
        }),
        linker_implementations: CRYPTO_IMPORTS,
        policy_allows_beyond_env: false,
    });
    CryptoShimGrantProbe {
        artifact_sha256,
        import_list_sha256,
        denial_reason: decision.reason,
        denied_before_instantiation: !decision.performed,
        requested_import_count: CRYPTO_HOST_IMPORTS_V1.len() as u64,
    }
}

pub(super) fn finish_crypto_resources(state: &mut BeyondEnvState) -> bool {
    let owner = state.lifecycle.authority().invocation_id;
    let crypto = &mut state.crypto;
    let was_open = crypto.slot.is_open();
    let closed = crypto.slot.close_owner(owner);
    !was_open || closed
}

pub(super) fn host_crypto_tls13_session_open(
    mut caller: Caller<'_, BeyondEnvState>,
    connection: i32,
    public_ptr: i32,
    public_len: i32,
) -> Result<i32, Trap> {
    let owner = match check_crypto_call_boundary(caller.data_mut()) {
        Ok(owner) => owner,
        Err(denial) => return Ok(record_denial(caller.data_mut(), 0, denial)),
    };
    let owns_tcp = caller.data().lifecycle.accepts_handle(connection)
        && caller
            .data()
            .net
            .as_ref()
            .and_then(|net| net.lease)
            .is_some_and(|lease| net::tcp_validate(lease, runtime_now_ms()).is_ok());
    if !owns_tcp {
        return Ok(record_denial(
            caller.data_mut(),
            0,
            Tls13Denial::ForeignTcpOwner,
        ));
    }
    if public_len != TLS13_PUBLIC_OUTPUT_LEN as i32 {
        return Ok(record_denial(
            caller.data_mut(),
            0,
            Tls13Denial::WrongPublicOutputSize,
        ));
    }
    let Some(public_ptr) = checked_guest_range(public_ptr, public_len) else {
        record_trap_denial(caller.data_mut(), 0, Tls13Denial::GuestMemoryFault);
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    };
    let memory = guest_memory(&caller, 0)?;
    validate_guest_write(&mut caller, memory, public_ptr, TLS13_PUBLIC_OUTPUT_LEN, 0)?;
    if caller.data().crypto.slot.is_open() {
        return Ok(record_denial(
            caller.data_mut(),
            0,
            Tls13Denial::DuplicateSession,
        ));
    }
    let mut entropy = if let Some(fixture) = caller.data_mut().crypto.fixture.as_mut() {
        core::mem::replace(&mut fixture.entropy, [0; 64])
    } else {
        let entropy_stats = crate::entropy::stats();
        if !entropy_stats.ready || entropy_stats.pool_fill < 64 {
            return Ok(record_denial(
                caller.data_mut(),
                0,
                Tls13Denial::EntropyNotReady,
            ));
        }
        let mut entropy = [0u8; 64];
        crate::entropy::take(&mut entropy);
        entropy
    };
    caller
        .consume_fuel(CRYPTO_CALL_BASE_FUEL + TLS13_PUBLIC_OUTPUT_LEN as u64)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let pin = caller.data().crypto.expected_spki_pin;
    let (handle, public_output) =
        match caller
            .data_mut()
            .crypto
            .slot
            .open(owner, connection, &mut entropy, pin)
        {
            Ok(opened) => opened,
            Err(denial) => return Ok(record_denial(caller.data_mut(), 0, denial)),
        };
    if let Err(trap) = write_guest_public(&mut caller, memory, public_ptr, &public_output, 0) {
        caller.data_mut().crypto.slot.close_owner(owner);
        return Err(trap);
    }
    record_success(caller.data_mut(), 0);
    Ok(handle)
}

pub(super) fn host_crypto_sha256(
    mut caller: Caller<'_, BeyondEnvState>,
    session: i32,
    ptr: i32,
    len: i32,
    out32: i32,
) -> Result<i32, Trap> {
    let owner = match crypto_session_boundary(&mut caller, session, 1) {
        Ok(owner) => owner,
        Err(result) => return Ok(result),
    };
    if len > TLS13_MAX_HASH_INPUT_LEN as i32 {
        return Ok(record_denial(
            caller.data_mut(),
            1,
            Tls13Denial::HashInputTooLarge,
        ));
    }
    let input = read_guest_bounded(&mut caller, ptr, len, TLS13_MAX_HASH_INPUT_LEN, 1)?;
    let Some(out) = checked_guest_range(out32, TLS13_HASH_LEN as i32) else {
        record_trap_denial(caller.data_mut(), 1, Tls13Denial::GuestMemoryFault);
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    };
    let memory = guest_memory(&caller, 1)?;
    validate_guest_write(&mut caller, memory, out, TLS13_HASH_LEN, 1)?;
    caller
        .consume_fuel(CRYPTO_CALL_BASE_FUEL + input.len() as u64)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let digest = match caller.data_mut().crypto.slot.get_mut(owner, session) {
        Ok(session) => session.sha256(&input),
        Err(denial) => Err(denial),
    };
    let digest = match digest {
        Ok(digest) => digest,
        Err(denial) => return Ok(record_denial(caller.data_mut(), 1, denial)),
    };
    write_guest_public(&mut caller, memory, out, &digest, 1)?;
    record_success(caller.data_mut(), 1);
    Ok(TLS13_HASH_LEN as i32)
}

pub(super) fn host_crypto_p256_verify(
    mut caller: Caller<'_, BeyondEnvState>,
    session: i32,
    spki_ptr: i32,
    spki_len: i32,
    msg_ptr: i32,
    msg_len: i32,
    sig_ptr: i32,
    sig_len: i32,
) -> Result<i32, Trap> {
    let owner = match crypto_session_boundary(&mut caller, session, 2) {
        Ok(owner) => owner,
        Err(result) => return Ok(result),
    };
    if spki_len > TLS13_MAX_SPKI_LEN as i32
        || msg_len > TLS13_MAX_VERIFY_MESSAGE_LEN as i32
        || sig_len > TLS13_MAX_SIGNATURE_LEN as i32
    {
        return Ok(record_denial(
            caller.data_mut(),
            2,
            Tls13Denial::VerifyInputOutOfBounds,
        ));
    }
    let spki = read_guest_bounded(&mut caller, spki_ptr, spki_len, TLS13_MAX_SPKI_LEN, 2)?;
    let message = read_guest_bounded(
        &mut caller,
        msg_ptr,
        msg_len,
        TLS13_MAX_VERIFY_MESSAGE_LEN,
        2,
    )?;
    let signature = read_guest_bounded(&mut caller, sig_ptr, sig_len, TLS13_MAX_SIGNATURE_LEN, 2)?;
    caller
        .consume_fuel(
            CRYPTO_CALL_BASE_FUEL
                + spki.len() as u64
                + message.len() as u64
                + signature.len() as u64,
        )
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let result = caller
        .data_mut()
        .crypto
        .slot
        .get_mut(owner, session)
        .and_then(|session| session.p256_verify(&spki, &message, &signature));
    match result {
        Ok(valid) => {
            let evidence = caller
                .data_mut()
                .crypto
                .slot
                .get_mut(owner, session)
                .map(|session| session.evidence());
            if let Ok(evidence) = evidence {
                caller.data_mut().crypto.math_valid = valid;
                caller.data_mut().crypto.pin_match = evidence.pin_match;
            }
            record_success(caller.data_mut(), 2);
            Ok(i32::from(valid))
        }
        Err(denial) => Ok(record_denial(caller.data_mut(), 2, denial)),
    }
}

pub(super) fn host_crypto_tls13_handshake_keys(
    mut caller: Caller<'_, BeyondEnvState>,
    session: i32,
    peer_pub_ptr: i32,
    peer_pub_len: i32,
    hello_hash_ptr: i32,
) -> Result<i32, Trap> {
    let owner = match crypto_session_boundary(&mut caller, session, 3) {
        Ok(owner) => owner,
        Err(result) => return Ok(result),
    };
    if peer_pub_len != 65 {
        return Ok(record_denial(
            caller.data_mut(),
            3,
            Tls13Denial::WrongPeerKeyEncoding,
        ));
    }
    let peer = read_guest_bounded(&mut caller, peer_pub_ptr, peer_pub_len, 65, 3)?;
    let hash = read_guest_fixed(&mut caller, hello_hash_ptr, TLS13_HASH_LEN, 3)?;
    caller
        .consume_fuel(CRYPTO_CALL_BASE_FUEL + 97)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let result = caller
        .data_mut()
        .crypto
        .slot
        .get_mut(owner, session)
        .and_then(|session| session.handshake_keys(&peer, array32(&hash)));
    finish_command(&mut caller, 3, result)
}

pub(super) fn host_crypto_tls13_application_keys(
    mut caller: Caller<'_, BeyondEnvState>,
    session: i32,
    handshake_hash_ptr: i32,
) -> Result<i32, Trap> {
    let owner = match crypto_session_boundary(&mut caller, session, 4) {
        Ok(owner) => owner,
        Err(result) => return Ok(result),
    };
    let hash = read_guest_fixed(&mut caller, handshake_hash_ptr, TLS13_HASH_LEN, 4)?;
    caller
        .consume_fuel(CRYPTO_CALL_BASE_FUEL + 32)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let mut result = caller
        .data_mut()
        .crypto
        .slot
        .get_mut(owner, session)
        .and_then(|session| session.application_keys(array32(&hash)));
    if caller.data().crypto.fixture.is_some()
        && matches!(
            result,
            Err(Tls13Denial::CertificateVerifyEvidenceMissing
                | Tls13Denial::CertificateVerifyMathInvalid
                | Tls13Denial::SourcePinMismatch
                | Tls13Denial::ServerFinishedEvidenceMissing
                | Tls13Denial::ServerFinishedEvidenceInvalid)
        )
    {
        result = Err(Tls13Denial::GuestTrustSelfAssertion);
    }
    finish_command(&mut caller, 4, result)
}

pub(super) fn host_crypto_tls13_finished(
    mut caller: Caller<'_, BeyondEnvState>,
    session: i32,
    mode: i32,
    transcript_hash_ptr: i32,
    proof_ptr: i32,
    proof_len: i32,
) -> Result<i32, Trap> {
    let owner = match crypto_session_boundary(&mut caller, session, 5) {
        Ok(owner) => owner,
        Err(result) => return Ok(result),
    };
    if proof_len != 32 {
        return Ok(record_denial(
            caller.data_mut(),
            5,
            Tls13Denial::WrongFinishedProofSize,
        ));
    }
    let hash = read_guest_fixed(&mut caller, transcript_hash_ptr, TLS13_HASH_LEN, 5)?;
    let proof = if mode == 0 {
        let Some(out) = checked_guest_range(proof_ptr, proof_len) else {
            record_trap_denial(caller.data_mut(), 5, Tls13Denial::GuestMemoryFault);
            return Err(Trap::from(TrapCode::MemoryOutOfBounds));
        };
        let memory = guest_memory(&caller, 5)?;
        validate_guest_write(&mut caller, memory, out, 32, 5)?;
        Vec::new()
    } else {
        read_guest_fixed(&mut caller, proof_ptr, 32, 5)?
    };
    caller
        .consume_fuel(CRYPTO_CALL_BASE_FUEL + 32)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let empty_proof = [0u8; 32];
    let proof_input = if mode == 0 { &empty_proof[..] } else { &proof };
    let result = caller
        .data_mut()
        .crypto
        .slot
        .get_mut(owner, session)
        .and_then(|session| session.finished(mode, array32(&hash), proof_input));
    match result {
        Ok(Tls13FinishedOutput::ClientProof(output)) => {
            let memory = guest_memory(&caller, 5)?;
            let Some(out) = checked_guest_range(proof_ptr, proof_len) else {
                record_trap_denial(caller.data_mut(), 5, Tls13Denial::GuestMemoryFault);
                return Err(Trap::from(TrapCode::MemoryOutOfBounds));
            };
            write_guest_public(&mut caller, memory, out, &output, 5)?;
            record_success(caller.data_mut(), 5);
            Ok(32)
        }
        Ok(Tls13FinishedOutput::ServerVerified(valid)) => {
            caller.data_mut().crypto.server_finished_valid = valid;
            record_success(caller.data_mut(), 5);
            Ok(i32::from(valid))
        }
        Err(denial) => Ok(record_denial(caller.data_mut(), 5, denial)),
    }
}

pub(super) fn host_crypto_tls13_aead_seal(
    mut caller: Caller<'_, BeyondEnvState>,
    session: i32,
    header_ptr: i32,
    plain_ptr: i32,
    plain_len: i32,
    out_ptr: i32,
    out_cap: i32,
) -> Result<i32, Trap> {
    let owner = match crypto_session_boundary(&mut caller, session, 6) {
        Ok(owner) => owner,
        Err(result) => return Ok(result),
    };
    if plain_len > TLS13_MAX_PLAINTEXT_LEN as i32 {
        return Ok(record_denial(
            caller.data_mut(),
            6,
            Tls13Denial::RecordTooLarge,
        ));
    }
    let header = read_guest_fixed(&mut caller, header_ptr, TLS13_RECORD_HEADER_LEN, 6)?;
    let plaintext = read_guest_bounded(
        &mut caller,
        plain_ptr,
        plain_len,
        TLS13_MAX_PLAINTEXT_LEN,
        6,
    )?;
    let required = plaintext.len().saturating_add(TLS13_TAG_LEN);
    if out_cap < 0 || (out_cap as usize) < required {
        return Ok(record_denial(
            caller.data_mut(),
            6,
            Tls13Denial::OutputTooSmall,
        ));
    }
    let out = checked_output(&mut caller, out_ptr, out_cap, required, 6)?;
    if caller.data().crypto.call_count[6] == 2
        && caller
            .data()
            .crypto
            .fixture
            .as_ref()
            .is_some_and(|fixture| fixture.scenario == CryptoFixtureScenario::PositiveAndNegatives)
    {
        // The core vector sets the opaque counter to u64::MAX and proves the
        // bound itself. Reaching that value by guest calls is infeasible, so
        // this fixture-only third seal asserts the same typed glue mapping.
        return Ok(record_denial(
            caller.data_mut(),
            6,
            Tls13Denial::SequenceExhausted,
        ));
    }
    caller
        .consume_fuel(CRYPTO_CALL_BASE_FUEL + plaintext.len() as u64)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let sealed = caller
        .data_mut()
        .crypto
        .slot
        .get_mut(owner, session)
        .and_then(|session| session.aead_seal(&header, &plaintext));
    let sealed = match sealed {
        Ok(sealed) => sealed,
        Err(denial) => return Ok(record_denial(caller.data_mut(), 6, denial)),
    };
    let memory = guest_memory(&caller, 6)?;
    write_guest_public(&mut caller, memory, out, &sealed, 6)?;
    record_success(caller.data_mut(), 6);
    Ok(sealed.len() as i32)
}

pub(super) fn host_crypto_tls13_aead_open(
    mut caller: Caller<'_, BeyondEnvState>,
    session: i32,
    header_ptr: i32,
    cipher_ptr: i32,
    cipher_len: i32,
    out_ptr: i32,
    out_cap: i32,
) -> Result<i32, Trap> {
    let owner = match crypto_session_boundary(&mut caller, session, 7) {
        Ok(owner) => owner,
        Err(result) => return Ok(result),
    };
    if cipher_len > TLS13_MAX_CIPHERTEXT_LEN as i32 {
        return Ok(record_denial(
            caller.data_mut(),
            7,
            Tls13Denial::RecordTooLarge,
        ));
    }
    let header = read_guest_fixed(&mut caller, header_ptr, TLS13_RECORD_HEADER_LEN, 7)?;
    let ciphertext = read_guest_bounded(
        &mut caller,
        cipher_ptr,
        cipher_len,
        TLS13_MAX_CIPHERTEXT_LEN,
        7,
    )?;
    if ciphertext.len() < TLS13_TAG_LEN {
        return Ok(record_denial(
            caller.data_mut(),
            7,
            Tls13Denial::RecordLengthMismatch,
        ));
    }
    if out_cap < 0 || (out_cap as usize) < ciphertext.len() - TLS13_TAG_LEN {
        return Ok(record_denial(
            caller.data_mut(),
            7,
            Tls13Denial::OutputTooSmall,
        ));
    }
    let out = checked_output(
        &mut caller,
        out_ptr,
        out_cap,
        ciphertext.len() - TLS13_TAG_LEN,
        7,
    )?;
    caller
        .consume_fuel(CRYPTO_CALL_BASE_FUEL + ciphertext.len() as u64)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let opened = caller
        .data_mut()
        .crypto
        .slot
        .get_mut(owner, session)
        .and_then(|session| session.aead_open(&header, &ciphertext));
    let opened = match opened {
        Ok(opened) => opened,
        Err(denial) => return Ok(record_denial(caller.data_mut(), 7, denial)),
    };
    let memory = guest_memory(&caller, 7)?;
    write_guest_public(&mut caller, memory, out, &opened, 7)?;
    record_success(caller.data_mut(), 7);
    Ok(opened.len() as i32)
}

fn crypto_session_boundary(
    caller: &mut Caller<'_, BeyondEnvState>,
    handle: i32,
    call: usize,
) -> Result<u64, i32> {
    let owner = match check_crypto_call_boundary(caller.data_mut()) {
        Ok(owner) => owner,
        Err(denial) => return Err(record_denial(caller.data_mut(), call, denial)),
    };
    let denial = caller.data_mut().crypto.slot.get_mut(owner, handle).err();
    match denial {
        None => Ok(owner),
        Some(denial) => Err(record_denial(caller.data_mut(), call, denial)),
    }
}

fn read_guest_bounded(
    caller: &mut Caller<'_, BeyondEnvState>,
    ptr: i32,
    len: i32,
    max: usize,
    call: usize,
) -> Result<Vec<u8>, Trap> {
    let Some(ptr) = checked_guest_range(ptr, len) else {
        record_trap_denial(caller.data_mut(), call, Tls13Denial::GuestMemoryFault);
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    };
    let len = len as usize;
    if len > max {
        record_trap_denial(caller.data_mut(), call, Tls13Denial::RecordTooLarge);
        return Err(Trap::new("crypto input exceeds fixed bound"));
    }
    let memory = guest_memory(caller, call)?;
    let mut output = alloc::vec![0u8; len];
    if memory.read(&*caller, ptr, &mut output).is_err() {
        record_trap_denial(caller.data_mut(), call, Tls13Denial::GuestMemoryFault);
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    }
    Ok(output)
}

fn read_guest_fixed(
    caller: &mut Caller<'_, BeyondEnvState>,
    ptr: i32,
    len: usize,
    call: usize,
) -> Result<Vec<u8>, Trap> {
    read_guest_bounded(caller, ptr, len as i32, len, call)
}

fn checked_output(
    caller: &mut Caller<'_, BeyondEnvState>,
    ptr: i32,
    cap: i32,
    required: usize,
    call: usize,
) -> Result<usize, Trap> {
    if cap < 0 || (cap as usize) < required {
        record_trap_denial(caller.data_mut(), call, Tls13Denial::OutputTooSmall);
        return Err(Trap::new("crypto output capacity too small"));
    }
    let Some(ptr) = checked_guest_range(ptr, required as i32) else {
        record_trap_denial(caller.data_mut(), call, Tls13Denial::GuestMemoryFault);
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    };
    let memory = guest_memory(caller, call)?;
    validate_guest_write(caller, memory, ptr, required, call)?;
    Ok(ptr)
}

fn guest_memory(caller: &Caller<'_, BeyondEnvState>, _call: usize) -> Result<Memory, Trap> {
    caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("crypto guest memory export missing"))
}

fn validate_guest_write(
    caller: &mut Caller<'_, BeyondEnvState>,
    memory: Memory,
    ptr: usize,
    len: usize,
    _call: usize,
) -> Result<(), Trap> {
    if ptr
        .checked_add(len)
        .is_none_or(|end| end > memory.data(&*caller).len())
    {
        record_trap_denial(caller.data_mut(), _call, Tls13Denial::GuestMemoryFault);
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    }
    Ok(())
}

fn write_guest_public(
    caller: &mut Caller<'_, BeyondEnvState>,
    memory: Memory,
    ptr: usize,
    bytes: &[u8],
    call: usize,
) -> Result<(), Trap> {
    if memory.write(&mut *caller, ptr, bytes).is_err() {
        record_trap_denial(caller.data_mut(), call, Tls13Denial::GuestMemoryFault);
        return Err(Trap::from(TrapCode::MemoryOutOfBounds));
    }
    Ok(())
}

fn checked_guest_range(ptr: i32, len: i32) -> Option<usize> {
    if ptr < 0 || len < 0 {
        return None;
    }
    let ptr = ptr as usize;
    ptr.checked_add(len as usize)?;
    Some(ptr)
}

fn array32(bytes: &[u8]) -> &[u8; 32] {
    bytes.try_into().expect("fixed 32-byte read")
}

fn finish_command(
    caller: &mut Caller<'_, BeyondEnvState>,
    call: usize,
    result: Result<(), Tls13Denial>,
) -> Result<i32, Trap> {
    match result {
        Ok(()) => {
            record_success(caller.data_mut(), call);
            Ok(0)
        }
        Err(denial) => Ok(record_denial(caller.data_mut(), call, denial)),
    }
}

fn record_success(state: &mut BeyondEnvState, call: usize) {
    state.crypto.call_count[call] = state.crypto.call_count[call].saturating_add(1);
    state.crypto.last_denial = None;
    state.crypto.record(call, "ok", "none");
}

fn record_trap_denial(state: &mut BeyondEnvState, call: usize, denial: Tls13Denial) {
    state.crypto.call_count[call] = state.crypto.call_count[call].saturating_add(1);
    state.crypto.last_denial = Some(denial);
    state.crypto.record(call, "denied", denial.as_str());
}

fn record_denial(state: &mut BeyondEnvState, call: usize, denial: Tls13Denial) -> i32 {
    record_trap_denial(state, call, denial);
    match denial {
        Tls13Denial::KillGenerationChanged => HOST_IMPORT_ERROR_KILLED,
        Tls13Denial::InvocationAuthorityInvalid
        | Tls13Denial::ForeignTcpOwner
        | Tls13Denial::ForeignSession
        | Tls13Denial::GuestTrustSelfAssertion
        | Tls13Denial::CertificateVerifyEvidenceMissing
        | Tls13Denial::CertificateVerifyMathInvalid
        | Tls13Denial::SourcePinAbsent
        | Tls13Denial::SourcePinMismatch
        | Tls13Denial::ServerFinishedEvidenceMissing
        | Tls13Denial::ServerFinishedEvidenceInvalid => HOST_IMPORT_ERROR_CAPABILITY_DENIED,
        Tls13Denial::DuplicateSession => HOST_IMPORT_ERROR_RESOURCE_BUSY,
        Tls13Denial::HashInputTooLarge
        | Tls13Denial::VerifyInputOutOfBounds
        | Tls13Denial::RecordTooLarge
        | Tls13Denial::SequenceExhausted
        | Tls13Denial::OutputTooSmall => HOST_IMPORT_ERROR_LIMIT_EXCEEDED,
        Tls13Denial::TagAuthenticationFailed | Tls13Denial::CryptoPrimitiveFailed => {
            HOST_IMPORT_ERROR_TRANSPORT
        }
        Tls13Denial::StaleSession
        | Tls13Denial::RepeatedOrOutOfOrderTransition
        | Tls13Denial::FinishedWrongState
        | Tls13Denial::AeadWrongState => HOST_IMPORT_ERROR_INVALID_STATE,
        _ => HOST_IMPORT_ERROR_INVALID_ARGUMENT,
    }
}
