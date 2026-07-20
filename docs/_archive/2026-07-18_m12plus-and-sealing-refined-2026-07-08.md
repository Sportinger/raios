# M12+ Direction Validation + The Owner-Key Sealing Ceremony — Refined (2026-07-08)

Refined against HEAD `209c450`. Companion to
`m12-plus-direction-2026-07-06.md` (which remains the direction authority and
is NOT edited). Two parts: (1) M12+ direction reconciled with actual milestone
state; (2) the sealing ceremony made precise — the SINGLE owner-physical final
step of the whole roadmap.

---

## Part 1 — M12+ direction vs reality

**Owner answers on record (2026-07-06) stand unchanged** — and two of them are
now DURABLE memory records on disk (M9A-3b):
`mem.decision.module_sharing_confirmed_vision.current_boot.v0` superseded by
`mem.decision.module_sharing_evidence_gated.current_boot.v0` (sharing =
candidate intake, NEVER install). The direction doc and the machine now agree.

**Prerequisite state (2026-07-08):** M6 ✅ M7 ✅ M8 ✅ M9 nearly closed
(M9C-1b in flight; M9C-2 + M9D remain) — M10, M11 open.

| M12+ item | Gate per direction doc | Status now | First concrete slice? |
|---|---|---|---|
| 1. USB-Ethernet (Wi-Fi Option B, CHOSEN) | M7 + M8 (+M11 preferred) | **Technically UNBLOCKED** (M7/M8 closed) | Design-map authoring only — see below |
| 2. External distribution / module sharing | M6 + M7 + M10 (+M11 pref) + **NEW ADR** | Blocked on M10 + ADR | ADR draft may be authored during M10C (no code) |
| 3. Re-binding | M7 + M9 | Blocked on M9 close (imminent) | Not before M10 (recommend: live in the M9 record world first — map's own advice) |
| 4. Core handoff / native graph | Everything, proven on hardware | Research-grade, unchanged | None |

**The FIRST concrete M12 slice (decisive):** the only unblocked item is
USB-Ethernet Option B, and per the direction doc's ORCHESTRATOR RULE, nothing
is dispatchable from a direction doc — the first slice is therefore
**authoring the USB-Ethernet design map** (CDC-ECM class driver on the existing
xHCI, QEMU `usb-net` emulation fidelity check, smoltcp attach point mirroring
e1000), and it requires explicit owner approval to open the lane. RECOMMENDED
sequencing: keep strict order (M9 close → M10 → M11); OFFER the owner the
option of authoring/executing the USB-Ethernet map as a PARALLEL lane during
M10 — the write set (usb.rs 2,503 lines / net.rs attach / a new profile) is
disjoint from every M10 provider file, so it parallelizes cleanly if the owner
wants hardware progress early. Do not open item 2's ADR lane before M10C
proves the second-provider transport.

**Standing M12+ tripwires unchanged** (new ADRs, trust-model changes,
destructive disk ops, secrets leaving RAM, physical-test procedures without a
power-cycle recovery path).

---

## Part 2 — The Sealing Ceremony (the ONLY owner-physical blocker)

### 2.1 What exists today (verified)

- The runtime promotion authority is a **deliberately public dev key**:
  `raios-core/src/promotion_attestation.rs:25`
  `PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SEC1` is the P-256 generator
  point — private scalar = 1, in-repo, anyone can forge. Honest by design
  under ADR 0007.
- The honesty latch: `PROMOTION_AUTHORITY_IS_PLACEHOLDER: bool = true`
  (`promotion_attestation.rs:3`) — consumed at ~20 sites
  (`durable_store.rs:604/:747/:1493/:1784`, `artifact_store.rs:322/...`,
  `repromotion.rs:327/:846/:1079`, `recovery_lifeline.rs:1165`) as the
  `promotion_authority_is_placeholder` evidence field.
- The trust-tier label `"dev_key_not_owner_sealed"` is pinned as separate
  consts at: `agent_protocol_module_grant.rs:451` (+ invariant `:866`),
  `durable_store.rs:80/:86/:98`, `granted_candidate_service.rs:74`,
  `repromotion.rs:44`.
- `owner_sealed` is a HARDCODED `false` literal at ~30 emission sites across
  `artifact_store.rs`, `durable_store.rs`, `granted_candidate_service.rs`,
  `memory_store.rs`, `recovery_lifeline.rs`, `repromotion.rs`,
  `agent_protocol_module_service_slot.rs:198`. **There is NO code path that
  sets owner_sealed=true — it "gets set" only by the ceremony's source change.**
- Dev signing tool: `ota/cli/src/bin/dev-promotion-signer.rs` (host, ota-tools;
  RFC6979-deterministic, same p256 crate the kernel verifies with) — signs
  attestation hashes with the public scalar-1 key. The two-boot proofs and all
  promotion flows run on it.
- Rotation model (ADR 0007): **rebuild + reflash** — the pinned public key is
  baked into the image; old images trust only their own key.

### 2.2 What the owner must PHYSICALLY provide (and nothing else)

1. **Generate P-256 promotion keypair K OUTSIDE the repository** (ADR 0007
   ratification pt 1). The private key never touches: build tree, boot image,
   logs, provider context, or this machine's disk (recommended: offline
   generation; storage is the owner's choice — hardware token / offline
   medium; a TPM-backed design would need its own ADR and is NOT required).
