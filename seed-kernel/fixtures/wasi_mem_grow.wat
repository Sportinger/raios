(module
  ;; Fixed 28-byte control block, independent of successful grow count:
  ;;   0: initial pages, 4: final pages, 8: successful grow-step count
  ;;  12: grow-denied marker, 16: stdout iovec pointer, 20: iovec length
  ;;  24: stdout bytes written. Decimal scratch is fixed at 32..41.
  ;; All fields and scratch precede the first touched grown page (initial << 16).
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
  (import "wasi" "thread-spawn" (func (param i32) (result i32)))
  (memory (import "env" "memory") 399 16384 shared)
  (export "memory" (memory 0))

  (func (export "_start")
    (local $old_pages i32)
    (local $pages i32)
    (local $value i32)
    (local $digits i32)
    (local $cursor i32)

    i32.const 0
    memory.size
    local.tee $pages
    i32.store
    i32.const 4
    local.get $pages
    i32.store
    i32.const 8
    i32.const 0
    i32.store
    i32.const 12
    i32.const 0
    i32.store

    (block $grow_done
      (loop $grow
        i32.const 115
        memory.grow
        local.tee $old_pages
        i32.const -1
        i32.eq
        if
          i32.const 12
          i32.const 1
          i32.store
          br $grow_done
        end

        ;; Back the growth honestly: touch the first byte of its first new page.
        local.get $old_pages
        i32.const 16
        i32.shl
        i32.const 1
        i32.store8

        memory.size
        local.set $pages
        i32.const 4
        local.get $pages
        i32.store
        i32.const 8
        i32.const 8
        i32.load
        i32.const 1
        i32.add
        i32.store
        br $grow))

    i32.const 4
    i32.load
    local.set $value
    i32.const 42
    local.set $cursor
    i32.const 0
    local.set $digits
    (loop $decimal
      local.get $cursor
      i32.const 1
      i32.sub
      local.tee $cursor
      local.get $value
      i32.const 10
      i32.rem_u
      i32.const 48
      i32.add
      i32.store8
      local.get $digits
      i32.const 1
      i32.add
      local.set $digits
      local.get $value
      i32.const 10
      i32.div_u
      local.tee $value
      br_if $decimal)

    i32.const 16
    local.get $cursor
    i32.store
    i32.const 20
    local.get $digits
    i32.store
    i32.const 1
    i32.const 16
    i32.const 1
    i32.const 24
    call $fd_write
    if
      i32.const 1
      call $proc_exit
    end
    i32.const 24
    i32.load
    local.get $digits
    i32.ne
    if
      i32.const 1
      call $proc_exit
    end
    i32.const 0
    call $proc_exit))
