# 0017 — WASI build-guest determinism and storage contracts

Date: 2026-07-18 · Status: active (owner veto window open — flagged in HANDOFF)

## Context

The WASI preview1 slice plan (plan-personal-rust-playground.md §6, planned
with an xhigh second opinion) left five design questions open with
recommendations. None touches an owner-reserved domain (SCOPE, money,
hardware, secrets); slices 3-5 block on them. Decided here as recommended,
with the reasoning recorded; T2's thread contract was already fixed in
ADR 0016.

## Decisions

1. **BuildFS format = chunk-CAS.** `BuildFsManifest v1`: canonically sorted
   directories and files; per file total length, total sha256, and an
   ordered list of 64-KiB content-addressed chunks. Range reads verify
   chunk hashes without materializing whole files — the 71-MB sysroot must
   never be loaded wholesale into the kernel heap. Rejected: packed image
   with index (simpler, but inherits the full-frame materialization of the
   current artifact readback).
2. **Guest realtime = fixed epoch.** `realtime_ns = 946_684_800e9
   (2000-01-01T00:00:00Z) + monotonic_ns`, where monotonic time derives
   from the job's fuel/pump counter (ADR 0016 virtual clock). No
   SOURCE_DATE_EPOCH manifest binding — real, auditable time stays outside
   the compiler guest. Known risk, accepted: rustc-internal freshness
   logic sees fake time; irrelevant for single-invocation builds without
   incremental caches, revisit if incremental ever lands.
3. **Root-tmp policy = any new root child, RAM-only, quota'd.** rustc
   creates its temp directory directly under `/`; prefix filters would be
   toolchain-fragile. Reserved names (`/sysroot`, `/src`, `/out`, `/tmp`)
   can never be created, replaced, or shadowed.
4. **Egress buffering = RAM until double-build equality.** `/out` freezes
   into a sorted manifest; only two byte-identical output manifests from
   the double build produce an egress plan; nothing persists before that.
   A scratch-persist path is deferred until measured RAM pressure and then
   needs its own delete/GC policy.
5. **Build receipt = new v2.** Single-`rustc`-invocation on-device builds
   get their own receipt version; the existing cargo/werkstatt receipt v1
   semantics are never reinterpreted.

## Alternatives & second opinions

The five recommendations originate from the read-only planning lane
(xhigh); each lists its alternative inline above. No dissent between the
planner and the orchestrator; the owner can veto any of the five before
the affected slice ships to hardware.

## Consequences

Slices 3-5 are unblocked with concrete contracts. Deterministic fake time
and predictable `random_get` mean the Bauplatz must never be used to
generate keys, tokens, or other secrets — that boundary belongs in the
Bauplatz policy text (slice 4/6) and in review checklists.
