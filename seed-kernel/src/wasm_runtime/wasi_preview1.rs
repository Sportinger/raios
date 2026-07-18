use alloc::vec::Vec;
use core::{fmt, ops::Range};

use raios_core::{
    wasi_build_output::FrozenOutput, wasi_preview1_import_abi::RUSTC_WASM_C6DCCF3E_IMPORTS,
};
use raios_wasi_preview1::{
    checked_aligned_range, checked_iovec_list, checked_iovecs, checked_range, CheckedIovecs,
    ClockSubscription, ClockTimeout, Errno, Fd, FdFlags, FileType, Filestat, GuestIovec,
    HostEffect, MountId, ProcessError, Rights, Subscription, ThreadHost, WasiBuildInstance, Whence,
};
use wasmi::{core::Trap, Caller, Linker, Memory};

use super::wasi_build_storage::GrantedChunkReader;

const SUCCESS: i32 = 0;
const SUBSCRIPTION_SIZE: u32 = 48;
const EVENT_SIZE: u32 = 32;
const DIRENT_SIZE: usize = 24;

pub(crate) struct DenyThreadHost;

impl ThreadHost for DenyThreadHost {
    fn spawn(&mut self, _start_arg: i32) -> i32 {
        -1
    }
}

pub(crate) struct WasiHostState {
    instance: WasiBuildInstance,
    memory: Option<Memory>,
    thread_host: DenyThreadHost,
    chunk_reader: GrantedChunkReader,
    stdout_bytes: u64,
    stderr_bytes: u64,
    logical_fuel: u64,
    terminal_exit: Option<u32>,
}

impl WasiHostState {
    pub(crate) fn new(instance: WasiBuildInstance, chunk_reader: GrantedChunkReader) -> Self {
        Self {
            instance,
            memory: None,
            thread_host: DenyThreadHost,
            chunk_reader,
            stdout_bytes: 0,
            stderr_bytes: 0,
            logical_fuel: 0,
            terminal_exit: None,
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
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let range = byte_range(caller, ptr as u32, len as u32)?;
        let bytes = caller
            .data_mut()
            .instance
            .process_mut()
            .random_get(range.len())
            .map_err(process_errno)?;
        write_validated(caller, range, &bytes)
    })
}

fn host_args_sizes_get(
    mut caller: Caller<'_, WasiHostState>,
    count_ptr: i32,
    size_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
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
    })
}

fn host_args_get(mut caller: Caller<'_, WasiHostState>, pointers_ptr: i32, buffer_ptr: i32) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let sizes = caller
            .data()
            .instance
            .process()
            .args_sizes_get()
            .map_err(process_errno)?;
        write_serialized_strings(caller, pointers_ptr as u32, buffer_ptr as u32, sizes, true)
    })
}

fn host_environ_sizes_get(
    mut caller: Caller<'_, WasiHostState>,
    count_ptr: i32,
    size_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
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
    })
}

fn host_environ_get(
    mut caller: Caller<'_, WasiHostState>,
    pointers_ptr: i32,
    buffer_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let sizes = caller
            .data()
            .instance
            .process()
            .environ_sizes_get()
            .map_err(process_errno)?;
        write_serialized_strings(caller, pointers_ptr as u32, buffer_ptr as u32, sizes, false)
    })
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
    errno_call(&mut caller, |caller| {
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
        write_validated(caller, result_range, &timestamp.to_le_bytes())
    })
}

fn host_fd_filestat_get(mut caller: Caller<'_, WasiHostState>, fd: i32, result_ptr: i32) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let result_range = aligned_range(caller, result_ptr as u32, 64, 8)?;
        let stat = caller.data().instance.fd_filestat_get(Fd(fd as u32))?;
        let bytes = encode_filestat(stat);
        write_validated(caller, result_range, &bytes)
    })
}

