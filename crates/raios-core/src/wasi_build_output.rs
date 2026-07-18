//! Typed RAM-resident output of one WASI build run.

pub use crate::buildfs_manifest::{BUILD_FS_CHUNK_SIZE, BUILD_FS_MANIFEST_V1};
use crate::parse_sha256_ref;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasiBuildOutputChunk {
    pub len: u64,
    pub sha256: [u8; 32],
}

/// One completed run. `chunks` are in canonical manifest/file/chunk order and
/// remain RAM-resident until the scoped double-build decision finishes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasiBuildOutputRun<'a> {
    pub exit_status: i32,
    pub output_manifest_sha256: &'a str,
    pub chunks: &'a [WasiBuildOutputChunk],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasiBuildOutputHashField {
    RunOneOutputManifestSha256,
    RunTwoOutputManifestSha256,
}

pub(crate) fn parse_output_manifest_sha256(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    parse_sha256_ref(value)
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec};

    use crate::buildfs_manifest::{BuildFsChunk, BuildFsDirectory, BuildFsFile, BuildFsManifest};

    use super::*;

    #[test]
    fn exact_hash_parser_rejects_prefix_whitespace_and_bad_hex() {
        let valid = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert!(parse_output_manifest_sha256(valid).is_some());
        assert!(parse_output_manifest_sha256(&format!("sha256:{valid}")).is_none());
        assert!(parse_output_manifest_sha256(&format!(" {valid}")).is_none());
        assert!(parse_output_manifest_sha256(
            "g123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        )
        .is_none());
        assert_eq!(BUILD_FS_CHUNK_SIZE, 65_536);
        assert_eq!(BUILD_FS_MANIFEST_V1, "raios.buildfs_manifest.v1");
    }

    #[test]
    fn canonical_fixture_pins_the_preview1_output_freeze_golden() {
        let abc = crate::sha256_bytes(b"abc");
        let manifest = BuildFsManifest::new(
            vec![BuildFsDirectory {
                path: "dir".to_string(),
            }],
            vec![BuildFsFile {
                path: "dir/a".to_string(),
                len: 3,
                sha256: abc,
                chunks: vec![BuildFsChunk {
                    len: 3,
                    sha256: abc,
                }],
            }],
        )
        .unwrap();
        assert_eq!(
            manifest.sha256().unwrap(),
            [
                0xb9, 0xab, 0x3c, 0x5b, 0x26, 0x3a, 0x3d, 0x6a, 0xd1, 0x4c, 0x9e, 0xb5, 0x20, 0xed,
                0xf6, 0xda, 0x1f, 0x67, 0x16, 0x6d, 0xa8, 0x15, 0xf8, 0x75, 0xd3, 0xcb, 0x6f, 0x63,
                0x0b, 0x9f, 0x8d, 0xc3,
            ]
        );
    }
}
