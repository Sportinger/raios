# Secure AI workspace and high-performance application plan (2026-07-12)

## Status and target

W1-W5 are implemented and verified on the disposable QEMU structured store,
owner-controlled workstation build path and bounded current-boot Wasm runtime.
The remaining plan opens no network fetch, provider export, durable install,
native-code, media-file, GPU, or physical-storage authority.

Capability target: a user and AI can create or import a reviewable source tree,
edit it through a project-scoped workspace, build a reproducible bounded
application candidate, inspect its code and evidence, approve its exact hash,
and run or roll it back without downloaded bytes or the builder becoming
authority.

The high-performance acceptance application is a video editor whose UI and
workflow are isolated from a native media engine. The AI assembles the editor
from existing codecs and services; it does not reinvent codecs or place FFmpeg
inside the permanent core.

## Current truth

- `program_workspace` is not a general source workspace. It retains one
  validated `RAIOS_UI_SPEC_V1`/canonical RUIP program in current-boot RAM.
- raiOS already has content-addressed artifact records, a structured-store
  path, external candidate intake, Wasm validation, exact import grants,
  promotion, persistence, recovery, and rollback mechanisms. Reuse them.
- ADR 0009 fixes the invariant `download = inert candidate, never install`.
  Its accepted first transport is a local signed registry over serial. General
  network fetch remains gated.
- The current Wasm runtime is `wasmi`, interpreter-only. ADR 0005 explicitly
  keeps drivers and performance-critical paths native and forbids a JIT in the
  permanent kernel.
- Full WebPKI chain and trusted-time validation are not complete. Arbitrary
  production HTTPS source downloads therefore remain denied.
- The durable QEMU store is real; production physical workspace storage is not
  yet proven on the owner's selected medium.

So the answer to “does the AI already have a secure place for downloaded code?”
is **not yet for network-downloaded code**. It now has the proven bounded local
path: immutable project revisions and quarantined local dependency bundles live
in the disposable QEMU structured store and can feed the exact offline
workstation build contract. This is not a general filesystem, production
physical store, network download cache, generic dependency resolver, owner-sealed
toolchain, or independently verified local build.

## Standing invariants

1. Workspace bytes are data, never executable authority.
2. Download, archive extraction, build, test, grant, install, and run are
   separate transitions with separate evidence.
3. A URL, registry signature, source signature, compiler receipt, stored
   `verified` flag, or provider answer can never authorize loading.
4. The loader accepts only the existing evidence/promotion chain bound to the
   exact output hash and computed import grant.
5. Source, dependencies, build outputs, media assets, evidence, and secrets use
   separate namespaces. Secrets never enter the workspace.
6. Every file visible to a cloud AI is explicitly selected, classified, and
   exported through the context/export gate. No whole-disk or whole-workspace
   prompt stuffing.
7. Recovery can ignore all workspaces and media projects and still restore
   last-good services.
8. The permanent core gains no compiler, package manager, archive parser,
   codec, media container parser, JIT, or GPU command builder.

## Target architecture

```text
user / agent request
  -> workspace service (project-scoped overlay)
  -> immutable source-tree revision + content hashes
  -> optional acquisition service -> quarantine only
  -> source viewer / search / diff / explicit provider export
  -> builder (network disabled, pinned toolchain + dependencies)
  -> Wasm/native candidate + build receipt
  -> Shadow tests + computed capabilities + user review
  -> existing promotion / persistence / rollback chain

video editor Wasm shell
  -> coarse media-job capability calls
  -> isolated native media engine
  -> codec / SIMD / GPU services
  -> opaque file, frame, audio, and GPU-buffer handles
```

## 1. Project workspace

Do not build a general filesystem first. Add a project namespace over the
existing content-addressed blob/structured-store mechanisms:

- `project_id`: stable opaque ID, never a host path.
- immutable blobs addressed by SHA-256;
- immutable tree manifest mapping normalized relative path to blob hash,
  classification, byte length, and media type;
