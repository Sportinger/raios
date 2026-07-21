# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~21:15, root orchestrator active)

Product line `product/h20-surface` holds both strands. Wi-Fi strand: pinned
firmware `1f061b1`, fail-closed WiFi `fc26cd5`, docs `29f14f9`; cold runs
proved firmware upload, live scan, WPA2 selection + passphrase entry.
**Full brake ACTIVE (ADR 0034):** H20 `Starting WiFi` coincided with total
HID loss, cause unproven — no Marvell ring tests, no WiFi checkbox closure;
the §3 suspicion box records it.

Audit strand: A69/A70 accepted. json-diag needles re-verified (positive
exit 0; planted E0308 → exit 1 @ `src\lib.rs:3:24`); scope-04 narrowed.
All four distribution boxes stay open (partial/partial/partial/missing);
gap map: S1 signed pre-exec, S2 independent rebuild, S3 grant-audit query,
S4 trigger ADR (owner decision). Bare-metal stick READY: `target/wt-bm-head`
@ `09751a7` (deliberately pre-WiFi-firmware), kernel `d4fbd3e6…`, signed.

Parked/owner-blocked: W59 checkpoint reset; rollback Rust verifier; NET8
Schannel; agent-fabric SCOPE wording. origin/main moved to `dc22477` (PCI
strand) — not this loop's line. W59 WIP, rollback Python WIP, NET8 WIP,
formatter-only `main.rs`, `release/diagnostics/`, fixture `Cargo.lock`
remain taboo.

## Next step

1. Owner: bare-metal escape-negative run — stick write from
   `target\wt-bm-head` (`-SkipBuild`), then
   `isolation.selftest` on the Surface console; photograph both
   RAIOS_ISOLATION lines. Settles the §3 QEMU+bare-metal box AND feeds the
   brake's escape-negative settlement.
2. Wi-Fi strand: Marvell DMA read-only review + negatives (bounds,
   ownership, non-overlap, indices, lifetimes, xHCI/kernel/heap/RECLOG
   separation); cold-boot retest only with independent acceptance plus
   owner authorization.
3. No new implementation lanes while the brake is active; S1/S3 stay staged.

## Recently (exactly 3, newest first)

### 2026-07-21 — audits A69/A70 accepted, stick built
json-diag verified + narrowed; distribution gap mapped (S1–S4); stage0 build
staged for the owner.

### 2026-07-20 — H20 full brake + product code secured
Pinned firmware `1f061b1`; fail-closed WiFi `fc26cd5`; ADR 0034 stops
Marvell ring tests after the HID-loss freeze.

### 2026-07-20 — serial RECLOG contract secured
R68 accepted the repaired protocol; `35191de` merged with the USB tip after
green docs and boundary gates.
