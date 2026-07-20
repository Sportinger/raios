#![cfg_attr(not(test), no_std)]

//! Pure, dependency-free WASI preview1 shim core for raiOS.
//! It owns typed process inputs/effects, deterministic clocks/randomness,
//! errno values, guest-range validation, handles, split rights, lexical paths,
//! deterministic filesystems, and the unified per-build FD/mount/process world.
//! It performs no ambient I/O and has no kernel, wasmi, or external dependency;
//! the next slice supplies only validated guest-memory and chunk-store adapters.

extern crate alloc;

pub mod buildfs;
pub mod determinism;
pub mod dir;
pub mod errno;
pub mod fd_table;
pub mod guest_range;
pub mod instance;
pub mod output_manifest;
pub mod path;
pub mod process;
pub mod ramfs;
pub mod readonly;
pub mod thread_host;
pub mod types;
pub mod writable;

pub use buildfs::{BuildFs, BuildFsManifestView};
pub use determinism::{
    clock_time_get, monotonic_ns, realtime_ns, ClockId, DeterministicRandom, FUEL_NS,
    MAX_RANDOM_GET,
};
pub use dir::Dirent;
pub use errno::Errno;
pub use fd_table::{intersect_rights, seek_offset, FdTable, PathOpenRequest, StandardNodes};
pub use guest_range::{
    checked_aligned_range, checked_iovec_list, checked_iovecs, checked_range, CheckedIovecs,
    GuestFault, GuestIovec, GuestRange,
};
pub use instance::{
    MountEntry, MountKind, WasiBuildInstance, WasiBuildLimits, OUTPUT_MOUNT, ROOT_MOUNT,
    ROOT_SCRATCH_MOUNT, SOURCE_MOUNT, SYSROOT_MOUNT, TMP_MOUNT,
};
pub use path::{DirectoryEntry, DirectorySnapshot, NormalizedPath};
pub use process::{
    ClockSubscription, ClockTimeout, HostEffect, JobContext, PollEvent, PollResult, ProcessContext,
    ProcessError, SerializedStrings, StringListSizes, Subscription,
};
pub use readonly::{
    ChunkRead, ChunkReadError, ChunkReadRequest, Filestat, ReadOnlyFs, EPOCH_2000_NS,
};
pub use thread_host::ThreadHost;
pub use types::{
    Fd, FdEntry, FdFlags, FileType, MountId, NodeRef, PreopenType, Prestat, Rights, Whence,
};
