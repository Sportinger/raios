//! One descriptor, mount, filesystem, and process world for a build guest.

use alloc::{vec, vec::Vec};

use crate::dir::read_directory;
use crate::output_manifest::{freeze_output, OutputManifest};
use crate::ramfs::{writable_rights, RamFs, RamQuotas};
use crate::readonly::{
    filestat_buildfs, has_mutating_rights, read_buildfs_file, readonly_rights, resolve_buildfs,
};
use crate::writable::{ROOT_NAMESPACE, TMP_NAMESPACE};
use crate::{
    BuildFs, ChunkRead, DirectoryEntry, DirectorySnapshot, Dirent, Errno, Fd, FdFlags, FdTable,
    FileType, Filestat, JobContext, MountId, NodeRef, NormalizedPath, PathOpenRequest,
    ProcessContext, Rights, StandardNodes, Whence, EPOCH_2000_NS,
};

pub const ROOT_MOUNT: MountId = MountId::ROOT;
pub const SYSROOT_MOUNT: MountId = MountId(1);
pub const SOURCE_MOUNT: MountId = MountId(2);
pub const OUTPUT_MOUNT: MountId = MountId(3);
pub const TMP_MOUNT: MountId = MountId(4);
pub const ROOT_SCRATCH_MOUNT: MountId = MountId(5);

