# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~18:25, loop running)

**§6 CORE PROVEN: hello.wasm compiled ON raiOS, hash-sealed** —
`exit=0 reason=none out_files=1 out_sha=bc5b7311…` (shadow-…-172854,
507/507; boxes 06:72+74 checked b8c573c). Today also closed: §7 whole
section, §4 PCI box, §2 storage boxes (both environments). §3 box 03:46
PARKED: 3 strategies exhausted (no durable service-B in image; external
persist disk forbidden by design; no real foreign-persist vocabulary —
artifact selftest is writes_persistent_state=false, run …-175703 157/158).
Root gap: kernel lacks per-record durable-scan evidence (B4 report).

## Next step

Unblock 03:46 via a kernel lane: read-only per-record RECLOG scan evidence
(mirror artifact.store_scan's records[]; family+seq+hashes, bounded) →
then rollback-isolation's seed (memory.observation_log_append is the one
REAL agent persist) + records-B observe it independently. Profile is
committed and fails closed at iso:B_seed until then (53b6a31). Also open:
§2 lifecycle (<1 s restart), §1 fuel/F12/watchdog boxes, §4 fabric rows.
Owner-gated: §5/§6 wording reframe; bare-metal run; loop hardware.

## Recently (exactly 3, newest first)

### 2026-07-19 — hello.wasm: raiOS compiles a real program on itself
Denied-open capture (7e5a55a) exposed the last wall live: lld creates its
output O_CREAT|O_EXCL. EXCL support (d116e01) + temps→/tmp (0e90e78) →
exit 0, out_files=1, sha bc5b7311…, 0 denials, 0 stderr. 06:72+74 checked.

### 2026-07-19 — §3 parked on a real evidence gap
Three B-strategies dead: no durable service in image; external disk
forbidden; no real foreign-persist command (selftests write nothing).
157/158 green run; profile fails closed at iso:B_seed. Unblock = kernel
per-record durable-scan lane, queued first for the next iteration.

### 2026-07-19 — §7 closed; §4 introspection + §2 storage negatives land
Rule 12 breakdown-consistency (red paths self-tested) → §7 all green.
device.graph carries PCI IDs/BARs/IRQs + pci_functions; fabricated PCI
fails. storage.selftest: absent/range/quota denied, disk hashes unchanged
in both quick+persist (507/507) and native persistence (47/47).
