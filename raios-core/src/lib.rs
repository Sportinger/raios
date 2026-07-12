#![cfg_attr(not(test), no_std)]

extern crate alloc;

use sha2::{Digest, Sha256};

pub mod artifact_blob_frame;
pub mod boot_control;
pub mod core_policy;
pub use raios_http_parse as http_response_parse;
pub use raios_x509_spki as x509_spki;
pub use raios_x509_time as cert_validity_window;
pub mod distribution_provenance;
pub mod distribution_registry;
pub mod dot11_scan;
pub mod driver_capability;
pub mod durable_record_frame;
pub mod genesis_layout;
pub mod gpt_layout;
pub mod kernel_surface;
pub mod marvell_wifi_cmd;
pub mod marvell_wifi_fw;
pub mod marvell_wifi_supplicant;
pub mod memory_record;
pub mod memory_record_resolve;
pub mod owner_key_tpm2;
pub mod personal_shell_abi;
pub mod project_dependency;
pub mod project_overlay;
pub mod project_workspace;
pub mod promotion_attestation;
pub mod provider_trust_descriptor;
pub mod record;
pub mod recovery_lifeline_table;
pub mod recovery_load_record;
pub mod repromotion_reverify;
pub mod scoped_artifact_persist_append;
pub mod scoped_artifact_store_blob;
pub mod scoped_boot_control_replace;
pub mod scoped_distribution_provenance_honesty;
pub mod scoped_external_acquisition_honesty;
pub mod scoped_memory_record_append;
pub mod scoped_promotion_transaction_append;
pub mod scoped_provider_export;
pub mod scoped_provider_trust_honesty;
pub mod scoped_recovery_action_append;
pub mod scoped_recovery_load_append;
pub mod scoped_repromotion_append;
pub mod scoped_rollback_apply;
pub mod scoped_secret_use;
pub mod scoped_seed_data_append;
pub mod scoped_time_authority_honesty;
pub mod scoped_wasm_import_grant;
pub mod secret_vault;
pub mod seed_data_layout;
pub mod structured_store;
pub mod structured_store_partition;
pub mod tpm2_commands;
pub mod ui_frame;
pub mod ui_program;
pub mod ui_program_spec;

/// Computes the SHA-256 digest for `bytes` and returns the raw 32-byte digest.
pub fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// Encodes a SHA-256 digest as 64 lowercase ASCII hexadecimal bytes.
pub fn sha256_hex(digest: &[u8; 32]) -> [u8; 64] {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut out = [0u8; 64];
    let mut idx = 0usize;
    while idx < digest.len() {
        let byte = digest[idx];
        out[idx * 2] = HEX[(byte >> 4) as usize];
        out[idx * 2 + 1] = HEX[(byte & 0x0f) as usize];
        idx += 1;
    }
    out
}