2. **Hand over ONLY the public half:** 65-byte SEC1 uncompressed public key +
   its SHA-256 fingerprint.
3. **Ratify** the exact pinned bytes/fingerprint that will replace the dev
   constants (ADR 0007 pt 2).
4. **Explicitly approve the ceremony** before any owner-sealed claim is
   emitted (ADR 0007 pt 3).
5. **Thereafter: sign promotions with K** using the owner-side signer tool
   (2.4-P2) whenever a new external module is promoted for real.

### 2.3 Where K plugs in (the exact ceremony diff)

1. `promotion_attestation.rs:25/:33` — replace SEC1 bytes + SHA-256 pin with
   K's public half.
2. `promotion_attestation.rs:3` — `PROMOTION_AUTHORITY_IS_PLACEHOLDER = false`.
3. The trust-tier consts flip `"dev_key_not_owner_sealed"` →
   `"owner_sealed"` and the `owner_sealed` literals flip — AFTER prep slice
   P1 below, this is ONE derived function, not ~35 scattered edits.
4. Delete/disable the `#[cfg(test)]` scalar-1 signing path's production reach
   (it is test-only already; the DEV signer bin gains a refusal to run against
   a non-placeholder build).
5. Rebuild, re-sign attested sources if touched, reflash
   (`scripts/write-stage0-usb.ps1`), harness needle set flips (see P3).

### 2.4 Pre-ceremony prep slices — ALL buildable NOW, no owner key needed

- **SEAL-PREP-P1 (grants nothing, hygiene):** centralize authority labels in
  raios-core — `pub fn promotion_trust_tier() -> &'static str` and
  `pub fn owner_sealed() -> bool`, both derived from
  `PROMOTION_AUTHORITY_IS_PLACEHOLDER`; replace the 5 tier consts and the ~30
  `owner_sealed:false` literals with calls. Serial output byte-identical while
  the flag is true (needles prove it). Makes the ceremony a single-const flip.
  Write set: raios-core + the 7 kernel files above. Verify: quick + recovery
  byte-identical. **Ready-to-scope: any time; ideal filler slice.**
- **SEAL-PREP-P2 (grants nothing, host tool):** `owner-promotion-signer` (new
  ota-tools bin, cloned from the dev signer): reads the private key from an
  owner-supplied file path/ENV (never embedded), REFUSES the known dev scalar,
  never prints key material, and `scan-secrets.ps1` learns its key-file shape.
  **Ready-to-scope: any time after P1.**
