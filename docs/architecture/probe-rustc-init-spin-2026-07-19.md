# Probe: rustc early-init spin — static differential

Date: 2026-07-19  
Mode: exploratory, report-only

## Result

The requested Wasmtime execution differential could **not** be run in this
lane. Wasmtime was absent, the workspace sandbox denied creation under
`C:\Users\admin\raios-artifacts\`, shell network access was blocked, and the
in-app browser had no browser instance. Therefore this note does not claim a
new reference reproduction or a no-thread execution result. The successful
Wasmtime 46.0.1 reference result below is the prior 2026-07-18 measurement,
not a rerun.

The available module could still be examined exactly with LLVM 19.1.1 and a
small temporary `wasmparser-nostd` scanner. The strongest static finding is:

> `_start`'s first synchronization call is libc++abi
> `__cxa_guard_acquire`, using the global-mutex implementation for a C++
> function-local static. It waits for that guard's `PENDING` bit to clear.

This is a concrete pre-arguments primitive, but without a sampled raiOS PC it
is a **candidate**, not a proved hot instruction.

## Inputs and reference status

| Item | Measured in this probe |
|---|---|
| Module | `C:\Users\admin\raios-artifacts\rustc-wasm\rustc_opt.wasm` |
| Size | 95,427,808 bytes |
| SHA-256 | `c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791` |
| Sysroot | Present, including `libwasi-emulated-{getpid,mman,process-clocks,signal}.a` under `self-contained/` |
| Disassembler | LLVM `llvm-objdump` 19.1.1-rust-1.84.0-nightly |
| Wasmtime 46.0.1 | Not installed and could not be obtained from this sandbox |

The intended pinned command, with `$WORK` containing `sysroot/`, is:

```powershell
wasmtime -W threads=y -W shared-memory=y -S threads=y `
  --dir "${WORK}::/" rustc_opt.wasm -- --version --sysroot /sysroot
```

The prior reference record in
`probe-rustc-wasm-wasmtime-2026-07-18.md` reports:

```text
rustc 1.83.0-dev
exit: 0
elapsed: 0.36 s
wasmtime: 46.0.1, -W threads=y -W shared-memory=y -S threads=y
```

That output was **not reproduced on 2026-07-19**. Likewise, neither a
spawn-denying host differential nor a fuel/profile run was executed. Merely
using `-S threads=n` is not an equivalent negative test if it prevents a
module importing `wasi:thread-spawn` from instantiating; the valid execution
test needs the import present and a host implementation that returns spawn
failure.

## Static pre-arguments boundary

The module imports 29 functions. Import 1 is `args_get`, import 2 is
`args_sizes_get`, and import 28 is `wasi:thread-spawn`. An exact instruction
scan found:

```text
direct args_sizes_get calls: func 113803 @ file offset 0x3f747d8 (one)
direct args_get calls:       func 113803 @ file offset 0x3f748a5 (one)
direct thread-spawn calls:   func 114041 @ file offset 0x3f8cedd (one)
```

Thus any observed PC before `0x3f747d8` on the startup path is statically
before the first arguments syscall. The first synchronization in `_start`
appears much earlier:

```text
0x4b20f  i32.const 26069796       ;; guard object
0x4b214  i32.atomic.load8_u
0x4b218  i32.const 1
0x4b21a  i32.and                  ;; COMPLETE?
0x4b21d  i32.const 26069796
0x4b222  call 8657                ;; __cxa_guard_acquire
```

Names are retained in the module's debug strings:
`__cxa_guard_acquire`, `__cxa_guard_release`, `__cxa_guard_abort`,
`cxa_guard_impl.h`, and `libcxx/src/call_once.cpp`.

Function 8657 matches libc++abi's global-mutex guard algorithm exactly:

```text
0x651e56  mutex-lock 25779500
0x651e61  load guard[1]
0x651e68  test 2                   ;; PENDING
0x651e6d  loop
0x651e73  or 4; store guard[1]     ;; WAITING
0x651e79  condvar 25779524
0x651e7e  mutex 25779500
0x651e83  call 114040              ;; pthread condition wait
0x651e88  reload guard[1]
0x651e8f  test 2; branch to loop   ;; wait until not PENDING
0x651e9d  store 2 to guard[1]      ;; become initializer
```

