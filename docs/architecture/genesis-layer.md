# Genesis layer: the substitutable Wasm floor

Status: description of the contract implemented at this revision, not a design
proposal. The substitutability boundary is the narrow Wasm-import and
service-capability floor; the custom kernel remains the current implementation
and kernel-private types are outside that boundary (ADR 0015). Wasm host imports
are the first enforced service isolation boundary, while native drivers and
performance-critical paths remain kernel-owned (ADR 0005, Decision 2).

## 1. Contract shape

For this floor, a **service** is one validated Wasm artifact, named by a
non-empty service id, instantiated with a fresh interpreter instance and only
the host imports authorized for that service. The ordinary service gate also
requires an artifact binding to be present; an observed empty import list is a
valid zero-authority surface. (`crates/raios-core/src/scoped_wasm_import_grant.rs:102-117,237-272`;
`seed-kernel/src/wasm_runtime/envelope.rs:300-408`)

A **grant** is an exact, ordered list of `(module, name)` imports scoped to one
service id. Authorization produces a decision with performed/status/reason and
an authorized count; the runtime derives an order-sensitive digest from the
service id and that list. The digest is evidence, not authority by itself.
(`crates/raios-core/src/scoped_wasm_import_grant.rs:120-126,624-653`)

Authority is split as follows:

- The kernel validates, links, instantiates, schedules, terminalizes and drops a
  service; no service import below creates, kills, restarts or manages another
  service. (`seed-kernel/src/wasm_runtime/envelope.rs:344-458,648-686`;
  `crates/raios-core/src/scoped_wasm_import_grant.rs:36-64`)
- F12 is core-only secure attention: it is not delivered as a guest key event,
  and pressing it advances the kernel kill generation. A running service checks
  that generation at its next pump boundary and becomes terminal when it
  differs. (`seed-kernel/src/input.rs:202-217,293-304`;
  `crates/raios-core/src/beyond_env_invocation.rs:201-218`;
  `seed-kernel/src/wasm_runtime/invocation.rs:678-700`)
- Terminal teardown invalidates issued handles, removes the pending operation,
  records one teardown receipt, then drops the continuation, memory, instance
  and interpreter state without holding a raiOS lock. Repeated teardown is a
  no-op. (`crates/raios-core/src/beyond_env_invocation.rs:225-252`;
  `seed-kernel/src/wasm_runtime/invocation.rs:823-878`)
- A later start constructs a new invocation id and fresh interpreter state only
  after the previous active owner is gone. This is the current restart
  mechanism; the harness proves a second fresh run after F12, not a guest-facing
  restart call. (`seed-kernel/src/main.rs:420-444`;
  `seed-kernel/src/wasm_runtime/invocation.rs:301-425`;
  `vm-harness/shadow-vm-smoke-profile-m11-beyond-env-lifecycle.ps1:83-90`)
- Direct hardware authority stays in the kernel; drivers are deliberately not
  Wasm services in the current architecture. (ADR 0005, Decision 2 and
  Non-Goals)

There are two non-interchangeable import worlds: the general service world
below links a per-service subset of five `env.*` functions, while the build
world admits one complete, measured 30-entry surface and nothing less or more.
(`seed-kernel/src/wasm_runtime/envelope.rs:648-686`;
`crates/raios-core/src/wasi_preview1_import_abi.rs:178-254`)

## 2. Signature and denial conventions

Signatures use WebAssembly value types and the notation
`(parameters) -> results`; `()` means no value. Build functions return an
`i32` WASI errno unless shown otherwise; success is `0`, and implemented error
codes include `Badf=8`, `Fault=21`, `Inval=28`, `Rofs=69` and
`Notcapable=76`. (`crates/raios-core/src/wasi_preview1_import_abi.rs:13-44`;
`crates/raios-wasi-preview1/src/errno.rs:6-31`;
`seed-kernel/src/wasm_runtime/wasi_preview1.rs:1757-1764`)

**U (ungranted or malformed import declaration).** In the service world, a
missing service/artifact/list, unknown or duplicate import, list over 16, or
non-`env` request is denied with authorized count zero. Only an actually
observed empty list is accepted as zero grants. The linker defines only the
authorized subset, scans the module for an extra import, and refuses before
instantiation; a signature mismatch also makes instantiation fail.
(`crates/raios-core/src/scoped_wasm_import_grant.rs:251-309,624-630`;
`seed-kernel/src/wasm_runtime/envelope.rs:313-329,377-408,648-708`)

