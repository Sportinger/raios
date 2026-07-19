(module
  (memory (export "memory") 1 1)

  (func (export "raios_service_main") (result i32)
    ;; Base + the maximum u32 static offset must trap, not wrap to address 0.
    i32.const 1
    i32.load offset=4294967295))
