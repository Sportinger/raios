use alloc::{vec, vec::Vec};
use core::cmp::{max, min};

use crate::dir::read_directory;
use crate::{
    seek_offset, BuildFs, Dirent, Errno, Fd, FdFlags, FdTable, FileType, NodeRef, NormalizedPath,
    PathOpenRequest, Rights, StandardNodes, Whence,
};

/// `2000-01-01T00:00:00Z`, expressed as a preview1 timestamp in nanoseconds.
pub const EPOCH_2000_NS: u64 = 946_684_800_000_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Filestat {
    pub device: u64,
    pub inode: u64,
    pub file_type: FileType,
    pub link_count: u64,
    pub size: u64,
    pub access_time: u64,
    pub modification_time: u64,
    pub status_change_time: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChunkReadError {
    Io,
    NotCapable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChunkReadRequest {
    pub file: NodeRef,
    pub file_sha256: [u8; 32],
    pub chunk_index: u64,
    pub chunk_sha256: [u8; 32],
    pub range_offset: u64,
    pub range_len: u64,
}

/// Abstract granted chunk reader. It has no ambient fallback lookup.
///
/// The read-only layer currently requests one complete chunk at a time so it
/// can verify the manifest hash before exposing any requested subrange.
pub trait ChunkRead {
    fn read_chunk(
        &mut self,
        request: ChunkReadRequest,
        destination: &mut [u8],
    ) -> Result<(), ChunkReadError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadOnlyFs {
    buildfs: BuildFs,
    fds: FdTable,
    offsets: Vec<Option<u64>>,
}

impl ReadOnlyFs {
    pub fn new(
        buildfs: BuildFs,
        fd_limit: u32,
        standard_nodes: StandardNodes,
    ) -> Result<Self, Errno> {
        if standard_nodes.root != buildfs.root_node() {
            return Err(Errno::Inval);
        }
        let fds = FdTable::new(fd_limit, standard_nodes, readonly_rights())?;
        Ok(Self {
            buildfs,
            fds,
            offsets: vec![Some(0); Fd::FIRST_DYNAMIC.0 as usize],
        })
    }

    pub fn fd_table(&self) -> &FdTable {
        &self.fds
    }

    pub fn path_open(
        &mut self,
        parent_fd: Fd,
        path: &[u8],
        requested_rights: Rights,
        flags: FdFlags,
    ) -> Result<Fd, Errno> {
        if has_mutating_rights(requested_rights) || flags != FdFlags::EMPTY {
            return Err(Errno::Rofs);
        }
        let node = self.resolve(parent_fd, path, Rights::PATH_OPEN)?;
        let file_type = self.buildfs.file_type(node)?;
        let fd = self.fds.path_open(PathOpenRequest {
            parent_fd,
            node,
            file_type,
            mount_rights: readonly_rights(),
            requested_rights,
            flags,
        })?;
        let index = fd.0 as usize;
        if self.offsets.len() <= index {
            self.offsets.resize(index + 1, None);
        }
        self.offsets[index] = Some(0);
        Ok(fd)
    }

    pub fn fd_close(&mut self, fd: Fd) -> Result<(), Errno> {
        self.fds.close(fd)?;
        if let Some(offset) = self.offsets.get_mut(fd.0 as usize) {
            *offset = None;
        }
        Ok(())
    }

    pub fn fd_read(
        &mut self,
        fd: Fd,
        len: usize,
        reader: &mut impl ChunkRead,
    ) -> Result<Vec<u8>, Errno> {
        let entry = *self.build_entry(fd)?;
        require_right(entry.rights, Rights::FD_READ)?;
        if entry.file_type != FileType::RegularFile {
            return Err(Errno::Isdir);
        }
        let offset = self.offset(fd)?;
        let bytes = self.read_file(entry.node, offset, len, reader)?;
        let next = offset
            .checked_add(u64::try_from(bytes.len()).map_err(|_| Errno::Inval)?)
            .ok_or(Errno::Inval)?;
        self.set_offset(fd, next)?;
        Ok(bytes)
    }

    pub fn fd_pread(
        &self,
        fd: Fd,
        offset: u64,
        len: usize,
        reader: &mut impl ChunkRead,
    ) -> Result<Vec<u8>, Errno> {
        let entry = self.build_entry(fd)?;
        require_right(entry.rights, Rights::FD_READ | Rights::FD_SEEK)?;
        if entry.file_type != FileType::RegularFile {
            return Err(Errno::Isdir);
        }
        self.read_file(entry.node, offset, len, reader)
    }

    pub fn fd_seek(&mut self, fd: Fd, delta: i64, whence: Whence) -> Result<u64, Errno> {
        let entry = *self.build_entry(fd)?;
        require_right(entry.rights, Rights::FD_SEEK)?;
        if entry.file_type != FileType::RegularFile {
            return Err(Errno::Isdir);
        }
        let current = self.offset(fd)?;
        let size = self.buildfs.file(entry.node)?.len;
        let target = seek_offset(current, size, delta, whence)?;
        self.set_offset(fd, target)?;
        Ok(target)
    }

    pub fn fd_tell(&self, fd: Fd) -> Result<u64, Errno> {
        let entry = self.build_entry(fd)?;
        require_right(entry.rights, Rights::FD_TELL)?;
        self.offset(fd)
    }

    pub fn fd_filestat_get(&self, fd: Fd) -> Result<Filestat, Errno> {
        let entry = self.build_entry_or_root(fd)?;
        require_right(entry.rights, Rights::FD_FILESTAT_GET)?;
        self.filestat(entry.node)
    }

    pub fn path_filestat_get(&self, parent_fd: Fd, path: &[u8]) -> Result<Filestat, Errno> {
        let node = self.resolve(parent_fd, path, Rights::PATH_FILESTAT_GET)?;
        self.filestat(node)
    }

    pub fn fd_readdir(
        &self,
        fd: Fd,
        cookie: u64,
        max_entries: usize,
    ) -> Result<Vec<Dirent>, Errno> {
        let entry = self.build_entry_or_root(fd)?;
        require_right(entry.rights, Rights::FD_READDIR)?;
        if entry.file_type != FileType::Directory {
            return Err(Errno::Notdir);
        }
        read_directory(self.buildfs.snapshot(entry.node)?, cookie, max_entries)
    }

    pub fn fd_write(&self, fd: Fd, _bytes: &[u8]) -> Result<usize, Errno> {
        self.build_entry(fd)?;
        Err(Errno::Rofs)
    }

    pub fn path_create_file(&self, parent_fd: Fd, _path: &[u8]) -> Result<(), Errno> {
        self.readonly_parent(parent_fd)?;
        Err(Errno::Rofs)
    }

    pub fn path_create_directory(&self, parent_fd: Fd, _path: &[u8]) -> Result<(), Errno> {
        self.readonly_parent(parent_fd)?;
        Err(Errno::Rofs)
    }

    pub fn path_rename(
        &self,
        source_parent: Fd,
        _source_path: &[u8],
        target_parent: Fd,
        _target_path: &[u8],
    ) -> Result<(), Errno> {
        self.readonly_parent(source_parent)?;
        self.readonly_parent(target_parent)?;
        Err(Errno::Rofs)
    }

    pub fn path_unlink_file(&self, parent_fd: Fd, _path: &[u8]) -> Result<(), Errno> {
        self.readonly_parent(parent_fd)?;
        Err(Errno::Rofs)
    }

    pub fn path_remove_directory(&self, parent_fd: Fd, _path: &[u8]) -> Result<(), Errno> {
        self.readonly_parent(parent_fd)?;
        Err(Errno::Rofs)
    }

    fn resolve(&self, parent_fd: Fd, path: &[u8], right: Rights) -> Result<NodeRef, Errno> {
        let parent = self.build_entry_or_root(parent_fd)?;
        if parent.file_type != FileType::Directory {
            return Err(Errno::Notdir);
        }
        require_right(parent.rights, right)?;
        let base = NormalizedPath::root().resolve(self.buildfs.path(parent.node)?)?;
        let resolved = base.resolve(path)?;
        self.buildfs.node_for_path(&resolved)
    }

    fn read_file(
        &self,
        node: NodeRef,
        offset: u64,
        len: usize,
        reader: &mut impl ChunkRead,
    ) -> Result<Vec<u8>, Errno> {
        let file = self.buildfs.file(node)?;
        let requested_len = u64::try_from(len).map_err(|_| Errno::Inval)?;
        let requested_end = offset.checked_add(requested_len).ok_or(Errno::Inval)?;
        let end = min(requested_end, file.len);
        if offset >= end {
            return Ok(Vec::new());
        }

        let chunk_size = self.buildfs.chunk_size();
        let first_chunk = offset / chunk_size;
        let last_chunk = (end - 1) / chunk_size;
        let output_len = usize::try_from(end - offset).map_err(|_| Errno::Fbig)?;
        let mut output = Vec::with_capacity(output_len);
        for chunk_index in first_chunk..=last_chunk {
            let descriptor_index = usize::try_from(chunk_index).map_err(|_| Errno::Inval)?;
            let descriptor = file.chunks.get(descriptor_index).ok_or(Errno::Io)?;
            let chunk_len = usize::try_from(descriptor.len).map_err(|_| Errno::Fbig)?;
            let mut chunk = vec![0; chunk_len];
            reader
                .read_chunk(
                    ChunkReadRequest {
                        file: node,
                        file_sha256: file.sha256,
                        chunk_index,
                        chunk_sha256: descriptor.sha256,
                        range_offset: 0,
                        range_len: descriptor.len,
                    },
                    &mut chunk,
                )
                .map_err(chunk_error)?;
            if sha256(&chunk) != descriptor.sha256 {
                return Err(Errno::Io);
            }

            let chunk_start = chunk_index.checked_mul(chunk_size).ok_or(Errno::Inval)?;
            let copy_start = max(offset, chunk_start) - chunk_start;
            let copy_end = min(
                end,
                chunk_start
                    .checked_add(descriptor.len)
                    .ok_or(Errno::Inval)?,
            ) - chunk_start;
            let copy_start = usize::try_from(copy_start).map_err(|_| Errno::Inval)?;
            let copy_end = usize::try_from(copy_end).map_err(|_| Errno::Inval)?;
            output.extend_from_slice(chunk.get(copy_start..copy_end).ok_or(Errno::Io)?);
        }
        if output.len() != output_len {
            return Err(Errno::Io);
        }
        Ok(output)
    }

    fn filestat(&self, node: NodeRef) -> Result<Filestat, Errno> {
        let file_type = self.buildfs.file_type(node)?;
        let size = match file_type {
            FileType::RegularFile => self.buildfs.file(node)?.len,
            FileType::Directory => 0,
            _ => return Err(Errno::Io),
        };
        Ok(Filestat {
            device: 0,
            inode: node.0,
            file_type,
            link_count: 1,
            size,
            access_time: EPOCH_2000_NS,
            modification_time: EPOCH_2000_NS,
            status_change_time: EPOCH_2000_NS,
        })
    }

    fn build_entry(&self, fd: Fd) -> Result<&crate::FdEntry, Errno> {
        if fd.0 < Fd::FIRST_DYNAMIC.0 {
            return Err(Errno::Badf);
        }
        self.fds.get(fd)
    }

    fn build_entry_or_root(&self, fd: Fd) -> Result<&crate::FdEntry, Errno> {
        if fd != Fd::ROOT_PREOPEN && fd.0 < Fd::FIRST_DYNAMIC.0 {
            return Err(Errno::Badf);
        }
        self.fds.get(fd)
    }

    fn readonly_parent(&self, fd: Fd) -> Result<(), Errno> {
        let entry = self.build_entry_or_root(fd)?;
        if entry.file_type != FileType::Directory {
            return Err(Errno::Notdir);
        }
        self.buildfs.snapshot(entry.node)?;
        Ok(())
    }

    fn offset(&self, fd: Fd) -> Result<u64, Errno> {
        self.offsets
            .get(fd.0 as usize)
            .and_then(|offset| *offset)
            .ok_or(Errno::Badf)
    }

    fn set_offset(&mut self, fd: Fd, value: u64) -> Result<(), Errno> {
        let offset = self.offsets.get_mut(fd.0 as usize).ok_or(Errno::Badf)?;
        if offset.is_none() {
            return Err(Errno::Badf);
        }
        *offset = Some(value);
        Ok(())
    }
}

fn readonly_rights() -> Rights {
    Rights::FD_READ
        | Rights::FD_SEEK
        | Rights::FD_TELL
        | Rights::PATH_OPEN
        | Rights::FD_READDIR
        | Rights::PATH_FILESTAT_GET
        | Rights::FD_FILESTAT_GET
}

fn mutating_rights() -> Rights {
    Rights::FD_DATASYNC
        | Rights::FD_SYNC
        | Rights::FD_WRITE
        | Rights::FD_ALLOCATE
        | Rights::PATH_CREATE_DIRECTORY
        | Rights::PATH_CREATE_FILE
        | Rights::PATH_LINK_SOURCE
        | Rights::PATH_LINK_TARGET
        | Rights::PATH_RENAME_SOURCE
        | Rights::PATH_RENAME_TARGET
        | Rights::PATH_FILESTAT_SET_SIZE
        | Rights::PATH_FILESTAT_SET_TIMES
        | Rights::FD_FILESTAT_SET_SIZE
        | Rights::FD_FILESTAT_SET_TIMES
        | Rights::PATH_SYMLINK
        | Rights::PATH_REMOVE_DIRECTORY
        | Rights::PATH_UNLINK_FILE
}

fn has_mutating_rights(rights: Rights) -> bool {
    rights.intersection(mutating_rights()).bits() != 0
}

fn require_right(actual: Rights, required: Rights) -> Result<(), Errno> {
    if actual.contains(required) {
        Ok(())
    } else {
        Err(Errno::Notcapable)
    }
}

fn chunk_error(error: ChunkReadError) -> Errno {
    match error {
        ChunkReadError::Io => Errno::Io,
        ChunkReadError::NotCapable => Errno::Notcapable,
    }
}

pub(crate) fn sha256(bytes: &[u8]) -> [u8; 32] {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut state = INITIAL;
    let mut chunks = bytes.chunks_exact(64);
    for block in &mut chunks {
        compress_sha256(&mut state, block);
    }
    let remainder = chunks.remainder();
    let mut tail = [0u8; 128];
    tail[..remainder.len()].copy_from_slice(remainder);
    tail[remainder.len()] = 0x80;
    let tail_len = if remainder.len() < 56 { 64 } else { 128 };
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    tail[tail_len - 8..tail_len].copy_from_slice(&bit_len.to_be_bytes());
    compress_sha256(&mut state, &tail[..64]);
    if tail_len == 128 {
        compress_sha256(&mut state, &tail[64..]);
    }
    let mut digest = [0u8; 32];
    for (word, output) in state.iter().zip(digest.chunks_exact_mut(4)) {
        output.copy_from_slice(&word.to_be_bytes());
    }
    digest
}

fn compress_sha256(state: &mut [u32; 8], block: &[u8]) {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut words = [0u32; 64];
    for (index, bytes) in block.chunks_exact(4).take(16).enumerate() {
        words[index] = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
    for index in 16..64 {
        let s0 = words[index - 15].rotate_right(7)
            ^ words[index - 15].rotate_right(18)
            ^ (words[index - 15] >> 3);
        let s1 = words[index - 2].rotate_right(17)
            ^ words[index - 2].rotate_right(19)
            ^ (words[index - 2] >> 10);
        words[index] = words[index - 16]
            .wrapping_add(s0)
            .wrapping_add(words[index - 7])
            .wrapping_add(s1);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for index in 0..64 {
        let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choice = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(sum1)
            .wrapping_add(choice)
            .wrapping_add(K[index])
            .wrapping_add(words[index]);
        let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = sum0.wrapping_add(majority);
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::BuildFsManifestView;

    const CHUNK_SIZE: u64 = 64 * 1024;
    const LARGE_LEN: u64 = 71 * 1024 * 1024;

    struct LargeManifest {
        reverse: bool,
        chunk_hash: [u8; 32],
    }

    impl BuildFsManifestView for LargeManifest {
        fn chunk_size(&self) -> u64 {
            CHUNK_SIZE
        }
        fn directory_count(&self) -> usize {
            2
        }
        fn directory_path(&self, index: usize) -> Option<&str> {
            match (self.reverse, index) {
                (false, 0) | (true, 1) => Some("src"),
                (false, 1) | (true, 0) => Some("sysroot"),
                _ => None,
            }
        }
        fn file_count(&self) -> usize {
            1
        }
        fn file_path(&self, file: usize) -> Option<&str> {
            (file == 0).then_some("src/large")
        }
        fn file_len(&self, file: usize) -> Option<u64> {
            (file == 0).then_some(LARGE_LEN)
        }
        fn file_sha256(&self, file: usize) -> Option<[u8; 32]> {
            (file == 0).then_some([7; 32])
        }
        fn chunk_count(&self, file: usize) -> Option<usize> {
            (file == 0).then_some((LARGE_LEN / CHUNK_SIZE) as usize)
        }
        fn chunk_len(&self, file: usize, chunk: usize) -> Option<u64> {
            (file == 0 && chunk < (LARGE_LEN / CHUNK_SIZE) as usize).then_some(CHUNK_SIZE)
        }
        fn chunk_sha256(&self, file: usize, chunk: usize) -> Option<[u8; 32]> {
            (file == 0 && chunk < (LARGE_LEN / CHUNK_SIZE) as usize).then_some(self.chunk_hash)
        }
    }

    #[derive(Default)]
    struct CountingReader {
        calls: Vec<ChunkReadRequest>,
        denied_chunk: Option<u64>,
        corrupt: bool,
    }

    impl ChunkRead for CountingReader {
        fn read_chunk(
            &mut self,
            request: ChunkReadRequest,
            destination: &mut [u8],
        ) -> Result<(), ChunkReadError> {
            self.calls.push(request);
            if self.denied_chunk == Some(request.chunk_index) {
                return Err(ChunkReadError::NotCapable);
            }
            destination.fill(0);
            if self.corrupt && !destination.is_empty() {
                destination[0] = 1;
            }
            Ok(())
        }
    }

    fn filesystem(reverse: bool) -> ReadOnlyFs {
        let chunk_hash = sha256(&vec![0; CHUNK_SIZE as usize]);
        let buildfs = BuildFs::project(&LargeManifest {
            reverse,
            chunk_hash,
        })
        .unwrap();
        let root = buildfs.root_node();
        ReadOnlyFs::new(
            buildfs,
            32,
            StandardNodes {
                stdin: NodeRef(u64::MAX - 2),
                stdout: NodeRef(u64::MAX - 1),
                stderr: NodeRef(u64::MAX),
                root,
            },
        )
        .unwrap()
    }

    fn open_large(fs: &mut ReadOnlyFs) -> Fd {
        fs.path_open(
            Fd::ROOT_PREOPEN,
            b"src/large",
            Rights::FD_READ | Rights::FD_SEEK | Rights::FD_TELL | Rights::FD_FILESTAT_GET,
            FdFlags::EMPTY,
        )
        .unwrap()
    }

    #[test]
    fn sha256_verifier_matches_the_standard_vector() {
        assert_eq!(
            sha256(b"abc"),
            [
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad
            ]
        );
    }

    #[test]
    fn seventy_one_mb_range_reads_touch_exactly_the_needed_chunks() {
        let mut fs = filesystem(false);
        let fd = open_large(&mut fs);
        let mut reader = CountingReader::default();
        let middle = CHUNK_SIZE * 500 + 1_000;
        assert_eq!(
            fs.fd_pread(fd, middle, 100, &mut reader).unwrap(),
            vec![0; 100]
        );
        assert_eq!(reader.calls.len(), 1);
        assert_eq!(reader.calls[0].chunk_index, 500);
        assert_eq!(reader.calls[0].range_len, CHUNK_SIZE);

        reader.calls.clear();
        assert_eq!(
            fs.fd_pread(fd, CHUNK_SIZE - 50, 100, &mut reader).unwrap(),
            vec![0; 100]
        );
        assert_eq!(reader.calls.len(), 2);
        assert_eq!(
            reader
                .calls
                .iter()
                .map(|call| call.chunk_index)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    #[test]
    fn hash_failure_and_ungranted_chunk_are_typed_and_atomic() {
        let mut fs = filesystem(false);
        let fd = open_large(&mut fs);
        let mut corrupt = CountingReader {
            corrupt: true,
            ..CountingReader::default()
        };
        assert_eq!(fs.fd_pread(fd, 20, 100, &mut corrupt), Err(Errno::Io));
        assert_eq!(fs.fd_tell(fd), Ok(0));

        let mut denied = CountingReader {
            denied_chunk: Some(1),
            ..CountingReader::default()
        };
        assert_eq!(
            fs.fd_pread(fd, CHUNK_SIZE - 50, 100, &mut denied),
            Err(Errno::Notcapable)
        );
        assert_eq!(fs.fd_tell(fd), Ok(0));
        assert_eq!(denied.calls.len(), 2);
    }

    #[test]
    fn read_seek_tell_filestat_readdir_and_rebuild_are_deterministic() {
        let mut first = filesystem(false);
        let mut second = filesystem(true);
        assert_eq!(
            first.fd_readdir(Fd::ROOT_PREOPEN, 0, 10),
            second.fd_readdir(Fd::ROOT_PREOPEN, 0, 10)
        );
        let first_fd = open_large(&mut first);
        let second_fd = open_large(&mut second);
        assert_eq!(first_fd, second_fd);
        assert_eq!(
            first.fd_filestat_get(first_fd),
            second.fd_filestat_get(second_fd)
        );
        let mut first_reader = CountingReader::default();
        let mut second_reader = CountingReader::default();
        assert_eq!(
            first.fd_read(first_fd, 12, &mut first_reader),
            second.fd_read(second_fd, 12, &mut second_reader)
        );
        assert_eq!(first.fd_tell(first_fd), Ok(12));
        assert_eq!(first.fd_seek(first_fd, -2, Whence::Current), Ok(10));
        assert_eq!(first.fd_seek(first_fd, -1, Whence::End), Ok(LARGE_LEN - 1));
    }

    #[test]
    fn visible_times_are_fixed_to_epoch_2000() {
        let mut fs = filesystem(false);
        let fd = open_large(&mut fs);
        let file = fs.fd_filestat_get(fd).unwrap();
        let root = fs.fd_filestat_get(Fd::ROOT_PREOPEN).unwrap();
        for stat in [file, root] {
            assert_eq!(stat.access_time, EPOCH_2000_NS);
            assert_eq!(stat.modification_time, EPOCH_2000_NS);
            assert_eq!(stat.status_change_time, EPOCH_2000_NS);
        }
    }

    #[test]
    fn sysroot_and_src_mutations_are_always_rofs() {
        let mut fs = filesystem(false);
        let file = open_large(&mut fs);
        let directory_rights = readonly_rights();
        let src = fs
            .path_open(Fd::ROOT_PREOPEN, b"src", directory_rights, FdFlags::EMPTY)
            .unwrap();
        let sysroot = fs
            .path_open(
                Fd::ROOT_PREOPEN,
                b"sysroot",
                directory_rights,
                FdFlags::EMPTY,
            )
            .unwrap();

        assert_eq!(fs.fd_write(file, b"x"), Err(Errno::Rofs));
        assert_eq!(fs.path_create_file(src, b"new"), Err(Errno::Rofs));
        assert_eq!(fs.path_create_directory(sysroot, b"new"), Err(Errno::Rofs));
        assert_eq!(
            fs.path_rename(src, b"large", sysroot, b"large"),
            Err(Errno::Rofs)
        );
        assert_eq!(fs.path_unlink_file(src, b"large"), Err(Errno::Rofs));
        assert_eq!(
            fs.path_remove_directory(Fd::ROOT_PREOPEN, b"src"),
            Err(Errno::Rofs)
        );
        assert_eq!(
            fs.path_open(
                Fd::ROOT_PREOPEN,
                b"src/large",
                Rights::FD_WRITE,
                FdFlags::EMPTY
            ),
            Err(Errno::Rofs)
        );
    }
}
