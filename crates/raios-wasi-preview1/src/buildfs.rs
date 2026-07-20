use alloc::{vec, vec::Vec};

use crate::{DirectoryEntry, DirectorySnapshot, Errno, FileType, NodeRef, NormalizedPath};

/// Dependency-free view over a validated core `BuildFsManifest`.
///
/// The adapter supplies the one authoritative chunk size from raios-core.
/// Index methods must return `Some` for every index below their declared count.
pub trait BuildFsManifestView {
    fn chunk_size(&self) -> u64;
    fn directory_count(&self) -> usize;
    fn directory_path(&self, index: usize) -> Option<&str>;
    fn file_count(&self) -> usize;
    fn file_path(&self, file: usize) -> Option<&str>;
    fn file_len(&self, file: usize) -> Option<u64>;
    fn file_sha256(&self, file: usize) -> Option<[u8; 32]>;
    fn chunk_count(&self, file: usize) -> Option<usize>;
    fn chunk_len(&self, file: usize, chunk: usize) -> Option<u64>;
    fn chunk_sha256(&self, file: usize, chunk: usize) -> Option<[u8; 32]>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ChunkDescriptor {
    pub len: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileNode {
    pub len: u64,
    pub sha256: [u8; 32],
    pub chunks: Vec<ChunkDescriptor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum NodeContent {
    Directory(DirectorySnapshot),
    File(FileNode),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BuildFsNode {
    path: Vec<u8>,
    content: NodeContent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildFs {
    chunk_size: u64,
    nodes: Vec<BuildFsNode>,
}

impl BuildFs {
    /// Projects a manifest view into deterministic path-sorted node numbers.
    pub fn project(manifest: &impl BuildFsManifestView) -> Result<Self, Errno> {
        let chunk_size = manifest.chunk_size();
        if chunk_size == 0 {
            return Err(Errno::Inval);
        }

        let mut pending = Vec::new();
        pending.push(BuildFsNode {
            path: Vec::new(),
            content: NodeContent::Directory(DirectorySnapshot::default()),
        });
        for index in 0..manifest.directory_count() {
            let path = manifest.directory_path(index).ok_or(Errno::Inval)?;
            validate_relative_path(path.as_bytes())?;
            pending.push(BuildFsNode {
                path: path.as_bytes().to_vec(),
                content: NodeContent::Directory(DirectorySnapshot::default()),
            });
        }
        for file_index in 0..manifest.file_count() {
            let path = manifest.file_path(file_index).ok_or(Errno::Inval)?;
            validate_relative_path(path.as_bytes())?;
            let len = manifest.file_len(file_index).ok_or(Errno::Inval)?;
            let sha256 = manifest.file_sha256(file_index).ok_or(Errno::Inval)?;
            let chunk_count = manifest.chunk_count(file_index).ok_or(Errno::Inval)?;
            let mut chunks = Vec::with_capacity(chunk_count);
            let mut sum = 0u64;
            for chunk_index in 0..chunk_count {
                let chunk_len = manifest
                    .chunk_len(file_index, chunk_index)
                    .ok_or(Errno::Inval)?;
                sum = sum.checked_add(chunk_len).ok_or(Errno::Inval)?;
                chunks.push(ChunkDescriptor {
                    len: chunk_len,
                    sha256: manifest
                        .chunk_sha256(file_index, chunk_index)
                        .ok_or(Errno::Inval)?,
                });
            }
            validate_chunks(len, chunk_size, sum, &chunks)?;
            pending.push(BuildFsNode {
                path: path.as_bytes().to_vec(),
                content: NodeContent::File(FileNode {
                    len,
                    sha256,
                    chunks,
                }),
            });
        }

        pending[1..].sort_by(|left, right| left.path.cmp(&right.path));
        if pending[1..]
            .windows(2)
            .any(|pair| pair[0].path == pair[1].path)
        {
            return Err(Errno::Exist);
        }
        for node in &pending[1..] {
            let parent = parent_path(&node.path);
            let parent_index = pending
                .binary_search_by(|candidate| candidate.path.as_slice().cmp(parent))
                .map_err(|_| Errno::Notdir)?;
            if !matches!(pending[parent_index].content, NodeContent::Directory(_)) {
                return Err(Errno::Notdir);
            }
        }

        let mut snapshots = vec![None; pending.len()];
        for (parent_index, parent) in pending.iter().enumerate() {
            if !matches!(parent.content, NodeContent::Directory(_)) {
                continue;
            }
            let mut entries = Vec::new();
            for (child_index, child) in pending.iter().enumerate().skip(1) {
                if parent_path(&child.path) == parent.path.as_slice() {
                    entries.push(DirectoryEntry::new(
                        base_name(&child.path),
                        node_ref(child_index)?,
                        child.file_type(),
                    )?);
                }
            }
            snapshots[parent_index] = Some(DirectorySnapshot::new(entries)?);
        }
        for (index, snapshot) in snapshots.into_iter().enumerate() {
            if let Some(snapshot) = snapshot {
                pending[index].content = NodeContent::Directory(snapshot);
            }
        }
        Ok(Self {
            chunk_size,
            nodes: pending,
        })
    }

    pub const fn root_node(&self) -> NodeRef {
        NodeRef(0)
    }

    pub const fn chunk_size(&self) -> u64 {
        self.chunk_size
    }

    pub fn node_for_path(&self, path: &NormalizedPath) -> Result<NodeRef, Errno> {
        let absolute = path.to_absolute_bytes();
        let relative = if absolute == b"/" {
            &b""[..]
        } else {
            &absolute[1..]
        };
        self.nodes
            .binary_search_by(|node| node.path.as_slice().cmp(relative))
            .map_err(|_| Errno::Noent)
            .and_then(node_ref)
    }

    pub(crate) fn path(&self, node: NodeRef) -> Result<&[u8], Errno> {
        Ok(&self.node(node)?.path)
    }

    pub(crate) fn file(&self, node: NodeRef) -> Result<&FileNode, Errno> {
        match &self.node(node)?.content {
            NodeContent::File(file) => Ok(file),
            NodeContent::Directory(_) => Err(Errno::Isdir),
        }
    }

    pub(crate) fn snapshot(&self, node: NodeRef) -> Result<&DirectorySnapshot, Errno> {
        match &self.node(node)?.content {
            NodeContent::Directory(snapshot) => Ok(snapshot),
            NodeContent::File(_) => Err(Errno::Notdir),
        }
    }

    pub(crate) fn file_type(&self, node: NodeRef) -> Result<FileType, Errno> {
        Ok(self.node(node)?.file_type())
    }

    fn node(&self, node: NodeRef) -> Result<&BuildFsNode, Errno> {
        let index = usize::try_from(node.0).map_err(|_| Errno::Badf)?;
        self.nodes.get(index).ok_or(Errno::Badf)
    }
}

impl BuildFsNode {
    fn file_type(&self) -> FileType {
        match self.content {
            NodeContent::Directory(_) => FileType::Directory,
            NodeContent::File(_) => FileType::RegularFile,
        }
    }
}

fn validate_relative_path(path: &[u8]) -> Result<(), Errno> {
    if path.is_empty() || path.first() == Some(&b'/') {
        return Err(Errno::Inval);
    }
    for component in path.split(|byte| *byte == b'/') {
        if component.is_empty() || component == b"." || component == b".." || component.contains(&0)
        {
            return Err(Errno::Inval);
        }
    }
    Ok(())
}

fn validate_chunks(
    len: u64,
    chunk_size: u64,
    sum: u64,
    chunks: &[ChunkDescriptor],
) -> Result<(), Errno> {
    let expected_count = if len == 0 {
        0
    } else {
        ((len - 1) / chunk_size) + 1
    };
    if u64::try_from(chunks.len()).map_err(|_| Errno::Inval)? != expected_count || sum != len {
        return Err(Errno::Inval);
    }
    for (index, chunk) in chunks.iter().enumerate() {
        let is_last = index + 1 == chunks.len();
        let expected_len = if is_last {
            let remainder = len % chunk_size;
            if remainder == 0 {
                chunk_size
            } else {
                remainder
            }
        } else {
            chunk_size
        };
        if chunk.len != expected_len {
            return Err(Errno::Inval);
        }
    }
    Ok(())
}

fn parent_path(path: &[u8]) -> &[u8] {
    path.iter()
        .rposition(|byte| *byte == b'/')
        .map_or(&b""[..], |index| &path[..index])
}

fn base_name(path: &[u8]) -> &[u8] {
    path.iter()
        .rposition(|byte| *byte == b'/')
        .map_or(path, |index| &path[index + 1..])
}

fn node_ref(index: usize) -> Result<NodeRef, Errno> {
    Ok(NodeRef(u64::try_from(index).map_err(|_| Errno::Inval)?))
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    struct View {
        directories: Vec<&'static str>,
        files: Vec<(&'static str, u64, Vec<(u64, [u8; 32])>)>,
    }

    impl BuildFsManifestView for View {
        fn chunk_size(&self) -> u64 {
            8
        }
        fn directory_count(&self) -> usize {
            self.directories.len()
        }
        fn directory_path(&self, index: usize) -> Option<&str> {
            self.directories.get(index).copied()
        }
        fn file_count(&self) -> usize {
            self.files.len()
        }
        fn file_path(&self, file: usize) -> Option<&str> {
            self.files.get(file).map(|value| value.0)
        }
        fn file_len(&self, file: usize) -> Option<u64> {
            self.files.get(file).map(|value| value.1)
        }
        fn file_sha256(&self, file: usize) -> Option<[u8; 32]> {
            self.files.get(file).map(|_| [9; 32])
        }
        fn chunk_count(&self, file: usize) -> Option<usize> {
            self.files.get(file).map(|value| value.2.len())
        }
        fn chunk_len(&self, file: usize, chunk: usize) -> Option<u64> {
            self.files.get(file)?.2.get(chunk).map(|value| value.0)
        }
        fn chunk_sha256(&self, file: usize, chunk: usize) -> Option<[u8; 32]> {
            self.files.get(file)?.2.get(chunk).map(|value| value.1)
        }
    }

    #[test]
    fn projection_is_path_sorted_and_construction_order_independent() {
        let first = View {
            directories: vec!["sysroot/lib", "src", "sysroot"],
            files: vec![
                ("sysroot/lib/z", 1, vec![(1, [1; 32])]),
                ("src/a", 1, vec![(1, [2; 32])]),
            ],
        };
        let second = View {
            directories: vec!["src", "sysroot", "sysroot/lib"],
            files: vec![
                ("src/a", 1, vec![(1, [2; 32])]),
                ("sysroot/lib/z", 1, vec![(1, [1; 32])]),
            ],
        };
        let first = BuildFs::project(&first).unwrap();
        let second = BuildFs::project(&second).unwrap();
        assert_eq!(first, second);
        let root = first.snapshot(first.root_node()).unwrap();
        assert_eq!(root.entries()[0].name(), b"src");
        assert_eq!(root.entries()[1].name(), b"sysroot");
        assert_eq!(root.entries()[0].node(), NodeRef(1));
    }

    #[test]
    fn projection_rejects_duplicates_missing_parents_and_bad_chunk_math() {
        let duplicate = View {
            directories: vec!["src", "src"],
            files: vec![],
        };
        assert_eq!(BuildFs::project(&duplicate), Err(Errno::Exist));
        let missing_parent = View {
            directories: vec![],
            files: vec![("src/a", 1, vec![(1, [0; 32])])],
        };
        assert_eq!(BuildFs::project(&missing_parent), Err(Errno::Notdir));
        let bad_chunks = View {
            directories: vec!["src"],
            files: vec![("src/a", 9, vec![(9, [0; 32])])],
        };
        assert_eq!(BuildFs::project(&bad_chunks), Err(Errno::Inval));
    }
}
