//! Canonical chunk-CAS manifest for deterministic build inputs.
//!
//! Paths are relative UTF-8 paths. The root is implicit; every other directory
//! is explicit, and every non-root parent must precede its children.

use alloc::{string::String, vec::Vec};
use core::cmp::Ordering;

use crate::sha256_bytes;

pub const BUILD_FS_MANIFEST_V1: &str = "raios.buildfs_manifest.v1";
pub const BUILD_FS_CHUNK_SIZE: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildFsChunk {
    pub len: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildFsDirectory {
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildFsFile {
    pub path: String,
    pub len: u64,
    pub sha256: [u8; 32],
    pub chunks: Vec<BuildFsChunk>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildFsManifest {
    pub directories: Vec<BuildFsDirectory>,
    pub files: Vec<BuildFsFile>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildFsManifestError {
    AbsolutePath,
    ChunkCount,
    ChunkLength,
    ChunkLengthOverflow,
    DuplicatePath,
    EmptyName,
    MissingParent,
    NonCanonicalOrder,
    ParentComponent,
}

impl BuildFsManifest {
    /// Builds canonical lists from construction-order-independent inputs.
    pub fn new(
        mut directories: Vec<BuildFsDirectory>,
        mut files: Vec<BuildFsFile>,
    ) -> Result<Self, BuildFsManifestError> {
        directories.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
        files.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
        let manifest = Self { directories, files };
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validates already-encoded order without silently canonicalizing it.
    pub fn validate(&self) -> Result<(), BuildFsManifestError> {
        validate_ordered_paths(self.directories.iter().map(|entry| entry.path.as_str()))?;
        validate_ordered_paths(self.files.iter().map(|entry| entry.path.as_str()))?;

        for directory in &self.directories {
            validate_path(&directory.path)?;
        }
        for file in &self.files {
            validate_path(&file.path)?;
        }

        for directory in &self.directories {
            if self
                .files
                .binary_search_by(|file| file.path.as_bytes().cmp(directory.path.as_bytes()))
                .is_ok()
            {
                return Err(BuildFsManifestError::DuplicatePath);
            }
            validate_parent(&directory.path, &self.directories)?;
        }

        for file in &self.files {
            validate_parent(&file.path, &self.directories)?;
            validate_chunks(file)?;
        }
        Ok(())
    }

    /// Canonical v1 bytes: length-prefixed domain, chunk size, then ordered
    /// directory and file records. All integers are little-endian `u64`.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BuildFsManifestError> {
        self.validate()?;
        let mut bytes = Vec::new();
        write_bytes(&mut bytes, BUILD_FS_MANIFEST_V1.as_bytes());
        bytes.extend_from_slice(&BUILD_FS_CHUNK_SIZE.to_le_bytes());
        write_len(&mut bytes, self.directories.len());
        for directory in &self.directories {
            write_bytes(&mut bytes, directory.path.as_bytes());
        }
        write_len(&mut bytes, self.files.len());
        for file in &self.files {
            write_bytes(&mut bytes, file.path.as_bytes());
            bytes.extend_from_slice(&file.len.to_le_bytes());
            bytes.extend_from_slice(&file.sha256);
            write_len(&mut bytes, file.chunks.len());
            for chunk in &file.chunks {
                bytes.extend_from_slice(&chunk.len.to_le_bytes());
                bytes.extend_from_slice(&chunk.sha256);
            }
        }
        Ok(bytes)
    }

    pub fn sha256(&self) -> Result<[u8; 32], BuildFsManifestError> {
        Ok(sha256_bytes(&self.canonical_bytes()?))
    }
}

fn validate_ordered_paths<'a>(
    paths: impl Iterator<Item = &'a str>,
) -> Result<(), BuildFsManifestError> {
    let mut previous: Option<&[u8]> = None;
    for path in paths {
        let bytes = path.as_bytes();
        if let Some(previous) = previous {
            match previous.cmp(bytes) {
                Ordering::Less => {}
                Ordering::Equal => return Err(BuildFsManifestError::DuplicatePath),
                Ordering::Greater => return Err(BuildFsManifestError::NonCanonicalOrder),
            }
        }
        previous = Some(bytes);
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<(), BuildFsManifestError> {
    if path.starts_with('/') {
        return Err(BuildFsManifestError::AbsolutePath);
    }
    if path.is_empty() {
        return Err(BuildFsManifestError::EmptyName);
    }
    for component in path.split('/') {
        if component.is_empty() || component == "." {
            return Err(BuildFsManifestError::EmptyName);
        }
        if component == ".." {
            return Err(BuildFsManifestError::ParentComponent);
        }
    }
    Ok(())
}

fn validate_parent(
    path: &str,
    directories: &[BuildFsDirectory],
) -> Result<(), BuildFsManifestError> {
    let Some((parent, _)) = path.rsplit_once('/') else {
        return Ok(());
    };
    if directories
        .binary_search_by(|directory| directory.path.as_bytes().cmp(parent.as_bytes()))
        .is_err()
    {
        return Err(BuildFsManifestError::MissingParent);
    }
    Ok(())
}

fn validate_chunks(file: &BuildFsFile) -> Result<(), BuildFsManifestError> {
    let mut sum = 0u64;
    for chunk in &file.chunks {
        sum = sum
            .checked_add(chunk.len)
            .ok_or(BuildFsManifestError::ChunkLengthOverflow)?;
    }

    let expected_count = if file.len == 0 {
        0
    } else {
        ((file.len - 1) / BUILD_FS_CHUNK_SIZE) + 1
    };
    let actual_count =
        u64::try_from(file.chunks.len()).map_err(|_| BuildFsManifestError::ChunkCount)?;
    if actual_count != expected_count {
        return Err(BuildFsManifestError::ChunkCount);
    }

    for (index, chunk) in file.chunks.iter().enumerate() {
        let is_last = index + 1 == file.chunks.len();
        let expected_len = if is_last {
            let remainder = file.len % BUILD_FS_CHUNK_SIZE;
            if remainder == 0 {
                BUILD_FS_CHUNK_SIZE
            } else {
                remainder
            }
        } else {
            BUILD_FS_CHUNK_SIZE
        };
        if chunk.len != expected_len {
            return Err(BuildFsManifestError::ChunkLength);
        }
    }
    if sum != file.len {
        return Err(BuildFsManifestError::ChunkLength);
    }
    Ok(())
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
    use alloc::{string::ToString, vec};

    use super::*;

    fn chunk(len: u64, tag: u8) -> BuildFsChunk {
        BuildFsChunk {
            len,
            sha256: [tag; 32],
        }
    }

    fn file(path: &str, len: u64, chunks: Vec<BuildFsChunk>) -> BuildFsFile {
        BuildFsFile {
            path: path.to_string(),
            len,
            sha256: [7; 32],
            chunks,
        }
    }

    fn directory(path: &str) -> BuildFsDirectory {
        BuildFsDirectory {
            path: path.to_string(),
        }
    }

    #[test]
    fn constructor_order_does_not_change_canonical_bytes_or_hash() {
        let first = BuildFsManifest::new(
            vec![
                directory("sysroot/lib"),
                directory("src"),
                directory("sysroot"),
            ],
            vec![
                file("sysroot/lib/z", 1, vec![chunk(1, 2)]),
                file("src/a", 1, vec![chunk(1, 1)]),
            ],
        )
        .unwrap();
        let second = BuildFsManifest::new(
            vec![
                directory("sysroot"),
                directory("sysroot/lib"),
                directory("src"),
            ],
            vec![
                file("src/a", 1, vec![chunk(1, 1)]),
                file("sysroot/lib/z", 1, vec![chunk(1, 2)]),
            ],
        )
        .unwrap();

        assert_eq!(first.directories[0].path, "src");
        assert_eq!(first.canonical_bytes(), second.canonical_bytes());
        assert_eq!(first.sha256(), second.sha256());
        assert!(first
            .canonical_bytes()
            .unwrap()
            .windows(BUILD_FS_MANIFEST_V1.len())
            .any(|window| window == BUILD_FS_MANIFEST_V1.as_bytes()));
    }

    #[test]
    fn validation_rejects_unsorted_duplicate_and_invalid_paths() {
        let unsorted = BuildFsManifest {
            directories: vec![directory("z"), directory("a")],
            files: vec![],
        };
        assert_eq!(
            unsorted.validate(),
            Err(BuildFsManifestError::NonCanonicalOrder)
        );

        let duplicate = BuildFsManifest {
            directories: vec![directory("a"), directory("a")],
            files: vec![],
        };
        assert_eq!(
            duplicate.validate(),
            Err(BuildFsManifestError::DuplicatePath)
        );
        for invalid in ["", "/src", "src//a", "src/..", "src/./a"] {
            let manifest = BuildFsManifest {
                directories: vec![directory(invalid)],
                files: vec![],
            };
            assert!(manifest.validate().is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn chunk_arithmetic_rejects_count_sum_shape_and_overflow() {
        let manifest = |candidate| BuildFsManifest {
            directories: vec![directory("src")],
            files: vec![candidate],
        };

        assert_eq!(
            manifest(file(
                "src/a",
                BUILD_FS_CHUNK_SIZE + 1,
                vec![chunk(BUILD_FS_CHUNK_SIZE + 1, 1)]
            ))
            .validate(),
            Err(BuildFsManifestError::ChunkCount)
        );
        assert_eq!(
            manifest(file(
                "src/a",
                BUILD_FS_CHUNK_SIZE,
                vec![chunk(BUILD_FS_CHUNK_SIZE - 1, 1)]
            ))
            .validate(),
            Err(BuildFsManifestError::ChunkLength)
        );
        assert_eq!(
            manifest(file(
                "src/a",
                BUILD_FS_CHUNK_SIZE * 2,
                vec![chunk(u64::MAX, 1), chunk(1, 2)],
            ))
            .validate(),
            Err(BuildFsManifestError::ChunkLengthOverflow)
        );
        assert_eq!(manifest(file("src/empty", 0, vec![])).validate(), Ok(()));
    }

    #[test]
    fn parent_directories_and_cross_kind_paths_are_fail_closed() {
        let missing = BuildFsManifest {
            directories: vec![],
            files: vec![file("src/a", 1, vec![chunk(1, 1)])],
        };
        assert_eq!(missing.validate(), Err(BuildFsManifestError::MissingParent));

        let collision = BuildFsManifest {
            directories: vec![directory("src")],
            files: vec![file("src", 1, vec![chunk(1, 1)])],
        };
        assert_eq!(
            collision.validate(),
            Err(BuildFsManifestError::DuplicatePath)
        );
    }
}