In the build world, malformed hashes and any extra, missing, reordered or
signature-changed entry produce a typed denial for the whole grant. The
observed module list is independently compared with the reference list; the
linker count must equal `30`, the reference count and the authorized count
before instantiation. (`crates/raios-core/src/scoped_wasi_build_grant.rs:23-53,79-169`;
`crates/raios-core/src/authorized_build_job.rs:84-98`;
`seed-kernel/src/wasm_runtime/wasi_build_job.rs:572-611,681-694`)

**M (malformed call).** Once linked, build shims validate guest ranges,
alignment, iovec lists, descriptors, flags and rights before effect and return
the relevant errno; path/link operations which are deliberately unavailable
return `Notcapable`. (`seed-kernel/src/wasm_runtime/wasi_preview1.rs:1834-1948`;
`seed-kernel/src/wasm_runtime/wasi_preview1.rs:1190-1242,1290-1313,1336-1358`)

## 3. General service imports (`env.*`)

U applies to every row. The source column gives the allow-list line, linker
line and implementation line; therefore both signature and effect are pinned
to the current implementation. (`crates/raios-core/src/scoped_wasm_import_grant.rs:36-42`;
`seed-kernel/src/wasm_runtime/envelope.rs:648-686`)

| Import | Signature | One-line semantics and malformed-call behavior | Source |
|---|---|---|---|
| `env.log` | `(i32 ptr, i32 len) -> ()` | Reads at most 256 guest bytes, maps non-printable bytes to spaces and emits one guest-log line; negative, overflowing, oversized, absent-memory or out-of-bounds input traps before logging. | `crates/raios-core/src/scoped_wasm_import_grant.rs:37`; `seed-kernel/src/wasm_runtime/envelope.rs:656-660,807-847` |
| `env.counter_get` | `() -> i64` | Increments the current-boot host counter with saturation and returns it capped at `i64::MAX`; fuel exhaustion traps before the increment. | `crates/raios-core/src/scoped_wasm_import_grant.rs:38`; `seed-kernel/src/wasm_runtime/envelope.rs:661-665,932-938` |
| `env.input_len` | `() -> i32` | Returns staged-input length; staged input over 4096 bytes or fuel exhaustion traps. | `crates/raios-core/src/scoped_wasm_import_grant.rs:39`; `seed-kernel/src/wasm_runtime/envelope.rs:666-670,850-859` |
| `env.input_read` | `(i32 ptr, i32 len) -> i32` | Copies `min(len, staged length)` bytes into guest memory and returns the copied count; negative, overflowing, over-4096, absent-memory or out-of-bounds input traps before the copy. | `crates/raios-core/src/scoped_wasm_import_grant.rs:40`; `seed-kernel/src/wasm_runtime/envelope.rs:671-675,861-887` |
| `env.output_write` | `(i32 ptr, i32 len) -> i32` | Appends guest bytes to a per-run captured-output buffer and returns the appended count; negative, overflowing, per-call or cumulative over-4096, absent-memory or out-of-bounds input traps before append. | `crates/raios-core/src/scoped_wasm_import_grant.rs:41`; `seed-kernel/src/wasm_runtime/envelope.rs:676-680,889-929` |

## 4. Frozen WASI build surface (`raios.wasi_build_imports.v1`)

U and M apply to every row. Declaration order is the numbered order below and
is part of the canonical hash. (`crates/raios-core/src/wasi_preview1_import_abi.rs:4-5,68-121,178-254`)

