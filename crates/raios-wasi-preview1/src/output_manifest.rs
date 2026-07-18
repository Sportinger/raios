//! Canonical `/out` freeze using the `BuildFsManifest v1` byte contract.

use alloc::{string::String, vec::Vec};

use crate::ramfs::{RamFs, RamSnapshotContent};
use crate::{BuildFsManifestView, Errno};

// These values are the authoritative raios-core BuildFsManifest-v1 constants.
// This dependency-free crate cannot import raios-core; the compatibility test
// below pins the exact canonical bytes so the two definitions cannot drift.
pub const BUILD_FS_MANIFEST_V1: &str = "raios.buildfs_manifest.v1";
pub const BUILD_FS_CHUNK_SIZE: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputChunk {
    pub len: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputDirectory {
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputFile {
    pub path: String,
    pub len: u64,
    pub sha256: [u8; 32],
    pub chunks: Vec<OutputChunk>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputManifest {
    pub directories: Vec<OutputDirectory>,
    pub files: Vec<OutputFile>,
    pub sha256: [u8; 32],
}

impl OutputManifest {
    /// Returns exactly the canonical bytes accepted by raios-core's
    /// `BuildFsManifest::canonical_bytes`.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        canonical_bytes(&self.directories, &self.files)
    }
}

impl BuildFsManifestView for OutputManifest {
    fn chunk_size(&self) -> u64 {
        BUILD_FS_CHUNK_SIZE
    }

    fn directory_count(&self) -> usize {
        self.directories.len()
    }

    fn directory_path(&self, index: usize) -> Option<&str> {
        self.directories.get(index).map(|entry| entry.path.as_str())
    }

    fn file_count(&self) -> usize {
        self.files.len()
    }

    fn file_path(&self, file: usize) -> Option<&str> {
        self.files.get(file).map(|entry| entry.path.as_str())
    }

    fn file_len(&self, file: usize) -> Option<u64> {
        self.files.get(file).map(|entry| entry.len)
    }

    fn file_sha256(&self, file: usize) -> Option<[u8; 32]> {
        self.files.get(file).map(|entry| entry.sha256)
    }

    fn chunk_count(&self, file: usize) -> Option<usize> {
        self.files.get(file).map(|entry| entry.chunks.len())
    }

    fn chunk_len(&self, file: usize, chunk: usize) -> Option<u64> {
        self.files
            .get(file)?
            .chunks
            .get(chunk)
            .map(|entry| entry.len)
    }

    fn chunk_sha256(&self, file: usize, chunk: usize) -> Option<[u8; 32]> {
        self.files
            .get(file)?
            .chunks
            .get(chunk)
            .map(|entry| entry.sha256)
    }
}

/// Freezes only the supplied output arena. Scratch arenas are not an input to
/// this function and therefore cannot leak into the output manifest.
pub fn freeze_output(output: &RamFs) -> Result<OutputManifest, Errno> {
    let mut directories = Vec::new();
    let mut files = Vec::new();
    for entry in output.snapshot_entries()? {
        let path = String::from_utf8(entry.path).map_err(|_| Errno::Inval)?;
        match entry.content {
            RamSnapshotContent::Directory => directories.push(OutputDirectory { path }),
            RamSnapshotContent::File(bytes) => {
                let len = u64::try_from(bytes.len()).map_err(|_| Errno::Inval)?;
                let chunks = bytes
                    .chunks(BUILD_FS_CHUNK_SIZE as usize)
                    .map(|chunk| OutputChunk {
                        len: chunk.len() as u64,
                        sha256: crate::readonly::sha256(chunk),
                    })
                    .collect();
                files.push(OutputFile {
                    path,
                    len,
                    sha256: crate::readonly::sha256(&bytes),
                    chunks,
                });
            }
        }
    }
    directories.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    files.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    let bytes = canonical_bytes(&directories, &files);
    Ok(OutputManifest {
        directories,
        files,
        sha256: crate::readonly::sha256(&bytes),
    })
}

fn canonical_bytes(directories: &[OutputDirectory], files: &[OutputFile]) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_bytes(&mut bytes, BUILD_FS_MANIFEST_V1.as_bytes());
    bytes.extend_from_slice(&BUILD_FS_CHUNK_SIZE.to_le_bytes());
    write_len(&mut bytes, directories.len());
    for directory in directories {
        write_bytes(&mut bytes, directory.path.as_bytes());
    }
    write_len(&mut bytes, files.len());
    for file in files {
        write_bytes(&mut bytes, file.path.as_bytes());
        bytes.extend_from_slice(&file.len.to_le_bytes());
        bytes.extend_from_slice(&file.sha256);
        write_len(&mut bytes, file.chunks.len());
        for chunk in &file.chunks {
            bytes.extend_from_slice(&chunk.len.to_le_bytes());
            bytes.extend_from_slice(&chunk.sha256);
        }
    }
    bytes
}

fn write_len(output: &mut Vec<u8>, len: usize) {
    output.extend_from_slice(&(len as u64).to_le_bytes());
}

fn write_bytes(output: &mut Vec<u8>, value: &[u8]) {
    write_len(output, value.len());
    output.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::ramfs::RamQuotas;
    use crate::Fd;

    #[test]
    fn freeze_is_sorted_stable_and_has_exact_chunk_boundaries() {
        let mut out = RamFs::new(RamQuotas::new(200_000, 4, 4, 200_000), 12).unwrap();
        out.path_create_directory(Fd::ROOT_PREOPEN, b"zdir")
            .unwrap();
        out.path_create_file(Fd::ROOT_PREOPEN, b"zdir/small")
            .unwrap();
        out.write_path(b"zdir/small", 0, b"small").unwrap();
        out.path_create_file(Fd::ROOT_PREOPEN, b"large").unwrap();
        let large = vec![0x5a; BUILD_FS_CHUNK_SIZE as usize + 7];
        out.write_path(b"large", 0, &large).unwrap();

        let first = freeze_output(&out).unwrap();
        let second = freeze_output(&out).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.files[0].path, "large");
        assert_eq!(first.files[1].path, "zdir/small");
        assert_eq!(first.directories[0].path, "zdir");
        assert_eq!(first.files[0].chunks.len(), 2);
        assert_eq!(first.files[0].chunks[0].len, BUILD_FS_CHUNK_SIZE);
        assert_eq!(first.files[0].chunks[1].len, 7);
        assert_eq!(
            first.files[0].chunks[0].sha256,
            crate::readonly::sha256(&large[..BUILD_FS_CHUNK_SIZE as usize])
        );
        assert_eq!(
            first.files[0].chunks[1].sha256,
            crate::readonly::sha256(&large[BUILD_FS_CHUNK_SIZE as usize..])
        );
        assert_eq!(
            first.sha256,
            crate::readonly::sha256(&first.canonical_bytes())
        );
    }

    #[test]
    fn canonical_prefix_pins_core_buildfs_v1_domain_and_chunk_size() {
        let out = RamFs::new(RamQuotas::new(1, 1, 1, 1), 4).unwrap();
        let manifest = freeze_output(&out).unwrap();
        let bytes = manifest.canonical_bytes();
        let domain_len = BUILD_FS_MANIFEST_V1.len() as u64;
        assert_eq!(&bytes[..8], &domain_len.to_le_bytes());
        assert_eq!(
            &bytes[8..8 + domain_len as usize],
            BUILD_FS_MANIFEST_V1.as_bytes()
        );
        assert_eq!(
            &bytes[8 + domain_len as usize..16 + domain_len as usize],
            &BUILD_FS_CHUNK_SIZE.to_le_bytes()
        );
    }

    #[test]
    fn canonical_fixture_hash_matches_raios_core_buildfs_v1_golden() {
        let mut out = RamFs::new(RamQuotas::new(8, 1, 1, 8), 4).unwrap();
        out.path_create_directory(Fd::ROOT_PREOPEN, b"dir").unwrap();
        out.path_create_file(Fd::ROOT_PREOPEN, b"dir/a").unwrap();
        out.write_path(b"dir/a", 0, b"abc").unwrap();
        let manifest = freeze_output(&out).unwrap();
        assert_eq!(
            manifest.sha256,
            [
                0xb9, 0xab, 0x3c, 0x5b, 0x26, 0x3a, 0x3d, 0x6a, 0xd1, 0x4c, 0x9e, 0xb5, 0x20, 0xed,
                0xf6, 0xda, 0x1f, 0x67, 0x16, 0x6d, 0xa8, 0x15, 0xf8, 0x75, 0xd3, 0xcb, 0x6f, 0x63,
                0x0b, 0x9f, 0x8d, 0xc3,
            ]
        );
    }
}
