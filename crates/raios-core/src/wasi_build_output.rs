//! Typed RAM-resident output of one WASI build run.

pub use crate::buildfs_manifest::{BUILD_FS_CHUNK_SIZE, BUILD_FS_MANIFEST_V1};
use crate::sha256_bytes;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrozenOutput {
    digest: [u8; 32],
}

impl FrozenOutput {
    /// Freezes canonical output-manifest bytes into a content-bound digest.
    ///
    /// No constructor accepts a claimed digest, so callers cannot detach the
    /// value used by the egress gate from the bytes produced by the build.
    pub fn from_manifest_bytes(manifest_bytes: &[u8]) -> Self {
        Self {
            digest: sha256_bytes(manifest_bytes),
        }
    }

    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec};

    use crate::buildfs_manifest::{BuildFsChunk, BuildFsDirectory, BuildFsFile, BuildFsManifest};

    use super::*;

    #[test]
    fn frozen_output_digest_is_computed_from_manifest_bytes() {
        let output = FrozenOutput::from_manifest_bytes(b"canonical manifest");
        assert_eq!(output.digest(), sha256_bytes(b"canonical manifest"));
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
