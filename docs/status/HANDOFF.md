# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~21:50, loop running)

~24 SCOPE boxes closed this session; all pushed. Green: §7 whole section;
§6 hello.wasm (hash-sealed) + granted-range; §4 PCI + 3 lane-rule boxes;
§3 rollback-isolation (280/280); §2 storage×2 + zero-grant-create +
cross-service + A-grants-nothing (capability/host_import.selftest, quick
509/509 both disk modes); §1 catalog, fuel-metered, F12, malformed-import,
negative-matrix, kernel-owns-hardware, device-import-denied. ADR 0023
(revocable grants via one gate, dual second opinion) written; its Slice 1
(env imports through the single ImportGate, pass-through) landed 85602a0,
verified 509/509 byte-identical.

## Next step

ADR-0023 Slice 2 (the security-critical core, in progress): in-memory
per-domain grant table + make env.counter_get revocable + a service.revoke
method flipping the slot + the gate consulting it before the counter
effect + the decisive negative (grant→call→revoke→SAME instance's next
call denied, host_effect=0, peer surface still works). Durable chain +
boot re-fold = Slice 3; rollback-delta = Slice 4; migrate the other 74
func_wrap sites = Slice 5. Slice 2-4 then close 02:11/22/25-26 + 03:47.
Migration is EXCLUSIVE-lane, gated on full quick staying 509/509.

## Recently (exactly 3, newest first)

### 2026-07-19 — grant/revoke architecture decided + Slice 1 laid
ADR 0023: one enforcement gate over a kernel-owned per-domain grant table
that is a fold of the append-only grant/revoke chain (Codex+Fable
concurrence, 1 recorded sub-decision on the suspended-call gap). Slice 1
routes env imports through ImportGate (pass-through), 509/509 unchanged.

### 2026-07-19 — §1 evidence sweep: 7 boxes on selftest proofs
capability.selftest (zero-grant create + cross-service refusal),
host_import.selftest (6-case malformed matrix + device-absent, all distinct
typed reasons, zero effect), lifecycle HEAD re-run (fuel-metered + F12),
catalog. A no-persist-disk invariance bug in the selftests, caught by the
loop's own verification, fixed (e9af257); both quick modes 509/509.

### 2026-07-19 — hello.wasm: raiOS compiles a real program on itself
Denied-open capture (7e5a55a) exposed the last wall: lld's output is
O_CREAT|O_EXCL. EXCL support (d116e01) + temps→/tmp (0e90e78) → exit 0,
out_files=1, sha bc5b7311…, 0 denials, 0 stderr. 06:72+74 checked.
