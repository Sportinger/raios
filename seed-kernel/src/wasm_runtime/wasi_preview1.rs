use alloc::vec::Vec;
use core::{fmt, ops::Range};

use raios_core::{
    sha256_bytes,
    thread_scheduler::{JobThreadScheduler, SchedulerError, ThreadId},
    wasi_build_output::FrozenOutput,
    wasi_preview1_import_abi::RUSTC_WASM_C6DCCF3E_IMPORTS,
};
use raios_wasi_preview1::{
    checked_aligned_range, checked_iovec_list, checked_iovecs, checked_range, CheckedIovecs,
    ClockSubscription, ClockTimeout, Errno, Fd, FdFlags, FileType, Filestat, GuestIovec,
    HostEffect, MountId, ProcessError, Rights, Subscription, WasiBuildInstance, Whence,
};
use wasmi::{core::Trap, Caller, Linker, Memory};

use super::wasi_build_storage::GrantedChunkReader;

const SUCCESS: i32 = 0;
const SUBSCRIPTION_SIZE: u32 = 48;
const EVENT_SIZE: u32 = 32;
const DIRENT_SIZE: usize = 24;
pub(crate) const WASI_IMPORT_CALL_COUNT: usize = 30;
pub(crate) const RECENT_WASI_CALL_COUNT: usize = 8;

const _: [(); WASI_IMPORT_CALL_COUNT] = [(); RUSTC_WASM_C6DCCF3E_IMPORTS.len()];

