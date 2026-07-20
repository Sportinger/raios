(module
  (import "env" "log" (func $log (param i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "rollback target log-only")
  (func (export "raios_service_main") (result i32)
    i32.const 0
    i32.const 24
    call $log
    i32.const 0))