Function 8658 is the paired release: it atomically stores `COMPLETE=1` in
the guard byte, changes the status byte to complete under the same mutex, and
broadcasts if `WAITING=4` was set. These constants and transitions agree with
LLVM libc++abi's
[`cxa_guard_impl.h`](https://github.com/llvm/llvm-project/blob/main/libcxxabi/src/cxa_guard_impl.h).

Startup function 31 zero-fills BSS at `25778640` for 304,864 bytes, which
contains both the mutex (`25779500`) and guard (`26069796`). On their first
use they are therefore zero. The mutex fast path (function 114045) is:

```text
0x3f8d2a9  mutex address
0x3f8d2ab  expected = 0
0x3f8d2ad  replacement = 10
0x3f8d2af  i32.atomic.rmw.cmpxchg mutex+4
0x3f8d2b3  i32.eqz
0x3f8d2b4  branch to success
```

Per Wasm semantics, a successful compare-exchange writes 10 and returns the
old value 0. The `eqz` is consequently true. Returning the replacement value,
not storing on equality, or losing this memory state across pump slices would
incorrectly enter the slow lock path in an otherwise uncontended main thread.

## Why an ordinary blocked thread is not enough to explain the raiOS trace

The linked libc wait helper (function 114031) polls for a finite number of
iterations and then executes:

```text
0x3f8c1af  ... check expected value ...
0x3f8c1ba  memory.atomic.wait32
```

The guard's mutex/condition-variable slow paths use this libc synchronization
machinery. Therefore a normally contended `__cxa_guard_acquire` should
eventually produce an `atomic.wait`. The raiOS measurement instead reports
200k+ pump rounds with zero `atomic.wait` and zero WASI calls. That is negative
evidence against simply “rustc spawned a worker and is normally waiting for
it.” It points more specifically to one of these conditions:

1. atomic compare-exchange result/write semantics or lock state is wrong;
2. execution repeatedly re-enters or replays a pre-park CAS/poll region;
3. the hot PC is another constructor region in the very large `_start` body.

The module's startup function (function 31, before exported `_start`) also has
a TLS-init barrier with `memory.atomic.wait32`. Since raiOS reports that this
barrier cleared and counted no waits, it is not the best match. Worker entry
`wasi_thread_start` writes the spawned TID into worker TLS; the main TLS image
starts with zero in the reference module too, so “invent a nonzero main TID”
is not supported by this disassembly.

## Named fix hypothesis

**Primary hypothesis:** the in-kernel implementation of
`i32.atomic.rmw.cmpxchg` violates the old-value result/write-on-equality
contract, or loses the mutex word between pump slices. Correct that primitive
so the uncontended `0 -> 10` mutex acquisition at `0x3f8d2af` returns `0` and
persists `10`. This is the first concrete check because the mutex is proven
zero-initialized and a correct single-thread fast path cannot spin.

**Secondary hypothesis, conditional on a PC sample landing in function
8657:** the green-thread pump is re-entering or replaying `_start` rather than
resuming one activation, including its Wasm PC, operand stack, call stack,
stack global, and memory. If an abandoned activation stored
`guard[1]=PENDING (2)` and a later activation re-entered
`__cxa_guard_acquire`, the logical main thread could wait for a guard release
that only the abandoned activation can perform. The corresponding fix is to
preserve/resume the activation until `__cxa_guard_release` marks guard address
`26069796` complete.

Before changing behavior, record at every pump boundary:

- PC and call depth;
- bytes at `26069796` and `26069797` (`COMPLETE` and status);
- mutex/condition state at `25779500` (especially `mutex+4`) and `25779524`.

A stable PC in `0x651e6d..0x651e92` with `guard[0]==0` and
`guard[1]&2==2` would prove this guard diagnosis. A stable PC in the mutex
fast/slow path (`0x3f8d289...`) with unexpected compare-exchange results would
prove the atomic-RMW diagnosis.

## Honest limits and next decisive measurement

The static probe locates the earliest named candidate and what it waits on,
but it does not prove that raiOS executes that instruction repeatedly. It
also cannot answer whether Wasmtime completes when `thread-spawn` is denied,
because Wasmtime 46.0.1 could not be run here.

One low-rate interpreter PC histogram, resolved against the offsets above,
would distinguish `__cxa_guard_acquire` (`0x651e39..0x651ef1`), libc lock/wait
(`0x3f8be69..0x3f8c1e9`), the TLS barrier, and another constructor. The
remaining reference differential should use exactly Wasmtime 46.0.1 with a
spawn-denying `wasi:thread-spawn` host while leaving the threads proposal and
shared memory enabled.
