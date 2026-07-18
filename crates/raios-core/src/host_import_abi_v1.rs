//! Declared host-import ABI for evidence-bound Wasm grants.
//!
//! Declarations and hashes in this module grant nothing. Linker implementations
//! and policy authorization remain separate requirements.

use alloc::vec;

use crate::record::{sha256_of_json, Field, Value};

pub const HOST_IMPORT_ABI_V1: &str = "raios.host_imports.v1";

pub const HOST_IMPORT_ERROR_INVALID_ARGUMENT: i32 = -1;
pub const HOST_IMPORT_ERROR_CAPABILITY_DENIED: i32 = -2;
pub const HOST_IMPORT_ERROR_RESOURCE_BUSY: i32 = -3;
pub const HOST_IMPORT_ERROR_TIMED_OUT: i32 = -4;
pub const HOST_IMPORT_ERROR_CLOSED: i32 = -5;
pub const HOST_IMPORT_ERROR_WOULD_BLOCK: i32 = -6;
pub const HOST_IMPORT_ERROR_LIMIT_EXCEEDED: i32 = -7;
pub const HOST_IMPORT_ERROR_INVALID_STATE: i32 = -8;
pub const HOST_IMPORT_ERROR_TRANSPORT: i32 = -9;
pub const HOST_IMPORT_ERROR_KILLED: i32 = -10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmValueType {
    I32,
    I64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostImportSignature {
    pub module: &'static str,
    pub name: &'static str,
    pub parameters: &'static [WasmValueType],
    pub result: WasmValueType,
}

impl HostImportSignature {
    pub const fn pair(self) -> (&'static str, &'static str) {
        (self.module, self.name)
    }
}

use WasmValueType::{I32, I64};

const P0: &[WasmValueType] = &[];
const P1: &[WasmValueType] = &[I32];
const P2: &[WasmValueType] = &[I32, I32];
const P3: &[WasmValueType] = &[I32, I32, I32];
const P4: &[WasmValueType] = &[I32, I32, I32, I32];
const P5: &[WasmValueType] = &[I32, I32, I32, I32, I32];
const P6: &[WasmValueType] = &[I32, I32, I32, I32, I32, I32];
const P7: &[WasmValueType] = &[I32, I32, I32, I32, I32, I32, I32];

pub const NET_TCP_OPEN: HostImportSignature = HostImportSignature {
    module: "net",
    name: "tcp_open",
    parameters: P0,
    result: I32,
};
pub const NET_TCP_SEND: HostImportSignature = HostImportSignature {
    module: "net",
    name: "tcp_send",
    parameters: P3,
    result: I32,
};
pub const NET_TCP_RECV: HostImportSignature = HostImportSignature {
    module: "net",
    name: "tcp_recv",
    parameters: P3,
    result: I32,
};
pub const NET_TCP_CLOSE: HostImportSignature = HostImportSignature {
    module: "net",
    name: "tcp_close",
    parameters: P1,
    result: I32,
};
pub const NET_HOST_IMPORTS_V1: &[HostImportSignature] =
    &[NET_TCP_OPEN, NET_TCP_SEND, NET_TCP_RECV, NET_TCP_CLOSE];

pub const CRYPTO_TLS13_SESSION_OPEN: HostImportSignature = HostImportSignature {
    module: "crypto",
    name: "tls13_session_open",
    parameters: P3,
    result: I32,
};
pub const CRYPTO_SHA256: HostImportSignature = HostImportSignature {
    module: "crypto",
    name: "sha256",
    parameters: P4,
    result: I32,
};
pub const CRYPTO_P256_VERIFY: HostImportSignature = HostImportSignature {
    module: "crypto",
    name: "p256_verify",
    parameters: P7,
    result: I32,
};
pub const CRYPTO_TLS13_HANDSHAKE_KEYS: HostImportSignature = HostImportSignature {
    module: "crypto",
    name: "tls13_handshake_keys",
    parameters: P4,
    result: I32,
};
pub const CRYPTO_TLS13_APPLICATION_KEYS: HostImportSignature = HostImportSignature {
    module: "crypto",
    name: "tls13_application_keys",
    parameters: P2,
    result: I32,
};
pub const CRYPTO_TLS13_FINISHED: HostImportSignature = HostImportSignature {
    module: "crypto",
    name: "tls13_finished",
    parameters: P5,
    result: I32,
};
pub const CRYPTO_TLS13_AEAD_SEAL: HostImportSignature = HostImportSignature {
    module: "crypto",
    name: "tls13_aead_seal",
    parameters: P6,
    result: I32,
};
pub const CRYPTO_TLS13_AEAD_OPEN: HostImportSignature = HostImportSignature {
    module: "crypto",
    name: "tls13_aead_open",
    parameters: P6,
    result: I32,
};
pub const CRYPTO_HOST_IMPORTS_V1: &[HostImportSignature] = &[
    CRYPTO_TLS13_SESSION_OPEN,
    CRYPTO_SHA256,
    CRYPTO_P256_VERIFY,
    CRYPTO_TLS13_HANDSHAKE_KEYS,
    CRYPTO_TLS13_APPLICATION_KEYS,
    CRYPTO_TLS13_FINISHED,
    CRYPTO_TLS13_AEAD_SEAL,
    CRYPTO_TLS13_AEAD_OPEN,
];

pub const TIME_MONOTONIC_MS: HostImportSignature = HostImportSignature {
    module: "time",
    name: "monotonic_ms",
    parameters: P0,
    result: I64,
};
pub const TIME_HOST_IMPORTS_V1: &[HostImportSignature] = &[TIME_MONOTONIC_MS];

/// Declared for the future provider-only Broker path, never for W7.
pub const SECRET_LEASE_OPENAI_AUTHORIZATION_SEND: HostImportSignature = HostImportSignature {
    module: "secret_lease",
    name: "openai_authorization_send",
    parameters: P1,
    result: I32,
};
pub const SECRET_LEASE_HOST_IMPORTS_V1: &[HostImportSignature] =
    &[SECRET_LEASE_OPENAI_AUTHORIZATION_SEND];

pub const ACQUIRE_CHUNK_ACCEPT: HostImportSignature = HostImportSignature {
    module: "acquire",
    name: "chunk_accept",
    parameters: P3,
    result: I32,
};
pub const ACQUIRE_FINALIZE: HostImportSignature = HostImportSignature {
    module: "acquire",
    name: "finalize",
    parameters: P0,
    result: I32,
};
pub const ACQUIRE_HOST_IMPORTS_V1: &[HostImportSignature] =
    &[ACQUIRE_CHUNK_ACCEPT, ACQUIRE_FINALIZE];

pub const BEYOND_ENV_HOST_IMPORTS_V1: &[HostImportSignature] = &[
    NET_TCP_OPEN,
    NET_TCP_SEND,
    NET_TCP_RECV,
    NET_TCP_CLOSE,
    CRYPTO_TLS13_SESSION_OPEN,
    CRYPTO_SHA256,
    CRYPTO_P256_VERIFY,
    CRYPTO_TLS13_HANDSHAKE_KEYS,
    CRYPTO_TLS13_APPLICATION_KEYS,
    CRYPTO_TLS13_FINISHED,
    CRYPTO_TLS13_AEAD_SEAL,
    CRYPTO_TLS13_AEAD_OPEN,
    TIME_MONOTONIC_MS,
    SECRET_LEASE_OPENAI_AUTHORIZATION_SEND,
    ACQUIRE_CHUNK_ACCEPT,
    ACQUIRE_FINALIZE,
];

/// Hashes the ABI id and exact ordered import-pair list.
///
/// The digest is evidence binding only and grants no import authority.
pub fn host_import_abi_ordered_list_sha256(
    abi_id: &str,
    ordered_imports: &[(&str, &str)],
) -> [u8; 32] {
    let imports = ordered_imports
        .iter()
        .map(|(module, name)| {
            Value::Object(vec![
                Field::new("module", Value::Str(module)),
                Field::new("name", Value::Str(name)),
            ])
        })
        .collect();
    sha256_of_json(&Value::Object(vec![
        Field::new("host_import_abi", Value::Str(abi_id)),
        Field::new("imports", Value::Array(imports)),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_declares_the_exact_ordered_family_surfaces_and_signatures() {
        let expected = [
            ("net", "tcp_open", 0, I32),
            ("net", "tcp_send", 3, I32),
            ("net", "tcp_recv", 3, I32),
            ("net", "tcp_close", 1, I32),
            ("crypto", "tls13_session_open", 3, I32),
            ("crypto", "sha256", 4, I32),
            ("crypto", "p256_verify", 7, I32),
            ("crypto", "tls13_handshake_keys", 4, I32),
            ("crypto", "tls13_application_keys", 2, I32),
            ("crypto", "tls13_finished", 5, I32),
            ("crypto", "tls13_aead_seal", 6, I32),
            ("crypto", "tls13_aead_open", 6, I32),
            ("time", "monotonic_ms", 0, I64),
            ("secret_lease", "openai_authorization_send", 1, I32),
            ("acquire", "chunk_accept", 3, I32),
            ("acquire", "finalize", 0, I32),
        ];

        assert_eq!(HOST_IMPORT_ABI_V1, "raios.host_imports.v1");
        assert_eq!(
            [
                HOST_IMPORT_ERROR_INVALID_ARGUMENT,
                HOST_IMPORT_ERROR_CAPABILITY_DENIED,
                HOST_IMPORT_ERROR_RESOURCE_BUSY,
                HOST_IMPORT_ERROR_TIMED_OUT,
                HOST_IMPORT_ERROR_CLOSED,
                HOST_IMPORT_ERROR_WOULD_BLOCK,
                HOST_IMPORT_ERROR_LIMIT_EXCEEDED,
                HOST_IMPORT_ERROR_INVALID_STATE,
                HOST_IMPORT_ERROR_TRANSPORT,
                HOST_IMPORT_ERROR_KILLED,
            ],
            [-1, -2, -3, -4, -5, -6, -7, -8, -9, -10]
        );
        assert_eq!(NET_HOST_IMPORTS_V1.len(), 4);
        assert_eq!(CRYPTO_HOST_IMPORTS_V1.len(), 8);
        assert_eq!(TIME_HOST_IMPORTS_V1.len(), 1);
        assert_eq!(SECRET_LEASE_HOST_IMPORTS_V1.len(), 1);
        assert_eq!(ACQUIRE_HOST_IMPORTS_V1.len(), 2);
        assert_eq!(BEYOND_ENV_HOST_IMPORTS_V1.len(), expected.len());
        for (import, (module, name, parameter_count, result)) in
            BEYOND_ENV_HOST_IMPORTS_V1.iter().zip(expected)
        {
            assert_eq!(import.module, module);
            assert_eq!(import.name, name);
            assert_eq!(import.parameters.len(), parameter_count);
            assert!(import.parameters.iter().all(|parameter| *parameter == I32));
            assert_eq!(import.result, result);
        }
    }

    #[test]
    fn abi_ordered_list_hash_is_deterministic_order_and_abi_sensitive() {
        let imports = &[("env", "input_len"), ("net", "tcp_open")];
        let first = host_import_abi_ordered_list_sha256(HOST_IMPORT_ABI_V1, imports);
        let same = host_import_abi_ordered_list_sha256(HOST_IMPORT_ABI_V1, imports);
        let reordered = host_import_abi_ordered_list_sha256(
            HOST_IMPORT_ABI_V1,
            &[("net", "tcp_open"), ("env", "input_len")],
        );
        let other_abi = host_import_abi_ordered_list_sha256("raios.host_imports.v2", imports);

        assert_eq!(first, same);
        assert_ne!(first, reordered);
        assert_ne!(first, other_abi);
    }
}