/// Compares two protocol method names case-insensitively.
pub fn method_eq(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

/// Returns true when `left` starts with `right` as a full method token.
///
/// `left` is trimmed before comparison; `right` is compared exactly.
pub fn method_head_eq(left: &str, right: &str) -> bool {
    let left = left.trim();
    if left.len() < right.len() {
        return false;
    }
    let (head, rest) = left.split_at(right.len());
    method_eq(head, right) && (rest.is_empty() || rest.as_bytes()[0].is_ascii_whitespace())
}

/// Parses a SHA-256 reference with an optional case-insensitive `sha256:` prefix.
pub fn parse_sha256_ref(value: &str) -> Option<[u8; 32]> {
    let mut value = value.trim();
    if value.len() >= 7 && value[..7].eq_ignore_ascii_case("sha256:") {
        value = &value[7..];
    }
    if value.len() != 64 {
        return None;
    }
    let bytes = value.as_bytes();
    let mut out = [0u8; 32];
    let mut idx = 0usize;
    while idx < out.len() {
        let high = hex_value(bytes[idx * 2])?;
        let low = hex_value(bytes[idx * 2 + 1])?;
        out[idx] = (high << 4) | low;
        idx += 1;
    }
    Some(out)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

/// Parses the numeric sequence from `event.current_boot.NNNNNNNN`.
pub fn parse_current_boot_event_sequence(value: &str) -> Option<u64> {
    let sequence = value.strip_prefix("event.current_boot.")?;
    if sequence.len() != 8 || !sequence.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let mut parsed = 0u64;
    for byte in sequence.bytes() {
        parsed = parsed
            .saturating_mul(10)
            .saturating_add((byte - b'0') as u64);
    }
    Some(parsed)
}

/// Receives bytes without requiring allocation in kernel callers.
pub trait ByteSink {
    fn write_bytes(&mut self, bytes: &[u8]);
}

#[cfg(test)]
impl ByteSink for std::vec::Vec<u8> {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        method_eq, method_head_eq, parse_current_boot_event_sequence, parse_sha256_ref,
        sha256_bytes, sha256_hex, ByteSink,
    };

    const SHA256_HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const SHA256_HEX_UPPER: &str =
        "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF";
    const SHA256_HEX_NON_HEX: &str =
        "g123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const SHA256_HEX_MIXED_PREFIX: &str =
        "ShA256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const SHA256_HEX_WITH_WHITESPACE: &str =
        "\t0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n";

    fn assert_sha256(input: &[u8], expected_hex: &[u8; 64]) {
        assert_eq!(&sha256_hex(&sha256_bytes(input)), expected_hex);
    }

    fn decode_hex(hex: &[u8; 64]) -> [u8; 32] {
        let mut out = [0u8; 32];
        let mut idx = 0usize;
        while idx < out.len() {
            out[idx] = (hex_nibble(hex[idx * 2]) << 4) | hex_nibble(hex[idx * 2 + 1]);
            idx += 1;
        }
        out
    }

    fn hex_nibble(byte: u8) -> u8 {
        match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            _ => panic!("test vector must be lowercase hex"),
        }
    }

    #[test]
    fn sha256_official_vectors() {
        assert_sha256(
            b"",
            b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
        assert_sha256(
            b"abc",
            b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        );
        assert_sha256(
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            b"248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        );
    }

    #[test]
    fn sha256_hex_round_trips() {
        let digest = sha256_bytes(b"raios-core");
        let hex = sha256_hex(&digest);
        assert_eq!(decode_hex(&hex), digest);
    }

    #[test]
    fn vec_byte_sink_collects_bytes() {
        let mut sink = std::vec::Vec::new();
        sink.write_bytes(b"rai");
        sink.write_bytes(b"OS");
        assert_eq!(sink.as_slice(), b"raiOS");
    }

    #[test]
    fn method_eq_matches_ascii_case_only() {
        assert!(method_eq("system.snapshot", "system.snapshot"));
        assert!(method_eq("System.Snapshot", "system.snapshot"));
        assert!(!method_eq("system.snapshot", "system.status"));
    }

    #[test]
    fn method_head_eq_requires_a_method_boundary() {
        assert!(method_head_eq("system.snapshot", "system.snapshot"));
        assert!(method_head_eq(
            "system.snapshot provider_minimal",
            "system.snapshot"
        ));
        assert!(!method_head_eq("system.snapshotx", "system.snapshot"));
        assert!(!method_head_eq("system.snap", "system.snapshot"));
        assert!(method_head_eq("  system.snapshot", "system.snapshot"));
        assert!(method_head_eq("system.snapshot  ", "system.snapshot"));
        assert!(!method_head_eq("system.snapshot", " system.snapshot"));
    }

    #[test]
    fn parse_sha256_ref_accepts_supported_forms() {
        let expected = parse_sha256_ref(SHA256_HEX).expect("bare hex parses");

        assert_eq!(expected[0], 0x01);
        assert_eq!(parse_sha256_ref(SHA256_HEX_MIXED_PREFIX), Some(expected));
        assert_eq!(parse_sha256_ref(SHA256_HEX_UPPER), Some(expected));
        assert_eq!(parse_sha256_ref(SHA256_HEX_WITH_WHITESPACE), Some(expected));
    }

    #[test]
    fn parse_sha256_ref_rejects_wrong_length_and_non_hex() {
        assert_eq!(parse_sha256_ref(&SHA256_HEX[..63]), None);
        assert_eq!(parse_sha256_ref(SHA256_HEX_NON_HEX), None);
    }

    #[test]
    fn parse_current_boot_event_sequence_accepts_exact_eight_digits() {
        assert_eq!(
            parse_current_boot_event_sequence("event.current_boot.00000042"),
            Some(42)
        );
        assert_eq!(
            parse_current_boot_event_sequence("event.current_boot.99999999"),
            Some(99_999_999)
        );
    }

    #[test]
    fn parse_current_boot_event_sequence_rejects_invalid_ids() {
        assert_eq!(
            parse_current_boot_event_sequence("event.previous_boot.00000042"),
            None
        );
        assert_eq!(
            parse_current_boot_event_sequence("event.current_boot.0000042"),
            None
        );
        assert_eq!(
            parse_current_boot_event_sequence("event.current_boot.000000042"),
            None
        );
        assert_eq!(
            parse_current_boot_event_sequence("event.current_boot.00000a42"),
            None
        );
    }
}