- revision manifest containing parent tree hash, author/action evidence,
  timestamp basis, and tree hash;
- one bounded copy-on-write overlay per active agent task;
- commit creates a new immutable revision; cancel drops the overlay;
- quotas for projects, files, path length, total source bytes, dependency
  bytes, build outputs, and revision count.

Paths are UTF-8 relative paths only: no absolute paths, `..`, device names,
alternate streams, case aliases, hard links, or symlinks in v1. Blob storage
does not carry an executable bit. A workspace revision can be deleted or
garbage-collected but cannot become active code without promotion.

Source revisions and build outputs use different roots. A build cannot rewrite
its source revision. Generated files land in a new derived-output tree.

## 2. Acquisition and quarantine

Reuse ADR 0009's source-agnostic content-addressed acquisition contract.

The acquisition request names:

- requesting project and agent action;
- source kind (`local_serial`, `owner_usb`, later `https`);
- locator classified as local-only unless explicitly public;
- expected hash/signature when known;
- maximum bytes, redirects, file count, expanded size, and deadline;
- whether source-provider export is allowed (default false).

Received bytes enter a quarantine blob namespace. The service records observed
hash, size, transport trust, origin, redirects, publisher evidence, and denial
reason. It grants no build, execute, install, provider-export, or persistence
authority beyond storing the quarantined blob in the approved workspace store.

Archive extraction is a separate sandboxed service. It rejects absolute paths,
parent traversal, links, duplicate/case-colliding paths, special files,
compression bombs, excessive nesting, unknown encodings, and quota overflow.
Extraction produces a candidate tree manifest; it never writes arbitrary paths.

Network fetch is not the first slice. Local signed registry/serial and explicit
owner file import prove the final quarantine path first. HTTPS opens only after
the existing network-service import boundary plus production trust/time gates
are positive. Credentials are supplied as one-use Vault leases and are never
returned to the workspace or fetch service.

## 3. AI and user inspection

The workspace service exposes only bounded project operations:

- list tree/revisions;
- read a byte range from one source blob;
- search text with capped results;
- show revision diff;
- create/replace/delete one overlay path;
- commit or discard the overlay;
- request import, build, test, or promotion.

Every mutation is project-scoped and audit-bound. The AI never receives a raw
block-device or generic filesystem handle.

Genesis needs a source review view with tree, file, diff, dependency/lockfile,
origin, classification, and exact revision hash. Before promotion the user can
inspect the exact source revision and exact generated artifact hash.

For a cloud AI, source remains `local_only` by default. The context broker sends
only selected ranges needed for the current action, with file/revision locators
and token budgets. Secret scanning and classification happen before export.
Search/index results are locators, not authority. A local model may read through
the same API without a cloud export, but receives no broader capability.

## 4. Languages and builder trust

Use the smallest language tier that fits:

1. `RAIOS_UI_SPEC_V1` for bounded UI/rule programs.
2. Rust compiled to `wasm32-unknown-unknown` for general application logic and
   replaceable services.
3. Existing audited Rust/C/C++ libraries only inside signed native services for
   codecs, SIMD, GPU, and other measured hot paths.
4. Native Rust for drivers and permanent substrate changes, through the stricter
   core-update/reboot path, never the normal app installer.

The AI authors readable source, not Wasm binary or WAT. Direct opaque Wasm may
still enter external candidate quarantine, but it cannot satisfy a policy that
requires user-reviewable source and source-to-artifact evidence.

The builder is an untrusted producer, not an authority. Its receipt binds:

- source tree/revision hash;
- dependency lock and every dependency blob hash;
- compiler/toolchain artifact hash;
- target, flags, environment contract, and build-script policy;
- stdout/stderr hashes and output artifact hashes;
- reproducibility result and test-report locators.

Build inputs are read-only, outputs use a fresh namespace, networking is off,
clock/random/environment are fixed or recorded, and dependencies must already
exist in the quarantined content-addressed cache. `Cargo.lock` is mandatory;
build scripts and procedural macros require explicit sandbox policy.

