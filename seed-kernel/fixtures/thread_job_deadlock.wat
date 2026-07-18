(module
  (import "env" "memory" (memory 1 1 shared))
  (import "wasi" "thread-spawn" (func $thread_spawn (param i32) (result i32)))
  (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))

  ;; Materialize one worker and wait until it has definitely started. Main
  ;; then parks forever at address 4 while the worker is parked at address 8;
  ;; nobody can notify either final wait.
  (func (export "_start")
    i32.const 0
    call $thread_spawn
    i32.const 0
    i32.lt_s
    if
      unreachable
    end

    (block $worker_started
      (loop $wait
        i32.const 0
        i32.atomic.load
        i32.const 1
        i32.ge_u
        br_if $worker_started

        i32.const 0
        i32.const 0
        i64.const -1
        memory.atomic.wait32
        drop
        br $wait))

    i32.const 4
    i32.const 0
    i64.const -1
    memory.atomic.wait32
    drop)

  (func (export "wasi_thread_start") (param $tid i32) (param $start_arg i32)
    i32.const 0
    i32.const 1
    i32.atomic.rmw.add
    drop

    i32.const 0
    i32.const 1
    memory.atomic.notify
    drop

    i32.const 8
    i32.const 0
    i64.const -1
    memory.atomic.wait32
    drop))
