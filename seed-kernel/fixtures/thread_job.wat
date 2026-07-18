(module
  (import "env" "memory" (memory 1 1 shared))

  ;; TID 0 parks first. Once woken it copies the final increment sum to offset 8.
  (func (export "thread0")
    i32.const 0
    i32.const 0
    i64.const -1
    memory.atomic.wait32
    drop

    i32.const 8
    i32.const 4
    i32.atomic.load
    i32.store)

  ;; TID 1 proves notify(0), performs 32 atomic increments, then wakes TID 0.
  (func (export "thread1")
    (local $iteration i32)

    i32.const 0
    i32.const 0
    memory.atomic.notify
    drop

    (loop $increment
      i32.const 4
      i32.const 1
      i32.atomic.rmw.add
      drop

      local.get $iteration
      i32.const 1
      i32.add
      local.tee $iteration
      i32.const 32
      i32.lt_u
      br_if $increment)

    i32.const 0
    i32.const 1
    i32.atomic.store

    i32.const 0
    i32.const 1
    memory.atomic.notify
    drop))
