#![cfg_attr(not(test), no_std)]

use sha2::{Digest, Sha256};

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
    use super::{sha256_bytes, sha256_hex, ByteSink};

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
}