The durable builder contract supports multiple implementations:

- first: an owner-controlled workstation builder feeding the existing signed
  candidate channel, honestly labeled `builder_attested_not_local_rebuild`;
- later: an on-device replaceable builder service after the native service/toolchain
  substrate exists;
- optional: two independent reproducible builders whose matching output hash
  raises evidence quality but still does not self-authorize promotion.

This is not a throwaway path: remote, workstation, and on-device builders all
produce the same non-authorizing receipt and candidate contract.

## 5. Install and runtime

The build output reuses the existing chain:

```text
candidate bytes
  -> recomputed hash + Wasm validation
  -> signed artifact/load descriptors
  -> exact import request
  -> Shadow VM tests bound to candidate + hardware profile
  -> computed grant
  -> physical exact-hash approval
  -> promotion transaction + artifact store
  -> boot-2 re-verification
  -> rollback target retained by hash
```

Workspace state and application state are separate. A promoted app receives a
versioned state namespace only through an explicit storage capability and state
migrator. Deleting a workspace must not delete installed state; rolling back an
app must not silently roll back user media.

## 6. High-performance video editor

Do not run codec loops through the current interpreter and do not put codecs in
the kernel. Split control plane from data plane.

### Wasm editor shell

- timeline/project model, commands, undo graph, UI and orchestration;
- coarse asynchronous calls such as `probe`, `decode_range`, `render_preview`,
  `analyze_audio`, and `export_timeline`;
- no raw DMA, GPU MMIO, codec pointers, or arbitrary filesystem access;
- project state stored through versioned project/media capabilities.

Wasmi is sufficient while these calls are coarse. Do not add JIT/AOT merely in
anticipation. Profile first. AOT belongs only after a native isolated-service
domain exists and Wasm control logic is a measured bottleneck.

### Native media engine

- signed, replaceable, versioned native service outside the permanent core;
- reuse FFmpeg/libav or narrowly selected existing codec/container libraries;
- CPU SIMD fallback plus hardware decode/encode where the bonded device supports it;
- strict parsers, quotas, cancellation, fuel/time budgets, and crash isolation;
- codecs and demuxers run with media-read/buffer capabilities, never project,
  Vault, provider, installer, or recovery authority.

### Handles and zero-copy

Wasm receives opaque handles, never pointers:

- `media_file`, `decode_session`, `frame`, `audio_block`, `gpu_buffer`, `job`;
- generation counters prevent stale-handle reuse;
- scoped leases define owner, read/write direction, byte range, format, lifetime,
  and cancellation;
- large frames remain in native/IOMMU-confined buffers;
- preview/compositor/encoder transfer handles, not copied RGBA frames;
- every DMA buffer is driver-owned and mapped only to the exact device/domain.

Media assets belong in a bulk file/media service, not the audit log or source
workspace CAS. The project stores stable media locators plus content fingerprints.
Original media is immutable by default; exports create new files atomically.

### Performance order

1. Correct CPU reference decode/export with bounded native service.
2. Coarse job API and cancellation.
3. Zero-copy native frame/audio handles.
4. GPU preview/compositing.
5. Hardware decode/encode for the exact bonded hardware.
6. Only then profile Wasm control overhead and consider AOT.

The largest gains come from codec/GPU reuse and avoiding copies, not from asking
the AI to emit lower-level Wasm.

## Vertical capability slices

Each slice ends with something a user or agent can do, not a schema-only gate.

### W1 - durable inspectable source revision

A user can import a bounded local source bundle into the QEMU structured store,
reboot, and inspect the same immutable tree and file hashes in Genesis. Foreign
media, path escapes, archive bombs, and quota overflow deny without partial trees.

Status: complete for direct bounded multi-file serial import (no archive
extraction yet), verified by `shadow-20260712-124220-8296.json` at 76/76.

### W2 - project-scoped AI editing

