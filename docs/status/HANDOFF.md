# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~15:45, loop running)

§7 CLOSED (first full top-level section, 85b931b: breakdown-consistency
rule 12 + all groups green). §4 PCI-introspection box green (d59aecc,
504/504). §2 storage.selftest landed (d18bcc0) — VM evidence in flight.
§6: rustc runs the whole pipeline to the LINK step (rmeta+object land in
/out); rust-lld fails only creating /out/hello.wasm (os76) — host
hypotheses falsified (28098c9), live-args diagnosis running. §3 profile
built + crash-proofed (8613708); donor spine 256/256 live-green; needs an
independent durable domain B (per-run image has none — B2 report).

## Next step

Read VM run 3 (quick+persist, running): §2 disk=pass markers, §4
regression, RAIOS_RUSTCDIAG fd/errno of the failing lld open → dispatch
surgical §6 fix (wasi_preview1.rs / one-shot arg capture if ambiguous).
Then: persistence profile -KeepImage (proves §2 disk invariance AND mints
the durable-domain persist disk) → rerun rollback-isolation with that disk
attached (B = seeded durable service; filter exists, 8613708). Owner-gated:
§5/§6 pre-ADR-0005 wording; bare-metal escape run; unattended-loop hardware.

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
