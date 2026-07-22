# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, H26 codec secured)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`1546fcf`. The detached old root `C:\Users\admin\Documents\raios2` remains
foreign WIP; never clean, reset, merge, or integrate it. Worktree is clean.

The `b18f272` Surface USB restored Genesis, HID and the WLAN UI. Scan and
passphrase entry reached the designed H25 terminal dialog `reboot_required`.
Returned-stick extraction found six valid RECLOG frames, clean zero tail and
USB `errors=0`. The post-PMK GET_HW_SPEC canary completed in the current epoch;
network stayed denied and cold reboot required exactly as H25 specifies. This
rules out a bad password and proves generic post-PMK mailbox liveness.

`1546fcf` secures the hardware-independent H26 protocol slice. Legacy scan
responses now require one unambiguous TSF TLV with one firmware TSF per BSS.
The source-compatible `build_associate_24ghz_with_tsf` emits `0x0113`, length
16, firmware TSF and AP beacon timestamp. The H25 callsite stays unchanged.
All 37 focused tests, negatives, diff checks and independent review are green
(`ACCEPT`); the stable host check reached the callsite with `E0063=0`.

## Next step

H26 kernel wiring must retain both timestamps, restore PMK -> Associate and use
the TSF builder. ADR 0027 forbids this Surface hardware lane while
`surface-pro-4.v1.json` lacks required observed CPU/memory facts. Owner must
authorize an explicit exception ADR for this device/test or supply a valid
observed manifest; until then no H26 stick image is authorized.

## Recently (exactly 3, newest first)

### 2026-07-22 - Compatible H26 TSF codec secured
`1546fcf`: scan TSF parsing plus explicit Associate TSF builder; 37 tests and
independent ACCEPT green, legacy kernel callsite remains source-compatible.

### 2026-07-22 - H25 reached its designed terminal state
Genesis/WLAN returned; canary completed, USB stayed clean, and network grant
was intentionally denied with reboot required.

### 2026-07-22 - Unsafe SMBIOS access rejected during boot
`b18f272`: fail-closed capture policy bypassed the deterministic SI fault and
restored the EB2 -> USB -> Genesis path.
