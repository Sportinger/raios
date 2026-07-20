# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-20 ~14:50, root orchestrator active)

HEAD/origin is `9f633d3`. The complete eight-file ADR-0029 RAM-only
recovery slice passed R35 and is pushed as `d094201`; failed execution clears
volatile authority, post-commit mismatch denies, and diagnostics use local
state. ADR 0030's per-generation grant-authority rule is pushed as `9f633d3`
after two neutral Codex opinions agreed.

The image half of `docs/scope/03-security-trust-pipeline.md:55` is
**owner-blocked (security stalemate)** after multiple failed strategies. R38
proved the Python model rejects real Rust authorization IDs, equates cursor
`install_log_sequence` with generation, and omits linked promotion/persist/blob
validation required by ADR 0030. Preserve the rejected uncommitted image
builder/helper/test; do not dispatch a fourth variation. Owner must authorize a
Rust-produced verifier/export plus file expansion, or change the evidence
contract.

NET8/QEMU is owner-blocked: UserKeySet and direct-certificate strategies both
hit Schannel `SEC_E_NO_CREDENTIALS`. Keep the rejected NET8 wrapper/Program
unstaged. Live rollback needs an owner-provided usable Windows TLS credential
provider or authorized alternative. Surface closure needs owner hardware facts.

Other dirty ownership: persistence harness awaits review; `seed-kernel/src/main.rs`
is inherited formatter-only work; diagnostics and the experiment lockfile are
foreign. All are taboo outside an exact lane.

## Next step

Run disjoint unsafe-inventory implementation and read-only persistence-harness
assessment lanes. Secure each accepted exact slice immediately. Keep blocked
image/NET8/QEMU/surface work and all foreign dirty files out of staging, then
update only genuinely green mapped boxes and continue by dependency/value.

## Recently (exactly 3, newest first)

### 2026-07-20 — generation authority recorded
R36/R37 selected per-generation signed authority; ADR 0030 is `9f633d3`.
R38 parked the incompatible Python image model with owner.

### 2026-07-20 — RAM-only recovery secured
Core 677+5 and the kernel host check passed; R35 accepted the eight-file slice,
pushed as `d094201`.

### 2026-07-20 — image helper foundation secured
The P-256/GPT/framing/recovery helper remains committed as `5f6bb10`; later
unaccepted integration/authority WIP is preserved but red.
