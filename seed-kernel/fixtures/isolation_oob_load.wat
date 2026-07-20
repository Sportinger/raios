(module
  (memory (export "memory") 1 1)

  (func (export "raios_service_main") (result i32)
    ;; Returning the load makes a missing bounds check guest-visible.
    i32.const 65536
    i32.load))
