use alloc::vec::Vec;

use crate::{Errno, Fd, FdEntry, FdFlags, FileType, NodeRef, PreopenType, Prestat, Rights, Whence};

const ROOT_PREOPEN_NAME: &[u8] = b"/";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardNodes {
    pub stdin: NodeRef,
    pub stdout: NodeRef,
    pub stderr: NodeRef,
    pub root: NodeRef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PathOpenRequest {
    pub parent_fd: Fd,
    pub node: NodeRef,
    pub file_type: FileType,
    pub mount_rights: Rights,
    pub requested_rights: Rights,
    pub flags: FdFlags,
}

/// Computes the mandatory mount / parent descriptor / request attenuation.
pub const fn intersect_rights(
    mount_rights: Rights,
    parent_rights: Rights,
    requested_rights: Rights,
) -> Rights {
    mount_rights
        .intersection(parent_rights)
        .intersection(requested_rights)
}

/// Computes a seek target using an intentionally signed-i64 offset domain.
pub const fn seek_offset(
    current: u64,
    file_size: u64,
    delta: i64,
    whence: Whence,
) -> Result<u64, Errno> {
    let base = match whence {
        Whence::Set => 0,
        Whence::Current => current,
        Whence::End => file_size,
    };
    if base > i64::MAX as u64 {
        return Err(Errno::Inval);
    }
    match (base as i64).checked_add(delta) {
        Some(offset) if offset >= 0 => Ok(offset as u64),
        _ => Err(Errno::Inval),
    }
}

/// One WASI instance's deterministic descriptor table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FdTable {
    limit: u32,
    slots: Vec<Option<FdEntry>>,
}

impl FdTable {
    pub fn new(limit: u32, nodes: StandardNodes, root_rights: Rights) -> Result<Self, Errno> {
        if limit < Fd::FIRST_DYNAMIC.0 {
            return Err(Errno::Mfile);
        }
        let slots = alloc::vec![
            Some(FdEntry::new(
                nodes.stdin,
                Rights::FD_READ,
                FdFlags::EMPTY,
                FileType::CharacterDevice,
            )),
            Some(FdEntry::new(
                nodes.stdout,
                Rights::FD_WRITE,
                FdFlags::EMPTY,
                FileType::CharacterDevice,
            )),
            Some(FdEntry::new(
                nodes.stderr,
                Rights::FD_WRITE,
                FdFlags::EMPTY,
                FileType::CharacterDevice,
            )),
            Some(FdEntry::new(
                nodes.root,
                root_rights,
                FdFlags::EMPTY,
                FileType::Directory,
            )),
        ];
        Ok(Self { limit, slots })
    }

    pub fn get(&self, fd: Fd) -> Result<&FdEntry, Errno> {
        self.slots
            .get(fd.0 as usize)
            .and_then(Option::as_ref)
            .ok_or(Errno::Badf)
    }

