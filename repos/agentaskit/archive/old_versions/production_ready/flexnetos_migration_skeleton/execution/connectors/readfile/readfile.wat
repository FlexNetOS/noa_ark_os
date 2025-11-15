(module
  ;; Imports
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_open"
    (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_read"
    (func $fd_read (param i32 i32 i32 i32) (result i32)))

  (memory (export "memory") 2)

  ;; Data: relative path within preopened dir fd=3
  (data (i32.const 1024) "hello.txt")

  (func (export "run")
    (local $newfd i32)        ;; file descriptor for opened file
    (local $nread i32)        ;; bytes read
    (local $errno i32)

    ;; iovec for read: base=2048, len=256 at offset 128
    (i32.store (i32.const 128) (i32.const 2048))
    (i32.store (i32.const 132) (i32.const 256))

    ;; Open "hello.txt" from preopened dir fd=3
    ;; rights: 1 = FD_READ, inheriting=0, oflags=0, fs_flags=0, lookupflags=0
    (local.set $errno
      (call $path_open
        (i32.const 3)        ;; dirfd (preopened /cap assumed at fd 3)
        (i32.const 0)        ;; lookupflags
        (i32.const 1024)     ;; path ptr
        (i32.const 9)        ;; path len
        (i32.const 0)        ;; oflags
        (i64.const 1)        ;; rights_base (FD_READ)
        (i64.const 0)        ;; rights_inherit
        (i32.const 0)        ;; fs_flags
        (i32.const 120)      ;; result newfd stored at [120..124]
      )
    )
    ;; load newfd
    (local.set $newfd (i32.load (i32.const 120)))

    ;; Read into buffer
    (local.set $errno
      (call $fd_read
        (local.get $newfd)
        (i32.const 128)    ;; iov ptr
        (i32.const 1)      ;; iov count
        (i32.const 124)    ;; nread stored at [124..128]
      )
    )
    (local.set $nread (i32.load (i32.const 124)))

    ;; Write to stdout (fd=1) using same iovec with updated len
    (i32.store (i32.const 132) (local.get $nread))
    (drop (call $fd_write (i32.const 1) (i32.const 128) (i32.const 1) (i32.const 160)))
  )
)