fn host_fd_read(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    iovecs_ptr: i32,
    iovecs_len: i32,
    nread_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let nread_range = aligned_range(caller, nread_ptr as u32, 4, 4)?;
        let plan = iovec_plan(caller, iovecs_ptr as u32, iovecs_len as u32)?;
        let length = usize::try_from(plan.total_len).map_err(|_| Errno::Inval)?;
        let state = caller.data_mut();
        let bytes = state
            .instance
            .fd_read(Fd(fd as u32), length, &mut state.chunk_reader)?;
        scatter_iovecs(caller, &plan, &bytes)?;
        let count = u32::try_from(bytes.len()).map_err(|_| Errno::Inval)?;
        write_validated(caller, nread_range, &count.to_le_bytes())
    })
}

fn host_fd_pread(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    iovecs_ptr: i32,
    iovecs_len: i32,
    offset: i64,
    nread_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
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
        scatter_iovecs(caller, &plan, &bytes)?;
        let count = u32::try_from(bytes.len()).map_err(|_| Errno::Inval)?;
        write_validated(caller, nread_range, &count.to_le_bytes())
    })
}

fn host_fd_write(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    iovecs_ptr: i32,
    iovecs_len: i32,
    nwritten_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let nwritten_range = aligned_range(caller, nwritten_ptr as u32, 4, 4)?;
        let plan = iovec_plan(caller, iovecs_ptr as u32, iovecs_len as u32)?;
        let bytes = gather_iovecs(caller, &plan)?;
        let written = caller.data_mut().write_fd(Fd(fd as u32), &bytes)?;
        let written = u32::try_from(written).map_err(|_| Errno::Inval)?;
        write_validated(caller, nwritten_range, &written.to_le_bytes())
    })
}

fn host_fd_seek(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    delta: i64,
    whence: i32,
    result_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
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
        write_validated(caller, result_range, &offset.to_le_bytes())
    })
}

fn host_fd_readdir(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    buffer_ptr: i32,
    buffer_len: i32,
    cookie: i64,
    used_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let buffer_range = byte_range(caller, buffer_ptr as u32, buffer_len as u32)?;
        let used_range = aligned_range(caller, used_ptr as u32, 4, 4)?;
        let entries =
            caller
                .data()
                .instance
                .fd_readdir(Fd(fd as u32), cookie as u64, buffer_range.len())?;
        let bytes = encode_dirents(&entries, buffer_range.len());
        write_validated(
            caller,
            buffer_range.start..buffer_range.start + bytes.len(),
            &bytes,
        )?;
        let used = u32::try_from(bytes.len()).map_err(|_| Errno::Inval)?;
        write_validated(caller, used_range, &used.to_le_bytes())
    })
}

fn host_path_create_directory(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path = read_bytes(caller, path_ptr as u32, path_len as u32)?;
        caller
            .data_mut()
            .instance
            .path_create_directory(Fd(fd as u32), &path)?;
        Ok(())
    })
}

fn host_path_filestat_get(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    lookup_flags: i32,
    path_ptr: i32,
    path_len: i32,
    result_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path_range = byte_range(caller, path_ptr as u32, path_len as u32)?;
        let result_range = aligned_range(caller, result_ptr as u32, 64, 8)?;
        let path = read_validated(caller, path_range)?;
        if lookup_flags != 0 {
            return Err(Errno::Notcapable);
        }
        let stat = caller
            .data()
            .instance
            .path_filestat_get(Fd(fd as u32), &path)?;
        write_validated(caller, result_range, &encode_filestat(stat))
    })
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
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        byte_range(caller, old_path_ptr as u32, old_path_len as u32)?;
        byte_range(caller, new_path_ptr as u32, new_path_len as u32)?;
        Err(Errno::Notcapable)
    })
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
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path_range = byte_range(caller, path_ptr as u32, path_len as u32)?;
        let result_range = aligned_range(caller, result_ptr as u32, 4, 4)?;
        let path = read_validated(caller, path_range)?;
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
        write_validated(caller, result_range, &opened.0.to_le_bytes())
    })
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
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        byte_range(caller, path_ptr as u32, path_len as u32)?;
        byte_range(caller, buffer_ptr as u32, buffer_len as u32)?;
        aligned_range(caller, used_ptr as u32, 4, 4)?;
        Err(Errno::Notcapable)
    })
}

