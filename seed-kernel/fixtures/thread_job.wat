(module
  (import "env" "memory" (memory 1 1 shared))
  (import "wasi" "thread-spawn" (func $thread_spawn (param i32) (result i32)))
  (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))

  ;; Main reserves two workers, then parks until both publish completion.
  ;; The workers stay parked, so main + 2 workers still consume the cap when
  ;; the deliberately denied third spawn proves the negative boundary.
  (func (export "_start")
    (local $done i32)

    i32.const 16
    call $thread_spawn
    drop
    i32.const 16
    call $thread_spawn
    drop

    (block $workers_done
      (loop $wait
        i32.const 0
        i32.atomic.load
        local.tee $done
        i32.const 2
        i32.ge_u
        br_if $workers_done

        i32.const 0
        local.get $done
        i64.const -1
        memory.atomic.wait32
        drop
        br $wait))

    i32.const 8
    i32.const 4
    i32.atomic.load
    i32.store

    ;; Cap is 3 in this selftest. A non-negative result is a fixture failure.
    i32.const 0
    call $thread_spawn
    i32.const 0
    i32.ge_s
    if
      unreachable
    end

    i32.const 0
    call $proc_exit
    ;; proc_exit is required never to return.
    unreachable)

  ;; wasi-threads entry point: each worker contributes start_arg increments,
  ;; publishes completion, notifies main, then parks until whole-job proc_exit.
  (func (export "wasi_thread_start") (param $tid i32) (param $start_arg i32)
    (local $iteration i32)

    (block $increments_done
      (loop $increment
        local.get $iteration
        local.get $start_arg
        i32.ge_u
        br_if $increments_done

        i32.const 4
        i32.const 1
        i32.atomic.rmw.add
        drop

        local.get $iteration
        i32.const 1
        i32.add
        local.set $iteration
        br $increment))

    i32.const 0
    i32.const 1
    i32.atomic.rmw.add
    drop

    i32.const 0
    i32.const 1
    memory.atomic.notify
    drop

    i32.const 12
    i32.const 0
    i64.const -1
    memory.atomic.wait32
    drop))
