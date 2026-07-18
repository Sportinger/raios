use alloc::{vec, vec::Vec};
use core::cmp::{max, min};

use crate::dir::read_directory;
use crate::{
    DirectoryEntry, DirectorySnapshot, Dirent, Errno, Fd, FdFlags, FdTable, FileType, NodeRef,
    NormalizedPath, PathOpenRequest, Rights, StandardNodes, Whence,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RamQuotas {
    pub max_bytes: u64,
    pub max_files: u64,
    pub max_directories: u64,
    pub max_file_size: u64,
}

impl RamQuotas {
    pub const fn new(
        max_bytes: u64,
        max_files: u64,
        max_directories: u64,
        max_file_size: u64,
    ) -> Self {
        Self {
            max_bytes,
            max_files,
            max_directories,
            max_file_size,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RamContent {
    Directory,
    File(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RamNode {
    parent: Option<NodeRef>,
    name: Vec<u8>,
    quota_counted: bool,
    content: RamContent,
}

impl RamNode {
    fn file_type(&self) -> FileType {
        match self.content {
            RamContent::Directory => FileType::Directory,
            RamContent::File(_) => FileType::RegularFile,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RamSnapshotContent {
    Directory,
    File(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RamSnapshotEntry {
    pub path: Vec<u8>,
    pub content: RamSnapshotContent,
}

/// One deterministic, volatile filesystem arena.
///
/// Node numbers are append-only indices. Removing a node leaves a tombstone,
/// so an identical mutation sequence always produces identical node numbers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RamFs {
    quotas: RamQuotas,
    nodes: Vec<Option<RamNode>>,
    bytes: u64,
    files: u64,
    directories: u64,
    fds: FdTable,
    offsets: Vec<Option<u64>>,
}

impl RamFs {
    pub fn new(quotas: RamQuotas, fd_limit: u32) -> Result<Self, Errno> {
        let root = RamNode {
            parent: None,
            name: Vec::new(),
            quota_counted: false,
            content: RamContent::Directory,
        };
        let fds = FdTable::new(
            fd_limit,
            StandardNodes {
                stdin: NodeRef(u64::MAX - 2),
                stdout: NodeRef(u64::MAX - 1),
                stderr: NodeRef(u64::MAX),
                root: NodeRef(0),
            },
            writable_rights(),
        )?;
        Ok(Self {
            quotas,
            nodes: vec![Some(root)],
            bytes: 0,
            files: 0,
            directories: 0,
            fds,
            offsets: vec![Some(0); Fd::FIRST_DYNAMIC.0 as usize],
        })
    }

    pub const fn root_node(&self) -> NodeRef {
        NodeRef(0)
    }

    pub fn fd_table(&self) -> &FdTable {
        &self.fds
    }

    pub const fn byte_len(&self) -> u64 {
        self.bytes
    }

    pub const fn file_count(&self) -> u64 {
        self.files
    }

    pub const fn directory_count(&self) -> u64 {
        self.directories
    }

    /// Creates an implementation-only directory that is excluded from guest quotas.
    pub(crate) fn create_namespace(&mut self, name: &[u8]) -> Result<NodeRef, Errno> {
        validate_name(name)?;
        if self.find_child(NodeRef(0), name).is_some() {
            return Err(Errno::Exist);
        }
        self.push_node(RamNode {
            parent: Some(NodeRef(0)),
            name: name.to_vec(),
            quota_counted: false,
            content: RamContent::Directory,
        })
    }

    pub fn path_create_file(&mut self, parent_fd: Fd, path: &[u8]) -> Result<NodeRef, Errno> {
        self.create_node(parent_fd, path, false)
    }

    pub fn path_create_directory(&mut self, parent_fd: Fd, path: &[u8]) -> Result<NodeRef, Errno> {
        self.create_node(parent_fd, path, true)
    }

    pub fn path_open(
        &mut self,
        parent_fd: Fd,
        path: &[u8],
        create: bool,
        requested_rights: Rights,
        flags: FdFlags,
    ) -> Result<Fd, Errno> {
        let parent = *self.fds.get(parent_fd)?;
        require_directory_right(&parent, Rights::PATH_OPEN)?;
        let existing = self.resolve_from_node(parent.node, path);
        let (node, file_type, pending_create) = match existing {
            Ok(node) => (node, self.node(node)?.file_type(), None),
            Err(Errno::Noent) if create => {
                let (directory, name) = self.resolve_parent(parent.node, path)?;
                self.check_file_quota()?;
                let node = self.next_node_ref()?;
                (node, FileType::RegularFile, Some((directory, name)))
            }
            Err(error) => return Err(error),
        };

        let mut next_fds = self.fds.clone();
        let fd = next_fds.path_open(PathOpenRequest {
            parent_fd,
            node,
            file_type,
            mount_rights: writable_rights(),
            requested_rights,
            flags,
        })?;

        if let Some((directory, name)) = pending_create {
            let created = self.push_node(RamNode {
                parent: Some(directory),
                name,
                quota_counted: true,
                content: RamContent::File(Vec::new()),
            })?;
            debug_assert_eq!(created, node);
            self.files += 1;
        }
        self.fds = next_fds;
        self.install_offset(fd, 0);
        Ok(fd)
    }

    pub fn fd_close(&mut self, fd: Fd) -> Result<(), Errno> {
        self.fds.close(fd)?;
        if let Some(offset) = self.offsets.get_mut(fd.0 as usize) {
            *offset = None;
        }
        Ok(())
    }

    pub fn fd_read(&mut self, fd: Fd, len: usize) -> Result<Vec<u8>, Errno> {
        let entry = *self.fds.get(fd)?;
        require_right(entry.rights, Rights::FD_READ)?;
        let offset = self.offset(fd)?;
        let bytes = self.read_node(entry.node, offset, len)?;
        let next = offset
            .checked_add(u64::try_from(bytes.len()).map_err(|_| Errno::Inval)?)
            .ok_or(Errno::Inval)?;
        self.set_offset(fd, next)?;
        Ok(bytes)
    }

    pub fn fd_pread(&self, fd: Fd, offset: u64, len: usize) -> Result<Vec<u8>, Errno> {
        let entry = self.fds.get(fd)?;
        require_right(entry.rights, Rights::FD_READ)?;
        self.read_node(entry.node, offset, len)
    }

    pub fn fd_write(&mut self, fd: Fd, bytes: &[u8]) -> Result<usize, Errno> {
        let entry = *self.fds.get(fd)?;
        require_right(entry.rights, Rights::FD_WRITE)?;
        let offset = if entry.flags.bits() & FdFlags::APPEND.bits() != 0 {
            self.file_data(entry.node)?.len() as u64
        } else {
            self.offset(fd)?
        };
        self.write_node(entry.node, offset, bytes)?;
        let next = offset
            .checked_add(u64::try_from(bytes.len()).map_err(|_| Errno::Inval)?)
            .ok_or(Errno::Inval)?;
        self.set_offset(fd, next)?;
        Ok(bytes.len())
    }

    pub fn fd_pwrite(&mut self, fd: Fd, offset: u64, bytes: &[u8]) -> Result<usize, Errno> {
        let entry = *self.fds.get(fd)?;
        require_right(entry.rights, Rights::FD_WRITE)?;
        self.write_node(entry.node, offset, bytes)?;
        Ok(bytes.len())
    }

    pub fn fd_seek(&mut self, fd: Fd, delta: i64, whence: Whence) -> Result<u64, Errno> {
        let entry = *self.fds.get(fd)?;
        require_right(entry.rights, Rights::FD_SEEK)?;
        let size = u64::try_from(self.file_data(entry.node)?.len()).map_err(|_| Errno::Inval)?;
        let target = crate::seek_offset(self.offset(fd)?, size, delta, whence)?;
        self.set_offset(fd, target)?;
        Ok(target)
    }

    pub fn filestat_set_size(&mut self, fd: Fd, size: u64) -> Result<(), Errno> {
        let entry = *self.fds.get(fd)?;
        require_right(entry.rights, Rights::FD_FILESTAT_SET_SIZE)?;
        self.resize_node(entry.node, size)
    }

    pub fn fd_readdir(
        &self,
        fd: Fd,
        cookie: u64,
        max_entries: usize,
    ) -> Result<Vec<Dirent>, Errno> {
        let entry = self.fds.get(fd)?;
        require_directory_right(entry, Rights::FD_READDIR)?;
        let snapshot = self.directory_snapshot(entry.node)?;
        read_directory(&snapshot, cookie, max_entries)
    }

    pub fn path_rename(
        &mut self,
        source_parent: Fd,
        source_path: &[u8],
        target_parent: Fd,
        target_path: &[u8],
    ) -> Result<(), Errno> {
        let source_parent = *self.fds.get(source_parent)?;
        let target_parent = *self.fds.get(target_parent)?;
        require_directory_right(&source_parent, Rights::PATH_RENAME_SOURCE)?;
        require_directory_right(&target_parent, Rights::PATH_RENAME_TARGET)?;
        let source = self.resolve_from_node(source_parent.node, source_path)?;
        if source == self.root_node() {
            return Err(Errno::Perm);
        }
        let (target_directory, target_name) =
            self.resolve_parent(target_parent.node, target_path)?;
        if self.is_descendant(target_directory, source)? {
            return Err(Errno::Inval);
        }
        let target = self.find_child(target_directory, &target_name);
        if target == Some(source) {
            return Ok(());
        }
        if let Some(target) = target {
            self.validate_replace(source, target)?;
        }
        if let Some(target) = target {
            self.remove_node(target);
        }
        let node = self.node_mut(source)?;
        node.parent = Some(target_directory);
        node.name = target_name;
        Ok(())
    }

    pub fn path_unlink_file(&mut self, parent_fd: Fd, path: &[u8]) -> Result<(), Errno> {
        let parent = *self.fds.get(parent_fd)?;
        require_directory_right(&parent, Rights::PATH_UNLINK_FILE)?;
        let node = self.resolve_from_node(parent.node, path)?;
        if matches!(self.node(node)?.content, RamContent::Directory) {
            return Err(Errno::Isdir);
        }
        self.remove_node(node);
        Ok(())
    }

    pub fn path_remove_directory(&mut self, parent_fd: Fd, path: &[u8]) -> Result<(), Errno> {
        let parent = *self.fds.get(parent_fd)?;
        require_directory_right(&parent, Rights::PATH_REMOVE_DIRECTORY)?;
        let node = self.resolve_from_node(parent.node, path)?;
        if node == self.root_node() {
            return Err(Errno::Perm);
        }
        if !matches!(self.node(node)?.content, RamContent::Directory) {
            return Err(Errno::Notdir);
        }
        if self.has_children(node) {
            return Err(Errno::Notempty);
        }
        self.remove_node(node);
        Ok(())
    }

    pub fn node_for_path(&self, path: &[u8]) -> Result<NodeRef, Errno> {
        self.resolve_from_node(self.root_node(), path)
    }

    pub fn write_path(&mut self, path: &[u8], offset: u64, bytes: &[u8]) -> Result<(), Errno> {
        let node = self.node_for_path(path)?;
        self.write_node(node, offset, bytes)
    }

    pub fn read_path(&self, path: &[u8]) -> Result<Vec<u8>, Errno> {
        let node = self.node_for_path(path)?;
        let len = self.file_data(node)?.len();
        self.read_node(node, 0, len)
    }

    pub fn directory_snapshot_for_path(&self, path: &[u8]) -> Result<DirectorySnapshot, Errno> {
        let node = self.node_for_path(path)?;
        self.directory_snapshot(node)
    }

    pub(crate) fn snapshot_entries(&self) -> Result<Vec<RamSnapshotEntry>, Errno> {
        let mut entries = Vec::new();
        for (index, slot) in self.nodes.iter().enumerate().skip(1) {
            let Some(node) = slot else { continue };
            if !node.quota_counted {
                continue;
            }
            let node_ref = NodeRef(u64::try_from(index).map_err(|_| Errno::Inval)?);
            let content = match &node.content {
                RamContent::Directory => RamSnapshotContent::Directory,
                RamContent::File(bytes) => RamSnapshotContent::File(bytes.clone()),
            };
            entries.push(RamSnapshotEntry {
                path: self.path_for_node(node_ref)?,
                content,
            });
        }
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(entries)
    }

    fn create_node(
        &mut self,
        parent_fd: Fd,
        path: &[u8],
        directory: bool,
    ) -> Result<NodeRef, Errno> {
        let parent = *self.fds.get(parent_fd)?;
        let right = if directory {
            Rights::PATH_CREATE_DIRECTORY
        } else {
            Rights::PATH_CREATE_FILE
        };
        require_directory_right(&parent, right)?;
        let (parent, name) = self.resolve_parent(parent.node, path)?;
        if self.find_child(parent, &name).is_some() {
            return Err(Errno::Exist);
        }
        if directory {
            self.check_directory_quota()?;
        } else {
            self.check_file_quota()?;
        }
        let node = self.push_node(RamNode {
            parent: Some(parent),
            name,
            quota_counted: true,
            content: if directory {
                RamContent::Directory
            } else {
                RamContent::File(Vec::new())
            },
        })?;
        if directory {
            self.directories += 1;
        } else {
            self.files += 1;
        }
        Ok(node)
    }

    fn write_node(&mut self, node: NodeRef, offset: u64, bytes: &[u8]) -> Result<(), Errno> {
        if bytes.is_empty() {
            self.file_data(node)?;
            return Ok(());
        }
        let old_len = u64::try_from(self.file_data(node)?.len()).map_err(|_| Errno::Inval)?;
        let write_len = u64::try_from(bytes.len()).map_err(|_| Errno::Inval)?;
        let new_len = max(old_len, offset.checked_add(write_len).ok_or(Errno::Fbig)?);
        self.check_resize(old_len, new_len)?;
        let start = usize::try_from(offset).map_err(|_| Errno::Fbig)?;
        let end = start.checked_add(bytes.len()).ok_or(Errno::Fbig)?;
        let file = self.file_data_mut(node)?;
        if file.len() < end {
            file.resize(end, 0);
        }
        file[start..end].copy_from_slice(bytes);
        self.bytes = self.bytes - old_len + new_len;
        Ok(())
    }

    fn resize_node(&mut self, node: NodeRef, size: u64) -> Result<(), Errno> {
        let old_len = u64::try_from(self.file_data(node)?.len()).map_err(|_| Errno::Inval)?;
        self.check_resize(old_len, size)?;
        let size = usize::try_from(size).map_err(|_| Errno::Fbig)?;
        self.file_data_mut(node)?.resize(size, 0);
        self.bytes = self.bytes - old_len + size as u64;
        Ok(())
    }

    fn check_resize(&self, old_len: u64, new_len: u64) -> Result<(), Errno> {
        if new_len > self.quotas.max_file_size || usize::try_from(new_len).is_err() {
            return Err(Errno::Fbig);
        }
        let next_total = self
            .bytes
            .checked_sub(old_len)
            .and_then(|value| value.checked_add(new_len))
            .ok_or(Errno::Nospc)?;
        if next_total > self.quotas.max_bytes {
            return Err(Errno::Nospc);
        }
        Ok(())
    }

    fn check_file_quota(&self) -> Result<(), Errno> {
        if self.files >= self.quotas.max_files {
            Err(Errno::Mfile)
        } else {
            Ok(())
        }
    }

    fn check_directory_quota(&self) -> Result<(), Errno> {
        if self.directories >= self.quotas.max_directories {
            Err(Errno::Mfile)
        } else {
            Ok(())
        }
    }

    fn resolve_parent(&self, base: NodeRef, path: &[u8]) -> Result<(NodeRef, Vec<u8>), Errno> {
        let base_path = self.path_for_node(base)?;
        let normalized = NormalizedPath::root().resolve(&base_path)?.resolve(path)?;
        let mut components: Vec<&[u8]> = normalized.components().collect();
        let name = components.pop().ok_or(Errno::Inval)?.to_vec();
        validate_name(&name)?;
        let parent = self.resolve_components(&components)?;
        if !matches!(self.node(parent)?.content, RamContent::Directory) {
            return Err(Errno::Notdir);
        }
        Ok((parent, name))
    }

    fn resolve_from_node(&self, base: NodeRef, path: &[u8]) -> Result<NodeRef, Errno> {
        let base_path = self.path_for_node(base)?;
        let normalized = NormalizedPath::root().resolve(&base_path)?.resolve(path)?;
        let components: Vec<&[u8]> = normalized.components().collect();
        self.resolve_components(&components)
    }

    fn resolve_components(&self, components: &[&[u8]]) -> Result<NodeRef, Errno> {
        let mut current = self.root_node();
        for component in components {
            if !matches!(self.node(current)?.content, RamContent::Directory) {
                return Err(Errno::Notdir);
            }
            current = self.find_child(current, component).ok_or(Errno::Noent)?;
        }
        Ok(current)
    }

    fn path_for_node(&self, node: NodeRef) -> Result<Vec<u8>, Errno> {
        let mut components = Vec::new();
        let mut current = node;
        loop {
            let candidate = self.node(current)?;
            let Some(parent) = candidate.parent else {
                break;
            };
            components.push(candidate.name.as_slice());
            current = parent;
        }
        components.reverse();
        let mut path = Vec::new();
        for (index, component) in components.iter().enumerate() {
            if index != 0 {
                path.push(b'/');
            }
            path.extend_from_slice(component);
        }
        Ok(path)
    }

    fn directory_snapshot(&self, directory: NodeRef) -> Result<DirectorySnapshot, Errno> {
        if !matches!(self.node(directory)?.content, RamContent::Directory) {
            return Err(Errno::Notdir);
        }
        let mut entries = Vec::new();
        for (index, slot) in self.nodes.iter().enumerate() {
            let Some(node) = slot else { continue };
            if node.parent == Some(directory) {
                entries.push(DirectoryEntry::new(
                    &node.name,
                    NodeRef(u64::try_from(index).map_err(|_| Errno::Inval)?),
                    node.file_type(),
                )?);
            }
        }
        DirectorySnapshot::new(entries)
    }

    fn validate_replace(&self, source: NodeRef, target: NodeRef) -> Result<(), Errno> {
        match (&self.node(source)?.content, &self.node(target)?.content) {
            (RamContent::File(_), RamContent::Directory) => Err(Errno::Isdir),
            (RamContent::Directory, RamContent::File(_)) => Err(Errno::Notdir),
            (RamContent::Directory, RamContent::Directory) if self.has_children(target) => {
                Err(Errno::Notempty)
            }
            _ => Ok(()),
        }
    }

    fn is_descendant(&self, mut candidate: NodeRef, ancestor: NodeRef) -> Result<bool, Errno> {
        loop {
            if candidate == ancestor {
                return Ok(true);
            }
            let Some(parent) = self.node(candidate)?.parent else {
                return Ok(false);
            };
            candidate = parent;
        }
    }

    fn has_children(&self, node: NodeRef) -> bool {
        self.nodes
            .iter()
            .flatten()
            .any(|candidate| candidate.parent == Some(node))
    }

    fn find_child(&self, parent: NodeRef, name: &[u8]) -> Option<NodeRef> {
        self.nodes.iter().enumerate().find_map(|(index, slot)| {
            let node = slot.as_ref()?;
            (node.parent == Some(parent) && node.name == name).then(|| NodeRef(index as u64))
        })
    }

    fn remove_node(&mut self, node: NodeRef) {
        let index = node.0 as usize;
        if let Some(removed) = self.nodes[index].take() {
            if removed.quota_counted {
                match removed.content {
                    RamContent::Directory => self.directories -= 1,
                    RamContent::File(bytes) => {
                        self.files -= 1;
                        self.bytes -= bytes.len() as u64;
                    }
                }
            }
        }
    }

    fn next_node_ref(&self) -> Result<NodeRef, Errno> {
        Ok(NodeRef(
            u64::try_from(self.nodes.len()).map_err(|_| Errno::Mfile)?,
        ))
    }

    fn push_node(&mut self, node: RamNode) -> Result<NodeRef, Errno> {
        let node_ref = self.next_node_ref()?;
        self.nodes.push(Some(node));
        Ok(node_ref)
    }

    fn node(&self, node: NodeRef) -> Result<&RamNode, Errno> {
        let index = usize::try_from(node.0).map_err(|_| Errno::Badf)?;
        self.nodes
            .get(index)
            .and_then(Option::as_ref)
            .ok_or(Errno::Badf)
    }

    fn node_mut(&mut self, node: NodeRef) -> Result<&mut RamNode, Errno> {
        let index = usize::try_from(node.0).map_err(|_| Errno::Badf)?;
        self.nodes
            .get_mut(index)
            .and_then(Option::as_mut)
            .ok_or(Errno::Badf)
    }

    fn file_data(&self, node: NodeRef) -> Result<&Vec<u8>, Errno> {
        match &self.node(node)?.content {
            RamContent::File(bytes) => Ok(bytes),
            RamContent::Directory => Err(Errno::Isdir),
        }
    }

    fn file_data_mut(&mut self, node: NodeRef) -> Result<&mut Vec<u8>, Errno> {
        match &mut self.node_mut(node)?.content {
            RamContent::File(bytes) => Ok(bytes),
            RamContent::Directory => Err(Errno::Isdir),
        }
    }

    fn read_node(&self, node: NodeRef, offset: u64, len: usize) -> Result<Vec<u8>, Errno> {
        let file = self.file_data(node)?;
        let start = min(usize::try_from(offset).unwrap_or(usize::MAX), file.len());
        let end = min(start.saturating_add(len), file.len());
        Ok(file[start..end].to_vec())
    }

    fn offset(&self, fd: Fd) -> Result<u64, Errno> {
        self.offsets
            .get(fd.0 as usize)
            .and_then(|slot| *slot)
            .ok_or(Errno::Badf)
    }

    fn set_offset(&mut self, fd: Fd, offset: u64) -> Result<(), Errno> {
        let slot = self.offsets.get_mut(fd.0 as usize).ok_or(Errno::Badf)?;
        if slot.is_none() {
            return Err(Errno::Badf);
        }
        *slot = Some(offset);
        Ok(())
    }

    fn install_offset(&mut self, fd: Fd, offset: u64) {
        let index = fd.0 as usize;
        if self.offsets.len() <= index {
            self.offsets.resize(index + 1, None);
        }
        self.offsets[index] = Some(offset);
    }
}

fn validate_name(name: &[u8]) -> Result<(), Errno> {
    DirectoryEntry::new(name, NodeRef(0), FileType::Unknown).map(|_| ())
}

fn require_right(actual: Rights, required: Rights) -> Result<(), Errno> {
    if actual.contains(required) {
        Ok(())
    } else {
        Err(Errno::Notcapable)
    }
}

fn require_directory_right(entry: &crate::FdEntry, right: Rights) -> Result<(), Errno> {
    if entry.file_type != FileType::Directory {
        return Err(Errno::Notdir);
    }
    require_right(entry.rights, right)
}

fn writable_rights() -> Rights {
    Rights::FD_READ
        | Rights::FD_SEEK
        | Rights::FD_TELL
        | Rights::FD_WRITE
        | Rights::FD_READDIR
        | Rights::FD_FILESTAT_GET
        | Rights::FD_FILESTAT_SET_SIZE
        | Rights::PATH_OPEN
        | Rights::PATH_CREATE_FILE
        | Rights::PATH_CREATE_DIRECTORY
        | Rights::PATH_RENAME_SOURCE
        | Rights::PATH_RENAME_TARGET
        | Rights::PATH_FILESTAT_GET
        | Rights::PATH_FILESTAT_SET_SIZE
        | Rights::PATH_UNLINK_FILE
        | Rights::PATH_REMOVE_DIRECTORY
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quotas() -> RamQuotas {
        RamQuotas::new(32, 4, 4, 16)
    }

    fn rw_rights() -> Rights {
        Rights::FD_READ | Rights::FD_WRITE | Rights::FD_SEEK | Rights::FD_FILESTAT_SET_SIZE
    }

    #[test]
    fn complete_lifecycle_and_identical_runs_have_identical_nodes_and_cookies() {
        fn run() -> (RamFs, NodeRef, Vec<Dirent>) {
            let mut fs = RamFs::new(quotas(), 16).unwrap();
            let dir = fs.path_create_directory(Fd::ROOT_PREOPEN, b"work").unwrap();
            let file = fs.path_create_file(Fd::ROOT_PREOPEN, b"work/a").unwrap();
            assert_eq!(file, NodeRef(2));
            let fd = fs
                .path_open(
                    Fd::ROOT_PREOPEN,
                    b"work/a",
                    false,
                    rw_rights(),
                    FdFlags::EMPTY,
                )
                .unwrap();
            fs.fd_write(fd, b"abcd").unwrap();
            fs.fd_pwrite(fd, 1, b"XY").unwrap();
            fs.fd_seek(fd, 0, Whence::Set).unwrap();
            assert_eq!(fs.fd_read(fd, 4).unwrap(), b"aXYd");
            fs.filestat_set_size(fd, 6).unwrap();
            assert_eq!(fs.fd_pread(fd, 0, 8).unwrap(), b"aXYd\0\0");
            fs.fd_close(fd).unwrap();
            fs.path_rename(Fd::ROOT_PREOPEN, b"work/a", Fd::ROOT_PREOPEN, b"work/b")
                .unwrap();
            let listing = fs.directory_snapshot_for_path(b"work").unwrap();
            let cookies = read_directory(&listing, 0, 8).unwrap();
            assert_eq!(cookies[0].next_cookie, 1);
            assert_eq!(fs.read_path(b"work/b").unwrap(), b"aXYd\0\0");
            (fs, NodeRef(dir.0), cookies)
        }
        let first = run();
        let second = run();
        assert_eq!(first, second);
        assert_eq!(first.1, NodeRef(1));
    }

    #[test]
    fn every_quota_fails_before_tree_and_fd_mutation() {
        let mut bytes = RamFs::new(RamQuotas::new(3, 2, 2, 8), 8).unwrap();
        bytes.path_create_file(Fd::ROOT_PREOPEN, b"a").unwrap();
        let fd = bytes
            .path_open(Fd::ROOT_PREOPEN, b"a", false, rw_rights(), FdFlags::EMPTY)
            .unwrap();
        bytes.fd_write(fd, b"abc").unwrap();
        let before = bytes.clone();
        assert_eq!(bytes.fd_write(fd, b"d"), Err(Errno::Nospc));
        assert_eq!(bytes, before);

        let mut size = RamFs::new(RamQuotas::new(20, 2, 2, 3), 8).unwrap();
        size.path_create_file(Fd::ROOT_PREOPEN, b"a").unwrap();
        let fd = size
            .path_open(Fd::ROOT_PREOPEN, b"a", false, rw_rights(), FdFlags::EMPTY)
            .unwrap();
        let before = size.clone();
        assert_eq!(size.fd_pwrite(fd, 3, b"x"), Err(Errno::Fbig));
        assert_eq!(size, before);

        let mut files = RamFs::new(RamQuotas::new(20, 1, 1, 20), 8).unwrap();
        files.path_create_file(Fd::ROOT_PREOPEN, b"a").unwrap();
        let before = files.clone();
        assert_eq!(
            files.path_create_file(Fd::ROOT_PREOPEN, b"b"),
            Err(Errno::Mfile)
        );
        assert_eq!(files, before);

        let mut directories = RamFs::new(RamQuotas::new(20, 1, 1, 20), 8).unwrap();
        directories
            .path_create_directory(Fd::ROOT_PREOPEN, b"a")
            .unwrap();
        let before = directories.clone();
        assert_eq!(
            directories.path_create_directory(Fd::ROOT_PREOPEN, b"b"),
            Err(Errno::Mfile)
        );
        assert_eq!(directories, before);
    }

    #[test]
    fn create_open_is_atomic_when_descriptor_table_is_full() {
        let mut fs = RamFs::new(quotas(), 4).unwrap();
        let before = fs.clone();
        assert_eq!(
            fs.path_open(Fd::ROOT_PREOPEN, b"new", true, rw_rights(), FdFlags::EMPTY),
            Err(Errno::Mfile)
        );
        assert_eq!(fs, before);
        assert_eq!(fs.node_for_path(b"new"), Err(Errno::Noent));
    }

    #[test]
    fn unlink_and_remove_directory_enforce_kind_and_empty_boundary() {
        let mut fs = RamFs::new(quotas(), 8).unwrap();
        fs.path_create_directory(Fd::ROOT_PREOPEN, b"d").unwrap();
        fs.path_create_file(Fd::ROOT_PREOPEN, b"d/f").unwrap();
        assert_eq!(
            fs.path_remove_directory(Fd::ROOT_PREOPEN, b"d"),
            Err(Errno::Notempty)
        );
        fs.path_unlink_file(Fd::ROOT_PREOPEN, b"d/f").unwrap();
        fs.path_remove_directory(Fd::ROOT_PREOPEN, b"d").unwrap();
        assert_eq!(fs.file_count(), 0);
        assert_eq!(fs.directory_count(), 0);
    }
}
