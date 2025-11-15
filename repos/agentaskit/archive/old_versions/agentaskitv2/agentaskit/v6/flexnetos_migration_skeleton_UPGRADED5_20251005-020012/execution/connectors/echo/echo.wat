(module
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 8) "echo connector ok\n")
  (func (export "run")
    (i32.store (i32.const 0) (i32.const 1))    ;; fd=1 stdout
    (i32.store (i32.const 4) (i32.const 8))    ;; iov base
    (i32.store (i32.const 12) (i32.const 18))  ;; iov len
    (call $fd_write (i32.const 0) (i32.const 4) (i32.const 1) (i32.const 20))
    drop))