    pub fn open_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_some()).count()
    }

    /// Installs an already-resolved node after all rights and capacity checks.
    pub fn path_open(&mut self, request: PathOpenRequest) -> Result<Fd, Errno> {
        let parent = self.get(request.parent_fd)?;
        if parent.file_type != FileType::Directory {
            return Err(Errno::Notdir);
        }
        if !parent.rights.contains(Rights::PATH_OPEN) {
            return Err(Errno::Notcapable);
        }
        let granted = intersect_rights(
            request.mount_rights,
            parent.rights,
            request.requested_rights,
        );
        if granted != request.requested_rights {
            return Err(Errno::Notcapable);
        }

        let slot = self.lowest_free_dynamic().ok_or(Errno::Mfile)?;
        let entry = FdEntry::new(request.node, granted, request.flags, request.file_type);
        if slot == self.slots.len() {
            self.slots.push(Some(entry));
        } else {
            self.slots[slot] = Some(entry);
        }
        Ok(Fd(slot as u32))
    }

    /// Closes dynamic descriptors. The four fixed descriptors cannot be displaced.
    pub fn close(&mut self, fd: Fd) -> Result<(), Errno> {
        if fd.0 < Fd::FIRST_DYNAMIC.0 {
            return Err(Errno::Perm);
        }
        let slot = self.slots.get_mut(fd.0 as usize).ok_or(Errno::Badf)?;
        if slot.take().is_none() {
            return Err(Errno::Badf);
        }
        Ok(())
    }

    /// Rights may only be removed; failed escalation leaves the entry untouched.
    pub fn set_rights(&mut self, fd: Fd, rights: Rights) -> Result<(), Errno> {
        let current = self.get(fd)?.rights;
        if !current.contains(rights) {
            return Err(Errno::Notcapable);
        }
        self.slots[fd.0 as usize]
            .as_mut()
            .ok_or(Errno::Badf)?
            .rights = rights;
        Ok(())
    }

    /// Updates an offset only after rights, type, and checked arithmetic succeed.
    pub fn seek(
        &mut self,
        fd: Fd,
        delta: i64,
        whence: Whence,
        file_size: u64,
    ) -> Result<u64, Errno> {
        let entry = self.get(fd)?;
        if entry.file_type == FileType::Directory {
            return Err(Errno::Isdir);
        }
        if !entry.rights.contains(Rights::FD_SEEK) {
            return Err(Errno::Notcapable);
        }
        let target = seek_offset(entry.offset, file_size, delta, whence)?;
        self.slots[fd.0 as usize]
            .as_mut()
            .ok_or(Errno::Badf)?
            .offset = target;
        Ok(target)
    }

    pub fn prestat_get(&self, fd: Fd) -> Result<Prestat, Errno> {
        if fd != Fd::ROOT_PREOPEN {
            return Err(Errno::Badf);
        }
        self.get(fd)?;
        Ok(Prestat {
            kind: PreopenType::Directory,
            name_len: ROOT_PREOPEN_NAME.len() as u32,
        })
    }

    pub fn prestat_dir_name(&self, fd: Fd) -> Result<&'static [u8], Errno> {
        self.prestat_get(fd)?;
        Ok(ROOT_PREOPEN_NAME)
    }

    fn lowest_free_dynamic(&self) -> Option<usize> {
        let first = Fd::FIRST_DYNAMIC.0 as usize;
        if let Some(relative) = self.slots[first..].iter().position(Option::is_none) {
            return first.checked_add(relative);
        }
        let next = self.slots.len();
        (next < self.limit as usize).then_some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::{intersect_rights, FdTable, PathOpenRequest, StandardNodes};
    use crate::{Errno, Fd, FdFlags, FileType, NodeRef, PreopenType, Rights, Whence};

    const ROOT_RIGHTS: Rights = Rights::ALL;

    fn table(limit: u32) -> FdTable {
        FdTable::new(
            limit,
            StandardNodes {
                stdin: NodeRef(10),
                stdout: NodeRef(11),
                stderr: NodeRef(12),
                root: NodeRef(13),
            },
            ROOT_RIGHTS,
        )
        .unwrap()
    }

    fn request(node: u64, rights: Rights) -> PathOpenRequest {
        PathOpenRequest {
            parent_fd: Fd::ROOT_PREOPEN,
            node: NodeRef(node),
            file_type: FileType::RegularFile,
            mount_rights: ROOT_RIGHTS,
            requested_rights: rights,
            flags: FdFlags::EMPTY,
        }
    }

    #[test]
    fn fixed_descriptors_and_single_root_preopen_are_modeled() {
        let table = table(8);
        assert_eq!(table.open_count(), 4);
        assert_eq!(table.get(Fd::STDIN).unwrap().node, NodeRef(10));
        assert_eq!(table.get(Fd::STDOUT).unwrap().node, NodeRef(11));
        assert_eq!(table.get(Fd::STDERR).unwrap().node, NodeRef(12));
        let prestat = table.prestat_get(Fd::ROOT_PREOPEN).unwrap();
        assert_eq!(prestat.kind, PreopenType::Directory);
        assert_eq!(prestat.name_len, 1);
        assert_eq!(table.prestat_dir_name(Fd::ROOT_PREOPEN), Ok(&b"/"[..]));
        assert_eq!(table.prestat_get(Fd::STDIN), Err(Errno::Badf));
    }

    #[test]
    fn dynamic_fds_are_lowest_free_and_reused() {
        let mut table = table(8);
        assert_eq!(table.path_open(request(20, Rights::FD_READ)), Ok(Fd(4)));
        assert_eq!(table.path_open(request(21, Rights::FD_READ)), Ok(Fd(5)));
        table.close(Fd(4)).unwrap();
        assert_eq!(table.path_open(request(22, Rights::FD_READ)), Ok(Fd(4)));
        assert_eq!(table.get(Fd(4)).unwrap().node, NodeRef(22));
    }

    #[test]
    fn fd_limit_failure_leaves_table_unchanged() {
        let mut table = table(6);
        table.path_open(request(20, Rights::FD_READ)).unwrap();
        table.path_open(request(21, Rights::FD_READ)).unwrap();
        let before = table.clone();
        assert_eq!(
            table.path_open(request(22, Rights::FD_READ)),
            Err(Errno::Mfile)
        );
        assert_eq!(table, before);
    }

    #[test]
    fn rights_are_triple_intersection_and_escalation_is_atomic() {
        let requested = Rights::FD_READ | Rights::FD_SEEK;
        let mount = Rights::FD_READ | Rights::FD_SEEK | Rights::FD_WRITE;
        let parent = Rights::PATH_OPEN | Rights::FD_READ | Rights::FD_SEEK;
        assert_eq!(intersect_rights(mount, parent, requested), requested);

        let mut table = FdTable::new(
            8,
            StandardNodes {
                stdin: NodeRef(1),
                stdout: NodeRef(2),
                stderr: NodeRef(3),
                root: NodeRef(4),
            },
            parent,
        )
        .unwrap();
        let mut open = request(30, requested);
        open.mount_rights = mount;
        assert_eq!(table.path_open(open), Ok(Fd(4)));
        assert_eq!(table.get(Fd(4)).unwrap().rights, requested);

        let before = table.clone();
        let mut escalation = request(31, Rights::FD_READ | Rights::FD_WRITE);
        escalation.mount_rights = mount;
        assert_eq!(table.path_open(escalation), Err(Errno::Notcapable));
        assert_eq!(table, before);
        assert_eq!(
            table.set_rights(Fd(4), requested | Rights::FD_WRITE),
            Err(Errno::Notcapable)
        );
        assert_eq!(table, before);
    }

    #[test]
    fn unknown_parent_fd_fails_before_allocation() {
        let mut table = table(8);
        let before = table.clone();
        let mut open = request(30, Rights::FD_READ);
        open.parent_fd = Fd(99);
        assert_eq!(table.path_open(open), Err(Errno::Badf));
        assert_eq!(table, before);
    }

    #[test]
    fn seek_i64_overflow_is_typed_and_offset_is_unchanged() {
        let mut table = table(8);
        let rights = Rights::FD_READ | Rights::FD_SEEK;
        let fd = table.path_open(request(20, rights)).unwrap();
        assert_eq!(
            table.seek(fd, i64::MAX, Whence::Set, 0),
            Ok(i64::MAX as u64)
        );
        let before = table.clone();
        assert_eq!(table.seek(fd, 1, Whence::Current, 0), Err(Errno::Inval));
        assert_eq!(table, before);
        assert_eq!(table.seek(fd, -1, Whence::Set, 0), Err(Errno::Inval));
        assert_eq!(table, before);
    }
}