#[derive(Clone, Copy)]
#[repr(u8)]
enum WasiImportIndex {
    RandomGet = 0,
    ArgsGet = 1,
    ArgsSizesGet = 2,
    ClockTimeGet = 3,
    FdFilestatGet = 4,
    FdRead = 5,
    FdReaddir = 6,
    FdSeek = 7,
    FdWrite = 8,
    PathCreateDirectory = 9,
    PathFilestatGet = 10,
    PathLink = 11,
    PathOpen = 12,
    PathReadlink = 13,
    PathRemoveDirectory = 14,
    PathRename = 15,
    PathUnlinkFile = 16,
    PollOneoff = 17,
    SchedYield = 18,
    EnvironGet = 19,
    EnvironSizesGet = 20,
    FdClose = 21,
    FdFdstatGet = 22,
    FdFilestatSetSize = 23,
    FdPread = 24,
    FdPrestatGet = 25,
    FdPrestatDirName = 26,
    ProcExit = 27,
    ThreadSpawn = 28,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RecentWasiCall {
    pub(crate) import_index: u8,
    pub(crate) arg0: u64,
    pub(crate) arg1: u64,
    pub(crate) result_lo: i32,
}

impl RecentWasiCall {
    const EMPTY: Self = Self {
        import_index: 0,
        arg0: 0,
        arg1: 0,
        result_lo: 0,
    };
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum WasiEffectOpcode {
    RandomGet = 1,
    ClockTimeGet = 2,
    FdRead = 3,
    FdPread = 4,
    FdWrite = 5,
    FdSeek = 6,
    FdReaddir = 7,
    FdClose = 8,
    FdFilestatSetSize = 9,
    PathCreateDirectory = 16,
    PathFilestatGet = 17,
    PathLink = 18,
    PathOpen = 19,
    PathReadlink = 20,
    PathRemoveDirectory = 21,
    PathRename = 22,
    PathUnlinkFile = 23,
    PollOneoff = 24,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpawnRequest {
    pub(crate) thread: ThreadId,
    pub(crate) start_arg: i32,
}

pub(crate) struct ThreadWorld {
    pub(crate) scheduler: JobThreadScheduler,
    pub(crate) pending_spawns: Vec<SpawnRequest>,
    pub(crate) spawns: u32,
    pub(crate) cap_denials: u32,
    pub(crate) active_thread: Option<ThreadId>,
    pub(crate) active_fuel_checkpoint: Option<u64>,
    pub(crate) fuel_accounting_failed: bool,
    pub(crate) stdout: Vec<u8>,
}

impl ThreadWorld {
    pub(crate) fn new(thread_cap: u32) -> Self {
        Self {
            scheduler: JobThreadScheduler::with_thread_cap(thread_cap),
            pending_spawns: Vec::new(),
            spawns: 0,
            cap_denials: 0,
            active_thread: None,
            active_fuel_checkpoint: None,
            fuel_accounting_failed: false,
            stdout: Vec::new(),
        }
    }

    pub(crate) fn reserve_spawn(&mut self, start_arg: i32) -> i32 {
        match self.scheduler.spawn() {
            Ok(thread) => {
                self.pending_spawns.push(SpawnRequest { thread, start_arg });
                self.spawns = self.spawns.saturating_add(1);
                thread.get() as i32
            }
            Err(SchedulerError::ThreadCap { .. }) => {
                self.cap_denials = self.cap_denials.saturating_add(1);
                -1
            }
            Err(_) => -1,
        }
    }
}

pub(crate) enum ThreadHostMode {
    Deny,
    Scheduled(ThreadWorld),
}

pub(crate) struct WasiHostState {
    instance: WasiBuildInstance,
    memory: Option<Memory>,
    pub(crate) thread_mode: ThreadHostMode,
    chunk_reader: GrantedChunkReader,
    stdout_bytes: u64,
    stderr_bytes: u64,
    logical_fuel: u64,
    terminal_exit: Option<u32>,
    effect_digest: [u8; 32],
    effect_sequence: u64,
    import_call_counts: [u64; WASI_IMPORT_CALL_COUNT],
    recent_calls: [RecentWasiCall; RECENT_WASI_CALL_COUNT],
    recent_call_next: u8,
    recent_call_len: u8,
}

impl WasiHostState {
    pub(crate) fn new(
        instance: WasiBuildInstance,
        chunk_reader: GrantedChunkReader,
        thread_mode: ThreadHostMode,
    ) -> Self {
        Self {
            instance,
            memory: None,
            thread_mode,
            chunk_reader,
            stdout_bytes: 0,
            stderr_bytes: 0,
            logical_fuel: 0,
            terminal_exit: None,
            effect_digest: [0; 32],
            effect_sequence: 0,
            import_call_counts: [0; WASI_IMPORT_CALL_COUNT],
            recent_calls: [RecentWasiCall::EMPTY; RECENT_WASI_CALL_COUNT],
            recent_call_next: 0,
            recent_call_len: 0,
        }
    }

    pub(crate) fn install_memory(&mut self, memory: Memory) {
        self.memory = Some(memory);
    }

    pub(crate) const fn stdout_bytes(&self) -> u64 {
        self.stdout_bytes
    }

    pub(crate) const fn terminal_exit(&self) -> Option<u32> {
        self.terminal_exit
    }

    pub(crate) const fn effect_digest(&self) -> [u8; 32] {
        self.effect_digest
    }

    pub(crate) const fn effect_count(&self) -> u64 {
        self.effect_sequence
    }

    pub(crate) const fn import_call_counts(&self) -> &[u64; WASI_IMPORT_CALL_COUNT] {
        &self.import_call_counts
    }

    pub(crate) fn recent_calls(&self) -> [Option<RecentWasiCall>; RECENT_WASI_CALL_COUNT] {
        let mut snapshot = [None; RECENT_WASI_CALL_COUNT];
        let len = self.recent_call_len as usize;
        let first = if len == RECENT_WASI_CALL_COUNT {
            self.recent_call_next as usize
        } else {
            0
        };
        let mut index = 0usize;
        while index < len {
            snapshot[index] = Some(self.recent_calls[(first + index) % RECENT_WASI_CALL_COUNT]);
            index += 1;
        }
        snapshot
    }

    fn begin_import_call(&mut self, import: WasiImportIndex, arg0: u64, arg1: u64) -> usize {
        let import_index = import as usize;
        self.import_call_counts[import_index] =
            self.import_call_counts[import_index].saturating_add(1);
        let recent_index = self.recent_call_next as usize;
        self.recent_calls[recent_index] = RecentWasiCall {
            import_index: import as u8,
            arg0,
            arg1,
            result_lo: 0,
        };
        self.recent_call_next = ((recent_index + 1) % RECENT_WASI_CALL_COUNT) as u8;
        self.recent_call_len = self
            .recent_call_len
            .saturating_add(1)
            .min(RECENT_WASI_CALL_COUNT as u8);
        recent_index
    }

    fn finish_import_call(&mut self, recent_index: usize, result_lo: i32) {
        self.recent_calls[recent_index].result_lo = result_lo;
    }

    pub(crate) const fn is_scheduled_thread_job(&self) -> bool {
        matches!(self.thread_mode, ThreadHostMode::Scheduled(_))
    }

    pub(crate) fn scheduled_world(&self) -> Option<&ThreadWorld> {
        match &self.thread_mode {
            ThreadHostMode::Deny => None,
            ThreadHostMode::Scheduled(world) => Some(world),
        }
    }

    pub(crate) fn scheduled_world_mut(&mut self) -> Option<&mut ThreadWorld> {
        match &mut self.thread_mode {
            ThreadHostMode::Deny => None,
            ThreadHostMode::Scheduled(world) => Some(world),
        }
    }

    pub(crate) fn activate_thread(&mut self, thread: ThreadId, remaining: u64) -> Result<(), ()> {
        let world = self.scheduled_world_mut().ok_or(())?;
        if world.active_thread.is_some() || world.active_fuel_checkpoint.is_some() {
            return Err(());
        }
        world.active_thread = Some(thread);
        world.active_fuel_checkpoint = Some(remaining);
        Ok(())
    }

    pub(crate) fn sync_active_fuel(&mut self, remaining: u64) -> Result<(), ()> {
        let (logical_fuel, thread_mode) = (&mut self.logical_fuel, &mut self.thread_mode);
        let world = match thread_mode {
            ThreadHostMode::Deny => return Ok(()),
            ThreadHostMode::Scheduled(world) => world,
        };
        let Some(checkpoint) = world.active_fuel_checkpoint else {
            world.fuel_accounting_failed = true;
            return Err(());
        };
        let Some(consumed) = checkpoint.checked_sub(remaining) else {
            world.fuel_accounting_failed = true;
            return Err(());
        };
        let Some(next_logical_fuel) = logical_fuel.checked_add(consumed) else {
            world.fuel_accounting_failed = true;
            return Err(());
        };
        *logical_fuel = next_logical_fuel;
        world.active_fuel_checkpoint = Some(remaining);
        Ok(())
    }

    pub(crate) fn deactivate_thread(&mut self) -> Result<(), ()> {
        let world = self.scheduled_world_mut().ok_or(())?;
        if world.active_thread.take().is_none() || world.active_fuel_checkpoint.take().is_none() {
            world.fuel_accounting_failed = true;
            return Err(());
        }
        if world.fuel_accounting_failed {
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn freeze_output_evidence(&self) -> Result<(usize, FrozenOutput, u64), Errno> {
        let manifest = self.instance.freeze_output()?;
        let entries = manifest
            .directories
            .len()
            .checked_add(manifest.files.len())
            .ok_or(Errno::Fbig)?;
        let canonical_bytes = manifest.canonical_bytes();
        let logical_size = u64::try_from(canonical_bytes.len()).map_err(|_| Errno::Fbig)?;
        Ok((
            entries,
            FrozenOutput::from_manifest_bytes(&canonical_bytes),
            logical_size,
        ))
    }

    fn write_fd(&mut self, fd: Fd, bytes: &[u8]) -> Result<usize, Errno> {
        let entry = *self.instance.fd_table().get(fd)?;
        if entry.mount_id == MountId::STDIO {
            if !entry.rights_base.contains(Rights::FD_WRITE)
                || entry.file_type != FileType::CharacterDevice
            {
                return Err(Errno::Notcapable);
            }
            let count = u64::try_from(bytes.len()).map_err(|_| Errno::Fbig)?;
            match fd {
                Fd::STDOUT => {
                    self.stdout_bytes = self.stdout_bytes.checked_add(count).ok_or(Errno::Fbig)?;
                    if let ThreadHostMode::Scheduled(world) = &mut self.thread_mode {
                        world.stdout.extend_from_slice(bytes);
                    }
                }
                Fd::STDERR => {
                    self.stderr_bytes = self.stderr_bytes.checked_add(count).ok_or(Errno::Fbig)?;
                }
                _ => return Err(Errno::Badf),
            }
            return Ok(bytes.len());
        }
        self.instance.fd_write(fd, bytes)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ProcExitTrap;

impl fmt::Display for ProcExitTrap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WASI build job exited")
    }
}

impl wasmi::core::HostError for ProcExitTrap {}

pub(crate) fn define_wasi_imports(
    linker: &mut Linker<WasiHostState>,
    memory: Memory,
) -> Result<usize, ()> {
    let mut registered = 0usize;
    for declaration in RUSTC_WASM_C6DCCF3E_IMPORTS {
        let result = match (declaration.module, declaration.name) {
            ("wasi_snapshot_preview1", "random_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_random_get)
            }
            ("wasi_snapshot_preview1", "args_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_args_get)
            }
            ("wasi_snapshot_preview1", "args_sizes_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_args_sizes_get)
            }
            ("wasi_snapshot_preview1", "clock_time_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_clock_time_get)
            }
            ("wasi_snapshot_preview1", "fd_filestat_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_fd_filestat_get)
            }
            ("wasi_snapshot_preview1", "fd_read") => {
                linker.func_wrap(declaration.module, declaration.name, host_fd_read)
            }
            ("wasi_snapshot_preview1", "fd_readdir") => {
                linker.func_wrap(declaration.module, declaration.name, host_fd_readdir)
            }
            ("wasi_snapshot_preview1", "fd_seek") => {
                linker.func_wrap(declaration.module, declaration.name, host_fd_seek)
            }
            ("wasi_snapshot_preview1", "fd_write") => {
                linker.func_wrap(declaration.module, declaration.name, host_fd_write)
            }
            ("wasi_snapshot_preview1", "path_create_directory") => linker.func_wrap(
                declaration.module,
                declaration.name,
                host_path_create_directory,
            ),
            ("wasi_snapshot_preview1", "path_filestat_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_path_filestat_get)
            }
            ("wasi_snapshot_preview1", "path_link") => {
                linker.func_wrap(declaration.module, declaration.name, host_path_link)
            }
            ("wasi_snapshot_preview1", "path_open") => {
                linker.func_wrap(declaration.module, declaration.name, host_path_open)
            }
            ("wasi_snapshot_preview1", "path_readlink") => {
                linker.func_wrap(declaration.module, declaration.name, host_path_readlink)
            }
            ("wasi_snapshot_preview1", "path_remove_directory") => linker.func_wrap(
                declaration.module,
                declaration.name,
                host_path_remove_directory,
            ),
            ("wasi_snapshot_preview1", "path_rename") => {
                linker.func_wrap(declaration.module, declaration.name, host_path_rename)
            }
            ("wasi_snapshot_preview1", "path_unlink_file") => {
                linker.func_wrap(declaration.module, declaration.name, host_path_unlink_file)
            }
            ("wasi_snapshot_preview1", "poll_oneoff") => {
                linker.func_wrap(declaration.module, declaration.name, host_poll_oneoff)
            }
            ("wasi_snapshot_preview1", "sched_yield") => {
                linker.func_wrap(declaration.module, declaration.name, host_sched_yield)
            }
            ("wasi_snapshot_preview1", "environ_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_environ_get)
            }
            ("wasi_snapshot_preview1", "environ_sizes_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_environ_sizes_get)
            }
            ("wasi_snapshot_preview1", "fd_close") => {
                linker.func_wrap(declaration.module, declaration.name, host_fd_close)
            }
            ("wasi_snapshot_preview1", "fd_fdstat_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_fd_fdstat_get)
            }
            ("wasi_snapshot_preview1", "fd_filestat_set_size") => linker.func_wrap(
                declaration.module,
                declaration.name,
                host_fd_filestat_set_size,
            ),
            ("wasi_snapshot_preview1", "fd_pread") => {
                linker.func_wrap(declaration.module, declaration.name, host_fd_pread)
            }
            ("wasi_snapshot_preview1", "fd_prestat_get") => {
                linker.func_wrap(declaration.module, declaration.name, host_fd_prestat_get)
            }
            ("wasi_snapshot_preview1", "fd_prestat_dir_name") => linker.func_wrap(
                declaration.module,
                declaration.name,
                host_fd_prestat_dir_name,
            ),
            ("wasi_snapshot_preview1", "proc_exit") => {
                linker.func_wrap(declaration.module, declaration.name, host_proc_exit)
            }
            ("wasi", "thread-spawn") => {
                linker.func_wrap(declaration.module, declaration.name, host_thread_spawn)
            }
            ("env", "memory") => linker.define(declaration.module, declaration.name, memory),
            _ => return Err(()),
        };
        result.map_err(|_| ())?;
        registered = registered.checked_add(1).ok_or(())?;
    }
    Ok(registered)
}

fn host_random_get(mut caller: Caller<'_, WasiHostState>, ptr: i32, len: i32) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::RandomGet,
        ptr as u32 as u64,
        len as u32 as u64,
    );
    let ptr_bytes = ptr.to_le_bytes();
    let len_bytes = len.to_le_bytes();
    let argument_digest = effect_digest(WasiEffectOpcode::RandomGet, &[&ptr_bytes, &len_bytes]);
    let mut random_bytes = Vec::new();
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let range = byte_range(caller, ptr as u32, len as u32)?;
        let bytes = caller
            .data_mut()
            .instance
            .process_mut()
            .random_get(range.len())
            .map_err(process_errno)?;
        random_bytes.extend_from_slice(&bytes);
        write_validated(caller, range, &bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::RandomGet,
        argument_digest,
        errno,
        &[&random_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_args_sizes_get(
    mut caller: Caller<'_, WasiHostState>,
    count_ptr: i32,
    size_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::ArgsSizesGet,
        count_ptr as u32 as u64,
        size_ptr as u32 as u64,
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let count_range = aligned_range(caller, count_ptr as u32, 4, 4)?;
        let size_range = aligned_range(caller, size_ptr as u32, 4, 4)?;
        let sizes = caller
            .data()
            .instance
            .process()
            .args_sizes_get()
            .map_err(process_errno)?;
        write_validated(caller, count_range, &sizes.count.to_le_bytes())?;
        write_validated(caller, size_range, &sizes.buffer_size.to_le_bytes())
    });
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_args_get(mut caller: Caller<'_, WasiHostState>, pointers_ptr: i32, buffer_ptr: i32) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::ArgsGet,
        pointers_ptr as u32 as u64,
        buffer_ptr as u32 as u64,
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let sizes = caller
            .data()
            .instance
            .process()
            .args_sizes_get()
            .map_err(process_errno)?;
        write_serialized_strings(caller, pointers_ptr as u32, buffer_ptr as u32, sizes, true)
    });
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_environ_sizes_get(
    mut caller: Caller<'_, WasiHostState>,
    count_ptr: i32,
    size_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::EnvironSizesGet,
        count_ptr as u32 as u64,
        size_ptr as u32 as u64,
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let count_range = aligned_range(caller, count_ptr as u32, 4, 4)?;
        let size_range = aligned_range(caller, size_ptr as u32, 4, 4)?;
        let sizes = caller
            .data()
            .instance
            .process()
            .environ_sizes_get()
            .map_err(process_errno)?;
        write_validated(caller, count_range, &sizes.count.to_le_bytes())?;
        write_validated(caller, size_range, &sizes.buffer_size.to_le_bytes())
    });
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_environ_get(
    mut caller: Caller<'_, WasiHostState>,
    pointers_ptr: i32,
    buffer_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::EnvironGet,
        pointers_ptr as u32 as u64,
        buffer_ptr as u32 as u64,
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let sizes = caller
            .data()
            .instance
            .process()
            .environ_sizes_get()
            .map_err(process_errno)?;
        write_serialized_strings(caller, pointers_ptr as u32, buffer_ptr as u32, sizes, false)
    });
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn write_serialized_strings(
    caller: &mut Caller<'_, WasiHostState>,
    pointers_ptr: u32,
    buffer_ptr: u32,
    sizes: raios_wasi_preview1::StringListSizes,
    args: bool,
) -> Result<(), Errno> {
    let pointers_len = sizes.count.checked_mul(4).ok_or(Errno::Fault)?;
    let pointers_range = aligned_range(caller, pointers_ptr, pointers_len, 4)?;
    let buffer_range = byte_range(caller, buffer_ptr, sizes.buffer_size)?;
    let serialized = if args {
        caller
            .data()
            .instance
            .process()
            .args_get()
            .map_err(process_errno)?
    } else {
        caller
            .data()
            .instance
            .process()
            .environ_get()
            .map_err(process_errno)?
    };
    let mut pointers = Vec::with_capacity(pointers_range.len());
    for offset in serialized.pointer_offsets {
        let pointer = buffer_ptr.checked_add(offset).ok_or(Errno::Fault)?;
        pointers.extend_from_slice(&pointer.to_le_bytes());
    }
    if pointers.len() != pointers_range.len() || serialized.buffer.len() != buffer_range.len() {
        return Err(Errno::Inval);
    }
    write_validated(caller, pointers_range, &pointers)?;
    write_validated(caller, buffer_range, &serialized.buffer)
}

fn host_clock_time_get(
    mut caller: Caller<'_, WasiHostState>,
    clock_id: i32,
    _precision: i64,
    result_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::ClockTimeGet,
        clock_id as u32 as u64,
        _precision as u64,
    );
    let clock_bytes = clock_id.to_le_bytes();
    let result_ptr_bytes = result_ptr.to_le_bytes();
    let argument_digest = effect_digest(
        WasiEffectOpcode::ClockTimeGet,
        &[&clock_bytes, &result_ptr_bytes],
    );
    let mut timestamp_bytes = [0u8; 8];
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let result_range = aligned_range(caller, result_ptr as u32, 8, 8)?;
        sync_logical_fuel(caller);
        let fuel = caller.data().logical_fuel;
        let timestamp = caller
            .data()
            .instance
            .process()
            .clock_time_get(clock_id as u32, fuel)
            .map_err(process_errno)?;
        timestamp_bytes = timestamp.to_le_bytes();
        write_validated(caller, result_range, &timestamp_bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::ClockTimeGet,
        argument_digest,
        errno,
        &[&timestamp_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_filestat_get(mut caller: Caller<'_, WasiHostState>, fd: i32, result_ptr: i32) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdFilestatGet,
        fd as u32 as u64,
        result_ptr as u32 as u64,
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let result_range = aligned_range(caller, result_ptr as u32, 64, 8)?;
        let stat = caller.data().instance.fd_filestat_get(Fd(fd as u32))?;
        let bytes = encode_filestat(stat);
        write_validated(caller, result_range, &bytes)
    });
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_read(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    iovecs_ptr: i32,
    iovecs_len: i32,
    nread_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdRead,
        fd as u32 as u64,
        iovecs_ptr as u32 as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let iovecs_len_bytes = iovecs_len.to_le_bytes();
    let argument_digest = effect_digest(WasiEffectOpcode::FdRead, &[&fd_bytes, &iovecs_len_bytes]);
    let mut read_bytes = Vec::new();
    let mut count_bytes = [0u8; 4];
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let nread_range = aligned_range(caller, nread_ptr as u32, 4, 4)?;
        let plan = iovec_plan(caller, iovecs_ptr as u32, iovecs_len as u32)?;
        let length = usize::try_from(plan.total_len).map_err(|_| Errno::Inval)?;
        let state = caller.data_mut();
        let bytes = state
            .instance
            .fd_read(Fd(fd as u32), length, &mut state.chunk_reader)?;
        read_bytes.extend_from_slice(&bytes);
        scatter_iovecs(caller, &plan, &bytes)?;
        let count = u32::try_from(bytes.len()).map_err(|_| Errno::Inval)?;
        count_bytes = count.to_le_bytes();
        write_validated(caller, nread_range, &count_bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::FdRead,
        argument_digest,
        errno,
        &[&count_bytes, &read_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_pread(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    iovecs_ptr: i32,
    iovecs_len: i32,
    offset: i64,
    nread_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdPread,
        fd as u32 as u64,
        iovecs_ptr as u32 as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let iovecs_len_bytes = iovecs_len.to_le_bytes();
    let offset_bytes = offset.to_le_bytes();
    let argument_digest = effect_digest(
        WasiEffectOpcode::FdPread,
        &[&fd_bytes, &iovecs_len_bytes, &offset_bytes],
    );
    let mut read_bytes = Vec::new();
    let mut count_bytes = [0u8; 4];
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let nread_range = aligned_range(caller, nread_ptr as u32, 4, 4)?;
        let plan = iovec_plan(caller, iovecs_ptr as u32, iovecs_len as u32)?;
        let length = usize::try_from(plan.total_len).map_err(|_| Errno::Inval)?;
        let state = caller.data_mut();
        let bytes = state.instance.fd_pread(
            Fd(fd as u32),
            offset as u64,
            length,
            &mut state.chunk_reader,
        )?;
        read_bytes.extend_from_slice(&bytes);
        scatter_iovecs(caller, &plan, &bytes)?;
        let count = u32::try_from(bytes.len()).map_err(|_| Errno::Inval)?;
        count_bytes = count.to_le_bytes();
        write_validated(caller, nread_range, &count_bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::FdPread,
        argument_digest,
        errno,
        &[&count_bytes, &read_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_write(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    iovecs_ptr: i32,
    iovecs_len: i32,
    nwritten_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdWrite,
        fd as u32 as u64,
        iovecs_ptr as u32 as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let iovecs_len_bytes = iovecs_len.to_le_bytes();
    let mut argument_digest =
        effect_digest(WasiEffectOpcode::FdWrite, &[&fd_bytes, &iovecs_len_bytes]);
    let mut written_bytes = [0u8; 4];
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let nwritten_range = aligned_range(caller, nwritten_ptr as u32, 4, 4)?;
        let plan = iovec_plan(caller, iovecs_ptr as u32, iovecs_len as u32)?;
        let bytes = gather_iovecs(caller, &plan)?;
        argument_digest = effect_digest(
            WasiEffectOpcode::FdWrite,
            &[&fd_bytes, &iovecs_len_bytes, &bytes],
        );
        let written = caller.data_mut().write_fd(Fd(fd as u32), &bytes)?;
        let written = u32::try_from(written).map_err(|_| Errno::Inval)?;
        written_bytes = written.to_le_bytes();
        write_validated(caller, nwritten_range, &written_bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::FdWrite,
        argument_digest,
        errno,
        &[&written_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_seek(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    delta: i64,
    whence: i32,
    result_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdSeek,
        fd as u32 as u64,
        delta as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let delta_bytes = delta.to_le_bytes();
    let whence_bytes = whence.to_le_bytes();
    let argument_digest = effect_digest(
        WasiEffectOpcode::FdSeek,
        &[&fd_bytes, &delta_bytes, &whence_bytes],
    );
    let mut offset_bytes = [0u8; 8];
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let result_range = aligned_range(caller, result_ptr as u32, 8, 8)?;
        let whence = match whence as u32 {
            0 => Whence::Set,
            1 => Whence::Current,
            2 => Whence::End,
            _ => return Err(Errno::Inval),
        };
        let offset = caller
            .data_mut()
            .instance
            .fd_seek(Fd(fd as u32), delta, whence)?;
        offset_bytes = offset.to_le_bytes();
        write_validated(caller, result_range, &offset_bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::FdSeek,
        argument_digest,
        errno,
        &[&offset_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_readdir(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    buffer_ptr: i32,
    buffer_len: i32,
    cookie: i64,
    used_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdReaddir,
        fd as u32 as u64,
        buffer_ptr as u32 as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let buffer_len_bytes = buffer_len.to_le_bytes();
    let cookie_bytes = cookie.to_le_bytes();
    let argument_digest = effect_digest(
        WasiEffectOpcode::FdReaddir,
        &[&fd_bytes, &buffer_len_bytes, &cookie_bytes],
    );
    let mut dirent_bytes = Vec::new();
    let mut used_bytes = [0u8; 4];
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let buffer_range = byte_range(caller, buffer_ptr as u32, buffer_len as u32)?;
        let used_range = aligned_range(caller, used_ptr as u32, 4, 4)?;
        let entries =
            caller
                .data()
                .instance
                .fd_readdir(Fd(fd as u32), cookie as u64, buffer_range.len())?;
        let bytes = encode_dirents(&entries, buffer_range.len());
        dirent_bytes.extend_from_slice(&bytes);
        write_validated(
            caller,
            buffer_range.start..buffer_range.start + bytes.len(),
            &bytes,
        )?;
        let used = u32::try_from(bytes.len()).map_err(|_| Errno::Inval)?;
        used_bytes = used.to_le_bytes();
        write_validated(caller, used_range, &used_bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::FdReaddir,
        argument_digest,
        errno,
        &[&used_bytes, &dirent_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_path_create_directory(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::PathCreateDirectory,
        fd as u32 as u64,
        path_ptr as u32 as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let mut argument_digest = effect_digest(WasiEffectOpcode::PathCreateDirectory, &[&fd_bytes]);
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path = read_bytes(caller, path_ptr as u32, path_len as u32)?;
        argument_digest = effect_digest(WasiEffectOpcode::PathCreateDirectory, &[&fd_bytes, &path]);
        caller
            .data_mut()
            .instance
            .path_create_directory(Fd(fd as u32), &path)?;
        Ok(())
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::PathCreateDirectory,
        argument_digest,
        errno,
        &[],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_path_filestat_get(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    lookup_flags: i32,
    path_ptr: i32,
    path_len: i32,
    result_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::PathFilestatGet,
        fd as u32 as u64,
        lookup_flags as u32 as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let lookup_bytes = lookup_flags.to_le_bytes();
    let mut argument_digest = effect_digest(
        WasiEffectOpcode::PathFilestatGet,
        &[&fd_bytes, &lookup_bytes],
    );
    let mut stat_bytes = [0u8; 64];
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path_range = byte_range(caller, path_ptr as u32, path_len as u32)?;
        let result_range = aligned_range(caller, result_ptr as u32, 64, 8)?;
        let path = read_validated(caller, path_range)?;
        argument_digest = effect_digest(
            WasiEffectOpcode::PathFilestatGet,
            &[&fd_bytes, &lookup_bytes, &path],
        );
        if lookup_flags != 0 {
            return Err(Errno::Notcapable);
        }
        let stat = caller
            .data()
            .instance
            .path_filestat_get(Fd(fd as u32), &path)?;
        stat_bytes = encode_filestat(stat);
        write_validated(caller, result_range, &stat_bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::PathFilestatGet,
        argument_digest,
        errno,
        &[&stat_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_path_link(
    mut caller: Caller<'_, WasiHostState>,
    _old_fd: i32,
    _old_flags: i32,
    old_path_ptr: i32,
    old_path_len: i32,
    _new_fd: i32,
    new_path_ptr: i32,
    new_path_len: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::PathLink,
        _old_fd as u32 as u64,
        _old_flags as u32 as u64,
    );
    let mut argument_digest = effect_digest(WasiEffectOpcode::PathLink, &[]);
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let old_path = read_bytes(caller, old_path_ptr as u32, old_path_len as u32)?;
        let new_path = read_bytes(caller, new_path_ptr as u32, new_path_len as u32)?;
        argument_digest = effect_digest(WasiEffectOpcode::PathLink, &[&old_path, &new_path]);
        Err(Errno::Notcapable)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::PathLink,
        argument_digest,
        errno,
        &[],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

#[allow(clippy::too_many_arguments)]
fn host_path_open(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    lookup_flags: i32,
    path_ptr: i32,
    path_len: i32,
    open_flags: i32,
    rights_base: i64,
    rights_inheriting: i64,
    fd_flags: i32,
    result_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::PathOpen,
        fd as u32 as u64,
        lookup_flags as u32 as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let lookup_bytes = lookup_flags.to_le_bytes();
    let open_bytes = open_flags.to_le_bytes();
    let rights_base_bytes = rights_base.to_le_bytes();
    let rights_inheriting_bytes = rights_inheriting.to_le_bytes();
    let fd_flags_bytes = fd_flags.to_le_bytes();
    let mut argument_digest = effect_digest(
        WasiEffectOpcode::PathOpen,
        &[
            &fd_bytes,
            &lookup_bytes,
            &open_bytes,
            &rights_base_bytes,
            &rights_inheriting_bytes,
            &fd_flags_bytes,
        ],
    );
    let mut opened_bytes = [0u8; 4];
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path_range = byte_range(caller, path_ptr as u32, path_len as u32)?;
        let result_range = aligned_range(caller, result_ptr as u32, 4, 4)?;
        let path = read_validated(caller, path_range)?;
        argument_digest = effect_digest(
            WasiEffectOpcode::PathOpen,
            &[
                &fd_bytes,
                &lookup_bytes,
                &open_bytes,
                &rights_base_bytes,
                &rights_inheriting_bytes,
                &fd_flags_bytes,
                &path,
            ],
        );
        if lookup_flags != 0 || (open_flags as u32) & !1 != 0 {
            return Err(Errno::Notcapable);
        }
        let fd_flags = u16::try_from(fd_flags as u32)
            .ok()
            .and_then(|bits| FdFlags::from_bits(bits).ok())
            .ok_or(Errno::Inval)?;
        let opened = caller.data_mut().instance.path_open(
            Fd(fd as u32),
            &path,
            (open_flags as u32) & 1 != 0,
            Rights::from_bits(rights_base as u64)?,
            Rights::from_bits(rights_inheriting as u64)?,
            fd_flags,
        )?;
        opened_bytes = opened.0.to_le_bytes();
        write_validated(caller, result_range, &opened_bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::PathOpen,
        argument_digest,
        errno,
        &[&opened_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_path_readlink(
    mut caller: Caller<'_, WasiHostState>,
    _fd: i32,
    path_ptr: i32,
    path_len: i32,
    buffer_ptr: i32,
    buffer_len: i32,
    used_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::PathReadlink,
        _fd as u32 as u64,
        path_ptr as u32 as u64,
    );
    let mut argument_digest = effect_digest(WasiEffectOpcode::PathReadlink, &[]);
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path = read_bytes(caller, path_ptr as u32, path_len as u32)?;
        argument_digest = effect_digest(WasiEffectOpcode::PathReadlink, &[&path]);
        byte_range(caller, buffer_ptr as u32, buffer_len as u32)?;
        aligned_range(caller, used_ptr as u32, 4, 4)?;
        Err(Errno::Notcapable)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::PathReadlink,
        argument_digest,
        errno,
        &[],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_path_remove_directory(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::PathRemoveDirectory,
        fd as u32 as u64,
        path_ptr as u32 as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let mut argument_digest = effect_digest(WasiEffectOpcode::PathRemoveDirectory, &[&fd_bytes]);
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path = read_bytes(caller, path_ptr as u32, path_len as u32)?;
        argument_digest = effect_digest(WasiEffectOpcode::PathRemoveDirectory, &[&fd_bytes, &path]);
        caller
            .data_mut()
            .instance
            .path_remove_directory(Fd(fd as u32), &path)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::PathRemoveDirectory,
        argument_digest,
        errno,
        &[],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_path_rename(
    mut caller: Caller<'_, WasiHostState>,
    old_fd: i32,
    old_path_ptr: i32,
    old_path_len: i32,
    new_fd: i32,
    new_path_ptr: i32,
    new_path_len: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::PathRename,
        old_fd as u32 as u64,
        old_path_ptr as u32 as u64,
    );
    let old_fd_bytes = old_fd.to_le_bytes();
    let new_fd_bytes = new_fd.to_le_bytes();
    let mut argument_digest = effect_digest(
        WasiEffectOpcode::PathRename,
        &[&old_fd_bytes, &new_fd_bytes],
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let old_path_range = byte_range(caller, old_path_ptr as u32, old_path_len as u32)?;
        let new_path_range = byte_range(caller, new_path_ptr as u32, new_path_len as u32)?;
        let old_path = read_validated(caller, old_path_range)?;
        let new_path = read_validated(caller, new_path_range)?;
        argument_digest = effect_digest(
            WasiEffectOpcode::PathRename,
            &[&old_fd_bytes, &old_path, &new_fd_bytes, &new_path],
        );
        caller.data_mut().instance.path_rename(
            Fd(old_fd as u32),
            &old_path,
            Fd(new_fd as u32),
            &new_path,
        )
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::PathRename,
        argument_digest,
        errno,
        &[],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_path_unlink_file(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::PathUnlinkFile,
        fd as u32 as u64,
        path_ptr as u32 as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let mut argument_digest = effect_digest(WasiEffectOpcode::PathUnlinkFile, &[&fd_bytes]);
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path = read_bytes(caller, path_ptr as u32, path_len as u32)?;
        argument_digest = effect_digest(WasiEffectOpcode::PathUnlinkFile, &[&fd_bytes, &path]);
        caller
            .data_mut()
            .instance
            .path_unlink_file(Fd(fd as u32), &path)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::PathUnlinkFile,
        argument_digest,
        errno,
        &[],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_poll_oneoff(
    mut caller: Caller<'_, WasiHostState>,
    input_ptr: i32,
    output_ptr: i32,
    count: i32,
    nevents_ptr: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::PollOneoff,
        input_ptr as u32 as u64,
        output_ptr as u32 as u64,
    );
    let count_bytes = count.to_le_bytes();
    let mut argument_digest = effect_digest(WasiEffectOpcode::PollOneoff, &[&count_bytes]);
    let mut event_bytes = Vec::new();
    let mut event_count_bytes = [0u8; 4];
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let count = count as u32;
        let input_len = count.checked_mul(SUBSCRIPTION_SIZE).ok_or(Errno::Fault)?;
        let output_len = count.checked_mul(EVENT_SIZE).ok_or(Errno::Fault)?;
        let input_range = aligned_range(caller, input_ptr as u32, input_len, 8)?;
        let output_range = aligned_range(caller, output_ptr as u32, output_len, 8)?;
        let nevents_range = aligned_range(caller, nevents_ptr as u32, 4, 4)?;
        let input = read_validated(caller, input_range)?;
        argument_digest = effect_digest(WasiEffectOpcode::PollOneoff, &[&count_bytes, &input]);
        let subscriptions = decode_subscriptions(&input)?;
        sync_logical_fuel(caller);
        let mut fuel = caller.data().logical_fuel;
        let mut result = caller
            .data()
            .instance
            .process()
            .poll_oneoff(fuel, &subscriptions)
            .map_err(process_errno)?;
        if result.events.is_empty() {
            if let Some(wake) = result.next_wake_fuel {
                caller.data_mut().logical_fuel = wake;
                fuel = wake;
                result = caller
                    .data()
                    .instance
                    .process()
                    .poll_oneoff(fuel, &subscriptions)
                    .map_err(process_errno)?;
            }
        }
        if result.events.is_empty() || result.next_wake_fuel.is_some() {
            return Err(Errno::Inval);
        }
        let events = encode_events(&result.events);
        event_bytes.extend_from_slice(&events);
        if events.len() > output_range.len() {
            return Err(Errno::Inval);
        }
        write_validated(
            caller,
            output_range.start..output_range.start + events.len(),
            &events,
        )?;
        let count = u32::try_from(result.events.len()).map_err(|_| Errno::Inval)?;
        event_count_bytes = count.to_le_bytes();
        write_validated(caller, nevents_range, &event_count_bytes)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::PollOneoff,
        argument_digest,
        errno,
        &[&event_count_bytes, &event_bytes],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_sched_yield(mut caller: Caller<'_, WasiHostState>) -> i32 {
    let recent = caller
        .data_mut()
        .begin_import_call(WasiImportIndex::SchedYield, 0, 0);
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        sync_logical_fuel(caller);
        match caller
            .data()
            .instance
            .process()
            .sched_yield()
            .map_err(process_errno)?
        {
            HostEffect::Yield => Ok(()),
            HostEffect::Exit(_) => Err(Errno::Inval),
        }
    });
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_close(mut caller: Caller<'_, WasiHostState>, fd: i32) -> i32 {
    let recent = caller
        .data_mut()
        .begin_import_call(WasiImportIndex::FdClose, fd as u32 as u64, 0);
    let fd_bytes = fd.to_le_bytes();
    let argument_digest = effect_digest(WasiEffectOpcode::FdClose, &[&fd_bytes]);
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        caller.data_mut().instance.fd_close(Fd(fd as u32))
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::FdClose,
        argument_digest,
        errno,
        &[],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_fdstat_get(mut caller: Caller<'_, WasiHostState>, fd: i32, result_ptr: i32) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdFdstatGet,
        fd as u32 as u64,
        result_ptr as u32 as u64,
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let result_range = aligned_range(caller, result_ptr as u32, 24, 8)?;
        let entry = *caller.data().instance.fd_table().get(Fd(fd as u32))?;
        let mut bytes = [0u8; 24];
        bytes[0] = entry.file_type as u8;
        bytes[2..4].copy_from_slice(&entry.flags.bits().to_le_bytes());
        bytes[8..16].copy_from_slice(&entry.rights_base.bits().to_le_bytes());
        bytes[16..24].copy_from_slice(&entry.rights_inheriting.bits().to_le_bytes());
        write_validated(caller, result_range, &bytes)
    });
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_filestat_set_size(mut caller: Caller<'_, WasiHostState>, fd: i32, size: i64) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdFilestatSetSize,
        fd as u32 as u64,
        size as u64,
    );
    let fd_bytes = fd.to_le_bytes();
    let size_bytes = size.to_le_bytes();
    let argument_digest = effect_digest(
        WasiEffectOpcode::FdFilestatSetSize,
        &[&fd_bytes, &size_bytes],
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        caller
            .data_mut()
            .instance
            .fd_filestat_set_size(Fd(fd as u32), size as u64)
    });
    finish_effect_call(
        &mut caller,
        WasiEffectOpcode::FdFilestatSetSize,
        argument_digest,
        errno,
        &[],
    );
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_prestat_get(mut caller: Caller<'_, WasiHostState>, fd: i32, result_ptr: i32) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdPrestatGet,
        fd as u32 as u64,
        result_ptr as u32 as u64,
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let result_range = aligned_range(caller, result_ptr as u32, 8, 4)?;
        let prestat = caller
            .data()
            .instance
            .fd_table()
            .prestat_get(Fd(fd as u32))?;
        let mut bytes = [0u8; 8];
        bytes[4..8].copy_from_slice(&prestat.name_len.to_le_bytes());
        write_validated(caller, result_range, &bytes)
    });
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_fd_prestat_dir_name(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::FdPrestatDirName,
        fd as u32 as u64,
        path_ptr as u32 as u64,
    );
    let errno = errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path_range = byte_range(caller, path_ptr as u32, path_len as u32)?;
        let name = caller
            .data()
            .instance
            .fd_table()
            .prestat_dir_name(Fd(fd as u32))?;
        if name.len() > path_range.len() {
            return Err(Errno::Inval);
        }
        write_validated(
            caller,
            path_range.start..path_range.start + name.len(),
            name,
        )
    });
    caller.data_mut().finish_import_call(recent, errno);
    errno
}

fn host_proc_exit(mut caller: Caller<'_, WasiHostState>, code: i32) -> Result<(), Trap> {
    let recent =
        caller
            .data_mut()
            .begin_import_call(WasiImportIndex::ProcExit, code as u32 as u64, 0);
    let result: Result<(), Trap> = (|| {
        if caller.data().terminal_exit.is_some() {
            return Err(ProcExitTrap.into());
        }
        let effect = caller
            .data_mut()
            .instance
            .process_mut()
            .proc_exit(code as u32)
            .map_err(|_| Trap::new("WASI proc_exit rejected"))?;
        let HostEffect::Exit(code) = effect else {
            return Err(Trap::new("WASI proc_exit returned non-exit effect"));
        };
        let state = caller.data_mut();
        if let ThreadHostMode::Scheduled(world) = &mut state.thread_mode {
            world
                .scheduler
                .proc_exit(code)
                .map_err(|_| Trap::new("WASI proc_exit rejected by scheduler"))?;
        }
        state.terminal_exit = Some(code);
        Err(ProcExitTrap.into())
    })();
    caller.data_mut().finish_import_call(recent, -1);
    result
}

fn host_thread_spawn(mut caller: Caller<'_, WasiHostState>, start_arg: i32) -> i32 {
    let recent = caller.data_mut().begin_import_call(
        WasiImportIndex::ThreadSpawn,
        start_arg as u32 as u64,
        0,
    );
    let result = if caller.data().terminal_exit.is_some() {
        -1
    } else {
        match &mut caller.data_mut().thread_mode {
            ThreadHostMode::Deny => -1,
            ThreadHostMode::Scheduled(world) => world.reserve_spawn(start_arg),
        }
    };
    caller.data_mut().finish_import_call(recent, result);
    result
}

fn errno_call(
    caller: &mut Caller<'_, WasiHostState>,
    operation: impl FnOnce(&mut Caller<'_, WasiHostState>) -> Result<(), Errno>,
) -> i32 {
    match operation(caller) {
        Ok(()) => SUCCESS,
        Err(error) => i32::from(error.code()),
    }
}

fn effect_digest(opcode: WasiEffectOpcode, fields: &[&[u8]]) -> [u8; 32] {
    let payload_len = fields.iter().fold(0usize, |total, field| {
        total.saturating_add(8).saturating_add(field.len())
    });
    let mut canonical = Vec::with_capacity(1usize.saturating_add(payload_len));
    canonical.push(opcode as u8);
    for field in fields {
        canonical.extend_from_slice(&(field.len() as u64).to_le_bytes());
        canonical.extend_from_slice(field);
    }
    sha256_bytes(&canonical)
}

fn finish_effect_call(
    caller: &mut Caller<'_, WasiHostState>,
    opcode: WasiEffectOpcode,
    argument_digest: [u8; 32],
    errno: i32,
    result_fields: &[&[u8]],
) {
    if caller.data().terminal_exit.is_some() {
        return;
    }
    sync_logical_fuel(caller);
    let errno_bytes = errno.to_le_bytes();
    let mut fields = Vec::with_capacity(result_fields.len().saturating_add(1));
    fields.push(&errno_bytes[..]);
    fields.extend_from_slice(result_fields);
    let result_digest = effect_digest(opcode, &fields);
    fold_wasi_effect(caller.data_mut(), opcode, argument_digest, result_digest);
}

fn fold_wasi_effect(
    state: &mut WasiHostState,
    opcode: WasiEffectOpcode,
    argument_digest: [u8; 32],
    result_digest: [u8; 32],
) {
    let (round, tid) = match &mut state.thread_mode {
        ThreadHostMode::Deny => (0, 0),
        ThreadHostMode::Scheduled(world) => {
            let Some(thread) = world.active_thread else {
                world.fuel_accounting_failed = true;
                return;
            };
            (world.scheduler.current_round().get(), thread.get())
        }
    };
    let sequence = state.effect_sequence;
    let Some(next_sequence) = sequence.checked_add(1) else {
        if let Some(world) = state.scheduled_world_mut() {
            world.fuel_accounting_failed = true;
        }
        return;
    };
    let mut canonical = [0u8; 117];
    canonical[0..32].copy_from_slice(&state.effect_digest);
    canonical[32..40].copy_from_slice(&round.to_le_bytes());
    canonical[40..44].copy_from_slice(&tid.to_le_bytes());
    canonical[44..52].copy_from_slice(&sequence.to_le_bytes());
    canonical[52] = opcode as u8;
    canonical[53..85].copy_from_slice(&argument_digest);
    canonical[85..117].copy_from_slice(&result_digest);
    state.effect_digest = sha256_bytes(&canonical);
    state.effect_sequence = next_sequence;
}

fn ensure_active(caller: &Caller<'_, WasiHostState>) -> Result<(), Errno> {
    if caller.data().terminal_exit.is_some() {
        return Err(Errno::Badf);
    }
    Ok(())
}

fn process_errno(error: ProcessError) -> Errno {
    match error {
        ProcessError::Wasi(error) => error,
        ProcessError::Exited(_) => Errno::Badf,
    }
}

fn sync_logical_fuel(caller: &mut Caller<'_, WasiHostState>) {
    if caller.data().is_scheduled_thread_job() {
        match caller.consume_fuel(0) {
            Ok(remaining) => {
                let _ = caller.data_mut().sync_active_fuel(remaining);
            }
            Err(_) => {
                if let Some(world) = caller.data_mut().scheduled_world_mut() {
                    world.fuel_accounting_failed = true;
                }
            }
        }
    } else if let Some(consumed) = caller.fuel_consumed() {
        caller.data_mut().logical_fuel = caller.data().logical_fuel.max(consumed);
    }
}

fn memory_and_len(caller: &Caller<'_, WasiHostState>) -> Result<(Memory, u32), Errno> {
    let memory = caller.data().memory.ok_or(Errno::Fault)?;
    let len = u32::try_from(memory.data(caller).len()).map_err(|_| Errno::Fault)?;
    Ok((memory, len))
}

fn byte_range(
    caller: &Caller<'_, WasiHostState>,
    ptr: u32,
    len: u32,
) -> Result<Range<usize>, Errno> {
    let (_, mem_len) = memory_and_len(caller)?;
    checked_range(ptr, len, mem_len)
        .map_err(|_| Errno::Fault)
        .map(usize_range)
}

fn aligned_range(
    caller: &Caller<'_, WasiHostState>,
    ptr: u32,
    len: u32,
    alignment: u32,
) -> Result<Range<usize>, Errno> {
    let (_, mem_len) = memory_and_len(caller)?;
    checked_aligned_range(ptr, len, mem_len, alignment)
        .map_err(|_| Errno::Fault)
        .map(usize_range)
}

fn usize_range(range: Range<u32>) -> Range<usize> {
    range.start as usize..range.end as usize
}

fn read_bytes(caller: &Caller<'_, WasiHostState>, ptr: u32, len: u32) -> Result<Vec<u8>, Errno> {
    let range = byte_range(caller, ptr, len)?;
    read_validated(caller, range)
}

fn read_validated(
    caller: &Caller<'_, WasiHostState>,
    range: Range<usize>,
) -> Result<Vec<u8>, Errno> {
    let (memory, _) = memory_and_len(caller)?;
    Ok(memory.data(caller)[range].to_vec())
}

fn write_validated(
    caller: &mut Caller<'_, WasiHostState>,
    range: Range<usize>,
    bytes: &[u8],
) -> Result<(), Errno> {
    if range.len() != bytes.len() {
        return Err(Errno::Inval);
    }
    let memory = caller.data().memory.ok_or(Errno::Fault)?;
    memory.data_mut(caller)[range].copy_from_slice(bytes);
    Ok(())
}

fn iovec_plan(
    caller: &Caller<'_, WasiHostState>,
    ptr: u32,
    count: u32,
) -> Result<CheckedIovecs, Errno> {
    let (_, mem_len) = memory_and_len(caller)?;
    let list = checked_iovec_list(ptr, count, mem_len).map_err(|_| Errno::Fault)?;
    let list = usize_range(list);
    let memory = caller.data().memory.ok_or(Errno::Fault)?;
    let data = &memory.data(caller)[list];
    let mut iovecs = Vec::with_capacity(count as usize);
    for record in data.chunks_exact(8) {
        iovecs.push(GuestIovec {
            ptr: u32::from_le_bytes(record[0..4].try_into().map_err(|_| Errno::Fault)?),
            len: u32::from_le_bytes(record[4..8].try_into().map_err(|_| Errno::Fault)?),
        });
    }
    checked_iovecs(&iovecs, mem_len).map_err(|_| Errno::Fault)
}

fn gather_iovecs(
    caller: &Caller<'_, WasiHostState>,
    plan: &CheckedIovecs,
) -> Result<Vec<u8>, Errno> {
    let memory = caller.data().memory.ok_or(Errno::Fault)?;
    let data = memory.data(caller);
    let mut bytes = Vec::with_capacity(plan.total_len as usize);
    for range in &plan.ranges {
        bytes.extend_from_slice(&data[usize_range(range.clone())]);
    }
    Ok(bytes)
}

fn scatter_iovecs(
    caller: &mut Caller<'_, WasiHostState>,
    plan: &CheckedIovecs,
    bytes: &[u8],
) -> Result<(), Errno> {
    if bytes.len() > plan.total_len as usize {
        return Err(Errno::Inval);
    }
    let memory = caller.data().memory.ok_or(Errno::Fault)?;
    let data = memory.data_mut(caller);
    let mut source = bytes;
    for range in &plan.ranges {
        let target = &mut data[usize_range(range.clone())];
        let count = target.len().min(source.len());
        target[..count].copy_from_slice(&source[..count]);
        source = &source[count..];
        if source.is_empty() {
            break;
        }
    }
    if !source.is_empty() {
        return Err(Errno::Inval);
    }
    Ok(())
}

fn encode_filestat(stat: Filestat) -> [u8; 64] {
    let mut bytes = [0u8; 64];
    bytes[0..8].copy_from_slice(&stat.device.to_le_bytes());
    bytes[8..16].copy_from_slice(&stat.inode.to_le_bytes());
    bytes[16] = stat.file_type as u8;
    bytes[24..32].copy_from_slice(&stat.link_count.to_le_bytes());
    bytes[32..40].copy_from_slice(&stat.size.to_le_bytes());
    bytes[40..48].copy_from_slice(&stat.access_time.to_le_bytes());
    bytes[48..56].copy_from_slice(&stat.modification_time.to_le_bytes());
    bytes[56..64].copy_from_slice(&stat.status_change_time.to_le_bytes());
    bytes
}

fn encode_dirents(entries: &[raios_wasi_preview1::Dirent], limit: usize) -> Vec<u8> {
    let mut output = Vec::new();
    for entry in entries {
        let mut record = [0u8; DIRENT_SIZE];
        record[0..8].copy_from_slice(&entry.next_cookie.to_le_bytes());
        record[8..16].copy_from_slice(&entry.inode.0.to_le_bytes());
        record[16..20].copy_from_slice(&(entry.name.len() as u32).to_le_bytes());
        record[20] = entry.file_type as u8;
        append_bounded(&mut output, &record, limit);
        append_bounded(&mut output, &entry.name, limit);
        if output.len() == limit {
            break;
        }
    }
    output
}

fn append_bounded(output: &mut Vec<u8>, bytes: &[u8], limit: usize) {
    let available = limit.saturating_sub(output.len());
    output.extend_from_slice(&bytes[..bytes.len().min(available)]);
}

fn decode_subscriptions(bytes: &[u8]) -> Result<Vec<Subscription>, Errno> {
    if bytes.len() % SUBSCRIPTION_SIZE as usize != 0 {
        return Err(Errno::Inval);
    }
    let mut subscriptions = Vec::with_capacity(bytes.len() / SUBSCRIPTION_SIZE as usize);
    for record in bytes.chunks_exact(SUBSCRIPTION_SIZE as usize) {
        let userdata = u64::from_le_bytes(record[0..8].try_into().map_err(|_| Errno::Inval)?);
        match record[8] {
            0 => {
                let flags =
                    u16::from_le_bytes(record[40..42].try_into().map_err(|_| Errno::Inval)?);
                let timeout = match flags {
                    0 => ClockTimeout::Relative,
                    1 => ClockTimeout::Absolute,
                    _ => return Err(Errno::Inval),
                };
                subscriptions.push(Subscription::Clock(ClockSubscription {
                    userdata,
                    clock_id: u32::from_le_bytes(
                        record[16..20].try_into().map_err(|_| Errno::Inval)?,
                    ),
                    timeout_ns: u64::from_le_bytes(
                        record[24..32].try_into().map_err(|_| Errno::Inval)?,
                    ),
                    timeout,
                }));
            }
            1 => subscriptions.push(Subscription::FdRead {
                userdata,
                fd: u32::from_le_bytes(record[16..20].try_into().map_err(|_| Errno::Inval)?),
            }),
            2 => subscriptions.push(Subscription::FdWrite {
                userdata,
                fd: u32::from_le_bytes(record[16..20].try_into().map_err(|_| Errno::Inval)?),
            }),
            _ => return Err(Errno::Inval),
        }
    }
    Ok(subscriptions)
}

fn encode_events(events: &[raios_wasi_preview1::PollEvent]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(events.len() * EVENT_SIZE as usize);
    for event in events {
        let mut record = [0u8; EVENT_SIZE as usize];
        record[0..8].copy_from_slice(&event.userdata.to_le_bytes());
        bytes.extend_from_slice(&record);
    }
    bytes
}
