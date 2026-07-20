(module
  (import "wasi_snapshot_preview1" "random_get" (func (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "args_get" (func (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "args_sizes_get" (func (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "clock_time_get" (func (param i32 i64 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_filestat_get" (func (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_read" (func (param i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_readdir" (func (param i32 i32 i32 i64 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_seek" (func (param i32 i64 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_create_directory" (func (param i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_filestat_get" (func (param i32 i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_link" (func (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_open" (func (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_readlink" (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_remove_directory" (func (param i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_rename" (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_unlink_file" (func (param i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "poll_oneoff" (func (param i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "sched_yield" (func (result i32)))
  (import "wasi_snapshot_preview1" "environ_get" (func (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "environ_sizes_get" (func (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_close" (func (param i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_fdstat_get" (func (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_filestat_set_size" (func (param i32 i64) (result i32)))
  (import "wasi_snapshot_preview1" "fd_pread" (func (param i32 i32 i32 i64 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_prestat_get" (func (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_prestat_dir_name" (func (param i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
  (import "wasi" "thread-spawn" (func $thread_spawn (param i32) (result i32)))
  (memory (import "env" "memory") 399 16384 shared)
  (export "memory" (memory 0))

  ;; Two stable iovecs and their payloads. No active data segment covers the
  ;; atomic at address 0, so worker materialization cannot reset main's flag.
  (data (i32.const 32) "\40\00\00\00\05\00\00\00\45\00\00\00\07\00\00\00")
  (data (i32.const 64) "main\0aworker\0a")

  (func $write_main
    i32.const 1
    i32.const 32
    i32.const 1
    i32.const 48
    call $fd_write
    i32.eqz
    if
    else
      unreachable
    end
    i32.const 48
    i32.load
    i32.const 5
    i32.ne
    if
      unreachable
    end)

  (func $write_worker
    i32.const 1
    i32.const 40
    i32.const 1
    i32.const 52
    call $fd_write
    i32.eqz
    if
    else
      unreachable
    end
    i32.const 52
    i32.load
    i32.const 7
    i32.ne
    if
      unreachable
    end)

  ;; Main writes first, publishes flag=1, then parks on that same aligned
  ;; atomic. The worker can only run after this suspension point.
  (func (export "_start")
    (local $worker i32)
    i32.const 7
    call $thread_spawn
    local.tee $worker
    i32.const 1
    i32.ne
    if
      unreachable
    end

    call $write_main
    i32.const 0
    i32.const 1
    i32.atomic.store
    i32.const 0
    i32.const 1
    memory.atomic.notify
    drop

    i32.const 0
    i32.const 1
    i64.const -1
    memory.atomic.wait32
    drop

    i32.const 0
    i32.atomic.load
    i32.const 2
    i32.ne
    if
      unreachable
    end
    i32.const 0
    call $proc_exit
    unreachable)

  ;; Worker verifies the shared publication, writes second, and wakes main.
  (func (export "wasi_thread_start") (param $tid i32) (param $start_arg i32)
    local.get $tid
    i32.const 1
    i32.ne
    if
      unreachable
    end
    local.get $start_arg
    i32.const 7
    i32.ne
    if
      unreachable
    end
    i32.const 0
    i32.atomic.load
    i32.const 1
    i32.ne
    if
      unreachable
    end

    call $write_worker
    i32.const 0
    i32.const 2
    i32.atomic.store
    i32.const 0
    i32.const 1
    memory.atomic.notify
    drop))
