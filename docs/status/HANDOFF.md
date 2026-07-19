# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~17:45, loop running)

**rustc COMPILES CLEAN ON-DEVICE: exit 0, 0 denials, 0 stderr, 297 KB out**
(shadow-20260719-171059; EXCL was the last wall, chain in STATUS). §7
CLOSED; §4 PCI box green; §2 storage boxes green in BOTH environments
(quick+persist 507/507, persistence native 47/47 after fixture resize
c0fe74f + per-command timeout e5d315f→062c9e2-rebase). §3: donor spine
green; B-seed persist disk minted (raios-shadow-20260719-170525). Remote
gained website-deploy commits from the other session (rebased over).

## Next step

Two closing lanes running: (a) §6 temps-dir argv (-Ctemps-dir=/tmp) so
/out holds exactly hello.wasm → completion contract computes out_sha →
rerun quick+persist (temp rustcbuild patch in tree, uncommitted) → check
§6 box 06:72. (b) §3 records-family fallback for domain B → rerun
rollback-isolation with -Network -PersistDiskPath
<temp>\raios-shadow-20260719-170525-6196\raios-persist-gpt.img → check
03:46. Owner-gated: §5/§6 wording reframe; bare-metal run; loop hardware.

## Recently (exactly 3, newest first)

### 2026-07-19 — §7 closed; §4 introspection + §2 storage negatives land
Rule 12 enforces top-level-vs-breakdown consistency (red paths self-tested)
→ all §7 groups green. device.graph carries PCI IDs/BARs/IRQs +
pci_functions walk; fabricated-PCI-for-absent-hardware fails. storage.
selftest: absent grant/out-of-range/quota denied + full-disk hash equality.

### 2026-07-19 — rustc reaches the LINKER on-device
Writable-arena attenuation (ccb31b2) tore down the rmeta create wall:
codegen completes, out_files=2, rust-lld runs and fails only on
/out/hello.wasm (735 rounds). Preopen/from_bits hypotheses host-falsified
(28098c9, 67/67). Frontier = guest-exact open args; diag run in flight.

### 2026-07-19 — rollback-isolation: spine green, B missing
New §3 profile mirrors m6d verbatim + 3 isolation predicates; live run:
256/256 donor predicates green, then a PS 5.1 [ordered]/@() quirk killed
the run pre-predicate — fixed crash-proof (8613708). Real finding: per-run
image has NO independent durable domain B → seed via persistence disk.