The AI can list/read/search one project, propose file edits in an overlay, and
the user can inspect and commit or discard the exact diff. It cannot see another
project, secrets, raw storage, or unselected cloud-export content.

Status: W2a list/read/search complete, verified by
`shadow-20260712-125335-27844.json` at 136/136. W2b is also complete: an exact
latest-revision-bound RAM overlay can add/replace/delete, expose its sorted
old/new hash-bound diff, discard without mutation, or commit one immutable
`agent_overlay_commit` child that survives reboot. Focused report
`shadow-20260712-130758-7668.json` passes 304/304 across 114 commands and three
boots. W2 is complete; direct cloud-provider tool use and provider export remain
closed.

### W3 - dependency quarantine

A user can import a locked dependency bundle, inspect origin/license/hash, and
bind it to one source revision. No dependency executes or runs a build script.

Status: complete for bounded local-serial exact-version package import. The
verified package bound owner-declared origin/license and the exact `Cargo.lock`
blob to one immutable source revision, included `LICENSE`, detected-but-never-run
`build.rs` and a greater-than-24-KiB multi-chunk `src/lib.rs`, survived reboot
with exact file/chunk/tree/bundle hashes, and re-imported idempotently without
writes. The source revision remained byte-identical. Focused report
`shadow-20260712-135131-25884.json` passes 600/600 across 214 commands and three
boots. Its harness child completed green 17 seconds after the outer 900-second
host wait expired, without retry or code change; the timeout was host wall-clock,
not guest behavior. This does not claim Cargo semantic parsing, verified
origin/license truth, archive extraction or network fetch. Network/export,
build-script execution, compiler, build, install, load and execution remain
denied.

### W4 - real offline Rust-to-Wasm build

An owner-controlled builder compiles one reviewed Rust project with networking
disabled and returns a Wasm candidate plus complete build receipt. Rebuilding
the same inputs reproduces the output hash or promotion denies.

Status: complete for one bounded Rust `cdylib` plus one safe quarantined local
path dependency. The workstation exact-read only the reviewed revision/bundle,
built twice `--frozen` and `--offline` with exact flags/environment and a pinned,
measured toolchain, and returned identical validated inert current-boot Wasm
candidate
`sha256:05854c56665a9fee9990712126e1f19269059375cb37fcdccacaa990ab3d30fb`
plus an inspectable receipt. Focused report
`shadow-20260712-145618-13408.json` passes 248/248 across 108 commands, one boot
and 313118 ms; report SHA-256 is
`e7fd8bf954e2b3b75af384d9215d13be7067316dd7e4cb47c5a1c332340e556c`.
The receipt remains `builder_attested_not_local_rebuild` and
`independently_verified=false`; no owner-sealed-toolchain or independent-rebuild
claim is made. Wrong toolchain/flags/environment, stale/wrong reads, source and
dependency `build.rs`, missing/failed/mismatched runs, output/candidate mismatch,
and receipt staleness after dependency mutation deny without authority. Install,
load, execution, promotion and persistence remain closed.

### W5 - tested current-boot application

The user can run that exact candidate current-boot under computed Wasm imports,
with Shadow evidence, physical approval, F12 recovery, and crash/fuel fallback.

Status: complete. Focused report
`shadow-20260712-153736-17972.json` passes 276/276 across 112 commands and one
boot. It proves the exact W4 receipt/candidate, locally observed zero imports,
core preview plus physical pointer approval, result 42 under fixed limits,
inventory/health, replay denials, and F12 cleanup/recovery.

### W6 - durable install and rollback

The approved app, source/build receipts, grant, state schema, and previous hash
survive reboot; a failed boot or explicit rollback restores last-good without
modifying source workspaces or user media.

Status: active next slice. ARTSTOR/RECLOG wiring, boot autoload, physical install
approval and focused VM evidence remain absent; no W6 authority is claimed yet.

### W7 - quarantined network acquisition

After the trust/time/import gates open, the user can approve one bounded HTTPS
source request. The result appears only as a quarantined hash/tree for review;
download still cannot build or install automatically.

