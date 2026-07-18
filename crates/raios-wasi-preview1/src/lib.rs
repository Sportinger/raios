#![cfg_attr(not(test), no_std)]

//! Pure, dependency-free WASI preview1 filesystem core for raiOS.
//! It owns typed errno values, handles, rights, lexical paths, directory
//! snapshots, and the deterministic per-instance file-descriptor table.
//! It performs no I/O and knows no kernel, wasmi, mount, node, memory, or store
//! backend; later slices provide those adapters after validating guest memory.

extern crate alloc;

pub mod buildfs;
pub mod dir;
pub mod errno;
pub mod fd_table;
pub mod path;
pub mod readonly;
pub mod types;

pub use buildfs::{BuildFs, BuildFsManifestView};
pub use dir::Dirent;
pub use errno::Errno;
pub use fd_table::{intersect_rights, seek_offset, FdTable, PathOpenRequest, StandardNodes};
pub use path::{DirectoryEntry, DirectorySnapshot, NormalizedPath};
pub use readonly::{
    ChunkRead, ChunkReadError, ChunkReadRequest, Filestat, ReadOnlyFs, EPOCH_2000_NS,
};
pub use types::{Fd, FdEntry, FdFlags, FileType, NodeRef, PreopenType, Prestat, Rights, Whence};