const ROOT_NODE: NodeRef = NodeRef(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MountKind {
    CompositeRoot,
    ReadOnlyBuildFs,
    Ram,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MountEntry {
    pub id: MountId,
    pub guest_path: &'static [u8],
    pub kind: MountKind,
}

/// Every guest-class-controlled filesystem limit enters through this value.
/// No policy quota is duplicated as a constant in the WASI layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasiBuildLimits {
    pub max_open_fds: u32,
    pub scratch: RamQuotas,
    pub output: RamQuotas,
}

impl WasiBuildLimits {
    pub const fn new(max_open_fds: u32, scratch: RamQuotas, output: RamQuotas) -> Self {
        Self {
            max_open_fds,
            scratch,
            output,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasiBuildInstance {
    limits: WasiBuildLimits,
    mounts: Vec<MountEntry>,
    fds: FdTable,
    sysroot: BuildFs,
    source: BuildFs,
    scratch: RamFs,
    output: RamFs,
    process: ProcessContext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResolvedNode {
    mount_id: MountId,
    node: NodeRef,
    file_type: FileType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MutationPath {
    mount_id: MountId,
    mount_root: NodeRef,
    relative: Vec<u8>,
}

impl WasiBuildInstance {
    pub fn new(
        sysroot: BuildFs,
        source: BuildFs,
        job: JobContext,
        limits: WasiBuildLimits,
    ) -> Result<Self, Errno> {
        let mut scratch = RamFs::new_arena(limits.scratch);
        scratch.create_namespace(TMP_NAMESPACE)?;
        scratch.create_namespace(ROOT_NAMESPACE)?;
        let output = RamFs::new_arena(limits.output);
        let root_rights = writable_rights();
        let fds = FdTable::new_with_root(
            limits.max_open_fds,
            StandardNodes {
                stdin: NodeRef(u64::MAX - 2),
                stdout: NodeRef(u64::MAX - 1),
                stderr: NodeRef(u64::MAX),
                root: ROOT_NODE,
            },
            ROOT_MOUNT,
            root_rights,
            root_rights,
        )?;
        let mounts = vec![
            MountEntry {
                id: ROOT_MOUNT,
                guest_path: b"/",
                kind: MountKind::CompositeRoot,
            },
            MountEntry {
                id: SYSROOT_MOUNT,
                guest_path: b"/sysroot",
                kind: MountKind::ReadOnlyBuildFs,
            },
            MountEntry {
                id: SOURCE_MOUNT,
                guest_path: b"/src",
                kind: MountKind::ReadOnlyBuildFs,
            },
            MountEntry {
                id: OUTPUT_MOUNT,
                guest_path: b"/out",
                kind: MountKind::Ram,
            },
            MountEntry {
                id: TMP_MOUNT,
                guest_path: b"/tmp",
                kind: MountKind::Ram,
            },
            MountEntry {
                id: ROOT_SCRATCH_MOUNT,
                guest_path: b"/",
                kind: MountKind::Ram,
            },
        ];
        Ok(Self {
            limits,
            mounts,
            fds,
            sysroot,
            source,
            scratch,
            output,
            process: ProcessContext::new(job),
        })
    }

    pub const fn limits(&self) -> WasiBuildLimits {
        self.limits
    }

    pub fn mounts(&self) -> &[MountEntry] {
        &self.mounts
    }

    pub fn fd_table(&self) -> &FdTable {
        &self.fds
    }

    pub fn process(&self) -> &ProcessContext {
        &self.process
    }

    pub fn process_mut(&mut self) -> &mut ProcessContext {
        &mut self.process
    }

    pub fn path_open(
        &mut self,
        parent_fd: Fd,
        path: &[u8],
        create: bool,
        requested_rights_base: Rights,
        requested_rights_inheriting: Rights,
        flags: FdFlags,
    ) -> Result<Fd, Errno> {
        let parent = self.directory_entry(parent_fd, Rights::PATH_OPEN)?;
        match self.resolve_existing(parent, path) {
            Ok(target) => {
                self.validate_open_policy(
                    target.mount_id,
                    requested_rights_base,
                    requested_rights_inheriting,
                    flags,
                )?;
                let (next_fds, fd) = self.open_resolved(
                    parent_fd,
                    target,
                    requested_rights_base,
                    requested_rights_inheriting,
                    flags,
                )?;
                self.fds = next_fds;
                Ok(fd)
            }
            Err(Errno::Noent) if create => {
                if !parent.rights_base.contains(Rights::PATH_CREATE_FILE) {
                    return Err(Errno::Notcapable);
                }
                let target = self.mutation_path(parent, path)?;
                self.validate_open_policy(
                    target.mount_id,
                    requested_rights_base,
                    requested_rights_inheriting,
                    flags,
                )?;
                self.create_and_open(
                    parent_fd,
                    target,
                    requested_rights_base,
                    requested_rights_inheriting,
                    flags,
                )
            }
            Err(error) => Err(error),
        }
    }

    pub fn fd_close(&mut self, fd: Fd) -> Result<(), Errno> {
        self.fds.close(fd)
    }

    pub fn fd_read(
        &mut self,
        fd: Fd,
        len: usize,
        reader: &mut impl ChunkRead,
    ) -> Result<Vec<u8>, Errno> {
        let entry = *self.fds.get(fd)?;
        require_right(entry.rights_base, Rights::FD_READ)?;
        if entry.file_type != FileType::RegularFile {
            return Err(Errno::Isdir);
        }
        let bytes = self.read_node(entry, entry.offset, len, reader)?;
        let next = entry
            .offset
            .checked_add(u64::try_from(bytes.len()).map_err(|_| Errno::Inval)?)
            .ok_or(Errno::Inval)?;
        self.fds.set_offset(fd, next)?;
        Ok(bytes)
    }

    pub fn fd_pread(
        &self,
        fd: Fd,
        offset: u64,
        len: usize,
        reader: &mut impl ChunkRead,
    ) -> Result<Vec<u8>, Errno> {
        let entry = *self.fds.get(fd)?;
        require_right(entry.rights_base, Rights::FD_READ | Rights::FD_SEEK)?;
        if entry.file_type != FileType::RegularFile {
            return Err(Errno::Isdir);
        }
        self.read_node(entry, offset, len, reader)
    }

    pub fn fd_write(&mut self, fd: Fd, bytes: &[u8]) -> Result<usize, Errno> {
        let entry = *self.fds.get(fd)?;
        if matches!(entry.mount_id, SYSROOT_MOUNT | SOURCE_MOUNT) {
            return Err(Errno::Rofs);
        }
        require_right(entry.rights_base, Rights::FD_WRITE)?;
        if entry.file_type != FileType::RegularFile {
            return Err(Errno::Isdir);
        }
        let offset = if entry.flags.bits() & FdFlags::APPEND.bits() != 0 {
            self.ram_node_size(entry.mount_id, entry.node)?
        } else {
            entry.offset
        };
        match entry.mount_id {
            OUTPUT_MOUNT => self.output.write_node(entry.node, offset, bytes)?,
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => self.scratch.write_node(entry.node, offset, bytes)?,
            SYSROOT_MOUNT | SOURCE_MOUNT => return Err(Errno::Rofs),
            _ => return Err(Errno::Badf),
        }
        let next = offset
            .checked_add(u64::try_from(bytes.len()).map_err(|_| Errno::Inval)?)
            .ok_or(Errno::Inval)?;
        self.fds.set_offset(fd, next)?;
        Ok(bytes.len())
    }

    pub fn fd_seek(&mut self, fd: Fd, delta: i64, whence: Whence) -> Result<u64, Errno> {
        let entry = *self.fds.get(fd)?;
        let size = self.node_size(entry.mount_id, entry.node)?;
        self.fds.seek(fd, delta, whence, size)
    }

    pub fn fd_tell(&self, fd: Fd) -> Result<u64, Errno> {
        let entry = self.fds.get(fd)?;
        require_right(entry.rights_base, Rights::FD_TELL)?;
        Ok(entry.offset)
    }

    pub fn fd_filestat_get(&self, fd: Fd) -> Result<Filestat, Errno> {
        let entry = *self.fds.get(fd)?;
        require_right(entry.rights_base, Rights::FD_FILESTAT_GET)?;
        self.filestat(entry.mount_id, entry.node, entry.file_type)
    }

    pub fn path_filestat_get(&self, parent_fd: Fd, path: &[u8]) -> Result<Filestat, Errno> {
        let parent = self.directory_entry(parent_fd, Rights::PATH_FILESTAT_GET)?;
        let target = self.resolve_existing(parent, path)?;
        self.filestat(target.mount_id, target.node, target.file_type)
    }

    pub fn fd_filestat_set_size(&mut self, fd: Fd, size: u64) -> Result<(), Errno> {
        let entry = *self.fds.get(fd)?;
        if matches!(entry.mount_id, SYSROOT_MOUNT | SOURCE_MOUNT) {
            return Err(Errno::Rofs);
        }
        require_right(entry.rights_base, Rights::FD_FILESTAT_SET_SIZE)?;
        match entry.mount_id {
            OUTPUT_MOUNT => self.output.resize_node(entry.node, size),
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => self.scratch.resize_node(entry.node, size),
            SYSROOT_MOUNT | SOURCE_MOUNT => Err(Errno::Rofs),
            _ => Err(Errno::Badf),
        }
    }

    pub fn fd_readdir(
        &self,
        fd: Fd,
        cookie: u64,
        max_entries: usize,
    ) -> Result<Vec<Dirent>, Errno> {
        let entry = *self.fds.get(fd)?;
        require_right(entry.rights_base, Rights::FD_READDIR)?;
        if entry.file_type != FileType::Directory {
            return Err(Errno::Notdir);
        }
        let snapshot = match entry.mount_id {
            ROOT_MOUNT => self.root_snapshot()?,
            SYSROOT_MOUNT => self.sysroot.snapshot(entry.node)?.clone(),
            SOURCE_MOUNT => self.source.snapshot(entry.node)?.clone(),
            OUTPUT_MOUNT => self.output.directory_snapshot(entry.node)?,
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => self.scratch.directory_snapshot(entry.node)?,
            _ => return Err(Errno::Badf),
        };
        read_directory(&snapshot, cookie, max_entries)
    }

    pub fn path_create_file(&mut self, parent_fd: Fd, path: &[u8]) -> Result<NodeRef, Errno> {
        self.path_create(parent_fd, path, false)
    }

    pub fn path_create_directory(&mut self, parent_fd: Fd, path: &[u8]) -> Result<NodeRef, Errno> {
        self.path_create(parent_fd, path, true)
    }

    pub fn path_rename(
        &mut self,
        source_parent_fd: Fd,
        source_path: &[u8],
        target_parent_fd: Fd,
        target_path: &[u8],
    ) -> Result<(), Errno> {
        let source_parent = self.directory_entry(source_parent_fd, Rights::PATH_RENAME_SOURCE)?;
        let target_parent = self.directory_entry(target_parent_fd, Rights::PATH_RENAME_TARGET)?;
        let source = self.mutation_path(source_parent, source_path)?;
        let target = self.mutation_path(target_parent, target_path)?;
        if source.mount_id != target.mount_id {
            return Err(Errno::Xdev);
        }
        match source.mount_id {
            OUTPUT_MOUNT => self.output.rename_at(
                source.mount_root,
                &source.relative,
                target.mount_root,
                &target.relative,
            ),
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => self.scratch.rename_at(
                source.mount_root,
                &source.relative,
                target.mount_root,
                &target.relative,
            ),
            SYSROOT_MOUNT | SOURCE_MOUNT => Err(Errno::Rofs),
            _ => Err(Errno::Perm),
        }
    }

    pub fn path_unlink_file(&mut self, parent_fd: Fd, path: &[u8]) -> Result<(), Errno> {
        let parent = self.directory_entry(parent_fd, Rights::PATH_UNLINK_FILE)?;
        let target = self.mutation_path(parent, path)?;
        match target.mount_id {
            OUTPUT_MOUNT => self
                .output
                .unlink_file_at(target.mount_root, &target.relative),
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => self
                .scratch
                .unlink_file_at(target.mount_root, &target.relative),
            SYSROOT_MOUNT | SOURCE_MOUNT => Err(Errno::Rofs),
            _ => Err(Errno::Perm),
        }
    }

    pub fn path_remove_directory(&mut self, parent_fd: Fd, path: &[u8]) -> Result<(), Errno> {
        let parent = self.directory_entry(parent_fd, Rights::PATH_REMOVE_DIRECTORY)?;
        let target = self.mutation_path(parent, path)?;
        match target.mount_id {
            OUTPUT_MOUNT => self
                .output
                .remove_directory_at(target.mount_root, &target.relative),
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => self
                .scratch
                .remove_directory_at(target.mount_root, &target.relative),
            SYSROOT_MOUNT | SOURCE_MOUNT => Err(Errno::Rofs),
            _ => Err(Errno::Perm),
        }
    }

    pub fn freeze_output(&self) -> Result<OutputManifest, Errno> {
        freeze_output(&self.output)
    }

    fn path_create(
        &mut self,
        parent_fd: Fd,
        path: &[u8],
        directory: bool,
    ) -> Result<NodeRef, Errno> {
        let right = if directory {
            Rights::PATH_CREATE_DIRECTORY
        } else {
            Rights::PATH_CREATE_FILE
        };
        let parent = self.directory_entry(parent_fd, right)?;
        let target = self.mutation_path(parent, path)?;
        match target.mount_id {
            OUTPUT_MOUNT => {
                self.output
                    .create_node_at(target.mount_root, &target.relative, directory)
            }
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => {
                self.scratch
                    .create_node_at(target.mount_root, &target.relative, directory)
            }
            SYSROOT_MOUNT | SOURCE_MOUNT => Err(Errno::Rofs),
            _ => Err(Errno::Perm),
        }
    }

    fn create_and_open(
        &mut self,
        parent_fd: Fd,
        target: MutationPath,
        requested_rights_base: Rights,
        requested_rights_inheriting: Rights,
        flags: FdFlags,
    ) -> Result<Fd, Errno> {
        match target.mount_id {
            OUTPUT_MOUNT => {
                let mut next_output = self.output.clone();
                let node =
                    next_output.create_node_at(target.mount_root, &target.relative, false)?;
                let resolved = ResolvedNode {
                    mount_id: target.mount_id,
                    node,
                    file_type: FileType::RegularFile,
                };
                let (next_fds, fd) = self.open_resolved(
                    parent_fd,
                    resolved,
                    requested_rights_base,
                    requested_rights_inheriting,
                    flags,
                )?;
                self.output = next_output;
                self.fds = next_fds;
                Ok(fd)
            }
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => {
                let mut next_scratch = self.scratch.clone();
                let node =
                    next_scratch.create_node_at(target.mount_root, &target.relative, false)?;
                let resolved = ResolvedNode {
                    mount_id: target.mount_id,
                    node,
                    file_type: FileType::RegularFile,
                };
                let (next_fds, fd) = self.open_resolved(
                    parent_fd,
                    resolved,
                    requested_rights_base,
                    requested_rights_inheriting,
                    flags,
                )?;
                self.scratch = next_scratch;
                self.fds = next_fds;
                Ok(fd)
            }
            SYSROOT_MOUNT | SOURCE_MOUNT => Err(Errno::Rofs),
            _ => Err(Errno::Perm),
        }
    }

    fn open_resolved(
        &self,
        parent_fd: Fd,
        target: ResolvedNode,
        requested_rights_base: Rights,
        requested_rights_inheriting: Rights,
        flags: FdFlags,
    ) -> Result<(FdTable, Fd), Errno> {
        let parent = self.fds.get(parent_fd)?;
        if parent.mount_id != ROOT_MOUNT && parent.mount_id != target.mount_id {
            return Err(Errno::Xdev);
        }
        let rights = mount_rights(target.mount_id)?;
        let mut next_fds = self.fds.clone();
        let fd = next_fds.path_open(PathOpenRequest {
            parent_fd,
            node: target.node,
            file_type: target.file_type,
            mount_id: target.mount_id,
            mount_rights_base: rights,
            mount_rights_inheriting: rights,
            requested_rights_base,
            requested_rights_inheriting,
            flags,
        })?;
        Ok((next_fds, fd))
    }

    fn validate_open_policy(
        &self,
        mount_id: MountId,
        rights_base: Rights,
        rights_inheriting: Rights,
        flags: FdFlags,
    ) -> Result<(), Errno> {
        if matches!(mount_id, SYSROOT_MOUNT | SOURCE_MOUNT)
            && (has_mutating_rights(rights_base)
                || has_mutating_rights(rights_inheriting)
                || flags != FdFlags::EMPTY)
        {
            return Err(Errno::Rofs);
        }
        Ok(())
    }

    fn directory_entry(&self, fd: Fd, right: Rights) -> Result<crate::FdEntry, Errno> {
        let entry = *self.fds.get(fd)?;
        if entry.file_type != FileType::Directory {
            return Err(Errno::Notdir);
        }
        require_right(entry.rights_base, right)?;
        Ok(entry)
    }

    fn resolve_existing(&self, parent: crate::FdEntry, path: &[u8]) -> Result<ResolvedNode, Errno> {
        if parent.mount_id == ROOT_MOUNT {
            if parent.node != ROOT_NODE {
                return Err(Errno::Badf);
            }
            let (mount_id, relative) = route_root(path)?;
            if mount_id == ROOT_MOUNT {
                return Ok(ResolvedNode {
                    mount_id,
                    node: ROOT_NODE,
                    file_type: FileType::Directory,
                });
            }
            return self.resolve_mount_path(mount_id, self.mount_root(mount_id)?, &relative);
        }
        self.resolve_mount_path(parent.mount_id, parent.node, path)
    }

    fn resolve_mount_path(
        &self,
        mount_id: MountId,
        base_node: NodeRef,
        path: &[u8],
    ) -> Result<ResolvedNode, Errno> {
        let node = match mount_id {
            SYSROOT_MOUNT => resolve_buildfs(&self.sysroot, base_node, path)?,
            SOURCE_MOUNT => resolve_buildfs(&self.source, base_node, path)?,
            OUTPUT_MOUNT => {
                let root = self.mount_root(mount_id)?;
                let relative = ram_relative_path(&self.output, root, base_node, path)?;
                self.output.resolve_from_node(root, &relative)?
            }
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => {
                let root = self.mount_root(mount_id)?;
                let relative = ram_relative_path(&self.scratch, root, base_node, path)?;
                self.scratch.resolve_from_node(root, &relative)?
            }
            _ => return Err(Errno::Badf),
        };
        let file_type = self.node_file_type(mount_id, node)?;
        Ok(ResolvedNode {
            mount_id,
            node,
            file_type,
        })
    }

    fn mutation_path(&self, parent: crate::FdEntry, path: &[u8]) -> Result<MutationPath, Errno> {
        let (mount_id, relative) = if parent.mount_id == ROOT_MOUNT {
            if parent.node != ROOT_NODE {
                return Err(Errno::Badf);
            }
            route_root(path)?
        } else {
            let root = self.mount_root(parent.mount_id)?;
            let relative = match parent.mount_id {
                SYSROOT_MOUNT => normalized_relative(self.sysroot.path(parent.node)?, b"", path)?,
                SOURCE_MOUNT => normalized_relative(self.source.path(parent.node)?, b"", path)?,
                OUTPUT_MOUNT => ram_relative_path(&self.output, root, parent.node, path)?,
                TMP_MOUNT | ROOT_SCRATCH_MOUNT => {
                    ram_relative_path(&self.scratch, root, parent.node, path)?
                }
                _ => return Err(Errno::Badf),
            };
            (parent.mount_id, relative)
        };
        if mount_id == ROOT_MOUNT || relative.is_empty() {
            return Err(Errno::Perm);
        }
        Ok(MutationPath {
            mount_id,
            mount_root: self.mount_root(mount_id)?,
            relative,
        })
    }

    fn mount_root(&self, mount_id: MountId) -> Result<NodeRef, Errno> {
        match mount_id {
            SYSROOT_MOUNT => Ok(self.sysroot.root_node()),
            SOURCE_MOUNT => Ok(self.source.root_node()),
            OUTPUT_MOUNT => Ok(self.output.root_node()),
            TMP_MOUNT => self.scratch.node_for_path(TMP_NAMESPACE),
            ROOT_SCRATCH_MOUNT => self.scratch.node_for_path(ROOT_NAMESPACE),
            ROOT_MOUNT => Ok(ROOT_NODE),
            _ => Err(Errno::Badf),
        }
    }

    fn read_node(
        &self,
        entry: crate::FdEntry,
        offset: u64,
        len: usize,
        reader: &mut impl ChunkRead,
    ) -> Result<Vec<u8>, Errno> {
        match entry.mount_id {
            SYSROOT_MOUNT => read_buildfs_file(
                &self.sysroot,
                entry.mount_id,
                entry.node,
                offset,
                len,
                reader,
            ),
            SOURCE_MOUNT => read_buildfs_file(
                &self.source,
                entry.mount_id,
                entry.node,
                offset,
                len,
                reader,
            ),
            OUTPUT_MOUNT => self.output.read_node(entry.node, offset, len),
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => self.scratch.read_node(entry.node, offset, len),
            _ => Err(Errno::Badf),
        }
    }

    fn node_file_type(&self, mount_id: MountId, node: NodeRef) -> Result<FileType, Errno> {
        match mount_id {
            ROOT_MOUNT => Ok(FileType::Directory),
            SYSROOT_MOUNT => self.sysroot.file_type(node),
            SOURCE_MOUNT => self.source.file_type(node),
            OUTPUT_MOUNT => self.output.node_file_type(node),
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => self.scratch.node_file_type(node),
            _ => Err(Errno::Badf),
        }
    }

    fn node_size(&self, mount_id: MountId, node: NodeRef) -> Result<u64, Errno> {
        match mount_id {
            ROOT_MOUNT => Ok(0),
            SYSROOT_MOUNT => self.sysroot.file(node).map(|file| file.len),
            SOURCE_MOUNT => self.source.file(node).map(|file| file.len),
            OUTPUT_MOUNT | TMP_MOUNT | ROOT_SCRATCH_MOUNT => self.ram_node_size(mount_id, node),
            _ => Err(Errno::Badf),
        }
    }

    fn ram_node_size(&self, mount_id: MountId, node: NodeRef) -> Result<u64, Errno> {
        match mount_id {
            OUTPUT_MOUNT => self.output.node_size(node),
            TMP_MOUNT | ROOT_SCRATCH_MOUNT => self.scratch.node_size(node),
            _ => Err(Errno::Badf),
        }
    }

    fn filestat(
        &self,
        mount_id: MountId,
        node: NodeRef,
        file_type: FileType,
    ) -> Result<Filestat, Errno> {
        match mount_id {
            SYSROOT_MOUNT => filestat_buildfs(&self.sysroot, mount_id, node),
            SOURCE_MOUNT => filestat_buildfs(&self.source, mount_id, node),
            ROOT_MOUNT | OUTPUT_MOUNT | TMP_MOUNT | ROOT_SCRATCH_MOUNT => Ok(Filestat {
                device: mount_id.0 as u64,
                inode: node.0,
                file_type,
                link_count: 1,
                size: self.node_size(mount_id, node)?,
                access_time: EPOCH_2000_NS,
                modification_time: EPOCH_2000_NS,
                status_change_time: EPOCH_2000_NS,
            }),
            _ => Err(Errno::Badf),
        }
    }

    fn root_snapshot(&self) -> Result<DirectorySnapshot, Errno> {
        let mut entries = vec![
            DirectoryEntry::new(
                b"sysroot",
                NodeRef(SYSROOT_MOUNT.0 as u64),
                FileType::Directory,
            )?,
            DirectoryEntry::new(b"src", NodeRef(SOURCE_MOUNT.0 as u64), FileType::Directory)?,
            DirectoryEntry::new(b"out", NodeRef(OUTPUT_MOUNT.0 as u64), FileType::Directory)?,
            DirectoryEntry::new(b"tmp", NodeRef(TMP_MOUNT.0 as u64), FileType::Directory)?,
        ];
        let root = self.mount_root(ROOT_SCRATCH_MOUNT)?;
        for entry in self.scratch.directory_snapshot(root)?.entries() {
            entries.push(DirectoryEntry::new(
                entry.name(),
                NodeRef(entry.node().0.checked_add(4).ok_or(Errno::Inval)?),
                entry.file_type(),
            )?);
        }
        DirectorySnapshot::new(entries)
    }
}

fn mount_rights(mount_id: MountId) -> Result<Rights, Errno> {
    match mount_id {
        SYSROOT_MOUNT | SOURCE_MOUNT => Ok(readonly_rights()),
        ROOT_MOUNT | OUTPUT_MOUNT | TMP_MOUNT | ROOT_SCRATCH_MOUNT => Ok(writable_rights()),
        _ => Err(Errno::Badf),
    }
}

fn route_root(path: &[u8]) -> Result<(MountId, Vec<u8>), Errno> {
    let normalized = NormalizedPath::root().resolve(path)?;
    let components: Vec<&[u8]> = normalized.components().collect();
    let Some(first) = components.first() else {
        return Ok((ROOT_MOUNT, Vec::new()));
    };
    let (mount_id, rest) = match *first {
        b"sysroot" => (SYSROOT_MOUNT, &components[1..]),
        b"src" => (SOURCE_MOUNT, &components[1..]),
        b"out" => (OUTPUT_MOUNT, &components[1..]),
        b"tmp" => (TMP_MOUNT, &components[1..]),
        _ => (ROOT_SCRATCH_MOUNT, components.as_slice()),
    };
    Ok((mount_id, join_components(rest)))
}

fn ram_relative_path(
    arena: &RamFs,
    mount_root: NodeRef,
    base: NodeRef,
    path: &[u8],
) -> Result<Vec<u8>, Errno> {
    normalized_relative(
        &arena.path_for_node(base)?,
        &arena.path_for_node(mount_root)?,
        path,
    )
}

fn normalized_relative(base: &[u8], root: &[u8], path: &[u8]) -> Result<Vec<u8>, Errno> {
    let relative_base = if root.is_empty() {
        base
    } else if base == root {
        &b""[..]
    } else if base.starts_with(root) && base.get(root.len()) == Some(&b'/') {
        &base[root.len() + 1..]
    } else {
        return Err(Errno::Badf);
    };
    let base = NormalizedPath::root().resolve(relative_base)?;
    Ok(join_components(
        &base.resolve(path)?.components().collect::<Vec<_>>(),
    ))
}

fn join_components(components: &[&[u8]]) -> Vec<u8> {
    let mut path = Vec::new();
    for (index, component) in components.iter().enumerate() {
        if index != 0 {
            path.push(b'/');
        }
        path.extend_from_slice(component);
    }
    path
}

fn require_right(actual: Rights, required: Rights) -> Result<(), Errno> {
    if actual.contains(required) {
        Ok(())
    } else {
        Err(Errno::Notcapable)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::readonly::sha256;
    use crate::{BuildFsManifestView, ChunkReadError, ChunkReadRequest};

    struct OneFileManifest {
        path: &'static str,
        bytes: &'static [u8],
    }

    impl BuildFsManifestView for OneFileManifest {
        fn chunk_size(&self) -> u64 {
            64 * 1024
        }
        fn directory_count(&self) -> usize {
            0
        }
        fn directory_path(&self, _index: usize) -> Option<&str> {
            None
        }
        fn file_count(&self) -> usize {
            1
        }
        fn file_path(&self, file: usize) -> Option<&str> {
            (file == 0).then_some(self.path)
        }
        fn file_len(&self, file: usize) -> Option<u64> {
            (file == 0).then_some(self.bytes.len() as u64)
        }
        fn file_sha256(&self, file: usize) -> Option<[u8; 32]> {
            (file == 0).then(|| sha256(self.bytes))
        }
        fn chunk_count(&self, file: usize) -> Option<usize> {
            (file == 0).then_some(1)
        }
        fn chunk_len(&self, file: usize, chunk: usize) -> Option<u64> {
            (file == 0 && chunk == 0).then_some(self.bytes.len() as u64)
        }
        fn chunk_sha256(&self, file: usize, chunk: usize) -> Option<[u8; 32]> {
            (file == 0 && chunk == 0).then(|| sha256(self.bytes))
        }
    }

    struct Reader;

    impl ChunkRead for Reader {
        fn read_chunk(
            &mut self,
            request: ChunkReadRequest,
            destination: &mut [u8],
        ) -> Result<(), ChunkReadError> {
            let bytes = match request.mount_id {
                SYSROOT_MOUNT => b"sys".as_slice(),
                SOURCE_MOUNT => b"src".as_slice(),
                _ => return Err(ChunkReadError::NotCapable),
            };
            destination.copy_from_slice(bytes);
            Ok(())
        }
    }

    fn instance() -> WasiBuildInstance {
        let sysroot = BuildFs::project(&OneFileManifest {
            path: "tool",
            bytes: b"sys",
        })
        .unwrap();
        let source = BuildFs::project(&OneFileManifest {
            path: "main.rs",
            bytes: b"src",
        })
        .unwrap();
        let job = JobContext::new(vec![b"rustc".to_vec()], Vec::new(), [7; 32]).unwrap();
        let quotas = RamQuotas::new(1024, 16, 16, 1024);
        WasiBuildInstance::new(
            sysroot,
            source,
            job,
            WasiBuildLimits::new(32, quotas, quotas),
        )
        .unwrap()
    }

    fn file_rights() -> Rights {
        Rights::FD_READ
            | Rights::FD_WRITE
            | Rights::FD_SEEK
            | Rights::FD_TELL
            | Rights::FD_FILESTAT_GET
            | Rights::FD_FILESTAT_SET_SIZE
    }

    #[test]
    fn one_fd_table_routes_all_mounts_with_policy() {
        let mut instance = instance();
        let mut reader = Reader;

        let sysroot = instance
            .path_open(
                Fd::ROOT_PREOPEN,
                b"sysroot",
                false,
                readonly_rights(),
                readonly_rights(),
                FdFlags::EMPTY,
            )
            .unwrap();
        let sys_file = instance
            .path_open(
                sysroot,
                b"tool",
                false,
                Rights::FD_READ | Rights::FD_SEEK,
                Rights::EMPTY,
                FdFlags::EMPTY,
            )
            .unwrap();
        assert_eq!(instance.fd_read(sys_file, 3, &mut reader).unwrap(), b"sys");
        assert_eq!(instance.fd_write(sys_file, b"x"), Err(Errno::Rofs));

        let src_file = instance
            .path_open(
                Fd::ROOT_PREOPEN,
                b"src/main.rs",
                false,
                Rights::FD_READ | Rights::FD_SEEK,
                Rights::EMPTY,
                FdFlags::EMPTY,
            )
            .unwrap();
        assert_eq!(
            instance.fd_pread(src_file, 0, 3, &mut reader).unwrap(),
            b"src"
        );

        let mut writable_fds = Vec::new();
        for (path, payload, mount_id) in [
            (b"out/artifact".as_slice(), b"out".as_slice(), OUTPUT_MOUNT),
            (b"tmp/cache".as_slice(), b"tmp".as_slice(), TMP_MOUNT),
            (b"work".as_slice(), b"root".as_slice(), ROOT_SCRATCH_MOUNT),
        ] {
            let fd = instance
                .path_open(
                    Fd::ROOT_PREOPEN,
                    path,
                    true,
                    file_rights(),
                    Rights::EMPTY,
                    FdFlags::EMPTY,
                )
                .unwrap();
            assert_eq!(instance.fd_table().get(fd).unwrap().mount_id, mount_id);
            instance.fd_write(fd, payload).unwrap();
            instance.fd_seek(fd, 0, Whence::Set).unwrap();
            assert_eq!(
                instance.fd_read(fd, payload.len(), &mut reader).unwrap(),
                payload
            );
            writable_fds.push(fd);
        }

        assert_eq!(instance.fd_table().open_count(), 7 + writable_fds.len());
        assert_eq!(instance.freeze_output().unwrap().files[0].path, "artifact");
        assert_eq!(instance.process().args_sizes_get().unwrap().count, 1);
    }

    #[test]
    fn cross_mount_escape_and_reserved_replacement_are_atomic() {
        let mut instance = instance();
        let sysroot = instance
            .path_open(
                Fd::ROOT_PREOPEN,
                b"sysroot",
                false,
                readonly_rights(),
                readonly_rights(),
                FdFlags::EMPTY,
            )
            .unwrap();
        let before_escape = instance.clone();
        assert_eq!(
            instance.path_open(
                sysroot,
                b"../out",
                false,
                readonly_rights(),
                Rights::EMPTY,
                FdFlags::EMPTY,
            ),
            Err(Errno::Notcapable)
        );
        assert_eq!(instance, before_escape);

        let tmp = instance
            .path_open(
                Fd::ROOT_PREOPEN,
                b"tmp",
                false,
                writable_rights(),
                writable_rights(),
                FdFlags::EMPTY,
            )
            .unwrap();
        let before_tmp_escape = instance.clone();
        assert_eq!(
            instance.path_open(
                tmp,
                b"../scratch",
                false,
                Rights::FD_READ,
                Rights::EMPTY,
                FdFlags::EMPTY,
            ),
            Err(Errno::Notcapable)
        );
        assert_eq!(instance, before_tmp_escape);

        let before_reserved = instance.clone();
        assert_eq!(
            instance.path_create_file(Fd::ROOT_PREOPEN, b"out"),
            Err(Errno::Perm)
        );
        assert_eq!(instance, before_reserved);

        instance
            .path_create_file(Fd::ROOT_PREOPEN, b"scratch")
            .unwrap();
        let before_reserved_rename = instance.clone();
        assert_eq!(
            instance.path_rename(Fd::ROOT_PREOPEN, b"scratch", Fd::ROOT_PREOPEN, b"out",),
            Err(Errno::Perm)
        );
        assert_eq!(instance, before_reserved_rename);

        let before_rename = instance.clone();
        assert_eq!(
            instance.path_rename(
                Fd::ROOT_PREOPEN,
                b"scratch",
                Fd::ROOT_PREOPEN,
                b"out/replaced",
            ),
            Err(Errno::Xdev)
        );
        assert_eq!(instance, before_rename);
    }
}
