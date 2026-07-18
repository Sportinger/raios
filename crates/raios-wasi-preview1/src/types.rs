use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use crate::Errno;

/// A descriptor number in one WASI instance's private table.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Fd(pub u32);

impl Fd {
    pub const STDIN: Self = Self(0);
    pub const STDOUT: Self = Self(1);
    pub const STDERR: Self = Self(2);
    pub const ROOT_PREOPEN: Self = Self(3);
    pub const FIRST_DYNAMIC: Self = Self(4);
}

/// An opaque reference to a node owned by a backend supplied in a later slice.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodeRef(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FileType {
    Unknown = 0,
    BlockDevice = 1,
    CharacterDevice = 2,
    Directory = 3,
    RegularFile = 4,
    SocketDgram = 5,
    SocketStream = 6,
    SymbolicLink = 7,
}

/// Preview1 rights bits. Unknown bits are rejected at the checked constructor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rights(u64);

impl Rights {
    pub const FD_DATASYNC: Self = Self(1 << 0);
    pub const FD_READ: Self = Self(1 << 1);
    pub const FD_SEEK: Self = Self(1 << 2);
    pub const FD_FDSTAT_SET_FLAGS: Self = Self(1 << 3);
    pub const FD_SYNC: Self = Self(1 << 4);
    pub const FD_TELL: Self = Self(1 << 5);
    pub const FD_WRITE: Self = Self(1 << 6);
    pub const FD_ADVISE: Self = Self(1 << 7);
    pub const FD_ALLOCATE: Self = Self(1 << 8);
    pub const PATH_CREATE_DIRECTORY: Self = Self(1 << 9);
    pub const PATH_CREATE_FILE: Self = Self(1 << 10);
    pub const PATH_LINK_SOURCE: Self = Self(1 << 11);
    pub const PATH_LINK_TARGET: Self = Self(1 << 12);
    pub const PATH_OPEN: Self = Self(1 << 13);
    pub const FD_READDIR: Self = Self(1 << 14);
    pub const PATH_READLINK: Self = Self(1 << 15);
    pub const PATH_RENAME_SOURCE: Self = Self(1 << 16);
    pub const PATH_RENAME_TARGET: Self = Self(1 << 17);
    pub const PATH_FILESTAT_GET: Self = Self(1 << 18);
    pub const PATH_FILESTAT_SET_SIZE: Self = Self(1 << 19);
    pub const PATH_FILESTAT_SET_TIMES: Self = Self(1 << 20);
    pub const FD_FILESTAT_GET: Self = Self(1 << 21);
    pub const FD_FILESTAT_SET_SIZE: Self = Self(1 << 22);
    pub const FD_FILESTAT_SET_TIMES: Self = Self(1 << 23);
    pub const PATH_SYMLINK: Self = Self(1 << 24);
    pub const PATH_REMOVE_DIRECTORY: Self = Self(1 << 25);
    pub const PATH_UNLINK_FILE: Self = Self(1 << 26);
    pub const POLL_FD_READWRITE: Self = Self(1 << 27);
    pub const SOCK_SHUTDOWN: Self = Self(1 << 28);
    pub const SOCK_ACCEPT: Self = Self(1 << 29);
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self((1 << 30) - 1);

    pub const fn from_bits(bits: u64) -> Result<Self, Errno> {
        if bits & !Self::ALL.0 == 0 {
            Ok(Self(bits))
        } else {
            Err(Errno::Inval)
        }
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn contains(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl BitAnd for Rights {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl BitAndAssign for Rights {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersection(rhs);
    }
}

impl BitOr for Rights {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Rights {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Validated preview1 descriptor flags.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FdFlags(u16);

impl FdFlags {
    pub const APPEND: Self = Self(1 << 0);
    pub const DSYNC: Self = Self(1 << 1);
    pub const NONBLOCK: Self = Self(1 << 2);
    pub const RSYNC: Self = Self(1 << 3);
    pub const SYNC: Self = Self(1 << 4);
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self((1 << 5) - 1);

    pub const fn from_bits(bits: u16) -> Result<Self, Errno> {
        if bits & !Self::ALL.0 == 0 {
            Ok(Self(bits))
        } else {
            Err(Errno::Inval)
        }
    }

    pub const fn bits(self) -> u16 {
        self.0
    }
}

impl BitOr for FdFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FdEntry {
    pub node: NodeRef,
    pub offset: u64,
    pub rights: Rights,
    pub flags: FdFlags,
    pub file_type: FileType,
}

impl FdEntry {
    pub const fn new(node: NodeRef, rights: Rights, flags: FdFlags, file_type: FileType) -> Self {
        Self {
            node,
            offset: 0,
            rights,
            flags,
            file_type,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Whence {
    Set,
    Current,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreopenType {
    Directory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Prestat {
    pub kind: PreopenType,
    pub name_len: u32,
}
