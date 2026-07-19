(module
  (memory (export "memory") 1 1)

  (func (export "raios_service_main") (result i32)
    ;; The first byte after the single 64-KiB page must never be writable.
    i32.const 65536
    i32.const 0x11223344
    i32.store
    i32.const 0))