### V1 - CPU media vertical slice

An editor Wasm shell imports one small video, requests native metadata and a
bounded CPU-decoded preview, makes one cut, and exports a new file. Original
media remains byte-identical; malformed media crashes only the media service.

### V2 - zero-copy preview

The same editor previews frames through opaque native buffer handles with no
full-frame Wasm copy, enforced lifetimes, cancellation, and bounded memory.

### V3 - hardware acceleration

On the exact bonded GPU/codec device, preview and export use measured hardware
decode/encode while IOMMU/buffer evidence proves the editor and codec cannot DMA
outside their granted buffers. CPU fallback remains correct.

## Parallel agent lanes

Agents may run together only with disjoint files and one integrated capability
per lane:

| Lane | Ownership | Result |
| --- | --- | --- |
| Workspace/CAS | core workspace/tree/blob modules | immutable revisions, quotas, host tests |
| Acquisition | fetch/quarantine/archive service | inert bounded import with provenance |
| Builder | toolchain/receipt/candidate bridge | reproducible source-to-Wasm candidate |
| Runtime | Wasm imports/state/promotion adapters | exact grant, run, rollback |
| Media engine | native media/job/buffer service | codec/GPU path with opaque handles |
| Genesis UI | project/source/diff/media views | physical review and approval |
| VM harness | focused workspace/builder/media profiles | positive behavior plus fail-closed cases |
| Docs | status/roadmap/dashboard after evidence | honest capability and remaining gaps |

No two agents edit `main.rs`, the same protocol registry, build scripts, or one
harness profile concurrently. The orchestrator owns integration, full diff
review, evidence classification, commits, and milestone gates.

## Verification cadence

- Parser/tree/diff logic: focused host tests first.
- Workspace persistence, builder receipts, install, rollback, recovery,
  capability grants, network trust, or DMA: focused VM profile at each boundary.
- Batch UI/docs/refactor-only changes; do not batch across the risky boundaries.
- Media performance evidence records throughput, latency, copies per frame,
  peak buffers, dropped frames, CPU/GPU path, cancellation latency, and output hash.
- Before durable install or physical release: relevant focused profiles, Full,
  Recovery byte-identical where required, power-cut test, and secret scan.

## Owner decisions required before implementation

1. First source import: local serial/USB bundle (recommended) or wait for HTTPS.
2. Workspace storage target: QEMU structured store first; physical partition only
   after the existing exact identity/write ceremony.
3. Cloud source export: per-file confirmation (recommended) or project policy;
   default remains no export.
4. First builder: owner workstation with receipts (recommended) or wait for an
   on-device native toolchain service.
5. Native media engine dependency: FFmpeg/libav bundle versus a narrower codec set;
   licenses and update provenance must be accepted explicitly.
6. First media hardware target and driver/API; do not promise generic GPU support.

## Definition of done

The workspace goal is complete only when an imported or AI-edited source tree is
reviewable, immutable by revision, persistent across reboot, secret-free,
project-scoped, reproducibly built into an inert candidate, tested, physically
approved, promoted through existing gates, and rolled back by exact hash.

The video-editor goal is complete only when a real project imports real media,
previews, edits, saves, reopens after reboot, exports a byte-verifiable result,
survives malformed media/service crash, and recovery remains available. Maximal
performance additionally requires measured zero-copy handles and hardware
acceleration on the named device; it is not claimed from Wasm isolation alone.

## Explicit non-goals

- No generic POSIX filesystem or Linux layer under raiOS.
- No compiler, package manager, archive parser, codec, FFmpeg, or JIT in the core.
- No AI-generated raw Wasm as the normal authoring path.
- No automatic install after download or build.
- No workspace access to Vault plaintext or provider credentials.
- No full-workspace cloud export by default.
- No custom codec, custom GPU API, or general-purpose build language until an
  existing solution demonstrably cannot satisfy the bounded service contract.
