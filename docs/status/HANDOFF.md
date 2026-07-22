# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, H26 R3 blocked)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`c64a975`. The detached old root remains foreign WIP; never clean, reset,
merge, or integrate it. Four uncommitted files belong to the stopped H26 lane:
`seed-kernel/src/{wifi.rs,marvell_wifi_pcie.rs}` and the two focused WLAN
predicate scripts. Preserve them exactly; do not commit, package or discard.

The one-time R3 recovery dispatch completed. Both focused executable models,
63 Marvell tests, 16 DMA tests, unsafe selftest, rustfmt, diff check and the
root freestanding release build are green. Acceptance is nevertheless blocked:
the independent review proved that two starts from an existing Ready state can
carry stale `replace_ready=true`; a loser can then quiesce a published winner
or erase its Ready snapshot, data-link and NET_STATE. Both models cover only an
Idle origin and miss this interleaving. Two fresh independent read-only
opinions materially confirmed the race and denied hardware release.

ADR 0045's explicit R3 stop condition now applies. Recovery claim
`target/state/adr0045-h26-r3-dispatch-recovery-1.claim` is consumed. No unsafe
baseline update, product commit, image package or USB write occurred.

## Next step

Owner decision required: leave H26 parked, or explicitly authorize a changed
strategy/new narrowly scoped lane for Ready-replacement ownership plus models.
No automatic R4, new claim, image or hardware test is allowed. Independent
non-H26 scope may continue meanwhile.

## Recently (exactly 3, newest first)

### 2026-07-22 - H26 R3 rejected at Ready replacement
Review plus two independent opinions proved stale Ready quarantine can destroy
a concurrent winner; hardware write denied despite green build/tests.

### 2026-07-22 - R3 product worker completed
Atomic Idle-start publication and terminal lease models landed in the preserved
four-file WIP; all root predicates and freestanding release compile passed.

### 2026-07-22 - Recovery launcher secured
`c64a975` bound OEM-850 failure evidence and the audited single-use Windows
dispatch; the recovery claim then started exactly one R3 worker.