- **SEAL-PREP-P3 (docs + harness dry-run):** `docs/SEALING_CEREMONY.md`
  runbook: generate → ratify → apply diff (2.3) → rebuild → reflash →
  re-promote under K → verify. MUST answer the durable-state question
  explicitly: **a sealed image's re-verification REJECTS every dev-key-signed
  promotion transaction / persisted artifact** (correct, fail-closed — the
  signature no longer verifies against the new pin). The runbook therefore
  includes: enumerate persisted dev-tier artifacts, re-promote each under K
  (or explicitly discard), and the mechanical needle flip
  (`dev_key_not_owner_sealed` → `owner_sealed` across profiles, derived from
  observed serial, never hand-invented). Optional dry-run: a throwaway local
  image built with a TEST second key (never committed) walks the whole
  runbook end-to-end — proving the ceremony procedure itself, still without
  the owner's real K. **Ready-to-scope: after P1+P2.**

### 2.5 Confirmation: everything up to the ceremony is buildable WITHOUT K

CONFIRMED at HEAD. Every authority mechanism — grant, load, run, rollback,
durable promotion, reboot re-promotion, recovery re-instatement, durable
memory, export audits (M9C-2), provider hardening (M10), kernel slimming
(M11), even module-sharing distribution (M12, dev-tier) — runs fully under the
dev key with the honest `dev_key_not_owner_sealed` / `owner_sealed:false` /
`promotion_authority_is_placeholder:true` labels. The ceremony changes WHO the
trust root is, not HOW anything works. There is no other owner-physical
blocker anywhere in M9C-2/M9D/M10/M11/M12+ planning (owner-run live smokes
with provider key images are routine operations, not blockers).

### 2.6 Timing (decisive)

The ceremony is the FINAL step by design (owner dashboard promise: "your own
key K seals it for real later — the very last step"). Do NOT run it before
M10+M11 close: sealing early would force re-promotions after every
infrastructure-service change (M11 promotes its own plumbing) and would put
needle churn on every intermediate slice. Schedule: P1/P2 as filler slices any
time; P3 after M11; the ceremony itself when the owner declares the system
shape final enough to bond.

---

## Part 3 — Cross-milestone dependency graph (whole remaining roadmap)

```text
NOW ──> M9C-1b commit (in flight)
         ├─> M9C-2a → M9C-2b → M9C-2c (SECURITY flip + owner-run live smoke)
         └─> M9D (two-boot, read-only)            [parallel]
        M9 CLOSE (full + recovery byte-identical)
         │
         ├────────────── optional parallel lane (owner opt-in): USB-Ethernet design map + slices (M12 item 1)
         v
        M10-0 → M10A-1 ∥ (M10B-1→2→3) → M10C-1 ∥ M10D-1 → M10 CLOSE
         │        (export gate carried byte-identical; OD-1 owner key for Anthropic smoke)
         v
        M11-0 → M11-1 ∥ M11-2a(ADR D1+import-surface) → M11-2 → M11-3 → M11-4 → M11-5 → M11-6 → M11-7 → M11 CLOSE
         │
         v
        M12+: 2. external distribution (NEW ADR, owner) → 3. re-binding → 4. core handoff (research)
         │
         v
        SEALING CEREMONY (owner-physical: key K)   [prep P1/P2/P3 built long before]
```

SECURITY boundaries flagged across the plan: M9C-2c (first memory-derived
export), M10B-1 (trust-table refactor), M10C-1/R1 (verifier scope), M11-2a
(grant vocabulary), M11-4/-5 (session handles / secret splice), M12-item-2
(network intake — largest attack-surface expansion, ADR-gated), and the
ceremony itself (trust-root replacement).

OWNER-PHYSICAL blockers, complete list: **the sealing ceremony (key K) — and
nothing else.** (Owner-run live smokes and OD-1's Anthropic key are routine
operational asks, not structural blockers.)
