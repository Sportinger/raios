# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~18:45, loop running)

**03:46 PROVEN (280/280): rollback of one domain leaves foreign durable
state bit-identical** — unpark succeeded via the per-record durable-scan
kernel lane (4e72924) + real memory-append seeds; box checked. Earlier
today: **hello.wasm compiled ON raiOS hash-sealed** (06:72+74), §7 whole
section closed, §4 PCI box, §2 storage boxes (both environments). 24
commits, all pushed. §3 remaining: 03:47 (fewer-grants delta — needs §2
grant/revoke first), bare-metal boundary runs + distribution phase
(owner-gated), isolation-suspicion protocol box.

## Next step

Next buildable frontier is §2: typed grant/revoke group (02:22-27, incl.
revocation-stops-next-call negative) and lifecycle (<1 s kill/restart,
crash-loop parking, 02:41-45) — scout first (the m11-wasm-import-grant
machinery is the base; what exists vs. missing for revoke + restart
timing). Then 03:47 rides on revoke. Also open: §1 fuel/F12/watchdog
boxes, §4 fabric rows. Owner-gated: §5/§6 pre-ADR-0005 wording reframe;
bare-metal escape run (Surface); unattended-loop hardware (money).

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