| # | Import | Signature | One-line semantics and call denial | Definition; link/implementation |
|---:|---|---|---|---|
| 1 | `wasi_snapshot_preview1.random_get` | `(i32 ptr, i32 len) -> i32` | Fills a validated guest range from the job's process random source; invalid range/process state returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:181`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:568-570,672-703` |
| 2 | `wasi_snapshot_preview1.args_get` | `(i32 argv, i32 buf) -> i32` | Writes configured argument pointers and NUL-terminated bytes; malformed ranges or inconsistent sizes return errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:182`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:571-573,732-749,803-838` |
| 3 | `wasi_snapshot_preview1.args_sizes_get` | `(i32 count, i32 size) -> i32` | Writes argument count and total buffer size to aligned outputs; bad outputs/state return errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:183`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:574-576,705-729` |
| 4 | `wasi_snapshot_preview1.clock_time_get` | `(i32 clock, i64 precision, i32 out) -> i32` | Writes the selected deterministic, logical-fuel-derived timestamp; unsupported clock or bad output returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:184-189`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:577-579,840-880` |
| 5 | `wasi_snapshot_preview1.fd_filestat_get` | `(i32 fd, i32 out) -> i32` | Writes the 64-byte file status for a descriptor; bad descriptor/output returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:190`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:580-582,883-898` |
| 6 | `wasi_snapshot_preview1.fd_read` | `(i32 fd, i32 iovs, i32 n, i32 nread) -> i32` | Reads sequentially into validated iovecs and writes the byte count; bad fd, iovecs, storage authority or output returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:191`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:583-585,900-941` |
| 7 | `wasi_snapshot_preview1.fd_readdir` | `(i32 fd, i32 buf, i32 len, i64 cookie, i32 used) -> i32` | Encodes directory entries starting at `cookie` into the bounded buffer and writes bytes used; malformed cookie/fd/ranges return errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:192-197`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:586-588,1083-1134` |
| 8 | `wasi_snapshot_preview1.fd_seek` | `(i32 fd, i64 delta, i32 whence, i32 out) -> i32` | Moves the descriptor cursor relative to start/current/end and writes the new offset; invalid `whence`, fd or output returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:198`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:589-591,1036-1081` |
| 9 | `wasi_snapshot_preview1.fd_write` | `(i32 fd, i32 iovs, i32 n, i32 nwritten) -> i32` | Gathers validated iovecs, writes to the selected writable fd and reports bytes written; bad ranges/fd/quota/rights return errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:199`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:592-594,994-1034` |
| 10 | `wasi_snapshot_preview1.path_create_directory` | `(i32 fd, i32 path, i32 len) -> i32` | Creates a directory below the supplied directory fd; invalid path, rights, mount or quota returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:200-205`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:595-599,1136-1168` |
| 11 | `wasi_snapshot_preview1.path_filestat_get` | `(i32 fd, i32 flags, i32 path, i32 len, i32 out) -> i32` | Writes status for a path below a directory fd; any nonzero lookup flags return `Notcapable`, and bad path/fd/output returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:206`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:600-602,1170-1218` |
| 12 | `wasi_snapshot_preview1.path_link` | `(i32 oldfd, i32 flags, i32 oldpath, i32 oldlen, i32 newfd, i32 newpath, i32 newlen) -> i32` | Validates both path ranges, then always returns `Notcapable`; no hard link is created. | `crates/raios-core/src/wasi_preview1_import_abi.rs:207`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:603-605,1220-1252` |
| 13 | `wasi_snapshot_preview1.path_open` | `(i32 fd, i32 lookup, i32 path, i32 len, i32 oflags, i64 base, i64 inheriting, i32 fdflags, i32 out) -> i32` | Opens, or creates with bit 0, a path under granted base/inheriting rights; unsupported lookup/open flags return `Notcapable`, other malformed fields return errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:208-213`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:606-608,1255-1334` |
| 14 | `wasi_snapshot_preview1.path_readlink` | `(i32 fd, i32 path, i32 len, i32 buf, i32 buflen, i32 used) -> i32` | Validates path and output ranges, then always returns `Notcapable`; no link target is exposed. | `crates/raios-core/src/wasi_preview1_import_abi.rs:214`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:609-611,1336-1368` |
| 15 | `wasi_snapshot_preview1.path_remove_directory` | `(i32 fd, i32 path, i32 len) -> i32` | Removes a directory below the supplied fd; malformed path, rights, type or non-empty directory returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:215-220`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:612-616,1370-1401` |
| 16 | `wasi_snapshot_preview1.path_rename` | `(i32 oldfd, i32 oldpath, i32 oldlen, i32 newfd, i32 newpath, i32 newlen) -> i32` | Renames between two validated paths; cross-mount, rights, type or malformed-path failure returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:221`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:617-619,1403-1449` |
| 17 | `wasi_snapshot_preview1.path_unlink_file` | `(i32 fd, i32 path, i32 len) -> i32` | Unlinks a file below the supplied fd; malformed path, rights or file type returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:222`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:620-622,1451-1482` |
| 18 | `wasi_snapshot_preview1.poll_oneoff` | `(i32 in, i32 out, i32 count, i32 nevents) -> i32` | Evaluates validated subscriptions against logical fuel, writes deterministic events and their count; malformed/empty unresolved results return errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:223`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:623-625,1484-1557` |
| 19 | `wasi_snapshot_preview1.sched_yield` | `() -> i32` | Requests the process yield effect after synchronizing logical fuel; inactive or non-yield state returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:224`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:626-628,1559-1579` |
| 20 | `wasi_snapshot_preview1.environ_get` | `(i32 envp, i32 buf) -> i32` | Writes configured environment pointers and NUL-terminated bytes; malformed ranges or inconsistent sizes return errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:225`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:629-631,779-838` |
| 21 | `wasi_snapshot_preview1.environ_sizes_get` | `(i32 count, i32 size) -> i32` | Writes environment entry count and total buffer size to aligned outputs; bad outputs/state return errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:226`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:632-634,752-777` |
| 22 | `wasi_snapshot_preview1.fd_close` | `(i32 fd) -> i32` | Closes the descriptor; stale, invalid or uncloseable descriptors return errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:227`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:635-637,1581-1600` |
| 23 | `wasi_snapshot_preview1.fd_fdstat_get` | `(i32 fd, i32 out) -> i32` | Writes file type, flags, base rights and inheriting rights for a descriptor; bad fd/output returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:228`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:638-640,1602-1621` |
| 24 | `wasi_snapshot_preview1.fd_filestat_set_size` | `(i32 fd, i64 size) -> i32` | Resizes a writable file; bad fd, rights, size or quota returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:229-234`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:641-645,1623-1651` |
| 25 | `wasi_snapshot_preview1.fd_pread` | `(i32 fd, i32 iovs, i32 n, i64 offset, i32 nread) -> i32` | Reads at the supplied offset into validated iovecs and reports the count; bad fd/range/storage authority returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:235`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:646-648,943-992` |
| 26 | `wasi_snapshot_preview1.fd_prestat_get` | `(i32 fd, i32 out) -> i32` | Writes the preopened-directory tag and name length; a non-preopen fd or bad output returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:236`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:649-651,1653-1673` |
| 27 | `wasi_snapshot_preview1.fd_prestat_dir_name` | `(i32 fd, i32 path, i32 len) -> i32` | Copies the preopened directory name into the validated buffer; non-preopen fd or short/bad buffer returns errno. | `crates/raios-core/src/wasi_preview1_import_abi.rs:237-242`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:652-656,1675-1705` |
| 28 | `wasi_snapshot_preview1.proc_exit` | `(i32 code) -> ()` | Records one process exit (and scheduled-world exit when active), latches the code and raises the dedicated terminal trap; a second/rejected exit traps. | `crates/raios-core/src/wasi_preview1_import_abi.rs:243`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:657-659,1707-1737` |
| 29 | `wasi.thread-spawn` | `(i32 start_arg) -> i32` | In scheduled mode reserves a thread id and queues the spawn; denial mode, terminal process or cap failure returns `-1`, with no interpreter re-entry. | `crates/raios-core/src/wasi_preview1_import_abi.rs:244`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:660-662,1739-1755`; ADR 0022, Decision 2 |
| 30 | `env.memory` | shared memory, initial 399 pages, maximum 16,384 pages | Supplies the one shared linear memory used by the authorized module and its workers; any different kind/limits/sharing/order is a grant signature mismatch. | `crates/raios-core/src/wasi_preview1_import_abi.rs:245-253`; `seed-kernel/src/wasm_runtime/wasi_preview1.rs:663`; `crates/raios-core/src/scoped_wasi_build_grant.rs:130-169`; ADR 0022, Decision 1-2 |

## 5. Grant recording, scope and refusal

### General services

The authorization decision schema is
`raios.scoped_wasm_import_grant_authorization_decision.v0` with current-boot id
`scoped_wasm_import_grant_authorization.current_boot.v0`. Its evaluated inputs
are service id, artifact-binding presence, the requested exact list and the
beyond-`env` policy bit; authorization or denial carries a stable reason and
count. (`crates/raios-core/src/scoped_wasm_import_grant.rs:20-24,102-126,251-309`)

Each execution emits a current-boot marker containing service id, performed,
status, reason, authorized count/list digest, linked count, subset result,
outcome and any missing import pair. (`seed-kernel/src/wasm_runtime/envelope.rs:525-550,747-774`)

Successful grants for exactly three mapped service ids can additionally become
durable, local-only `capability_grant` records. The record carries service id,
the ordered import objects, count/list digest, linked count and subset result;
unmapped services, mismatched evidence, a full dedupe table, construction
failure or append failure stay RAM-only with an explicit reason. Duplicate
service/list pairs cite the first append. (`seed-kernel/src/memory_store.rs:2033-2135,2173-2185,2220-2286`)

There is no current revoke transition: these grant records have no superseded
record, and the live linker surface is fixed when the instance is built. A
revocation flow, next-call revocation guarantee and durable revoke audit are
not part of the current floor. (`seed-kernel/src/memory_store.rs:2226-2241,2311-2333`;
`docs/scope/02-genesis-layer.md:20-31`)

### WASI build jobs

The all-or-nothing grant binds three exact SHA-256 values (compiler artifact,
job manifest and canonical import inventory) plus the complete typed import
list. A usable job also independently binds the observed import list, validated
resource class and the two mount-manifest hashes before the kernel receives an
immutable authorization. (`crates/raios-core/src/scoped_wasi_build_grant.rs:57-121`;
`crates/raios-core/src/authorized_build_job.rs:19-26,63-119`)

Build storage authority is conjunctive and separate: manifests are rebound to
the authorized job, reads resolve only through the materialized per-job table
and are rehashed, and build runs have no ambient persistent write authority;
only verified egress can reach the store commit path. (ADR 0020, Decision 1-3;
`seed-kernel/src/wasm_runtime/wasi_build_job.rs:612-639,1024-1043`)

## 6. Versioning and hash discipline

The service decision/audit records are `.v0`, but the import family id used by
evidence-bound beyond-`env` gates is `raios.host_imports.v1`. Its digest binds
the ABI id and exact ordered import **pairs**; it is order- and id-sensitive but
does not encode the five `env.*` signatures. (`crates/raios-core/src/host_import_abi_v1.rs:10,192-212,272-285`;
`crates/raios-core/src/scoped_wasm_import_grant.rs:20-24,637-653`)

No executable compatibility checker implements a general “v0 means additive”
rule. That scope item remains open; consequently additive ABI evolution is not
a guarantee of the current floor. Existing names/signatures and exact grant
lists are the only implemented compatibility facts. (`docs/scope/01-rust-kernel.md:41-42`;
`seed-kernel/src/wasm_runtime/envelope.rs:648-708`)

The build family is stronger and frozen. Its canonical bytes are domain-
separated by `raios.wasi_build_imports.v1` and include ordered module/name,
import kind, parameter/result types and memory limits. The reference digest is
`4145184d6ae43a57e1f75e1cfc2b4a19c6dd27fb1a29dd7104e1df9817616e65`.
(`crates/raios-core/src/wasi_preview1_import_abi.rs:11,68-121`;
`crates/raios-core/src/scoped_wasi_build_grant.rs:13-14`)

Under this v1 id, even an otherwise sensible additive 31st import is a complete
denial, as are removal, reorder and signature change. Changing this surface is
therefore a new contract/owner decision, not an in-place v1 extension.
(`crates/raios-core/src/scoped_wasi_build_grant.rs:130-169,237-307`;
`docs/scope/02-genesis-layer.md:1-7`)

## 7. Explicit non-guarantees

- Kernel implementation representations, Rust types, interpreter objects,
  scheduler state, file-table layout, storage offsets, device interfaces and
  event-log internals are not contract types. Only the import signatures,
  externally observable results/denials and authority rules above form the
  substitutable floor. (ADR 0015; ADR 0018, Decision)
- The general floor does not promise `net.*`, `crypto.*`, `time.*`,
  `secret_lease.*`, `acquire.*` or `ui.*`. They occur in the internal known-name
  registry or service-specific paths, but the generic per-service linker here
  implements only the five `env.*` rows and rejects any other requested
  implementation. (`crates/raios-core/src/scoped_wasm_import_grant.rs:36-64`;
  `seed-kernel/src/wasm_runtime/envelope.rs:648-686`)
- There is no guest-callable create/kill/restart API, no cross-service manager,
  no revocation operation, no crash-loop parking contract and no `<1 s` restart
  SLA in this floor. F12 kill and subsequent fresh invocation are the implemented
  primitives. (`crates/raios-core/src/scoped_wasm_import_grant.rs:36-64`;
  `seed-kernel/src/input.rs:293-304`;
  `docs/scope/02-genesis-layer.md:11-18,39-50`)
- The build surface is the measured import inventory of one pinned compiler
  artifact, not a promise of general WASI Preview1 completeness or arbitrary
  WASI-module compatibility. (`crates/raios-core/src/wasi_preview1_import_abi.rs:178-180`;
  ADR 0018; ADR 0022)
- No JIT, native guest code, Wasm drivers, raw hardware access, raw secret
  exposure, trusted-time/WebPKI result or kernel-equivalence security proof is
  promised here. (ADR 0005, Non-Goals; ADR 0008, Consequences 296-305)
- seL4 substitutability is an architectural option, not a maintained
  primitive-by-primitive mapping or a second implementation in this revision.
  (ADR 0015; `docs/scope/02-genesis-layer.md:58-61`)