fn host_path_remove_directory(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path = read_bytes(caller, path_ptr as u32, path_len as u32)?;
        caller
            .data_mut()
            .instance
            .path_remove_directory(Fd(fd as u32), &path)
    })
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
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let old_path_range = byte_range(caller, old_path_ptr as u32, old_path_len as u32)?;
        let new_path_range = byte_range(caller, new_path_ptr as u32, new_path_len as u32)?;
        let old_path = read_validated(caller, old_path_range)?;
        let new_path = read_validated(caller, new_path_range)?;
        caller.data_mut().instance.path_rename(
            Fd(old_fd as u32),
            &old_path,
            Fd(new_fd as u32),
            &new_path,
        )
    })
}

fn host_path_unlink_file(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let path = read_bytes(caller, path_ptr as u32, path_len as u32)?;
        caller
            .data_mut()
            .instance
            .path_unlink_file(Fd(fd as u32), &path)
    })
}

fn host_poll_oneoff(
    mut caller: Caller<'_, WasiHostState>,
    input_ptr: i32,
    output_ptr: i32,
    count: i32,
    nevents_ptr: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let count = count as u32;
        let input_len = count.checked_mul(SUBSCRIPTION_SIZE).ok_or(Errno::Fault)?;
        let output_len = count.checked_mul(EVENT_SIZE).ok_or(Errno::Fault)?;
        let input_range = aligned_range(caller, input_ptr as u32, input_len, 8)?;
        let output_range = aligned_range(caller, output_ptr as u32, output_len, 8)?;
        let nevents_range = aligned_range(caller, nevents_ptr as u32, 4, 4)?;
        let input = read_validated(caller, input_range)?;
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
        if events.len() > output_range.len() {
            return Err(Errno::Inval);
        }
        write_validated(
            caller,
            output_range.start..output_range.start + events.len(),
            &events,
        )?;
        let count = u32::try_from(result.events.len()).map_err(|_| Errno::Inval)?;
        write_validated(caller, nevents_range, &count.to_le_bytes())
    })
}

fn host_sched_yield(mut caller: Caller<'_, WasiHostState>) -> i32 {
    errno_call(&mut caller, |caller| {
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
    })
}

fn host_fd_close(mut caller: Caller<'_, WasiHostState>, fd: i32) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        caller.data_mut().instance.fd_close(Fd(fd as u32))
    })
}

fn host_fd_fdstat_get(mut caller: Caller<'_, WasiHostState>, fd: i32, result_ptr: i32) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        let result_range = aligned_range(caller, result_ptr as u32, 24, 8)?;
        let entry = *caller.data().instance.fd_table().get(Fd(fd as u32))?;
        let mut bytes = [0u8; 24];
        bytes[0] = entry.file_type as u8;
        bytes[2..4].copy_from_slice(&entry.flags.bits().to_le_bytes());
        bytes[8..16].copy_from_slice(&entry.rights_base.bits().to_le_bytes());
        bytes[16..24].copy_from_slice(&entry.rights_inheriting.bits().to_le_bytes());
        write_validated(caller, result_range, &bytes)
    })
}

fn host_fd_filestat_set_size(mut caller: Caller<'_, WasiHostState>, fd: i32, size: i64) -> i32 {
    errno_call(&mut caller, |caller| {
        ensure_active(caller)?;
        caller
            .data_mut()
            .instance
            .fd_filestat_set_size(Fd(fd as u32), size as u64)
    })
}

fn host_fd_prestat_get(mut caller: Caller<'_, WasiHostState>, fd: i32, result_ptr: i32) -> i32 {
    errno_call(&mut caller, |caller| {
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
    })
}

fn host_fd_prestat_dir_name(
    mut caller: Caller<'_, WasiHostState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    errno_call(&mut caller, |caller| {
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
    })
}

fn host_proc_exit(mut caller: Caller<'_, WasiHostState>, code: i32) -> Result<(), Trap> {
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
    caller.data_mut().terminal_exit = Some(code);
    Err(ProcExitTrap.into())
}

fn host_thread_spawn(mut caller: Caller<'_, WasiHostState>, start_arg: i32) -> i32 {
    if caller.data().terminal_exit.is_some() {
        return -1;
    }
    caller.data_mut().thread_host.spawn(start_arg)
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
    if let Some(consumed) = caller.fuel_consumed() {
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
